//! Shape rasterization primitives for pixel-perfect rendering.
//!
//! This module provides functions to convert geometric shapes into sets of
//! integer pixel coordinates using standard rasterization algorithms.

use std::collections::HashSet;

/// Rasterize a set of points.
///
/// Takes a collection of (x, y) coordinates and returns them as a HashSet.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::rasterize_points;
///
/// let points = vec![(0, 0), (1, 1), (2, 2)];
/// let pixels = rasterize_points(&points);
/// assert_eq!(pixels.len(), 3);
/// assert!(pixels.contains(&(1, 1)));
/// ```
pub fn rasterize_points(points: &[(i32, i32)]) -> HashSet<(i32, i32)> {
    points.iter().copied().collect()
}

/// Rasterize a line using Bresenham's line algorithm.
///
/// Returns all pixels that form a line between two points.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::rasterize_line;
///
/// let pixels = rasterize_line((0, 0), (3, 3));
/// assert_eq!(pixels.len(), 4);
/// assert!(pixels.contains(&(0, 0)));
/// assert!(pixels.contains(&(3, 3)));
/// ```
pub fn rasterize_line(p0: (i32, i32), p1: (i32, i32)) -> HashSet<(i32, i32)> {
    let mut pixels = HashSet::new();

    let (mut x0, mut y0) = p0;
    let (x1, y1) = p1;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        pixels.insert((x0, y0));

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    pixels
}

/// Rasterize a filled rectangle.
///
/// Returns all pixels within a rectangle defined by top-left corner (x, y)
/// and dimensions (w, h).
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::rasterize_rect;
///
/// let pixels = rasterize_rect(0, 0, 3, 2);
/// assert_eq!(pixels.len(), 6);
/// assert!(pixels.contains(&(0, 0)));
/// assert!(pixels.contains(&(2, 1)));
/// ```
pub fn rasterize_rect(x: i32, y: i32, w: i32, h: i32) -> HashSet<(i32, i32)> {
    let mut pixels = HashSet::new();

    if w <= 0 || h <= 0 {
        return pixels;
    }

    for dy in 0..h {
        for dx in 0..w {
            pixels.insert((x + dx, y + dy));
        }
    }

    pixels
}

/// Rasterize a stroked rectangle (outline only).
///
/// Returns pixels forming the outline of a rectangle with specified thickness.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::rasterize_stroke;
///
/// let pixels = rasterize_stroke(0, 0, 4, 4, 1);
/// assert!(pixels.contains(&(0, 0)));
/// assert!(pixels.contains(&(3, 0)));
/// assert!(!pixels.contains(&(1, 1))); // Interior should be empty
/// ```
pub fn rasterize_stroke(x: i32, y: i32, w: i32, h: i32, thickness: i32) -> HashSet<(i32, i32)> {
    let mut pixels = HashSet::new();

    if w <= 0 || h <= 0 || thickness <= 0 {
        return pixels;
    }

    // Top and bottom edges
    for dx in 0..w {
        for t in 0..thickness.min(h) {
            pixels.insert((x + dx, y + t)); // Top
            pixels.insert((x + dx, y + h - 1 - t)); // Bottom
        }
    }

    // Left and right edges
    for dy in 0..h {
        for t in 0..thickness.min(w) {
            pixels.insert((x + t, y + dy)); // Left
            pixels.insert((x + w - 1 - t, y + dy)); // Right
        }
    }

    pixels
}

/// Rasterize a filled ellipse using the midpoint algorithm.
///
/// Returns all pixels within an ellipse centered at (cx, cy) with radii (rx, ry).
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::rasterize_ellipse;
///
/// let pixels = rasterize_ellipse(5, 5, 3, 2);
/// assert!(pixels.contains(&(5, 5))); // Center
/// assert!(pixels.len() > 0);
/// ```
pub fn rasterize_ellipse(cx: i32, cy: i32, rx: i32, ry: i32) -> HashSet<(i32, i32)> {
    let mut pixels = HashSet::new();

    if rx <= 0 || ry <= 0 {
        return pixels;
    }

    // Convert to i64 to avoid overflow in calculations
    let rx = rx as i64;
    let ry = ry as i64;
    let cx = cx as i64;
    let cy = cy as i64;

    // Region 1
    let mut x = 0i64;
    let mut y = ry;

    let rx_sq = rx * rx;
    let ry_sq = ry * ry;

    let mut p1 = ry_sq - (rx_sq * ry) + (rx_sq / 4);
    let mut dx = 2 * ry_sq * x;
    let mut dy = 2 * rx_sq * y;

    // Region 1 decision parameter
    while dx < dy {
        draw_ellipse_points(cx, cy, x, y, &mut pixels);

        if p1 < 0 {
            x += 1;
            dx += 2 * ry_sq;
            p1 += dx + ry_sq;
        } else {
            x += 1;
            y -= 1;
            dx += 2 * ry_sq;
            dy -= 2 * rx_sq;
            p1 += dx - dy + ry_sq;
        }
    }

    // Region 2 decision parameter
    let mut p2 = ry_sq * (x + 1) * (x + 1) / 4 + rx_sq * (y - 1) * (y - 1) - rx_sq * ry_sq;

    while y >= 0 {
        draw_ellipse_points(cx, cy, x, y, &mut pixels);

        if p2 > 0 {
            y -= 1;
            dy -= 2 * rx_sq;
            p2 += rx_sq - dy;
        } else {
            x += 1;
            y -= 1;
            dx += 2 * ry_sq;
            dy -= 2 * rx_sq;
            p2 += dx - dy + rx_sq;
        }
    }

    pixels
}

