//! Shape detection for sprite analysis

use std::collections::HashSet;

use crate::shapes;

/// Result of shape detection with confidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeDetection<T> {
    /// The detected shape parameters
    pub shape: T,
    /// Confidence score from 0.0 to 1.0
    pub confidence: f64,
}

impl<T> ShapeDetection<T> {
    /// Create a new shape detection result.
    pub fn new(shape: T, confidence: f64) -> Self {
        Self { shape, confidence: confidence.clamp(0.0, 1.0) }
    }
}

/// Detected shape type with parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedShape {
    /// Filled rectangle: [x, y, width, height]
    Rect([i32; 4]),
    /// Stroked (hollow) rectangle: [x, y, width, height]
    Stroke([i32; 4]),
    /// Ellipse: [cx, cy, rx, ry]
    Ellipse([i32; 4]),
    /// Line defined by endpoints
    Line(Vec<[i32; 2]>),
    /// Polygon defined by vertices (fallback)
    Polygon(Vec<[i32; 2]>),
}

/// Represents the type of symmetry detected in a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symmetric {
    /// Symmetric along the X-axis (left-right mirroring)
    X,
    /// Symmetric along the Y-axis (top-bottom mirroring)
    Y,
    /// Symmetric along both axes
    XY,
}

/// Compute bounding box of a pixel set.
///
/// Returns (min_x, min_y, max_x, max_y) or None if empty.
pub(crate) fn bounding_box(pixels: &HashSet<(i32, i32)>) -> Option<(i32, i32, i32, i32)> {
    if pixels.is_empty() {
        return None;
    }

    let min_x = pixels.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = pixels.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = pixels.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = pixels.iter().map(|(_, y)| *y).max().unwrap();

    Some((min_x, min_y, max_x, max_y))
}

