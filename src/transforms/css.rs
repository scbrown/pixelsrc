//! CSS transform parsing (CSS-14)
//!
//! Parses CSS-style transform strings like `translate(10, 5) rotate(90deg) scale(2)`.

use super::types::Transform;

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
#[non_exhaustive]
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
    let angle_str =
        if args.to_lowercase().ends_with("deg") { &args[..args.len() - 3] } else { args };

    let degrees =
        angle_str.trim().parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
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

        let y_val =
            y_str.trim().parse::<f64>().map_err(|_| CssTransformError::InvalidParameter {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
