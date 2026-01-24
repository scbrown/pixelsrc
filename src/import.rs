//! PNG import functionality for converting images to Pixelsrc format.
//!
//! This module provides functionality to:
//! - Read PNG images and extract unique colors
//! - Quantize colors using median cut algorithm if too many colors
//! - Generate Pixelsrc JSONL output with palette and sprite definitions
//! - Detect shapes, symmetry, roles, and relationships when analysis is enabled

use image::{GenericImageView, Rgba};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::analyze::{
    detect_symmetry, infer_relationships_batch, infer_roles_batch, RegionData,
    RelationshipInference, RoleInference, RoleInferenceContext, Symmetric,
};
use crate::models::{RelationshipType, Role};

/// A structured region representation extracted from points.
#[derive(Debug, Clone)]
pub enum StructuredRegion {
    /// A simple rectangle [x, y, width, height]
    Rect([u32; 4]),
    /// A polygon defined by vertices
    Polygon(Vec<[i32; 2]>),
    /// A union of multiple shapes
    Union(Vec<StructuredRegion>),
    /// Raw points (fallback when no structure detected)
    Points(Vec<[u32; 2]>),
}

/// Options for PNG import.
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Enable role/relationship inference
    pub analyze: bool,
    /// Confidence threshold for inferences (0.0-1.0)
    pub confidence_threshold: f64,
    /// Generate token naming hints
    pub hints: bool,
    /// Extract structured regions (polygons, rects) instead of raw points
    pub extract_shapes: bool,
}

/// A naming hint for a token based on detected features.
#[derive(Debug, Clone)]
pub struct NamingHint {
    /// The current token name
    pub token: String,
    /// Suggested name based on detected features
    pub suggested_name: String,
    /// Reason for the suggestion
    pub reason: String,
}

/// Analysis results from import.
#[derive(Debug, Clone, Default)]
pub struct ImportAnalysis {
    /// Inferred roles for tokens (token -> role)
    pub roles: HashMap<String, Role>,
    /// Inferred relationships between tokens
    pub relationships: Vec<(String, RelationshipType, String)>,
    /// Detected symmetry
    pub symmetry: Option<Symmetric>,
    /// Token naming hints
    pub naming_hints: Vec<NamingHint>,
    /// Inferred z-order from spatial containment (token -> z-level)
    /// Higher z means the region should be rendered on top.
    pub z_order: HashMap<String, i32>,
}

/// Result of importing a PNG image.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// The generated sprite name.
    pub name: String,
    /// Width of the sprite in pixels.
    pub width: u32,
    /// Height of the sprite in pixels.
    pub height: u32,
    /// Color palette mapping tokens to hex colors.
    pub palette: HashMap<String, String>,
    /// Grid rows with token sequences (legacy format).
    pub grid: Vec<String>,
    /// Region definitions for each token (v2 format) - raw points.
    pub regions: HashMap<String, Vec<[u32; 2]>>,
    /// Structured region definitions (polygons, rects, unions).
    pub structured_regions: Option<HashMap<String, StructuredRegion>>,
    /// Analysis results (if analysis was enabled).
    pub analysis: Option<ImportAnalysis>,
}

impl ImportResult {
    /// Serialize to legacy JSONL format (palette line + sprite line with grid).
    pub fn to_jsonl(&self) -> String {
        let palette_json = serde_json::json!({
            "type": "palette",
            "name": format!("{}_palette", self.name),
            "colors": self.palette
        });

        let sprite_json = serde_json::json!({
            "type": "sprite",
            "name": self.name,
            "size": [self.width, self.height],
            "palette": format!("{}_palette", self.name),
            "grid": self.grid
        });

        format!("{}\n{}", palette_json, sprite_json)
    }

    /// Serialize to structured JSONL format (v2 with regions, roles, relationships).
    pub fn to_structured_jsonl(&self) -> String {
        let mut palette_obj = serde_json::json!({
            "type": "palette",
            "name": format!("{}_palette", self.name),
            "colors": self.palette
        });

        // Add roles if analysis was performed
        if let Some(ref analysis) = self.analysis {
            if !analysis.roles.is_empty() {
                let roles: HashMap<String, String> = analysis
                    .roles
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();
                palette_obj["roles"] = serde_json::json!(roles);
            }

            // Add relationships
            if !analysis.relationships.is_empty() {
                let relationships: HashMap<String, serde_json::Value> = analysis
                    .relationships
                    .iter()
                    .map(|(source, rel_type, target)| {
                        let rel_str = match rel_type {
                            RelationshipType::DerivesFrom => "derives-from",
                            RelationshipType::ContainedWithin => "contained-within",
                            RelationshipType::AdjacentTo => "adjacent-to",
                            RelationshipType::PairedWith => "paired-with",
                        };
                        (
                            source.clone(),
                            serde_json::json!({
                                "type": rel_str,
                                "target": target
                            }),
                        )
                    })
                    .collect();
                palette_obj["relationships"] = serde_json::json!(relationships);
            }
        }

        // Build regions object - use structured regions if available, adding z-order if present
        let z_order = self.analysis.as_ref().map(|a| &a.z_order);
        let regions: HashMap<String, serde_json::Value> = if let Some(ref structured) = self.structured_regions {
            structured
                .iter()
                .map(|(token, region)| {
                    let mut region_json = region.to_json();
                    // Add z-order if available
                    if let Some(z_map) = z_order {
                        if let Some(&z) = z_map.get(token) {
                            if let serde_json::Value::Object(ref mut obj) = region_json {
                                obj.insert("z".to_string(), serde_json::json!(z));
                            }
                        }
                    }
                    (token.clone(), region_json)
                })
                .collect()
        } else {
            self.regions
                .iter()
                .map(|(token, points)| {
                    let mut region_json = serde_json::json!({ "points": points });
                    // Add z-order if available
                    if let Some(z_map) = z_order {
                        if let Some(&z) = z_map.get(token) {
                            if let serde_json::Value::Object(ref mut obj) = region_json {
                                obj.insert("z".to_string(), serde_json::json!(z));
                            }
                        }
                    }
                    (token.clone(), region_json)
                })
                .collect()
        };

        let mut sprite_obj = serde_json::json!({
            "type": "sprite",
            "name": self.name,
            "size": [self.width, self.height],
            "palette": format!("{}_palette", self.name),
            "regions": regions
        });

        // Add symmetry if detected
        if let Some(ref analysis) = self.analysis {
            if let Some(ref symmetry) = analysis.symmetry {
                let sym_str = match symmetry {
                    Symmetric::X => "x",
                    Symmetric::Y => "y",
                    Symmetric::XY => "both",
                };
                // Note: symmetry could be added as metadata or hint
                sprite_obj["_symmetry"] = serde_json::json!(sym_str);
            }
        }

        format!("{}\n{}", palette_obj, sprite_obj)
    }
}