/// Detect if pixels form a filled rectangle.
///
/// Checks if the pixel set exactly matches a filled rectangle by comparing
/// the pixel count to the bounding box area.
///
/// Returns the rectangle parameters [x, y, width, height] if detected.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_rect;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().collect();
/// let result = detect_rect(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// assert_eq!(detection.shape, [0, 0, 2, 2]);
/// assert!((detection.confidence - 1.0).abs() < 0.001);
/// ```
pub fn detect_rect(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<[i32; 4]>> {
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let expected_area = (width * height) as usize;
    let actual_area = pixels.len();

    if actual_area == expected_area {
        // Perfect match - all pixels within bounding box are filled
        Some(ShapeDetection::new([min_x, min_y, width, height], 1.0))
    } else {
        // Calculate confidence based on fill ratio
        let fill_ratio = actual_area as f64 / expected_area as f64;
        // Only consider it a rectangle if fill ratio is very high
        if fill_ratio >= 0.95 {
            Some(ShapeDetection::new([min_x, min_y, width, height], fill_ratio))
        } else {
            None
        }
    }
}

/// Detect if pixels form a stroked (hollow) rectangle.
///
/// Checks if the pixel set matches a hollow rectangle by verifying:
/// 1. The outline pixels match a stroked rectangle
/// 2. The interior is empty
///
/// Returns the rectangle parameters [x, y, width, height] if detected.
/// Assumes thickness of 1 pixel.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_stroke;
/// use pixelsrc::shapes::rasterize_stroke;
///
/// let pixels = rasterize_stroke(0, 0, 5, 5, 1);
/// let result = detect_stroke(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// assert_eq!(detection.shape, [0, 0, 5, 5]);
/// ```
pub fn detect_stroke(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<[i32; 4]>> {
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    // A stroke needs to be at least 3x3 to have an interior
    if width < 3 || height < 3 {
        return None;
    }

    // Check that interior is empty (for 1-pixel thickness)
    let mut has_interior_pixel = false;
    for x in (min_x + 1)..max_x {
        for y in (min_y + 1)..max_y {
            if pixels.contains(&(x, y)) {
                has_interior_pixel = true;
                break;
            }
        }
        if has_interior_pixel {
            break;
        }
    }

    if has_interior_pixel {
        return None;
    }

    // Generate expected stroke and compare
    let expected = shapes::rasterize_stroke(min_x, min_y, width, height, 1);
    let matching = pixels.intersection(&expected).count();
    let confidence = matching as f64 / pixels.len().max(expected.len()) as f64;

    if confidence >= 0.95 {
        Some(ShapeDetection::new([min_x, min_y, width, height], confidence))
    } else {
        None
    }
}

/// Detect if pixels form a Bresenham line.
///
/// Checks if the pixel set matches a line by testing all possible endpoint
/// combinations and finding the best match using Bresenham's algorithm.
///
/// Returns the line endpoints as a vector of [x, y] pairs if detected.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_line;
/// use pixelsrc::shapes::rasterize_line;
///
/// let pixels = rasterize_line((0, 0), (5, 3));
/// let result = detect_line(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// assert_eq!(detection.shape.len(), 2);
/// ```
pub fn detect_line(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<Vec<[i32; 2]>>> {
    if pixels.is_empty() {
        return None;
    }

    // For very small pixel sets, they're trivially lines
    if pixels.len() == 1 {
        let (x, y) = *pixels.iter().next().unwrap();
        return Some(ShapeDetection::new(vec![[x, y], [x, y]], 1.0));
    }

    if pixels.len() == 2 {
        let points: Vec<_> = pixels.iter().collect();
        let (x0, y0) = *points[0];
        let (x1, y1) = *points[1];
        // Check if they're adjacent (valid 2-pixel line)
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        if dx <= 1 && dy <= 1 {
            return Some(ShapeDetection::new(vec![[x0, y0], [x1, y1]], 1.0));
        }
    }

    // Find extreme points that could be endpoints
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    // Collect candidate endpoints (pixels at the extremes)
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for &(x, y) in pixels {
        if x == min_x || x == max_x || y == min_y || y == max_y {
            candidates.push((x, y));
        }
    }

    // Try all pairs of candidates to find the best line fit
    let mut best_match: Option<((i32, i32), (i32, i32), f64)> = None;

    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let p0 = candidates[i];
            let p1 = candidates[j];

            let line_pixels = shapes::rasterize_line(p0, p1);

            // Check if the rasterized line matches the input pixels
            if line_pixels.len() == pixels.len() {
                let matching = pixels.intersection(&line_pixels).count();
                let confidence = matching as f64 / pixels.len() as f64;

                if confidence > best_match.map(|(_, _, c)| c).unwrap_or(0.0) {
                    best_match = Some((p0, p1, confidence));
                }
            }
        }
    }

    best_match.and_then(|(p0, p1, confidence)| {
        if confidence >= 0.95 {
            Some(ShapeDetection::new(vec![[p0.0, p0.1], [p1.0, p1.1]], confidence))
        } else {
            None
        }
    })
}

/// Detects symmetry in a pixel buffer.
///
/// Analyzes the given pixel data to determine if the image is symmetric
/// along the X-axis (left-right), Y-axis (top-bottom), or both.
///
/// # Arguments
///
/// * `pixels` - A slice of RGBA pixel data (4 bytes per pixel)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
///
/// # Returns
///
/// * `Some(Symmetric::XY)` if symmetric along both axes
/// * `Some(Symmetric::X)` if symmetric along X-axis only (left-right)
/// * `Some(Symmetric::Y)` if symmetric along Y-axis only (top-bottom)
/// * `None` if not symmetric
pub fn detect_symmetry(pixels: &[u8], width: u32, height: u32) -> Option<Symmetric> {
    let width = width as usize;
    let height = height as usize;
    let bytes_per_pixel = 4;

    // Check for empty or invalid input
    if width == 0 || height == 0 || pixels.len() != width * height * bytes_per_pixel {
        return None;
    }

    let x_symmetric = is_x_symmetric(pixels, width, height, bytes_per_pixel);
    let y_symmetric = is_y_symmetric(pixels, width, height, bytes_per_pixel);

    match (x_symmetric, y_symmetric) {
        (true, true) => Some(Symmetric::XY),
        (true, false) => Some(Symmetric::X),
        (false, true) => Some(Symmetric::Y),
        (false, false) => None,
    }
}

/// Checks if the image is symmetric along the X-axis (left-right mirroring).
///
/// Compares columns from the left edge with corresponding columns from the right edge.
fn is_x_symmetric(pixels: &[u8], width: usize, height: usize, bpp: usize) -> bool {
    let half_width = width / 2;

    for y in 0..height {
        for x in 0..half_width {
            let left_idx = (y * width + x) * bpp;
            let right_idx = (y * width + (width - 1 - x)) * bpp;

            // Compare all 4 bytes (RGBA)
            if pixels[left_idx..left_idx + bpp] != pixels[right_idx..right_idx + bpp] {
                return false;
            }
        }
    }

    true
}

/// Checks if the image is symmetric along the Y-axis (top-bottom mirroring).
///
/// Compares rows from the top edge with corresponding rows from the bottom edge.
fn is_y_symmetric(pixels: &[u8], width: usize, height: usize, bpp: usize) -> bool {
    let half_height = height / 2;

    for y in 0..half_height {
        let top_row_start = y * width * bpp;
        let bottom_row_start = (height - 1 - y) * width * bpp;

        // Compare entire rows
        let top_row = &pixels[top_row_start..top_row_start + width * bpp];
        let bottom_row = &pixels[bottom_row_start..bottom_row_start + width * bpp];

        if top_row != bottom_row {
            return false;
        }
    }

    true
}

/// Detect if pixels form an ellipse.
///
/// Uses a roundness heuristic to determine if the pixel set matches an ellipse
/// by comparing the actual pixel count to the expected ellipse area.
///
/// Returns the ellipse parameters [cx, cy, rx, ry] if detected.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_ellipse;
/// use pixelsrc::shapes::rasterize_ellipse;
///
/// let pixels = rasterize_ellipse(10, 10, 5, 3);
/// let result = detect_ellipse(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// // Center should be close to (10, 10)
/// assert!((detection.shape[0] - 10).abs() <= 1);
/// assert!((detection.shape[1] - 10).abs() <= 1);
/// ```
pub fn detect_ellipse(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<[i32; 4]>> {
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    // Ellipse needs reasonable size
    if width < 3 || height < 3 {
        return None;
    }

    // Calculate center and radii
    let cx = (min_x + max_x) / 2;
    let cy = (min_y + max_y) / 2;
    let rx = width / 2;
    let ry = height / 2;

    // Skip if radii are too small
    if rx < 1 || ry < 1 {
        return None;
    }

    // Generate expected ellipse and compare
    let expected = shapes::rasterize_ellipse(cx, cy, rx, ry);

    if expected.is_empty() {
        return None;
    }

    // Calculate overlap between actual and expected pixels
    let intersection = pixels.intersection(&expected).count();
    let union_size = pixels.len() + expected.len() - intersection;

    // Jaccard similarity (intersection over union)
    let jaccard = if union_size > 0 { intersection as f64 / union_size as f64 } else { 0.0 };

    // Also check the expected ellipse area vs actual
    // Expected area of ellipse = π * rx * ry
    let expected_area = std::f64::consts::PI * (rx as f64) * (ry as f64);
    let area_ratio = pixels.len() as f64 / expected_area;

    // Combine metrics for confidence
    // Good ellipse: Jaccard > 0.8 and area ratio close to 1.0
    let area_confidence = 1.0 - (area_ratio - 1.0).abs().min(1.0);
    let confidence = (jaccard + area_confidence) / 2.0;

    if confidence >= 0.7 {
        Some(ShapeDetection::new([cx, cy, rx, ry], confidence))
    } else {
        None
    }
}

/// Extract polygon vertices from a set of pixels using convex hull.
///
/// Computes the convex hull of the pixel set to find the vertices
/// that define the polygon boundary.
pub(crate) fn extract_polygon_vertices(pixels: &HashSet<(i32, i32)>) -> Vec<[i32; 2]> {
    if pixels.is_empty() {
        return Vec::new();
    }

    let mut points: Vec<(i32, i32)> = pixels.iter().copied().collect();

    // Find convex hull using Graham scan
    // First, find the lowest point (and leftmost if tied)
    points.sort_by(|a, b| {
        if a.1 != b.1 {
            a.1.cmp(&b.1)
        } else {
            a.0.cmp(&b.0)
        }
    });

    let start = points[0];

    // Sort remaining points by polar angle from start
    points[1..].sort_by(|a, b| {
        let cross = cross_product(start, *a, *b);
        if cross == 0 {
            // Collinear - sort by distance
            let dist_a = (a.0 - start.0).pow(2) + (a.1 - start.1).pow(2);
            let dist_b = (b.0 - start.0).pow(2) + (b.1 - start.1).pow(2);
            dist_a.cmp(&dist_b)
        } else if cross > 0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    // Build convex hull
    let mut hull: Vec<(i32, i32)> = Vec::new();
    for point in points {
        while hull.len() >= 2 {
            let top = hull[hull.len() - 1];
            let second = hull[hull.len() - 2];
            if cross_product(second, top, point) <= 0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(point);
    }

    hull.into_iter().map(|(x, y)| [x, y]).collect()
}

/// Compute cross product for convex hull algorithm.
fn cross_product(o: (i32, i32), a: (i32, i32), b: (i32, i32)) -> i64 {
    let ox = o.0 as i64;
    let oy = o.1 as i64;
    let ax = a.0 as i64;
    let ay = a.1 as i64;
    let bx = b.0 as i64;
    let by = b.1 as i64;
    (ax - ox) * (by - oy) - (ay - oy) * (bx - ox)
}

/// Detect the shape of a pixel set with confidence scoring.
///
/// Tries to detect the shape in order of specificity:
/// 1. Line (simplest)
/// 2. Stroked rectangle (hollow)
/// 3. Filled rectangle
/// 4. Ellipse
/// 5. Falls back to polygon
///
/// Returns the detected shape with confidence score.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::{detect_shape, DetectedShape};
/// use pixelsrc::shapes::rasterize_rect;
///
/// let pixels = rasterize_rect(0, 0, 4, 3);
/// let (shape, confidence) = detect_shape(&pixels);
/// assert!(matches!(shape, DetectedShape::Rect(_)));
/// assert!(confidence >= 0.95);
/// ```
pub fn detect_shape(pixels: &HashSet<(i32, i32)>) -> (DetectedShape, f64) {
    if pixels.is_empty() {
        return (DetectedShape::Polygon(Vec::new()), 0.0);
    }

    // Try line detection first (simplest shape)
    if let Some(detection) = detect_line(pixels) {
        if detection.confidence >= 0.95 {
            return (DetectedShape::Line(detection.shape), detection.confidence);
        }
    }

    // Try stroked rectangle (before filled, as strokes are more specific)
    if let Some(detection) = detect_stroke(pixels) {
        if detection.confidence >= 0.95 {
            return (DetectedShape::Stroke(detection.shape), detection.confidence);
        }
    }

    // Try filled rectangle
    if let Some(detection) = detect_rect(pixels) {
        if detection.confidence >= 0.95 {
            return (DetectedShape::Rect(detection.shape), detection.confidence);
        }
    }

    // Try ellipse
    if let Some(detection) = detect_ellipse(pixels) {
        if detection.confidence >= 0.7 {
            return (DetectedShape::Ellipse(detection.shape), detection.confidence);
        }
    }

    // Fall back to polygon
    let vertices = extract_polygon_vertices(pixels);
    // Polygon confidence based on how well the convex hull represents the pixels
    let hull_pixels = shapes::rasterize_polygon(
        &vertices.iter().map(|[x, y]| (*x, *y)).collect::<Vec<_>>(),
    );
    let intersection = pixels.intersection(&hull_pixels).count();
    let confidence = if pixels.is_empty() {
        0.0
    } else {
        intersection as f64 / pixels.len() as f64
    };

    (DetectedShape::Polygon(vertices), confidence)
}
