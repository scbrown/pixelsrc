//! SVG-lite path parser for basic path commands
//!
//! Supports a subset of SVG path syntax:
//! - M (moveto)
//! - L (lineto)
//! - H (horizontal lineto)
//! - V (vertical lineto)
//! - Z (closepath)
//!
//! Each command supports both absolute (uppercase) and relative (lowercase) variants.

use thiserror::Error;

/// Error type for path parsing failures
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum PathError {
    /// Path string is empty
    #[error("empty path string")]
    Empty,
    /// Unknown command character
    #[error("unknown command '{0}'")]
    UnknownCommand(char),
    /// Not enough coordinates for command
    #[error("not enough coordinates for command '{0}'")]
    NotEnoughCoordinates(char),
    /// Invalid number format
    #[error("invalid number '{0}': {1}")]
    InvalidNumber(String, String),
    /// Missing initial moveto command
    #[error("path must start with M or m command")]
    MissingMoveto,
}

/// Parse an SVG-lite path string into polygon vertices.
///
/// # Supported Commands
///
/// - `M x,y` - Move to absolute position
/// - `m dx,dy` - Move to relative position
/// - `L x,y` - Line to absolute position
/// - `l dx,dy` - Line to relative position
/// - `H x` - Horizontal line to absolute x
/// - `h dx` - Horizontal line to relative x
/// - `V y` - Vertical line to absolute y
/// - `v dy` - Vertical line to relative y
/// - `Z` or `z` - Close path (line back to start)
///
/// # Examples
///
/// ```
/// use pixelsrc::path::parse_path;
///
/// // Rectangle
/// let vertices = parse_path("M0,0 L5,0 L5,5 L0,5 Z").unwrap();
/// assert_eq!(vertices, vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 5.0]]);
///
/// // Triangle with relative coordinates
/// let vertices = parse_path("M0,0 l10,0 l-5,10 Z").unwrap();
/// assert_eq!(vertices, vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]]);
/// ```
///
/// # Errors
///
/// Returns `PathError` if the path is invalid or malformed.
pub fn parse_path(path: &str) -> Result<Vec<[f32; 2]>, PathError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(PathError::Empty);
    }

    let mut vertices = Vec::new();
    let mut current_pos = [0.0, 0.0];
    let mut start_pos = [0.0, 0.0];
    let mut tokens = tokenize_path(path);
    let mut has_moveto = false;

    while !tokens.is_empty() {
        let cmd = tokens.remove(0);

        // First command must be M or m
        if !has_moveto && cmd != "M" && cmd != "m" {
            return Err(PathError::MissingMoveto);
        }

        match cmd.as_str() {
            "M" => {
                // Absolute moveto
                let (x, y) = parse_coordinate_pair(&mut tokens, &cmd)?;
                current_pos = [x, y];
                start_pos = current_pos;
                vertices.push(current_pos);
                has_moveto = true;
            }
            "m" => {
                // Relative moveto
                let (dx, dy) = parse_coordinate_pair(&mut tokens, &cmd)?;
                if has_moveto {
                    current_pos[0] += dx;
                    current_pos[1] += dy;
                } else {
                    // First moveto is treated as absolute
                    current_pos = [dx, dy];
                }
                start_pos = current_pos;
                vertices.push(current_pos);
                has_moveto = true;
            }
            "L" => {
                // Absolute lineto
                let (x, y) = parse_coordinate_pair(&mut tokens, &cmd)?;
                current_pos = [x, y];
                vertices.push(current_pos);
            }
            "l" => {
                // Relative lineto
                let (dx, dy) = parse_coordinate_pair(&mut tokens, &cmd)?;
                current_pos[0] += dx;
                current_pos[1] += dy;
                vertices.push(current_pos);
            }
            "H" => {
                // Absolute horizontal
                let x = parse_single_coordinate(&mut tokens, &cmd)?;
                current_pos[0] = x;
                vertices.push(current_pos);
            }
            "h" => {
                // Relative horizontal
                let dx = parse_single_coordinate(&mut tokens, &cmd)?;
                current_pos[0] += dx;
                vertices.push(current_pos);
            }
            "V" => {
                // Absolute vertical
                let y = parse_single_coordinate(&mut tokens, &cmd)?;
                current_pos[1] = y;
                vertices.push(current_pos);
            }
            "v" => {
                // Relative vertical
                let dy = parse_single_coordinate(&mut tokens, &cmd)?;
                current_pos[1] += dy;
                vertices.push(current_pos);
            }
            "Z" | "z" => {
                // Close path - no coordinates needed
                // Don't add start_pos again if we're already there
                if current_pos != start_pos {
                    current_pos = start_pos;
                    // Z doesn't add a vertex, it just closes to start
                }
            }
            _ => {
                return Err(PathError::UnknownCommand(cmd.chars().next().unwrap_or('?')));
            }
        }
    }

    Ok(vertices)
}

