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

/// How to handle detected dither patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DitherHandling {
    /// Keep dithered regions as-is (separate color tokens).
    #[default]
    Keep,
    /// Merge dithered regions into a single averaged color.
    Merge,
    /// Only detect and flag dithered regions (no merging).
    Analyze,
}

/// A detected dither pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherPattern {
    /// Checkerboard pattern (alternating colors in 2x2 grid).
    Checkerboard,
    /// Ordered dither (Bayer matrix patterns).
    Ordered,
    /// Horizontal line dither.
    HorizontalLines,
    /// Vertical line dither.
    VerticalLines,
}

impl std::fmt::Display for DitherPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DitherPattern::Checkerboard => write!(f, "checkerboard"),
            DitherPattern::Ordered => write!(f, "ordered"),
            DitherPattern::HorizontalLines => write!(f, "horizontal-lines"),
            DitherPattern::VerticalLines => write!(f, "vertical-lines"),
        }
    }
}

/// Information about a detected dithered region.
#[derive(Debug, Clone)]
pub struct DitherInfo {
    /// The tokens involved in the dither pattern.
    pub tokens: Vec<String>,
    /// The detected pattern type.
    pub pattern: DitherPattern,
    /// Bounding box of the dithered region [x, y, width, height].
    pub bounds: [u32; 4],
    /// Suggested merged color (hex) if merging is desired.
    pub merged_color: String,
    /// Confidence of the detection (0.0-1.0).
    pub confidence: f64,
}

/// Information about detected upscaling.
#[derive(Debug, Clone)]
pub struct UpscaleInfo {
    /// Detected scale factor (e.g., 2 means 2x upscaled).
    pub scale: u32,
    /// Native resolution [width, height] before upscaling.
    pub native_size: [u32; 2],
    /// Confidence of the detection (0.0-1.0).
    pub confidence: f64,
}

/// Information about a detected outline/stroke region.
#[derive(Debug, Clone)]
pub struct OutlineInfo {
    /// The token that appears to be an outline.
    pub token: String,
    /// The tokens this outlines/borders.
    pub borders: Vec<String>,
    /// Average width of the outline in pixels.
    pub width: f64,
    /// Confidence of the detection (0.0-1.0).
    pub confidence: f64,
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
    /// Export only half the sprite data when symmetry is detected.
    /// The symmetry flag will indicate how to mirror the data during rendering.
    pub half_sprite: bool,
    /// How to handle dither patterns.
    pub dither_handling: DitherHandling,
    /// Detect if image appears to be upscaled pixel art.
    pub detect_upscale: bool,
    /// Detect thin dark regions that may be outlines/strokes.
    pub detect_outlines: bool,
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
    /// Detected dither patterns and their info.
    pub dither_patterns: Vec<DitherInfo>,
    /// Detected upscaling info (if image appears to be upscaled pixel art).
    pub upscale_info: Option<UpscaleInfo>,
    /// Detected outline/stroke regions.
    pub outlines: Vec<OutlineInfo>,
}

/// Filter points to only include the primary half based on symmetry.
///
/// For X symmetry (left-right mirror), keeps only the left half.
/// For Y symmetry (top-bottom mirror), keeps only the top half.
/// For XY symmetry, keeps only the top-left quarter.
///
/// Returns the filtered points.
fn filter_points_for_half_sprite(
    points: &[[u32; 2]],
    symmetry: Symmetric,
    width: u32,
    height: u32,
) -> Vec<[u32; 2]> {
    let half_width = (width + 1) / 2; // Include center column for odd widths
    let half_height = (height + 1) / 2; // Include center row for odd heights

    points.iter()
        .filter(|p| {
            let in_left_half = p[0] < half_width;
            let in_top_half = p[1] < half_height;

            match symmetry {
                Symmetric::X => in_left_half,
                Symmetric::Y => in_top_half,
                Symmetric::XY => in_left_half && in_top_half,
            }
        })
        .copied()
        .collect()
}

