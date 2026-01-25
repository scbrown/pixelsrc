//! Transform operations for sprites and animations
//!
//! Supports both CLI transforms (`pxl transform`) and format attributes
//! (`"transform": ["mirror-h", "rotate:90"]`).

use image::{imageops::FilterType, RgbaImage};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Dither Patterns (ATF-8)
// ============================================================================

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
                if (x + y) % 2 == 0 {
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

/// Errors that can occur during transform parsing or application
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransformError {
    /// Unknown transform operation
    #[error("unknown transform operation: {0}")]
    UnknownOperation(String),

    /// Invalid parameter value
    #[error("invalid parameter for {op}: {message}")]
    InvalidParameter { op: String, message: String },

    /// Missing required parameter
    #[error("missing required parameter for {op}: {param}")]
    MissingParameter { op: String, param: String },

    /// Invalid rotation degrees (must be 90, 180, or 270)
    #[error("invalid rotation degrees: {0} (must be 90, 180, or 270)")]
    InvalidRotation(u16),

    /// Invalid tile dimensions
    #[error("invalid tile dimensions: {0}")]
    InvalidTileDimensions(String),

    /// Invalid crop region
    #[error("invalid crop region: {0}")]
    InvalidCropRegion(String),

    /// Invalid shift values
    #[error("invalid shift values: {0}")]
    InvalidShift(String),

    /// General parse error
    #[error("parse error: {0}")]
    ParseError(String),
}

/// A single transform operation with optional parameters
#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    // Geometric
    MirrorH,
    MirrorV,
    Rotate {
        degrees: u16,
    }, // 90, 180, 270

    // Expansion
    Tile {
        w: u32,
        h: u32,
    },
    Pad {
        size: u32,
    },
    Crop {
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    },

    // Effects
    Outline {
        token: Option<String>,
        width: u32,
    },
    Shift {
        x: i32,
        y: i32,
    },
    Shadow {
        x: i32,
        y: i32,
        token: Option<String>,
    },
    SelOut {
        /// Fallback token for outline pixels that can't determine neighbor color
        fallback: Option<String>,
        /// Explicit mapping from fill token to outline token
        /// Key "*" is used as default fallback
        mapping: Option<HashMap<String, String>>,
    },
    /// Scale transform for squash & stretch effects
    /// Scales the grid by the given X and Y factors
    Scale {
        /// Horizontal scale factor (1.0 = no change, 2.0 = double width)
        x: f32,
        /// Vertical scale factor (1.0 = no change, 0.5 = half height)
        y: f32,
    },

    /// Skew along the X axis (horizontal shear)
    /// Positive angles skew the top edge to the right
    SkewX {
        /// Skew angle in degrees
        degrees: f32,
    },

    /// Skew along the Y axis (vertical shear)
    /// Positive angles skew the left edge downward
    SkewY {
        /// Skew angle in degrees
        degrees: f32,
    },

    // Animation (only valid for Animation type)
    Pingpong {
        exclude_ends: bool,
    },
    Reverse,
    FrameOffset {
        offset: i32,
    },
    Hold {
        frame: usize,
        count: usize,
    },

    // Dithering (ATF-8)
    /// Apply dither pattern to blend between two tokens
    Dither {
        /// Dither pattern to use
        pattern: DitherPattern,
        /// Two-element array: [dark_token, light_token]
        tokens: (String, String),
        /// Blend threshold (0.0-1.0), default 0.5
        threshold: f64,
        /// Random seed for noise pattern
        seed: u64,
    },
    /// Apply dithered gradient across the sprite
    DitherGradient {
        /// Gradient direction
        direction: GradientDirection,
        /// Starting token (at gradient start)
        from: String,
        /// Ending token (at gradient end)
        to: String,
        /// Dither pattern to use
        pattern: DitherPattern,
    },

    // Sub-pixel Animation (ATF-13)
    /// Create apparent motion smaller than 1 pixel via color blending
    ///
    /// The subpixel values (0.0-1.0) control how much to blend toward
    /// adjacent pixels:
    /// - `subpixel_x: 0.5` = 50% blend toward the right pixel
    /// - `subpixel_y: 0.5` = 50% blend toward the bottom pixel
    Subpixel {
        /// Horizontal sub-pixel offset (0.0-1.0)
        x: f64,
        /// Vertical sub-pixel offset (0.0-1.0)
        y: f64,
    },
}

/// Generate a plain-language explanation of a transform's effect
///
/// Returns a human-readable description suitable for LSP hover info.
///
/// # Examples
///
/// ```
/// use pixelsrc::transforms::{Transform, explain_transform};
///
/// let t = Transform::MirrorH;
/// assert_eq!(explain_transform(&t), "Flip horizontally (mirror left ↔ right)");
///
/// let t = Transform::Rotate { degrees: 90 };
/// assert_eq!(explain_transform(&t), "Rotate 90° clockwise");
/// ```
pub fn explain_transform(transform: &Transform) -> String {
    match transform {
        // Geometric
        Transform::MirrorH => "Flip horizontally (mirror left ↔ right)".to_string(),
        Transform::MirrorV => "Flip vertically (mirror top ↔ bottom)".to_string(),
        Transform::Rotate { degrees } => {
            let dir = if *degrees == 180 { "" } else { " clockwise" };
            format!("Rotate {}°{}", degrees, dir)
        }

        // Expansion
        Transform::Tile { w, h } => {
            format!("Tile {}×{} (repeat sprite in a grid)", w, h)
        }
        Transform::Pad { size } => {
            format!("Add {} pixel(s) of transparent padding around edges", size)
        }
        Transform::Crop { x, y, w, h } => {
            format!("Crop to {}×{} region starting at ({}, {})", w, h, x, y)
        }

        // Effects
        Transform::Outline { token, width } => {
            let color = token.as_ref().map(|t| t.as_str()).unwrap_or("default color");
            if *width == 1 {
                format!("Add 1px outline using {}", color)
            } else {
                format!("Add {}px outline using {}", width, color)
            }
        }
        Transform::Shift { x, y } => {
            let x_dir = match x.cmp(&0) {
                std::cmp::Ordering::Greater => format!("{} right", x),
                std::cmp::Ordering::Less => format!("{} left", x.abs()),
                std::cmp::Ordering::Equal => "0".to_string(),
            };
            let y_dir = match y.cmp(&0) {
                std::cmp::Ordering::Greater => format!("{} down", y),
                std::cmp::Ordering::Less => format!("{} up", y.abs()),
                std::cmp::Ordering::Equal => "0".to_string(),
            };
            format!("Shift {} pixels {}, {} pixels {}", x.abs(), x_dir, y.abs(), y_dir)
        }
        Transform::Shadow { x, y, token } => {
            let color = token.as_ref().map(|t| t.as_str()).unwrap_or("shadow color");
            format!("Add drop shadow offset ({}, {}) using {}", x, y, color)
        }
        Transform::SelOut { fallback, mapping } => {
            let desc = if mapping.is_some() {
                "with custom color mapping"
            } else if fallback.is_some() {
                "with fallback color"
            } else {
                "using neighbor colors"
            };
            format!("Selective outline (outline matching adjacent fill colors) {}", desc)
        }
        Transform::Scale { x, y } => {
            let x_pct = (x * 100.0) as i32;
            let y_pct = (y * 100.0) as i32;
            if (x - y).abs() < 0.001 {
                format!("Scale to {}%", x_pct)
            } else {
                format!("Scale width to {}%, height to {}%", x_pct, y_pct)
            }
        }
        Transform::SkewX { degrees } => {
            format!("Skew horizontally by {}° (shear along X axis)", degrees)
        }
        Transform::SkewY { degrees } => {
            format!("Skew vertically by {}° (shear along Y axis)", degrees)
        }

        // Animation
        Transform::Pingpong { exclude_ends } => {
            if *exclude_ends {
                "Play frames forward then backward (excluding first/last to avoid doubling)"
                    .to_string()
            } else {
                "Play frames forward then backward (1,2,3 → 1,2,3,2,1)".to_string()
            }
        }
        Transform::Reverse => "Reverse frame order (play animation backwards)".to_string(),
        Transform::FrameOffset { offset } => {
            if *offset > 0 {
                format!("Shift animation timing {} frame(s) later", offset)
            } else {
                format!("Shift animation timing {} frame(s) earlier", offset.abs())
            }
        }
        Transform::Hold { frame, count } => {
            format!("Hold frame {} for {} extra frame(s)", frame + 1, count)
        }

        // Dithering
        Transform::Dither { pattern, tokens, threshold, .. } => {
            let pattern_name = match pattern {
                DitherPattern::Checker => "checkerboard",
                DitherPattern::Ordered2x2 => "2×2 Bayer",
                DitherPattern::Ordered4x4 => "4×4 Bayer",
                DitherPattern::Ordered8x8 => "8×8 Bayer",
                DitherPattern::Diagonal => "diagonal lines",
                DitherPattern::Horizontal => "horizontal lines",
                DitherPattern::Vertical => "vertical lines",
                DitherPattern::Noise => "random noise",
            };
            let pct = (threshold * 100.0) as i32;
            format!(
                "Apply {} dither pattern between {} and {} at {}% threshold",
                pattern_name, tokens.0, tokens.1, pct
            )
        }
        Transform::DitherGradient { direction, from, to, pattern } => {
            let dir_name = match direction {
                GradientDirection::Vertical => "top to bottom",
                GradientDirection::Horizontal => "left to right",
                GradientDirection::Radial => "center outward",
            };
            let pattern_name = match pattern {
                DitherPattern::Checker => "checkerboard",
                DitherPattern::Ordered2x2 => "2×2 Bayer",
                DitherPattern::Ordered4x4 => "4×4 Bayer",
                DitherPattern::Ordered8x8 => "8×8 Bayer",
                DitherPattern::Diagonal => "diagonal",
                DitherPattern::Horizontal => "horizontal",
                DitherPattern::Vertical => "vertical",
                DitherPattern::Noise => "noise",
            };
            format!(
                "Dithered gradient from {} to {} ({}, {} pattern)",
                from, to, dir_name, pattern_name
            )
        }

        // Sub-pixel
        Transform::Subpixel { x, y } => {
            let x_pct = (x * 100.0) as i32;
            let y_pct = (y * 100.0) as i32;
            format!(
                "Sub-pixel shift: {}% right, {}% down (smooth motion via color blending)",
                x_pct, y_pct
            )
        }
    }
}

/// Parse transform from string syntax: "mirror-h", "rotate:90", "tile:3x2"
///
/// # Alias Resolution
/// - `symmetry-h`, `flip-h` → `MirrorH`
/// - `symmetry-v`, `flip-v` → `MirrorV`
/// - `rot` → `Rotate`
pub fn parse_transform_str(s: &str) -> Result<Transform, TransformError> {
    let s = s.trim();

    // Split on colon to get operation and params
    let (op, params) =
        if let Some(idx) = s.find(':') { (&s[..idx], Some(&s[idx + 1..])) } else { (s, None) };

    let op_lower = op.to_lowercase();

    match op_lower.as_str() {
        // Geometric - with aliases
        "mirror-h" | "symmetry-h" | "flip-h" | "mirrorh" => Ok(Transform::MirrorH),
        "mirror-v" | "symmetry-v" | "flip-v" | "mirrorv" => Ok(Transform::MirrorV),
        "rotate" | "rot" => {
            let degrees = params
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "rotate".to_string(),
                    param: "degrees".to_string(),
                })?
                .parse::<u16>()
                .map_err(|_| TransformError::InvalidParameter {
                    op: "rotate".to_string(),
                    message: format!("cannot parse '{}' as degrees", params.unwrap_or("")),
                })?;
            validate_rotation(degrees)?;
            Ok(Transform::Rotate { degrees })
        }

        // Expansion
        "tile" => {
            let dims = params.ok_or_else(|| TransformError::MissingParameter {
                op: "tile".to_string(),
                param: "WxH".to_string(),
            })?;
            let (w, h) = parse_dimensions(dims)?;
            Ok(Transform::Tile { w, h })
        }
        "pad" => {
            let size = params
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "pad".to_string(),
                    param: "size".to_string(),
                })?
                .parse::<u32>()
                .map_err(|_| TransformError::InvalidParameter {
                    op: "pad".to_string(),
                    message: format!("cannot parse '{}' as size", params.unwrap_or("")),
                })?;
            Ok(Transform::Pad { size })
        }
        "crop" => {
            let region = params.ok_or_else(|| TransformError::MissingParameter {
                op: "crop".to_string(),
                param: "X,Y,W,H".to_string(),
            })?;
            let (x, y, w, h) = parse_crop_region(region)?;
            Ok(Transform::Crop { x, y, w, h })
        }

        // Effects
        "outline" => {
            let (token, width) =
                if let Some(p) = params { parse_outline_params(p)? } else { (None, 1) };
            Ok(Transform::Outline { token, width })
        }
        "shift" => {
            let shift_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "shift".to_string(),
                param: "X,Y".to_string(),
            })?;
            let (x, y) = parse_shift(shift_params)?;
            Ok(Transform::Shift { x, y })
        }
        "shadow" => {
            let shadow_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "shadow".to_string(),
                param: "X,Y[,token]".to_string(),
            })?;
            let (x, y, token) = parse_shadow_params(shadow_params)?;
            Ok(Transform::Shadow { x, y, token })
        }
        "sel-out" | "selout" => {
            // String syntax: "sel-out" or "sel-out:{fallback_token}"
            let fallback = params.map(|p| p.trim().to_string());
            Ok(Transform::SelOut { fallback, mapping: None })
        }
        "scale" => {
            // String syntax: "scale:X,Y" e.g., "scale:1.2,0.8"
            let scale_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "scale".to_string(),
                param: "X,Y".to_string(),
            })?;
            let (x, y) = parse_scale_params(scale_params)?;
            Ok(Transform::Scale { x, y })
        }
        "skew-x" | "skewx" => {
            // String syntax: "skew-x:ANGLE" e.g., "skew-x:20"
            let angle_str = params.ok_or_else(|| TransformError::MissingParameter {
                op: "skew-x".to_string(),
                param: "angle".to_string(),
            })?;
            let degrees = parse_angle(angle_str)?;
            Ok(Transform::SkewX { degrees })
        }
        "skew-y" | "skewy" => {
            // String syntax: "skew-y:ANGLE" e.g., "skew-y:20"
            let angle_str = params.ok_or_else(|| TransformError::MissingParameter {
                op: "skew-y".to_string(),
                param: "angle".to_string(),
            })?;
            let degrees = parse_angle(angle_str)?;
            Ok(Transform::SkewY { degrees })
        }

        // Animation
        "pingpong" => {
            let exclude_ends = params.map(|p| p == "true" || p == "exclude_ends").unwrap_or(false);
            Ok(Transform::Pingpong { exclude_ends })
        }
        "reverse" => Ok(Transform::Reverse),
        "frame-offset" | "frameoffset" => {
            let offset = params
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "frame-offset".to_string(),
                    param: "offset".to_string(),
                })?
                .parse::<i32>()
                .map_err(|_| TransformError::InvalidParameter {
                    op: "frame-offset".to_string(),
                    message: format!("cannot parse '{}' as offset", params.unwrap_or("")),
                })?;
            Ok(Transform::FrameOffset { offset })
        }
        "hold" => {
            let hold_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "hold".to_string(),
                param: "frame,count".to_string(),
            })?;
            let (frame, count) = parse_hold_params(hold_params)?;
            Ok(Transform::Hold { frame, count })
        }

        // Dithering (ATF-8)
        // String syntax: dither:pattern:dark_token,light_token[:threshold]
        // Example: dither:checker:{dark},{light}:0.5
        "dither" => {
            let dither_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "dither".to_string(),
                param: "pattern:dark_token,light_token".to_string(),
            })?;
            parse_dither_str(dither_params)
        }

        // String syntax: dither-gradient:direction:from_token,to_token[:pattern]
        // Example: dither-gradient:vertical:{sky_light},{sky_dark}:ordered-4x4
        "dither-gradient" | "dithergradient" => {
            let gradient_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "dither-gradient".to_string(),
                param: "direction:from_token,to_token".to_string(),
            })?;
            parse_dither_gradient_str(gradient_params)
        }

        // Sub-pixel Animation (ATF-13)
        // String syntax: subpixel:x,y (values 0.0-1.0)
        // Example: subpixel:0.5,0.25
        "subpixel" | "sub-pixel" | "subpixel-shift" => {
            let subpixel_params = params.ok_or_else(|| TransformError::MissingParameter {
                op: "subpixel".to_string(),
                param: "x,y".to_string(),
            })?;
            parse_subpixel_str(subpixel_params)
        }

        _ => Err(TransformError::UnknownOperation(op.to_string())),
    }
}

