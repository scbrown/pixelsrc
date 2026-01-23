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
}