/// Filter a structured region to only include the primary half.
fn filter_structured_region_for_half_sprite(
    region: &StructuredRegion,
    symmetry: Symmetric,
    width: u32,
    height: u32,
) -> StructuredRegion {
    let half_width = (width + 1) / 2;
    let half_height = (height + 1) / 2;

    match region {
        StructuredRegion::Rect([x, y, w, h]) => {
            // Compute intersection with the primary half
            let (new_x, new_y, new_w, new_h) = match symmetry {
                Symmetric::X => {
                    let end_x = (x + w).min(half_width);
                    if *x >= half_width || end_x <= *x {
                        return StructuredRegion::Points(vec![]);
                    }
                    (*x, *y, end_x - x, *h)
                }
                Symmetric::Y => {
                    let end_y = (y + h).min(half_height);
                    if *y >= half_height || end_y <= *y {
                        return StructuredRegion::Points(vec![]);
                    }
                    (*x, *y, *w, end_y - y)
                }
                Symmetric::XY => {
                    let end_x = (x + w).min(half_width);
                    let end_y = (y + h).min(half_height);
                    if *x >= half_width || *y >= half_height || end_x <= *x || end_y <= *y {
                        return StructuredRegion::Points(vec![]);
                    }
                    (*x, *y, end_x - x, end_y - y)
                }
            };
            StructuredRegion::Rect([new_x, new_y, new_w, new_h])
        }
        StructuredRegion::Polygon(vertices) => {
            // For polygons, we clip to the half region
            // This is complex, so we fall back to filtering the rasterized points
            let rasterized = rasterize_polygon(vertices);
            let points: Vec<[u32; 2]> = rasterized.into_iter()
                .map(|(x, y)| [x, y])
                .collect();
            let filtered = filter_points_for_half_sprite(&points, symmetry, width, height);
            StructuredRegion::Points(filtered)
        }
        StructuredRegion::Union(regions) => {
            let filtered: Vec<StructuredRegion> = regions.iter()
                .map(|r| filter_structured_region_for_half_sprite(r, symmetry, width, height))
                .filter(|r| !matches!(r, StructuredRegion::Points(p) if p.is_empty()))
                .collect();
            if filtered.is_empty() {
                StructuredRegion::Points(vec![])
            } else if filtered.len() == 1 {
                filtered.into_iter().next().unwrap()
            } else {
                StructuredRegion::Union(filtered)
            }
        }
        StructuredRegion::Points(points) => {
            StructuredRegion::Points(filter_points_for_half_sprite(points, symmetry, width, height))
        }
    }
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
    /// Whether half-sprite export is enabled.
    pub half_sprite: bool,
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
    ///
    /// If `half_sprite` is true and symmetry is detected, only the primary half
    /// of the sprite data is exported, with a `symmetry` field indicating how
    /// to mirror the data during rendering.
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

        // Determine if we should apply half-sprite filtering
        let apply_half_sprite = self.half_sprite
            && self.analysis.as_ref().map(|a| a.symmetry.is_some()).unwrap_or(false);
        let symmetry = self.analysis.as_ref().and_then(|a| a.symmetry);

        // Build regions object - use structured regions if available, adding z-order if present
        let z_order = self.analysis.as_ref().map(|a| &a.z_order);
        let regions: HashMap<String, serde_json::Value> = if let Some(ref structured) = self.structured_regions {
            structured
                .iter()
                .filter_map(|(token, region)| {
                    // Apply half-sprite filtering if enabled
                    let filtered_region = if apply_half_sprite {
                        let sym = symmetry.unwrap();
                        let filtered = filter_structured_region_for_half_sprite(
                            region, sym, self.width, self.height
                        );
                        // Skip empty regions
                        if matches!(&filtered, StructuredRegion::Points(p) if p.is_empty()) {
                            return None;
                        }
                        filtered
                    } else {
                        region.clone()
                    };

                    let mut region_json = filtered_region.to_json();
                    // Add z-order if available
                    if let Some(z_map) = z_order {
                        if let Some(&z) = z_map.get(token) {
                            if let serde_json::Value::Object(ref mut obj) = region_json {
                                obj.insert("z".to_string(), serde_json::json!(z));
                            }
                        }
                    }
                    Some((token.clone(), region_json))
                })
                .collect()
        } else {
            self.regions
                .iter()
                .filter_map(|(token, points)| {
                    // Apply half-sprite filtering if enabled
                    let filtered_points = if apply_half_sprite {
                        let sym = symmetry.unwrap();
                        let pts = filter_points_for_half_sprite(points, sym, self.width, self.height);
                        // Skip empty regions
                        if pts.is_empty() {
                            return None;
                        }
                        pts
                    } else {
                        points.clone()
                    };

                    let mut region_json = serde_json::json!({ "points": filtered_points });
                    // Add z-order if available
                    if let Some(z_map) = z_order {
                        if let Some(&z) = z_map.get(token) {
                            if let serde_json::Value::Object(ref mut obj) = region_json {
                                obj.insert("z".to_string(), serde_json::json!(z));
                            }
                        }
                    }
                    Some((token.clone(), region_json))
                })
                .collect()
        };

        // Compute effective size for half-sprite export
        let (export_width, export_height) = if apply_half_sprite {
            let sym = symmetry.unwrap();
            let half_w = (self.width + 1) / 2;
            let half_h = (self.height + 1) / 2;
            match sym {
                Symmetric::X => (half_w, self.height),
                Symmetric::Y => (self.width, half_h),
                Symmetric::XY => (half_w, half_h),
            }
        } else {
            (self.width, self.height)
        };

        let mut sprite_obj = serde_json::json!({
            "type": "sprite",
            "name": self.name,
            "size": [export_width, export_height],
            "palette": format!("{}_palette", self.name),
            "regions": regions
        });

        // Add symmetry metadata
        if let Some(ref analysis) = self.analysis {
            if let Some(ref sym) = analysis.symmetry {
                let sym_str = match sym {
                    Symmetric::X => "x",
                    Symmetric::Y => "y",
                    Symmetric::XY => "both",
                };

                if apply_half_sprite {
                    // For half-sprite export, add required symmetry field for reconstruction
                    sprite_obj["symmetry"] = serde_json::json!(sym_str);
                    sprite_obj["full_size"] = serde_json::json!([self.width, self.height]);
                } else {
                    // Just add as hint (underscore prefix indicates metadata)
                    sprite_obj["_symmetry"] = serde_json::json!(sym_str);
                }
            }

            // Add dither pattern info if detected
            if !analysis.dither_patterns.is_empty() {
                let dither_info: Vec<serde_json::Value> = analysis
                    .dither_patterns
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "tokens": d.tokens,
                            "pattern": d.pattern.to_string(),
                            "bounds": d.bounds,
                            "merged_color": d.merged_color,
                            "confidence": d.confidence
                        })
                    })
                    .collect();
                sprite_obj["_dither"] = serde_json::json!(dither_info);
            }

            // Add upscale info if detected
            if let Some(ref upscale) = analysis.upscale_info {
                sprite_obj["_upscale"] = serde_json::json!({
                    "scale": upscale.scale,
                    "native_size": upscale.native_size,
                    "confidence": upscale.confidence
                });
            }

            // Add outline info if detected
            if !analysis.outlines.is_empty() {
                let outline_info: Vec<serde_json::Value> = analysis
                    .outlines
                    .iter()
                    .map(|o| {
                        serde_json::json!({
                            "token": o.token,
                            "borders": o.borders,
                            "width": o.width,
                            "confidence": o.confidence
                        })
                    })
                    .collect();
                sprite_obj["_outlines"] = serde_json::json!(outline_info);
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
        half_sprite: options.half_sprite,
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

/// Detect dither patterns in the image.
///
/// Looks for common dithering patterns:
/// - Checkerboard: alternating colors in a 2x2 grid pattern
/// - Horizontal lines: alternating colors in horizontal stripes
/// - Vertical lines: alternating colors in vertical stripes
/// - Ordered dither: Bayer matrix patterns (2x2, 4x4)
fn detect_dither_patterns(
    width: u32,
    height: u32,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    token_to_color: &HashMap<String, [u8; 4]>,
) -> Vec<DitherInfo> {
    let mut dither_patterns = Vec::new();

    // Build a grid for quick token lookup
    let mut pixel_to_token: HashMap<(i32, i32), String> = HashMap::new();
    for (token, pixels) in token_pixels {
        for &(x, y) in pixels {
            pixel_to_token.insert((x, y), token.clone());
        }
    }

    // Get all token pairs to check for dithering
    let tokens: Vec<String> = token_pixels.keys().cloned().collect();

    for i in 0..tokens.len() {
        for j in (i + 1)..tokens.len() {
            let token1 = &tokens[i];
            let token2 = &tokens[j];

            // Skip transparent token
            if token1 == "{_}" || token2 == "{_}" {
                continue;
            }

            // Get pixels for both tokens
            let pixels1 = &token_pixels[token1];
            let pixels2 = &token_pixels[token2];

            // Check for checkerboard pattern between these two tokens
            if let Some(info) = detect_checkerboard_pattern(
                token1,
                token2,
                pixels1,
                pixels2,
                &pixel_to_token,
                token_to_color,
                width,
                height,
            ) {
                dither_patterns.push(info);
            }

            // Check for horizontal line pattern
            if let Some(info) = detect_line_pattern(
                token1,
                token2,
                pixels1,
                pixels2,
                &pixel_to_token,
                token_to_color,
                width,
                height,
                true, // horizontal
            ) {
                dither_patterns.push(info);
            }

            // Check for vertical line pattern
            if let Some(info) = detect_line_pattern(
                token1,
                token2,
                pixels1,
                pixels2,
                &pixel_to_token,
                token_to_color,
                width,
                height,
                false, // vertical
            ) {
                dither_patterns.push(info);
            }
        }
    }

    dither_patterns
}

/// Check if two tokens form a checkerboard dither pattern.
fn detect_checkerboard_pattern(
    token1: &str,
    token2: &str,
    pixels1: &HashSet<(i32, i32)>,
    pixels2: &HashSet<(i32, i32)>,
    pixel_to_token: &HashMap<(i32, i32), String>,
    token_to_color: &HashMap<String, [u8; 4]>,
    width: u32,
    height: u32,
) -> Option<DitherInfo> {
    // Find overlapping bounding box
    let min_x1 = pixels1.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x1 = pixels1.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y1 = pixels1.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y1 = pixels1.iter().map(|(_, y)| *y).max().unwrap_or(0);

    let min_x2 = pixels2.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x2 = pixels2.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y2 = pixels2.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y2 = pixels2.iter().map(|(_, y)| *y).max().unwrap_or(0);

    // Find the intersection of bounding boxes
    let min_x = min_x1.max(min_x2);
    let max_x = max_x1.min(max_x2);
    let min_y = min_y1.max(min_y2);
    let max_y = max_y1.min(max_y2);

    // Must have overlap
    if max_x < min_x || max_y < min_y {
        return None;
    }

    // Count checkerboard matches
    let mut checkerboard_matches = 0;
    let mut total_cells = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Expected token based on checkerboard pattern
            let is_even_cell = (x + y) % 2 == 0;
            let expected_token = if is_even_cell { token1 } else { token2 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                total_cells += 1;
                if actual_token == expected_token {
                    checkerboard_matches += 1;
                }
            }
        }
    }

    // Also try the inverse pattern
    let mut inverse_matches = 0;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let is_even_cell = (x + y) % 2 == 0;
            let expected_token = if is_even_cell { token2 } else { token1 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                if actual_token == expected_token {
                    inverse_matches += 1;
                }
            }
        }
    }

    let best_matches = checkerboard_matches.max(inverse_matches);

    // Minimum coverage and match ratio for detection
    let coverage = total_cells as f64 / ((max_x - min_x + 1) * (max_y - min_y + 1)) as f64;
    let match_ratio = if total_cells > 0 {
        best_matches as f64 / total_cells as f64
    } else {
        0.0
    };

    // Need at least 4 cells (2x2 minimum) and 80% match ratio
    if total_cells >= 4 && match_ratio >= 0.8 && coverage >= 0.7 {
        // Compute merged color
        let color1 = token_to_color.get(token1).copied().unwrap_or([0, 0, 0, 255]);
        let color2 = token_to_color.get(token2).copied().unwrap_or([0, 0, 0, 255]);
        let merged = average_colors(&[color1, color2]);

        Some(DitherInfo {
            tokens: vec![token1.to_string(), token2.to_string()],
            pattern: DitherPattern::Checkerboard,
            bounds: [min_x as u32, min_y as u32, (max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32],
            merged_color: format!("#{:02X}{:02X}{:02X}", merged[0], merged[1], merged[2]),
            confidence: match_ratio * coverage,
        })
    } else {
        None
    }
}

/// Check if two tokens form a horizontal or vertical line dither pattern.
fn detect_line_pattern(
    token1: &str,
    token2: &str,
    pixels1: &HashSet<(i32, i32)>,
    pixels2: &HashSet<(i32, i32)>,
    pixel_to_token: &HashMap<(i32, i32), String>,
    token_to_color: &HashMap<String, [u8; 4]>,
    width: u32,
    height: u32,
    horizontal: bool,
) -> Option<DitherInfo> {
    // Find overlapping bounding box
    let min_x1 = pixels1.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x1 = pixels1.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y1 = pixels1.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y1 = pixels1.iter().map(|(_, y)| *y).max().unwrap_or(0);

    let min_x2 = pixels2.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x2 = pixels2.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y2 = pixels2.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y2 = pixels2.iter().map(|(_, y)| *y).max().unwrap_or(0);

    let min_x = min_x1.max(min_x2);
    let max_x = max_x1.min(max_x2);
    let min_y = min_y1.max(min_y2);
    let max_y = max_y1.min(max_y2);

    if max_x < min_x || max_y < min_y {
        return None;
    }

    // Count line pattern matches
    let mut line_matches = 0;
    let mut total_cells = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Expected token based on line pattern
            let line_idx = if horizontal { y } else { x };
            let is_even_line = line_idx % 2 == 0;
            let expected_token = if is_even_line { token1 } else { token2 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                total_cells += 1;
                if actual_token == expected_token {
                    line_matches += 1;
                }
            }
        }
    }

    // Also try inverse
    let mut inverse_matches = 0;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let line_idx = if horizontal { y } else { x };
            let is_even_line = line_idx % 2 == 0;
            let expected_token = if is_even_line { token2 } else { token1 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                if actual_token == expected_token {
                    inverse_matches += 1;
                }
            }
        }
    }

    let best_matches = line_matches.max(inverse_matches);
    let coverage = total_cells as f64 / ((max_x - min_x + 1) * (max_y - min_y + 1)) as f64;
    let match_ratio = if total_cells > 0 {
        best_matches as f64 / total_cells as f64
    } else {
        0.0
    };

    // Need at least 4 cells and 90% match ratio for line patterns (stricter than checkerboard)
    // Also need at least 2 lines in the primary direction
    let line_count = if horizontal { max_y - min_y + 1 } else { max_x - min_x + 1 };

    if total_cells >= 4 && match_ratio >= 0.9 && coverage >= 0.8 && line_count >= 2 {
        let color1 = token_to_color.get(token1).copied().unwrap_or([0, 0, 0, 255]);
        let color2 = token_to_color.get(token2).copied().unwrap_or([0, 0, 0, 255]);
        let merged = average_colors(&[color1, color2]);

        let pattern = if horizontal {
            DitherPattern::HorizontalLines
        } else {
            DitherPattern::VerticalLines
        };

        Some(DitherInfo {
            tokens: vec![token1.to_string(), token2.to_string()],
            pattern,
            bounds: [min_x as u32, min_y as u32, (max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32],
            merged_color: format!("#{:02X}{:02X}{:02X}", merged[0], merged[1], merged[2]),
            confidence: match_ratio * coverage,
        })
    } else {
        None
    }
}

