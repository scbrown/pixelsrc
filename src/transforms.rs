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
                write!(f, "invalid rotation degrees: {} (must be 90, 180, or 270)", degrees)
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
    Rotate { degrees: u16 }, // 90, 180, 270

    // Expansion
    Tile { w: u32, h: u32 },
    Pad { size: u32 },
    Crop { x: u32, y: u32, w: u32, h: u32 },

    // Effects
    Outline { token: Option<String>, width: u32 },
    Shift { x: i32, y: i32 },
    Shadow { x: i32, y: i32, token: Option<String> },

    // Animation (only valid for Animation type)
    Pingpong { exclude_ends: bool },
    Reverse,
    FrameOffset { offset: i32 },
    Hold { frame: usize, count: usize },
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
            let token = params.get("token").and_then(|v| v.as_str()).map(String::from);
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
            let token = params.get("token").and_then(|v| v.as_str()).map(String::from);
            Ok(Transform::Shadow { x, y, token })
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
            let width = parts[1]
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
            Transform::Shadow { x: 1, y: 1, token: None }
        );
        assert_eq!(
            parse_transform_str("shadow:2,2,{shadow}").unwrap(),
            Transform::Shadow { x: 2, y: 2, token: Some("{shadow}".to_string()) }
        );
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
            Transform::Outline { token: Some("{border}".to_string()), width: 2 }
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
}