/// Extract structured regions from point arrays.
///
/// This converts raw point data into higher-level primitives:
/// - Rectangles for rectangular regions
/// - Polygons for irregular but contiguous regions
/// - Unions for multiple disconnected components
pub fn extract_structured_regions(points: &[[u32; 2]], width: u32, height: u32) -> StructuredRegion {
    if points.is_empty() {
        return StructuredRegion::Points(vec![]);
    }

    // Convert to HashSet for efficient lookups
    let point_set: HashSet<(u32, u32)> = points.iter().map(|p| (p[0], p[1])).collect();

    // Find connected components using flood fill
    let components = find_connected_components(&point_set);

    if components.is_empty() {
        return StructuredRegion::Points(points.to_vec());
    }

    // Convert each component to a structured region
    let mut structured: Vec<StructuredRegion> = Vec::new();

    for component in components {
        // Small components (< 16 pixels) - just use points for simplicity
        if component.len() < 16 {
            let pts: Vec<[u32; 2]> = component.into_iter().map(|(x, y)| [x, y]).collect();
            structured.push(StructuredRegion::Points(pts));
            continue;
        }

        // Check if it's a rectangle (only use rects, not polygons, for pixel-perfect accuracy)
        if let Some(rect) = try_extract_rect(&component) {
            structured.push(StructuredRegion::Rect(rect));
            continue;
        }

        // Fall back to points for non-rectangular shapes
        let pts: Vec<[u32; 2]> = component.into_iter().map(|(x, y)| [x, y]).collect();
        structured.push(StructuredRegion::Points(pts));
    }

    // Return single region or union
    if structured.len() == 1 {
        structured.pop().unwrap()
    } else {
        StructuredRegion::Union(structured)
    }
}

/// Find connected components in a set of points using 4-connectivity.
fn find_connected_components(points: &HashSet<(u32, u32)>) -> Vec<HashSet<(u32, u32)>> {
    let mut remaining: HashSet<(u32, u32)> = points.clone();
    let mut components = Vec::new();

    while !remaining.is_empty() {
        let start = *remaining.iter().next().unwrap();
        let mut component = HashSet::new();
        let mut queue = vec![start];

        while let Some(p) = queue.pop() {
            if remaining.remove(&p) {
                component.insert(p);

                // Check 4-connected neighbors
                let (x, y) = p;
                for (dx, dy) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                    let nx = (x as i32 + dx) as u32;
                    let ny = (y as i32 + dy) as u32;
                    if remaining.contains(&(nx, ny)) {
                        queue.push((nx, ny));
                    }
                }
            }
        }

        if !component.is_empty() {
            components.push(component);
        }
    }

    components
}

/// Try to extract a rectangle from a component.
/// Returns Some([x, y, width, height]) if the component is rectangular.
fn try_extract_rect(component: &HashSet<(u32, u32)>) -> Option<[u32; 4]> {
    if component.is_empty() {
        return None;
    }

    let min_x = component.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = component.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = component.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = component.iter().map(|(_, y)| *y).max().unwrap();

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let expected_size = (width * height) as usize;

    // Check if all pixels in the bounding box are present
    if component.len() == expected_size {
        Some([min_x, min_y, width, height])
    } else {
        None
    }
}

/// Extract a polygon boundary from a component using edge tracing.
fn extract_polygon_boundary(component: &HashSet<(u32, u32)>) -> Option<Vec<[i32; 2]>> {
    if component.len() < 3 {
        return None;
    }

    // Find bounding box
    let min_x = component.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = component.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = component.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = component.iter().map(|(_, y)| *y).max().unwrap();

    // Group points by y coordinate to find left and right edges
    let mut by_y: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(x, y) in component {
        by_y.entry(y).or_default().push(x);
    }

    // Build left and right edges
    let mut left_edge: Vec<[i32; 2]> = Vec::new();
    let mut right_edge: Vec<[i32; 2]> = Vec::new();

    for y in min_y..=max_y {
        if let Some(xs) = by_y.get(&y) {
            let min_x = *xs.iter().min().unwrap();
            let max_x = *xs.iter().max().unwrap();
            left_edge.push([min_x as i32, y as i32]);
            right_edge.push([max_x as i32, y as i32]);
        }
    }

    // Simplify edges using Douglas-Peucker algorithm
    let left_simple = douglas_peucker(&left_edge, 1.5);
    let right_simple = douglas_peucker(&right_edge, 1.5);

    // Combine into closed polygon (left edge top-to-bottom, right edge bottom-to-top)
    let mut polygon = left_simple;
    polygon.extend(right_simple.into_iter().rev());

    // Remove duplicate consecutive points
    polygon.dedup();

    // Limit polygon size for sanity
    if polygon.len() > 50 {
        // Subsample
        let step = polygon.len() / 30;
        polygon = polygon.into_iter().step_by(step.max(1)).collect();
    }

    if polygon.len() >= 3 {
        Some(polygon)
    } else {
        None
    }
}

/// Douglas-Peucker line simplification algorithm.
fn douglas_peucker(points: &[[i32; 2]], epsilon: f64) -> Vec<[i32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Find the point with maximum distance from the line
    let start = points[0];
    let end = points[points.len() - 1];

    let mut max_dist = 0.0f64;
    let mut max_idx = 0;

    for (i, point) in points.iter().enumerate().skip(1).take(points.len() - 2) {
        let dist = perpendicular_distance(point, &start, &end);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        // Recursively simplify
        let mut left = douglas_peucker(&points[..=max_idx], epsilon);
        let right = douglas_peucker(&points[max_idx..], epsilon);

        left.pop(); // Remove duplicate point
        left.extend(right);
        left
    } else {
        // Return just endpoints
        vec![start, end]
    }
}

