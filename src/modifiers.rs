//! Region modifiers for transforming pixel sets.
//!
//! This module provides operations to modify pixel regions including symmetry,
//! range selection, repetition, and jittering.

use std::collections::HashSet;
use std::ops::Range;

/// Axis specification for symmetric operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryAxis {
    /// Mirror along horizontal axis (y-axis, reflect left-right)
    X,
    /// Mirror along vertical axis (x-axis, reflect up-down)
    Y,
    /// Mirror along both axes (four-way symmetry)
    XY,
}

/// Apply symmetric reflection to a set of pixels.
///
/// Reflects pixels across the specified axis within the given bounds.
/// - `X` axis: reflect horizontally (mirror across x-axis)
/// - `Y` axis: reflect vertically (mirror across y-axis)
/// - `XY` axis: reflect both horizontally and vertically (four-way symmetry)
///
/// # Arguments
///
/// * `pixels` - Original set of pixels to reflect
/// * `axis` - Axis of symmetry
/// * `width` - Width of the bounding box for reflection
/// * `height` - Height of the bounding box for reflection
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::{apply_symmetric, SymmetryAxis};
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(1, 1), (2, 2)].into_iter().collect();
/// let result = apply_symmetric(&pixels, SymmetryAxis::X, 10, 10);
/// assert!(result.contains(&(1, 1)));
/// assert!(result.contains(&(8, 1))); // reflected: 10 - 1 - 1 = 8
/// ```
pub fn apply_symmetric(
    pixels: &HashSet<(i32, i32)>,
    axis: SymmetryAxis,
    width: i32,
    height: i32,
) -> HashSet<(i32, i32)> {
    let mut result = pixels.clone();

    for &(x, y) in pixels {
        match axis {
            SymmetryAxis::X => {
                let reflected_x = width - x - 1;
                if reflected_x >= 0 && reflected_x < width {
                    result.insert((reflected_x, y));
                }
            }
            SymmetryAxis::Y => {
                let reflected_y = height - y - 1;
                if reflected_y >= 0 && reflected_y < height {
                    result.insert((x, reflected_y));
                }
            }
            SymmetryAxis::XY => {
                let reflected_x = width - x - 1;
                let reflected_y = height - y - 1;
                if reflected_x >= 0 && reflected_x < width {
                    result.insert((reflected_x, y));
                }
                if reflected_y >= 0 && reflected_y < height {
                    result.insert((x, reflected_y));
                }
                if reflected_x >= 0
                    && reflected_x < width
                    && reflected_y >= 0
                    && reflected_y < height
                {
                    result.insert((reflected_x, reflected_y));
                }
            }
        }
    }

    result
}

/// Apply range filtering to a set of pixels.
///
/// Filters pixels to only include those within the specified x and y ranges.
/// Pixels outside the bounds are clipped (removed).
///
/// # Arguments
///
/// * `pixels` - Original set of pixels to filter
/// * `x_range` - Range of valid x coordinates (clipped to width if needed)
/// * `y_range` - Range of valid y coordinates (clipped to height if needed)
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_range;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(0, 0), (5, 5), (15, 15)].into_iter().collect();
/// let result = apply_range(&pixels, 0..10, 0..10);
/// assert_eq!(result.len(), 2);
/// assert!(result.contains(&(0, 0)));
/// assert!(result.contains(&(5, 5)));
/// assert!(!result.contains(&(15, 15)));
/// ```
pub fn apply_range(
    pixels: &HashSet<(i32, i32)>,
    x_range: Range<i32>,
    y_range: Range<i32>,
) -> HashSet<(i32, i32)> {
    pixels
        .iter()
        .filter(|&&(x, y)| {
            x >= x_range.start && x < x_range.end && y >= y_range.start && y < y_range.end
        })
        .copied()
        .collect()
}

/// Apply repetition to a set of pixels.
///
/// Creates copies of the pixel set at regular intervals (spacing) in both
/// x and y directions. The count specifies how many copies to make in each direction.
///
/// # Arguments
///
/// * `pixels` - Original set of pixels to repeat
/// * `count` - Number of repetitions in each direction (1 = original only)
/// * `spacing` - Distance between repetitions in pixels
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_repeat;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
/// let result = apply_repeat(&pixels, 3, 10);
/// assert!(result.contains(&(0, 0)));
/// assert!(result.contains(&(10, 0)));
/// assert!(result.contains(&(20, 0)));
/// ```
pub fn apply_repeat(pixels: &HashSet<(i32, i32)>, count: i32, spacing: i32) -> HashSet<(i32, i32)> {
    let mut result = HashSet::new();

    for dx in 0..count {
        for dy in 0..count {
            let offset_x = dx * spacing;
            let offset_y = dy * spacing;
            for &(x, y) in pixels {
                result.insert((x + offset_x, y + offset_y));
            }
        }
    }

    result
}

/// Apply jitter (random displacement) to a set of pixels.
///
/// Randomly displaces each pixel by up to the specified jitter amount
/// in both x and y directions. Uses the seed for deterministic randomness.
///
/// # Arguments
///
/// * `pixels` - Original set of pixels to jitter
/// * `jitter` - Maximum displacement in each direction (in pixels)
/// * `seed` - Seed for deterministic random number generation
///
/// # Examples
///
/// ```
/// use pixelsrc::modifiers::apply_jitter;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(10, 10)].into_iter().collect();
/// let result = apply_jitter(&pixels, 5, 42);
/// assert!(!result.is_empty());
/// // Result will have one pixel near (10, 10), within ±5 in x and y
/// ```
pub fn apply_jitter(pixels: &HashSet<(i32, i32)>, jitter: i32, seed: u64) -> HashSet<(i32, i32)> {
    let mut result = HashSet::new();

    for &(x, y) in pixels {
        let mut local_seed = seed.wrapping_add(x as u64).wrapping_add(y as u64);
        let dx = pseudo_random(&mut local_seed) % (2 * jitter + 1) - jitter;
        let dy = pseudo_random(&mut local_seed) % (2 * jitter + 1) - jitter;
        result.insert((x + dx, y + dy));
    }

    result
}

