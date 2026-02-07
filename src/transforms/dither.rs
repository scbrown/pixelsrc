//! Dither patterns for pixel art effects (ATF-8)
//!
//! Provides various dithering patterns for blending between two colors
//! without using alpha transparency.

/// Built-in dither pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherPattern {
    /// 2x2 checkerboard pattern
    Checker,
    /// 2x2 Bayer ordered dither (4 threshold levels)
    Ordered2x2,
    /// 4x4 Bayer ordered dither (16 threshold levels)
    Ordered4x4,
    /// 8x8 Bayer ordered dither (64 threshold levels)
    Ordered8x8,
    /// Diagonal line pattern
    Diagonal,
    /// Horizontal line pattern
    Horizontal,
    /// Vertical line pattern
    Vertical,
    /// Random noise dither (seeded)
    Noise,
}

impl DitherPattern {
    /// Parse a pattern name string into a DitherPattern
    pub fn from_str(s: &str) -> Option<DitherPattern> {
        match s.to_lowercase().as_str() {
            "checker" | "checkerboard" => Some(DitherPattern::Checker),
            "ordered-2x2" | "ordered2x2" | "bayer-2x2" | "bayer2x2" => {
                Some(DitherPattern::Ordered2x2)
            }
            "ordered-4x4" | "ordered4x4" | "bayer-4x4" | "bayer4x4" => {
                Some(DitherPattern::Ordered4x4)
            }
            "ordered-8x8" | "ordered8x8" | "bayer-8x8" | "bayer8x8" => {
                Some(DitherPattern::Ordered8x8)
            }
            "diagonal" => Some(DitherPattern::Diagonal),
            "horizontal" => Some(DitherPattern::Horizontal),
            "vertical" => Some(DitherPattern::Vertical),
            "noise" | "random" => Some(DitherPattern::Noise),
            _ => None,
        }
    }

