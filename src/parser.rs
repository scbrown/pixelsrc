//! Streaming JSON5 parsing for Pixelsrc objects
//!
//! Supports both single-line JSONL and multi-line JSON5 formats.
//! JSON5 adds support for comments, trailing commas, and unquoted keys.

use crate::models::{TtpObject, Warning};
use std::io::Read;
use thiserror::Error;

/// Error type for parsing failures.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("line {line}: {message}")]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

/// Result of parsing a JSON stream.
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub objects: Vec<TtpObject>,
    pub warnings: Vec<Warning>,
}

/// Parse a single JSON5 string into a TtpObject.
///
/// Returns `Ok(TtpObject)` on success, or `Err(ParseError)` if parsing fails.
/// Supports JSON5 features: comments, trailing commas, and unquoted keys.
pub fn parse_line(line: &str, line_number: usize) -> Result<TtpObject, ParseError> {
    json5::from_str(line).map_err(|e| ParseError { message: e.to_string(), line: line_number })
}

/// Parse a stream of JSON5 objects into Pixelsrc objects.
///
/// Supports both formats:
/// - Single-line JSONL (one JSON5 object per line)
/// - Multi-line JSON5 (objects can span multiple lines, separated by whitespace)
///
/// JSON5 features supported:
/// - Comments (// single-line and /* multi-line */)
/// - Trailing commas in arrays and objects
/// - Unquoted object keys
///
/// Collects warnings for malformed objects and continues parsing.
pub fn parse_stream<R: Read>(reader: R) -> ParseResult {
    use std::io::BufRead;

    let mut result = ParseResult::default();
    let buf_reader = std::io::BufReader::new(reader);
    let mut lines = buf_reader.lines();

    let mut accumulator = String::new();
    let mut start_line = 1;
    let mut current_line = 1;
    let mut brace_depth = 0;
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(Ok(line)) = lines.next() {
        // Skip empty lines when not accumulating
        if accumulator.is_empty() && line.trim().is_empty() {
            current_line += 1;
            continue;
        }

        // Add line to accumulator
        if !accumulator.is_empty() {
            accumulator.push('\n');
        }
        accumulator.push_str(&line);

        // Track brace/bracket depth to detect complete objects
        for ch in line.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string => brace_depth -= 1,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string => bracket_depth -= 1,
                _ => {}
            }
        }

        // Try to parse when braces are balanced
        if brace_depth == 0 && bracket_depth == 0 && !accumulator.trim().is_empty() {
            match json5::from_str::<TtpObject>(&accumulator) {
                Ok(obj) => result.objects.push(obj),
                Err(e) => {
                    result.warnings.push(Warning {
                        message: e.to_string(),
                        line: start_line,
                    });
                    // Stop parsing after error - can't reliably find next object boundary
                    return result;
                }
            }

            accumulator.clear();
            start_line = current_line + 1;
            in_string = false;
            escape_next = false;
        }

        current_line += 1;
    }

    // Handle any remaining accumulated content
    if !accumulator.trim().is_empty() {
        match json5::from_str::<TtpObject>(&accumulator) {
            Ok(obj) => result.objects.push(obj),
            Err(e) => {
                result.warnings.push(Warning {
                    message: e.to_string(),
                    line: start_line,
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
    use serial_test::serial;
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
        // With streaming JSON parser, syntax errors stop parsing
        // (can't recover since we don't know where next object starts)
        let input = r##"{"type": "palette", "name": "mono", "colors": {"{on}": "#FFFFFF"}}
{invalid json}
{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}"##;
        let result = parse_stream(Cursor::new(input));
        // First object parses successfully, then we hit the error
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].line, 2);
    }

    #[test]
    fn test_parse_stream_multiline_json() {
        // Multi-line JSON objects should parse correctly
        let input = r##"{
  "type": "palette",
  "name": "colors",
  "colors": {
    "{_}": "#00000000",
    "{a}": "#FF0000"
  }
}
{
  "type": "sprite",
  "name": "test",
  "palette": "colors",
  "grid": [
    "{_}{a}{a}{_}",
    "{a}{a}{a}{a}"
  ]
}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 2);
        assert!(result.warnings.is_empty());

        // Verify first is palette
        match &result.objects[0] {
            TtpObject::Palette(p) => {
                assert_eq!(p.name, "colors");
                assert_eq!(p.colors.len(), 2);
            }
            _ => panic!("Expected palette"),
        }

        // Verify second is sprite with multi-line grid
        match &result.objects[1] {
            TtpObject::Sprite(s) => {
                assert_eq!(s.name, "test");
            }
            _ => panic!("Expected sprite"),
        }
    }

    #[test]
    fn test_parse_stream_mixed_single_and_multiline() {
        // Mix of single-line and multi-line objects
        let input = r##"{"type": "palette", "name": "p1", "colors": {"{x}": "#FF0000"}}
{
  "type": "sprite",
  "name": "s1",
  "palette": "p1",
  "grid": ["{x}"]
}
{"type": "palette", "name": "p2", "colors": {"{y}": "#00FF00"}}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 3);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_parse_stream_whitespace_between_objects() {
        // Objects separated by various whitespace
        let input = r#"{"type": "palette", "name": "p1", "colors": {}}


{"type": "palette", "name": "p2", "colors": {}}

{"type": "palette", "name": "p3", "colors": {}}"#;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 3);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_parse_json5_comments() {
        let input = r##"{
  // Single-line comment
  type: "palette",
  name: "test", /* inline comment */
  colors: {
    "{x}": "#FF0000" // trailing comment
  }
}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 1);
        assert!(result.warnings.is_empty());
        match &result.objects[0] {
            TtpObject::Palette(p) => {
                assert_eq!(p.name, "test");
                assert_eq!(p.colors.get("{x}"), Some(&"#FF0000".to_string()));
            }
            _ => panic!("Expected palette"),
        }
    }

    #[test]
    fn test_parse_json5_trailing_commas() {
        let input = r##"{
  type: "palette",
  name: "colors",
  colors: {
    "{_}": "#0000",
    "{a}": "#FF00",
  },
}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 1);
        assert!(result.warnings.is_empty());
        match &result.objects[0] {
            TtpObject::Palette(p) => {
                assert_eq!(p.name, "colors");
                assert_eq!(p.colors.len(), 2);
            }
            _ => panic!("Expected palette"),
        }
    }

    #[test]
    fn test_parse_json5_unquoted_keys() {
        let input = r##"{type: "palette", name: "test", colors: {"{_}": "#0000"}}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 1);
        assert!(result.warnings.is_empty());
        match &result.objects[0] {
            TtpObject::Palette(p) => {
                assert_eq!(p.name, "test");
                assert_eq!(p.colors.get("{_}"), Some(&"#0000".to_string()));
            }
            _ => panic!("Expected palette"),
        }
    }

    #[test]
    fn test_parse_json5_all_features() {
        // Test all JSON5 features together
        let input = r##"{
  // This is a palette with JSON5 features
  type: "palette",
  name: "test",
  colors: {
    "{_}": "#0000", // transparent
    "{x}": "#FF00", // red
  }, // trailing comma after colors object
}"##;
        let result = parse_stream(Cursor::new(input));
        assert_eq!(result.objects.len(), 1);
        assert!(result.warnings.is_empty());
        match &result.objects[0] {
            TtpObject::Palette(p) => {
                assert_eq!(p.name, "test");
                assert_eq!(p.colors.len(), 2);
            }
            _ => panic!("Expected palette"),
        }
    }

    #[test]
    #[serial]
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
            // Support both .jsonl and .pxl extensions
            let is_pixelsrc = path.extension().is_some_and(|e| e == "jsonl" || e == "pxl");
            if is_pixelsrc {
                let file = fs::File::open(&path).unwrap();
                let reader = std::io::BufReader::new(file);
                let result = parse_stream(reader);
                assert!(!result.objects.is_empty(), "Expected objects in {:?}", path);
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
    #[serial]
    fn test_parse_invalid_fixtures() {
        use std::fs;
        use std::path::Path;

        let fixtures_dir = Path::new("tests/fixtures/invalid");
        if !fixtures_dir.exists() {
            return; // Skip if fixtures not available
        }

        // Files with semantic errors (valid JSON but invalid semantics)
        // These parse successfully but fail during later validation stages
        let semantic_error_files = [
            "unknown_palette_ref.jsonl",
            "invalid_color.jsonl",
            "validate_errors.jsonl",
            "validate_typo.jsonl",
        ];

        for entry in fs::read_dir(fixtures_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            // Support both .pxl and .jsonl extensions
            if path.extension().is_some_and(|e| e == "jsonl" || e == "pxl") {
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