// ============================================================================
// Compound Operations
// ============================================================================

/// Combine all pixels from multiple regions into a single set.
///
/// Returns the union of all input regions. Empty regions are skipped.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::union;
/// use std::collections::HashSet;
///
/// let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
/// let region2: HashSet<(i32, i32)> = [(1, 0), (2, 0)].into_iter().collect();
/// let result = union(&[region1, region2]);
/// assert_eq!(result.len(), 3);
/// assert!(result.contains(&(0, 0)));
/// assert!(result.contains(&(1, 0)));
/// assert!(result.contains(&(2, 0)));
/// ```
pub fn union(regions: &[HashSet<(i32, i32)>]) -> HashSet<(i32, i32)> {
    let mut result = HashSet::new();
    for region in regions {
        for pixel in region {
            result.insert(*pixel);
        }
    }
    result
}

/// Remove pixels from a base region that appear in any of the removal regions.
///
/// Returns pixels that are in `base` but not in any of `removals`.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::subtract;
/// use std::collections::HashSet;
///
/// let base: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0)].into_iter().collect();
/// let remove: HashSet<(i32, i32)> = [(1, 0)].into_iter().collect();
/// let result = subtract(&base, &[remove]);
/// assert_eq!(result.len(), 2);
/// assert!(result.contains(&(0, 0)));
/// assert!(result.contains(&(2, 0)));
/// assert!(!result.contains(&(1, 0)));
/// ```
pub fn subtract(
    base: &HashSet<(i32, i32)>,
    removals: &[HashSet<(i32, i32)>],
) -> HashSet<(i32, i32)> {
    let mut result = base.clone();
    for removal in removals {
        for pixel in removal {
            result.remove(pixel);
        }
    }
    result
}

/// Find pixels that appear in all input regions.
///
/// Returns the intersection of all input regions. If no regions are provided,
/// returns an empty set. If only one region is provided, returns a copy of it.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::intersect;
/// use std::collections::HashSet;
///
/// let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0)].into_iter().collect();
/// let region2: HashSet<(i32, i32)> = [(1, 0), (2, 0), (3, 0)].into_iter().collect();
/// let result = intersect(&[region1, region2]);
/// assert_eq!(result.len(), 2);
/// assert!(result.contains(&(1, 0)));
/// assert!(result.contains(&(2, 0)));
/// ```
pub fn intersect(regions: &[HashSet<(i32, i32)>]) -> HashSet<(i32, i32)> {
    if regions.is_empty() {
        return HashSet::new();
    }

    let mut result = regions[0].clone();
    for region in &regions[1..] {
        result.retain(|pixel| region.contains(pixel));
    }
    result
}

// ============================================================================
// Fill Operations
// ============================================================================