    /// Get the threshold value at a given position (0.0 to 1.0)
    ///
    /// For noise pattern, uses a simple hash-based pseudo-random function with the seed.
    pub fn threshold_at(&self, x: u32, y: u32, seed: u64) -> f64 {
        match self {
            DitherPattern::Checker => {
                // 2x2 checkerboard: alternating 0 and 1
                if (x + y).is_multiple_of(2) {
                    0.25
                } else {
                    0.75
                }
            }
            DitherPattern::Ordered2x2 => {
                // 2x2 Bayer matrix:
                // | 0 2 |   normalized: | 0.0  0.5  |
                // | 3 1 |               | 0.75 0.25 |
                const BAYER_2X2: [[f64; 2]; 2] = [[0.0 / 4.0, 2.0 / 4.0], [3.0 / 4.0, 1.0 / 4.0]];
                let px = (x % 2) as usize;
                let py = (y % 2) as usize;
                BAYER_2X2[py][px]
            }
            DitherPattern::Ordered4x4 => {
                // 4x4 Bayer matrix
                const BAYER_4X4: [[f64; 4]; 4] = [
                    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
                    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
                    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
                    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
                ];
                let px = (x % 4) as usize;
                let py = (y % 4) as usize;
                BAYER_4X4[py][px]
            }
            DitherPattern::Ordered8x8 => {
                // 8x8 Bayer matrix
                const BAYER_8X8: [[f64; 8]; 8] = [
                    [
                        0.0 / 64.0,
                        32.0 / 64.0,
                        8.0 / 64.0,
                        40.0 / 64.0,
                        2.0 / 64.0,
                        34.0 / 64.0,
                        10.0 / 64.0,
                        42.0 / 64.0,
                    ],
                    [
                        48.0 / 64.0,
                        16.0 / 64.0,
                        56.0 / 64.0,
                        24.0 / 64.0,
                        50.0 / 64.0,
                        18.0 / 64.0,
                        58.0 / 64.0,
                        26.0 / 64.0,
                    ],
                    [
                        12.0 / 64.0,
                        44.0 / 64.0,
                        4.0 / 64.0,
                        36.0 / 64.0,
                        14.0 / 64.0,
                        46.0 / 64.0,
                        6.0 / 64.0,
                        38.0 / 64.0,
                    ],
                    [
                        60.0 / 64.0,
                        28.0 / 64.0,
                        52.0 / 64.0,
                        20.0 / 64.0,
                        62.0 / 64.0,
                        30.0 / 64.0,
                        54.0 / 64.0,
                        22.0 / 64.0,
                    ],
                    [
                        3.0 / 64.0,
                        35.0 / 64.0,
                        11.0 / 64.0,
                        43.0 / 64.0,
                        1.0 / 64.0,
                        33.0 / 64.0,
                        9.0 / 64.0,
                        41.0 / 64.0,
                    ],
                    [
                        51.0 / 64.0,
                        19.0 / 64.0,
                        59.0 / 64.0,
                        27.0 / 64.0,
                        49.0 / 64.0,
                        17.0 / 64.0,
                        57.0 / 64.0,
                        25.0 / 64.0,
                    ],
                    [
                        15.0 / 64.0,
                        47.0 / 64.0,
                        7.0 / 64.0,
                        39.0 / 64.0,
                        13.0 / 64.0,
                        45.0 / 64.0,
                        5.0 / 64.0,
                        37.0 / 64.0,
                    ],
                    [
                        63.0 / 64.0,
                        31.0 / 64.0,
                        55.0 / 64.0,
                        23.0 / 64.0,
                        61.0 / 64.0,
                        29.0 / 64.0,
                        53.0 / 64.0,
                        21.0 / 64.0,
                    ],
                ];
                let px = (x % 8) as usize;
                let py = (y % 8) as usize;
                BAYER_8X8[py][px]
            }
            DitherPattern::Diagonal => {
                // Diagonal lines: threshold based on (x + y) mod pattern_size
                let pattern_size = 4;
                let pos = (x + y) % pattern_size;
                pos as f64 / pattern_size as f64
            }
            DitherPattern::Horizontal => {
                // Horizontal lines: threshold based on y mod pattern_size
                let pattern_size = 4;
                let pos = y % pattern_size;
                pos as f64 / pattern_size as f64
            }
            DitherPattern::Vertical => {
                // Vertical lines: threshold based on x mod pattern_size
                let pattern_size = 4;
                let pos = x % pattern_size;
                pos as f64 / pattern_size as f64
            }
            DitherPattern::Noise => {
                // Simple hash-based pseudo-random noise
                // Uses a variation of splitmix64 for quick hashing
                let mut hash = seed;
                hash ^= (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
                hash ^= (y as u64).wrapping_mul(0xBF58476D1CE4E5B9);
                hash = hash.wrapping_mul(0x94D049BB133111EB);
                hash ^= hash >> 30;
                // Convert to 0.0-1.0 range
                (hash as f64) / (u64::MAX as f64)
            }
        }
    }

    /// Determine if a pixel should use the "dark" token (false) or "light" token (true)
    /// based on position and threshold
    pub fn should_use_light(&self, x: u32, y: u32, threshold: f64, seed: u64) -> bool {
        self.threshold_at(x, y, seed) >= threshold
    }
}

/// Direction for gradient dithering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientDirection {
    /// Top to bottom
    Vertical,
    /// Left to right
    Horizontal,
    /// Center outward (circular)
    Radial,
}