/// Average multiple colors together.
fn average_colors(colors: &[[u8; 4]]) -> [u8; 4] {
    if colors.is_empty() {
        return [0, 0, 0, 255];
    }

    let mut r: u32 = 0;
    let mut g: u32 = 0;
    let mut b: u32 = 0;
    let mut a: u32 = 0;

    for c in colors {
        r += c[0] as u32;
        g += c[1] as u32;
        b += c[2] as u32;
        a += c[3] as u32;
    }

    let n = colors.len() as u32;
    [(r / n) as u8, (g / n) as u8, (b / n) as u8, (a / n) as u8]
}

/// Detect if an image appears to be upscaled pixel art.
///
/// Checks for repeated NxN blocks of identical colors, which indicates
/// the image was upscaled from a lower resolution using nearest-neighbor scaling.
///
/// Returns the detected scale and native resolution if upscaling is found.
fn detect_upscale(
    pixel_data: &[u8],
    width: u32,
    height: u32,
) -> Option<UpscaleInfo> {
    // Check common scale factors: 2x, 3x, 4x, 5x, 6x, 8x
    let scales_to_check = [2, 3, 4, 5, 6, 8];

    for scale in scales_to_check {
        // Skip if dimensions aren't divisible by scale
        if width % scale != 0 || height % scale != 0 {
            continue;
        }

        let native_width = width / scale;
        let native_height = height / scale;

        // Check if all scale x scale blocks are uniform
        let confidence = check_uniform_blocks(pixel_data, width, height, scale);

        // Require high confidence (>95% of blocks are uniform)
        if confidence >= 0.95 {
            return Some(UpscaleInfo {
                scale,
                native_size: [native_width, native_height],
                confidence,
            });
        }
    }

    None
}

/// Check what fraction of scale x scale blocks in the image are uniform.
fn check_uniform_blocks(
    pixel_data: &[u8],
    width: u32,
    height: u32,
    scale: u32,
) -> f64 {
    let native_width = width / scale;
    let native_height = height / scale;
    let mut uniform_blocks = 0u64;
    let total_blocks = (native_width * native_height) as u64;

    for block_y in 0..native_height {
        for block_x in 0..native_width {
            // Get the color at the top-left of this block
            let base_x = block_x * scale;
            let base_y = block_y * scale;
            let base_idx = ((base_y * width + base_x) * 4) as usize;

            if base_idx + 3 >= pixel_data.len() {
                continue;
            }

            let base_color = [
                pixel_data[base_idx],
                pixel_data[base_idx + 1],
                pixel_data[base_idx + 2],
                pixel_data[base_idx + 3],
            ];

            // Check if all pixels in this block match
            let mut is_uniform = true;
            'block_check: for dy in 0..scale {
                for dx in 0..scale {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    let idx = ((py * width + px) * 4) as usize;

                    if idx + 3 >= pixel_data.len() {
                        is_uniform = false;
                        break 'block_check;
                    }

                    let color = [
                        pixel_data[idx],
                        pixel_data[idx + 1],
                        pixel_data[idx + 2],
                        pixel_data[idx + 3],
                    ];

                    if color != base_color {
                        is_uniform = false;
                        break 'block_check;
                    }
                }
            }

            if is_uniform {
                uniform_blocks += 1;
            }
        }
    }

    if total_blocks == 0 {
        return 0.0;
    }

    uniform_blocks as f64 / total_blocks as f64
}

/// Detect thin dark regions that appear to be outlines/strokes.
///
/// Looks for regions that are:
/// - Thin (1-3 pixels wide on average)
/// - Dark colored (low luminosity)
/// - Adjacent to other regions (bordering them)
fn detect_outlines(
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    token_to_color: &HashMap<String, [u8; 4]>,
    width: u32,
    height: u32,
) -> Vec<OutlineInfo> {
    let mut outlines = Vec::new();

    for (token, pixels) in token_pixels {
        // Skip transparent token
        if token == "{_}" || pixels.is_empty() {
            continue;
        }

        // Check if color is dark
        let color = token_to_color.get(token).copied().unwrap_or([0, 0, 0, 255]);
        let luminosity = (color[0] as f64 * 0.299 + color[1] as f64 * 0.587 + color[2] as f64 * 0.114) / 255.0;

        // Skip if not dark enough (luminosity > 0.3 means not dark)
        if luminosity > 0.3 {
            continue;
        }

        // Calculate average width using distance transform approximation
        let avg_width = calculate_average_width(pixels);

        // Skip if not thin (1-3px average width)
        if avg_width < 0.8 || avg_width > 3.5 {
            continue;
        }

        // Find what regions this borders
        let borders = find_bordered_regions(token, pixels, token_pixels);

        // Skip if it doesn't border anything substantial
        if borders.is_empty() {
            continue;
        }

        // Calculate confidence based on width consistency and border coverage
        let width_score = if avg_width >= 1.0 && avg_width <= 2.0 {
            1.0
        } else if avg_width <= 3.0 {
            0.8
        } else {
            0.6
        };

        let border_score = (borders.len() as f64 / 5.0).min(1.0);
        let confidence = (width_score * 0.6 + border_score * 0.4) * (1.0 - luminosity);

        if confidence >= 0.3 {
            outlines.push(OutlineInfo {
                token: token.clone(),
                borders,
                width: avg_width,
                confidence,
            });
        }
    }

    outlines
}

/// Calculate the average width of a region using morphological thinning approximation.
fn calculate_average_width(pixels: &HashSet<(i32, i32)>) -> f64 {
    if pixels.is_empty() {
        return 0.0;
    }

    // Approximate average width by looking at the ratio of perimeter to area
    // A thin line has high perimeter to area ratio

    let area = pixels.len() as f64;

    // Count perimeter pixels (pixels with at least one non-region neighbor)
    let mut perimeter = 0;
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    for &(x, y) in pixels {
        for (dx, dy) in directions {
            let nx = x + dx;
            let ny = y + dy;
            if !pixels.contains(&(nx, ny)) {
                perimeter += 1;
                break;
            }
        }
    }

    if perimeter == 0 {
        return area.sqrt(); // Rough estimate for solid regions
    }

    // For a thin line: area ≈ length * width, perimeter ≈ 2 * length + 2 * width
    // For very thin lines (width=1): perimeter ≈ 2 * length, so width ≈ area / (perimeter / 2)
    // This gives us an approximation of average width
    let estimated_length = perimeter as f64 / 2.0;
    if estimated_length > 0.0 {
        area / estimated_length
    } else {
        1.0
    }
}

/// Find regions that are adjacent to (bordered by) the given region.
fn find_bordered_regions(
    outline_token: &str,
    outline_pixels: &HashSet<(i32, i32)>,
    all_token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
) -> Vec<String> {
    let mut bordered = HashSet::new();
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1)];

    // Build reverse lookup for efficiency
    let mut pixel_to_token: HashMap<(i32, i32), &str> = HashMap::new();
    for (token, pixels) in all_token_pixels {
        for &(x, y) in pixels {
            pixel_to_token.insert((x, y), token.as_str());
        }
    }

    // Check each outline pixel for adjacent non-outline regions
    for &(x, y) in outline_pixels {
        for (dx, dy) in directions {
            let nx = x + dx;
            let ny = y + dy;

            if let Some(&adjacent_token) = pixel_to_token.get(&(nx, ny)) {
                if adjacent_token != outline_token && adjacent_token != "{_}" {
                    bordered.insert(adjacent_token.to_string());
                }
            }
        }
    }

    bordered.into_iter().collect()
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

    // Detect dither patterns if dither handling is not Keep
    if options.dither_handling != DitherHandling::Keep {
        let mut patterns = detect_dither_patterns(width, height, token_pixels, &token_to_color);
        // Filter by confidence threshold
        patterns.retain(|p| p.confidence >= options.confidence_threshold);
        analysis.dither_patterns = patterns;
    }

    // Detect upscaled pixel art if requested
    if options.detect_upscale {
        if let Some(upscale_info) = detect_upscale(&pixel_data, width, height) {
            if upscale_info.confidence >= options.confidence_threshold {
                analysis.upscale_info = Some(upscale_info);
            }
        }
    }

    // Detect outline/stroke regions if requested
    if options.detect_outlines {
        let mut outlines = detect_outlines(token_pixels, &token_to_color, width, height);
        outlines.retain(|o| o.confidence >= options.confidence_threshold);
        analysis.outlines = outlines;
    }

    // Generate naming hints if requested
    if options.hints {
        analysis.naming_hints = generate_naming_hints(
            &analysis.roles,
            token_pixels,
            &token_to_color,
            width,
            height,
        );
    }

    analysis
}