/// Perform a flood fill operation bounded by a boundary region.
///
/// Fills all pixels reachable from the seed point that are not part of the
/// boundary, staying within the specified canvas bounds. Uses 4-connectivity
/// (cardinal directions only).
///
/// If `seed` is `None`, the function attempts to auto-detect an interior point
/// by finding the centroid of the boundary's bounding box and searching for
/// a valid starting point.
///
/// # Arguments
///
/// * `boundary` - Set of pixels that act as walls for the flood fill
/// * `seed` - Optional starting point; if None, auto-detects interior point
/// * `canvas_width` - Width of the canvas (x range: 0..canvas_width)
/// * `canvas_height` - Height of the canvas (y range: 0..canvas_height)
///
/// # Returns
///
/// A HashSet of all filled pixels (excluding the boundary itself).
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::{rasterize_stroke, flood_fill};
///
/// // Create a hollow rectangle as boundary
/// let boundary = rasterize_stroke(1, 1, 5, 5, 1);
/// // Fill from the center
/// let filled = flood_fill(&boundary, Some((3, 3)), 10, 10);
/// assert!(filled.contains(&(2, 2)));
/// assert!(filled.contains(&(3, 3)));
/// assert!(!filled.contains(&(1, 1))); // Boundary not included
/// ```
pub fn flood_fill(
    boundary: &HashSet<(i32, i32)>,
    seed: Option<(i32, i32)>,
    canvas_width: i32,
    canvas_height: i32,
) -> HashSet<(i32, i32)> {
    let mut filled = HashSet::new();

    if canvas_width <= 0 || canvas_height <= 0 {
        return filled;
    }

    // Determine seed point
    let seed_point = match seed {
        Some(s) => s,
        None => match find_interior_seed(boundary, canvas_width, canvas_height) {
            Some(s) => s,
            None => return filled, // No valid seed found
        },
    };

    // Validate seed is within bounds and not on boundary
    if !is_valid_fill_point(seed_point, boundary, canvas_width, canvas_height) {
        return filled;
    }

    // BFS flood fill
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(seed_point);
    filled.insert(seed_point);

    while let Some((x, y)) = queue.pop_front() {
        // 4-connectivity: check cardinal directions
        let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];

        for neighbor in neighbors {
            if is_valid_fill_point(neighbor, boundary, canvas_width, canvas_height)
                && !filled.contains(&neighbor)
            {
                filled.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    filled
}

/// Perform a flood fill with exclusion regions (holes).
///
/// Similar to `flood_fill`, but also excludes pixels in the `except` regions
/// from being filled. This is useful for filling around holes or other
/// obstacles within a bounded region.
///
/// # Arguments
///
/// * `boundary` - Set of pixels that act as walls for the flood fill
/// * `except` - Additional regions to exclude from filling (treated as obstacles)
/// * `seed` - Optional starting point; if None, auto-detects interior point
/// * `canvas_width` - Width of the canvas
/// * `canvas_height` - Height of the canvas
///
/// # Returns
///
/// A HashSet of all filled pixels (excluding boundary and except regions).
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::{rasterize_stroke, rasterize_rect, flood_fill_except};
///
/// // Create a hollow rectangle as boundary
/// let boundary = rasterize_stroke(0, 0, 10, 10, 1);
/// // Create a hole in the middle
/// let hole = rasterize_rect(4, 4, 2, 2);
/// // Fill around the hole
/// let filled = flood_fill_except(&boundary, &[hole], Some((2, 2)), 10, 10);
/// assert!(filled.contains(&(2, 2)));
/// assert!(!filled.contains(&(4, 4))); // Hole not filled
/// ```
pub fn flood_fill_except(
    boundary: &HashSet<(i32, i32)>,
    except: &[HashSet<(i32, i32)>],
    seed: Option<(i32, i32)>,
    canvas_width: i32,
    canvas_height: i32,
) -> HashSet<(i32, i32)> {
    // Combine boundary with all except regions
    let mut combined_boundary = boundary.clone();
    for region in except {
        for pixel in region {
            combined_boundary.insert(*pixel);
        }
    }

    flood_fill(&combined_boundary, seed, canvas_width, canvas_height)
}

/// Check if a point is valid for flood fill.
fn is_valid_fill_point(
    point: (i32, i32),
    boundary: &HashSet<(i32, i32)>,
    canvas_width: i32,
    canvas_height: i32,
) -> bool {
    let (x, y) = point;
    x >= 0 && x < canvas_width && y >= 0 && y < canvas_height && !boundary.contains(&point)
}

/// Find a valid interior seed point by analyzing the boundary.
///
/// Attempts to find the center of the boundary's bounding box, then searches
/// outward in a spiral pattern if the center is blocked.
fn find_interior_seed(
    boundary: &HashSet<(i32, i32)>,
    canvas_width: i32,
    canvas_height: i32,
) -> Option<(i32, i32)> {
    if boundary.is_empty() {
        // No boundary means fill entire canvas from origin
        if canvas_width > 0 && canvas_height > 0 {
            return Some((0, 0));
        }
        return None;
    }

    // Find bounding box of boundary
    let min_x = boundary.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = boundary.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = boundary.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = boundary.iter().map(|(_, y)| *y).max().unwrap();

    // Start from center of bounding box
    let center_x = (min_x + max_x) / 2;
    let center_y = (min_y + max_y) / 2;

    // Try center first
    if is_valid_fill_point((center_x, center_y), boundary, canvas_width, canvas_height) {
        return Some((center_x, center_y));
    }

    // Search outward in a spiral pattern
    let max_radius = (max_x - min_x).max(max_y - min_y);
    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() == radius || dy.abs() == radius {
                    let point = (center_x + dx, center_y + dy);
                    if is_valid_fill_point(point, boundary, canvas_width, canvas_height) {
                        return Some(point);
                    }
                }
            }
        }
    }

    None
}

