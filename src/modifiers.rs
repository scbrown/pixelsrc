//! Region modifiers for structured sprites.
//!
//! This module provides functions to transform and modify pixel regions
//! after they've been rasterized from shapes. Modifiers include:
//! - Symmetry (mirroring across axes)
//! - Range constraints (x/y limits)
//! - Repeating/tiling
//! - Jitter (controlled randomness)

use std::collections::HashSet;
use crate::models::JitterSpec;
use rand::Rng;

/// Apply symmetry to a set of pixels.
///
/// Mirrors pixels across the specified axis within the canvas bounds.
/// The axis parameter can be:
/// - `"x"` - Mirror horizontally (mirror across vertical centerline)
/// - `"y"` - Mirror vertically (mirror across horizontal centerline)
/// - `"xy"` - Mirror both horizontally and vertically
/// - A coordinate like `4` - Mirror across the line x=4 or y=4 depending on position
///
/// # Arguments
///
/// * `pixels` - Set of pixel coordinates to mirror
/// * `axis` - Symmetry axis specification ("x", "y", "xy", or coordinate)
/// * `width` - Canvas width for calculating mirror position
/// * `height` - Canvas height for calculating mirror position
///
/// # Returns
///
/// A new HashSet containing both original and mirrored pixels.
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_symmetric;
/// use std::collections::HashSet;
///
/// let mut pixels: HashSet<(i32, i32)> = [(0, 0), (1, 0)].iter().cloned().collect();
///
/// // Mirror horizontally across x-axis (vertical centerline of 8x8 canvas)
/// let mirrored = apply_symmetric(&pixels, "x", 8, 8);
/// assert!(mirrored.contains(&(0, 0)));
/// assert!(mirrored.contains(&(7, 0))); // Mirrored point
/// ```
pub fn apply_symmetric(
    pixels: &HashSet<(i32, i32)>,
    axis: &str,
    width: i32,
    height: i32,
) -> HashSet<(i32, i32)> {
    let mut result = pixels.clone();

    match axis {
        "x" => {
            // Mirror horizontally: (x, y) -> (width - 1 - x, y)
            for &(x, y) in pixels {
                let mirrored_x = width - 1 - x;
                if mirrored_x >= 0 && mirrored_x < width {
                    result.insert((mirrored_x, y));
                }
            }
        }
        "y" => {
            // Mirror vertically: (x, y) -> (x, height - 1 - y)
            for &(x, y) in pixels {
                let mirrored_y = height - 1 - y;
                if mirrored_y >= 0 && mirrored_y < height {
                    result.insert((x, mirrored_y));
                }
            }
        }
        "xy" => {
            // Mirror both: apply x and y symmetry
            result = apply_symmetric(&result, "x", width, height);
            result = apply_symmetric(&result, "y", width, height);
        }
        _ => {
            // Try to parse as a coordinate number
            if let Ok(coord) = axis.parse::<i32>() {
                // Mirror across line at this coordinate
                for &(x, y) in pixels {
                    let mirrored_x = 2 * coord - x;
                    if mirrored_x >= 0 && mirrored_x < width {
                        result.insert((mirrored_x, y));
                    }
                }
            }
        }
    }

    result
}

/// Apply range constraints to a set of pixels.
///
/// Limits pixels to specific x (column) and y (row) ranges.
/// Pixels outside the specified ranges are removed.
///
/// # Arguments
///
/// * `pixels` - Set of pixel coordinates to constrain
/// * `x_range` - Optional x range [min, max] (inclusive)
/// * `y_range` - Optional y range [min, max] (inclusive)
///
/// # Returns
///
/// A new HashSet containing only pixels within the specified ranges.
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_range;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(0, 0), (5, 0), (10, 0)].iter().cloned().collect();
///
/// // Keep only pixels with x in range [2, 7]
/// let constrained = apply_range(&pixels, Some((2, 7)), None);
/// assert!(!constrained.contains(&(0, 0)));
/// assert!(constrained.contains(&(5, 0)));
/// assert!(!constrained.contains(&(10, 0)));
/// ```
pub fn apply_range(
    pixels: &HashSet<(i32, i32)>,
    x_range: Option<(i32, i32)>,
    y_range: Option<(i32, i32)>,
) -> HashSet<(i32, i32)> {
    let mut result = HashSet::new();

    for &(x, y) in pixels {
        let x_ok = match x_range {
            Some((min, max)) => x >= min && x <= max,
            None => true,
        };

        let y_ok = match y_range {
            Some((min, max)) => y >= min && y <= max,
            None => true,
        };

        if x_ok && y_ok {
            result.insert((x, y));
        }
    }

    result
}