/// Semantic region position within the sprite.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SemanticPosition {
    TopCenter,     // Hair region
    Center,        // Face/body center
    Bottom,        // Feet/base
    Edge,          // Border/background
    TopCorner,     // Hair edge or accessory
    Surrounding,   // Background
}

/// Analyze the semantic position of a region within the sprite.
fn analyze_semantic_position(
    pixels: &HashSet<(i32, i32)>,
    width: u32,
    height: u32,
) -> (SemanticPosition, f64) {
    if pixels.is_empty() {
        return (SemanticPosition::Edge, 0.0);
    }

    let total_pixels = (width * height) as f64;
    let coverage = pixels.len() as f64 / total_pixels;

    // Large coverage suggests background
    if coverage > 0.5 {
        return (SemanticPosition::Surrounding, 0.9);
    }

    // Calculate centroid
    let sum_x: i32 = pixels.iter().map(|(x, _)| *x).sum();
    let sum_y: i32 = pixels.iter().map(|(_, y)| *y).sum();
    let centroid_x = sum_x as f64 / pixels.len() as f64;
    let centroid_y = sum_y as f64 / pixels.len() as f64;

    // Normalize to 0-1 range
    let norm_x = centroid_x / width as f64;
    let norm_y = centroid_y / height as f64;

    // Check if region touches edges (potential background)
    let touches_left = pixels.iter().any(|(x, _)| *x == 0);
    let touches_right = pixels.iter().any(|(x, _)| *x as u32 == width - 1);
    let touches_top = pixels.iter().any(|(_, y)| *y == 0);
    let touches_bottom = pixels.iter().any(|(_, y)| *y as u32 == height - 1);
    let edge_count = [touches_left, touches_right, touches_top, touches_bottom]
        .iter()
        .filter(|&&b| b)
        .count();

    // Touches multiple edges and moderate coverage = background
    if edge_count >= 2 && coverage > 0.15 {
        return (SemanticPosition::Surrounding, 0.8);
    }

    // Determine position based on centroid
    let is_center_x = (0.25..0.75).contains(&norm_x);
    let is_top = norm_y < 0.33;
    let is_middle = (0.33..0.66).contains(&norm_y);
    let is_bottom = norm_y >= 0.66;

    if is_center_x && is_top {
        (SemanticPosition::TopCenter, 0.7)
    } else if is_center_x && is_middle {
        (SemanticPosition::Center, 0.7)
    } else if is_center_x && is_bottom {
        (SemanticPosition::Bottom, 0.7)
    } else if is_top && !is_center_x {
        (SemanticPosition::TopCorner, 0.6)
    } else {
        (SemanticPosition::Edge, 0.5)
    }
}

/// Check if a color is a skin tone (various skin tones).
fn is_skin_tone(color: &[u8; 4]) -> bool {
    let lab = LabColor::from_rgb(color[0], color[1], color[2]);

    // Skin tones have:
    // - Moderate to high lightness (L: 40-90)
    // - Positive a (reddish)
    // - Moderate positive b (yellowish)
    lab.l > 40.0 && lab.l < 90.0 && lab.a > 5.0 && lab.a < 40.0 && lab.b > 5.0 && lab.b < 50.0
}

/// Check if a color is dark (potential outline/shadow/hair).
fn is_dark_color(color: &[u8; 4]) -> bool {
    let lab = LabColor::from_rgb(color[0], color[1], color[2]);
    lab.l < 35.0
}

/// Check if a color is light (potential highlight/white).
fn is_light_color(color: &[u8; 4]) -> bool {
    let lab = LabColor::from_rgb(color[0], color[1], color[2]);
    lab.l > 85.0
}

/// Generate token naming suggestions based on detected features.
fn generate_naming_hints(
    roles: &HashMap<String, Role>,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    token_colors: &HashMap<String, [u8; 4]>,
    width: u32,
    height: u32,
) -> Vec<NamingHint> {
    let mut hints = Vec::new();
    let total_pixels = (width * height) as usize;

    for (token, pixels) in token_pixels {
        // Skip transparent token
        if token == "{_}" {
            continue;
        }

        let color = token_colors.get(token);
        let role = roles.get(token);
        let (position, position_confidence) = analyze_semantic_position(pixels, width, height);
        let size = pixels.len();
        let coverage = size as f64 / total_pixels as f64;

        // Build suggestion based on multiple factors
        let (suggested, reason) = suggest_semantic_name(
            position,
            position_confidence,
            color,
            role,
            size,
            coverage,
            width,
            height,
        );

        if let Some(suggested_name) = suggested {
            if token != &suggested_name {
                hints.push(NamingHint {
                    token: token.clone(),
                    suggested_name,
                    reason,
                });
            }
        }
    }

    hints
}