/// Helper function to draw ellipse points with fill.
fn draw_ellipse_points(cx: i64, cy: i64, x: i64, y: i64, pixels: &mut HashSet<(i32, i32)>) {
    // Fill horizontal lines for each y-coordinate
    for scan_x in -x..=x {
        pixels.insert(((cx + scan_x) as i32, (cy + y) as i32));
        pixels.insert(((cx + scan_x) as i32, (cy - y) as i32));
    }
}

/// Rasterize a filled polygon using scanline fill algorithm.
///
/// Returns all pixels within a polygon defined by a list of vertices.
/// Uses an even-odd fill rule.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::rasterize_polygon;
///
/// let triangle = vec![(0, 0), (4, 0), (2, 3)];
/// let pixels = rasterize_polygon(&triangle);
/// assert!(pixels.len() > 0);
/// assert!(pixels.contains(&(2, 1)));
/// ```
pub fn rasterize_polygon(vertices: &[(i32, i32)]) -> HashSet<(i32, i32)> {
    let mut pixels = HashSet::new();

    if vertices.len() < 3 {
        return pixels;
    }

    // Find bounding box
    let min_y = vertices.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = vertices.iter().map(|(_, y)| *y).max().unwrap();

    // First, add all vertices to ensure they're included
    // (vertices on the boundary should always be part of the filled polygon)
    for &(x, y) in vertices {
        pixels.insert((x, y));
    }

    // Scanline fill
    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        // Find intersections with edges
        for i in 0..vertices.len() {
            let j = (i + 1) % vertices.len();
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[j];

            // Skip horizontal edges (handled separately below)
            if y1 == y2 {
                // For horizontal edges at this y, fill the entire segment
                if y1 == y {
                    let x_min = x1.min(x2);
                    let x_max = x1.max(x2);
                    for x in x_min..=x_max {
                        pixels.insert((x, y));
                    }
                }
                continue;
            }

            // Check if scanline intersects this edge
            // IMPORTANT: Use exclusive upper bound to avoid double-counting vertices.
            // When a scanline passes through a vertex where two edges meet, we want
            // to count that intersection exactly once, not twice.
            // By excluding the maximum y (upper endpoint), each vertex is counted
            // only by the edge for which it is the lower endpoint.
            let y_min = y1.min(y2);
            let y_max = y1.max(y2);
            if y >= y_min && y < y_max {
                let x = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                intersections.push(x);
            }
        }

        // Sort intersections
        intersections.sort_unstable();

        // Fill between pairs of intersections
        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                let x_start = chunk[0];
                let x_end = chunk[1];
                for x in x_start..=x_end {
                    pixels.insert((x, y));
                }
            }
        }
    }

    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rasterize_points() {
        let points = vec![(0, 0), (1, 1), (2, 2)];
        let pixels = rasterize_points(&points);
        assert_eq!(pixels.len(), 3);
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(1, 1)));
        assert!(pixels.contains(&(2, 2)));
    }

    #[test]
    fn test_rasterize_points_duplicates() {
        let points = vec![(0, 0), (0, 0), (1, 1)];
        let pixels = rasterize_points(&points);
        assert_eq!(pixels.len(), 2);
    }

    #[test]
    fn test_rasterize_line_horizontal() {
        let pixels = rasterize_line((0, 0), (3, 0));
        assert_eq!(pixels.len(), 4);
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(1, 0)));
        assert!(pixels.contains(&(2, 0)));
        assert!(pixels.contains(&(3, 0)));
    }

    #[test]
    fn test_rasterize_line_vertical() {
        let pixels = rasterize_line((0, 0), (0, 3));
        assert_eq!(pixels.len(), 4);
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(0, 1)));
        assert!(pixels.contains(&(0, 2)));
        assert!(pixels.contains(&(0, 3)));
    }

    #[test]
    fn test_rasterize_line_diagonal() {
        let pixels = rasterize_line((0, 0), (3, 3));
        assert_eq!(pixels.len(), 4);
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(1, 1)));
        assert!(pixels.contains(&(2, 2)));
        assert!(pixels.contains(&(3, 3)));
    }

    #[test]
    fn test_rasterize_line_negative_coords() {
        let pixels = rasterize_line((-2, -2), (2, 2));
        assert!(pixels.contains(&(-2, -2)));
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(2, 2)));
    }

    #[test]
    fn test_rasterize_rect() {
        let pixels = rasterize_rect(0, 0, 3, 2);
        assert_eq!(pixels.len(), 6);
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(1, 0)));
        assert!(pixels.contains(&(2, 0)));
        assert!(pixels.contains(&(0, 1)));
        assert!(pixels.contains(&(1, 1)));
        assert!(pixels.contains(&(2, 1)));
    }

    #[test]
    fn test_rasterize_rect_zero_size() {
        let pixels = rasterize_rect(0, 0, 0, 2);
        assert_eq!(pixels.len(), 0);

        let pixels = rasterize_rect(0, 0, 2, 0);
        assert_eq!(pixels.len(), 0);
    }

    #[test]
    fn test_rasterize_rect_negative_coords() {
        let pixels = rasterize_rect(-2, -2, 2, 2);
        assert_eq!(pixels.len(), 4);
        assert!(pixels.contains(&(-2, -2)));
        assert!(pixels.contains(&(-1, -1)));
    }

    #[test]
    fn test_rasterize_stroke_basic() {
        let pixels = rasterize_stroke(0, 0, 4, 4, 1);

        // Check corners
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(3, 0)));
        assert!(pixels.contains(&(0, 3)));
        assert!(pixels.contains(&(3, 3)));

        // Check interior is empty (except edges)
        assert!(!pixels.contains(&(1, 1)));
        assert!(!pixels.contains(&(2, 2)));
    }

    #[test]
    fn test_rasterize_stroke_thick() {
        let pixels = rasterize_stroke(0, 0, 5, 5, 2);

        // Should have thicker edges
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(1, 0)));
        assert!(pixels.contains(&(0, 1)));

        // Center should still be empty
        assert!(!pixels.contains(&(2, 2)));
    }

    #[test]
    fn test_rasterize_stroke_zero_thickness() {
        let pixels = rasterize_stroke(0, 0, 4, 4, 0);
        assert_eq!(pixels.len(), 0);
    }

    #[test]
    fn test_rasterize_ellipse_circle() {
        let pixels = rasterize_ellipse(5, 5, 3, 3);
        assert!(pixels.contains(&(5, 5))); // Center
        assert!(pixels.len() > 0);
    }

    #[test]
    fn test_rasterize_ellipse_horizontal() {
        let pixels = rasterize_ellipse(5, 5, 4, 2);
        assert!(pixels.contains(&(5, 5))); // Center
        assert!(pixels.len() > 0);
    }

    #[test]
    fn test_rasterize_ellipse_zero_radius() {
        let pixels = rasterize_ellipse(5, 5, 0, 3);
        assert_eq!(pixels.len(), 0);

        let pixels = rasterize_ellipse(5, 5, 3, 0);
        assert_eq!(pixels.len(), 0);
    }

    #[test]
    fn test_rasterize_polygon_triangle() {
        let triangle = vec![(0, 0), (4, 0), (2, 3)];
        let pixels = rasterize_polygon(&triangle);

        assert!(pixels.len() > 0);

        // Check base
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(4, 0)));

        // Check interior point
        assert!(pixels.contains(&(2, 1)));
    }

    #[test]
    fn test_rasterize_polygon_square() {
        let square = vec![(0, 0), (3, 0), (3, 3), (0, 3)];
        let pixels = rasterize_polygon(&square);

        assert_eq!(pixels.len(), 16);

        // Check corners
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(3, 0)));
        assert!(pixels.contains(&(3, 3)));
        assert!(pixels.contains(&(0, 3)));

        // Check interior
        assert!(pixels.contains(&(1, 1)));
        assert!(pixels.contains(&(2, 2)));
    }

    #[test]
    fn test_rasterize_polygon_too_few_vertices() {
        let pixels = rasterize_polygon(&[(0, 0), (1, 1)]);
        assert_eq!(pixels.len(), 0);
    }

    #[test]
    fn test_rasterize_polygon_negative_coords() {
        let triangle = vec![(-2, -2), (2, -2), (0, 2)];
        let pixels = rasterize_polygon(&triangle);
        assert!(pixels.len() > 0);
        assert!(pixels.contains(&(0, 0)));
    }

    #[test]
    fn test_rasterize_polygon_hexagon_no_gaps() {
        // Hexagon with 6 vertices - should have no horizontal gaps/stripes
        // This tests the scanline algorithm's vertex handling
        let hexagon = vec![
            (5, 0), // top
            (9, 2), // upper right
            (9, 6), // lower right
            (5, 8), // bottom
            (1, 6), // lower left
            (1, 2), // upper left
        ];
        let pixels = rasterize_polygon(&hexagon);

        // Check that every row from y=0 to y=8 has at least some pixels
        // A gap/stripe bug would result in missing rows
        for y in 0..=8 {
            let row_pixels: Vec<_> = pixels.iter().filter(|(_, py)| *py == y).collect();
            assert!(
                !row_pixels.is_empty(),
                "Row y={} has no pixels - stripe artifact detected!",
                y
            );
        }
    }

    #[test]
    fn test_rasterize_polygon_convex_no_stripes() {
        // Pentagon (5 vertices) - another test for vertex handling
        let pentagon = vec![
            (5, 0),  // top
            (10, 4), // right
            (8, 10), // bottom right
            (2, 10), // bottom left
            (0, 4),  // left
        ];
        let pixels = rasterize_polygon(&pentagon);

        // Every row from min_y to max_y should have pixels
        for y in 0..=10 {
            let row_pixels: Vec<_> = pixels.iter().filter(|(_, py)| *py == y).collect();
            assert!(
                !row_pixels.is_empty(),
                "Row y={} has no pixels - stripe artifact detected!",
                y
            );
        }
    }

    #[test]
    fn test_rasterize_polygon_vertex_double_count() {
        // This test checks for the classic scanline bug where vertices
        // are double-counted (once for each edge meeting at the vertex).
        // The bug manifests when a scanline passes through a vertex where
        // edges have different slopes.
        //
        // Diamond shape where scanline at y=2 passes through vertex:
        //     *     (2,0)
        //    / \
        //   *   *   (0,2) and (4,2) - these are the critical vertices
        //    \ /
        //     *     (2,4)
        let diamond = vec![
            (2, 0), // top
            (4, 2), // right
            (2, 4), // bottom
            (0, 2), // left
        ];
        let pixels = rasterize_polygon(&diamond);

        // Row y=2 passes through vertices at (0,2) and (4,2)
        // With correct handling, pixels should be filled from x=0 to x=4
        // With double-counting bug, might get no fill or wrong fill
        let row2_pixels: Vec<_> = pixels.iter().filter(|(_, y)| *y == 2).collect();
        assert!(
            row2_pixels.len() >= 5,
            "Row y=2 should have at least 5 pixels (0..=4), got {} pixels: {:?}",
            row2_pixels.len(),
            row2_pixels
        );

        // Verify specific pixels in row 2
        assert!(pixels.contains(&(0, 2)), "Missing left vertex pixel at (0,2)");
        assert!(pixels.contains(&(2, 2)), "Missing center pixel at (2,2)");
        assert!(pixels.contains(&(4, 2)), "Missing right vertex pixel at (4,2)");
    }

    #[test]
    fn test_rasterize_polygon_local_minmax_vertices() {
        // Test vertices that are local minima/maxima in y-direction
        // These require special handling in scanline algorithms
        //
        //       * (5, 0) - local max
        //      / \
        //     /   \
        //    * (0,5) * (10,5)
        //     \   /
        //      \ /
        //       * (5, 10) - local min
        let shape = vec![
            (5, 0),  // top (local max)
            (10, 5), // right
            (5, 10), // bottom (local min)
            (0, 5),  // left
        ];
        let pixels = rasterize_polygon(&shape);

        // Every row should have pixels
        for y in 0..=10 {
            let row_count = pixels.iter().filter(|(_, py)| *py == y).count();
            assert!(row_count > 0, "Row y={} is empty - scanline vertex handling bug!", y);
        }

        // The widest row should be y=5 (through both side vertices)
        let row5_count = pixels.iter().filter(|(_, y)| *y == 5).count();
        assert!(
            row5_count >= 11,
            "Row y=5 should span x=0 to x=10 (11 pixels), got {} pixels",
            row5_count
        );
    }

    // ========================================================================
    // Compound Operations Tests
    // ========================================================================

    #[test]
    fn test_union_basic() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(1, 0), (2, 0)].into_iter().collect();
        let result = union(&[region1, region2]);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&(0, 0)));
        assert!(result.contains(&(1, 0)));
        assert!(result.contains(&(2, 0)));
    }

    #[test]
    fn test_union_no_overlap() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(5, 5), (6, 5)].into_iter().collect();
        let result = union(&[region1, region2]);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_union_full_overlap() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let result = union(&[region1, region2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_union_empty_input() {
        let result: HashSet<(i32, i32)> = union(&[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_union_single_region() {
        let region: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let result = union(&[region]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_union_with_empty_region() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = HashSet::new();
        let result = union(&[region1, region2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_union_multiple_regions() {
        let region1: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(1, 0)].into_iter().collect();
        let region3: HashSet<(i32, i32)> = [(2, 0)].into_iter().collect();
        let result = union(&[region1, region2, region3]);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_subtract_basic() {
        let base: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0)].into_iter().collect();
        let remove: HashSet<(i32, i32)> = [(1, 0)].into_iter().collect();
        let result = subtract(&base, &[remove]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&(0, 0)));
        assert!(result.contains(&(2, 0)));
        assert!(!result.contains(&(1, 0)));
    }

    #[test]
    fn test_subtract_no_overlap() {
        let base: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let remove: HashSet<(i32, i32)> = [(5, 5)].into_iter().collect();
        let result = subtract(&base, &[remove]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_subtract_full_removal() {
        let base: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let remove: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let result = subtract(&base, &[remove]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_subtract_empty_base() {
        let base: HashSet<(i32, i32)> = HashSet::new();
        let remove: HashSet<(i32, i32)> = [(1, 0)].into_iter().collect();
        let result = subtract(&base, &[remove]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_subtract_empty_removals() {
        let base: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let result = subtract(&base, &[]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_subtract_with_empty_removal() {
        let base: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let remove: HashSet<(i32, i32)> = HashSet::new();
        let result = subtract(&base, &[remove]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_subtract_multiple_removals() {
        let base: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
        let remove1: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
        let remove2: HashSet<(i32, i32)> = [(3, 0)].into_iter().collect();
        let result = subtract(&base, &[remove1, remove2]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&(1, 0)));
        assert!(result.contains(&(2, 0)));
    }

    #[test]
    fn test_intersect_basic() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(1, 0), (2, 0), (3, 0)].into_iter().collect();
        let result = intersect(&[region1, region2]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&(1, 0)));
        assert!(result.contains(&(2, 0)));
    }

    #[test]
    fn test_intersect_no_overlap() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(5, 5), (6, 5)].into_iter().collect();
        let result = intersect(&[region1, region2]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_intersect_full_overlap() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let result = intersect(&[region1, region2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_intersect_empty_input() {
        let result: HashSet<(i32, i32)> = intersect(&[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_intersect_single_region() {
        let region: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let result = intersect(&[region]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_intersect_with_empty_region() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = HashSet::new();
        let result = intersect(&[region1, region2]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_intersect_multiple_regions() {
        let region1: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0)].into_iter().collect();
        let region2: HashSet<(i32, i32)> = [(1, 0), (2, 0), (3, 0)].into_iter().collect();
        let region3: HashSet<(i32, i32)> = [(2, 0), (3, 0), (4, 0)].into_iter().collect();
        let result = intersect(&[region1, region2, region3]);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&(2, 0)));
    }

    // ========================================================================
    // Flood Fill Tests
    // ========================================================================

    #[test]
    fn test_flood_fill_basic() {
        // Create a hollow rectangle as boundary (5x5 with 1px border)
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);
        // Fill from the center
        let filled = flood_fill(&boundary, Some((2, 2)), 10, 10);

        // Should fill the interior (3x3 = 9 pixels)
        assert_eq!(filled.len(), 9);
        assert!(filled.contains(&(1, 1)));
        assert!(filled.contains(&(2, 2)));
        assert!(filled.contains(&(3, 3)));

        // Should not include boundary
        assert!(!filled.contains(&(0, 0)));
        assert!(!filled.contains(&(4, 4)));
    }

    #[test]
    fn test_flood_fill_auto_seed() {
        // Create a hollow rectangle
        let boundary = rasterize_stroke(1, 1, 5, 5, 1);
        // Auto-detect seed
        let filled = flood_fill(&boundary, None, 10, 10);

        // Should find interior and fill it
        assert!(filled.len() > 0);
        assert!(filled.contains(&(3, 3))); // Center of rectangle
    }

    #[test]
    fn test_flood_fill_seed_on_boundary() {
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);
        // Seed is on the boundary
        let filled = flood_fill(&boundary, Some((0, 0)), 10, 10);

        // Should return empty since seed is invalid
        assert_eq!(filled.len(), 0);
    }

    #[test]
    fn test_flood_fill_seed_outside_canvas() {
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);
        // Seed is outside canvas
        let filled = flood_fill(&boundary, Some((15, 15)), 10, 10);

        assert_eq!(filled.len(), 0);
    }

    #[test]
    fn test_flood_fill_empty_boundary() {
        let boundary: HashSet<(i32, i32)> = HashSet::new();
        let filled = flood_fill(&boundary, Some((0, 0)), 5, 5);

        // Should fill entire canvas (5x5 = 25 pixels)
        assert_eq!(filled.len(), 25);
    }

    #[test]
    fn test_flood_fill_zero_canvas() {
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);
        let filled = flood_fill(&boundary, Some((2, 2)), 0, 0);

        assert_eq!(filled.len(), 0);
    }

    #[test]
    fn test_flood_fill_bounded_by_canvas() {
        // No boundary, but canvas limits fill
        let boundary: HashSet<(i32, i32)> = HashSet::new();
        let filled = flood_fill(&boundary, Some((0, 0)), 3, 3);

        // Should fill exactly 3x3 = 9 pixels
        assert_eq!(filled.len(), 9);
        assert!(filled.contains(&(0, 0)));
        assert!(filled.contains(&(2, 2)));
        assert!(!filled.contains(&(3, 3)));
    }

    #[test]
    fn test_flood_fill_except_basic() {
        // Create a hollow rectangle as boundary
        let boundary = rasterize_stroke(0, 0, 7, 7, 1);
        // Create a hole in the middle
        let hole = rasterize_rect(3, 3, 1, 1);
        // Fill around the hole
        let filled = flood_fill_except(&boundary, &[hole], Some((1, 1)), 10, 10);

        // Should fill interior except the hole
        assert!(filled.contains(&(1, 1)));
        assert!(filled.contains(&(2, 2)));
        assert!(!filled.contains(&(3, 3))); // Hole

        // Hole should not be filled
        assert!(!filled.contains(&(3, 3)));
    }

    #[test]
    fn test_flood_fill_except_multiple_holes() {
        let boundary = rasterize_stroke(0, 0, 10, 10, 1);
        let hole1 = rasterize_rect(2, 2, 1, 1);
        let hole2 = rasterize_rect(5, 5, 1, 1);
        let filled = flood_fill_except(&boundary, &[hole1, hole2], Some((1, 1)), 10, 10);

        // Holes should not be filled
        assert!(!filled.contains(&(2, 2)));
        assert!(!filled.contains(&(5, 5)));

        // Other interior should be filled
        assert!(filled.contains(&(1, 1)));
        assert!(filled.contains(&(4, 4)));
    }

    #[test]
    fn test_flood_fill_except_empty_holes() {
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);
        let filled = flood_fill_except(&boundary, &[], Some((2, 2)), 10, 10);

        // Should be same as regular flood fill
        let regular = flood_fill(&boundary, Some((2, 2)), 10, 10);
        assert_eq!(filled.len(), regular.len());
    }

    #[test]
    fn test_flood_fill_l_shaped_region() {
        // Create an L-shaped boundary
        let mut boundary: HashSet<(i32, i32)> = HashSet::new();
        // Vertical part of L
        for y in 0..=5 {
            boundary.insert((0, y));
            boundary.insert((2, y));
        }
        // Horizontal part of L
        for x in 0..=5 {
            boundary.insert((x, 5));
        }
        // Close the L
        boundary.insert((1, 0));
        for y in 0..=2 {
            boundary.insert((5, y));
        }
        for x in 2..=5 {
            boundary.insert((x, 2));
        }

        // Fill from inside
        let filled = flood_fill(&boundary, Some((1, 1)), 10, 10);

        // Should fill the narrow corridor
        assert!(filled.contains(&(1, 1)));
    }

    #[test]
    fn test_flood_fill_disconnected_region() {
        // Create two separate rectangles
        let mut boundary: HashSet<(i32, i32)> = HashSet::new();
        // First rectangle
        for x in 0..=3 {
            boundary.insert((x, 0));
            boundary.insert((x, 3));
        }
        for y in 0..=3 {
            boundary.insert((0, y));
            boundary.insert((3, y));
        }

        // Fill only reaches the region containing the seed
        let filled = flood_fill(&boundary, Some((1, 1)), 10, 10);

        // Should only fill the interior of the first rectangle
        assert!(filled.contains(&(1, 1)));
        assert!(filled.contains(&(2, 2)));

        // Should not reach outside
        assert!(!filled.contains(&(5, 5)));
    }
}