/// Calculate perpendicular distance from a point to a line.
fn perpendicular_distance(point: &[i32; 2], line_start: &[i32; 2], line_end: &[i32; 2]) -> f64 {
    let dx = line_end[0] - line_start[0];
    let dy = line_end[1] - line_start[1];

    let len_sq = (dx * dx + dy * dy) as f64;
    if len_sq == 0.0 {
        // Line is a point
        let px = point[0] - line_start[0];
        let py = point[1] - line_start[1];
        return ((px * px + py * py) as f64).sqrt();
    }

    // Project point onto line
    let t = ((point[0] - line_start[0]) * dx + (point[1] - line_start[1]) * dy) as f64 / len_sq;
    let t = t.clamp(0.0, 1.0);

    let proj_x = line_start[0] as f64 + t * dx as f64;
    let proj_y = line_start[1] as f64 + t * dy as f64;

    let dist_x = point[0] as f64 - proj_x;
    let dist_y = point[1] as f64 - proj_y;

    (dist_x * dist_x + dist_y * dist_y).sqrt()
}

/// Rasterize a polygon to get the set of pixels it covers.
/// Uses scanline algorithm to fill the polygon.
fn rasterize_polygon(polygon: &[[i32; 2]]) -> HashSet<(u32, u32)> {
    let mut pixels = HashSet::new();

    if polygon.len() < 3 {
        return pixels;
    }

    // Find bounding box
    let min_y = polygon.iter().map(|p| p[1]).min().unwrap();
    let max_y = polygon.iter().map(|p| p[1]).max().unwrap();

    // Scanline fill
    for y in min_y..=max_y {
        let mut intersections: Vec<i32> = Vec::new();

        // Find intersections with polygon edges
        for i in 0..polygon.len() {
            let p1 = polygon[i];
            let p2 = polygon[(i + 1) % polygon.len()];

            // Check if edge crosses this scanline
            if (p1[1] <= y && p2[1] > y) || (p2[1] <= y && p1[1] > y) {
                // Calculate x intersection
                let dy = p2[1] - p1[1];
                if dy != 0 {
                    let x = p1[0] + (y - p1[1]) * (p2[0] - p1[0]) / dy;
                    intersections.push(x);
                }
            }
        }

        // Sort intersections and fill between pairs
        intersections.sort();
        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                for x in chunk[0]..=chunk[1] {
                    if x >= 0 && y >= 0 {
                        pixels.insert((x as u32, y as u32));
                    }
                }
            }
        }
    }

    pixels
}

/// Calculate coverage ratio between original component and polygon pixels.
/// Returns value between 0.0 and 1.0, where 1.0 means perfect match.
fn calculate_coverage(original: &HashSet<(u32, u32)>, polygon: &HashSet<(u32, u32)>) -> f64 {
    if original.is_empty() || polygon.is_empty() {
        return 0.0;
    }

    // Calculate intersection (pixels in both)
    let intersection: HashSet<_> = original.intersection(polygon).collect();

    // Calculate union (pixels in either)
    let union: HashSet<_> = original.union(polygon).collect();

    // Jaccard similarity: intersection / union
    intersection.len() as f64 / union.len() as f64
}

impl StructuredRegion {
    /// Convert to JSON value for serialization.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            StructuredRegion::Rect(r) => serde_json::json!({ "rect": r }),
            StructuredRegion::Polygon(p) => serde_json::json!({ "polygon": p }),
            StructuredRegion::Union(regions) => {
                let shapes: Vec<serde_json::Value> = regions.iter().map(|r| r.to_json()).collect();
                serde_json::json!({ "union": shapes })
            }
            StructuredRegion::Points(p) => serde_json::json!({ "points": p }),
        }
    }
}

/// A color represented as RGBA values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn from_rgba(rgba: Rgba<u8>) -> Self {
        Self { r: rgba[0], g: rgba[1], b: rgba[2], a: rgba[3] }
    }

    fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

/// A box of colors for median cut algorithm.
#[derive(Debug, Clone)]
struct ColorBox {
    colors: Vec<(Color, u32)>, // Color and count
}

impl ColorBox {
    fn new(colors: Vec<(Color, u32)>) -> Self {
        Self { colors }
    }

    /// Find which channel has the largest range.
    fn widest_channel(&self) -> Channel {
        let (mut min_r, mut max_r) = (255u8, 0u8);
        let (mut min_g, mut max_g) = (255u8, 0u8);
        let (mut min_b, mut max_b) = (255u8, 0u8);

        for (color, _) in &self.colors {
            min_r = min_r.min(color.r);
            max_r = max_r.max(color.r);
            min_g = min_g.min(color.g);
            max_g = max_g.max(color.g);
            min_b = min_b.min(color.b);
            max_b = max_b.max(color.b);
        }

        let range_r = max_r.saturating_sub(min_r);
        let range_g = max_g.saturating_sub(min_g);
        let range_b = max_b.saturating_sub(min_b);

        if range_r >= range_g && range_r >= range_b {
            Channel::Red
        } else if range_g >= range_b {
            Channel::Green
        } else {
            Channel::Blue
        }
    }

    /// Split the box into two along the widest channel.
    fn split(mut self) -> (ColorBox, ColorBox) {
        let channel = self.widest_channel();

        // Sort by the widest channel
        self.colors.sort_by_key(|(color, _)| match channel {
            Channel::Red => color.r,
            Channel::Green => color.g,
            Channel::Blue => color.b,
        });

        // Find median by pixel count
        let total: u32 = self.colors.iter().map(|(_, count)| count).sum();
        let mut running = 0u32;
        let mut split_idx = self.colors.len() / 2;

        for (i, (_, count)) in self.colors.iter().enumerate() {
            running += count;
            if running >= total / 2 {
                split_idx = (i + 1).min(self.colors.len() - 1);
                break;
            }
        }

        // Ensure we don't create empty boxes
        split_idx = split_idx.max(1).min(self.colors.len() - 1);

        let right = self.colors.split_off(split_idx);
        (ColorBox::new(self.colors), ColorBox::new(right))
    }