/// Apply repeat/tiling to a set of pixels.
///
/// Tiles the pixel pattern by repeating it in x and y directions.
/// Each repeat can have spacing between copies.
///
/// # Arguments
///
/// * `pixels` - Set of pixel coordinates to repeat
/// * `count` - Number of repetitions [count_x, count_y]
/// * `spacing` - Spacing between repetitions [spacing_x, spacing_y]
///
/// # Returns
///
/// A new HashSet containing repeated pixels at offsets.
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_repeat;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(0, 0), (1, 0)].iter().cloned().collect();
///
/// // Repeat 3 times horizontally with spacing of 1
/// let repeated = apply_repeat(&pixels, (3, 1), (1, 0));
/// // Original pixels at (0,0), (1,0) plus repeats at (3,0), (4,0) and (6,0), (7,0)
/// assert_eq!(repeated.len(), 6);
/// ```
pub fn apply_repeat(
    pixels: &HashSet<(i32, i32)>,
    count: (u32, u32),
    spacing: (u32, u32),
) -> HashSet<(i32, i32)> {
    let mut result = HashSet::new();

    let (count_x, count_y) = count;
    let (spacing_x, spacing_y) = spacing;

    for repeat_x in 0..count_x {
        for repeat_y in 0..count_y {
            let offset_x = repeat_x as i32 * (spacing_x as i32 + 1);
            let offset_y = repeat_y as i32 * (spacing_y as i32 + 1);

            for &(x, y) in pixels {
                result.insert((x + offset_x, y + offset_y));
            }
        }
    }

    result
}

