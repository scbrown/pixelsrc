//! Core transform types and error definitions
//!
//! Contains the `Transform` enum representing all supported transform operations
//! and `TransformError` for error handling during parsing and application.

use std::collections::HashMap;

use super::dither::{DitherPattern, GradientDirection};

/// Errors that can occur during transform parsing or application
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
