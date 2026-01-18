//! Transform operations for sprites and animations
//!
//! Supports both CLI transforms (`pxl transform`) and format attributes
//! (`"transform": ["mirror-h", "rotate:90"]`).

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

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
            "ordered-2x2" | "ordered2x2" | "bayer-2x2" | "bayer2x2" => Some(DitherPattern::Ordered2x2),
            "ordered-4x4" | "ordered4x4" | "bayer-4x4" | "bayer4x4" => Some(DitherPattern::Ordered4x4),
            "ordered-8x8" | "ordered8x8" | "bayer-8x8" | "bayer8x8" => Some(DitherPattern::Ordered8x8),
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
                if (x + y) % 2 == 0 { 0.25 } else { 0.75 }
            }
            DitherPattern::Ordered2x2 => {
                // 2x2 Bayer matrix:
                // | 0 2 |   normalized: | 0.0  0.5  |
                // | 3 1 |               | 0.75 0.25 |
                const BAYER_2X2: [[f64; 2]; 2] = [
                    [0.0 / 4.0, 2.0 / 4.0],
                    [3.0 / 4.0, 1.0 / 4.0],
                ];
                let px = (x % 2) as usize;
                let py = (y % 2) as usize;
                BAYER_2X2[py][px]
            }
            DitherPattern::Ordered4x4 => {
                // 4x4 Bayer matrix
                const BAYER_4X4: [[f64; 4]; 4] = [
                    [ 0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0],
                    [12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0],
                    [ 3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0],
                    [15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0],
                ];
                let px = (x % 4) as usize;
                let py = (y % 4) as usize;
                BAYER_4X4[py][px]
            }
            DitherPattern::Ordered8x8 => {
                // 8x8 Bayer matrix
                const BAYER_8X8: [[f64; 8]; 8] = [
                    [ 0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,  2.0/64.0, 34.0/64.0, 10.0/64.0, 42.0/64.0],
                    [48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0, 50.0/64.0, 18.0/64.0, 58.0/64.0, 26.0/64.0],
                    [12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0, 14.0/64.0, 46.0/64.0,  6.0/64.0, 38.0/64.0],
                    [60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0, 62.0/64.0, 30.0/64.0, 54.0/64.0, 22.0/64.0],
                    [ 3.0/64.0, 35.0/64.0, 11.0/64.0, 43.0/64.0,  1.0/64.0, 33.0/64.0,  9.0/64.0, 41.0/64.0],
                    [51.0/64.0, 19.0/64.0, 59.0/64.0, 27.0/64.0, 49.0/64.0, 17.0/64.0, 57.0/64.0, 25.0/64.0],
                    [15.0/64.0, 47.0/64.0,  7.0/64.0, 39.0/64.0, 13.0/64.0, 45.0/64.0,  5.0/64.0, 37.0/64.0],
                    [63.0/64.0, 31.0/64.0, 55.0/64.0, 23.0/64.0, 61.0/64.0, 29.0/64.0, 53.0/64.0, 21.0/64.0],
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformError {
    /// Unknown transform operation
    UnknownOperation(String),

    /// Invalid parameter value
    InvalidParameter { op: String, message: String },

    /// Missing required parameter
    MissingParameter { op: String, param: String },

    /// Invalid rotation degrees (must be 90, 180, or 270)
    InvalidRotation(u16),

    /// Invalid tile dimensions
    InvalidTileDimensions(String),

    /// Invalid crop region
    InvalidCropRegion(String),

    /// Invalid shift values
    InvalidShift(String),

    /// General parse error
    ParseError(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::UnknownOperation(op) => {
                write!(f, "unknown transform operation: {}", op)
            }
            TransformError::InvalidParameter { op, message } => {
                write!(f, "invalid parameter for {}: {}", op, message)
            }
            TransformError::MissingParameter { op, param } => {
                write!(f, "missing required parameter for {}: {}", op, param)
            }
            TransformError::InvalidRotation(degrees) => {
                write!(
                    f,
                    "invalid rotation degrees: {} (must be 90, 180, or 270)",
                    degrees
                )
            }
            TransformError::InvalidTileDimensions(dims) => {
                write!(f, "invalid tile dimensions: {}", dims)
            }
            TransformError::InvalidCropRegion(region) => {
                write!(f, "invalid crop region: {}", region)
            }
            TransformError::InvalidShift(shift) => {
                write!(f, "invalid shift values: {}", shift)
            }
            TransformError::ParseError(msg) => {
                write!(f, "parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TransformError {}

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

/// Parse transform from string syntax: "mirror-h", "rotate:90", "tile:3x2"
///
/// # Alias Resolution
/// - `symmetry-h`, `flip-h` → `MirrorH`
/// - `symmetry-v`, `flip-v` → `MirrorV`
/// - `rot` → `Rotate`
pub fn parse_transform_str(s: &str) -> Result<Transform, TransformError> {
    let s = s.trim();

    // Split on colon to get operation and params
    let (op, params) = if let Some(idx) = s.find(':') {
        (&s[..idx], Some(&s[idx + 1..]))
    } else {
        (s, None)
    };

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
            let (token, width) = if let Some(p) = params {
                parse_outline_params(p)?
            } else {
                (None, 1)
            };
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
            Ok(Transform::SelOut {
                fallback,
                mapping: None,
            })
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

        // Animation
        "pingpong" => {
            let exclude_ends = params
                .map(|p| p == "true" || p == "exclude_ends")
                .unwrap_or(false);
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
        _ => Err(TransformError::ParseError(
            "transform must be string or object".to_string(),
        )),
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
            let token = params
                .get("token")
                .and_then(|v| v.as_str())
                .map(String::from);
            let width = params
                .get("width")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(1);
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
            let token = params
                .get("token")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(Transform::Shadow { x, y, token })
        }
        "sel-out" | "selout" => {
            let fallback = params
                .get("fallback")
                .and_then(|v| v.as_str())
                .map(String::from);
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
            let x = params
                .get("x")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "scale".to_string(),
                    param: "x".to_string(),
                })?;
            let y = params
                .get("y")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "scale".to_string(),
                    param: "y".to_string(),
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

        // Animation
        "pingpong" => {
            let exclude_ends = params
                .get("exclude_ends")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Transform::Pingpong { exclude_ends })
        }
        "reverse" => Ok(Transform::Reverse),
        "frame-offset" | "frameoffset" => {
            let offset = get_i32_param(params, "offset", "frame-offset")?;
            Ok(Transform::FrameOffset { offset })
        }
        "hold" => {
            let frame = params
                .get("frame")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "hold".to_string(),
                    param: "frame".to_string(),
                })?;
            let count = params
                .get("count")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "hold".to_string(),
                    param: "count".to_string(),
                })?;
            Ok(Transform::Hold { frame, count })
        }

        // Dithering (ATF-8)
        "dither" => {
            let pattern_str = params
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "dither".to_string(),
                    param: "pattern".to_string(),
                })?;
            let pattern = DitherPattern::from_str(pattern_str).ok_or_else(|| {
                TransformError::InvalidParameter {
                    op: "dither".to_string(),
                    message: format!("unknown dither pattern: {}", pattern_str),
                }
            })?;

            let tokens_arr = params
                .get("tokens")
                .and_then(|v| v.as_array())
                .ok_or_else(|| TransformError::MissingParameter {
                    op: "dither".to_string(),
                    param: "tokens".to_string(),
                })?;
            if tokens_arr.len() != 2 {
                return Err(TransformError::InvalidParameter {
                    op: "dither".to_string(),
                    message: format!("tokens must have exactly 2 elements, got {}", tokens_arr.len()),
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

            let threshold = params
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);
            let seed = params
                .get("seed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            Ok(Transform::Dither {
                pattern,
                tokens: (dark_token, light_token),
                threshold,
                seed,
            })
        }

        "dither-gradient" | "dithergradient" => {
            let direction_str = params
                .get("direction")
                .and_then(|v| v.as_str())
                .unwrap_or("vertical");
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

            let pattern_str = params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("ordered-4x4");
            let pattern = DitherPattern::from_str(pattern_str).ok_or_else(|| {
                TransformError::InvalidParameter {
                    op: "dither-gradient".to_string(),
                    message: format!("unknown dither pattern: {}", pattern_str),
                }
            })?;

            Ok(Transform::DitherGradient {
                direction,
                from,
                to,
                pattern,
            })
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
    let x = parts[0]
        .trim()
        .parse::<i32>()
        .map_err(|_| TransformError::InvalidShift(s.to_string()))?;
    let y = parts[1]
        .trim()
        .parse::<i32>()
        .map_err(|_| TransformError::InvalidShift(s.to_string()))?;
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
                parts[1]
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| TransformError::InvalidParameter {
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
    let x = parts[0]
        .trim()
        .parse::<i32>()
        .map_err(|_| TransformError::InvalidParameter {
            op: "shadow".to_string(),
            message: format!("cannot parse '{}' as X offset", parts[0]),
        })?;
    let y = parts[1]
        .trim()
        .parse::<i32>()
        .map_err(|_| TransformError::InvalidParameter {
            op: "shadow".to_string(),
            message: format!("cannot parse '{}' as Y offset", parts[1]),
        })?;
    let token = if parts.len() > 2 {
        Some(parts[2].trim().to_string())
    } else {
        None
    };
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
    let x = parts[0]
        .trim()
        .parse::<f32>()
        .map_err(|_| TransformError::InvalidParameter {
            op: "scale".to_string(),
            message: format!("cannot parse '{}' as X scale factor", parts[0]),
        })?;
    let y = parts[1]
        .trim()
        .parse::<f32>()
        .map_err(|_| TransformError::InvalidParameter {
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

fn parse_hold_params(s: &str) -> Result<(usize, usize), TransformError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(TransformError::InvalidParameter {
            op: "hold".to_string(),
            message: format!("expected 'frame,count', got '{}'", s),
        });
    }
    let frame = parts[0]
        .trim()
        .parse::<usize>()
        .map_err(|_| TransformError::InvalidParameter {
            op: "hold".to_string(),
            message: format!("cannot parse '{}' as frame index", parts[0]),
        })?;
    let count = parts[1]
        .trim()
        .parse::<usize>()
        .map_err(|_| TransformError::InvalidParameter {
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
    let pattern = DitherPattern::from_str(parts[0]).ok_or_else(|| {
        TransformError::InvalidParameter {
            op: "dither".to_string(),
            message: format!("unknown dither pattern: {}", parts[0]),
        }
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
    let (dark_token, light_token) = parse_token_pair(tokens_str).ok_or_else(|| {
        TransformError::InvalidParameter {
            op: "dither".to_string(),
            message: format!("expected 'dark_token,light_token', got '{}'", tokens_str),
        }
    })?;

    // Third part (optional) is threshold
    let threshold = if parts.len() >= 3 {
        parts[2]
            .trim()
            .parse::<f64>()
            .map_err(|_| TransformError::InvalidParameter {
                op: "dither".to_string(),
                message: format!("cannot parse '{}' as threshold", parts[2]),
            })?
    } else {
        0.5
    };

    // Fourth part (optional) is seed
    let seed = if parts.len() >= 4 {
        parts[3]
            .trim()
            .parse::<u64>()
            .map_err(|_| TransformError::InvalidParameter {
                op: "dither".to_string(),
                message: format!("cannot parse '{}' as seed", parts[3]),
            })?
    } else {
        0
    };

    Ok(Transform::Dither {
        pattern,
        tokens: (dark_token, light_token),
        threshold,
        seed,
    })
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
    let direction = GradientDirection::from_str(parts[0]).ok_or_else(|| {
        TransformError::InvalidParameter {
            op: "dither-gradient".to_string(),
            message: format!("unknown direction: {} (expected vertical, horizontal, or radial)", parts[0]),
        }
    })?;

    // Second part should be tokens: from,to
    if parts.len() < 2 {
        return Err(TransformError::MissingParameter {
            op: "dither-gradient".to_string(),
            param: "tokens (from,to)".to_string(),
        });
    }
    let tokens_str = parts[1];
    let (from, to) = parse_token_pair(tokens_str).ok_or_else(|| {
        TransformError::InvalidParameter {
            op: "dither-gradient".to_string(),
            message: format!("expected 'from_token,to_token', got '{}'", tokens_str),
        }
    })?;

    // Third part (optional) is pattern
    let pattern = if parts.len() >= 3 {
        DitherPattern::from_str(parts[2]).ok_or_else(|| {
            TransformError::InvalidParameter {
                op: "dither-gradient".to_string(),
                message: format!("unknown dither pattern: {}", parts[2]),
            }
        })?
    } else {
        DitherPattern::Ordered4x4 // Default pattern
    };

    Ok(Transform::DitherGradient {
        direction,
        from,
        to,
        pattern,
    })
}

/// Parse subpixel from string syntax: x,y (values 0.0-1.0)
/// Example: "0.5,0.25"
fn parse_subpixel_str(s: &str) -> Result<Transform, TransformError> {
    let parts: Vec<&str> = s.split(',').collect();

    let x = if !parts.is_empty() && !parts[0].is_empty() {
        parts[0]
            .trim()
            .parse::<f64>()
            .map_err(|_| TransformError::InvalidParameter {
                op: "subpixel".to_string(),
                message: format!("cannot parse '{}' as x offset", parts[0]),
            })?
    } else {
        0.0
    };

    let y = if parts.len() > 1 && !parts[1].is_empty() {
        parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|_| TransformError::InvalidParameter {
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
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .ok_or_else(|| TransformError::MissingParameter {
            op: op.to_string(),
            param: key.to_string(),
        })
}

fn get_u32_param(
    params: &HashMap<String, Value>,
    key: &str,
    op: &str,
) -> Result<u32, TransformError> {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .ok_or_else(|| TransformError::MissingParameter {
            op: op.to_string(),
            param: key.to_string(),
        })
}

fn get_i32_param(
    params: &HashMap<String, Value>,
    key: &str,
    op: &str,
) -> Result<i32, TransformError> {
    params
        .get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| TransformError::MissingParameter {
            op: op.to_string(),
            param: key.to_string(),
        })
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
// Grid Transform Application Functions
// ============================================================================

/// The transparent token used in grids
const TRANSPARENT_TOKEN: &str = "{_}";

/// Apply selective outline (sel-out) transform to a grid.
///
/// Selective outline varies the outline color based on adjacent fill pixels,
/// creating softer edges. For each outline pixel (a pixel adjacent to both
/// opaque and transparent pixels), the transform picks a color based on the
/// most common neighboring fill color.
///
/// # Arguments
/// * `grid` - The grid of token rows
/// * `fallback` - Optional fallback token for pixels where neighbor can't be determined
/// * `mapping` - Optional explicit mapping from fill token to outline token.
///               Key "*" serves as the default fallback.
///
/// # Returns
/// A new grid with outline pixels recolored based on neighbors
///
/// # Algorithm
/// 1. Parse each row into tokens
/// 2. For each pixel, check if it's an "outline" pixel:
///    - An outline pixel is opaque (not {_}) and adjacent to at least one {_}
/// 3. For outline pixels, find the most common non-transparent neighbor
/// 4. Map that neighbor to an outline color using the mapping or by
///    appending "_dark" to the token name
pub fn apply_selout(
    grid: &[String],
    fallback: Option<&str>,
    mapping: Option<&HashMap<String, String>>,
) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    // Parse grid into 2D token array
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();

    let height = parsed.len();
    let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width == 0 {
        return grid.to_vec();
    }

    // Helper to get token at position (with bounds checking)
    let get_token = |x: i32, y: i32| -> Option<&String> {
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return None;
        }
        parsed
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
    };

    // Helper to check if a token is transparent
    let is_transparent = |token: &str| -> bool { token == TRANSPARENT_TOKEN };

    // Helper to check if a position is an outline pixel
    // (opaque pixel adjacent to at least one transparent {_} pixel)
    // Note: out-of-bounds does NOT count as transparent - only explicit {_} does
    let is_outline_pixel = |x: i32, y: i32| -> bool {
        let token = match get_token(x, y) {
            Some(t) => t,
            None => return false,
        };

        // Must be opaque
        if is_transparent(token) {
            return false;
        }

        // Check 4-connected neighbors for explicit {_} transparency
        let neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for (dx, dy) in neighbors {
            if let Some(t) = get_token(x + dx, y + dy) {
                if is_transparent(t) {
                    return true;
                }
            }
            // Out of bounds does NOT count as transparent
        }
        false
    };

    // Helper to find the most common non-transparent neighbor
    // This determines what fill color the outline pixel should base its outline on
    let get_dominant_neighbor = |x: i32, y: i32| -> Option<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();

        // Check 8-connected neighbors (including diagonals)
        let neighbors = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for (dx, dy) in neighbors {
            if let Some(token) = get_token(x + dx, y + dy) {
                if !is_transparent(token) {
                    *counts.entry(token.clone()).or_insert(0) += 1;
                }
            }
        }

        // Find most common opaque neighbor
        // Don't filter out current token - the outline pixel should transform
        // based on the dominant fill color in its neighborhood
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(token, _)| token)
    };

    // Helper to map a fill token to its outline color
    // Priority: explicit mapping > wildcard mapping > auto-dark suffix
    // Fallback is only used when NO dominant neighbor is found (not here)
    let get_outline_token = |fill_token: &str| -> String {
        // First check explicit mapping
        if let Some(map) = mapping {
            if let Some(outline) = map.get(fill_token) {
                return outline.clone();
            }
            // Check wildcard
            if let Some(default) = map.get("*") {
                return default.clone();
            }
        }

        // Auto-generate: append _dark to token name
        // {skin} -> {skin_dark}
        if fill_token.starts_with('{') && fill_token.ends_with('}') {
            let inner = &fill_token[1..fill_token.len() - 1];
            format!("{{{}_dark}}", inner)
        } else {
            fill_token.to_string()
        }
    };

    // Transform the grid
    let mut result: Vec<String> = Vec::with_capacity(height);

    for (y, row) in parsed.iter().enumerate() {
        let mut new_row = String::new();
        for (x, token) in row.iter().enumerate() {
            if is_outline_pixel(x as i32, y as i32) {
                // Find dominant neighbor and map to outline color
                if let Some(neighbor) = get_dominant_neighbor(x as i32, y as i32) {
                    new_row.push_str(&get_outline_token(&neighbor));
                } else {
                    // No non-self neighbor found, use fallback or keep original
                    if let Some(fb) = fallback {
                        new_row.push_str(fb);
                    } else if let Some(map) = mapping {
                        if let Some(default) = map.get("*") {
                            new_row.push_str(default);
                        } else {
                            new_row.push_str(token);
                        }
                    } else {
                        new_row.push_str(token);
                    }
                }
            } else {
                new_row.push_str(token);
            }
        }
        result.push(new_row);
    }

    result
}

/// Apply scale transform to a grid.
///
/// Scales the grid by the given X and Y factors using nearest-neighbor
/// interpolation, which preserves the crisp pixel art look.
///
/// # Arguments
/// * `grid` - The grid of token rows
/// * `scale_x` - Horizontal scale factor (1.0 = no change)
/// * `scale_y` - Vertical scale factor (1.0 = no change)
///
/// # Returns
/// A new grid scaled by the given factors
///
/// # Algorithm
/// Uses nearest-neighbor scaling:
/// - For each pixel in the output, finds the corresponding pixel in the input
/// - Works for both scaling up (duplication) and down (sampling)
pub fn apply_scale(grid: &[String], scale_x: f32, scale_y: f32) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() || scale_x <= 0.0 || scale_y <= 0.0 {
        return Vec::new();
    }

    // Parse grid into 2D token array
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();

    let src_height = parsed.len();
    let src_width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if src_width == 0 {
        return grid.to_vec();
    }

    // Calculate new dimensions
    let dst_width = ((src_width as f32) * scale_x).round() as usize;
    let dst_height = ((src_height as f32) * scale_y).round() as usize;

    if dst_width == 0 || dst_height == 0 {
        return Vec::new();
    }

    // Build the scaled grid using nearest-neighbor sampling
    let mut result: Vec<String> = Vec::with_capacity(dst_height);

    for dst_y in 0..dst_height {
        let mut new_row = String::new();

        // Find source Y coordinate
        let src_y = ((dst_y as f32) / scale_y).floor() as usize;
        let src_y = src_y.min(src_height - 1);

        let src_row = &parsed[src_y];

        for dst_x in 0..dst_width {
            // Find source X coordinate
            let src_x = ((dst_x as f32) / scale_x).floor() as usize;
            let src_x = src_x.min(src_row.len().saturating_sub(1));

            if src_x < src_row.len() {
                new_row.push_str(&src_row[src_x]);
            } else {
                // Pad with transparent if source row is shorter
                new_row.push_str(TRANSPARENT_TOKEN);
            }
        }

        result.push(new_row);
    }

    result
}

/// Apply outline transform to a grid.
///
/// Adds an outline of the specified token around all opaque (non-transparent)
/// pixels. The outline is placed on transparent pixels adjacent to opaque ones.
///
/// # Arguments
/// * `grid` - The grid of token rows
/// * `token` - The token to use for the outline (defaults to `{outline}` if None)
/// * `width` - The outline width in pixels (default 1)
///
/// # Returns
/// A new grid with outline added around opaque pixels
///
/// # Algorithm
/// 1. Parse each row into tokens
/// 2. For each transparent pixel, check if any opaque pixel exists within `width` distance
/// 3. If so, replace the transparent pixel with the outline token
pub fn apply_outline(grid: &[String], token: Option<&str>, width: u32) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() || width == 0 {
        return grid.to_vec();
    }

    let outline_token = token.unwrap_or("{outline}");

    // Parse grid into 2D token array
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();

    let height = parsed.len();
    let width_pixels = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width_pixels == 0 {
        return grid.to_vec();
    }

    // Helper to check if a token is transparent
    let is_transparent = |token: &str| -> bool { token == TRANSPARENT_TOKEN };

    // Helper to get token at position (with bounds checking)
    let get_token = |x: i32, y: i32| -> Option<&String> {
        if x < 0 || y < 0 || x >= width_pixels as i32 || y >= height as i32 {
            return None;
        }
        parsed
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
    };

    // Check if any opaque pixel exists within `width` distance of position
    // Uses Chebyshev distance (max of |dx|, |dy|) to include diagonals
    let has_opaque_neighbor = |x: i32, y: i32, outline_width: u32| -> bool {
        let w = outline_width as i32;
        for dy in -w..=w {
            for dx in -w..=w {
                if dx == 0 && dy == 0 {
                    continue;
                }
                // Chebyshev distance: max of |dx| and |dy|
                // This includes diagonal neighbors at distance 1
                if dx.abs().max(dy.abs()) > w {
                    continue;
                }
                if let Some(t) = get_token(x + dx, y + dy) {
                    if !is_transparent(t) {
                        return true;
                    }
                }
            }
        }
        false
    };

    // Transform the grid
    let mut result: Vec<String> = Vec::with_capacity(height);

    for (y, row) in parsed.iter().enumerate() {
        let mut new_row = String::new();
        for (x, tok) in row.iter().enumerate() {
            if is_transparent(tok) && has_opaque_neighbor(x as i32, y as i32, width) {
                new_row.push_str(outline_token);
            } else {
                new_row.push_str(tok);
            }
        }
        result.push(new_row);
    }

    result
}

/// Apply shift transform to a grid.
///
/// Shifts all pixels in the grid by the specified x and y offsets.
/// Pixels that shift outside the bounds are lost, and empty space
/// is filled with transparent tokens.
///
/// # Arguments
/// * `grid` - The grid of token rows
/// * `x` - Horizontal shift (positive = right, negative = left)
/// * `y` - Vertical shift (positive = down, negative = up)
///
/// # Returns
/// A new grid with content shifted by the specified amounts
pub fn apply_shift(grid: &[String], x: i32, y: i32) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    // Parse grid into 2D token array
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();

    let height = parsed.len();
    let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width == 0 {
        return grid.to_vec();
    }

    // Helper to get token at position (with bounds checking)
    let get_token = |px: i32, py: i32| -> Option<&String> {
        if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
            return None;
        }
        parsed
            .get(py as usize)
            .and_then(|row| row.get(px as usize))
    };

    // Build shifted grid
    let mut result: Vec<String> = Vec::with_capacity(height);

    for dst_y in 0..height {
        let mut new_row = String::new();
        let src_y = dst_y as i32 - y;

        for dst_x in 0..width {
            let src_x = dst_x as i32 - x;

            if let Some(token) = get_token(src_x, src_y) {
                new_row.push_str(token);
            } else {
                new_row.push_str(TRANSPARENT_TOKEN);
            }
        }
        result.push(new_row);
    }

    result
}