/// Apply jitter to a set of pixels.
///
/// Adds controlled randomness to pixel positions. Each pixel can be offset
/// by a random amount within the specified ranges. The same seed produces
/// deterministic results.
///
/// # Arguments
///
/// * `pixels` - Set of pixel coordinates to jitter
/// * `jitter` - Jitter specification with x and y ranges
/// * `seed` - Random seed for reproducible jitter
///
/// # Returns
///
/// A new HashSet containing jittered pixels.
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_jitter;
/// use pixelsrc::models::JitterSpec;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(5, 5)].iter().cloned().collect();
/// let jitter = JitterSpec {
///     x: Some([-1, 1]),
///     y: Some([0, 0]),
/// };
///
/// // Apply jitter with seed 42
/// let jittered = apply_jitter(&pixels, &jitter, Some(42));
/// // Pixel will be at (4,5), (5,5), or (6,5) depending on seed
/// assert_eq!(jittered.len(), 1);
/// ```
pub fn apply_jitter(
    pixels: &HashSet<(i32, i32)>,
    jitter: &JitterSpec,
    seed: Option<u32>,
) -> HashSet<(i32, i32)> {
    let mut result = HashSet::new();

    // Use deterministic random for reproducibility
    let mut rng: rand::rngs::StdRng;
    match seed {
        Some(s) => rng = rand::SeedableRng::seed_from_u64(s as u64),
        None => rng = rand::SeedableRng::from_entropy(),
    }

    for &(x, y) in pixels {
        let jitter_x = match jitter.x {
            Some(range) => rng.gen_range(range[0]..=range[1]),
            None => 0,
        };

        let jitter_y = match jitter.y {
            Some(range) => rng.gen_range(range[0]..=range[1]),
            None => 0,
        };

        result.insert((x + jitter_x, y + jitter_y));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Symmetry Tests
    // ============================================================================

    #[test]
    fn test_apply_symmetric_x() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((1, 0));

        let mirrored = apply_symmetric(&pixels, "x", 8, 8);

        assert!(mirrored.contains(&(0, 0)));
        assert!(mirrored.contains(&(1, 0)));
        assert!(mirrored.contains(&(7, 0))); // Mirror of (0,0) at x=8-1-0=7
        assert!(mirrored.contains(&(6, 0))); // Mirror of (1,0) at x=8-1-1=6
    }

    #[test]
    fn test_apply_symmetric_y() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((0, 1));

        let mirrored = apply_symmetric(&pixels, "y", 8, 8);

        assert!(mirrored.contains(&(0, 0)));
        assert!(mirrored.contains(&(0, 1)));
        assert!(mirrored.contains(&(0, 7))); // Mirror of (0,0) at y=8-1-0=7
        assert!(mirrored.contains(&(0, 6))); // Mirror of (0,1) at y=8-1-1=6
    }

    #[test]
    fn test_apply_symmetric_xy() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((1, 1));

        let mirrored = apply_symmetric(&pixels, "xy", 8, 8);

        assert!(mirrored.contains(&(1, 1)));
        assert!(mirrored.contains(&(6, 1))); // x-mirror
        assert!(mirrored.contains(&(1, 6))); // y-mirror
        assert!(mirrored.contains(&(6, 6))); // xy-mirror
    }

    #[test]
    fn test_apply_symmetric_coordinate() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((2, 0));

        let mirrored = apply_symmetric(&pixels, "4", 8, 8);

        assert!(mirrored.contains(&(2, 0)));
        assert!(mirrored.contains(&(6, 0))); // Mirror at 2*4-2=6
    }

    #[test]
    fn test_apply_symmetric_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let mirrored = apply_symmetric(&pixels, "x", 8, 8);
        assert_eq!(mirrored.len(), 0);
    }

    // ============================================================================
    // Range Tests
    // ============================================================================

    #[test]
    fn test_apply_range_x_only() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((5, 0));
        pixels.insert((10, 0));

        let constrained = apply_range(&pixels, Some((2, 7)), None);

        assert!(!constrained.contains(&(0, 0)));
        assert!(constrained.contains(&(5, 0)));
        assert!(!constrained.contains(&(10, 0)));
    }

    #[test]
    fn test_apply_range_y_only() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((0, 5));
        pixels.insert((0, 10));

        let constrained = apply_range(&pixels, None, Some((2, 7)));

        assert!(!constrained.contains(&(0, 0)));
        assert!(constrained.contains(&(0, 5)));
        assert!(!constrained.contains(&(0, 10)));
    }

    #[test]
    fn test_apply_range_both() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((5, 5));
        pixels.insert((10, 10));

        let constrained = apply_range(&pixels, Some((2, 7)), Some((2, 7)));

        assert!(!constrained.contains(&(0, 0)));
        assert!(constrained.contains(&(5, 5)));
        assert!(!constrained.contains(&(10, 10)));
    }

    #[test]
    fn test_apply_range_none() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((5, 5));

        let constrained = apply_range(&pixels, None, None);

        assert_eq!(constrained.len(), 2);
        assert!(constrained.contains(&(0, 0)));
        assert!(constrained.contains(&(5, 5)));
    }

    #[test]
    fn test_apply_range_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let constrained = apply_range(&pixels, Some((0, 10)), Some((0, 10)));
        assert_eq!(constrained.len(), 0);
    }

    // ============================================================================
    // Repeat Tests
    // ============================================================================

    #[test]
    fn test_apply_repeat_horizontal() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((1, 0));

        let repeated = apply_repeat(&pixels, (3, 1), (1, 0));

        // Original: (0,0), (1,0)
        // Repeat 1 at offset 2: (2,0), (3,0)
        // Repeat 2 at offset 4: (4,0), (5,0)
        assert_eq!(repeated.len(), 6);
        assert!(repeated.contains(&(0, 0)));
        assert!(repeated.contains(&(1, 0)));
        assert!(repeated.contains(&(2, 0)));
        assert!(repeated.contains(&(3, 0)));
        assert!(repeated.contains(&(4, 0)));
        assert!(repeated.contains(&(5, 0)));
    }

    #[test]
    fn test_apply_repeat_vertical() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((0, 1));

        let repeated = apply_repeat(&pixels, (1, 3), (0, 1));

        assert_eq!(repeated.len(), 6);
        assert!(repeated.contains(&(0, 0)));
        assert!(repeated.contains(&(0, 1)));
        assert!(repeated.contains(&(0, 2)));
        assert!(repeated.contains(&(0, 3)));
        assert!(repeated.contains(&(0, 4)));
        assert!(repeated.contains(&(0, 5)));
    }

    #[test]
    fn test_apply_repeat_both() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));

        let repeated = apply_repeat(&pixels, (2, 2), (1, 1));

        // 2x2 grid with spacing 1: positions at (0,0), (2,0), (0,2), (2,2)
        assert_eq!(repeated.len(), 4);
        assert!(repeated.contains(&(0, 0)));
        assert!(repeated.contains(&(2, 0)));
        assert!(repeated.contains(&(0, 2)));
        assert!(repeated.contains(&(2, 2)));
    }

    #[test]
    fn test_apply_repeat_zero_count() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));

        let repeated = apply_repeat(&pixels, (0, 1), (0, 0));

        // Zero count should produce no pixels
        assert_eq!(repeated.len(), 0);
    }

    #[test]
    fn test_apply_repeat_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let repeated = apply_repeat(&pixels, (2, 2), (0, 0));
        assert_eq!(repeated.len(), 0);
    }

    // ============================================================================
    // Jitter Tests
    // ============================================================================

    #[test]
    fn test_apply_jitter_deterministic() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((5, 5));

        let jitter = JitterSpec {
            x: Some([-1, 1]),
            y: Some([0, 0]),
        };

        let jittered1 = apply_jitter(&pixels, &jitter, Some(42));
        let jittered2 = apply_jitter(&pixels, &jitter, Some(42));

        // Same seed should produce same result
        assert_eq!(jittered1, jittered2);
    }

    #[test]
    fn test_apply_jitter_range() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((5, 5));

        let jitter = JitterSpec {
            x: Some([-1, 1]),
            y: Some([0, 0]),
        };

        let jittered = apply_jitter(&pixels, &jitter, Some(42));
        assert_eq!(jittered.len(), 1);

        // Result should be within range [4,6] on x, exactly 5 on y
        let (x, y) = jittered.iter().next().unwrap();
        assert!(*x >= 4 && *x <= 6);
        assert_eq!(*y, 5);
    }

    #[test]
    fn test_apply_jitter_no_jitter() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((5, 5));

        let jitter = JitterSpec {
            x: None,
            y: None,
        };

        let jittered = apply_jitter(&pixels, &jitter, Some(42));

        // No jitter should leave pixel unchanged
        assert_eq!(jittered, pixels);
    }

    #[test]
    fn test_apply_jitter_multiple_pixels() {
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        pixels.insert((0, 0));
        pixels.insert((5, 5));

        let jitter = JitterSpec {
            x: Some([0, 0]),
            y: Some([0, 0]),
        };

        let jittered = apply_jitter(&pixels, &jitter, Some(42));

        // Zero jitter should preserve all pixels
        assert_eq!(jittered.len(), 2);
        assert!(jittered.contains(&(0, 0)));
        assert!(jittered.contains(&(5, 5)));
    }

    #[test]
    fn test_apply_jitter_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();

        let jitter = JitterSpec {
            x: Some([-1, 1]),
            y: Some([-1, 1]),
        };

        let jittered = apply_jitter(&pixels, &jitter, Some(42));
        assert_eq!(jittered.len(), 0);
    }
}