/// Parse transform from JSON value (string or object)
///
/// Supports both syntaxes:
/// - String: `"mirror-h"`, `"rotate:90"`, `"tile:3x2"`
/// - Object: `{"op": "tile", "w": 3, "h": 2}`
pub fn parse_transform_value(value: &Value) -> Result<Transform, TransformError> {
    match value {
        Value::String(s) => parse_transform_str(s),
        Value::Object(obj) => {
            let op = obj
                .get("op")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TransformError::ParseError("missing 'op' field".to_string()))?;

            let params: HashMap<String, Value> = obj
                .iter()
                .filter(|(k, _)| *k != "op")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            parse_transform_object(op, &params)
        }
        _ => Err(TransformError::ParseError("transform must be string or object".to_string())),
    }
}

/// Parse transform from object syntax with explicit parameters
fn parse_transform_object(
    op: &str,
    params: &HashMap<String, Value>,
) -> Result<Transform, TransformError> {
    let op_lower = op.to_lowercase();

    match op_lower.as_str() {
        // Geometric
        "mirror-h" | "symmetry-h" | "flip-h" | "mirrorh" => Ok(Transform::MirrorH),
        "mirror-v" | "symmetry-v" | "flip-v" | "mirrorv" => Ok(Transform::MirrorV),
        "rotate" | "rot" => {
            let degrees = get_u16_param(params, "degrees", "rotate")?;
            validate_rotation(degrees)?;
            Ok(Transform::Rotate { degrees })
        }

        // Expansion
        "tile" => {
            let w = get_u32_param(params, "w", "tile")?;
            let h = get_u32_param(params, "h", "tile")?;
            Ok(Transform::Tile { w, h })
        }
        "pad" => {
            let size = get_u32_param(params, "size", "pad")?;
            Ok(Transform::Pad { size })
        }
        "crop" => {
            let x = get_u32_param(params, "x", "crop")?;
            let y = get_u32_param(params, "y", "crop")?;
            let w = get_u32_param(params, "w", "crop")?;
            let h = get_u32_param(params, "h", "crop")?;
            Ok(Transform::Crop { x, y, w, h })
        }

        // Effects
        "outline" => {
            let token = params.get("token").and_then(|v| v.as_str()).map(String::from);
            let width = params.get("width").and_then(|v| v.as_u64()).map(|v| v as u32).unwrap_or(1);
            Ok(Transform::Outline { token, width })
        }
        "shift" => {
            let x = get_i32_param(params, "x", "shift")?;
            let y = get_i32_param(params, "y", "shift")?;
            Ok(Transform::Shift { x, y })
        }
        "shadow" => {
            let x = get_i32_param(params, "x", "shadow")?;
            let y = get_i32_param(params, "y", "shadow")?;
            let token = params.get("token").and_then(|v| v.as_str()).map(String::from);
            Ok(Transform::Shadow { x, y, token })
        }
        "sel-out" | "selout" => {
            let fallback = params.get("fallback").and_then(|v| v.as_str()).map(String::from);
            let mapping = params.get("mapping").and_then(|v| {
                v.as_object().map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<String, String>>()
                })
            });
            Ok(Transform::SelOut { fallback, mapping })
        }
        "scale" => {
            let x =
                params.get("x").and_then(|v| v.as_f64()).map(|v| v as f32).ok_or_else(|| {
                    TransformError::MissingParameter {
                        op: "scale".to_string(),
                        param: "x".to_string(),
                    }
                })?;
            let y =
                params.get("y").and_then(|v| v.as_f64()).map(|v| v as f32).ok_or_else(|| {
                    TransformError::MissingParameter {
                        op: "scale".to_string(),
                        param: "y".to_string(),
                    }
                })?;

            // Validate scale factors are positive
            if x <= 0.0 || y <= 0.0 {
                return Err(TransformError::InvalidParameter {
                    op: "scale".to_string(),
                    message: "scale factors must be positive".to_string(),
                });
            }

            Ok(Transform::Scale { x, y })
        }
        "skew-x" | "skewx" => {
            let degrees =
                params.get("degrees").and_then(|v| v.as_f64()).map(|v| v as f32).ok_or_else(
                    || TransformError::MissingParameter {
                        op: "skew-x".to_string(),
                        param: "degrees".to_string(),
                    },
                )?;

            // Validate angle is within reasonable bounds
            if degrees.abs() >= 89.0 {
                return Err(TransformError::InvalidParameter {
                    op: "skew-x".to_string(),
                    message: "skew angle must be between -89 and 89 degrees".to_string(),
                });
            }

            Ok(Transform::SkewX { degrees })
        }
        "skew-y" | "skewy" => {
            let degrees =
                params.get("degrees").and_then(|v| v.as_f64()).map(|v| v as f32).ok_or_else(
                    || TransformError::MissingParameter {
                        op: "skew-y".to_string(),
                        param: "degrees".to_string(),
                    },
                )?;

            // Validate angle is within reasonable bounds
            if degrees.abs() >= 89.0 {
                return Err(TransformError::InvalidParameter {
                    op: "skew-y".to_string(),
                    message: "skew angle must be between -89 and 89 degrees".to_string(),
                });
            }

            Ok(Transform::SkewY { degrees })
        }

        // Animation
        "pingpong" => {
            let exclude_ends =
                params.get("exclude_ends").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(Transform::Pingpong { exclude_ends })
        }
        "reverse" => Ok(Transform::Reverse),
        "frame-offset" | "frameoffset" => {
            let offset = get_i32_param(params, "offset", "frame-offset")?;
            Ok(Transform::FrameOffset { offset })
        }
        "hold" => {
            let frame =
                params.get("frame").and_then(|v| v.as_u64()).map(|v| v as usize).ok_or_else(
                    || TransformError::MissingParameter {
                        op: "hold".to_string(),
                        param: "frame".to_string(),
                    },
                )?;
            let count =
                params.get("count").and_then(|v| v.as_u64()).map(|v| v as usize).ok_or_else(
                    || TransformError::MissingParameter {
                        op: "hold".to_string(),
                        param: "count".to_string(),
                    },
                )?;
            Ok(Transform::Hold { frame, count })
        }

        // Dithering (ATF-8)
        "dither" => {
            let pattern_str = params.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| {
                TransformError::MissingParameter {
                    op: "dither".to_string(),
                    param: "pattern".to_string(),
                }
            })?;
            let pattern = DitherPattern::from_str(pattern_str).ok_or_else(|| {
                TransformError::InvalidParameter {
                    op: "dither".to_string(),
                    message: format!("unknown dither pattern: {}", pattern_str),
                }
            })?;

            let tokens_arr = params.get("tokens").and_then(|v| v.as_array()).ok_or_else(|| {
                TransformError::MissingParameter {
                    op: "dither".to_string(),
                    param: "tokens".to_string(),
                }
            })?;
            if tokens_arr.len() != 2 {
                return Err(TransformError::InvalidParameter {
                    op: "dither".to_string(),
                    message: format!(
                        "tokens must have exactly 2 elements, got {}",
                        tokens_arr.len()
                    ),
                });
            }
            let dark_token = tokens_arr[0]
                .as_str()
                .ok_or_else(|| TransformError::InvalidParameter {
                    op: "dither".to_string(),
                    message: "first token must be a string".to_string(),
                })?
                .to_string();
            let light_token = tokens_arr[1]
                .as_str()
                .ok_or_else(|| TransformError::InvalidParameter {
                    op: "dither".to_string(),
                    message: "second token must be a string".to_string(),
                })?
                .to_string();

            let threshold = params.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.5);
            let seed = params.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

            Ok(Transform::Dither { pattern, tokens: (dark_token, light_token), threshold, seed })
        }

        "dither-gradient" | "dithergradient" => {
            let direction_str =
                params.get("direction").and_then(|v| v.as_str()).unwrap_or("vertical");
            let direction = GradientDirection::from_str(direction_str).ok_or_else(|| {
                TransformError::InvalidParameter {
                    op: "dither-gradient".to_string(),
                    message: format!("unknown direction: {}", direction_str),
                }
            })?;

            let from = params
                .get("from")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "dither-gradient".to_string(),
                    param: "from".to_string(),
                })?
                .to_string();
            let to = params
                .get("to")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "dither-gradient".to_string(),
                    param: "to".to_string(),
                })?
                .to_string();

            let pattern_str =
                params.get("pattern").and_then(|v| v.as_str()).unwrap_or("ordered-4x4");
            let pattern = DitherPattern::from_str(pattern_str).ok_or_else(|| {
                TransformError::InvalidParameter {
                    op: "dither-gradient".to_string(),
                    message: format!("unknown dither pattern: {}", pattern_str),
                }
            })?;

            Ok(Transform::DitherGradient { direction, from, to, pattern })
        }

        // Sub-pixel Animation (ATF-13)
        "subpixel" | "sub-pixel" | "subpixel-shift" => {
            let x = params
                .get("x")
                .or_else(|| params.get("subpixel-x"))
                .or_else(|| params.get("subpixel_x"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let y = params
                .get("y")
                .or_else(|| params.get("subpixel-y"))
                .or_else(|| params.get("subpixel_y"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            Ok(Transform::Subpixel { x, y })
        }

        _ => Err(TransformError::UnknownOperation(op.to_string())),
    }
}

// Helper functions for parameter parsing

fn validate_rotation(degrees: u16) -> Result<(), TransformError> {
    if degrees != 90 && degrees != 180 && degrees != 270 {
        return Err(TransformError::InvalidRotation(degrees));
    }
    Ok(())
}

fn parse_dimensions(s: &str) -> Result<(u32, u32), TransformError> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err(TransformError::InvalidTileDimensions(s.to_string()));
    }
    let w = parts[0]
        .parse::<u32>()
        .map_err(|_| TransformError::InvalidTileDimensions(s.to_string()))?;
    let h = parts[1]
        .parse::<u32>()
        .map_err(|_| TransformError::InvalidTileDimensions(s.to_string()))?;
    Ok((w, h))
}

fn parse_crop_region(s: &str) -> Result<(u32, u32, u32, u32), TransformError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err(TransformError::InvalidCropRegion(s.to_string()));
    }
    let x = parts[0]
        .trim()
        .parse::<u32>()
        .map_err(|_| TransformError::InvalidCropRegion(s.to_string()))?;
    let y = parts[1]
        .trim()
        .parse::<u32>()
        .map_err(|_| TransformError::InvalidCropRegion(s.to_string()))?;
    let w = parts[2]
        .trim()
        .parse::<u32>()
        .map_err(|_| TransformError::InvalidCropRegion(s.to_string()))?;
    let h = parts[3]
        .trim()
        .parse::<u32>()
        .map_err(|_| TransformError::InvalidCropRegion(s.to_string()))?;
    Ok((x, y, w, h))
}

fn parse_shift(s: &str) -> Result<(i32, i32), TransformError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(TransformError::InvalidShift(s.to_string()));
    }
    let x =
        parts[0].trim().parse::<i32>().map_err(|_| TransformError::InvalidShift(s.to_string()))?;
    let y =
        parts[1].trim().parse::<i32>().map_err(|_| TransformError::InvalidShift(s.to_string()))?;
    Ok((x, y))
}

fn parse_outline_params(s: &str) -> Result<(Option<String>, u32), TransformError> {
    // Format: "token" or "token,width" or just "width"
    let parts: Vec<&str> = s.split(',').collect();
    match parts.len() {
        1 => {
            // Could be token or width
            if let Ok(width) = parts[0].trim().parse::<u32>() {
                Ok((None, width))
            } else {
                Ok((Some(parts[0].trim().to_string()), 1))
            }
        }
        2 => {
            let token = Some(parts[0].trim().to_string());
            let width =
                parts[1].trim().parse::<u32>().map_err(|_| TransformError::InvalidParameter {
                    op: "outline".to_string(),
                    message: format!("cannot parse '{}' as width", parts[1]),
                })?;
            Ok((token, width))
        }
        _ => Err(TransformError::InvalidParameter {
            op: "outline".to_string(),
            message: format!("expected 'token' or 'token,width', got '{}'", s),
        }),
    }
}

fn parse_shadow_params(s: &str) -> Result<(i32, i32, Option<String>), TransformError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() < 2 {
        return Err(TransformError::InvalidParameter {
            op: "shadow".to_string(),
            message: format!("expected 'X,Y[,token]', got '{}'", s),
        });
    }
    let x = parts[0].trim().parse::<i32>().map_err(|_| TransformError::InvalidParameter {
        op: "shadow".to_string(),
        message: format!("cannot parse '{}' as X offset", parts[0]),
    })?;
    let y = parts[1].trim().parse::<i32>().map_err(|_| TransformError::InvalidParameter {
        op: "shadow".to_string(),
        message: format!("cannot parse '{}' as Y offset", parts[1]),
    })?;
    let token = if parts.len() > 2 { Some(parts[2].trim().to_string()) } else { None };
    Ok((x, y, token))
}

fn parse_scale_params(s: &str) -> Result<(f32, f32), TransformError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(TransformError::InvalidParameter {
            op: "scale".to_string(),
            message: format!("expected 'X,Y', got '{}'", s),
        });
    }
    let x = parts[0].trim().parse::<f32>().map_err(|_| TransformError::InvalidParameter {
        op: "scale".to_string(),
        message: format!("cannot parse '{}' as X scale factor", parts[0]),
    })?;
    let y = parts[1].trim().parse::<f32>().map_err(|_| TransformError::InvalidParameter {
        op: "scale".to_string(),
        message: format!("cannot parse '{}' as Y scale factor", parts[1]),
    })?;

    // Validate scale factors are positive
    if x <= 0.0 || y <= 0.0 {
        return Err(TransformError::InvalidParameter {
            op: "scale".to_string(),
            message: "scale factors must be positive".to_string(),
        });
    }

    Ok((x, y))
}

/// Parse an angle value from a string.
///
/// Accepts formats:
/// - "20" - degrees without suffix
/// - "20deg" - degrees with suffix
/// - "20°" - degrees with degree symbol
///
/// Skew angles are limited to -89 to 89 degrees (tangent approaches infinity at 90°).
fn parse_angle(s: &str) -> Result<f32, TransformError> {
    // Remove common suffixes
    let s = s.trim();
    let s = s.strip_suffix("deg").unwrap_or(s);
    let s = s.strip_suffix('°').unwrap_or(s);
    let s = s.trim();

    let degrees = s.parse::<f32>().map_err(|_| TransformError::InvalidParameter {
        op: "skew".to_string(),
        message: format!("cannot parse '{}' as angle", s),
    })?;

    // Validate angle is within reasonable bounds
    if degrees.abs() >= 89.0 {
        return Err(TransformError::InvalidParameter {
            op: "skew".to_string(),
            message: "skew angle must be between -89 and 89 degrees".to_string(),
        });
    }

    Ok(degrees)
}

fn parse_hold_params(s: &str) -> Result<(usize, usize), TransformError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(TransformError::InvalidParameter {
            op: "hold".to_string(),
            message: format!("expected 'frame,count', got '{}'", s),
        });
    }
    let frame = parts[0].trim().parse::<usize>().map_err(|_| TransformError::InvalidParameter {
        op: "hold".to_string(),
        message: format!("cannot parse '{}' as frame index", parts[0]),
    })?;
    let count = parts[1].trim().parse::<usize>().map_err(|_| TransformError::InvalidParameter {
        op: "hold".to_string(),
        message: format!("cannot parse '{}' as count", parts[1]),
    })?;
    Ok((frame, count))
}

/// Parse dither from string syntax: pattern:dark_token,light_token[:threshold[:seed]]
/// Example: "checker:{dark},{light}:0.5"
fn parse_dither_str(s: &str) -> Result<Transform, TransformError> {
    // Split by colon, but be careful with tokens that contain colons (they shouldn't, but...)
    let parts: Vec<&str> = s.splitn(4, ':').collect();
    if parts.is_empty() {
        return Err(TransformError::MissingParameter {
            op: "dither".to_string(),
            param: "pattern".to_string(),
        });
    }

    // First part is pattern
    let pattern =
        DitherPattern::from_str(parts[0]).ok_or_else(|| TransformError::InvalidParameter {
            op: "dither".to_string(),
            message: format!("unknown dither pattern: {}", parts[0]),
        })?;

    // Second part should be tokens: dark,light
    if parts.len() < 2 {
        return Err(TransformError::MissingParameter {
            op: "dither".to_string(),
            param: "tokens (dark,light)".to_string(),
        });
    }
    let tokens_str = parts[1];
    // Find the comma that separates the two tokens
    // Tokens might be like {dark},{light} so we need to handle braces
    let (dark_token, light_token) =
        parse_token_pair(tokens_str).ok_or_else(|| TransformError::InvalidParameter {
            op: "dither".to_string(),
            message: format!("expected 'dark_token,light_token', got '{}'", tokens_str),
        })?;

    // Third part (optional) is threshold
    let threshold = if parts.len() >= 3 {
        parts[2].trim().parse::<f64>().map_err(|_| TransformError::InvalidParameter {
            op: "dither".to_string(),
            message: format!("cannot parse '{}' as threshold", parts[2]),
        })?
    } else {
        0.5
    };

    // Fourth part (optional) is seed
    let seed = if parts.len() >= 4 {
        parts[3].trim().parse::<u64>().map_err(|_| TransformError::InvalidParameter {
            op: "dither".to_string(),
            message: format!("cannot parse '{}' as seed", parts[3]),
        })?
    } else {
        0
    };

    Ok(Transform::Dither { pattern, tokens: (dark_token, light_token), threshold, seed })
}