    /// Get the average color of this box (weighted by pixel count).
    fn average_color(&self) -> Color {
        let total: u64 = self.colors.iter().map(|(_, count)| *count as u64).sum();
        if total == 0 {
            return Color { r: 0, g: 0, b: 0, a: 255 };
        }

        let r: u64 = self.colors.iter().map(|(c, count)| c.r as u64 * *count as u64).sum();
        let g: u64 = self.colors.iter().map(|(c, count)| c.g as u64 * *count as u64).sum();
        let b: u64 = self.colors.iter().map(|(c, count)| c.b as u64 * *count as u64).sum();
        let a: u64 = self.colors.iter().map(|(c, count)| c.a as u64 * *count as u64).sum();

        Color {
            r: (r / total) as u8,
            g: (g / total) as u8,
            b: (b / total) as u8,
            a: (a / total) as u8,
        }
    }

    /// Total pixel count in this box.
    fn pixel_count(&self) -> u32 {
        self.colors.iter().map(|(_, count)| count).sum()
    }
}

#[derive(Debug, Clone, Copy)]
enum Channel {
    Red,
    Green,
    Blue,
}

/// LAB color representation for perceptual color quantization.
#[derive(Debug, Clone, Copy)]
struct LabColor {
    l: f64, // Lightness: 0-100
    a: f64, // Green-Red axis: -128 to 127
    b: f64, // Blue-Yellow axis: -128 to 127
}

impl LabColor {
    /// Convert RGB color to LAB color space.
    /// Uses D65 illuminant standard.
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        // Step 1: RGB to linear RGB (sRGB gamma correction)
        let r_lin = srgb_to_linear(r as f64 / 255.0);
        let g_lin = srgb_to_linear(g as f64 / 255.0);
        let b_lin = srgb_to_linear(b as f64 / 255.0);

        // Step 2: Linear RGB to XYZ (sRGB to XYZ matrix, D65 illuminant)
        let x = r_lin * 0.4124564 + g_lin * 0.3575761 + b_lin * 0.1804375;
        let y = r_lin * 0.2126729 + g_lin * 0.7151522 + b_lin * 0.0721750;
        let z = r_lin * 0.0193339 + g_lin * 0.1191920 + b_lin * 0.9503041;

        // Step 3: XYZ to LAB (using D65 reference white)
        // D65 reference white point
        let x_n = 0.95047;
        let y_n = 1.00000;
        let z_n = 1.08883;

        let fx = lab_f(x / x_n);
        let fy = lab_f(y / y_n);
        let fz = lab_f(z / z_n);

        let l = 116.0 * fy - 16.0;
        let a = 500.0 * (fx - fy);
        let b = 200.0 * (fy - fz);

        Self { l, a, b }
    }

    /// Calculate perceptual distance to another LAB color (CIE76 Delta E).
    fn distance(&self, other: &LabColor) -> f64 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }
}