impl GradientDirection {
    /// Parse a direction string
    pub fn from_str(s: &str) -> Option<GradientDirection> {
        match s.to_lowercase().as_str() {
            "vertical" | "v" => Some(GradientDirection::Vertical),
            "horizontal" | "h" => Some(GradientDirection::Horizontal),
            "radial" | "r" => Some(GradientDirection::Radial),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dither_pattern_from_str() {
        assert_eq!(DitherPattern::from_str("checker"), Some(DitherPattern::Checker));
        assert_eq!(DitherPattern::from_str("checkerboard"), Some(DitherPattern::Checker));
        assert_eq!(DitherPattern::from_str("ordered-2x2"), Some(DitherPattern::Ordered2x2));
        assert_eq!(DitherPattern::from_str("ordered-4x4"), Some(DitherPattern::Ordered4x4));
        assert_eq!(DitherPattern::from_str("ordered-8x8"), Some(DitherPattern::Ordered8x8));
        assert_eq!(DitherPattern::from_str("bayer-4x4"), Some(DitherPattern::Ordered4x4));
        assert_eq!(DitherPattern::from_str("diagonal"), Some(DitherPattern::Diagonal));
        assert_eq!(DitherPattern::from_str("horizontal"), Some(DitherPattern::Horizontal));
        assert_eq!(DitherPattern::from_str("vertical"), Some(DitherPattern::Vertical));
        assert_eq!(DitherPattern::from_str("noise"), Some(DitherPattern::Noise));
        assert_eq!(DitherPattern::from_str("random"), Some(DitherPattern::Noise));
        assert_eq!(DitherPattern::from_str("unknown"), None);
    }

    #[test]
    fn test_dither_pattern_checker_threshold() {
        let pattern = DitherPattern::Checker;
        // Checker pattern: (0,0) = 0.25, (0,1) = 0.75, (1,0) = 0.75, (1,1) = 0.25
        assert_eq!(pattern.threshold_at(0, 0, 0), 0.25);
        assert_eq!(pattern.threshold_at(1, 0, 0), 0.75);
        assert_eq!(pattern.threshold_at(0, 1, 0), 0.75);
        assert_eq!(pattern.threshold_at(1, 1, 0), 0.25);
        // Pattern repeats
        assert_eq!(pattern.threshold_at(2, 2, 0), 0.25);
        assert_eq!(pattern.threshold_at(3, 3, 0), 0.25);
    }

    #[test]
    fn test_dither_pattern_ordered_2x2() {
        let pattern = DitherPattern::Ordered2x2;
        // 2x2 Bayer: [[0, 2], [3, 1]] normalized by /4
        assert_eq!(pattern.threshold_at(0, 0, 0), 0.0);
        assert_eq!(pattern.threshold_at(1, 0, 0), 0.5);
        assert_eq!(pattern.threshold_at(0, 1, 0), 0.75);
        assert_eq!(pattern.threshold_at(1, 1, 0), 0.25);
    }

    #[test]
    fn test_dither_pattern_should_use_light() {
        let pattern = DitherPattern::Checker;
        // At threshold 0.5: (0,0)=0.25 < 0.5 -> false (dark), (0,1)=0.75 >= 0.5 -> true (light)
        assert!(!pattern.should_use_light(0, 0, 0.5, 0));
        assert!(pattern.should_use_light(0, 1, 0.5, 0));
    }

    #[test]
    fn test_dither_pattern_noise_seeded() {
        let pattern = DitherPattern::Noise;
        // Same position + seed should give same result
        let t1 = pattern.threshold_at(5, 10, 42);
        let t2 = pattern.threshold_at(5, 10, 42);
        assert_eq!(t1, t2);

        // Different seed should give different result (very unlikely to be same)
        let t3 = pattern.threshold_at(5, 10, 123);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_gradient_direction_from_str() {
        assert_eq!(GradientDirection::from_str("vertical"), Some(GradientDirection::Vertical));
        assert_eq!(GradientDirection::from_str("v"), Some(GradientDirection::Vertical));
        assert_eq!(GradientDirection::from_str("horizontal"), Some(GradientDirection::Horizontal));
        assert_eq!(GradientDirection::from_str("h"), Some(GradientDirection::Horizontal));
        assert_eq!(GradientDirection::from_str("radial"), Some(GradientDirection::Radial));
        assert_eq!(GradientDirection::from_str("r"), Some(GradientDirection::Radial));
        assert_eq!(GradientDirection::from_str("unknown"), None);
    }
}