/// Apply shadow transform to a grid.
///
/// Creates a drop shadow effect by painting a copy of the opaque pixels
/// at the specified offset, then overlaying the original on top.
///
/// # Arguments
/// * `grid` - The grid of token rows
/// * `x` - Horizontal shadow offset (positive = right, negative = left)
/// * `y` - Vertical shadow offset (positive = down, negative = up)
/// * `token` - The token to use for the shadow (defaults to `{shadow}` if None)
///
/// # Returns
/// A new grid with shadow effect applied
///
/// # Algorithm
/// 1. Create a shadow layer: shift opaque pixels by (x, y) and paint with shadow token
/// 2. Composite original on top of shadow layer
pub fn apply_shadow(grid: &[String], x: i32, y: i32, token: Option<&str>) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    let shadow_token = token.unwrap_or("{shadow}");

    // Parse grid into 2D token array
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();

    let height = parsed.len();
    let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width == 0 {
        return grid.to_vec();
    }

    // Helper to check if a token is transparent
    let is_transparent = |tok: &str| -> bool { tok == TRANSPARENT_TOKEN };

    // Helper to get token at position (with bounds checking)
    let get_token = |px: i32, py: i32| -> Option<&String> {
        if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
            return None;
        }
        parsed
            .get(py as usize)
            .and_then(|row| row.get(px as usize))
    };

    // Build the result grid
    // For each position, check:
    // 1. If original pixel is opaque -> use original
    // 2. Else if shadow source pixel (at offset) is opaque -> use shadow token
    // 3. Else -> use transparent
    let mut result: Vec<String> = Vec::with_capacity(height);

    for dst_y in 0..height {
        let mut new_row = String::new();

        for dst_x in 0..width {
            // Check original pixel
            if let Some(orig_token) = get_token(dst_x as i32, dst_y as i32) {
                if !is_transparent(orig_token) {
                    // Original is opaque, keep it
                    new_row.push_str(orig_token);
                    continue;
                }
            }

            // Check shadow source (original position minus offset = where shadow comes from)
            let shadow_src_x = dst_x as i32 - x;
            let shadow_src_y = dst_y as i32 - y;

            if let Some(src_token) = get_token(shadow_src_x, shadow_src_y) {
                if !is_transparent(src_token) {
                    // Shadow source is opaque, paint shadow
                    new_row.push_str(shadow_token);
                    continue;
                }
            }

            // Both transparent, keep transparent
            new_row.push_str(TRANSPARENT_TOKEN);
        }
        result.push(new_row);
    }

    result
}