/// Simple deterministic pseudo-random number generator for jitter.
fn pseudo_random(seed: &mut u64) -> i32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (*seed % 2147483648) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_symmetric_x() {
        let pixels: HashSet<(i32, i32)> = [(1, 1), (2, 2)].into_iter().collect();
        let result = apply_symmetric(&pixels, SymmetryAxis::X, 10, 10);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(2, 2)));
        assert!(result.contains(&(8, 1)));
        assert!(result.contains(&(7, 2)));
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_apply_symmetric_y() {
        let pixels: HashSet<(i32, i32)> = [(1, 1), (2, 2)].into_iter().collect();
        let result = apply_symmetric(&pixels, SymmetryAxis::Y, 10, 10);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(2, 2)));
        assert!(result.contains(&(1, 8)));
        assert!(result.contains(&(2, 7)));
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_apply_symmetric_xy() {
        let pixels: HashSet<(i32, i32)> = [(1, 1)].into_iter().collect();
        let result = apply_symmetric(&pixels, SymmetryAxis::XY, 10, 10);
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(8, 1)));
        assert!(result.contains(&(1, 8)));
        assert!(result.contains(&(8, 8)));
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_apply_symmetric_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = apply_symmetric(&pixels, SymmetryAxis::X, 10, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_symmetric_out_of_bounds() {
        let pixels: HashSet<(i32, i32)> = [(9, 9)].into_iter().collect();
        let result = apply_symmetric(&pixels, SymmetryAxis::X, 10, 10);
        assert!(result.contains(&(9, 9)));
        assert!(result.contains(&(0, 9)));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_apply_range_basic() {
        let pixels: HashSet<(i32, i32)> = [(0, 0), (5, 5), (15, 15)].into_iter().collect();
        let result = apply_range(&pixels, 0..10, 0..10);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&(0, 0)));
        assert!(result.contains(&(5, 5)));
    }

    #[test]
    fn test_apply_range_clipping() {
        let pixels: HashSet<(i32, i32)> = [(-5, -5), (0, 0), (100, 100)].into_iter().collect();
        let result = apply_range(&pixels, 0..10, 0..10);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&(0, 0)));
    }

    #[test]
    fn test_apply_range_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = apply_range(&pixels, 0..10, 0..10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_range_all_out() {
        let pixels: HashSet<(i32, i32)> = [(100, 100)].into_iter().collect();
        let result = apply_range(&pixels, 0..10, 0..10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_repeat_basic() {
        let pixels: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
        let result = apply_repeat(&pixels, 3, 10);
        assert!(result.contains(&(0, 0)));
        assert!(result.contains(&(10, 0)));
        assert!(result.contains(&(20, 0)));
        assert!(result.contains(&(0, 10)));
        assert!(result.contains(&(10, 10)));
        assert!(result.contains(&(20, 10)));
        assert!(result.contains(&(0, 20)));
        assert!(result.contains(&(10, 20)));
        assert!(result.contains(&(20, 20)));
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn test_apply_repeat_single() {
        let pixels: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
        let result = apply_repeat(&pixels, 1, 10);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&(0, 0)));
    }

    #[test]
    fn test_apply_repeat_multiple_pixels() {
        let pixels: HashSet<(i32, i32)> = [(0, 0), (1, 1)].into_iter().collect();
        let result = apply_repeat(&pixels, 2, 5);
        assert!(result.contains(&(0, 0)));
        assert!(result.contains(&(1, 1)));
        assert!(result.contains(&(5, 0)));
        assert!(result.contains(&(6, 1)));
        assert!(result.contains(&(0, 5)));
        assert!(result.contains(&(1, 6)));
        assert!(result.contains(&(5, 5)));
        assert!(result.contains(&(6, 6)));
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_apply_jitter_basic() {
        let pixels: HashSet<(i32, i32)> = [(10, 10)].into_iter().collect();
        let result = apply_jitter(&pixels, 5, 42);
        assert_eq!(result.len(), 1);
        let (x, y) = result.into_iter().next().unwrap();
        assert!(x >= 5 && x <= 15);
        assert!(y >= 5 && y <= 15);
    }

    #[test]
    fn test_apply_jitter_deterministic() {
        let pixels: HashSet<(i32, i32)> = [(10, 10), (20, 20)].into_iter().collect();
        let result1 = apply_jitter(&pixels, 5, 42);
        let result2 = apply_jitter(&pixels, 5, 42);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_apply_jitter_different_seeds() {
        let pixels: HashSet<(i32, i32)> = [(10, 10)].into_iter().collect();
        let result1 = apply_jitter(&pixels, 5, 42);
        let result2 = apply_jitter(&pixels, 5, 99);
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_apply_jitter_zero() {
        let pixels: HashSet<(i32, i32)> = [(10, 10), (20, 20)].into_iter().collect();
        let result = apply_jitter(&pixels, 0, 42);
        assert_eq!(result, pixels);
    }

    #[test]
    fn test_apply_jitter_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = apply_jitter(&pixels, 5, 42);
        assert!(result.is_empty());
    }
}
