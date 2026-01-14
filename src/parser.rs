//! JSONL stream parsing for TTP objects

use crate::models::{TtpObject, Warning};
use std::io::BufRead;

/// Error type for parsing failures.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Result of parsing a JSONL stream.
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub objects: Vec<TtpObject>,
    pub warnings: Vec<Warning>,
}

/// Parse a single line into a TtpObject.
///
/// Returns `Ok(TtpObject)` on success, or `Err(ParseError)` if parsing fails.
/// This function does not handle blank lines - the caller should filter those.
pub fn parse_line(line: &str, line_number: usize) -> Result<TtpObject, ParseError> {
    serde_json::from_str(line).map_err(|e| ParseError {
        message: e.to_string(),
        line: line_number,
    })
}

/// Parse a JSONL stream into TTP objects.
///
/// - Skips blank lines
/// - Collects warnings in lenient mode
/// - Returns all successfully parsed objects
pub fn parse_stream<R: BufRead>(reader: R) -> ParseResult {
    let mut result = ParseResult::default();

    for (line_number, line_result) in reader.lines().enumerate() {
        let line_number = line_number + 1; // 1-indexed

        let line = match line_result {
            Ok(line) => line,
            Err(e) => {
                result.warnings.push(Warning {
                    message: format!("IO error reading line: {}", e),
                    line: line_number,
                });
                continue;
            }
        };

        // Skip blank lines
        if line.trim().is_empty() {
            continue;
        }

        match parse_line(&line, line_number) {
            Ok(obj) => result.objects.push(obj),
            Err(e) => {
                result.warnings.push(Warning {
                    message: e.message,
                    line: line_number,
                });
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaletteRef;
    use std::io::Cursor;

    #[test]
    fn test_parse_line_palette() {
        let line = r##"{"type": "palette", "name": "mono", "colors": {"{on}": "#FFFFFF"}}"##;
        let result = parse_line(line, 1).unwrap();
        match result {
            TtpObject::Palette(p) => {
                assert_eq!(p.name, "mono");
                assert_eq!(p.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
            }
            _ => panic!("Expected palette"),
        }
    }

    #[test]
    fn test_parse_line_sprite() {
        let line = r#"{"type": "sprite", "name": "dot", "palette": "colors", "grid": ["{x}"]}"#;
        let result = parse_line(line, 1).unwrap();
        match result {
            TtpObject::Sprite(s) => {
                assert_eq!(s.name, "dot");
                assert!(matches!(s.palette, PaletteRef::Named(ref n) if n == "colors"));
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_parse_line_invalid_json() {
        let line = "{not valid json}";
        let result = parse_line(line, 5);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 5);
    }

    #[test]
    fn test_parse_line_missing_type() {
        let line = r#"{"name": "test", "grid": []}"#;
        let result = parse_line(line, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_stream_simple() {
        let input = r##"{"type": "palette", "name": "mono", "colors": {"{on}": "#FFFFFF"}}
{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 2);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_parse_stream_skips_blank_lines() {
        let input = r##"{"type": "palette", "name": "mono", "colors": {"{on}": "#FFFFFF"}}

{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}

"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 2);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_parse_stream_collects_warnings() {
        let input = r##"{"type": "palette", "name": "mono", "colors": {"{on}": "#FFFFFF"}}
{invalid json}
{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 2);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].line, 2);
    }

    #[test]
    fn test_parse_valid_fixtures() {
        use std::fs;
        use std::path::Path;

        let fixtures_dir = Path::new("tests/fixtures/valid");
        if !fixtures_dir.exists() {
            return; // Skip if fixtures not available
        }

        for entry in fs::read_dir(fixtures_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "jsonl") {
                let file = fs::File::open(&path).unwrap();
                let reader = std::io::BufReader::new(file);
                let result = parse_stream(reader);
                assert!(
                    !result.objects.is_empty(),
                    "Expected objects in {:?}",
                    path
                );
                assert!(
                    result.warnings.is_empty(),
                    "Unexpected warnings in {:?}: {:?}",
                    path,
                    result.warnings
                );
            }
        }
    }

    #[test]
    fn test_parse_invalid_fixtures() {
        use std::fs;
        use std::path::Path;

        let fixtures_dir = Path::new("tests/fixtures/invalid");
        if !fixtures_dir.exists() {
            return; // Skip if fixtures not available
        }

        // Files with semantic errors (valid JSON but invalid semantics)
        // These parse successfully but fail during later validation stages
        let semantic_error_files = ["unknown_palette_ref.jsonl", "invalid_color.jsonl"];

        for entry in fs::read_dir(fixtures_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "jsonl") {
                let filename = path.file_name().unwrap().to_str().unwrap();

                // Skip semantic error files - they parse successfully
                if semantic_error_files.contains(&filename) {
                    continue;
                }

                let file = fs::File::open(&path).unwrap();
                let reader = std::io::BufReader::new(file);
                let result = parse_stream(reader);
                // In lenient mode, invalid lines produce warnings
                // Some may still produce objects if only some lines are invalid
                assert!(
                    !result.warnings.is_empty() || result.objects.is_empty(),
                    "Expected warnings or no objects in {:?}",
                    path
                );
            }
        }
    }
}