/// sRGB gamma expansion (inverse companding).
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// LAB f function for XYZ to LAB conversion.
fn lab_f(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta.powi(3) {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

/// LAB channel for perceptual median cut.
#[derive(Debug, Clone, Copy)]
enum LabChannel {
    L,
    A,
    B,
}

/// A box of colors in LAB space for perceptual median cut.
#[derive(Debug, Clone)]
struct LabColorBox {
    colors: Vec<(Color, LabColor, u32)>, // Original color, LAB color, count
}

impl LabColorBox {
    fn new(colors: Vec<(Color, LabColor, u32)>) -> Self {
        Self { colors }
    }

    /// Find which LAB channel has the largest range.
    fn widest_channel(&self) -> LabChannel {
        let (mut min_l, mut max_l) = (f64::MAX, f64::MIN);
        let (mut min_a, mut max_a) = (f64::MAX, f64::MIN);
        let (mut min_b, mut max_b) = (f64::MAX, f64::MIN);

        for (_, lab, _) in &self.colors {
            min_l = min_l.min(lab.l);
            max_l = max_l.max(lab.l);
            min_a = min_a.min(lab.a);
            max_a = max_a.max(lab.a);
            min_b = min_b.min(lab.b);
            max_b = max_b.max(lab.b);
        }

        let range_l = max_l - min_l;
        let range_a = max_a - min_a;
        let range_b = max_b - min_b;

        if range_l >= range_a && range_l >= range_b {
            LabChannel::L
        } else if range_a >= range_b {
            LabChannel::A
        } else {
            LabChannel::B
        }
    }

    /// Split the box into two along the widest LAB channel.
    fn split(mut self) -> (LabColorBox, LabColorBox) {
        let channel = self.widest_channel();

        // Sort by the widest channel
        self.colors.sort_by(|(_, lab1, _), (_, lab2, _)| {
            let v1 = match channel {
                LabChannel::L => lab1.l,
                LabChannel::A => lab1.a,
                LabChannel::B => lab1.b,
            };
            let v2 = match channel {
                LabChannel::L => lab2.l,
                LabChannel::A => lab2.a,
                LabChannel::B => lab2.b,
            };
            v1.partial_cmp(&v2).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Find median by pixel count
        let total: u32 = self.colors.iter().map(|(_, _, count)| count).sum();
        let mut running = 0u32;
        let mut split_idx = self.colors.len() / 2;

        for (i, (_, _, count)) in self.colors.iter().enumerate() {
            running += count;
            if running >= total / 2 {
                split_idx = (i + 1).min(self.colors.len() - 1);
                break;
            }
        }

        // Ensure we don't create empty boxes
        split_idx = split_idx.max(1).min(self.colors.len() - 1);

        let right = self.colors.split_off(split_idx);
        (LabColorBox::new(self.colors), LabColorBox::new(right))
    }

    /// Get the average color of this box (weighted by pixel count).
    /// Returns the original RGB color closest to the average LAB.
    fn average_color(&self) -> Color {
        let total: u64 = self.colors.iter().map(|(_, _, count)| *count as u64).sum();
        if total == 0 {
            return Color { r: 0, g: 0, b: 0, a: 255 };
        }

        // Calculate weighted average in LAB space
        let l: f64 = self.colors.iter().map(|(_, lab, count)| lab.l * *count as f64).sum::<f64>() / total as f64;
        let a: f64 = self.colors.iter().map(|(_, lab, count)| lab.a * *count as f64).sum::<f64>() / total as f64;
        let b: f64 = self.colors.iter().map(|(_, lab, count)| lab.b * *count as f64).sum::<f64>() / total as f64;
        let avg_lab = LabColor { l, a, b };

        // Find the original color closest to this average
        // (We return an actual palette color rather than synthesizing one)
        self.colors
            .iter()
            .min_by(|(_, lab1, _), (_, lab2, _)| {
                let d1 = avg_lab.distance(lab1);
                let d2 = avg_lab.distance(lab2);
                d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(c, _, _)| *c)
            .unwrap_or(Color { r: 0, g: 0, b: 0, a: 255 })
    }

    /// Total pixel count in this box.
    fn pixel_count(&self) -> u32 {
        self.colors.iter().map(|(_, _, count)| count).sum()
    }
}

/// Quantize colors using median cut algorithm in perceptual LAB color space.
/// This produces better results for skin tones, gradients, and similar colors.
fn median_cut_quantize_lab(colors: HashMap<Color, u32>, max_colors: usize) -> Vec<Color> {
    if colors.len() <= max_colors {
        return colors.into_keys().collect();
    }

    // Separate transparent colors from opaque colors
    let mut transparent: Option<Color> = None;
    let mut opaque_colors: Vec<(Color, LabColor, u32)> = Vec::new();

    for (color, count) in colors {
        if color.is_transparent() {
            transparent = Some(color);
        } else {
            let lab = LabColor::from_rgb(color.r, color.g, color.b);
            opaque_colors.push((color, lab, count));
        }
    }

    // Adjust max_colors if we have a transparent color
    let effective_max =
        if transparent.is_some() { max_colors.saturating_sub(1) } else { max_colors };

    if opaque_colors.len() <= effective_max {
        let mut result: Vec<Color> = opaque_colors.into_iter().map(|(c, _, _)| c).collect();
        if let Some(t) = transparent {
            result.push(t);
        }
        return result;
    }

    // Initial box with all opaque colors
    let mut boxes = vec![LabColorBox::new(opaque_colors)];

    // Split until we have enough boxes
    while boxes.len() < effective_max {
        // Find the box with the most pixels to split
        let (idx, _) = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.colors.len() > 1)
            .max_by_key(|(_, b)| b.pixel_count())
            .unwrap_or((0, &boxes[0]));

        if boxes[idx].colors.len() <= 1 {
            break;
        }

        let box_to_split = boxes.remove(idx);
        let (left, right) = box_to_split.split();
        boxes.push(left);
        boxes.push(right);
    }

    // Get average color from each box
    let mut result: Vec<Color> = boxes.into_iter().map(|b| b.average_color()).collect();

    // Add transparent color if present
    if let Some(t) = transparent {
        result.push(t);
    }

    result
}

/// Quantize colors using median cut algorithm (legacy RGB version).
fn median_cut_quantize(colors: HashMap<Color, u32>, max_colors: usize) -> Vec<Color> {
    if colors.len() <= max_colors {
        return colors.into_keys().collect();
    }

    // Separate transparent colors from opaque colors
    let mut transparent: Option<Color> = None;
    let mut opaque_colors: Vec<(Color, u32)> = Vec::new();

    for (color, count) in colors {
        if color.is_transparent() {
            transparent = Some(color);
        } else {
            opaque_colors.push((color, count));
        }
    }

    // Adjust max_colors if we have a transparent color
    let effective_max =
        if transparent.is_some() { max_colors.saturating_sub(1) } else { max_colors };

    if opaque_colors.len() <= effective_max {
        let mut result: Vec<Color> = opaque_colors.into_iter().map(|(c, _)| c).collect();
        if let Some(t) = transparent {
            result.push(t);
        }
        return result;
    }

    // Initial box with all opaque colors
    let mut boxes = vec![ColorBox::new(opaque_colors)];

    // Split until we have enough boxes
    while boxes.len() < effective_max {
        // Find the box with the most pixels to split
        let (idx, _) = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.colors.len() > 1)
            .max_by_key(|(_, b)| b.pixel_count())
            .unwrap_or((0, &boxes[0]));

        if boxes[idx].colors.len() <= 1 {
            break;
        }

        let box_to_split = boxes.remove(idx);
        let (left, right) = box_to_split.split();
        boxes.push(left);
        boxes.push(right);
    }

    // Get average color from each box
    let mut result: Vec<Color> = boxes.into_iter().map(|b| b.average_color()).collect();

    // Add transparent color if present
    if let Some(t) = transparent {
        result.push(t);
    }

    result
}

/// Find the closest color in the palette to a given color using LAB perceptual distance.
fn find_closest_color(color: Color, palette: &[Color]) -> usize {
    // Handle transparent colors specially - match by alpha
    if color.is_transparent() {
        return palette.iter().position(|p| p.is_transparent()).unwrap_or(0);
    }

    let color_lab = LabColor::from_rgb(color.r, color.g, color.b);

    palette
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.is_transparent()) // Skip transparent when matching opaque
        .min_by(|(_, p1), (_, p2)| {
            let lab1 = LabColor::from_rgb(p1.r, p1.g, p1.b);
            let lab2 = LabColor::from_rgb(p2.r, p2.g, p2.b);
            let d1 = color_lab.distance(&lab1);
            let d2 = color_lab.distance(&lab2);
            d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Import a PNG file and convert it to Pixelsrc format (legacy, no analysis).
pub fn import_png<P: AsRef<Path>>(
    path: P,
    name: &str,
    max_colors: usize,
) -> Result<ImportResult, String> {
    let options = ImportOptions::default();
    import_png_with_options(path, name, max_colors, &options)
}

/// Import a PNG file with analysis options.
pub fn import_png_with_options<P: AsRef<Path>>(
    path: P,
    name: &str,
    max_colors: usize,
    options: &ImportOptions,
) -> Result<ImportResult, String> {
    let img = image::open(path.as_ref()).map_err(|e| format!("Failed to open image: {}", e))?;

    let (width, height) = img.dimensions();

    // Extract all unique colors with their pixel counts
    let mut color_counts: HashMap<Color, u32> = HashMap::new();
    for (_, _, pixel) in img.pixels() {
        let color = Color::from_rgba(pixel);
        *color_counts.entry(color).or_insert(0) += 1;
    }

    // Quantize if needed using perceptual LAB color space
    let palette_colors = median_cut_quantize_lab(color_counts.clone(), max_colors);

    // Build color to index mapping
    let original_colors: Vec<Color> = color_counts.keys().cloned().collect();

    // Map original colors to palette colors
    let mut color_to_palette_idx: HashMap<Color, usize> = HashMap::new();
    for orig_color in &original_colors {
        let idx = find_closest_color(*orig_color, &palette_colors);
        color_to_palette_idx.insert(*orig_color, idx);
    }

    // Generate token names
    // Find transparent color index for special {_} token
    let transparent_idx = palette_colors.iter().position(|c| c.is_transparent());

    let mut palette: HashMap<String, String> = HashMap::new();
    let mut idx_to_token: HashMap<usize, String> = HashMap::new();
    let mut idx_to_color: HashMap<usize, Color> = HashMap::new();

    let mut color_num = 1;
    for (idx, color) in palette_colors.iter().enumerate() {
        let token = if Some(idx) == transparent_idx {
            "{_}".to_string()
        } else {
            let t = format!("{{c{}}}", color_num);
            color_num += 1;
            t
        };
        palette.insert(token.clone(), color.to_hex());
        idx_to_token.insert(idx, token);
        idx_to_color.insert(idx, *color);
    }

    // Build grid and regions simultaneously
    let mut grid: Vec<String> = Vec::with_capacity(height as usize);
    let mut regions: HashMap<String, Vec<[u32; 2]>> = HashMap::new();
    let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();

    for y in 0..height {
        let mut row = String::new();
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let color = Color::from_rgba(pixel);
            let palette_idx = color_to_palette_idx[&color];
            let token = &idx_to_token[&palette_idx];
            row.push_str(token);

            // Add to regions
            regions.entry(token.clone()).or_default().push([x, y]);
            token_pixels
                .entry(token.clone())
                .or_default()
                .insert((x as i32, y as i32));
        }
        grid.push(row);
    }

    // Perform analysis if requested
    let analysis = if options.analyze {
        Some(perform_analysis(
            width,
            height,
            &token_pixels,
            &idx_to_token,
            &idx_to_color,
            options,
        ))
    } else {
        None
    };

    // Extract structured regions if requested
    let structured_regions = if options.extract_shapes {
        Some(
            regions
                .iter()
                .map(|(token, points)| {
                    (token.clone(), extract_structured_regions(points, width, height))
                })
                .collect(),
        )
    } else {
        None
    };

    Ok(ImportResult {
        name: name.to_string(),
        width,
        height,
        palette,
        grid,
        regions,
        structured_regions,
        analysis,
    })
}

/// Infer z-order values from spatial containment relationships.
///
/// If region A is contained within region B, A should be rendered on top (higher z).
/// This computes z-levels by finding how deeply nested each region is.
///
/// Algorithm:
/// 1. Build a containment graph from ContainedWithin relationships
/// 2. For each region, z = 1 + max(z of all containers)
/// 3. Regions not contained get z = 0
fn infer_z_order(
    tokens: &[String],
    relationships: &[(String, RelationshipType, String)],
) -> HashMap<String, i32> {
    // Build containment graph: token -> set of tokens it's contained in
    let mut contained_in: HashMap<String, Vec<String>> = HashMap::new();

    for (source, rel_type, target) in relationships {
        if matches!(rel_type, RelationshipType::ContainedWithin) {
            // source is contained within target
            contained_in
                .entry(source.clone())
                .or_default()
                .push(target.clone());
        }
    }

    // Compute z-order using memoization
    let mut z_order: HashMap<String, i32> = HashMap::new();
    let mut computing: HashSet<String> = HashSet::new(); // Cycle detection

    fn compute_z(
        token: &str,
        contained_in: &HashMap<String, Vec<String>>,
        z_order: &mut HashMap<String, i32>,
        computing: &mut HashSet<String>,
    ) -> i32 {
        // Already computed
        if let Some(&z) = z_order.get(token) {
            return z;
        }

        // Cycle detection - return 0 if we're in a cycle
        if computing.contains(token) {
            return 0;
        }
        computing.insert(token.to_string());

        // Get containers
        let z = if let Some(containers) = contained_in.get(token) {
            // z = 1 + max(z of containers)
            let max_container_z = containers
                .iter()
                .map(|c| compute_z(c, contained_in, z_order, computing))
                .max()
                .unwrap_or(0);
            max_container_z + 1
        } else {
            // Not contained in anything - base level
            0
        };

        computing.remove(token);
        z_order.insert(token.to_string(), z);
        z
    }

    // Compute z for all tokens
    for token in tokens {
        compute_z(token, &contained_in, &mut z_order, &mut computing);
    }

    z_order
}

/// Perform analysis on imported regions.
fn perform_analysis(
    width: u32,
    height: u32,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    idx_to_token: &HashMap<usize, String>,
    idx_to_color: &HashMap<usize, Color>,
    options: &ImportOptions,
) -> ImportAnalysis {
    let mut analysis = ImportAnalysis::default();

    // Build token to color mapping
    let token_to_color: HashMap<String, [u8; 4]> = idx_to_token
        .iter()
        .filter_map(|(idx, token)| {
            idx_to_color.get(idx).map(|c| (token.clone(), [c.r, c.g, c.b, c.a]))
        })
        .collect();

    // Detect symmetry using raw pixel data
    // For symmetry detection, we need the raw pixel bytes
    // We'll create a simplified version based on the grid
    let bpp = 4;
    let mut pixel_data = vec![0u8; (width * height * bpp as u32) as usize];
    for (token, pixels) in token_pixels {
        if let Some(color) = token_to_color.get(token) {
            for &(x, y) in pixels {
                let idx = ((y as u32 * width + x as u32) * bpp as u32) as usize;
                if idx + 3 < pixel_data.len() {
                    pixel_data[idx] = color[0];
                    pixel_data[idx + 1] = color[1];
                    pixel_data[idx + 2] = color[2];
                    pixel_data[idx + 3] = color[3];
                }
            }
        }
    }

    // Detect symmetry
    analysis.symmetry = detect_symmetry(&pixel_data, width, height);

    // Prepare data for role inference
    let ctx = RoleInferenceContext::new(width, height);
    let role_input: HashMap<String, (HashSet<(i32, i32)>, Option<[u8; 4]>)> = token_pixels
        .iter()
        .map(|(token, pixels)| {
            let color = token_to_color.get(token).copied();
            (token.clone(), (pixels.clone(), color))
        })
        .collect();

    // Infer roles
    let (role_inferences, _warnings) = infer_roles_batch(&role_input, &ctx);
    for (token, inference) in role_inferences {
        if inference.confidence >= options.confidence_threshold {
            analysis.roles.insert(token, inference.role);
        }
    }

    // Prepare data for relationship inference
    let region_data: Vec<RegionData> = token_pixels
        .iter()
        .map(|(token, pixels)| {
            let color = token_to_color.get(token).copied().unwrap_or([0, 0, 0, 255]);
            RegionData { name: token.clone(), pixels: pixels.clone(), color }
        })
        .collect();

    // Infer relationships
    let rel_inferences = infer_relationships_batch(&region_data, width);
    for rel in rel_inferences {
        if rel.confidence >= options.confidence_threshold {
            analysis.relationships.push((rel.source, rel.relationship_type, rel.target));
        }
    }

    // Infer z-order from containment relationships
    let tokens: Vec<String> = token_pixels.keys().cloned().collect();
    analysis.z_order = infer_z_order(&tokens, &analysis.relationships);

    // Generate naming hints if requested
    if options.hints {
        analysis.naming_hints = generate_naming_hints(&analysis.roles, token_pixels);
    }

    analysis
}

/// Generate token naming suggestions based on detected features.
fn generate_naming_hints(
    roles: &HashMap<String, Role>,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
) -> Vec<NamingHint> {
    let mut hints = Vec::new();

    for (token, role) in roles {
        // Skip transparent token
        if token == "{_}" {
            continue;
        }

        let suggested = match role {
            Role::Boundary => format!("{{outline}}"),
            Role::Anchor => {
                // Small features might be eyes, buttons, etc.
                let size = token_pixels.get(token).map(|p| p.len()).unwrap_or(0);
                if size == 1 {
                    "{dot}".to_string()
                } else if size <= 4 {
                    "{eye}".to_string()
                } else {
                    "{marker}".to_string()
                }
            }
            Role::Fill => "{fill}".to_string(),
            Role::Shadow => "{shadow}".to_string(),
            Role::Highlight => "{highlight}".to_string(),
        };

        if token != &suggested {
            hints.push(NamingHint {
                token: token.clone(),
                suggested_name: suggested,
                reason: format!("Detected as {} role", role),
            });
        }
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_hex_opaque() {
        let color = Color { r: 255, g: 128, b: 0, a: 255 };
        assert_eq!(color.to_hex(), "#FF8000");
    }

    #[test]
    fn test_color_to_hex_transparent() {
        let color = Color { r: 255, g: 128, b: 0, a: 128 };
        assert_eq!(color.to_hex(), "#FF800080");
    }

    #[test]
    fn test_color_to_hex_fully_transparent() {
        let color = Color { r: 0, g: 0, b: 0, a: 0 };
        assert_eq!(color.to_hex(), "#00000000");
    }

    #[test]
    fn test_lab_color_conversion_black() {
        // Black should be L=0, a=0, b=0
        let lab = LabColor::from_rgb(0, 0, 0);
        assert!(lab.l < 1.0, "Black L should be ~0, got {}", lab.l);
        assert!(lab.a.abs() < 1.0, "Black a should be ~0, got {}", lab.a);
        assert!(lab.b.abs() < 1.0, "Black b should be ~0, got {}", lab.b);
    }

    #[test]
    fn test_lab_color_conversion_white() {
        // White should be L=100, a=0, b=0
        let lab = LabColor::from_rgb(255, 255, 255);
        assert!(lab.l > 99.0, "White L should be ~100, got {}", lab.l);
        assert!(lab.a.abs() < 1.0, "White a should be ~0, got {}", lab.a);
        assert!(lab.b.abs() < 1.0, "White b should be ~0, got {}", lab.b);
    }

    #[test]
    fn test_lab_color_conversion_red() {
        // Red should have high L, positive a
        let lab = LabColor::from_rgb(255, 0, 0);
        assert!(lab.l > 50.0, "Red L should be > 50, got {}", lab.l);
        assert!(lab.a > 50.0, "Red a should be positive, got {}", lab.a);
    }

    #[test]
    fn test_lab_distance() {
        let black = LabColor::from_rgb(0, 0, 0);
        let white = LabColor::from_rgb(255, 255, 255);
        let dark_gray = LabColor::from_rgb(30, 30, 30);

        // Distance from black to white should be large
        let bw_dist = black.distance(&white);
        assert!(bw_dist > 90.0, "Black-white distance should be large, got {}", bw_dist);

        // Distance from black to dark gray should be small
        let bg_dist = black.distance(&dark_gray);
        assert!(bg_dist < bw_dist, "Black-gray distance should be less than black-white");
    }

    #[test]
    fn test_lab_quantize_no_quantization_needed() {
        let mut colors = HashMap::new();
        colors.insert(Color { r: 255, g: 0, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 255, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 0, b: 255, a: 255 }, 10);

        let result = median_cut_quantize_lab(colors, 4);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_lab_quantize_reduces_colors() {
        let mut colors = HashMap::new();
        // Create more colors than max
        for i in 0..20 {
            colors.insert(Color { r: i * 10, g: i * 5, b: i * 2, a: 255 }, 1);
        }

        let result = median_cut_quantize_lab(colors, 4);
        assert!(result.len() <= 4);
    }

    #[test]
    fn test_lab_quantize_preserves_transparent() {
        let mut colors = HashMap::new();
        colors.insert(Color { r: 0, g: 0, b: 0, a: 0 }, 10); // Transparent
        colors.insert(Color { r: 255, g: 0, b: 0, a: 255 }, 10);
        colors.insert(Color { r: 0, g: 255, b: 0, a: 255 }, 10);

        let result = median_cut_quantize_lab(colors, 3);
        assert!(result.iter().any(|c| c.is_transparent()));
    }

    #[test]
    fn test_lab_skin_tone_grouping() {
        // Test that similar skin tones are grouped together in LAB space
        let skin_light = LabColor::from_rgb(255, 220, 185);  // Light skin
        let skin_medium = LabColor::from_rgb(210, 160, 120); // Medium skin
        let skin_dark = LabColor::from_rgb(140, 90, 60);     // Dark skin
        let pure_red = LabColor::from_rgb(255, 0, 0);        // Pure red

        // Skin tones should be closer to each other than to pure red
        let light_to_medium = skin_light.distance(&skin_medium);
        let light_to_red = skin_light.distance(&pure_red);

        assert!(light_to_medium < light_to_red,
            "Skin tones should be closer to each other than to pure red");
    }

    #[test]
    fn test_find_closest_color() {
        let palette =
            vec![Color { r: 0, g: 0, b: 0, a: 255 }, Color { r: 255, g: 255, b: 255, a: 255 }];

        let dark = Color { r: 30, g: 30, b: 30, a: 255 };
        let light = Color { r: 200, g: 200, b: 200, a: 255 };

        assert_eq!(find_closest_color(dark, &palette), 0);
        assert_eq!(find_closest_color(light, &palette), 1);
    }

    #[test]
    fn test_import_result_to_jsonl() {
        let mut palette = HashMap::new();
        palette.insert("{_}".to_string(), "#00000000".to_string());
        palette.insert("{c1}".to_string(), "#FF0000".to_string());

        let mut regions = HashMap::new();
        regions.insert("{c1}".to_string(), vec![[0, 0], [1, 1]]);
        regions.insert("{_}".to_string(), vec![[1, 0], [0, 1]]);

        let result = ImportResult {
            name: "test_sprite".to_string(),
            width: 2,
            height: 2,
            palette,
            grid: vec!["{c1}{_}".to_string(), "{_}{c1}".to_string()],
            regions,
            structured_regions: None,
            analysis: None,
        };

        let jsonl = result.to_jsonl();
        assert!(jsonl.contains("\"type\":\"palette\""));
        assert!(jsonl.contains("\"type\":\"sprite\""));
        assert!(jsonl.contains("test_sprite_palette"));
        assert!(jsonl.contains("test_sprite"));
    }

    #[test]
    fn test_infer_z_order_no_containment() {
        // No containment relationships - all tokens get z = 0
        let tokens = vec!["{a}".to_string(), "{b}".to_string(), "{c}".to_string()];
        let relationships = vec![];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{a}"), Some(&0));
        assert_eq!(z_order.get("{b}"), Some(&0));
        assert_eq!(z_order.get("{c}"), Some(&0));
    }

    #[test]
    fn test_infer_z_order_single_containment() {
        // {inner} is contained within {outer}
        // {inner} should have z = 1, {outer} should have z = 0
        let tokens = vec!["{inner}".to_string(), "{outer}".to_string()];
        let relationships = vec![(
            "{inner}".to_string(),
            RelationshipType::ContainedWithin,
            "{outer}".to_string(),
        )];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{inner}"), Some(&1));
        assert_eq!(z_order.get("{outer}"), Some(&0));
    }

    #[test]
    fn test_infer_z_order_nested_containment() {
        // {innermost} contained in {middle}, {middle} contained in {outer}
        // z-levels: innermost=2, middle=1, outer=0
        let tokens = vec![
            "{innermost}".to_string(),
            "{middle}".to_string(),
            "{outer}".to_string(),
        ];
        let relationships = vec![
            (
                "{innermost}".to_string(),
                RelationshipType::ContainedWithin,
                "{middle}".to_string(),
            ),
            (
                "{middle}".to_string(),
                RelationshipType::ContainedWithin,
                "{outer}".to_string(),
            ),
        ];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{innermost}"), Some(&2));
        assert_eq!(z_order.get("{middle}"), Some(&1));
        assert_eq!(z_order.get("{outer}"), Some(&0));
    }

    #[test]
    fn test_infer_z_order_multiple_containers() {
        // {inner} is contained in both {outer1} and {outer2}
        // {inner} z should be 1 (max of containers + 1)
        let tokens = vec![
            "{inner}".to_string(),
            "{outer1}".to_string(),
            "{outer2}".to_string(),
        ];
        let relationships = vec![
            (
                "{inner}".to_string(),
                RelationshipType::ContainedWithin,
                "{outer1}".to_string(),
            ),
            (
                "{inner}".to_string(),
                RelationshipType::ContainedWithin,
                "{outer2}".to_string(),
            ),
        ];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{inner}"), Some(&1));
        assert_eq!(z_order.get("{outer1}"), Some(&0));
        assert_eq!(z_order.get("{outer2}"), Some(&0));
    }

    #[test]
    fn test_infer_z_order_ignores_other_relationships() {
        // Only ContainedWithin should affect z-order
        let tokens = vec!["{a}".to_string(), "{b}".to_string()];
        let relationships = vec![
            (
                "{a}".to_string(),
                RelationshipType::AdjacentTo,
                "{b}".to_string(),
            ),
            (
                "{a}".to_string(),
                RelationshipType::DerivesFrom,
                "{b}".to_string(),
            ),
        ];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{a}"), Some(&0));
        assert_eq!(z_order.get("{b}"), Some(&0));
    }

    #[test]
    fn test_z_order_in_structured_jsonl() {
        // Test that z-order is included in structured JSONL output
        let mut palette = HashMap::new();
        palette.insert("{outer}".to_string(), "#FF0000".to_string());
        palette.insert("{inner}".to_string(), "#00FF00".to_string());

        let mut regions = HashMap::new();
        regions.insert("{outer}".to_string(), vec![[0, 0], [2, 0], [0, 2], [2, 2]]);
        regions.insert("{inner}".to_string(), vec![[1, 1]]);

        let mut z_order = HashMap::new();
        z_order.insert("{outer}".to_string(), 0);
        z_order.insert("{inner}".to_string(), 1);

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![(
                "{inner}".to_string(),
                RelationshipType::ContainedWithin,
                "{outer}".to_string(),
            )],
            symmetry: None,
            naming_hints: vec![],
            z_order,
        };

        let result = ImportResult {
            name: "test".to_string(),
            width: 3,
            height: 3,
            palette,
            grid: vec![],
            regions,
            structured_regions: None,
            analysis: Some(analysis),
        };

        let jsonl = result.to_structured_jsonl();

        // Check that z values are included in regions
        assert!(jsonl.contains("\"z\":0") || jsonl.contains("\"z\": 0"));
        assert!(jsonl.contains("\"z\":1") || jsonl.contains("\"z\": 1"));
    }
}
