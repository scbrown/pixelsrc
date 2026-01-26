//! Transform parsing from strings and JSON values
//!
//! Provides functions to parse transform specifications from:
//! - String syntax: `"mirror-h"`, `"rotate:90"`, `"tile:3x2"`
//! - JSON objects: `{"op": "tile", "w": 3, "h": 2}`

use serde_json::Value;
use std::collections::HashMap;

use super::dither::{DitherPattern, GradientDirection};
use super::types::{Transform, TransformError};

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
pub fn parse_token_pair(s: &str) -> Option<(String, String)> {
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
        assert_eq!(parse_transform_str("skew-x:20").unwrap(), Transform::SkewX { degrees: 20.0 });
        assert_eq!(parse_transform_str("skewx:45deg").unwrap(), Transform::SkewX { degrees: 45.0 });
        assert_eq!(parse_transform_str("skew-x:-30").unwrap(), Transform::SkewX { degrees: -30.0 });
    }

    #[test]
    fn test_parse_skew_y() {
        assert_eq!(parse_transform_str("skew-y:15").unwrap(), Transform::SkewY { degrees: 15.0 });
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
}