/// Parse dither-gradient from string: direction:from_token,to_token[:pattern]
/// Example: "vertical:{sky_light},{sky_dark}:ordered-4x4"
fn parse_dither_gradient_str(s: &str) -> Result<Transform, TransformError> {
    let parts: Vec<&str> = s.splitn(4, ':').collect();
    if parts.is_empty() {
        return Err(TransformError::MissingParameter {
            op: "dither-gradient".to_string(),
            param: "direction".to_string(),
        });
    }

    // First part is direction
    let direction =
        GradientDirection::from_str(parts[0]).ok_or_else(|| TransformError::InvalidParameter {
            op: "dither-gradient".to_string(),
            message: format!(
                "unknown direction: {} (expected vertical, horizontal, or radial)",
                parts[0]
            ),
        })?;

    // Second part should be tokens: from,to
    if parts.len() < 2 {
        return Err(TransformError::MissingParameter {
            op: "dither-gradient".to_string(),
            param: "tokens (from,to)".to_string(),
        });
    }
    let tokens_str = parts[1];
    let (from, to) =
        parse_token_pair(tokens_str).ok_or_else(|| TransformError::InvalidParameter {
            op: "dither-gradient".to_string(),
            message: format!("expected 'from_token,to_token', got '{}'", tokens_str),
        })?;

    // Third part (optional) is pattern
    let pattern = if parts.len() >= 3 {
        DitherPattern::from_str(parts[2]).ok_or_else(|| TransformError::InvalidParameter {
            op: "dither-gradient".to_string(),
            message: format!("unknown dither pattern: {}", parts[2]),
        })?
    } else {
        DitherPattern::Ordered4x4 // Default pattern
    };

    Ok(Transform::DitherGradient { direction, from, to, pattern })
}

/// Parse subpixel from string syntax: x,y (values 0.0-1.0)
/// Example: "0.5,0.25"
fn parse_subpixel_str(s: &str) -> Result<Transform, TransformError> {
    let parts: Vec<&str> = s.split(',').collect();

    let x = if !parts.is_empty() && !parts[0].is_empty() {
        parts[0].trim().parse::<f64>().map_err(|_| TransformError::InvalidParameter {
            op: "subpixel".to_string(),
            message: format!("cannot parse '{}' as x offset", parts[0]),
        })?
    } else {
        0.0
    };

    let y = if parts.len() > 1 && !parts[1].is_empty() {
        parts[1].trim().parse::<f64>().map_err(|_| TransformError::InvalidParameter {
            op: "subpixel".to_string(),
            message: format!("cannot parse '{}' as y offset", parts[1]),
        })?
    } else {
        0.0
    };

    Ok(Transform::Subpixel { x, y })
}

/// Parse a token pair like "{dark},{light}" or "dark,light"
/// Handles braces correctly for tokens like {token_name}
fn parse_token_pair(s: &str) -> Option<(String, String)> {
    let s = s.trim();

    // Find the comma that separates the two tokens
    // We need to handle braces: {foo},{bar} should split on the comma between them
    let mut depth: i32 = 0;
    let mut split_pos = None;

    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                split_pos = Some(i);
                break;
            }
            _ => {}
        }
    }

    let pos = split_pos?;
    let first = s[..pos].trim().to_string();
    let second = s[pos + 1..].trim().to_string();

    if first.is_empty() || second.is_empty() {
        return None;
    }

    Some((first, second))
}

fn get_u16_param(
    params: &HashMap<String, Value>,
    key: &str,
    op: &str,
) -> Result<u16, TransformError> {
    params.get(key).and_then(|v| v.as_u64()).map(|v| v as u16).ok_or_else(|| {
        TransformError::MissingParameter { op: op.to_string(), param: key.to_string() }
    })
}

fn get_u32_param(
    params: &HashMap<String, Value>,
    key: &str,
    op: &str,
) -> Result<u32, TransformError> {
    params.get(key).and_then(|v| v.as_u64()).map(|v| v as u32).ok_or_else(|| {
        TransformError::MissingParameter { op: op.to_string(), param: key.to_string() }
    })
}

fn get_i32_param(
    params: &HashMap<String, Value>,
    key: &str,
    op: &str,
) -> Result<i32, TransformError> {
    params.get(key).and_then(|v| v.as_i64()).map(|v| v as i32).ok_or_else(|| {
        TransformError::MissingParameter { op: op.to_string(), param: key.to_string() }
    })
}

// ============================================================================
// CSS Transform Parsing (CSS-14)
// ============================================================================

/// CSS transform representation for parsing CSS-style transform strings.
///
/// Supports CSS transform functions: `translate()`, `rotate()`, `scale()`, `flip()`.
/// Multiple transforms can be chained in a single string.
///
/// # Example
///
/// ```
/// use pixelsrc::transforms::{parse_css_transform, CssTransform};
///
/// let transform = parse_css_transform("translate(10, 5) rotate(90deg) scale(2)").unwrap();
/// assert_eq!(transform.translate, Some((10, 5)));
/// assert_eq!(transform.rotate, Some(90.0));
/// assert_eq!(transform.scale, Some((2.0, 2.0)));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CssTransform {
    /// Translation offset (x, y) in pixels
    pub translate: Option<(i32, i32)>,
    /// Rotation in degrees (positive = clockwise)
    pub rotate: Option<f64>,
    /// Scale factors (x, y)
    pub scale: Option<(f64, f64)>,
    /// Skew along X axis in degrees
    pub skew_x: Option<f64>,
    /// Skew along Y axis in degrees
    pub skew_y: Option<f64>,
    /// Flip horizontally
    pub flip_x: bool,
    /// Flip vertically
    pub flip_y: bool,
}

/// Error type for CSS transform parsing failures
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CssTransformError {
    /// Unknown transform function
    #[error("unknown CSS transform function: {0}")]
    UnknownFunction(String),
    /// Invalid parameter value
    #[error("invalid parameter for {func}(): {message}")]
    InvalidParameter { func: String, message: String },
    /// Missing required parameter
    #[error("missing required parameter for {func}(): {param}")]
    MissingParameter { func: String, param: String },
    /// Invalid rotation value (for pixel art, must be 90, 180, or 270)
    #[error("invalid rotation: {0}deg (pixel art requires 90, 180, or 270)")]
    InvalidRotation(f64),
    /// Syntax error
    #[error("CSS transform syntax error: {0}")]
    SyntaxError(String),
}

impl CssTransform {
    /// Create a new empty CSS transform
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert this CSS transform into a sequence of Transform operations.
    ///
    /// The order of operations follows CSS transform order: translate, rotate, scale, flip.
    /// For pixel art, rotation is restricted to 90, 180, or 270 degrees.
    ///
    /// # Returns
    ///
    /// A Vec of Transform operations that can be applied to a sprite grid.
    pub fn to_transforms(&self) -> Result<Vec<Transform>, CssTransformError> {
        let mut transforms = Vec::new();

        // Apply in CSS order: translate → rotate → scale → flip
        if let Some((x, y)) = self.translate {
            transforms.push(Transform::Shift { x, y });
        }

        if let Some(degrees) = self.rotate {
            // For pixel art, we only support 90, 180, 270 degree rotations
            let deg_normalized = degrees.rem_euclid(360.0);
            let deg_i32 = deg_normalized.round() as i32;
            match deg_i32 {
                0 => {} // No rotation needed
                90 => transforms.push(Transform::Rotate { degrees: 90 }),
                180 => transforms.push(Transform::Rotate { degrees: 180 }),
                270 => transforms.push(Transform::Rotate { degrees: 270 }),
                _ => return Err(CssTransformError::InvalidRotation(degrees)),
            }
        }

        if let Some((x, y)) = self.scale {
            transforms.push(Transform::Scale { x: x as f32, y: y as f32 });
        }

        if let Some(degrees) = self.skew_x {
            transforms.push(Transform::SkewX { degrees: degrees as f32 });
        }
        if let Some(degrees) = self.skew_y {
            transforms.push(Transform::SkewY { degrees: degrees as f32 });
        }

        if self.flip_x {
            transforms.push(Transform::MirrorH);
        }
        if self.flip_y {
            transforms.push(Transform::MirrorV);
        }

        Ok(transforms)
    }

    /// Check if this transform is empty (no operations)
    pub fn is_empty(&self) -> bool {
        self.translate.is_none()
            && self.rotate.is_none()
            && self.scale.is_none()
            && self.skew_x.is_none()
            && self.skew_y.is_none()
            && !self.flip_x
            && !self.flip_y
    }
}

/// Parse a CSS transform string into a CssTransform struct.
///
/// Supports CSS transform functions:
/// - `translate(x, y)` - Translation in pixels (integers)
/// - `rotate(deg)` - Rotation in degrees (90, 180, or 270 for pixel art)
/// - `scale(n)` or `scale(x, y)` - Uniform or non-uniform scaling
/// - `flip(x)` or `flip(y)` - Horizontal or vertical flip
/// - `skewX(deg)` or `skewY(deg)` - Horizontal or vertical skew
/// - `skew(x, y)` - Combined skew (x required, y optional)
///
/// # Arguments
///
/// * `css` - CSS transform string, e.g., "translate(10, 5) rotate(90deg)"
///
/// # Returns
///
/// A CssTransform struct with parsed values, or an error if parsing fails.
///
/// # Example
///
/// ```
/// use pixelsrc::transforms::parse_css_transform;
///
/// // Single transform
/// let t = parse_css_transform("rotate(90deg)").unwrap();
/// assert_eq!(t.rotate, Some(90.0));
///
/// // Multiple transforms
/// let t = parse_css_transform("translate(5, 10) scale(2)").unwrap();
/// assert_eq!(t.translate, Some((5, 10)));
/// assert_eq!(t.scale, Some((2.0, 2.0)));
///
/// // Flip transforms
/// let t = parse_css_transform("flip(x) flip(y)").unwrap();
/// assert!(t.flip_x);
/// assert!(t.flip_y);
///
/// // Skew transforms for isometric sprites
/// let t = parse_css_transform("skewX(26.57deg)").unwrap();
/// assert_eq!(t.skew_x, Some(26.57));
/// ```
pub fn parse_css_transform(css: &str) -> Result<CssTransform, CssTransformError> {
    let mut result = CssTransform::new();
    let css = css.trim();

    if css.is_empty() {
        return Ok(result);
    }

    // Parse CSS transform functions: func(args) func(args) ...
    // Use a simple state machine to extract function calls
    let mut pos = 0;
    let chars: Vec<char> = css.chars().collect();

    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }

        // Read function name
        let func_start = pos;
        while pos < chars.len() && chars[pos] != '(' && !chars[pos].is_whitespace() {
            pos += 1;
        }
        let func_name: String = chars[func_start..pos].iter().collect();
        let func_name = func_name.to_lowercase();

        // Skip whitespace before '('
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // Expect '('
        if pos >= chars.len() || chars[pos] != '(' {
            return Err(CssTransformError::SyntaxError(format!(
                "expected '(' after '{}', got '{}'",
                func_name,
                if pos < chars.len() {
                    chars[pos].to_string()
                } else {
                    "end of string".to_string()
                }
            )));
        }
        pos += 1; // Skip '('

        // Find matching ')'
        let args_start = pos;
        let mut paren_depth = 1;
        while pos < chars.len() && paren_depth > 0 {
            match chars[pos] {
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                _ => {}
            }
            if paren_depth > 0 {
                pos += 1;
            }
        }

        if paren_depth != 0 {
            return Err(CssTransformError::SyntaxError("unmatched parentheses".to_string()));
        }

        let args: String = chars[args_start..pos].iter().collect();
        pos += 1; // Skip ')'

        // Parse the function
        match func_name.as_str() {
            "translate" => {
                let (x, y) = parse_css_translate(&args)?;
                result.translate = Some((x, y));
            }
            "rotate" => {
                let deg = parse_css_rotate(&args)?;
                result.rotate = Some(deg);
            }
            "scale" | "scalex" | "scaley" => {
                let (x, y) = parse_css_scale(&func_name, &args)?;
                result.scale = Some((x, y));
            }
            "flip" | "flipx" | "flipy" => {
                let (fx, fy) = parse_css_flip(&func_name, &args)?;
                if fx {
                    result.flip_x = true;
                }
                if fy {
                    result.flip_y = true;
                }
            }
            "skewx" | "skew-x" => {
                let deg = parse_css_skew_angle(&args, "skewX")?;
                result.skew_x = Some(deg);
            }
            "skewy" | "skew-y" => {
                let deg = parse_css_skew_angle(&args, "skewY")?;
                result.skew_y = Some(deg);
            }
            "skew" => {
                // CSS skew(x) or skew(x, y)
                let (skew_x, skew_y) = parse_css_skew(&args)?;
                result.skew_x = Some(skew_x);
                if let Some(y) = skew_y {
                    result.skew_y = Some(y);
                }
            }
            _ => {
                return Err(CssTransformError::UnknownFunction(func_name));
            }
        }
    }

    Ok(result)
}

/// Parse translate(x, y) arguments
fn parse_css_translate(args: &str) -> Result<(i32, i32), CssTransformError> {
    let args = args.trim();
    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

    if parts.is_empty() || parts[0].is_empty() {
        return Err(CssTransformError::MissingParameter {
            func: "translate".to_string(),
            param: "x".to_string(),
        });
    }

    let x = parse_css_length(parts[0]).map_err(|_| CssTransformError::InvalidParameter {
        func: "translate".to_string(),
        message: format!("cannot parse '{}' as x offset", parts[0]),
    })?;

    let y = if parts.len() > 1 {
        parse_css_length(parts[1]).map_err(|_| CssTransformError::InvalidParameter {
            func: "translate".to_string(),
            message: format!("cannot parse '{}' as y offset", parts[1]),
        })?
    } else {
        0 // CSS defaults to 0 if y is not specified
    };

    Ok((x, y))
}

/// Parse rotate(deg) argument
fn parse_css_rotate(args: &str) -> Result<f64, CssTransformError> {
    let args = args.trim();

    if args.is_empty() {
        return Err(CssTransformError::MissingParameter {
            func: "rotate".to_string(),
            param: "angle".to_string(),
        });
    }

    // Remove 'deg' suffix if present
    let num_str = if args.to_lowercase().ends_with("deg") { &args[..args.len() - 3] } else { args };

    num_str.trim().parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
        func: "rotate".to_string(),
        message: format!("cannot parse '{}' as angle", args),
    })
}

/// Parse scale(n) or scale(x, y) arguments
fn parse_css_scale(func: &str, args: &str) -> Result<(f64, f64), CssTransformError> {
    let args = args.trim();

    if args.is_empty() {
        return Err(CssTransformError::MissingParameter {
            func: func.to_string(),
            param: "factor".to_string(),
        });
    }

    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

    match func {
        "scalex" => {
            let x = parts[0].parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
                func: "scaleX".to_string(),
                message: format!("cannot parse '{}' as scale factor", parts[0]),
            })?;
            Ok((x, 1.0))
        }
        "scaley" => {
            let y = parts[0].parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
                func: "scaleY".to_string(),
                message: format!("cannot parse '{}' as scale factor", parts[0]),
            })?;
            Ok((1.0, y))
        }
        _ => {
            // "scale"
            let x = parts[0].parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
                func: "scale".to_string(),
                message: format!("cannot parse '{}' as scale factor", parts[0]),
            })?;

            let y = if parts.len() > 1 {
                parts[1].parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
                    func: "scale".to_string(),
                    message: format!("cannot parse '{}' as y scale factor", parts[1]),
                })?
            } else {
                x // Uniform scaling if only one value
            };

            if x <= 0.0 || y <= 0.0 {
                return Err(CssTransformError::InvalidParameter {
                    func: "scale".to_string(),
                    message: "scale factors must be positive".to_string(),
                });
            }

            Ok((x, y))
        }
    }
}