/// Tokenize a path string into command and number tokens
fn tokenize_path(path: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in path.chars() {
        match ch {
            // Command characters
            'M' | 'm' | 'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'Z' | 'z' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            // Separators
            ',' | ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            // Number characters (including negative sign and decimal point)
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Parse a coordinate pair (x, y) from tokens
fn parse_coordinate_pair(tokens: &mut Vec<String>, cmd: &str) -> Result<(f32, f32), PathError> {
    if tokens.len() < 2 {
        return Err(PathError::NotEnoughCoordinates(cmd.chars().next().expect("cmd is a non-empty path command")));
    }

    let x_str = tokens.remove(0);
    let y_str = tokens.remove(0);

    let x =
        x_str.parse::<f32>().map_err(|e| PathError::InvalidNumber(x_str.clone(), e.to_string()))?;
    let y =
        y_str.parse::<f32>().map_err(|e| PathError::InvalidNumber(y_str.clone(), e.to_string()))?;

    Ok((x, y))
}

/// Parse a single coordinate from tokens
fn parse_single_coordinate(tokens: &mut Vec<String>, cmd: &str) -> Result<f32, PathError> {
    if tokens.is_empty() {
        return Err(PathError::NotEnoughCoordinates(cmd.chars().next().expect("cmd is a non-empty path command")));
    }

    let num_str = tokens.remove(0);
    num_str.parse::<f32>().map_err(|e| PathError::InvalidNumber(num_str.clone(), e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path_rectangle() {
        let vertices = parse_path("M0,0 L5,0 L5,5 L0,5 Z").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 5.0]]);
    }

    #[test]
    fn test_parse_path_triangle() {
        let vertices = parse_path("M0,0 L10,0 L5,10 Z").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]]);
    }

    #[test]
    fn test_parse_path_relative_lineto() {
        let vertices = parse_path("M0,0 l10,0 l0,10 l-10,0 Z").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]);
    }

    #[test]
    fn test_parse_path_horizontal_vertical() {
        let vertices = parse_path("M0,0 H5 V5 H0 V0").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 5.0], [0.0, 0.0]]);
    }

    #[test]
    fn test_parse_path_relative_horizontal_vertical() {
        let vertices = parse_path("M0,0 h5 v5 h-5 v-5").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 5.0], [0.0, 0.0]]);
    }

    #[test]
    fn test_parse_path_relative_moveto() {
        let vertices = parse_path("M10,10 m5,5 L20,20").unwrap();
        assert_eq!(vertices, vec![[10.0, 10.0], [15.0, 15.0], [20.0, 20.0]]);
    }

    #[test]
    fn test_parse_path_first_moveto_relative_treated_absolute() {
        let vertices = parse_path("m5,5 L10,10").unwrap();
        assert_eq!(vertices, vec![[5.0, 5.0], [10.0, 10.0]]);
    }

    #[test]
    fn test_parse_path_decimal_coordinates() {
        let vertices = parse_path("M0.5,0.5 L5.5,0.5 L5.5,5.5 Z").unwrap();
        assert_eq!(vertices, vec![[0.5, 0.5], [5.5, 0.5], [5.5, 5.5]]);
    }

    #[test]
    fn test_parse_path_negative_coordinates() {
        let vertices = parse_path("M0,0 L-5,0 L-5,-5 Z").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [-5.0, 0.0], [-5.0, -5.0]]);
    }

    #[test]
    fn test_parse_path_empty() {
        let result = parse_path("");
        assert!(matches!(result, Err(PathError::Empty)));
    }

    #[test]
    fn test_parse_path_missing_moveto() {
        let result = parse_path("L10,10");
        assert!(matches!(result, Err(PathError::MissingMoveto)));
    }

    #[test]
    fn test_parse_path_unknown_command() {
        let result = parse_path("M0,0 Q10,10");
        assert!(matches!(result, Err(PathError::UnknownCommand('Q'))));
    }

    #[test]
    fn test_parse_path_not_enough_coordinates() {
        let result = parse_path("M0,0 L5");
        assert!(matches!(result, Err(PathError::NotEnoughCoordinates('L'))));
    }

    #[test]
    fn test_parse_path_invalid_number() {
        let result = parse_path("M0,0 Labc,5");
        assert!(matches!(result, Err(PathError::InvalidNumber(_, _))));
    }

    #[test]
    fn test_parse_path_whitespace_variations() {
        // Test various whitespace formats
        let v1 = parse_path("M0,0 L5,0 L5,5 Z").unwrap();
        let v2 = parse_path("M0 0 L5 0 L5 5 Z").unwrap();
        let v3 = parse_path("M 0 , 0 L 5 , 0 L 5 , 5 Z").unwrap();
        assert_eq!(v1, v2);
        assert_eq!(v1, v3);
    }

    #[test]
    fn test_parse_path_lowercase_z() {
        let vertices = parse_path("M0,0 L5,0 L5,5 z").unwrap();
        assert_eq!(vertices, vec![[0.0, 0.0], [5.0, 0.0], [5.0, 5.0]]);
    }
}