/// Suggest a semantic name based on position, color, role, and size.
fn suggest_semantic_name(
    position: SemanticPosition,
    position_confidence: f64,
    color: Option<&[u8; 4]>,
    role: Option<&Role>,
    size: usize,
    coverage: f64,
    _width: u32,
    _height: u32,
) -> (Option<String>, String) {
    // Background detection: large coverage or surrounding position
    if coverage > 0.4 || position == SemanticPosition::Surrounding {
        return (Some("{bg}".to_string()), "Large coverage, likely background".to_string());
    }

    // Check for skin tones in center = face/skin
    if let Some(c) = color {
        if is_skin_tone(c) {
            if position == SemanticPosition::Center {
                return (Some("{face}".to_string()), "Skin tone in center region".to_string());
            } else if position == SemanticPosition::TopCenter {
                return (Some("{skin}".to_string()), "Skin tone in upper region".to_string());
            } else if coverage > 0.05 {
                return (Some("{skin}".to_string()), "Detected skin tone color".to_string());
            }
        }

        // Dark colors
        if is_dark_color(c) {
            // Top center dark = hair
            if position == SemanticPosition::TopCenter {
                return (Some("{hair}".to_string()), "Dark color in top center".to_string());
            }
            // Small dark spots near center = eyes
            if size <= 6 && position == SemanticPosition::Center {
                return (Some("{eye}".to_string()), "Small dark region in center".to_string());
            }
            // Role-based dark suggestions
            if matches!(role, Some(Role::Boundary)) {
                return (Some("{outline}".to_string()), "Dark boundary region".to_string());
            }
            if matches!(role, Some(Role::Shadow)) {
                return (Some("{shadow}".to_string()), "Dark shadow region".to_string());
            }
            // Generic dark
            if position == SemanticPosition::Edge {
                return (Some("{outline}".to_string()), "Dark edge region".to_string());
            }
        }

        // Light colors
        if is_light_color(c) {
            if matches!(role, Some(Role::Highlight)) {
                return (Some("{highlight}".to_string()), "Light highlight region".to_string());
            }
            // Small light spots = eyes/reflections
            if size <= 4 {
                return (Some("{gleam}".to_string()), "Small light region (reflection)".to_string());
            }
        }
    }

    // Size-based suggestions for small features
    if size == 1 {
        return (Some("{dot}".to_string()), "Single pixel feature".to_string());
    }
    if size <= 4 {
        if position == SemanticPosition::Center {
            return (Some("{eye}".to_string()), "Small centered feature".to_string());
        }
        return (Some("{detail}".to_string()), "Small detail region".to_string());
    }

    // Position-based fallbacks
    match position {
        SemanticPosition::TopCenter if position_confidence > 0.6 => {
            (Some("{top}".to_string()), "Top center region".to_string())
        }
        SemanticPosition::Center if position_confidence > 0.6 => {
            (Some("{body}".to_string()), "Central body region".to_string())
        }
        SemanticPosition::Bottom if position_confidence > 0.6 => {
            (Some("{base}".to_string()), "Bottom base region".to_string())
        }
        _ => {
            // Fall back to role-based hints
            if let Some(r) = role {
                let name = match r {
                    Role::Boundary => "{outline}",
                    Role::Anchor => "{marker}",
                    Role::Fill => "{fill}",
                    Role::Shadow => "{shadow}",
                    Role::Highlight => "{highlight}",
                };
                (Some(name.to_string()), format!("Detected as {} role", r))
            } else {
                (None, String::new())
            }
        }
    }
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
            half_sprite: false,
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
            dither_patterns: vec![],
            upscale_info: None,
            outlines: vec![],
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
            half_sprite: false,
        };

        let jsonl = result.to_structured_jsonl();

        // Check that z values are included in regions
        assert!(jsonl.contains("\"z\":0") || jsonl.contains("\"z\": 0"));
        assert!(jsonl.contains("\"z\":1") || jsonl.contains("\"z\": 1"));
    }

    #[test]
    fn test_semantic_position_surrounding() {
        // Large region covering >50% = surrounding
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        for x in 0..8 {
            for y in 0..8 {
                pixels.insert((x, y));
            }
        }

        let (pos, conf) = analyze_semantic_position(&pixels, 10, 10);
        assert_eq!(pos, SemanticPosition::Surrounding);
        assert!(conf > 0.8);
    }

    #[test]
    fn test_semantic_position_top_center() {
        // Region in top center
        let pixels: HashSet<(i32, i32)> = [(4, 1), (5, 1), (4, 2), (5, 2)].into_iter().collect();

        let (pos, _conf) = analyze_semantic_position(&pixels, 10, 10);
        assert_eq!(pos, SemanticPosition::TopCenter);
    }

    #[test]
    fn test_semantic_position_center() {
        // Region in center
        let pixels: HashSet<(i32, i32)> = [(4, 4), (5, 4), (4, 5), (5, 5)].into_iter().collect();

        let (pos, _conf) = analyze_semantic_position(&pixels, 10, 10);
        assert_eq!(pos, SemanticPosition::Center);
    }

    #[test]
    fn test_semantic_position_bottom() {
        // Region at bottom
        let pixels: HashSet<(i32, i32)> = [(4, 8), (5, 8), (4, 9), (5, 9)].into_iter().collect();

        let (pos, _conf) = analyze_semantic_position(&pixels, 10, 10);
        assert_eq!(pos, SemanticPosition::Bottom);
    }

    #[test]
    fn test_is_skin_tone() {
        // Light skin tone (typical Caucasian/light skin)
        assert!(is_skin_tone(&[255, 220, 185, 255]));
        // Medium skin tone
        assert!(is_skin_tone(&[210, 160, 120, 255]));
        // Not skin: pure red
        assert!(!is_skin_tone(&[255, 0, 0, 255]));
        // Not skin: pure white
        assert!(!is_skin_tone(&[255, 255, 255, 255]));
        // Not skin: pure black
        assert!(!is_skin_tone(&[0, 0, 0, 255]));
    }

    #[test]
    fn test_is_dark_color() {
        assert!(is_dark_color(&[0, 0, 0, 255]));       // Black
        assert!(is_dark_color(&[30, 30, 30, 255]));    // Dark gray
        assert!(!is_dark_color(&[128, 128, 128, 255])); // Medium gray
        assert!(!is_dark_color(&[255, 255, 255, 255])); // White
    }

    #[test]
    fn test_is_light_color() {
        assert!(is_light_color(&[255, 255, 255, 255])); // White
        assert!(is_light_color(&[240, 240, 240, 255])); // Near white
        assert!(!is_light_color(&[128, 128, 128, 255])); // Medium gray
        assert!(!is_light_color(&[0, 0, 0, 255]));       // Black
    }

    #[test]
    fn test_semantic_naming_background() {
        // Large coverage = background
        let (name, reason) = suggest_semantic_name(
            SemanticPosition::Edge,
            0.5,
            None,
            None,
            60,
            0.6, // 60% coverage
            10,
            10,
        );
        assert_eq!(name, Some("{bg}".to_string()));
        assert!(reason.contains("background"));
    }

    #[test]
    fn test_semantic_naming_dark_top_center() {
        // Dark color in top center = hair
        let dark_color = [20, 20, 20, 255];
        let (name, reason) = suggest_semantic_name(
            SemanticPosition::TopCenter,
            0.8,
            Some(&dark_color),
            None,
            20,
            0.2,
            10,
            10,
        );
        assert_eq!(name, Some("{hair}".to_string()));
        assert!(reason.contains("Dark"));
    }

    #[test]
    fn test_semantic_naming_skin_center() {
        // Skin tone in center = face
        let skin_color = [220, 180, 150, 255];
        let (name, reason) = suggest_semantic_name(
            SemanticPosition::Center,
            0.8,
            Some(&skin_color),
            None,
            30,
            0.3,
            10,
            10,
        );
        assert_eq!(name, Some("{face}".to_string()));
        assert!(reason.contains("Skin tone"));
    }

    #[test]
    fn test_semantic_naming_small_dark_center() {
        // Small dark spot in center = eye
        let dark_color = [10, 10, 10, 255];
        let (name, reason) = suggest_semantic_name(
            SemanticPosition::Center,
            0.8,
            Some(&dark_color),
            None,
            4,   // Small
            0.04,
            10,
            10,
        );
        assert_eq!(name, Some("{eye}".to_string()));
        assert!(reason.contains("Small dark"));
    }

    #[test]
    fn test_semantic_naming_small_light() {
        // Small light spot = gleam/reflection
        let light_color = [250, 250, 250, 255];
        let (name, reason) = suggest_semantic_name(
            SemanticPosition::Center,
            0.8,
            Some(&light_color),
            None,
            2,   // Very small
            0.02,
            10,
            10,
        );
        assert_eq!(name, Some("{gleam}".to_string()));
        assert!(reason.contains("reflection"));
    }

    // ========================================================================
    // Half-Sprite Export Tests (TTP-trwxq.5)
    // ========================================================================

    #[test]
    fn test_filter_points_for_half_sprite_x() {
        // X symmetry: keep left half only
        // 4x2 sprite, left half is columns 0,1 (half_width = 2)
        let points = vec![[0, 0], [1, 0], [2, 0], [3, 0], [0, 1], [1, 1], [2, 1], [3, 1]];
        let filtered = filter_points_for_half_sprite(&points, Symmetric::X, 4, 2);

        assert_eq!(filtered.len(), 4);
        assert!(filtered.contains(&[0, 0]));
        assert!(filtered.contains(&[1, 0]));
        assert!(filtered.contains(&[0, 1]));
        assert!(filtered.contains(&[1, 1]));
        // Right half should be excluded
        assert!(!filtered.contains(&[2, 0]));
        assert!(!filtered.contains(&[3, 0]));
    }

    #[test]
    fn test_filter_points_for_half_sprite_y() {
        // Y symmetry: keep top half only
        // 2x4 sprite, top half is rows 0,1 (half_height = 2)
        let points = vec![[0, 0], [1, 0], [0, 1], [1, 1], [0, 2], [1, 2], [0, 3], [1, 3]];
        let filtered = filter_points_for_half_sprite(&points, Symmetric::Y, 2, 4);

        assert_eq!(filtered.len(), 4);
        assert!(filtered.contains(&[0, 0]));
        assert!(filtered.contains(&[1, 0]));
        assert!(filtered.contains(&[0, 1]));
        assert!(filtered.contains(&[1, 1]));
        // Bottom half should be excluded
        assert!(!filtered.contains(&[0, 2]));
        assert!(!filtered.contains(&[1, 3]));
    }

    #[test]
    fn test_filter_points_for_half_sprite_xy() {
        // XY symmetry: keep top-left quarter only
        // 4x4 sprite, quarter is columns 0,1 and rows 0,1
        let mut points = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                points.push([x, y]);
            }
        }
        let filtered = filter_points_for_half_sprite(&points, Symmetric::XY, 4, 4);

        assert_eq!(filtered.len(), 4);
        assert!(filtered.contains(&[0, 0]));
        assert!(filtered.contains(&[1, 0]));
        assert!(filtered.contains(&[0, 1]));
        assert!(filtered.contains(&[1, 1]));
    }

    #[test]
    fn test_filter_points_for_half_sprite_odd_width() {
        // Odd width: include center column for X symmetry
        // 5x2 sprite, left half includes columns 0,1,2 (center)
        let points = vec![[0, 0], [1, 0], [2, 0], [3, 0], [4, 0]];
        let filtered = filter_points_for_half_sprite(&points, Symmetric::X, 5, 2);

        assert_eq!(filtered.len(), 3); // Columns 0,1,2
        assert!(filtered.contains(&[0, 0]));
        assert!(filtered.contains(&[1, 0]));
        assert!(filtered.contains(&[2, 0])); // Center column included
        assert!(!filtered.contains(&[3, 0]));
        assert!(!filtered.contains(&[4, 0]));
    }

    #[test]
    fn test_filter_structured_region_rect_x() {
        // A 4x2 rect spanning full width, X symmetry should clip to left half
        let region = StructuredRegion::Rect([0, 0, 4, 2]);
        let filtered = filter_structured_region_for_half_sprite(&region, Symmetric::X, 4, 2);

        match filtered {
            StructuredRegion::Rect([x, y, w, h]) => {
                assert_eq!(x, 0);
                assert_eq!(y, 0);
                assert_eq!(w, 2); // Half of 4
                assert_eq!(h, 2); // Full height
            }
            _ => panic!("Expected Rect"),
        }
    }

    #[test]
    fn test_filter_structured_region_rect_y() {
        // A 2x4 rect spanning full height, Y symmetry should clip to top half
        let region = StructuredRegion::Rect([0, 0, 2, 4]);
        let filtered = filter_structured_region_for_half_sprite(&region, Symmetric::Y, 2, 4);

        match filtered {
            StructuredRegion::Rect([x, y, w, h]) => {
                assert_eq!(x, 0);
                assert_eq!(y, 0);
                assert_eq!(w, 2); // Full width
                assert_eq!(h, 2); // Half of 4
            }
            _ => panic!("Expected Rect"),
        }
    }

    #[test]
    fn test_filter_structured_region_rect_xy() {
        // A 4x4 rect, XY symmetry should clip to quarter
        let region = StructuredRegion::Rect([0, 0, 4, 4]);
        let filtered = filter_structured_region_for_half_sprite(&region, Symmetric::XY, 4, 4);

        match filtered {
            StructuredRegion::Rect([x, y, w, h]) => {
                assert_eq!(x, 0);
                assert_eq!(y, 0);
                assert_eq!(w, 2);
                assert_eq!(h, 2);
            }
            _ => panic!("Expected Rect"),
        }
    }

    #[test]
    fn test_filter_structured_region_rect_outside() {
        // A rect fully in the right half should become empty
        let region = StructuredRegion::Rect([3, 0, 1, 2]);
        let filtered = filter_structured_region_for_half_sprite(&region, Symmetric::X, 4, 2);

        match filtered {
            StructuredRegion::Points(pts) => assert!(pts.is_empty()),
            _ => panic!("Expected empty Points"),
        }
    }

    #[test]
    fn test_half_sprite_export_jsonl() {
        // Create an X-symmetric sprite with half_sprite enabled
        let mut palette = HashMap::new();
        palette.insert("{a}".to_string(), "#FF0000".to_string());

        // Full points: [[0,0], [1,0], [2,0], [3,0]]
        // With X symmetry and half_sprite, only [[0,0], [1,0]] should be exported
        let mut regions = HashMap::new();
        regions.insert("{a}".to_string(), vec![[0, 0], [1, 0], [2, 0], [3, 0]]);

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![],
            symmetry: Some(Symmetric::X),
            naming_hints: vec![],
            z_order: HashMap::new(),
            dither_patterns: vec![],
            upscale_info: None,
            outlines: vec![],
        };

        let result = ImportResult {
            name: "half_test".to_string(),
            width: 4,
            height: 1,
            palette,
            grid: vec![],
            regions,
            structured_regions: None,
            analysis: Some(analysis),
            half_sprite: true,
        };

        let jsonl = result.to_structured_jsonl();

        // Should have symmetry field (not _symmetry)
        assert!(jsonl.contains("\"symmetry\":\"x\"") || jsonl.contains("\"symmetry\": \"x\""));
        // Should have full_size
        assert!(jsonl.contains("\"full_size\":[4,1]") || jsonl.contains("\"full_size\": [4, 1]"));
        // Size should be half width
        assert!(jsonl.contains("\"size\":[2,1]") || jsonl.contains("\"size\": [2, 1]"));
        // Points should only include left half
        assert!(jsonl.contains("[0,0]") || jsonl.contains("[0, 0]"));
        assert!(jsonl.contains("[1,0]") || jsonl.contains("[1, 0]"));
        // Right half points should NOT be in output
        assert!(!jsonl.contains("[3,0]") && !jsonl.contains("[3, 0]"));
    }

    #[test]
    fn test_half_sprite_disabled_preserves_full_data() {
        // When half_sprite is false, all data should be preserved
        let mut palette = HashMap::new();
        palette.insert("{a}".to_string(), "#FF0000".to_string());

        let mut regions = HashMap::new();
        regions.insert("{a}".to_string(), vec![[0, 0], [1, 0], [2, 0], [3, 0]]);

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![],
            symmetry: Some(Symmetric::X),
            naming_hints: vec![],
            z_order: HashMap::new(),
            dither_patterns: vec![],
            upscale_info: None,
            outlines: vec![],
        };

        let result = ImportResult {
            name: "full_test".to_string(),
            width: 4,
            height: 1,
            palette,
            grid: vec![],
            regions,
            structured_regions: None,
            analysis: Some(analysis),
            half_sprite: false,
        };

        let jsonl = result.to_structured_jsonl();

        // Should have _symmetry (hint) not symmetry (required)
        assert!(jsonl.contains("\"_symmetry\":\"x\"") || jsonl.contains("\"_symmetry\": \"x\""));
        // Should NOT have full_size (since we're exporting full data)
        assert!(!jsonl.contains("full_size"));
        // Size should be full size
        assert!(jsonl.contains("\"size\":[4,1]") || jsonl.contains("\"size\": [4, 1]"));
        // All points should be present
        assert!(jsonl.contains("[0,0]") || jsonl.contains("[0, 0]"));
        assert!(jsonl.contains("[3,0]") || jsonl.contains("[3, 0]"));
    }

    #[test]
    fn test_half_sprite_no_symmetry_no_effect() {
        // When no symmetry detected, half_sprite option has no effect
        let mut palette = HashMap::new();
        palette.insert("{a}".to_string(), "#FF0000".to_string());

        let mut regions = HashMap::new();
        regions.insert("{a}".to_string(), vec![[0, 0], [1, 0], [2, 0], [3, 0]]);

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![],
            symmetry: None, // No symmetry detected
            naming_hints: vec![],
            z_order: HashMap::new(),
            dither_patterns: vec![],
            upscale_info: None,
            outlines: vec![],
        };

        let result = ImportResult {
            name: "nosym_test".to_string(),
            width: 4,
            height: 1,
            palette,
            grid: vec![],
            regions,
            structured_regions: None,
            analysis: Some(analysis),
            half_sprite: true, // Even with this true, no filtering happens
        };

        let jsonl = result.to_structured_jsonl();

        // Should NOT have symmetry or full_size
        assert!(!jsonl.contains("symmetry"));
        assert!(!jsonl.contains("full_size"));
        // All points should be present
        assert!(jsonl.contains("[3,0]") || jsonl.contains("[3, 0]"));
    }

    // ========================================================================
    // Dither Detection Tests (TTP-trwxq.6)
    // ========================================================================

    #[test]
    fn test_detect_checkerboard_pattern_basic() {
        // 4x4 checkerboard pattern
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut c2_pixels = HashSet::new();

        // Checkerboard: c1 on even cells, c2 on odd cells
        for y in 0..4 {
            for x in 0..4 {
                if (x + y) % 2 == 0 {
                    c1_pixels.insert((x, y));
                } else {
                    c2_pixels.insert((x, y));
                }
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{c2}".to_string(), c2_pixels);
        token_to_color.insert("{c1}".to_string(), [255, 0, 0, 255]);
        token_to_color.insert("{c2}".to_string(), [0, 0, 255, 255]);

        let patterns = detect_dither_patterns(4, 4, &token_pixels, &token_to_color);

        assert!(!patterns.is_empty(), "Should detect checkerboard pattern");
        let pattern = &patterns[0];
        assert_eq!(pattern.pattern, DitherPattern::Checkerboard);
        assert!(pattern.confidence > 0.8);
        assert!(pattern.tokens.contains(&"{c1}".to_string()));
        assert!(pattern.tokens.contains(&"{c2}".to_string()));
    }

    #[test]
    fn test_detect_checkerboard_pattern_merged_color() {
        // Verify merged color is average of the two colors
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut c2_pixels = HashSet::new();

        for y in 0..4 {
            for x in 0..4 {
                if (x + y) % 2 == 0 {
                    c1_pixels.insert((x, y));
                } else {
                    c2_pixels.insert((x, y));
                }
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{c2}".to_string(), c2_pixels);
        // Red (255, 0, 0) and Blue (0, 0, 255) average to (127, 0, 127) => purple
        token_to_color.insert("{c1}".to_string(), [254, 0, 0, 255]);
        token_to_color.insert("{c2}".to_string(), [0, 0, 254, 255]);

        let patterns = detect_dither_patterns(4, 4, &token_pixels, &token_to_color);

        assert!(!patterns.is_empty());
        let pattern = &patterns[0];
        // Average of 254 and 0 is 127
        assert_eq!(pattern.merged_color, "#7F007F");
    }

    #[test]
    fn test_detect_horizontal_line_pattern() {
        // Alternating horizontal lines
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut c2_pixels = HashSet::new();

        for y in 0..4 {
            for x in 0..4 {
                if y % 2 == 0 {
                    c1_pixels.insert((x, y));
                } else {
                    c2_pixels.insert((x, y));
                }
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{c2}".to_string(), c2_pixels);
        token_to_color.insert("{c1}".to_string(), [255, 255, 255, 255]);
        token_to_color.insert("{c2}".to_string(), [0, 0, 0, 255]);

        let patterns = detect_dither_patterns(4, 4, &token_pixels, &token_to_color);

        let line_pattern = patterns.iter().find(|p| p.pattern == DitherPattern::HorizontalLines);
        assert!(line_pattern.is_some(), "Should detect horizontal line pattern");
    }

    #[test]
    fn test_detect_vertical_line_pattern() {
        // Alternating vertical lines
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut c2_pixels = HashSet::new();

        for y in 0..4 {
            for x in 0..4 {
                if x % 2 == 0 {
                    c1_pixels.insert((x, y));
                } else {
                    c2_pixels.insert((x, y));
                }
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{c2}".to_string(), c2_pixels);
        token_to_color.insert("{c1}".to_string(), [255, 255, 255, 255]);
        token_to_color.insert("{c2}".to_string(), [0, 0, 0, 255]);

        let patterns = detect_dither_patterns(4, 4, &token_pixels, &token_to_color);

        let line_pattern = patterns.iter().find(|p| p.pattern == DitherPattern::VerticalLines);
        assert!(line_pattern.is_some(), "Should detect vertical line pattern");
    }

    #[test]
    fn test_no_dither_pattern_solid_regions() {
        // Two separate solid regions - no dither
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut c2_pixels = HashSet::new();

        // c1 fills left half
        for y in 0..4 {
            for x in 0..2 {
                c1_pixels.insert((x, y));
            }
        }
        // c2 fills right half
        for y in 0..4 {
            for x in 2..4 {
                c2_pixels.insert((x, y));
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{c2}".to_string(), c2_pixels);
        token_to_color.insert("{c1}".to_string(), [255, 0, 0, 255]);
        token_to_color.insert("{c2}".to_string(), [0, 0, 255, 255]);

        let patterns = detect_dither_patterns(4, 4, &token_pixels, &token_to_color);

        assert!(patterns.is_empty(), "Should not detect dither in solid regions");
    }

    #[test]
    fn test_dither_detection_skips_transparent() {
        // Transparent token should not be considered for dithering
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut transparent_pixels = HashSet::new();

        // Checkerboard with transparent
        for y in 0..4 {
            for x in 0..4 {
                if (x + y) % 2 == 0 {
                    c1_pixels.insert((x, y));
                } else {
                    transparent_pixels.insert((x, y));
                }
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{_}".to_string(), transparent_pixels);
        token_to_color.insert("{c1}".to_string(), [255, 0, 0, 255]);
        token_to_color.insert("{_}".to_string(), [0, 0, 0, 0]);

        let patterns = detect_dither_patterns(4, 4, &token_pixels, &token_to_color);

        assert!(patterns.is_empty(), "Should not detect dither with transparent token");
    }

    #[test]
    fn test_dither_bounds_calculation() {
        // Partial checkerboard - verify bounds are correct
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut c1_pixels = HashSet::new();
        let mut c2_pixels = HashSet::new();

        // Checkerboard only in region (2,2) to (5,5) within 8x8 image
        for y in 2..6 {
            for x in 2..6 {
                if (x + y) % 2 == 0 {
                    c1_pixels.insert((x, y));
                } else {
                    c2_pixels.insert((x, y));
                }
            }
        }

        token_pixels.insert("{c1}".to_string(), c1_pixels);
        token_pixels.insert("{c2}".to_string(), c2_pixels);
        token_to_color.insert("{c1}".to_string(), [100, 100, 100, 255]);
        token_to_color.insert("{c2}".to_string(), [200, 200, 200, 255]);

        let patterns = detect_dither_patterns(8, 8, &token_pixels, &token_to_color);

        assert!(!patterns.is_empty());
        let pattern = &patterns[0];
        assert_eq!(pattern.bounds, [2, 2, 4, 4], "Bounds should be [2, 2, 4, 4]");
    }

    #[test]
    fn test_dither_handling_default_is_keep() {
        let options = ImportOptions::default();
        assert_eq!(options.dither_handling, DitherHandling::Keep);
    }

    #[test]
    fn test_average_colors() {
        // Test averaging colors
        let colors = [[255, 0, 0, 255], [0, 255, 0, 255]];
        let avg = average_colors(&colors);
        assert_eq!(avg, [127, 127, 0, 255]);

        let colors2 = [[100, 100, 100, 255], [200, 200, 200, 255]];
        let avg2 = average_colors(&colors2);
        assert_eq!(avg2, [150, 150, 150, 255]);

        // Empty colors
        let empty: &[[u8; 4]] = &[];
        assert_eq!(average_colors(empty), [0, 0, 0, 255]);
    }

    #[test]
    fn test_dither_info_in_structured_jsonl() {
        // Verify dither info is included in structured JSONL
        let mut palette = HashMap::new();
        palette.insert("{c1}".to_string(), "#FF0000".to_string());
        palette.insert("{c2}".to_string(), "#0000FF".to_string());

        let mut regions = HashMap::new();
        regions.insert("{c1}".to_string(), vec![[0, 0], [1, 1], [0, 2], [1, 3]]);
        regions.insert("{c2}".to_string(), vec![[1, 0], [0, 1], [1, 2], [0, 3]]);

        let dither_patterns = vec![DitherInfo {
            tokens: vec!["{c1}".to_string(), "{c2}".to_string()],
            pattern: DitherPattern::Checkerboard,
            bounds: [0, 0, 2, 4],
            merged_color: "#7F007F".to_string(),
            confidence: 0.95,
        }];

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![],
            symmetry: None,
            naming_hints: vec![],
            z_order: HashMap::new(),
            dither_patterns,
            upscale_info: None,
            outlines: vec![],
        };

        let result = ImportResult {
            name: "test".to_string(),
            width: 2,
            height: 4,
            palette,
            grid: vec![],
            regions,
            structured_regions: None,
            analysis: Some(analysis),
            half_sprite: false,
        };

        let jsonl = result.to_structured_jsonl();

        assert!(jsonl.contains("_dither"), "Should include _dither field");
        assert!(jsonl.contains("checkerboard"), "Should include pattern type");
        assert!(jsonl.contains("#7F007F"), "Should include merged color");
    }

    #[test]
    fn test_dither_pattern_display() {
        assert_eq!(DitherPattern::Checkerboard.to_string(), "checkerboard");
        assert_eq!(DitherPattern::Ordered.to_string(), "ordered");
        assert_eq!(DitherPattern::HorizontalLines.to_string(), "horizontal-lines");
        assert_eq!(DitherPattern::VerticalLines.to_string(), "vertical-lines");
    }

    // ========================================================================
    // Upscale Detection Tests (TTP-trwxq.7)
    // ========================================================================

    #[test]
    fn test_detect_upscale_2x() {
        // 4x4 image that is actually 2x2 pixel art upscaled 2x
        // Native pixels: [[R,G], [B,W]] becomes 4x4 with 2x2 blocks
        let mut pixel_data = vec![0u8; 4 * 4 * 4]; // 4x4 RGBA

        // Set up 2x2 blocks
        // Block (0,0) = Red
        for dy in 0..2 {
            for dx in 0..2 {
                let idx = ((dy * 4 + dx) * 4) as usize;
                pixel_data[idx..idx + 4].copy_from_slice(&[255, 0, 0, 255]);
            }
        }
        // Block (1,0) = Green
        for dy in 0..2 {
            for dx in 2..4 {
                let idx = ((dy * 4 + dx) * 4) as usize;
                pixel_data[idx..idx + 4].copy_from_slice(&[0, 255, 0, 255]);
            }
        }
        // Block (0,1) = Blue
        for dy in 2..4 {
            for dx in 0..2 {
                let idx = ((dy * 4 + dx) * 4) as usize;
                pixel_data[idx..idx + 4].copy_from_slice(&[0, 0, 255, 255]);
            }
        }
        // Block (1,1) = White
        for dy in 2..4 {
            for dx in 2..4 {
                let idx = ((dy * 4 + dx) * 4) as usize;
                pixel_data[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }

        let result = detect_upscale(&pixel_data, 4, 4);

        assert!(result.is_some(), "Should detect 2x upscale");
        let info = result.unwrap();
        assert_eq!(info.scale, 2);
        assert_eq!(info.native_size, [2, 2]);
        assert!(info.confidence >= 0.95);
    }

    #[test]
    fn test_detect_upscale_3x() {
        // 6x6 image that is actually 2x2 pixel art upscaled 3x
        let mut pixel_data = vec![0u8; 6 * 6 * 4]; // 6x6 RGBA

        // Fill with 3x3 blocks
        for block_y in 0..2 {
            for block_x in 0..2 {
                let color: [u8; 4] = match (block_x, block_y) {
                    (0, 0) => [255, 0, 0, 255],   // Red
                    (1, 0) => [0, 255, 0, 255],   // Green
                    (0, 1) => [0, 0, 255, 255],   // Blue
                    _ => [255, 255, 255, 255],    // White
                };

                for dy in 0..3 {
                    for dx in 0..3 {
                        let px = block_x * 3 + dx;
                        let py = block_y * 3 + dy;
                        let idx = ((py * 6 + px) * 4) as usize;
                        pixel_data[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
        }

        let result = detect_upscale(&pixel_data, 6, 6);

        assert!(result.is_some(), "Should detect 3x upscale");
        let info = result.unwrap();
        assert_eq!(info.scale, 3);
        assert_eq!(info.native_size, [2, 2]);
    }

    #[test]
    fn test_detect_upscale_none_for_native() {
        // 4x4 image with no repeating pattern (native pixel art)
        let mut pixel_data = vec![0u8; 4 * 4 * 4];

        // Fill each pixel with a different color
        for y in 0..4 {
            for x in 0..4 {
                let idx = ((y * 4 + x) * 4) as usize;
                pixel_data[idx] = (x * 64) as u8;
                pixel_data[idx + 1] = (y * 64) as u8;
                pixel_data[idx + 2] = ((x + y) * 32) as u8;
                pixel_data[idx + 3] = 255;
            }
        }

        let result = detect_upscale(&pixel_data, 4, 4);

        assert!(result.is_none(), "Should not detect upscale for native pixel art");
    }

    #[test]
    fn test_detect_upscale_prefers_smaller_scale() {
        // 8x8 image that is 2x2 upscaled 4x
        // Should detect 4x, not 2x (which would also match but less efficiently)
        let mut pixel_data = vec![0u8; 8 * 8 * 4];

        // Fill with 4x4 blocks
        for block_y in 0..2 {
            for block_x in 0..2 {
                let color: [u8; 4] = match (block_x, block_y) {
                    (0, 0) => [255, 0, 0, 255],
                    (1, 0) => [0, 255, 0, 255],
                    (0, 1) => [0, 0, 255, 255],
                    _ => [255, 255, 255, 255],
                };

                for dy in 0..4 {
                    for dx in 0..4 {
                        let px = block_x * 4 + dx;
                        let py = block_y * 4 + dy;
                        let idx = ((py * 8 + px) * 4) as usize;
                        pixel_data[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
        }

        let result = detect_upscale(&pixel_data, 8, 8);

        assert!(result.is_some());
        let info = result.unwrap();
        // Should detect smallest valid scale (2x) first
        assert_eq!(info.scale, 2);
    }

    #[test]
    fn test_check_uniform_blocks() {
        // 4x4 with 2x2 uniform blocks
        let mut pixel_data = vec![0u8; 4 * 4 * 4];
        for i in 0..16 {
            let idx = i * 4;
            pixel_data[idx..idx + 4].copy_from_slice(&[128, 128, 128, 255]);
        }

        let confidence = check_uniform_blocks(&pixel_data, 4, 4, 2);
        assert!((confidence - 1.0).abs() < 0.001, "All blocks should be uniform");
    }

    #[test]
    fn test_check_uniform_blocks_partial() {
        // 4x4 with some non-uniform 2x2 blocks
        let mut pixel_data = vec![0u8; 4 * 4 * 4];

        // First three blocks are uniform gray
        for y in 0..4 {
            for x in 0..4 {
                let idx = ((y * 4 + x) * 4) as usize;
                pixel_data[idx..idx + 4].copy_from_slice(&[128, 128, 128, 255]);
            }
        }

        // Break uniformity in bottom-right block
        let idx = ((3 * 4 + 3) * 4) as usize;
        pixel_data[idx..idx + 4].copy_from_slice(&[255, 0, 0, 255]);

        let confidence = check_uniform_blocks(&pixel_data, 4, 4, 2);
        // 3 out of 4 blocks are uniform = 0.75
        assert!((confidence - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_upscale_info_in_structured_jsonl() {
        let mut palette = HashMap::new();
        palette.insert("{c1}".to_string(), "#FF0000".to_string());

        let mut regions = HashMap::new();
        regions.insert("{c1}".to_string(), vec![[0, 0]]);

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![],
            symmetry: None,
            naming_hints: vec![],
            z_order: HashMap::new(),
            dither_patterns: vec![],
            upscale_info: Some(UpscaleInfo {
                scale: 2,
                native_size: [16, 16],
                confidence: 0.98,
            }),
            outlines: vec![],
        };

        let result = ImportResult {
            name: "test".to_string(),
            width: 32,
            height: 32,
            palette,
            grid: vec![],
            regions,
            structured_regions: None,
            analysis: Some(analysis),
            half_sprite: false,
        };

        let jsonl = result.to_structured_jsonl();

        assert!(jsonl.contains("_upscale"), "Should include _upscale field");
        assert!(jsonl.contains("\"scale\":2") || jsonl.contains("\"scale\": 2"));
        assert!(jsonl.contains("[16,16]") || jsonl.contains("[16, 16]"));
    }

    #[test]
    fn test_detect_upscale_option_default() {
        let options = ImportOptions::default();
        assert!(!options.detect_upscale);
    }

    // ========================================================================
    // Outline Detection Tests (TTP-trwxq.8)
    // ========================================================================

    #[test]
    fn test_detect_outline_thin_dark_region() {
        // Create a 6x6 sprite with a 1px dark outline around a 4x4 colored region
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        // Outline pixels (dark, 1px border)
        let mut outline_pixels = HashSet::new();
        for x in 0..6 {
            outline_pixels.insert((x, 0)); // top
            outline_pixels.insert((x, 5)); // bottom
        }
        for y in 1..5 {
            outline_pixels.insert((0, y)); // left
            outline_pixels.insert((5, y)); // right
        }

        // Inner region
        let mut inner_pixels = HashSet::new();
        for y in 1..5 {
            for x in 1..5 {
                inner_pixels.insert((x, y));
            }
        }

        token_pixels.insert("{outline}".to_string(), outline_pixels);
        token_pixels.insert("{inner}".to_string(), inner_pixels);
        token_to_color.insert("{outline}".to_string(), [10, 10, 10, 255]); // Dark
        token_to_color.insert("{inner}".to_string(), [200, 100, 50, 255]); // Light

        let outlines = detect_outlines(&token_pixels, &token_to_color, 6, 6);

        assert!(!outlines.is_empty(), "Should detect outline");
        let outline = outlines.iter().find(|o| o.token == "{outline}");
        assert!(outline.is_some(), "Should find {{outline}} token as outline");
        let outline = outline.unwrap();
        assert!(outline.borders.contains(&"{inner}".to_string()));
    }

    #[test]
    fn test_detect_outline_skips_light_colors() {
        // Light colored thin region should not be detected as outline
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        let mut thin_pixels = HashSet::new();
        for y in 0..10 {
            thin_pixels.insert((0, y));
        }

        let mut adjacent_pixels = HashSet::new();
        for y in 0..10 {
            for x in 1..5 {
                adjacent_pixels.insert((x, y));
            }
        }

        token_pixels.insert("{thin}".to_string(), thin_pixels);
        token_pixels.insert("{adjacent}".to_string(), adjacent_pixels);
        token_to_color.insert("{thin}".to_string(), [255, 255, 200, 255]); // Light yellow
        token_to_color.insert("{adjacent}".to_string(), [100, 100, 100, 255]);

        let outlines = detect_outlines(&token_pixels, &token_to_color, 5, 10);

        assert!(outlines.is_empty(), "Should not detect light color as outline");
    }

    #[test]
    fn test_detect_outline_skips_thick_regions() {
        // Thick dark region should not be detected as outline
        let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut token_to_color: HashMap<String, [u8; 4]> = HashMap::new();

        // 5px thick dark region
        let mut thick_pixels = HashSet::new();
        for y in 0..10 {
            for x in 0..5 {
                thick_pixels.insert((x, y));
            }
        }

        let mut adjacent_pixels = HashSet::new();
        for y in 0..10 {
            for x in 5..10 {
                adjacent_pixels.insert((x, y));
            }
        }

        token_pixels.insert("{thick}".to_string(), thick_pixels);
        token_pixels.insert("{adjacent}".to_string(), adjacent_pixels);
        token_to_color.insert("{thick}".to_string(), [10, 10, 10, 255]); // Dark
        token_to_color.insert("{adjacent}".to_string(), [200, 200, 200, 255]);

        let outlines = detect_outlines(&token_pixels, &token_to_color, 10, 10);

        // Should not detect as outline since it's too thick
        let thick_outline = outlines.iter().find(|o| o.token == "{thick}");
        assert!(thick_outline.is_none(), "Should not detect thick region as outline");
    }

    #[test]
    fn test_calculate_average_width_single_line() {
        // Single vertical line (1px wide, 10px tall)
        // Note: The perimeter-based approximation gives ~2 for thin lines
        // because all pixels are perimeter pixels
        let mut pixels = HashSet::new();
        for y in 0..10 {
            pixels.insert((0, y));
        }

        let width = calculate_average_width(&pixels);
        // Thin lines (1-2px) get estimated as ~2 due to the approximation
        // This is fine for outline detection which uses a threshold of ~3.5
        assert!(width <= 3.0, "Single line should have width <= 3, got {}", width);
    }

    #[test]
    fn test_calculate_average_width_rectangle() {
        // 4x10 rectangle (should not be detected as thin)
        let mut pixels = HashSet::new();
        for y in 0..10 {
            for x in 0..4 {
                pixels.insert((x, y));
            }
        }

        let width = calculate_average_width(&pixels);
        assert!(width > 2.0, "4px wide rectangle should have width > 2, got {}", width);
    }

    #[test]
    fn test_find_bordered_regions() {
        let mut all_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();

        // Outline pixels (vertical line)
        let mut outline = HashSet::new();
        for y in 0..5 {
            outline.insert((2, y));
        }

        // Left region
        let mut left = HashSet::new();
        for y in 0..5 {
            for x in 0..2 {
                left.insert((x, y));
            }
        }

        // Right region
        let mut right = HashSet::new();
        for y in 0..5 {
            for x in 3..5 {
                right.insert((x, y));
            }
        }

        all_pixels.insert("{outline}".to_string(), outline.clone());
        all_pixels.insert("{left}".to_string(), left);
        all_pixels.insert("{right}".to_string(), right);

        let borders = find_bordered_regions("{outline}", &outline, &all_pixels);

        assert!(borders.contains(&"{left}".to_string()));
        assert!(borders.contains(&"{right}".to_string()));
    }

    #[test]
    fn test_outline_info_in_structured_jsonl() {
        let mut palette = HashMap::new();
        palette.insert("{outline}".to_string(), "#0A0A0A".to_string());
        palette.insert("{inner}".to_string(), "#FF0000".to_string());

        let mut regions = HashMap::new();
        regions.insert("{outline}".to_string(), vec![[0, 0], [1, 0]]);
        regions.insert("{inner}".to_string(), vec![[1, 1]]);

        let analysis = ImportAnalysis {
            roles: HashMap::new(),
            relationships: vec![],
            symmetry: None,
            naming_hints: vec![],
            z_order: HashMap::new(),
            dither_patterns: vec![],
            upscale_info: None,
            outlines: vec![OutlineInfo {
                token: "{outline}".to_string(),
                borders: vec!["{inner}".to_string()],
                width: 1.2,
                confidence: 0.85,
            }],
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
            half_sprite: false,
        };

        let jsonl = result.to_structured_jsonl();

        assert!(jsonl.contains("_outlines"), "Should include _outlines field");
        assert!(jsonl.contains("{outline}"), "Should include outline token");
    }

    #[test]
    fn test_detect_outlines_option_default() {
        let options = ImportOptions::default();
        assert!(!options.detect_outlines);
    }
}