/// Parse flip(x) or flip(y) arguments
fn parse_css_flip(func: &str, args: &str) -> Result<(bool, bool), CssTransformError> {
    let args = args.trim().to_lowercase();

    match func {
        "flipx" => Ok((true, false)),
        "flipy" => Ok((false, true)),
        "flip" => {
            if args.is_empty() {
                return Err(CssTransformError::MissingParameter {
                    func: "flip".to_string(),
                    param: "axis (x or y)".to_string(),
                });
            }
            match args.as_str() {
                "x" | "h" | "horizontal" => Ok((true, false)),
                "y" | "v" | "vertical" => Ok((false, true)),
                _ => Err(CssTransformError::InvalidParameter {
                    func: "flip".to_string(),
                    message: format!("unknown axis '{}', expected 'x' or 'y'", args),
                }),
            }
        }
        _ => unreachable!(),
    }
}

/// Parse a CSS length value (with optional 'px' suffix)
fn parse_css_length(s: &str) -> Result<i32, std::num::ParseIntError> {
    let s = s.trim();
    // Remove 'px' suffix if present
    let num_str = if s.to_lowercase().ends_with("px") { &s[..s.len() - 2] } else { s };
    num_str.trim().parse::<i32>()
}

/// Parse a single skew angle from CSS skewX/skewY function argument
fn parse_css_skew_angle(args: &str, func: &str) -> Result<f64, CssTransformError> {
    let args = args.trim();

    if args.is_empty() {
        return Err(CssTransformError::MissingParameter {
            func: func.to_string(),
            param: "angle".to_string(),
        });
    }

    // Parse angle - remove 'deg' suffix if present
    let angle_str = if args.to_lowercase().ends_with("deg") {
        &args[..args.len() - 3]
    } else {
        args
    };

    let degrees = angle_str.trim().parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
        func: func.to_string(),
        message: format!("cannot parse '{}' as angle", args),
    })?;

    // Validate angle is within reasonable bounds (avoid approaching tan(90°))
    if degrees.abs() >= 89.0 {
        return Err(CssTransformError::InvalidParameter {
            func: func.to_string(),
            message: "skew angle must be between -89 and 89 degrees".to_string(),
        });
    }

    Ok(degrees)
}

/// Parse CSS skew(x) or skew(x, y) arguments
fn parse_css_skew(args: &str) -> Result<(f64, Option<f64>), CssTransformError> {
    let args = args.trim();

    if args.is_empty() {
        return Err(CssTransformError::MissingParameter {
            func: "skew".to_string(),
            param: "x angle".to_string(),
        });
    }

    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

    // Parse X angle
    let x_str = if parts[0].to_lowercase().ends_with("deg") {
        &parts[0][..parts[0].len() - 3]
    } else {
        parts[0]
    };

    let x = x_str.trim().parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
        func: "skew".to_string(),
        message: format!("cannot parse '{}' as x angle", parts[0]),
    })?;

    if x.abs() >= 89.0 {
        return Err(CssTransformError::InvalidParameter {
            func: "skew".to_string(),
            message: "skew x angle must be between -89 and 89 degrees".to_string(),
        });
    }

    // Parse optional Y angle
    let y = if parts.len() > 1 {
        let y_str = if parts[1].to_lowercase().ends_with("deg") {
            &parts[1][..parts[1].len() - 3]
        } else {
            parts[1]
        };

        let y_val = y_str.trim().parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
            func: "skew".to_string(),
            message: format!("cannot parse '{}' as y angle", parts[1]),
        })?;

        if y_val.abs() >= 89.0 {
            return Err(CssTransformError::InvalidParameter {
                func: "skew".to_string(),
                message: "skew y angle must be between -89 and 89 degrees".to_string(),
            });
        }

        Some(y_val)
    } else {
        None // CSS skew(x) defaults to y=0, but we use None to indicate only x was specified
    };

    Ok((x, y))
}

// ============================================================================
// Animation Transform Application Functions
// ============================================================================

/// Apply pingpong transform: duplicate frames in reverse order for forward-backward play.
///
/// Given frames [A, B, C], produces:
/// - `exclude_ends=false`: [A, B, C, C, B, A]
/// - `exclude_ends=true`:  [A, B, C, B]
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
/// * `exclude_ends` - If true, don't duplicate first/last frame in reverse
///
/// # Returns
/// A new Vec with the pingpong sequence
pub fn apply_pingpong<T: Clone>(frames: &[T], exclude_ends: bool) -> Vec<T> {
    if frames.is_empty() {
        return Vec::new();
    }
    if frames.len() == 1 {
        return frames.to_vec();
    }

    let mut result = frames.to_vec();

    if exclude_ends {
        // Reverse without first and last: [A, B, C] -> [A, B, C, B]
        // Skip first (0) and last (len-1) when reversing
        for i in (1..frames.len() - 1).rev() {
            result.push(frames[i].clone());
        }
    } else {
        // Full reverse including ends: [A, B, C] -> [A, B, C, C, B, A]
        for i in (0..frames.len()).rev() {
            result.push(frames[i].clone());
        }
    }

    result
}

/// Apply reverse transform: reverse the order of frames.
///
/// Given frames [A, B, C], produces [C, B, A].
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
///
/// # Returns
/// A new Vec with reversed frame order
pub fn apply_reverse<T: Clone>(frames: &[T]) -> Vec<T> {
    frames.iter().rev().cloned().collect()
}

/// Apply frame-offset transform: rotate frames by offset positions.
///
/// Given frames [A, B, C, D] with offset=1, produces [B, C, D, A].
/// Negative offsets rotate in the opposite direction.
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
/// * `offset` - Number of positions to rotate (positive = forward, negative = backward)
///
/// # Returns
/// A new Vec with rotated frame order
pub fn apply_frame_offset<T: Clone>(frames: &[T], offset: i32) -> Vec<T> {
    if frames.is_empty() {
        return Vec::new();
    }

    let len = frames.len() as i32;
    // Normalize offset to positive value within range
    let normalized = ((offset % len) + len) % len;

    let mut result = Vec::with_capacity(frames.len());
    for i in 0..frames.len() {
        let idx = (i as i32 + normalized) % len;
        result.push(frames[idx as usize].clone());
    }
    result
}

/// Apply hold transform: duplicate a specific frame multiple times.
///
/// Given frames [A, B, C] with frame=1 and count=3, produces [A, B, B, B, C].
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
/// * `frame` - Index of the frame to hold (0-based)
/// * `count` - Number of times to repeat the frame (total occurrences)
///
/// # Returns
/// A new Vec with the held frame duplicated, or original if frame index is invalid
pub fn apply_hold<T: Clone>(frames: &[T], frame: usize, count: usize) -> Vec<T> {
    if frames.is_empty() || frame >= frames.len() {
        return frames.to_vec();
    }
    if count == 0 {
        // count=0 means remove the frame
        let mut result = Vec::with_capacity(frames.len().saturating_sub(1));
        for (i, f) in frames.iter().enumerate() {
            if i != frame {
                result.push(f.clone());
            }
        }
        return result;
    }

    let mut result = Vec::with_capacity(frames.len() + count - 1);
    for (i, f) in frames.iter().enumerate() {
        if i == frame {
            // Insert count copies
            for _ in 0..count {
                result.push(f.clone());
            }
        } else {
            result.push(f.clone());
        }
    }
    result
}

/// Apply an animation transform to a list of frames.
///
/// Only animation-specific transforms are applied here. Non-animation transforms
/// (geometric, expansion, effects) return an error.
///
/// # Arguments
/// * `transform` - The transform to apply
/// * `frames` - The animation frames (sprite names)
///
/// # Returns
/// A new Vec with the transform applied, or an error if not an animation transform
pub fn apply_animation_transform<T: Clone>(
    transform: &Transform,
    frames: &[T],
) -> Result<Vec<T>, TransformError> {
    match transform {
        Transform::Pingpong { exclude_ends } => Ok(apply_pingpong(frames, *exclude_ends)),
        Transform::Reverse => Ok(apply_reverse(frames)),
        Transform::FrameOffset { offset } => Ok(apply_frame_offset(frames, *offset)),
        Transform::Hold { frame, count } => Ok(apply_hold(frames, *frame, *count)),
        _ => Err(TransformError::InvalidParameter {
            op: "animation_transform".to_string(),
            message: "transform is not an animation transform".to_string(),
        }),
    }
}

/// Check if a transform is an animation transform.
///
/// Animation transforms only make sense for animations (not sprites):
/// - Pingpong, Reverse, FrameOffset, Hold
pub fn is_animation_transform(transform: &Transform) -> bool {
    matches!(
        transform,
        Transform::Pingpong { .. }
            | Transform::Reverse
            | Transform::FrameOffset { .. }
            | Transform::Hold { .. }
    )
}

// ============================================================================
// User-Defined Transform Support (TRF-10)
// ============================================================================

/// Error type for expression evaluation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExpressionError {
    /// Unknown variable in expression
    #[error("unknown variable: {0}")]
    UnknownVariable(String),
    /// Unknown function in expression
    #[error("unknown function: {0}")]
    UnknownFunction(String),
    /// Invalid syntax
    #[error("syntax error: {0}")]
    SyntaxError(String),
    /// Division by zero
    #[error("division by zero")]
    DivisionByZero,
    /// Invalid number of arguments
    #[error("function {func} expected {expected} arguments, got {got}")]
    WrongArity { func: String, expected: usize, got: usize },
}

/// Simple expression evaluator for keyframe animations.
///
/// Supports:
/// - Variables: `frame`, `t`, `total_frames`, and user-defined params
/// - Operators: `+`, `-`, `*`, `/`, `%`, `^` (power)
/// - Functions: `sin`, `cos`, `tan`, `pow`, `sqrt`, `min`, `max`, `abs`, `floor`, `ceil`, `round`
/// - Parentheses for grouping
/// - Parameter substitution with `${param_name}` syntax
pub struct ExpressionEvaluator {
    variables: std::collections::HashMap<String, f64>,
}

impl ExpressionEvaluator {
    /// Create a new evaluator with the given variables.
    pub fn new(variables: std::collections::HashMap<String, f64>) -> Self {
        Self { variables }
    }

    /// Create an evaluator for keyframe animation with standard variables.
    ///
    /// Sets up `frame`, `t` (normalized 0.0-1.0), and `total_frames`.
    pub fn for_keyframe(frame: u32, total_frames: u32) -> Self {
        let mut vars = std::collections::HashMap::new();
        vars.insert("frame".to_string(), frame as f64);
        vars.insert("total_frames".to_string(), total_frames as f64);
        let t = if total_frames > 1 { frame as f64 / (total_frames - 1) as f64 } else { 0.0 };
        vars.insert("t".to_string(), t);
        Self { variables: vars }
    }

    /// Add a variable to the evaluator.
    pub fn with_var(mut self, name: &str, value: f64) -> Self {
        self.variables.insert(name.to_string(), value);
        self
    }

    /// Add multiple variables from a map.
    pub fn with_vars(mut self, vars: &std::collections::HashMap<String, f64>) -> Self {
        for (k, v) in vars {
            self.variables.insert(k.clone(), *v);
        }
        self
    }

    /// Substitute `${param}` placeholders in the expression.
    fn substitute_params(&self, expr: &str) -> String {
        let mut result = expr.to_string();
        for (name, value) in &self.variables {
            let placeholder = format!("${{{}}}", name);
            result = result.replace(&placeholder, &value.to_string());
        }
        result
    }

    /// Evaluate an expression string.
    pub fn evaluate(&self, expr: &str) -> Result<f64, ExpressionError> {
        let expr = self.substitute_params(expr);
        self.parse_expression(&expr)
    }

    /// Parse and evaluate an expression.
    fn parse_expression(&self, expr: &str) -> Result<f64, ExpressionError> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(ExpressionError::SyntaxError("empty expression".to_string()));
        }

        // Try parsing as a simple number first
        if let Ok(n) = expr.parse::<f64>() {
            return Ok(n);
        }

        // Check for variable reference
        if expr.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return self
                .variables
                .get(expr)
                .copied()
                .ok_or_else(|| ExpressionError::UnknownVariable(expr.to_string()));
        }

        // Handle parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            let inner = &expr[1..expr.len() - 1];
            if self.count_parens(inner) == 0 {
                return self.parse_expression(inner);
            }
        }

        // Handle binary operators (lowest precedence first)
        // We scan right-to-left for left-associativity
        for ops in &[&['+', '-'][..], &['*', '/', '%'][..]] {
            let mut paren_depth = 0;
            let chars: Vec<char> = expr.chars().collect();
            for i in (0..chars.len()).rev() {
                match chars[i] {
                    ')' => paren_depth += 1,
                    '(' => paren_depth -= 1,
                    c if paren_depth == 0 && ops.contains(&c) => {
                        // Handle negative numbers at start
                        if i == 0 && c == '-' {
                            continue;
                        }
                        // Check if this is a binary operator (not unary minus)
                        if i > 0 {
                            let prev = chars[i - 1];
                            if c == '-'
                                && (prev == '('
                                    || prev == '+'
                                    || prev == '-'
                                    || prev == '*'
                                    || prev == '/'
                                    || prev == '%'
                                    || prev == '^'
                                    || prev == ',')
                            {
                                continue;
                            }
                        }
                        let left = &expr[..i];
                        let right = &expr[i + 1..];
                        if !left.is_empty() && !right.is_empty() {
                            let l = self.parse_expression(left)?;
                            let r = self.parse_expression(right)?;
                            return match c {
                                '+' => Ok(l + r),
                                '-' => Ok(l - r),
                                '*' => Ok(l * r),
                                '/' => {
                                    if r == 0.0 {
                                        Err(ExpressionError::DivisionByZero)
                                    } else {
                                        Ok(l / r)
                                    }
                                }
                                '%' => {
                                    if r == 0.0 {
                                        Err(ExpressionError::DivisionByZero)
                                    } else {
                                        Ok(l % r)
                                    }
                                }
                                _ => unreachable!(),
                            };
                        }
                    }
                    _ => {}
                }
            }
        }

        // Handle power operator (^) - right-to-left
        {
            let mut paren_depth = 0;
            let chars: Vec<char> = expr.chars().collect();
            for i in 0..chars.len() {
                match chars[i] {
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    '^' if paren_depth == 0 => {
                        let left = &expr[..i];
                        let right = &expr[i + 1..];
                        if !left.is_empty() && !right.is_empty() {
                            let l = self.parse_expression(left)?;
                            let r = self.parse_expression(right)?;
                            return Ok(l.powf(r));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Handle function calls
        if let Some(paren_pos) = expr.find('(') {
            if expr.ends_with(')') {
                let func_name = expr[..paren_pos].trim();
                let args_str = &expr[paren_pos + 1..expr.len() - 1];
                let args = self.parse_args(args_str)?;
                return self.call_function(func_name, &args);
            }
        }

        // Handle unary minus at start
        if let Some(inner) = expr.strip_prefix('-') {
            return Ok(-self.parse_expression(inner)?);
        }

        Err(ExpressionError::SyntaxError(format!("cannot parse: {}", expr)))
    }

    /// Count unmatched parentheses (positive = more opens, negative = more closes).
    fn count_parens(&self, s: &str) -> i32 {
        let mut count = 0;
        for c in s.chars() {
            match c {
                '(' => count += 1,
                ')' => count -= 1,
                _ => {}
            }
        }
        count
    }

    /// Parse function arguments, handling nested parentheses.
    fn parse_args(&self, args_str: &str) -> Result<Vec<f64>, ExpressionError> {
        if args_str.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut args = Vec::new();
        let mut current = String::new();
        let mut paren_depth = 0;

        for c in args_str.chars() {
            match c {
                '(' => {
                    paren_depth += 1;
                    current.push(c);
                }
                ')' => {
                    paren_depth -= 1;
                    current.push(c);
                }
                ',' if paren_depth == 0 => {
                    args.push(self.parse_expression(&current)?);
                    current.clear();
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            args.push(self.parse_expression(&current)?);
        }

        Ok(args)
    }

    /// Call a built-in function with the given arguments.
    fn call_function(&self, name: &str, args: &[f64]) -> Result<f64, ExpressionError> {
        match name.to_lowercase().as_str() {
            "sin" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].sin())
            }
            "cos" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].cos())
            }
            "tan" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].tan())
            }
            "pow" => {
                self.check_arity(name, args, 2)?;
                Ok(args[0].powf(args[1]))
            }
            "sqrt" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].sqrt())
            }
            "min" => {
                self.check_arity(name, args, 2)?;
                Ok(args[0].min(args[1]))
            }
            "max" => {
                self.check_arity(name, args, 2)?;
                Ok(args[0].max(args[1]))
            }
            "abs" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].abs())
            }
            "floor" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].floor())
            }
            "ceil" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].ceil())
            }
            "round" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].round())
            }
            "clamp" => {
                self.check_arity(name, args, 3)?;
                Ok(args[0].clamp(args[1], args[2]))
            }
            _ => Err(ExpressionError::UnknownFunction(name.to_string())),
        }
    }

    fn check_arity(
        &self,
        name: &str,
        args: &[f64],
        expected: usize,
    ) -> Result<(), ExpressionError> {
        if args.len() != expected {
            Err(ExpressionError::WrongArity { func: name.to_string(), expected, got: args.len() })
        } else {
            Ok(())
        }
    }
}

