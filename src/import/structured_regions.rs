//! Structured region extraction for PNG import.
//!
//! Converts raw point data into higher-level primitives like rectangles
//! and polygons.

use std::collections::{HashMap, HashSet};

use crate::analyze::Symmetric;

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

/// Extract structured regions from point arrays.
///
/// This converts raw point data into higher-level primitives:
/// - Rectangles for rectangular regions
/// - Polygons for irregular but contiguous regions
/// - Unions for multiple disconnected components
pub fn extract_structured_regions(
    points: &[[u32; 2]],
    _width: u32,
    _height: u32,
) -> StructuredRegion {
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
#[allow(dead_code)]
fn extract_polygon_boundary(component: &HashSet<(u32, u32)>) -> Option<Vec<[i32; 2]>> {
    if component.len() < 3 {
        return None;
    }

    // Find bounding box
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
pub(crate) fn rasterize_polygon(polygon: &[[i32; 2]]) -> HashSet<(u32, u32)> {
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
#[allow(dead_code)]
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

/// Filter points to only include the primary half based on symmetry.
///
/// For X symmetry (left-right mirror), keeps only the left half.
/// For Y symmetry (top-bottom mirror), keeps only the top half.
/// For XY symmetry, keeps only the top-left quarter.
///
/// Returns the filtered points.
pub fn filter_points_for_half_sprite(
    points: &[[u32; 2]],
    symmetry: Symmetric,
    width: u32,
    height: u32,
) -> Vec<[u32; 2]> {
    let half_width = width.div_ceil(2); // Include center column for odd widths
    let half_height = height.div_ceil(2); // Include center row for odd heights

    points
        .iter()
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
pub fn filter_structured_region_for_half_sprite(
    region: &StructuredRegion,
    symmetry: Symmetric,
    width: u32,
    height: u32,
) -> StructuredRegion {
    let half_width = width.div_ceil(2);
    let half_height = height.div_ceil(2);

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
            let points: Vec<[u32; 2]> = rasterized.into_iter().map(|(x, y)| [x, y]).collect();
            let filtered = filter_points_for_half_sprite(&points, symmetry, width, height);
            StructuredRegion::Points(filtered)
        }
        StructuredRegion::Union(regions) => {
            let filtered: Vec<StructuredRegion> = regions
                .iter()
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
