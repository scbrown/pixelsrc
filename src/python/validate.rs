//! Validation functions for the Python API.
//!
//! Exposes `.pxl` content validation for CI linting:
//! - `validate()` -- validate a PXL string, return list of messages
//! - `validate_file()` -- validate a file on disk, return list of messages

use std::io::{BufRead, BufReader, Cursor};

use pyo3::prelude::*;

use crate::validate::Validator;

/// Format a validation issue as a human-readable string.
fn format_issue(issue: &crate::validate::ValidationIssue) -> String {
    let mut msg = format!("line {}: {}: {}", issue.line, issue.severity, issue.message);
    if let Some(ref suggestion) = issue.suggestion {
        msg.push_str(&format!(" ({})", suggestion));
    }
    msg
}

/// Validate a `.pxl` string and return a list of warning/error messages.
///
/// Each message is a human-readable string like:
///   "line 3: ERROR: Invalid color \"#GGG\" for token x: ..."
///   "line 5: WARNING: Undefined token y"
///
/// An empty list means the input is valid.
#[pyfunction]
pub fn validate(pxl: &str) -> Vec<String> {
    let mut validator = Validator::new();
    let reader = BufReader::new(Cursor::new(pxl));

    let mut accumulator = String::new();
    let mut start_line = 1;
    let mut current_line = 0;
    let mut brace_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut in_single_line_comment;
    let mut in_multi_line_comment = false;
    let mut prev_char: Option<char> = None;

    for line_result in reader.lines() {
        current_line += 1;

        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        in_single_line_comment = false;

        if accumulator.is_empty() && line.trim().is_empty() {
            continue;
        }

        if accumulator.is_empty() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
        }

        if accumulator.is_empty() {
            start_line = current_line;
        }

        if !accumulator.is_empty() {
            accumulator.push('\n');
        }
        accumulator.push_str(&line);

        for ch in line.chars() {
            if in_multi_line_comment {
                if prev_char == Some('*') && ch == '/' {
                    in_multi_line_comment = false;
                }
                prev_char = Some(ch);
                continue;
            }

            if !in_string && !in_single_line_comment {
                if prev_char == Some('/') && ch == '/' {
                    in_single_line_comment = true;
                    prev_char = Some(ch);
                    continue;
                }
                if prev_char == Some('/') && ch == '*' {
                    in_multi_line_comment = true;
                    prev_char = Some(ch);
                    continue;
                }
            }

            prev_char = Some(ch);

            if in_single_line_comment {
                continue;
            }

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

        prev_char = None;

        if brace_depth == 0 && bracket_depth == 0 && !accumulator.trim().is_empty() {
            validator.validate_line(start_line, &accumulator);
            accumulator.clear();
            in_string = false;
            escape_next = false;
        }
    }

    if !accumulator.trim().is_empty() {
        validator.validate_line(start_line, &accumulator);
    }

    validator.into_issues().iter().map(format_issue).collect()
}

/// Validate a `.pxl` file on disk and return a list of warning/error messages.
///
/// Same output format as `validate()`, but reads from a file path.
/// Raises `OSError` if the file cannot be read.
#[pyfunction]
pub fn validate_file(path: &str) -> PyResult<Vec<String>> {
    let mut validator = Validator::new();
    let file_path = std::path::Path::new(path);

    validator.validate_file(file_path).map_err(|e| {
        pyo3::exceptions::PyOSError::new_err(format!("{}", e))
    })?;

    Ok(validator.into_issues().iter().map(format_issue).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_input() {
        let pxl = r##"{"type": "palette", "name": "test", "colors": {"x": "#FF0000"}}
{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "test", "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;
        let messages = validate(pxl);
        assert!(messages.is_empty(), "Expected no issues, got: {:?}", messages);
    }

    #[test]
    fn test_validate_invalid_json() {
        let messages = validate("{not valid json}");
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("ERROR"));
    }

    #[test]
    fn test_validate_missing_type() {
        let messages = validate(r#"{"name": "test"}"#);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("ERROR"));
        assert!(messages[0].contains("type"));
    }

    #[test]
    fn test_validate_invalid_color() {
        let messages =
            validate(r##"{"type": "palette", "name": "test", "colors": {"x": "#GGG"}}"##);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("ERROR"));
        assert!(messages[0].contains("color"));
    }

    #[test]
    fn test_validate_empty_input() {
        let messages = validate("");
        assert!(messages.is_empty());
    }

    #[test]
    fn test_validate_undefined_token() {
        let pxl = r##"{"type": "palette", "name": "pal", "colors": {"a": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "pal", "regions": {"b": {"rect": [0, 0, 4, 4]}}}"##;
        let messages = validate(pxl);
        assert!(!messages.is_empty());
        assert!(messages.iter().any(|m| m.contains("WARNING") && m.contains("Undefined token")));
    }

    #[test]
    fn test_validate_multiline_json5() {
        let pxl = r##"{
  type: "palette",
  name: "test",
  colors: {
    "x": "#FF0000"
  }
}"##;
        let messages = validate(pxl);
        assert!(messages.is_empty(), "Expected no issues, got: {:?}", messages);
    }
}