/// Interpolate between keyframes for a given frame number.
///
/// Given a list of (frame, value) keyframes, interpolates the value at the specified frame
/// using the provided easing function.
pub fn interpolate_keyframes(
    keyframes: &[[f64; 2]],
    frame: f64,
    easing: &crate::models::Easing,
) -> f64 {
    if keyframes.is_empty() {
        return 0.0;
    }

    if keyframes.len() == 1 {
        return keyframes[0][1];
    }

    // Sort keyframes by frame number
    let mut sorted: Vec<[f64; 2]> = keyframes.to_vec();
    sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

    // Before first keyframe
    if frame <= sorted[0][0] {
        return sorted[0][1];
    }

    // After last keyframe
    if frame >= sorted[sorted.len() - 1][0] {
        return sorted[sorted.len() - 1][1];
    }

    // Find surrounding keyframes
    for i in 0..sorted.len() - 1 {
        let (f1, v1) = (sorted[i][0], sorted[i][1]);
        let (f2, v2) = (sorted[i + 1][0], sorted[i + 1][1]);

        if frame >= f1 && frame <= f2 {
            // Calculate normalized time between keyframes
            let t = if (f2 - f1).abs() < f64::EPSILON { 0.0 } else { (frame - f1) / (f2 - f1) };

            // Apply easing
            let eased_t = easing.apply(t);

            // Linear interpolation with eased t
            return v1 + (v2 - v1) * eased_t;
        }
    }

    // Fallback
    sorted[sorted.len() - 1][1]
}

/// Generate transforms for a specific frame from a user-defined transform.
///
/// This handles:
/// - Expression-based keyframes (evaluates the expression for the frame)
/// - Explicit keyframes (interpolates between them)
/// - Cycling transforms (picks the transform for this frame)
pub fn generate_frame_transforms(
    transform_def: &crate::models::TransformDef,
    frame: u32,
    total_frames: u32,
    params: &std::collections::HashMap<String, f64>,
) -> Result<Vec<Transform>, TransformError> {
    // Handle simple ops-only transform
    if transform_def.is_simple() {
        if let Some(ops) = &transform_def.ops {
            return ops.iter().map(parse_transform_spec_internal).collect();
        }
        return Ok(vec![]);
    }

    // Handle cycling transforms
    if transform_def.is_cycling() {
        if let Some(cycle) = &transform_def.cycle {
            let cycle_len = cycle.len();
            if cycle_len > 0 {
                let cycle_index = (frame as usize) % cycle_len;
                return cycle[cycle_index].iter().map(parse_transform_spec_internal).collect();
            }
        }
        return Ok(vec![]);
    }

    // Handle keyframe animation
    if transform_def.generates_animation() {
        let keyframes = transform_def.keyframes.as_ref().unwrap();
        let default_easing = transform_def.easing.clone().unwrap_or_default();

        let mut transforms = Vec::new();

        // Create evaluator with standard variables and user params
        let eval = ExpressionEvaluator::for_keyframe(frame, total_frames).with_vars(params);

        match keyframes {
            crate::models::KeyframeSpec::Array(kfs) => {
                // Collect all unique property names
                let mut property_values: std::collections::HashMap<String, f64> =
                    std::collections::HashMap::new();

                for kf in kfs {
                    for prop in kf.values.keys() {
                        // Build keyframes for this property
                        let kf_pairs: Vec<[f64; 2]> = kfs
                            .iter()
                            .filter_map(|k| k.values.get(prop).map(|v| [k.frame as f64, *v]))
                            .collect();

                        let interpolated =
                            interpolate_keyframes(&kf_pairs, frame as f64, &default_easing);
                        property_values.insert(prop.clone(), interpolated);
                    }
                }

                // Convert property values to transforms
                transforms.extend(property_values_to_transforms(&property_values)?);
            }
            crate::models::KeyframeSpec::Properties(props) => {
                let mut property_values: std::collections::HashMap<String, f64> =
                    std::collections::HashMap::new();

                for (prop, prop_kf) in props {
                    let easing = prop_kf.easing.as_ref().unwrap_or(&default_easing);

                    let value = if let Some(expr) = &prop_kf.expr {
                        // Evaluate expression
                        eval.evaluate(expr).map_err(|e| TransformError::InvalidParameter {
                            op: "keyframe".to_string(),
                            message: e.to_string(),
                        })?
                    } else if let Some(kfs) = &prop_kf.keyframes {
                        // Interpolate keyframes
                        interpolate_keyframes(kfs, frame as f64, easing)
                    } else {
                        0.0
                    };

                    property_values.insert(prop.clone(), value);
                }

                transforms.extend(property_values_to_transforms(&property_values)?);
            }
        }

        return Ok(transforms);
    }

    // Handle compose (parallel composition)
    if let Some(compose) = &transform_def.compose {
        return compose.iter().map(parse_transform_spec_internal).collect();
    }

    Ok(vec![])
}

/// Convert property name/value pairs to Transform enum variants.
fn property_values_to_transforms(
    properties: &std::collections::HashMap<String, f64>,
) -> Result<Vec<Transform>, TransformError> {
    let mut transforms = Vec::new();

    // Handle shift properties
    let shift_x = properties.get("shift-x").or_else(|| properties.get("shift_x"));
    let shift_y = properties.get("shift-y").or_else(|| properties.get("shift_y"));

    if shift_x.is_some() || shift_y.is_some() {
        transforms.push(Transform::Shift {
            x: shift_x.map(|v| v.round() as i32).unwrap_or(0),
            y: shift_y.map(|v| v.round() as i32).unwrap_or(0),
        });
    }

    // Handle scale properties
    let scale_x = properties.get("scale-x").or_else(|| properties.get("scale_x"));
    let scale_y = properties.get("scale-y").or_else(|| properties.get("scale_y"));
    let scale = properties.get("scale");

    if scale_x.is_some() || scale_y.is_some() || scale.is_some() {
        let x = scale_x.or(scale).copied().unwrap_or(1.0) as f32;
        let y = scale_y.or(scale).copied().unwrap_or(1.0) as f32;
        transforms.push(Transform::Scale { x, y });
    }

    // Handle rotation
    if let Some(degrees) = properties.get("rotate").or_else(|| properties.get("rotation")) {
        let deg = degrees.round() as i32;
        // Normalize to 0, 90, 180, 270
        let normalized = ((deg % 360) + 360) % 360;
        if normalized == 90 || normalized == 180 || normalized == 270 {
            transforms.push(Transform::Rotate { degrees: normalized as u16 });
        }
    }

    // Handle pad
    if let Some(pad) = properties.get("pad").or_else(|| properties.get("padding")) {
        transforms.push(Transform::Pad { size: pad.max(0.0).round() as u32 });
    }

    // Handle subpixel
    let subpixel_x = properties.get("subpixel-x").or_else(|| properties.get("subpixel_x"));
    let subpixel_y = properties.get("subpixel-y").or_else(|| properties.get("subpixel_y"));

    if subpixel_x.is_some() || subpixel_y.is_some() {
        transforms.push(Transform::Subpixel {
            x: subpixel_x.copied().unwrap_or(0.0),
            y: subpixel_y.copied().unwrap_or(0.0),
        });
    }

    Ok(transforms)
}

/// Parse a TransformSpec into a Transform (internal version for this module).
fn parse_transform_spec_internal(
    spec: &crate::models::TransformSpec,
) -> Result<Transform, TransformError> {
    match spec {
        crate::models::TransformSpec::String(s) => parse_transform_str(s),
        crate::models::TransformSpec::Object { op, params } => {
            // Convert params to serde_json::Value object for parsing
            let mut obj = serde_json::Map::new();
            obj.insert("op".to_string(), serde_json::Value::String(op.clone()));
            for (k, v) in params {
                obj.insert(k.clone(), v.clone());
            }
            parse_transform_value(&serde_json::Value::Object(obj))
        }
    }
}

// ============================================================================
// Anchor-Preserving Scaling (TTP-ca8cj)
// ============================================================================

/// Bounding box for an anchor region.
///
/// Used to track regions that should be preserved during downscaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchorBounds {
    /// Left edge (inclusive)
    pub x: u32,
    /// Top edge (inclusive)
    pub y: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl AnchorBounds {
    /// Create a new anchor bounds.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Create anchor bounds from a set of pixel coordinates.
    ///
    /// Returns `None` if the points set is empty.
    pub fn from_points(points: &[(i32, i32)]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let min_x = points.iter().map(|(x, _)| *x).min().unwrap();
        let max_x = points.iter().map(|(x, _)| *x).max().unwrap();
        let min_y = points.iter().map(|(_, y)| *y).min().unwrap();
        let max_y = points.iter().map(|(_, y)| *y).max().unwrap();

        // Handle negative coordinates by clamping to 0
        let x = min_x.max(0) as u32;
        let y = min_y.max(0) as u32;
        let width = (max_x - min_x + 1).max(1) as u32;
        let height = (max_y - min_y + 1).max(1) as u32;

        Some(Self { x, y, width, height })
    }

    /// Get the center point of the bounding box.
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Scale the bounds by the given factors.
    ///
    /// For downscaling, this may result in very small or zero dimensions.
    pub fn scaled(&self, scale_x: f32, scale_y: f32) -> Self {
        let new_x = (self.x as f32 * scale_x).round() as u32;
        let new_y = (self.y as f32 * scale_y).round() as u32;
        let new_width = (self.width as f32 * scale_x).round() as u32;
        let new_height = (self.height as f32 * scale_y).round() as u32;

        Self { x: new_x, y: new_y, width: new_width, height: new_height }
    }
}

/// Scale an image with preservation of anchor regions.
///
/// When downscaling (scale factors < 1.0), ensures that anchor regions
/// maintain at least 1x1 pixel bounds. This is important for pixel art
/// where small details like eyes should not disappear during scaling.
///
/// # Arguments
///
/// * `image` - The source image to scale
/// * `scale_x` - Horizontal scale factor
/// * `scale_y` - Vertical scale factor
/// * `anchors` - List of anchor region bounds to preserve
///
/// # Returns
///
/// The scaled image with anchor regions preserved.
///
/// # Example
///
/// ```ignore
/// use pixelsrc::transforms::{scale_image_with_anchor_preservation, AnchorBounds};
///
/// let anchors = vec![
///     AnchorBounds::new(10, 5, 2, 2),  // Eye region
/// ];
///
/// let scaled = scale_image_with_anchor_preservation(&image, 0.5, 0.5, &anchors);
/// // The eye region will be preserved at minimum 1x1 pixel
/// ```
pub fn scale_image_with_anchor_preservation(
    image: &RgbaImage,
    scale_x: f32,
    scale_y: f32,
    anchors: &[AnchorBounds],
) -> RgbaImage {
    // For upscaling or no anchors, use standard nearest-neighbor scaling
    if (scale_x >= 1.0 && scale_y >= 1.0) || anchors.is_empty() {
        return scale_image(image, scale_x, scale_y);
    }

    let (src_width, src_height) = image.dimensions();
    let dst_width = ((src_width as f32 * scale_x).round() as u32).max(1);
    let dst_height = ((src_height as f32 * scale_y).round() as u32).max(1);

    // First, do standard nearest-neighbor scaling
    let mut result = scale_image(image, scale_x, scale_y);

    // For downscaling, ensure each anchor region has at least 1x1 representation
    // by explicitly writing the anchor's center pixel to the scaled image
    for anchor in anchors {
        // Find the center of the original anchor region
        let (center_x, center_y) = anchor.center();

        // Map the center to destination coordinates
        let dst_x = ((center_x as f32 * scale_x).round() as u32).min(dst_width.saturating_sub(1));
        let dst_y = ((center_y as f32 * scale_y).round() as u32).min(dst_height.saturating_sub(1));

        // Get the color from the center of the original anchor region
        if center_x < src_width && center_y < src_height {
            let pixel = *image.get_pixel(center_x, center_y);

            // Write the anchor pixel - this ensures the anchor is always visible
            // even if the standard scaling algorithm would have skipped it
            if dst_x < dst_width && dst_y < dst_height {
                result.put_pixel(dst_x, dst_y, pixel);
            }
        }
    }

    result
}

