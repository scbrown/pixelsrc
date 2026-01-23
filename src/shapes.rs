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
pub fn subtract(base: &HashSet<(i32, i32)>, removals: &[HashSet<(i32, i32)>]) -> HashSet<(i32, i32)> {
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

    // Scanline fill
    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        // Find intersections with edges
        for i in 0..vertices.len() {
            let j = (i + 1) % vertices.len();
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[j];

            // Skip horizontal edges
            if y1 == y2 {
                continue;
            }

            // Check if scanline intersects this edge (inclusive of both endpoints)
            let y_min = y1.min(y2);
            let y_max = y1.max(y2);
            if y >= y_min && y <= y_max {
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

// ============================================================================
// Fill Operations
// ============================================================================

/// Perform a flood fill operation starting from a seed point.
///
/// Fills all pixels reachable from the seed that are not part of the boundary
/// and are within the canvas bounds. Uses a standard 4-connected flood fill
/// algorithm (BFS).
///
/// # Arguments
/// * `boundary` - Set of pixels that cannot be filled (the boundary/walls)
/// * `seed` - Optional starting point. If None, auto-detects interior center
/// * `canvas_size` - (width, height) tuple for bounds checking
///
/// # Returns
/// A HashSet containing all filled pixels.
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::{flood_fill, rasterize_rect, rasterize_stroke};
/// use std::collections::HashSet;
///
/// // Create a hollow rectangle boundary
/// let boundary = rasterize_stroke(0, 0, 5, 5, 1);
///
/// // Fill from the center
/// let filled = flood_fill(&boundary, Some((2, 2)), (10, 10));
///
/// // Should fill the interior (3x3 = 9 pixels)
/// assert_eq!(filled.len(), 9);
/// assert!(filled.contains(&(2, 2)));
/// assert!(!filled.contains(&(0, 0))); // Boundary not filled
/// ```
pub fn flood_fill(
    boundary: &HashSet<(i32, i32)>,
    seed: Option<(i32, i32)>,
    canvas_size: (i32, i32),
) -> HashSet<(i32, i32)> {
    let (width, height) = canvas_size;

    // Auto-detect seed if not provided
    let seed = match seed {
        Some(s) => s,
        None => match find_interior_seed(boundary, canvas_size) {
            Some(s) => s,
            None => return HashSet::new(), // No valid seed found
        },
    };

    // Validate seed is within bounds and not on boundary
    if seed.0 < 0 || seed.0 >= width || seed.1 < 0 || seed.1 >= height {
        return HashSet::new();
    }
    if boundary.contains(&seed) {
        return HashSet::new();
    }

    // BFS flood fill
    let mut filled = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(seed);
    filled.insert(seed);

    while let Some((x, y)) = queue.pop_front() {
        // Check 4-connected neighbors
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;

            // Bounds check
            if nx < 0 || nx >= width || ny < 0 || ny >= height {
                continue;
            }

            let neighbor = (nx, ny);

            // Skip if already filled or on boundary
            if filled.contains(&neighbor) || boundary.contains(&neighbor) {
                continue;
            }

            filled.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    filled
}

/// Find an interior seed point for flood fill auto-detection.
///
/// Attempts to find a point inside the boundary by looking at the bounding box
/// center and searching outward if necessary.
fn find_interior_seed(
    boundary: &HashSet<(i32, i32)>,
    canvas_size: (i32, i32),
) -> Option<(i32, i32)> {
    if boundary.is_empty() {
        // No boundary - use canvas center
        let (width, height) = canvas_size;
        if width > 0 && height > 0 {
            return Some((width / 2, height / 2));
        }
        return None;
    }

    // Find bounding box of boundary
    let min_x = boundary.iter().map(|(x, _)| *x).min()?;
    let max_x = boundary.iter().map(|(x, _)| *x).max()?;
    let min_y = boundary.iter().map(|(_, y)| *y).min()?;
    let max_y = boundary.iter().map(|(_, y)| *y).max()?;

    // Start from center of bounding box and search for a non-boundary point
    let center_x = (min_x + max_x) / 2;
    let center_y = (min_y + max_y) / 2;

    let (width, height) = canvas_size;

    // Try center first
    if center_x >= 0
        && center_x < width
        && center_y >= 0
        && center_y < height
        && !boundary.contains(&(center_x, center_y))
    {
        return Some((center_x, center_y));
    }

    // Search in expanding squares from center
    let max_radius = ((max_x - min_x).max(max_y - min_y) / 2 + 1).max(1);
    for r in 1..=max_radius {
        for dx in -r..=r {
            for dy in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue; // Only check the perimeter of the square
                }
                let x = center_x + dx;
                let y = center_y + dy;
                if x >= 0 && x < width && y >= 0 && y < height && !boundary.contains(&(x, y)) {
                    return Some((x, y));
                }
            }
        }
    }

    None
}

/// Fill a region bounded by a boundary, excluding specified holes.
///
/// This is a convenience function that combines flood fill with hole exclusion.
/// The holes are added to the boundary before filling, so the fill will go
/// around them.
///
/// # Arguments
/// * `boundary` - The outer boundary of the region to fill
/// * `holes` - Regions to exclude from the fill (treated as additional boundaries)
/// * `seed` - Optional starting point. If None, auto-detects interior center
/// * `canvas_size` - (width, height) tuple for bounds checking
///
/// # Examples
///
/// ```
/// use pixelsrc::shapes::{flood_fill_with_holes, rasterize_stroke, rasterize_rect};
/// use std::collections::HashSet;
///
/// // Outer boundary (5x5 hollow rectangle)
/// let outer = rasterize_stroke(0, 0, 7, 7, 1);
///
/// // Inner hole (small filled rectangle)
/// let hole = rasterize_rect(3, 3, 1, 1);
///
/// // Fill with hole excluded
/// let filled = flood_fill_with_holes(&outer, &[hole], Some((1, 1)), (10, 10));
///
/// // The fill should not include the hole
/// assert!(!filled.contains(&(3, 3)));
/// ```
pub fn flood_fill_with_holes(
    boundary: &HashSet<(i32, i32)>,
    holes: &[HashSet<(i32, i32)>],
    seed: Option<(i32, i32)>,
    canvas_size: (i32, i32),
) -> HashSet<(i32, i32)> {
    // Combine boundary with all holes
    let mut combined_boundary = boundary.clone();
    for hole in holes {
        for pixel in hole {
            combined_boundary.insert(*pixel);
        }
    }

    flood_fill(&combined_boundary, seed, canvas_size)
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
        // Create a hollow rectangle boundary
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);

        // Fill from the center
        let filled = flood_fill(&boundary, Some((2, 2)), (10, 10));

        // Interior is 3x3 = 9 pixels
        assert_eq!(filled.len(), 9);
        assert!(filled.contains(&(1, 1)));
        assert!(filled.contains(&(2, 2)));
        assert!(filled.contains(&(3, 3)));

        // Boundary should not be filled
        assert!(!filled.contains(&(0, 0)));
        assert!(!filled.contains(&(4, 0)));
    }

    #[test]
    fn test_flood_fill_auto_seed() {
        // Create a hollow rectangle boundary
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);

        // Fill without explicit seed
        let filled = flood_fill(&boundary, None, (10, 10));

        // Should fill interior (auto-detect center)
        assert_eq!(filled.len(), 9);
        assert!(filled.contains(&(2, 2)));
    }

    #[test]
    fn test_flood_fill_empty_boundary() {
        let boundary = HashSet::new();

        // Fill entire canvas
        let filled = flood_fill(&boundary, Some((0, 0)), (3, 3));

        // Should fill all 9 pixels
        assert_eq!(filled.len(), 9);
    }

    #[test]
    fn test_flood_fill_seed_on_boundary() {
        let boundary: HashSet<(i32, i32)> = [(2, 2)].into_iter().collect();

        // Seed on boundary should return empty
        let filled = flood_fill(&boundary, Some((2, 2)), (5, 5));
        assert_eq!(filled.len(), 0);
    }

    #[test]
    fn test_flood_fill_seed_out_of_bounds() {
        let boundary = HashSet::new();

        // Seed outside canvas
        let filled = flood_fill(&boundary, Some((10, 10)), (5, 5));
        assert_eq!(filled.len(), 0);

        // Negative seed
        let filled = flood_fill(&boundary, Some((-1, 0)), (5, 5));
        assert_eq!(filled.len(), 0);
    }

    #[test]
    fn test_flood_fill_bounded_by_canvas() {
        // No boundary, but limited by canvas
        let boundary = HashSet::new();
        let filled = flood_fill(&boundary, Some((0, 0)), (3, 3));

        // Should fill only within 3x3 canvas
        assert_eq!(filled.len(), 9);
        assert!(!filled.contains(&(3, 0)));
        assert!(!filled.contains(&(0, 3)));
    }

    #[test]
    fn test_flood_fill_complex_boundary() {
        // Create an L-shaped boundary
        let mut boundary = HashSet::new();
        // Vertical part
        for y in 0..5 {
            boundary.insert((0, y));
        }
        // Horizontal part
        for x in 0..5 {
            boundary.insert((x, 4));
        }
        // Close the shape
        for y in 0..5 {
            boundary.insert((4, y));
        }
        boundary.insert((0, 0));

        // Fill from inside
        let filled = flood_fill(&boundary, Some((2, 2)), (10, 10));

        // Should fill inside the L
        assert!(filled.contains(&(2, 2)));
        assert!(filled.contains(&(1, 1)));

        // Should not fill boundary
        assert!(!filled.contains(&(0, 0)));
        assert!(!filled.contains(&(0, 4)));
    }

    #[test]
    fn test_flood_fill_with_holes_basic() {
        // Outer boundary (7x7 hollow rectangle)
        let outer = rasterize_stroke(0, 0, 7, 7, 1);

        // Inner hole (small filled rectangle in center)
        let hole = rasterize_rect(3, 3, 1, 1);

        // Fill with hole excluded
        let filled = flood_fill_with_holes(&outer, &[hole], Some((1, 1)), (10, 10));

        // The fill should not include the hole
        assert!(!filled.contains(&(3, 3)));

        // But should include surrounding interior
        assert!(filled.contains(&(1, 1)));
        assert!(filled.contains(&(2, 2)));
        assert!(filled.contains(&(4, 4)));
    }

    #[test]
    fn test_flood_fill_with_multiple_holes() {
        // Outer boundary
        let outer = rasterize_stroke(0, 0, 9, 9, 1);

        // Two holes
        let hole1: HashSet<(i32, i32)> = [(2, 2)].into_iter().collect();
        let hole2: HashSet<(i32, i32)> = [(6, 6)].into_iter().collect();

        let filled = flood_fill_with_holes(&outer, &[hole1, hole2], Some((4, 4)), (10, 10));

        // Neither hole should be filled
        assert!(!filled.contains(&(2, 2)));
        assert!(!filled.contains(&(6, 6)));

        // Interior should be filled
        assert!(filled.contains(&(4, 4)));
    }

    #[test]
    fn test_flood_fill_with_empty_holes() {
        let boundary = rasterize_stroke(0, 0, 5, 5, 1);

        // No holes
        let filled_no_holes = flood_fill_with_holes(&boundary, &[], Some((2, 2)), (10, 10));

        // Should be same as regular flood fill
        let filled_regular = flood_fill(&boundary, Some((2, 2)), (10, 10));

        assert_eq!(filled_no_holes, filled_regular);
    }

    #[test]
    fn test_flood_fill_disconnected_regions() {
        // Create two separate boundaries
        let mut boundary = HashSet::new();

        // First box
        for x in 0..3 {
            boundary.insert((x, 0));
            boundary.insert((x, 2));
        }
        boundary.insert((0, 1));
        boundary.insert((2, 1));

        // Second box (separate)
        for x in 5..8 {
            boundary.insert((x, 0));
            boundary.insert((x, 2));
        }
        boundary.insert((5, 1));
        boundary.insert((7, 1));

        // Fill first box
        let filled = flood_fill(&boundary, Some((1, 1)), (10, 10));

        // Should only fill first box interior
        assert!(filled.contains(&(1, 1)));
        assert!(!filled.contains(&(6, 1))); // Second box interior
    }

    #[test]
    fn test_flood_fill_single_pixel_interior() {
        // Create boundary with single pixel interior
        let mut boundary = HashSet::new();
        for x in 0..3 {
            boundary.insert((x, 0));
            boundary.insert((x, 2));
        }
        boundary.insert((0, 1));
        boundary.insert((2, 1));

        let filled = flood_fill(&boundary, Some((1, 1)), (10, 10));

        assert_eq!(filled.len(), 1);
        assert!(filled.contains(&(1, 1)));
    }
}
