//! Transform operations for sprites and animations
//!
//! Supports both CLI transforms (`pxl transform`) and format attributes
//! (`"transform": ["mirror-h", "rotate:90"]`).

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

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
    // Selective Outline (sel-out) Tests
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
}