/// Scale an image by fractional factors using nearest-neighbor interpolation.
///
/// This preserves crisp pixel edges for pixel art. Unlike `output::scale_image`
/// which only handles integer upscaling, this function supports any scale factor.
///
/// # Arguments
///
/// * `image` - The image to scale
/// * `scale_x` - Horizontal scale factor (e.g., 0.5 for half width)
/// * `scale_y` - Vertical scale factor (e.g., 2.0 for double height)
///
/// # Returns
///
/// The scaled image.
pub fn scale_image(image: &RgbaImage, scale_x: f32, scale_y: f32) -> RgbaImage {
    // Handle no-op case
    if (scale_x - 1.0).abs() < 0.001 && (scale_y - 1.0).abs() < 0.001 {
        return image.clone();
    }

    let (w, h) = image.dimensions();
    let new_w = ((w as f32 * scale_x).round() as u32).max(1);
    let new_h = ((h as f32 * scale_y).round() as u32).max(1);

    image::imageops::resize(image, new_w, new_h, FilterType::Nearest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mirror_h() {
        assert_eq!(parse_transform_str("mirror-h").unwrap(), Transform::MirrorH);
        assert_eq!(parse_transform_str("symmetry-h").unwrap(), Transform::MirrorH);
        assert_eq!(parse_transform_str("flip-h").unwrap(), Transform::MirrorH);
        assert_eq!(parse_transform_str("MIRROR-H").unwrap(), Transform::MirrorH);
    }

    #[test]
    fn test_parse_mirror_v() {
        assert_eq!(parse_transform_str("mirror-v").unwrap(), Transform::MirrorV);
        assert_eq!(parse_transform_str("symmetry-v").unwrap(), Transform::MirrorV);
        assert_eq!(parse_transform_str("flip-v").unwrap(), Transform::MirrorV);
    }

    #[test]
    fn test_parse_rotate() {
        assert_eq!(parse_transform_str("rotate:90").unwrap(), Transform::Rotate { degrees: 90 });
        assert_eq!(parse_transform_str("rotate:180").unwrap(), Transform::Rotate { degrees: 180 });
        assert_eq!(parse_transform_str("rotate:270").unwrap(), Transform::Rotate { degrees: 270 });
        assert_eq!(parse_transform_str("rot:90").unwrap(), Transform::Rotate { degrees: 90 });
    }

    #[test]
    fn test_parse_rotate_invalid() {
        assert!(parse_transform_str("rotate:45").is_err());
        assert!(parse_transform_str("rotate:360").is_err());
        assert!(parse_transform_str("rotate").is_err());
    }

    #[test]
    fn test_parse_tile() {
        assert_eq!(parse_transform_str("tile:3x2").unwrap(), Transform::Tile { w: 3, h: 2 });
        assert_eq!(parse_transform_str("tile:1x1").unwrap(), Transform::Tile { w: 1, h: 1 });
    }

    #[test]
    fn test_parse_pad() {
        assert_eq!(parse_transform_str("pad:4").unwrap(), Transform::Pad { size: 4 });
        assert_eq!(parse_transform_str("pad:0").unwrap(), Transform::Pad { size: 0 });
    }

    #[test]
    fn test_parse_crop() {
        assert_eq!(
            parse_transform_str("crop:0,0,8,8").unwrap(),
            Transform::Crop { x: 0, y: 0, w: 8, h: 8 }
        );
        assert_eq!(
            parse_transform_str("crop:4, 4, 16, 16").unwrap(),
            Transform::Crop { x: 4, y: 4, w: 16, h: 16 }
        );
    }

    #[test]
    fn test_parse_outline() {
        assert_eq!(
            parse_transform_str("outline").unwrap(),
            Transform::Outline { token: None, width: 1 }
        );
        assert_eq!(
            parse_transform_str("outline:{border}").unwrap(),
            Transform::Outline { token: Some("{border}".to_string()), width: 1 }
        );
        assert_eq!(
            parse_transform_str("outline:{border},2").unwrap(),
            Transform::Outline { token: Some("{border}".to_string()), width: 2 }
        );
    }

    #[test]
    fn test_parse_shift() {
        assert_eq!(parse_transform_str("shift:4,0").unwrap(), Transform::Shift { x: 4, y: 0 });
        assert_eq!(parse_transform_str("shift:-2,3").unwrap(), Transform::Shift { x: -2, y: 3 });
    }

    #[test]
    fn test_parse_shadow() {
        assert_eq!(
            parse_transform_str("shadow:1,1").unwrap(),
            Transform::Shadow { x: 1, y: 1, token: None }
        );
        assert_eq!(
            parse_transform_str("shadow:2,2,{shadow}").unwrap(),
            Transform::Shadow { x: 2, y: 2, token: Some("{shadow}".to_string()) }
        );
    }

    #[test]
    fn test_parse_skew_x() {
        assert_eq!(
            parse_transform_str("skew-x:20").unwrap(),
            Transform::SkewX { degrees: 20.0 }
        );
        assert_eq!(
            parse_transform_str("skewx:45deg").unwrap(),
            Transform::SkewX { degrees: 45.0 }
        );
        assert_eq!(
            parse_transform_str("skew-x:-30").unwrap(),
            Transform::SkewX { degrees: -30.0 }
        );
    }

    #[test]
    fn test_parse_skew_y() {
        assert_eq!(
            parse_transform_str("skew-y:15").unwrap(),
            Transform::SkewY { degrees: 15.0 }
        );
        assert_eq!(
            parse_transform_str("skewy:26.57°").unwrap(),
            Transform::SkewY { degrees: 26.57 }
        );
    }

    #[test]
    fn test_parse_skew_invalid() {
        // 89+ degrees should fail
        assert!(parse_transform_str("skew-x:89").is_err());
        assert!(parse_transform_str("skew-y:-90").is_err());
        // Missing angle
        assert!(parse_transform_str("skew-x").is_err());
    }

    #[test]
    fn test_parse_pingpong() {
        assert_eq!(
            parse_transform_str("pingpong").unwrap(),
            Transform::Pingpong { exclude_ends: false }
        );
        assert_eq!(
            parse_transform_str("pingpong:true").unwrap(),
            Transform::Pingpong { exclude_ends: true }
        );
        assert_eq!(
            parse_transform_str("pingpong:exclude_ends").unwrap(),
            Transform::Pingpong { exclude_ends: true }
        );
    }

    #[test]
    fn test_parse_reverse() {
        assert_eq!(parse_transform_str("reverse").unwrap(), Transform::Reverse);
    }

    #[test]
    fn test_parse_frame_offset() {
        assert_eq!(
            parse_transform_str("frame-offset:2").unwrap(),
            Transform::FrameOffset { offset: 2 }
        );
        assert_eq!(
            parse_transform_str("frame-offset:-1").unwrap(),
            Transform::FrameOffset { offset: -1 }
        );
    }

    #[test]
    fn test_parse_hold() {
        assert_eq!(
            parse_transform_str("hold:0,3").unwrap(),
            Transform::Hold { frame: 0, count: 3 }
        );
    }

    #[test]
    fn test_parse_unknown_operation() {
        assert!(parse_transform_str("unknown").is_err());
        assert!(parse_transform_str("invalid-op:123").is_err());
    }

    #[test]
    fn test_parse_transform_value_string() {
        let value = serde_json::json!("mirror-h");
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::MirrorH);

        let value = serde_json::json!("rotate:90");
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::Rotate { degrees: 90 });
    }

    #[test]
    fn test_parse_transform_value_object() {
        let value = serde_json::json!({"op": "tile", "w": 3, "h": 2});
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::Tile { w: 3, h: 2 });

        let value = serde_json::json!({"op": "outline", "token": "{border}", "width": 2});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Outline { token: Some("{border}".to_string()), width: 2 }
        );

        let value = serde_json::json!({"op": "rotate", "degrees": 180});
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::Rotate { degrees: 180 });
    }

    #[test]
    fn test_parse_transform_value_object_pingpong() {
        let value = serde_json::json!({"op": "pingpong", "exclude_ends": true});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Pingpong { exclude_ends: true }
        );
    }

    #[test]
    fn test_parse_transform_value_invalid() {
        let value = serde_json::json!(123);
        assert!(parse_transform_value(&value).is_err());

        let value = serde_json::json!({"no_op": "tile"});
        assert!(parse_transform_value(&value).is_err());
    }

    // ========================================================================
    // Animation Transform Application Tests
    // ========================================================================

    #[test]
    fn test_apply_pingpong_basic() {
        let frames = vec!["A", "B", "C"];
        let result = apply_pingpong(&frames, false);
        assert_eq!(result, vec!["A", "B", "C", "C", "B", "A"]);
    }

    #[test]
    fn test_apply_pingpong_exclude_ends() {
        let frames = vec!["A", "B", "C"];
        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["A", "B", "C", "B"]);
    }

    #[test]
    fn test_apply_pingpong_exclude_ends_longer() {
        let frames = vec!["A", "B", "C", "D", "E"];
        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["A", "B", "C", "D", "E", "D", "C", "B"]);
    }

    #[test]
    fn test_apply_pingpong_two_frames() {
        let frames = vec!["A", "B"];
        // Without exclude_ends: [A, B, B, A]
        let result = apply_pingpong(&frames, false);
        assert_eq!(result, vec!["A", "B", "B", "A"]);

        // With exclude_ends: [A, B] (nothing to add - first and last are excluded)
        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_apply_pingpong_single_frame() {
        let frames = vec!["A"];
        let result = apply_pingpong(&frames, false);
        assert_eq!(result, vec!["A"]);

        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_apply_pingpong_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_pingpong(&frames, false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_reverse_basic() {
        let frames = vec!["A", "B", "C"];
        let result = apply_reverse(&frames);
        assert_eq!(result, vec!["C", "B", "A"]);
    }

    #[test]
    fn test_apply_reverse_single() {
        let frames = vec!["A"];
        let result = apply_reverse(&frames);
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_apply_reverse_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_reverse(&frames);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_frame_offset_positive() {
        let frames = vec!["A", "B", "C", "D"];
        let result = apply_frame_offset(&frames, 1);
        assert_eq!(result, vec!["B", "C", "D", "A"]);
    }

    #[test]
    fn test_apply_frame_offset_positive_multiple() {
        let frames = vec!["A", "B", "C", "D"];
        let result = apply_frame_offset(&frames, 2);
        assert_eq!(result, vec!["C", "D", "A", "B"]);
    }

    #[test]
    fn test_apply_frame_offset_negative() {
        let frames = vec!["A", "B", "C", "D"];
        let result = apply_frame_offset(&frames, -1);
        assert_eq!(result, vec!["D", "A", "B", "C"]);
    }

    #[test]
    fn test_apply_frame_offset_wrap_around() {
        let frames = vec!["A", "B", "C"];
        // Offset 4 on 3 frames = offset 1
        let result = apply_frame_offset(&frames, 4);
        assert_eq!(result, vec!["B", "C", "A"]);
    }

    #[test]
    fn test_apply_frame_offset_zero() {
        let frames = vec!["A", "B", "C"];
        let result = apply_frame_offset(&frames, 0);
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_apply_frame_offset_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_frame_offset(&frames, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_hold_basic() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 1, 3);
        assert_eq!(result, vec!["A", "B", "B", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_first_frame() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 0, 2);
        assert_eq!(result, vec!["A", "A", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_last_frame() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 2, 3);
        assert_eq!(result, vec!["A", "B", "C", "C", "C"]);
    }

    #[test]
    fn test_apply_hold_count_one() {
        // count=1 means keep one copy (no change)
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 1, 1);
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_count_zero() {
        // count=0 removes the frame
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 1, 0);
        assert_eq!(result, vec!["A", "C"]);
    }

    #[test]
    fn test_apply_hold_invalid_frame_index() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 10, 5);
        // Should return unchanged
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_hold(&frames, 0, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_animation_transform_pingpong() {
        let frames = vec!["A", "B", "C"];
        let transform = Transform::Pingpong { exclude_ends: false };
        let result = apply_animation_transform(&transform, &frames).unwrap();
        assert_eq!(result, vec!["A", "B", "C", "C", "B", "A"]);
    }

    #[test]
    fn test_apply_animation_transform_reverse() {
        let frames = vec!["A", "B", "C"];
        let transform = Transform::Reverse;
        let result = apply_animation_transform(&transform, &frames).unwrap();
        assert_eq!(result, vec!["C", "B", "A"]);
    }

    #[test]
    fn test_apply_animation_transform_frame_offset() {
        let frames = vec!["A", "B", "C", "D"];
        let transform = Transform::FrameOffset { offset: 2 };
        let result = apply_animation_transform(&transform, &frames).unwrap();
        assert_eq!(result, vec!["C", "D", "A", "B"]);
    }

    #[test]
    fn test_apply_animation_transform_hold() {
        let frames = vec!["A", "B", "C"];
        let transform = Transform::Hold { frame: 0, count: 3 };
        let result = apply_animation_transform(&transform, &frames).unwrap();
        assert_eq!(result, vec!["A", "A", "A", "B", "C"]);
    }

    #[test]
    fn test_apply_animation_transform_non_animation() {
        let frames = vec!["A", "B", "C"];
        let transform = Transform::MirrorH;
        let result = apply_animation_transform(&transform, &frames);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_animation_transform() {
        assert!(is_animation_transform(&Transform::Pingpong { exclude_ends: false }));
        assert!(is_animation_transform(&Transform::Reverse));
        assert!(is_animation_transform(&Transform::FrameOffset { offset: 1 }));
        assert!(is_animation_transform(&Transform::Hold { frame: 0, count: 2 }));

        assert!(!is_animation_transform(&Transform::MirrorH));
        assert!(!is_animation_transform(&Transform::MirrorV));
        assert!(!is_animation_transform(&Transform::Rotate { degrees: 90 }));
        assert!(!is_animation_transform(&Transform::Tile { w: 2, h: 2 }));
        assert!(!is_animation_transform(&Transform::Pad { size: 4 }));
        assert!(!is_animation_transform(&Transform::Crop { x: 0, y: 0, w: 8, h: 8 }));
        assert!(!is_animation_transform(&Transform::Outline { token: None, width: 1 }));
        assert!(!is_animation_transform(&Transform::Shift { x: 1, y: 1 }));
        assert!(!is_animation_transform(&Transform::Shadow { x: 1, y: 1, token: None }));
    }

    #[test]
    fn test_chained_animation_transforms() {
        // Test applying multiple transforms in sequence
        let frames = vec!["A", "B", "C"];

        // First reverse: [C, B, A]
        let frames = apply_reverse(&frames);
        assert_eq!(frames, vec!["C", "B", "A"]);

        // Then pingpong: [C, B, A, A, B, C]
        let frames = apply_pingpong(&frames, false);
        assert_eq!(frames, vec!["C", "B", "A", "A", "B", "C"]);

        // Then hold frame 2 for 3: [C, B, A, A, A, A, B, C]
        let frames = apply_hold(&frames, 2, 3);
        assert_eq!(frames, vec!["C", "B", "A", "A", "A", "A", "B", "C"]);
    }

    #[test]
    fn test_animation_transforms_with_strings() {
        // Test with actual String type (not &str)
        let frames: Vec<String> =
            vec!["frame1".to_string(), "frame2".to_string(), "frame3".to_string()];

        let result = apply_reverse(&frames);
        assert_eq!(result, vec!["frame3", "frame2", "frame1"]);

        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["frame1", "frame2", "frame3", "frame2"]);
    }

    // ========================================================================
    // Dithering Tests (ATF-8)
    // ========================================================================

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

    #[test]
    fn test_parse_dither_str_basic() {
        let result = parse_transform_str("dither:checker:{dark},{light}").unwrap();
        assert_eq!(
            result,
            Transform::Dither {
                pattern: DitherPattern::Checker,
                tokens: ("{dark}".to_string(), "{light}".to_string()),
                threshold: 0.5,
                seed: 0,
            }
        );
    }

    // ========================================================================
    // Selective Outline (sel-out) Tests (ATF-9)
    // ========================================================================

    #[test]
    fn test_parse_selout_string() {
        assert_eq!(
            parse_transform_str("sel-out").unwrap(),
            Transform::SelOut { fallback: None, mapping: None }
        );
        assert_eq!(
            parse_transform_str("selout").unwrap(),
            Transform::SelOut { fallback: None, mapping: None }
        );
    }

    #[test]
    fn test_parse_selout_with_fallback() {
        assert_eq!(
            parse_transform_str("sel-out:{outline}").unwrap(),
            Transform::SelOut { fallback: Some("{outline}".to_string()), mapping: None }
        );
    }

    // ========================================================================
    // More Dithering Tests (ATF-8 continued)
    // ========================================================================

    #[test]
    fn test_parse_dither_str_with_threshold() {
        let result = parse_transform_str("dither:ordered-4x4:{a},{b}:0.3").unwrap();
        assert_eq!(
            result,
            Transform::Dither {
                pattern: DitherPattern::Ordered4x4,
                tokens: ("{a}".to_string(), "{b}".to_string()),
                threshold: 0.3,
                seed: 0,
            }
        );
    }

    #[test]
    fn test_parse_dither_str_with_seed() {
        let result = parse_transform_str("dither:noise:{a},{b}:0.5:42").unwrap();
        assert_eq!(
            result,
            Transform::Dither {
                pattern: DitherPattern::Noise,
                tokens: ("{a}".to_string(), "{b}".to_string()),
                threshold: 0.5,
                seed: 42,
            }
        );
    }

    #[test]
    fn test_parse_dither_str_missing_tokens() {
        assert!(parse_transform_str("dither:checker").is_err());
    }

    #[test]
    fn test_parse_dither_str_invalid_pattern() {
        assert!(parse_transform_str("dither:invalid:{a},{b}").is_err());
    }

    #[test]
    fn test_parse_dither_gradient_str_basic() {
        let result =
            parse_transform_str("dither-gradient:vertical:{sky_light},{sky_dark}").unwrap();
        assert_eq!(
            result,
            Transform::DitherGradient {
                direction: GradientDirection::Vertical,
                from: "{sky_light}".to_string(),
                to: "{sky_dark}".to_string(),
                pattern: DitherPattern::Ordered4x4, // default
            }
        );
    }

    #[test]
    fn test_parse_dither_gradient_str_with_pattern() {
        let result = parse_transform_str("dither-gradient:horizontal:{a},{b}:checker").unwrap();
        assert_eq!(
            result,
            Transform::DitherGradient {
                direction: GradientDirection::Horizontal,
                from: "{a}".to_string(),
                to: "{b}".to_string(),
                pattern: DitherPattern::Checker,
            }
        );
    }

    #[test]
    fn test_parse_dither_value_object() {
        let value = serde_json::json!({
            "op": "dither",
            "pattern": "checker",
            "tokens": ["{dark}", "{light}"],
            "threshold": 0.5
        });
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Dither {
                pattern: DitherPattern::Checker,
                tokens: ("{dark}".to_string(), "{light}".to_string()),
                threshold: 0.5,
                seed: 0,
            }
        );
    }

    // More sel-out tests (ATF-9 continued)

    #[test]
    fn test_parse_selout_object() {
        let value = serde_json::json!({"op": "sel-out", "fallback": "{border}"});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::SelOut { fallback: Some("{border}".to_string()), mapping: None }
        );
    }

    #[test]
    fn test_parse_selout_object_with_mapping() {
        let value = serde_json::json!({
            "op": "sel-out",
            "mapping": {
                "{skin}": "{skin_dark}",
                "{hair}": "{hair_dark}",
                "*": "{outline}"
            }
        });
        let result = parse_transform_value(&value).unwrap();
        match result {
            Transform::SelOut { fallback, mapping } => {
                assert!(fallback.is_none());
                let map = mapping.unwrap();
                assert_eq!(map.get("{skin}"), Some(&"{skin_dark}".to_string()));
                assert_eq!(map.get("{hair}"), Some(&"{hair_dark}".to_string()));
                assert_eq!(map.get("*"), Some(&"{outline}".to_string()));
            }
            _ => panic!("Expected SelOut transform"),
        }
    }

    // More dither value tests (ATF-8 continued)

    #[test]
    fn test_parse_dither_value_object_with_seed() {
        let value = serde_json::json!({
            "op": "dither",
            "pattern": "noise",
            "tokens": ["{a}", "{b}"],
            "seed": 12345
        });
        let result = parse_transform_value(&value).unwrap();
        match result {
            Transform::Dither { seed, .. } => assert_eq!(seed, 12345),
            _ => panic!("expected Dither transform"),
        }
    }

    #[test]
    fn test_parse_dither_value_object_missing_tokens() {
        let value = serde_json::json!({
            "op": "dither",
            "pattern": "checker"
        });
        assert!(parse_transform_value(&value).is_err());
    }

    #[test]
    fn test_parse_dither_value_object_wrong_tokens_count() {
        let value = serde_json::json!({
            "op": "dither",
            "pattern": "checker",
            "tokens": ["{only_one}"]
        });
        assert!(parse_transform_value(&value).is_err());
    }

    #[test]
    fn test_parse_dither_gradient_value_object() {
        let value = serde_json::json!({
            "op": "dither-gradient",
            "direction": "vertical",
            "from": "{sky_light}",
            "to": "{sky_dark}",
            "pattern": "ordered-4x4"
        });
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::DitherGradient {
                direction: GradientDirection::Vertical,
                from: "{sky_light}".to_string(),
                to: "{sky_dark}".to_string(),
                pattern: DitherPattern::Ordered4x4,
            }
        );
    }

    #[test]
    fn test_parse_dither_gradient_value_object_defaults() {
        // Test default direction and pattern
        let value = serde_json::json!({
            "op": "dither-gradient",
            "from": "{a}",
            "to": "{b}"
        });
        let result = parse_transform_value(&value).unwrap();
        match result {
            Transform::DitherGradient { direction, pattern, .. } => {
                assert_eq!(direction, GradientDirection::Vertical);
                assert_eq!(pattern, DitherPattern::Ordered4x4);
            }
            _ => panic!("expected DitherGradient transform"),
        }
    }

    #[test]
    fn test_parse_token_pair() {
        // Basic tokens
        assert_eq!(
            parse_token_pair("{dark},{light}"),
            Some(("{dark}".to_string(), "{light}".to_string()))
        );

        // Tokens without braces
        assert_eq!(parse_token_pair("dark,light"), Some(("dark".to_string(), "light".to_string())));

        // With spaces
        assert_eq!(parse_token_pair("  {a} , {b}  "), Some(("{a}".to_string(), "{b}".to_string())));

        // Empty input
        assert_eq!(parse_token_pair(""), None);

        // Missing second token
        assert_eq!(parse_token_pair("{a},"), None);

        // Missing first token
        assert_eq!(parse_token_pair(",{b}"), None);
    }

    // ========================================================================
    // Sub-pixel Animation Tests (ATF-13)
    // ========================================================================

    #[test]
    fn test_parse_subpixel_str_basic() {
        let result = parse_transform_str("subpixel:0.5,0.25").unwrap();
        assert_eq!(result, Transform::Subpixel { x: 0.5, y: 0.25 });
    }

    #[test]
    fn test_parse_subpixel_str_x_only() {
        let result = parse_transform_str("subpixel:0.5,").unwrap();
        assert_eq!(result, Transform::Subpixel { x: 0.5, y: 0.0 });
    }

    #[test]
    fn test_parse_subpixel_str_aliases() {
        // Test alternative names
        let result1 = parse_transform_str("sub-pixel:0.3,0.7").unwrap();
        let result2 = parse_transform_str("subpixel-shift:0.3,0.7").unwrap();
        assert_eq!(result1, Transform::Subpixel { x: 0.3, y: 0.7 });
        assert_eq!(result2, Transform::Subpixel { x: 0.3, y: 0.7 });
    }

    #[test]
    fn test_parse_subpixel_str_invalid() {
        assert!(parse_transform_str("subpixel:invalid,0.5").is_err());
        assert!(parse_transform_str("subpixel:0.5,invalid").is_err());
    }

    #[test]
    fn test_parse_subpixel_value_object() {
        let value = serde_json::json!({
            "op": "subpixel",
            "x": 0.5,
            "y": 0.25
        });
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::Subpixel { x: 0.5, y: 0.25 });
    }

    #[test]
    fn test_parse_subpixel_value_object_alt_keys() {
        // Test alternative key names (subpixel-x, subpixel_x)
        let value1 = serde_json::json!({
            "op": "subpixel",
            "subpixel-x": 0.3,
            "subpixel-y": 0.7
        });
        assert_eq!(parse_transform_value(&value1).unwrap(), Transform::Subpixel { x: 0.3, y: 0.7 });

        let value2 = serde_json::json!({
            "op": "subpixel",
            "subpixel_x": 0.2,
            "subpixel_y": 0.8
        });
        assert_eq!(parse_transform_value(&value2).unwrap(), Transform::Subpixel { x: 0.2, y: 0.8 });
    }

    #[test]
    fn test_parse_subpixel_value_object_defaults() {
        // Missing x and y should default to 0.0
        let value = serde_json::json!({
            "op": "subpixel"
        });
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::Subpixel { x: 0.0, y: 0.0 });
    }

    #[test]
    fn test_parse_subpixel_value_object_partial() {
        // Only x specified
        let value = serde_json::json!({
            "op": "subpixel",
            "x": 0.5
        });
        assert_eq!(parse_transform_value(&value).unwrap(), Transform::Subpixel { x: 0.5, y: 0.0 });
    }

    // =========================================================================
    // CSS Transform Parsing Tests (CSS-14)
    // =========================================================================

    #[test]
    fn test_parse_css_transform_empty() {
        let result = parse_css_transform("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_css_transform_whitespace() {
        let result = parse_css_transform("   ").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_css_translate_basic() {
        let result = parse_css_transform("translate(10, 5)").unwrap();
        assert_eq!(result.translate, Some((10, 5)));
        assert_eq!(result.rotate, None);
        assert_eq!(result.scale, None);
        assert!(!result.flip_x);
        assert!(!result.flip_y);
    }

    #[test]
    fn test_parse_css_translate_with_px() {
        let result = parse_css_transform("translate(10px, 5px)").unwrap();
        assert_eq!(result.translate, Some((10, 5)));
    }

    #[test]
    fn test_parse_css_translate_negative() {
        let result = parse_css_transform("translate(-5, -10)").unwrap();
        assert_eq!(result.translate, Some((-5, -10)));
    }

    #[test]
    fn test_parse_css_translate_single_value() {
        let result = parse_css_transform("translate(10)").unwrap();
        assert_eq!(result.translate, Some((10, 0)));
    }

    #[test]
    fn test_parse_css_rotate_with_deg() {
        let result = parse_css_transform("rotate(90deg)").unwrap();
        assert_eq!(result.rotate, Some(90.0));
    }

    #[test]
    fn test_parse_css_rotate_without_deg() {
        let result = parse_css_transform("rotate(180)").unwrap();
        assert_eq!(result.rotate, Some(180.0));
    }

    #[test]
    fn test_parse_css_rotate_270() {
        let result = parse_css_transform("rotate(270deg)").unwrap();
        assert_eq!(result.rotate, Some(270.0));
    }

    #[test]
    fn test_parse_css_scale_uniform() {
        let result = parse_css_transform("scale(2)").unwrap();
        assert_eq!(result.scale, Some((2.0, 2.0)));
    }

    #[test]
    fn test_parse_css_scale_non_uniform() {
        let result = parse_css_transform("scale(2, 1.5)").unwrap();
        assert_eq!(result.scale, Some((2.0, 1.5)));
    }

    #[test]
    fn test_parse_css_scale_fractional() {
        let result = parse_css_transform("scale(0.5, 0.25)").unwrap();
        assert_eq!(result.scale, Some((0.5, 0.25)));
    }

    #[test]
    fn test_parse_css_scalex() {
        let result = parse_css_transform("scaleX(2)").unwrap();
        assert_eq!(result.scale, Some((2.0, 1.0)));
    }

    #[test]
    fn test_parse_css_scaley() {
        let result = parse_css_transform("scaleY(1.5)").unwrap();
        assert_eq!(result.scale, Some((1.0, 1.5)));
    }

    #[test]
    fn test_parse_css_skewx() {
        let result = parse_css_transform("skewX(20deg)").unwrap();
        assert_eq!(result.skew_x, Some(20.0));
        assert_eq!(result.skew_y, None);
    }

    #[test]
    fn test_parse_css_skewy() {
        let result = parse_css_transform("skewY(15deg)").unwrap();
        assert_eq!(result.skew_x, None);
        assert_eq!(result.skew_y, Some(15.0));
    }

    #[test]
    fn test_parse_css_skew_single() {
        let result = parse_css_transform("skew(30)").unwrap();
        assert_eq!(result.skew_x, Some(30.0));
        assert_eq!(result.skew_y, None);
    }

    #[test]
    fn test_parse_css_skew_both() {
        let result = parse_css_transform("skew(20deg, 10deg)").unwrap();
        assert_eq!(result.skew_x, Some(20.0));
        assert_eq!(result.skew_y, Some(10.0));
    }

    #[test]
    fn test_parse_css_skew_negative() {
        let result = parse_css_transform("skewX(-25deg)").unwrap();
        assert_eq!(result.skew_x, Some(-25.0));
    }

    #[test]
    fn test_parse_css_skew_invalid_angle() {
        // 90 degrees is invalid (tangent approaches infinity)
        assert!(parse_css_transform("skewX(90deg)").is_err());
        assert!(parse_css_transform("skewY(-89deg)").is_err());
    }

    #[test]
    fn test_parse_css_flip_x() {
        let result = parse_css_transform("flip(x)").unwrap();
        assert!(result.flip_x);
        assert!(!result.flip_y);
    }

    #[test]
    fn test_parse_css_flip_y() {
        let result = parse_css_transform("flip(y)").unwrap();
        assert!(!result.flip_x);
        assert!(result.flip_y);
    }

    #[test]
    fn test_parse_css_flip_horizontal() {
        let result = parse_css_transform("flip(horizontal)").unwrap();
        assert!(result.flip_x);
        assert!(!result.flip_y);
    }

    #[test]
    fn test_parse_css_flip_vertical() {
        let result = parse_css_transform("flip(vertical)").unwrap();
        assert!(!result.flip_x);
        assert!(result.flip_y);
    }

    #[test]
    fn test_parse_css_flipx() {
        let result = parse_css_transform("flipX()").unwrap();
        assert!(result.flip_x);
        assert!(!result.flip_y);
    }

    #[test]
    fn test_parse_css_flipy() {
        let result = parse_css_transform("flipY()").unwrap();
        assert!(!result.flip_x);
        assert!(result.flip_y);
    }

    #[test]
    fn test_parse_css_multiple_transforms() {
        let result = parse_css_transform("translate(10, 5) rotate(90deg) scale(2)").unwrap();
        assert_eq!(result.translate, Some((10, 5)));
        assert_eq!(result.rotate, Some(90.0));
        assert_eq!(result.scale, Some((2.0, 2.0)));
    }

    #[test]
    fn test_parse_css_all_transforms() {
        let result =
            parse_css_transform("translate(5, 10) rotate(180deg) scale(1.5) flip(x) flip(y)")
                .unwrap();
        assert_eq!(result.translate, Some((5, 10)));
        assert_eq!(result.rotate, Some(180.0));
        assert_eq!(result.scale, Some((1.5, 1.5)));
        assert!(result.flip_x);
        assert!(result.flip_y);
    }

    #[test]
    fn test_parse_css_case_insensitive() {
        let result = parse_css_transform("TRANSLATE(5, 5) ROTATE(90DEG)").unwrap();
        assert_eq!(result.translate, Some((5, 5)));
        assert_eq!(result.rotate, Some(90.0));
    }

    #[test]
    fn test_parse_css_extra_whitespace() {
        let result = parse_css_transform("  translate( 10 , 5 )   rotate( 90deg )  ").unwrap();
        assert_eq!(result.translate, Some((10, 5)));
        assert_eq!(result.rotate, Some(90.0));
    }

    #[test]
    fn test_parse_css_transform_error_unknown_function() {
        let result = parse_css_transform("unknown(1, 2)");
        assert!(result.is_err());
        match result.unwrap_err() {
            CssTransformError::UnknownFunction(func) => assert_eq!(func, "unknown"),
            _ => panic!("Expected UnknownFunction error"),
        }
    }

    #[test]
    fn test_parse_css_transform_error_missing_paren() {
        let result = parse_css_transform("translate 10, 5");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CssTransformError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_css_transform_error_unmatched_paren() {
        let result = parse_css_transform("translate(10, 5");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CssTransformError::SyntaxError(_)));
    }

    #[test]
    fn test_parse_css_transform_error_scale_zero() {
        let result = parse_css_transform("scale(0)");
        assert!(result.is_err());
        match result.unwrap_err() {
            CssTransformError::InvalidParameter { func, .. } => assert_eq!(func, "scale"),
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_parse_css_transform_error_scale_negative() {
        let result = parse_css_transform("scale(-1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_css_transform_error_flip_invalid_axis() {
        let result = parse_css_transform("flip(z)");
        assert!(result.is_err());
        match result.unwrap_err() {
            CssTransformError::InvalidParameter { func, .. } => assert_eq!(func, "flip"),
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_parse_css_transform_error_flip_empty() {
        let result = parse_css_transform("flip()");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CssTransformError::MissingParameter { .. }));
    }

    #[test]
    fn test_parse_css_transform_error_translate_invalid() {
        let result = parse_css_transform("translate(abc, def)");
        assert!(result.is_err());
        match result.unwrap_err() {
            CssTransformError::InvalidParameter { func, .. } => assert_eq!(func, "translate"),
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_parse_css_transform_error_rotate_invalid() {
        let result = parse_css_transform("rotate(abc)");
        assert!(result.is_err());
        match result.unwrap_err() {
            CssTransformError::InvalidParameter { func, .. } => assert_eq!(func, "rotate"),
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    // =========================================================================
    // CssTransform::to_transforms Tests
    // =========================================================================

    #[test]
    fn test_css_transform_to_transforms_empty() {
        let css = CssTransform::new();
        let transforms = css.to_transforms().unwrap();
        assert!(transforms.is_empty());
    }

    #[test]
    fn test_css_transform_to_transforms_translate() {
        let mut css = CssTransform::new();
        css.translate = Some((10, 5));
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Shift { x: 10, y: 5 }));
    }

    #[test]
    fn test_css_transform_to_transforms_rotate_90() {
        let mut css = CssTransform::new();
        css.rotate = Some(90.0);
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Rotate { degrees: 90 }));
    }

    #[test]
    fn test_css_transform_to_transforms_rotate_180() {
        let mut css = CssTransform::new();
        css.rotate = Some(180.0);
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Rotate { degrees: 180 }));
    }

    #[test]
    fn test_css_transform_to_transforms_rotate_270() {
        let mut css = CssTransform::new();
        css.rotate = Some(270.0);
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Rotate { degrees: 270 }));
    }

    #[test]
    fn test_css_transform_to_transforms_rotate_0() {
        let mut css = CssTransform::new();
        css.rotate = Some(0.0);
        let transforms = css.to_transforms().unwrap();
        assert!(transforms.is_empty()); // 0 degrees = no rotation
    }

    #[test]
    fn test_css_transform_to_transforms_rotate_360() {
        let mut css = CssTransform::new();
        css.rotate = Some(360.0);
        let transforms = css.to_transforms().unwrap();
        assert!(transforms.is_empty()); // 360 degrees = 0 degrees = no rotation
    }

    #[test]
    fn test_css_transform_to_transforms_rotate_invalid() {
        let mut css = CssTransform::new();
        css.rotate = Some(45.0);
        let result = css.to_transforms();
        assert!(result.is_err());
        match result.unwrap_err() {
            CssTransformError::InvalidRotation(deg) => assert!((deg - 45.0).abs() < 0.001),
            _ => panic!("Expected InvalidRotation error"),
        }
    }

    #[test]
    fn test_css_transform_to_transforms_scale() {
        let mut css = CssTransform::new();
        css.scale = Some((2.0, 1.5));
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        match &transforms[0] {
            Transform::Scale { x, y } => {
                assert!((x - 2.0).abs() < 0.001);
                assert!((y - 1.5).abs() < 0.001);
            }
            _ => panic!("Expected Scale transform"),
        }
    }

    #[test]
    fn test_css_transform_to_transforms_flip_x() {
        let mut css = CssTransform::new();
        css.flip_x = true;
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::MirrorH));
    }

    #[test]
    fn test_css_transform_to_transforms_flip_y() {
        let mut css = CssTransform::new();
        css.flip_y = true;
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::MirrorV));
    }

    #[test]
    fn test_css_transform_to_transforms_flip_both() {
        let mut css = CssTransform::new();
        css.flip_x = true;
        css.flip_y = true;
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 2);
        assert!(matches!(transforms[0], Transform::MirrorH));
        assert!(matches!(transforms[1], Transform::MirrorV));
    }

    #[test]
    fn test_css_transform_to_transforms_all() {
        let mut css = CssTransform::new();
        css.translate = Some((10, 5));
        css.rotate = Some(90.0);
        css.scale = Some((2.0, 2.0));
        css.flip_x = true;
        css.flip_y = true;
        let transforms = css.to_transforms().unwrap();
        assert_eq!(transforms.len(), 5);
        // Order: translate → rotate → scale → flip_x → flip_y
        assert!(matches!(transforms[0], Transform::Shift { .. }));
        assert!(matches!(transforms[1], Transform::Rotate { .. }));
        assert!(matches!(transforms[2], Transform::Scale { .. }));
        assert!(matches!(transforms[3], Transform::MirrorH));
        assert!(matches!(transforms[4], Transform::MirrorV));
    }

    #[test]
    fn test_css_transform_is_empty() {
        let css = CssTransform::new();
        assert!(css.is_empty());

        let mut css2 = CssTransform::new();
        css2.translate = Some((0, 0));
        assert!(!css2.is_empty());

        let mut css3 = CssTransform::new();
        css3.flip_x = true;
        assert!(!css3.is_empty());
    }

    #[test]
    fn test_css_transform_error_display() {
        let err = CssTransformError::UnknownFunction("foo".to_string());
        assert_eq!(err.to_string(), "unknown CSS transform function: foo");

        let err = CssTransformError::InvalidParameter {
            func: "scale".to_string(),
            message: "must be positive".to_string(),
        };
        assert_eq!(err.to_string(), "invalid parameter for scale(): must be positive");

        let err = CssTransformError::MissingParameter {
            func: "translate".to_string(),
            param: "x".to_string(),
        };
        assert_eq!(err.to_string(), "missing required parameter for translate(): x");

        let err = CssTransformError::InvalidRotation(45.0);
        assert_eq!(err.to_string(), "invalid rotation: 45deg (pixel art requires 90, 180, or 270)");

        let err = CssTransformError::SyntaxError("bad syntax".to_string());
        assert_eq!(err.to_string(), "CSS transform syntax error: bad syntax");
    }

    // ========================================================================
    // Explain Transform Tests (LSP-12)
    // ========================================================================

    #[test]
    fn test_explain_transform_mirror() {
        assert_eq!(
            explain_transform(&Transform::MirrorH),
            "Flip horizontally (mirror left ↔ right)"
        );
        assert_eq!(explain_transform(&Transform::MirrorV), "Flip vertically (mirror top ↔ bottom)");
    }

    #[test]
    fn test_explain_transform_rotate() {
        assert_eq!(explain_transform(&Transform::Rotate { degrees: 90 }), "Rotate 90° clockwise");
        assert_eq!(explain_transform(&Transform::Rotate { degrees: 180 }), "Rotate 180°");
        assert_eq!(explain_transform(&Transform::Rotate { degrees: 270 }), "Rotate 270° clockwise");
    }

    #[test]
    fn test_explain_transform_tile() {
        assert_eq!(
            explain_transform(&Transform::Tile { w: 3, h: 2 }),
            "Tile 3×2 (repeat sprite in a grid)"
        );
    }

    #[test]
    fn test_explain_transform_pad() {
        assert_eq!(
            explain_transform(&Transform::Pad { size: 4 }),
            "Add 4 pixel(s) of transparent padding around edges"
        );
    }

    #[test]
    fn test_explain_transform_crop() {
        assert_eq!(
            explain_transform(&Transform::Crop { x: 2, y: 3, w: 10, h: 8 }),
            "Crop to 10×8 region starting at (2, 3)"
        );
    }

    #[test]
    fn test_explain_transform_outline() {
        assert_eq!(
            explain_transform(&Transform::Outline { token: None, width: 1 }),
            "Add 1px outline using default color"
        );
        assert_eq!(
            explain_transform(&Transform::Outline { token: Some("{black}".to_string()), width: 2 }),
            "Add 2px outline using {black}"
        );
    }

    #[test]
    fn test_explain_transform_shift() {
        assert_eq!(
            explain_transform(&Transform::Shift { x: 5, y: -3 }),
            "Shift 5 pixels 5 right, 3 pixels 3 up"
        );
    }

    #[test]
    fn test_explain_transform_scale() {
        assert_eq!(explain_transform(&Transform::Scale { x: 2.0, y: 2.0 }), "Scale to 200%");
        assert_eq!(
            explain_transform(&Transform::Scale { x: 2.0, y: 0.5 }),
            "Scale width to 200%, height to 50%"
        );
    }

    #[test]
    fn test_explain_transform_skew() {
        assert_eq!(
            explain_transform(&Transform::SkewX { degrees: 20.0 }),
            "Skew horizontally by 20° (shear along X axis)"
        );
        assert_eq!(
            explain_transform(&Transform::SkewY { degrees: -15.0 }),
            "Skew vertically by -15° (shear along Y axis)"
        );
    }

    #[test]
    fn test_explain_transform_pingpong() {
        assert_eq!(
            explain_transform(&Transform::Pingpong { exclude_ends: false }),
            "Play frames forward then backward (1,2,3 → 1,2,3,2,1)"
        );
        assert_eq!(
            explain_transform(&Transform::Pingpong { exclude_ends: true }),
            "Play frames forward then backward (excluding first/last to avoid doubling)"
        );
    }

    #[test]
    fn test_explain_transform_reverse() {
        assert_eq!(
            explain_transform(&Transform::Reverse),
            "Reverse frame order (play animation backwards)"
        );
    }

    #[test]
    fn test_explain_transform_frameoffset() {
        assert_eq!(
            explain_transform(&Transform::FrameOffset { offset: 2 }),
            "Shift animation timing 2 frame(s) later"
        );
        assert_eq!(
            explain_transform(&Transform::FrameOffset { offset: -3 }),
            "Shift animation timing 3 frame(s) earlier"
        );
    }

    #[test]
    fn test_explain_transform_hold() {
        assert_eq!(
            explain_transform(&Transform::Hold { frame: 0, count: 5 }),
            "Hold frame 1 for 5 extra frame(s)"
        );
    }

    #[test]
    fn test_explain_transform_dither() {
        assert_eq!(
            explain_transform(&Transform::Dither {
                pattern: DitherPattern::Checker,
                tokens: ("{dark}".to_string(), "{light}".to_string()),
                threshold: 0.5,
                seed: 0,
            }),
            "Apply checkerboard dither pattern between {dark} and {light} at 50% threshold"
        );
    }

    #[test]
    fn test_explain_transform_dither_gradient() {
        assert_eq!(
            explain_transform(&Transform::DitherGradient {
                direction: GradientDirection::Vertical,
                from: "{sky_top}".to_string(),
                to: "{sky_bottom}".to_string(),
                pattern: DitherPattern::Ordered4x4,
            }),
            "Dithered gradient from {sky_top} to {sky_bottom} (top to bottom, 4×4 Bayer pattern)"
        );
    }

    #[test]
    fn test_explain_transform_subpixel() {
        assert_eq!(
            explain_transform(&Transform::Subpixel { x: 0.5, y: 0.25 }),
            "Sub-pixel shift: 50% right, 25% down (smooth motion via color blending)"
        );
    }

    // ========================================================================
    // User-Defined Transform Tests (TRF-10)
    // ========================================================================

    #[test]
    fn test_expression_evaluator_numbers() {
        let eval = ExpressionEvaluator::new(std::collections::HashMap::new());
        assert!((eval.evaluate("42").unwrap() - 42.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("2.5").unwrap() - 2.5).abs() < 0.001);
        assert!((eval.evaluate("-5").unwrap() - (-5.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_expression_evaluator_variables() {
        let eval = ExpressionEvaluator::for_keyframe(5, 10);
        assert!((eval.evaluate("frame").unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("total_frames").unwrap() - 10.0).abs() < f64::EPSILON);
        // t = 5 / (10 - 1) = 5/9 ≈ 0.5556
        assert!((eval.evaluate("t").unwrap() - 0.5556).abs() < 0.01);
    }

    #[test]
    fn test_expression_evaluator_arithmetic() {
        let eval = ExpressionEvaluator::new(std::collections::HashMap::new());
        assert!((eval.evaluate("2 + 3").unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("10 - 4").unwrap() - 6.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("3 * 4").unwrap() - 12.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("15 / 3").unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("2 + 3 * 4").unwrap() - 14.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("(2 + 3) * 4").unwrap() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_expression_evaluator_functions() {
        let eval = ExpressionEvaluator::new(std::collections::HashMap::new());
        assert!((eval.evaluate("abs(-5)").unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("floor(3.7)").unwrap() - 3.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("ceil(3.2)").unwrap() - 4.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("round(3.5)").unwrap() - 4.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("min(5, 3)").unwrap() - 3.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("max(5, 3)").unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("sqrt(16)").unwrap() - 4.0).abs() < f64::EPSILON);
        assert!((eval.evaluate("pow(2, 3)").unwrap() - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_expression_evaluator_trig() {
        let eval = ExpressionEvaluator::new(std::collections::HashMap::new());
        assert!((eval.evaluate("sin(0)").unwrap()).abs() < 0.001);
        assert!((eval.evaluate("cos(0)").unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_expression_evaluator_param_substitution() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("amplitude".to_string(), 10.0);
        let eval = ExpressionEvaluator::new(vars);
        assert!((eval.evaluate("${amplitude} * 2").unwrap() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpolate_keyframes_linear() {
        let keyframes = vec![[0.0, 0.0], [10.0, 100.0]];
        let easing = crate::models::Easing::Linear;

        assert!((interpolate_keyframes(&keyframes, 0.0, &easing) - 0.0).abs() < f64::EPSILON);
        assert!((interpolate_keyframes(&keyframes, 5.0, &easing) - 50.0).abs() < f64::EPSILON);
        assert!((interpolate_keyframes(&keyframes, 10.0, &easing) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpolate_keyframes_before_first() {
        let keyframes = vec![[5.0, 50.0], [10.0, 100.0]];
        let easing = crate::models::Easing::Linear;

        // Before first keyframe should return first value
        assert!((interpolate_keyframes(&keyframes, 0.0, &easing) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpolate_keyframes_after_last() {
        let keyframes = vec![[0.0, 0.0], [5.0, 50.0]];
        let easing = crate::models::Easing::Linear;

        // After last keyframe should return last value
        assert!((interpolate_keyframes(&keyframes, 10.0, &easing) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_easing_linear() {
        let easing = crate::models::Easing::Linear;
        assert!((easing.apply(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((easing.apply(0.5) - 0.5).abs() < f64::EPSILON);
        assert!((easing.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = crate::models::Easing::EaseIn;
        assert!((easing.apply(0.0) - 0.0).abs() < f64::EPSILON);
        assert!(easing.apply(0.5) < 0.5); // Should be slower than linear at midpoint
        assert!((easing.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = crate::models::Easing::EaseOut;
        assert!((easing.apply(0.0) - 0.0).abs() < f64::EPSILON);
        assert!(easing.apply(0.5) > 0.5); // Should be faster than linear at midpoint
        assert!((easing.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_property_values_to_transforms_shift() {
        let mut props = std::collections::HashMap::new();
        props.insert("shift-x".to_string(), 5.0);
        props.insert("shift-y".to_string(), -3.0);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        match &transforms[0] {
            Transform::Shift { x, y } => {
                assert_eq!(*x, 5);
                assert_eq!(*y, -3);
            }
            _ => panic!("Expected Shift transform"),
        }
    }

    #[test]
    fn test_property_values_to_transforms_scale() {
        let mut props = std::collections::HashMap::new();
        props.insert("scale-x".to_string(), 2.0);
        props.insert("scale-y".to_string(), 0.5);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        match &transforms[0] {
            Transform::Scale { x, y } => {
                assert!((x - 2.0).abs() < 0.001);
                assert!((y - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected Scale transform"),
        }
    }

    // ============================================================================
    // Anchor Preservation Scaling Tests (TTP-ca8cj)
    // ============================================================================

    #[test]
    fn test_anchor_bounds_new() {
        let bounds = AnchorBounds::new(10, 20, 5, 3);
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 20);
        assert_eq!(bounds.width, 5);
        assert_eq!(bounds.height, 3);
    }

    #[test]
    fn test_anchor_bounds_from_points() {
        let points = vec![(5, 10), (7, 10), (6, 11), (5, 12)];
        let bounds = AnchorBounds::from_points(&points).unwrap();

        assert_eq!(bounds.x, 5);
        assert_eq!(bounds.y, 10);
        assert_eq!(bounds.width, 3); // 5, 6, 7 -> width 3
        assert_eq!(bounds.height, 3); // 10, 11, 12 -> height 3
    }

    #[test]
    fn test_anchor_bounds_from_points_single() {
        let points = vec![(5, 10)];
        let bounds = AnchorBounds::from_points(&points).unwrap();

        assert_eq!(bounds.x, 5);
        assert_eq!(bounds.y, 10);
        assert_eq!(bounds.width, 1);
        assert_eq!(bounds.height, 1);
    }

    #[test]
    fn test_anchor_bounds_from_points_empty() {
        let points: Vec<(i32, i32)> = vec![];
        let bounds = AnchorBounds::from_points(&points);
        assert!(bounds.is_none());
    }

    #[test]
    fn test_anchor_bounds_center() {
        let bounds = AnchorBounds::new(10, 20, 6, 4);
        let (cx, cy) = bounds.center();
        assert_eq!(cx, 13); // 10 + 6/2 = 13
        assert_eq!(cy, 22); // 20 + 4/2 = 22
    }

    #[test]
    fn test_anchor_bounds_scaled() {
        let bounds = AnchorBounds::new(10, 20, 4, 6);
        let scaled = bounds.scaled(0.5, 0.5);

        assert_eq!(scaled.x, 5); // 10 * 0.5 = 5
        assert_eq!(scaled.y, 10); // 20 * 0.5 = 10
        assert_eq!(scaled.width, 2); // 4 * 0.5 = 2
        assert_eq!(scaled.height, 3); // 6 * 0.5 = 3
    }

    #[test]
    fn test_scale_image_noop() {
        use image::Rgba;

        let mut image = RgbaImage::new(4, 4);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let scaled = scale_image(&image, 1.0, 1.0);

        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);
        assert_eq!(*scaled.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_scale_image_upscale() {
        use image::Rgba;

        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Red
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // Green
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255])); // Blue
        image.put_pixel(1, 1, Rgba([255, 255, 0, 255])); // Yellow

        let scaled = scale_image(&image, 2.0, 2.0);

        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);

        // Red block (0,0) -> (0,0), (1,0), (0,1), (1,1)
        assert_eq!(*scaled.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*scaled.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_scale_image_downscale() {
        use image::Rgba;

        let mut image = RgbaImage::new(4, 4);
        // Fill with red
        for y in 0..4 {
            for x in 0..4 {
                image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let scaled = scale_image(&image, 0.5, 0.5);

        assert_eq!(scaled.width(), 2);
        assert_eq!(scaled.height(), 2);
    }

    #[test]
    fn test_scale_image_with_anchor_preservation_no_anchors() {
        use image::Rgba;

        let mut image = RgbaImage::new(4, 4);
        image.put_pixel(1, 1, Rgba([255, 0, 0, 255]));

        let anchors: Vec<AnchorBounds> = vec![];
        let scaled = scale_image_with_anchor_preservation(&image, 0.5, 0.5, &anchors);

        assert_eq!(scaled.width(), 2);
        assert_eq!(scaled.height(), 2);
    }

    #[test]
    fn test_scale_image_with_anchor_preservation_preserves_small_anchor() {
        use image::Rgba;

        // Create an 8x8 image with a 2x2 "eye" region at (3, 3)
        let mut image = RgbaImage::new(8, 8);

        // Fill with transparent
        for y in 0..8 {
            for x in 0..8 {
                image.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }

        // Draw the "eye" (anchor) at (3, 3) with size 2x2
        image.put_pixel(3, 3, Rgba([0, 0, 255, 255])); // Blue eye
        image.put_pixel(4, 3, Rgba([0, 0, 255, 255]));
        image.put_pixel(3, 4, Rgba([0, 0, 255, 255]));
        image.put_pixel(4, 4, Rgba([0, 0, 255, 255]));

        // Define the anchor region
        let anchors = vec![AnchorBounds::new(3, 3, 2, 2)];

        // Scale down to 25% - the 2x2 eye would normally shrink to 0x0 or less than 1px
        let scaled = scale_image_with_anchor_preservation(&image, 0.25, 0.25, &anchors);

        assert_eq!(scaled.width(), 2); // 8 * 0.25 = 2
        assert_eq!(scaled.height(), 2);

        // The anchor center is at (4, 4) -> maps to (1, 1) in scaled image
        // Check that at least one blue pixel exists in the scaled image
        let mut found_blue = false;
        for y in 0..scaled.height() {
            for x in 0..scaled.width() {
                let pixel = scaled.get_pixel(x, y);
                if pixel[2] > 200 && pixel[3] > 200 {
                    // Blue with alpha
                    found_blue = true;
                    break;
                }
            }
        }
        assert!(found_blue, "Anchor region should be preserved during downscaling");
    }

    #[test]
    fn test_scale_image_with_anchor_preservation_upscale_passthrough() {
        use image::Rgba;

        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        // Even with anchors, upscaling should work normally
        let anchors = vec![AnchorBounds::new(0, 0, 1, 1)];
        let scaled = scale_image_with_anchor_preservation(&image, 2.0, 2.0, &anchors);

        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);
    }
}