/// Apply horizontal mirror (flip left-to-right) to a grid.
///
/// Each row is reversed, so the leftmost column becomes the rightmost.
///
/// # Arguments
/// * `grid` - The grid of token rows
///
/// # Returns
/// A new grid with rows reversed horizontally
pub fn apply_mirror_horizontal(grid: &[String]) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    // Parse grid into 2D token array and reverse each row
    grid.iter()
        .map(|row| {
            let tokens = tokenize(row).0;
            let reversed: Vec<&str> = tokens.iter().rev().map(|s| s.as_str()).collect();
            reversed.concat()
        })
        .collect()
}

/// Apply vertical mirror (flip top-to-bottom) to a grid.
///
/// The row order is reversed, so the top row becomes the bottom.
///
/// # Arguments
/// * `grid` - The grid of token rows
///
/// # Returns
/// A new grid with rows in reversed order
pub fn apply_mirror_vertical(grid: &[String]) -> Vec<String> {
    grid.iter().rev().cloned().collect()
}

/// Apply rotation to a grid.
///
/// Rotates the grid clockwise by the specified degrees (90, 180, or 270).
///
/// # Arguments
/// * `grid` - The grid of token rows
/// * `degrees` - The rotation angle (must be 90, 180, or 270)
///
/// # Returns
/// A new rotated grid, or the original grid if degrees is not 90, 180, or 270
pub fn apply_rotate(grid: &[String], degrees: u16) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    match degrees {
        180 => {
            // 180 degrees: reverse rows and reverse within each row
            // This is equivalent to mirror_h + mirror_v
            apply_mirror_horizontal(&apply_mirror_vertical(grid))
        }
        90 => {
            // 90 degrees clockwise: columns become rows (bottom to top)
            // Original (0,0) goes to (0, h-1)
            // Original (x,y) goes to (h-1-y, x)
            let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
            let height = parsed.len();
            let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

            if width == 0 {
                return Vec::new();
            }

            // New dimensions: width becomes height, height becomes width
            let mut result: Vec<String> = Vec::with_capacity(width);

            for x in 0..width {
                let mut new_row = String::new();
                // Read from bottom to top for this column
                for y in (0..height).rev() {
                    if let Some(token) = parsed[y].get(x) {
                        new_row.push_str(token);
                    } else {
                        new_row.push_str(TRANSPARENT_TOKEN);
                    }
                }
                result.push(new_row);
            }

            result
        }
        270 => {
            // 270 degrees clockwise (= 90 CCW): columns become rows (top to bottom)
            // Original (x,y) goes to (y, w-1-x)
            let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
            let height = parsed.len();
            let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

            if width == 0 {
                return Vec::new();
            }

            // New dimensions: width becomes height, height becomes width
            let mut result: Vec<String> = Vec::with_capacity(width);

            // Iterate columns right to left
            for x in (0..width).rev() {
                let mut new_row = String::new();
                // Read from top to bottom for this column
                for y in 0..height {
                    if let Some(token) = parsed[y].get(x) {
                        new_row.push_str(token);
                    } else {
                        new_row.push_str(TRANSPARENT_TOKEN);
                    }
                }
                result.push(new_row);
            }

            result
        }
        _ => {
            // For 0 or invalid degrees, return unchanged
            grid.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mirror_h() {
        assert_eq!(parse_transform_str("mirror-h").unwrap(), Transform::MirrorH);
        assert_eq!(
            parse_transform_str("symmetry-h").unwrap(),
            Transform::MirrorH
        );
        assert_eq!(parse_transform_str("flip-h").unwrap(), Transform::MirrorH);
        assert_eq!(parse_transform_str("MIRROR-H").unwrap(), Transform::MirrorH);
    }

    #[test]
    fn test_parse_mirror_v() {
        assert_eq!(parse_transform_str("mirror-v").unwrap(), Transform::MirrorV);
        assert_eq!(
            parse_transform_str("symmetry-v").unwrap(),
            Transform::MirrorV
        );
        assert_eq!(parse_transform_str("flip-v").unwrap(), Transform::MirrorV);
    }

    #[test]
    fn test_parse_rotate() {
        assert_eq!(
            parse_transform_str("rotate:90").unwrap(),
            Transform::Rotate { degrees: 90 }
        );
        assert_eq!(
            parse_transform_str("rotate:180").unwrap(),
            Transform::Rotate { degrees: 180 }
        );
        assert_eq!(
            parse_transform_str("rotate:270").unwrap(),
            Transform::Rotate { degrees: 270 }
        );
        assert_eq!(
            parse_transform_str("rot:90").unwrap(),
            Transform::Rotate { degrees: 90 }
        );
    }

    #[test]
    fn test_parse_rotate_invalid() {
        assert!(parse_transform_str("rotate:45").is_err());
        assert!(parse_transform_str("rotate:360").is_err());
        assert!(parse_transform_str("rotate").is_err());
    }

    #[test]
    fn test_parse_tile() {
        assert_eq!(
            parse_transform_str("tile:3x2").unwrap(),
            Transform::Tile { w: 3, h: 2 }
        );
        assert_eq!(
            parse_transform_str("tile:1x1").unwrap(),
            Transform::Tile { w: 1, h: 1 }
        );
    }

    #[test]
    fn test_parse_pad() {
        assert_eq!(
            parse_transform_str("pad:4").unwrap(),
            Transform::Pad { size: 4 }
        );
        assert_eq!(
            parse_transform_str("pad:0").unwrap(),
            Transform::Pad { size: 0 }
        );
    }

    #[test]
    fn test_parse_crop() {
        assert_eq!(
            parse_transform_str("crop:0,0,8,8").unwrap(),
            Transform::Crop {
                x: 0,
                y: 0,
                w: 8,
                h: 8
            }
        );
        assert_eq!(
            parse_transform_str("crop:4, 4, 16, 16").unwrap(),
            Transform::Crop {
                x: 4,
                y: 4,
                w: 16,
                h: 16
            }
        );
    }

    #[test]
    fn test_parse_outline() {
        assert_eq!(
            parse_transform_str("outline").unwrap(),
            Transform::Outline {
                token: None,
                width: 1
            }
        );
        assert_eq!(
            parse_transform_str("outline:{border}").unwrap(),
            Transform::Outline {
                token: Some("{border}".to_string()),
                width: 1
            }
        );
        assert_eq!(
            parse_transform_str("outline:{border},2").unwrap(),
            Transform::Outline {
                token: Some("{border}".to_string()),
                width: 2
            }
        );
    }

    #[test]
    fn test_parse_shift() {
        assert_eq!(
            parse_transform_str("shift:4,0").unwrap(),
            Transform::Shift { x: 4, y: 0 }
        );
        assert_eq!(
            parse_transform_str("shift:-2,3").unwrap(),
            Transform::Shift { x: -2, y: 3 }
        );
    }

    #[test]
    fn test_parse_shadow() {
        assert_eq!(
            parse_transform_str("shadow:1,1").unwrap(),
            Transform::Shadow {
                x: 1,
                y: 1,
                token: None
            }
        );
        assert_eq!(
            parse_transform_str("shadow:2,2,{shadow}").unwrap(),
            Transform::Shadow {
                x: 2,
                y: 2,
                token: Some("{shadow}".to_string())
            }
        );
    }

    #[test]
    fn test_parse_pingpong() {
        assert_eq!(
            parse_transform_str("pingpong").unwrap(),
            Transform::Pingpong {
                exclude_ends: false
            }
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
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Rotate { degrees: 90 }
        );
    }

    #[test]
    fn test_parse_transform_value_object() {
        let value = serde_json::json!({"op": "tile", "w": 3, "h": 2});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Tile { w: 3, h: 2 }
        );

        let value = serde_json::json!({"op": "outline", "token": "{border}", "width": 2});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Outline {
                token: Some("{border}".to_string()),
                width: 2
            }
        );

        let value = serde_json::json!({"op": "rotate", "degrees": 180});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Rotate { degrees: 180 }
        );
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
        let transform = Transform::Pingpong {
            exclude_ends: false,
        };
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
        assert!(is_animation_transform(&Transform::Pingpong {
            exclude_ends: false
        }));
        assert!(is_animation_transform(&Transform::Reverse));
        assert!(is_animation_transform(&Transform::FrameOffset {
            offset: 1
        }));
        assert!(is_animation_transform(&Transform::Hold {
            frame: 0,
            count: 2
        }));

        assert!(!is_animation_transform(&Transform::MirrorH));
        assert!(!is_animation_transform(&Transform::MirrorV));
        assert!(!is_animation_transform(&Transform::Rotate { degrees: 90 }));
        assert!(!is_animation_transform(&Transform::Tile { w: 2, h: 2 }));
        assert!(!is_animation_transform(&Transform::Pad { size: 4 }));
        assert!(!is_animation_transform(&Transform::Crop {
            x: 0,
            y: 0,
            w: 8,
            h: 8
        }));
        assert!(!is_animation_transform(&Transform::Outline {
            token: None,
            width: 1
        }));
        assert!(!is_animation_transform(&Transform::Shift { x: 1, y: 1 }));
        assert!(!is_animation_transform(&Transform::Shadow {
            x: 1,
            y: 1,
            token: None
        }));
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
        let frames: Vec<String> = vec![
            "frame1".to_string(),
            "frame2".to_string(),
            "frame3".to_string(),
        ];

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
            Transform::SelOut {
                fallback: None,
                mapping: None
            }
        );
        assert_eq!(
            parse_transform_str("selout").unwrap(),
            Transform::SelOut {
                fallback: None,
                mapping: None
            }
        );
    }

    #[test]
    fn test_parse_selout_with_fallback() {
        assert_eq!(
            parse_transform_str("sel-out:{outline}").unwrap(),
            Transform::SelOut {
                fallback: Some("{outline}".to_string()),
                mapping: None
            }
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
        let result = parse_transform_str("dither-gradient:vertical:{sky_light},{sky_dark}").unwrap();
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
            Transform::SelOut {
                fallback: Some("{border}".to_string()),
                mapping: None
            }
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
        assert_eq!(
            parse_token_pair("dark,light"),
            Some(("dark".to_string(), "light".to_string()))
        );

        // With spaces
        assert_eq!(
            parse_token_pair("  {a} , {b}  "),
            Some(("{a}".to_string(), "{b}".to_string()))
        );

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
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Subpixel { x: 0.5, y: 0.25 }
        );
    }

    #[test]
    fn test_parse_subpixel_value_object_alt_keys() {
        // Test alternative key names (subpixel-x, subpixel_x)
        let value1 = serde_json::json!({
            "op": "subpixel",
            "subpixel-x": 0.3,
            "subpixel-y": 0.7
        });
        assert_eq!(
            parse_transform_value(&value1).unwrap(),
            Transform::Subpixel { x: 0.3, y: 0.7 }
        );

        let value2 = serde_json::json!({
            "op": "subpixel",
            "subpixel_x": 0.2,
            "subpixel_y": 0.8
        });
        assert_eq!(
            parse_transform_value(&value2).unwrap(),
            Transform::Subpixel { x: 0.2, y: 0.8 }
        );
    }

    #[test]
    fn test_parse_subpixel_value_object_defaults() {
        // Missing x and y should default to 0.0
        let value = serde_json::json!({
            "op": "subpixel"
        });
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Subpixel { x: 0.0, y: 0.0 }
        );
    }

    #[test]
    fn test_parse_subpixel_value_object_partial() {
        // Only x specified
        let value = serde_json::json!({
            "op": "subpixel",
            "x": 0.5
        });
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Subpixel { x: 0.5, y: 0.0 }
        );
    }

    // ========================================================================
    // Apply Selout Tests (ATF-9 continued)
    // ========================================================================

    #[test]
    fn test_apply_selout_empty_grid() {
        let grid: Vec<String> = vec![];
        let result = apply_selout(&grid, None, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_selout_no_outline_pixels() {
        // All pixels are interior (no adjacent transparency)
        let grid = vec![
            "{a}{a}{a}".to_string(),
            "{a}{a}{a}".to_string(),
            "{a}{a}{a}".to_string(),
        ];
        let result = apply_selout(&grid, None, None);
        assert_eq!(result, grid);
    }

    #[test]
    fn test_apply_selout_single_pixel() {
        // Single pixel with no transparent neighbors is NOT an outline pixel
        // (edges of image don't count as transparent)
        let grid = vec!["{a}".to_string()];
        let result = apply_selout(&grid, Some("{outline}"), None);
        // Not an outline pixel, so it stays unchanged
        assert_eq!(result, vec!["{a}".to_string()]);
    }

    #[test]
    fn test_apply_selout_single_pixel_with_transparent() {
        // Single pixel surrounded by transparent IS an outline pixel
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{a}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let result = apply_selout(&grid, Some("{outline}"), None);
        // The center pixel is an outline pixel with no opaque neighbors
        // So it uses fallback
        assert_eq!(result[1], "{_}{outline}{_}");
    }

    #[test]
    fn test_apply_selout_basic() {
        // A simple 3x3 grid with transparent corners
        // The edge pixels should become outlined based on interior
        let grid = vec![
            "{_}{skin}{_}".to_string(),
            "{skin}{skin}{skin}".to_string(),
            "{_}{skin}{_}".to_string(),
        ];

        let result = apply_selout(&grid, Some("{outline}"), None);

        // Corner transparent pixels stay transparent
        // Edge skin pixels (adjacent to {_}) get transformed
        // Center skin pixel stays {skin} (not adjacent to {_})
        assert_eq!(result[0], "{_}{skin_dark}{_}");
        assert_eq!(result[1], "{skin_dark}{skin}{skin_dark}");
        assert_eq!(result[2], "{_}{skin_dark}{_}");
    }

    #[test]
    fn test_apply_selout_with_mapping() {
        let grid = vec![
            "{_}{skin}{_}".to_string(),
            "{skin}{skin}{skin}".to_string(),
            "{_}{skin}{_}".to_string(),
        ];

        let mut mapping = HashMap::new();
        mapping.insert("{skin}".to_string(), "{skin_shadow}".to_string());

        let result = apply_selout(&grid, None, Some(&mapping));

        // Outline pixels adjacent to {skin} interior should use mapped value
        assert_eq!(result[0], "{_}{skin_shadow}{_}");
        assert_eq!(result[1], "{skin_shadow}{skin}{skin_shadow}");
        assert_eq!(result[2], "{_}{skin_shadow}{_}");
    }

    #[test]
    fn test_apply_selout_with_wildcard() {
        let grid = vec![
            "{_}{a}{_}".to_string(),
            "{a}{b}{a}".to_string(),
            "{_}{a}{_}".to_string(),
        ];

        let mut mapping = HashMap::new();
        mapping.insert("*".to_string(), "{dark}".to_string());

        let result = apply_selout(&grid, None, Some(&mapping));

        // All outline pixels should use wildcard
        assert_eq!(result[0], "{_}{dark}{_}");
        assert_eq!(result[1], "{dark}{b}{dark}");
        assert_eq!(result[2], "{_}{dark}{_}");
    }

    #[test]
    fn test_apply_selout_mixed_colors() {
        // Test with different colors to verify dominant neighbor selection
        // Create a grid where the outline pixels have a clear dominant neighbor
        let grid = vec![
            "{_}{skin}{skin}{_}".to_string(),
            "{skin}{skin}{skin}{skin}".to_string(),
            "{skin}{skin}{skin}{skin}".to_string(),
            "{hair}{hair}{hair}{hair}".to_string(),
            "{_}{hair}{hair}{_}".to_string(),
        ];

        let mut mapping = HashMap::new();
        mapping.insert("{skin}".to_string(), "{skin_dark}".to_string());
        mapping.insert("{hair}".to_string(), "{hair_dark}".to_string());

        let result = apply_selout(&grid, None, Some(&mapping));

        // Top row: skin pixels adjacent to {_} should become skin_dark
        // (dominant neighbor is {skin} from surrounding pixels)
        assert_eq!(result[0], "{_}{skin_dark}{skin_dark}{_}");

        // Bottom row: hair pixels adjacent to {_} should become hair_dark
        // (dominant neighbor is {hair} from surrounding pixels - row 3 and 4)
        assert_eq!(result[4], "{_}{hair_dark}{hair_dark}{_}");
    }

    #[test]
    fn test_apply_selout_auto_dark_suffix() {
        // Without mapping or fallback, should auto-generate {token_dark}
        let grid = vec![
            "{_}{x}{_}".to_string(),
            "{x}{x}{x}".to_string(),
            "{_}{x}{_}".to_string(),
        ];

        let result = apply_selout(&grid, None, None);

        assert_eq!(result[0], "{_}{x_dark}{_}");
        assert_eq!(result[1], "{x_dark}{x}{x_dark}");
        assert_eq!(result[2], "{_}{x_dark}{_}");
    }

    #[test]
    fn test_is_not_animation_transform_selout() {
        assert!(!is_animation_transform(&Transform::SelOut {
            fallback: None,
            mapping: None
        }));
    }

    // ========================================================================
    // Scale Transform Tests
    // ========================================================================

    #[test]
    fn test_parse_scale_string() {
        assert_eq!(
            parse_transform_str("scale:2.0,1.5").unwrap(),
            Transform::Scale { x: 2.0, y: 1.5 }
        );
        assert_eq!(
            parse_transform_str("scale:0.5,0.5").unwrap(),
            Transform::Scale { x: 0.5, y: 0.5 }
        );
    }

    #[test]
    fn test_parse_scale_string_invalid() {
        // Missing parameters
        assert!(parse_transform_str("scale").is_err());

        // Invalid format
        assert!(parse_transform_str("scale:2.0").is_err());

        // Non-numeric
        assert!(parse_transform_str("scale:abc,def").is_err());

        // Negative/zero values
        assert!(parse_transform_str("scale:-1.0,1.0").is_err());
        assert!(parse_transform_str("scale:1.0,0").is_err());
    }

    #[test]
    fn test_parse_scale_object() {
        let value = serde_json::json!({"op": "scale", "x": 2.0, "y": 1.5});
        assert_eq!(
            parse_transform_value(&value).unwrap(),
            Transform::Scale { x: 2.0, y: 1.5 }
        );
    }

    #[test]
    fn test_parse_scale_object_missing_params() {
        // Missing x
        let value = serde_json::json!({"op": "scale", "y": 1.5});
        assert!(parse_transform_value(&value).is_err());

        // Missing y
        let value = serde_json::json!({"op": "scale", "x": 2.0});
        assert!(parse_transform_value(&value).is_err());
    }

    #[test]
    fn test_apply_scale_empty_grid() {
        let grid: Vec<String> = vec![];
        let result = apply_scale(&grid, 2.0, 2.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_scale_identity() {
        // Scale by 1.0 should return same dimensions
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let result = apply_scale(&grid, 1.0, 1.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "{a}{b}");
        assert_eq!(result[1], "{c}{d}");
    }

    #[test]
    fn test_apply_scale_double_horizontal() {
        // Scale 2x horizontally
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let result = apply_scale(&grid, 2.0, 1.0);
        assert_eq!(result.len(), 2);
        // Each column is duplicated
        assert_eq!(result[0], "{a}{a}{b}{b}");
        assert_eq!(result[1], "{c}{c}{d}{d}");
    }

    #[test]
    fn test_apply_scale_double_vertical() {
        // Scale 2x vertically
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let result = apply_scale(&grid, 1.0, 2.0);
        assert_eq!(result.len(), 4);
        // Each row is duplicated
        assert_eq!(result[0], "{a}{b}");
        assert_eq!(result[1], "{a}{b}");
        assert_eq!(result[2], "{c}{d}");
        assert_eq!(result[3], "{c}{d}");
    }

    #[test]
    fn test_apply_scale_double_both() {
        // Scale 2x in both directions
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let result = apply_scale(&grid, 2.0, 2.0);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "{a}{a}{b}{b}");
        assert_eq!(result[1], "{a}{a}{b}{b}");
        assert_eq!(result[2], "{c}{c}{d}{d}");
        assert_eq!(result[3], "{c}{c}{d}{d}");
    }

    #[test]
    fn test_apply_scale_half() {
        // Scale down by half
        let grid = vec![
            "{a}{b}{c}{d}".to_string(),
            "{e}{f}{g}{h}".to_string(),
            "{i}{j}{k}{l}".to_string(),
            "{m}{n}{o}{p}".to_string(),
        ];
        let result = apply_scale(&grid, 0.5, 0.5);
        assert_eq!(result.len(), 2);
        // Should sample every other pixel
        assert_eq!(result[0], "{a}{c}");
        assert_eq!(result[1], "{i}{k}");
    }

    #[test]
    fn test_apply_scale_squash() {
        // Squash effect: wider horizontally, shorter vertically
        let grid = vec![
            "{_}{x}{_}".to_string(),
            "{x}{x}{x}".to_string(),
            "{_}{x}{_}".to_string(),
        ];
        let result = apply_scale(&grid, 1.5, 0.5);
        // Original: 3x3, Result: 5x2 (rounded)
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("{x}") || result[0].contains("{_}"));
    }

    #[test]
    fn test_apply_scale_stretch() {
        // Stretch effect: narrower horizontally, taller vertically
        let grid = vec![
            "{_}{x}{_}".to_string(),
            "{x}{x}{x}".to_string(),
            "{_}{x}{_}".to_string(),
        ];
        let result = apply_scale(&grid, 0.67, 1.5);
        // Original: 3x3, Result: 2x5 (rounded)
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_is_not_animation_transform_scale() {
        assert!(!is_animation_transform(&Transform::Scale { x: 2.0, y: 2.0 }));
    }

    // ============================================================================
    // Effect Transform Tests (TRF-4)
    // ============================================================================

    #[test]
    fn test_apply_outline_basic() {
        // Simple 3x3 grid with center pixel opaque
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{x}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let result = apply_outline(&grid, None, 1);
        // All surrounding transparent pixels should become outline
        assert_eq!(result[0], "{outline}{outline}{outline}");
        assert_eq!(result[1], "{outline}{x}{outline}");
        assert_eq!(result[2], "{outline}{outline}{outline}");
    }

    #[test]
    fn test_apply_outline_custom_token() {
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{x}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let result = apply_outline(&grid, Some("{border}"), 1);
        assert_eq!(result[0], "{border}{border}{border}");
        assert_eq!(result[1], "{border}{x}{border}");
        assert_eq!(result[2], "{border}{border}{border}");
    }

    #[test]
    fn test_apply_outline_width_2() {
        // 5x5 grid with center pixel
        let grid = vec![
            "{_}{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}{_}".to_string(),
            "{_}{_}{x}{_}{_}".to_string(),
            "{_}{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}{_}".to_string(),
        ];
        let result = apply_outline(&grid, None, 2);
        // With width 2, outline should extend further (Manhattan distance <= 2)
        // Row 0: positions within distance 2 from center (2,2)
        // (2,0) is dist 2, so should be outlined
        assert!(result[0].contains("{outline}"));
        assert_eq!(result[2], "{outline}{outline}{x}{outline}{outline}");
    }

    #[test]
    fn test_apply_outline_preserves_opaque() {
        // Multiple opaque pixels
        let grid = vec![
            "{_}{a}{_}".to_string(),
            "{b}{c}{d}".to_string(),
            "{_}{e}{_}".to_string(),
        ];
        let result = apply_outline(&grid, None, 1);
        // Corners should get outlined, center pixels preserved
        assert_eq!(result[0], "{outline}{a}{outline}");
        assert_eq!(result[1], "{b}{c}{d}");
        assert_eq!(result[2], "{outline}{e}{outline}");
    }

    #[test]
    fn test_apply_outline_empty_grid() {
        let grid: Vec<String> = vec![];
        let result = apply_outline(&grid, None, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_outline_width_zero() {
        let grid = vec![
            "{_}{x}{_}".to_string(),
        ];
        let result = apply_outline(&grid, None, 0);
        // Width 0 should return unchanged
        assert_eq!(result[0], "{_}{x}{_}");
    }

    #[test]
    fn test_apply_shift_right() {
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let result = apply_shift(&grid, 1, 0);
        // Shift right by 1: left column becomes transparent
        assert_eq!(result[0], "{_}{a}{b}");
        assert_eq!(result[1], "{_}{d}{e}");
    }

    #[test]
    fn test_apply_shift_left() {
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let result = apply_shift(&grid, -1, 0);
        // Shift left by 1: right column becomes transparent
        assert_eq!(result[0], "{b}{c}{_}");
        assert_eq!(result[1], "{e}{f}{_}");
    }

    #[test]
    fn test_apply_shift_down() {
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
            "{e}{f}".to_string(),
        ];
        let result = apply_shift(&grid, 0, 1);
        // Shift down by 1: top row becomes transparent
        assert_eq!(result[0], "{_}{_}");
        assert_eq!(result[1], "{a}{b}");
        assert_eq!(result[2], "{c}{d}");
    }

    #[test]
    fn test_apply_shift_up() {
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
            "{e}{f}".to_string(),
        ];
        let result = apply_shift(&grid, 0, -1);
        // Shift up by 1: bottom row becomes transparent
        assert_eq!(result[0], "{c}{d}");
        assert_eq!(result[1], "{e}{f}");
        assert_eq!(result[2], "{_}{_}");
    }

    #[test]
    fn test_apply_shift_diagonal() {
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
            "{g}{h}{i}".to_string(),
        ];
        let result = apply_shift(&grid, 1, 1);
        // Shift right 1, down 1
        assert_eq!(result[0], "{_}{_}{_}");
        assert_eq!(result[1], "{_}{a}{b}");
        assert_eq!(result[2], "{_}{d}{e}");
    }

    #[test]
    fn test_apply_shift_empty_grid() {
        let grid: Vec<String> = vec![];
        let result = apply_shift(&grid, 1, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_shadow_basic() {
        // Small sprite, shadow offset (1,1)
        let grid = vec![
            "{_}{_}{_}{_}".to_string(),
            "{_}{x}{x}{_}".to_string(),
            "{_}{x}{x}{_}".to_string(),
            "{_}{_}{_}{_}".to_string(),
        ];
        let result = apply_shadow(&grid, 1, 1, None);
        // Original pixels preserved
        assert!(result[1].contains("{x}"));
        assert!(result[2].contains("{x}"));
        // Shadow should appear at offset
        assert!(result[2].contains("{shadow}") || result[3].contains("{shadow}"));
    }

    #[test]
    fn test_apply_shadow_custom_token() {
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{x}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let result = apply_shadow(&grid, 1, 1, Some("{dark}"));
        // Shadow at (2,2) should be {dark}
        assert_eq!(result[2], "{_}{_}{dark}");
    }

    #[test]
    fn test_apply_shadow_negative_offset() {
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{x}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let result = apply_shadow(&grid, -1, -1, None);
        // Shadow should appear at (0,0)
        assert_eq!(result[0], "{shadow}{_}{_}");
    }

    #[test]
    fn test_apply_shadow_overlapping() {
        // Shadow overlaps with original - original should win
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{a}{b}".to_string(),
            "{_}{c}{d}".to_string(),
        ];
        let result = apply_shadow(&grid, 1, 0, None);
        // Rightmost pixels would be shadowed but original is there
        assert_eq!(result[1], "{_}{a}{b}"); // {b} preserved, not shadowed
        assert_eq!(result[2], "{_}{c}{d}"); // {d} preserved, not shadowed
        // But check for shadow where original is transparent
        // At (0,1) we're transparent, shadow from (-1,1) - which is out of bounds - no shadow
        // But {a} at (1,1) casts shadow to (2,1) which has {b} - preserved
    }

    #[test]
    fn test_apply_shadow_empty_grid() {
        let grid: Vec<String> = vec![];
        let result = apply_shadow(&grid, 1, 1, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_shadow_preserves_original() {
        // Ensure original opaque pixels are never replaced by shadow
        let grid = vec![
            "{a}{_}".to_string(),
            "{_}{b}".to_string(),
        ];
        let result = apply_shadow(&grid, 1, 1, None);
        // {a} at (0,0) - original preserved
        // {b} at (1,1) - original preserved
        // Shadow of {a} would be at (1,1) but {b} is there
        assert!(result[0].starts_with("{a}"));
        assert!(result[1].ends_with("{b}"));
    }

    // ========================================================================
    // Geometric Transform Tests
    // ========================================================================

    #[test]
    fn test_apply_mirror_horizontal_basic() {
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let result = apply_mirror_horizontal(&grid);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "{c}{b}{a}");
        assert_eq!(result[1], "{f}{e}{d}");
    }

    #[test]
    fn test_apply_mirror_horizontal_empty() {
        let grid: Vec<String> = vec![];
        let result = apply_mirror_horizontal(&grid);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_mirror_horizontal_single_row() {
        let grid = vec!["{1}{2}{3}{4}".to_string()];
        let result = apply_mirror_horizontal(&grid);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "{4}{3}{2}{1}");
    }

    #[test]
    fn test_apply_mirror_horizontal_single_column() {
        let grid = vec![
            "{a}".to_string(),
            "{b}".to_string(),
            "{c}".to_string(),
        ];
        let result = apply_mirror_horizontal(&grid);
        assert_eq!(result.len(), 3);
        // Single column should be unchanged
        assert_eq!(result[0], "{a}");
        assert_eq!(result[1], "{b}");
        assert_eq!(result[2], "{c}");
    }

    #[test]
    fn test_apply_mirror_vertical_basic() {
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
            "{e}{f}".to_string(),
        ];
        let result = apply_mirror_vertical(&grid);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "{e}{f}");
        assert_eq!(result[1], "{c}{d}");
        assert_eq!(result[2], "{a}{b}");
    }

    #[test]
    fn test_apply_mirror_vertical_empty() {
        let grid: Vec<String> = vec![];
        let result = apply_mirror_vertical(&grid);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_mirror_vertical_single_row() {
        let grid = vec!["{x}{y}{z}".to_string()];
        let result = apply_mirror_vertical(&grid);
        assert_eq!(result.len(), 1);
        // Single row should be unchanged
        assert_eq!(result[0], "{x}{y}{z}");
    }

    #[test]
    fn test_apply_rotate_90_square() {
        // 2x2 grid rotated 90 degrees clockwise
        // Original:    Rotated 90:
        // a b          c a
        // c d          d b
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let result = apply_rotate(&grid, 90);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "{c}{a}");
        assert_eq!(result[1], "{d}{b}");
    }

    #[test]
    fn test_apply_rotate_90_rectangular() {
        // 2x3 grid (2 columns, 3 rows) rotated 90 degrees clockwise
        // Original:    Rotated 90 (3 columns, 2 rows):
        // a b          e c a
        // c d          f d b
        // e f
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
            "{e}{f}".to_string(),
        ];
        let result = apply_rotate(&grid, 90);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "{e}{c}{a}");
        assert_eq!(result[1], "{f}{d}{b}");
    }

    #[test]
    fn test_apply_rotate_180() {
        // 180 degree rotation (same as mirror_h + mirror_v)
        // Original:    Rotated 180:
        // a b c        i h g
        // d e f        f e d
        // g h i        c b a
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
            "{g}{h}{i}".to_string(),
        ];
        let result = apply_rotate(&grid, 180);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "{i}{h}{g}");
        assert_eq!(result[1], "{f}{e}{d}");
        assert_eq!(result[2], "{c}{b}{a}");
    }

    #[test]
    fn test_apply_rotate_270() {
        // 270 degrees clockwise (= 90 counter-clockwise)
        // Original:    Rotated 270:
        // a b          b d
        // c d          a c
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let result = apply_rotate(&grid, 270);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "{b}{d}");
        assert_eq!(result[1], "{a}{c}");
    }

    #[test]
    fn test_apply_rotate_270_rectangular() {
        // 3x2 grid (3 columns, 2 rows) rotated 270 degrees clockwise
        // Original:    Rotated 270 (2 columns, 3 rows):
        // a b c        c f
        // d e f        b e
        //              a d
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let result = apply_rotate(&grid, 270);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "{c}{f}");
        assert_eq!(result[1], "{b}{e}");
        assert_eq!(result[2], "{a}{d}");
    }

    #[test]
    fn test_apply_rotate_empty() {
        let grid: Vec<String> = vec![];
        let result = apply_rotate(&grid, 90);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_rotate_invalid_degrees() {
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        // 0 degrees should return unchanged
        let result = apply_rotate(&grid, 0);
        assert_eq!(result, grid);

        // 45 degrees (invalid) should return unchanged
        let result = apply_rotate(&grid, 45);
        assert_eq!(result, grid);
    }

    #[test]
    fn test_rotate_90_then_90_equals_180() {
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
        ];
        let rotated_twice = apply_rotate(&apply_rotate(&grid, 90), 90);
        let rotated_180 = apply_rotate(&grid, 180);
        assert_eq!(rotated_twice, rotated_180);
    }

    #[test]
    fn test_rotate_four_times_identity() {
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let rotated = apply_rotate(&apply_rotate(&apply_rotate(&apply_rotate(&grid, 90), 90), 90), 90);
        assert_eq!(rotated, grid);
    }

    #[test]
    fn test_mirror_h_twice_identity() {
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let mirrored = apply_mirror_horizontal(&apply_mirror_horizontal(&grid));
        assert_eq!(mirrored, grid);
    }

    #[test]
    fn test_mirror_v_twice_identity() {
        let grid = vec![
            "{a}{b}".to_string(),
            "{c}{d}".to_string(),
            "{e}{f}".to_string(),
        ];
        let mirrored = apply_mirror_vertical(&apply_mirror_vertical(&grid));
        assert_eq!(mirrored, grid);
    }

    #[test]
    fn test_rotate_with_transparent() {
        // Test that transparent tokens are handled correctly during rotation
        let grid = vec![
            "{a}{_}".to_string(),
            "{_}{b}".to_string(),
        ];
        let result = apply_rotate(&grid, 90);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "{_}{a}");
        assert_eq!(result[1], "{b}{_}");
    }
}
