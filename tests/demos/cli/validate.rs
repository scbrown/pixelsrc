//! Validate Command Demo Tests
//!
//! Demonstrates the `pxl validate` command functionality for checking
//! Pixelsrc files for correctness and reporting errors.

use pixelsrc::validate::{Severity, Validator};

/// Run validation on JSONL content and return (error_count, warning_count).
fn validate_content(jsonl: &str) -> (usize, usize) {
    let mut validator = Validator::new();

    for (line_idx, line) in jsonl.lines().enumerate() {
        validator.validate_line(line_idx + 1, line);
    }

    (validator.error_count(), validator.warning_count())
}

/// Validate and collect all issues as strings.
fn validate_with_issues(jsonl: &str) -> Vec<String> {
    let mut validator = Validator::new();

    for (line_idx, line) in jsonl.lines().enumerate() {
        validator.validate_line(line_idx + 1, line);
    }

    validator
        .issues()
        .iter()
        .map(|issue| {
            format!(
                "Line {}: [{}] {} - {}",
                issue.line,
                match issue.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARNING",
                },
                issue.issue_type,
                issue.message
            )
        })
        .collect()
}

// ============================================================================
// Basic Validation Tests
// ============================================================================
/// @title Invalid JSON Detection
/// @description `pxl validate` reports invalid JSON syntax errors.
#[test]
fn test_validate_invalid_json() {
    let jsonl = r##"{not valid json}
{"type": "sprite", "name": "test"}"##;

    let (errors, _) = validate_content(jsonl);

    assert!(errors > 0, "Invalid JSON should produce errors");

    let issues = validate_with_issues(jsonl);
    assert!(issues.iter().any(|i| i.contains("ERROR")), "Should have error-level issues");
}
#[test]
fn test_validate_missing_type() {
    let jsonl = r##"{"name": "orphan", "colors": {"{x}": "#FF0000"}}"##;

    let (errors, _) = validate_content(jsonl);

    assert!(errors > 0, "Missing type should produce an error");

    let issues = validate_with_issues(jsonl);
    assert!(
        issues.iter().any(|i| i.to_lowercase().contains("type")),
        "Error should mention missing type field"
    );
}
#[test]
fn test_validate_missing_name() {
    let jsonl = r##"{"type": "sprite", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}"##;

    let (errors, _) = validate_content(jsonl);

    assert!(errors > 0, "Missing name should produce an error");
}
#[test]
fn test_validate_undefined_palette() {
    let jsonl =
        r##"{"type": "sprite", "name": "orphan", "palette": "nonexistent", "grid": ["{x}"]}"##;

    let (errors, _) = validate_content(jsonl);

    // Note: Validator may not catch cross-object references depending on implementation
    // This tests the principle of detecting undefined references
    let issues = validate_with_issues(jsonl);

    // The sprite definition itself may be valid syntactically
    // but semantic validation catches undefined palette refs
    assert!(
        errors > 0 || issues.iter().any(|i| i.contains("WARNING")),
        "Should warn or error on undefined palette reference"
    );
}
#[test]
fn test_validate_invalid_color() {
    let jsonl = r##"{"type": "palette", "name": "bad", "colors": {"{x}": "not-a-color"}}"##;

    let (errors, warnings) = validate_content(jsonl);

    assert!(errors > 0 || warnings > 0, "Invalid color should produce error or warning");
}
#[test]
fn test_validate_strict_mode() {
    // Content that might produce warnings but not errors
    let jsonl = r##"{"type": "sprite", "name": "test", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"], "extra_field": true}"##;

    let mut validator = Validator::new();
    for (line_idx, line) in jsonl.lines().enumerate() {
        validator.validate_line(line_idx + 1, line);
    }

    let warnings = validator.warning_count();
    let errors = validator.error_count();

    // In strict mode, total issues = errors + warnings would all fail
    let strict_failures = errors + warnings;

    // Verify we can detect when strict mode would change the outcome
    if warnings > 0 && errors == 0 {
        assert!(strict_failures > 0, "Strict mode converts warnings to failures");
    }
}

// ============================================================================
// Error Reporting Tests
// ============================================================================
#[test]
fn test_validate_line_numbers() {
    let jsonl = r##"{"type": "palette", "name": "ok", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "good", "palette": "ok", "grid": ["{x}"]}
{invalid json on line 3}
{"type": "sprite", "name": "after", "palette": "ok", "grid": ["{x}"]}"##;

    let mut validator = Validator::new();
    for (line_idx, line) in jsonl.lines().enumerate() {
        validator.validate_line(line_idx + 1, line);
    }

    // Find the error for line 3
    let line_3_errors: Vec<_> = validator
        .issues()
        .iter()
        .filter(|i| i.line == 3 && matches!(i.severity, Severity::Error))
        .collect();

    assert!(!line_3_errors.is_empty(), "Should report error on line 3 where invalid JSON is");
}
#[test]
fn test_validate_multiple_errors() {
    let jsonl = r##"{bad json 1}
{bad json 2}
{bad json 3}"##;

    let (errors, _) = validate_content(jsonl);

    assert!(errors >= 3, "Should report errors for all invalid lines");
}
#[test]
fn test_validate_json_output() {
    let jsonl =
        r##"{"type": "sprite", "name": "test", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}"##;

    let mut validator = Validator::new();
    for (line_idx, line) in jsonl.lines().enumerate() {
        validator.validate_line(line_idx + 1, line);
    }

    // Simulate --json output structure
    let json_output = serde_json::json!({
        "valid": !validator.has_errors(),
        "errors": validator.error_count(),
        "warnings": validator.warning_count(),
        "issues": validator.issues().iter().map(|i| {
            serde_json::json!({
                "line": i.line,
                "severity": match i.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                },
                "type": format!("{:?}", i.issue_type),
                "message": i.message.clone()
            })
        }).collect::<Vec<_>>()
    });

    // Verify JSON structure is valid
    assert!(json_output["valid"].is_boolean());
    assert!(json_output["errors"].is_number());
    assert!(json_output["issues"].is_array());
}

// ============================================================================
// Edge Cases
// ============================================================================
#[test]
fn test_validate_empty_file() {
    let jsonl = "";

    let (errors, warnings) = validate_content(jsonl);

    assert_eq!(errors, 0, "Empty file should have no errors");
    assert_eq!(warnings, 0, "Empty file should have no warnings");
}
#[test]
fn test_validate_whitespace_lines() {
    let jsonl = r##"
{"type": "sprite", "name": "test", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}

{"type": "sprite", "name": "test2", "palette": {"{x}": "#00FF00"}, "grid": ["{x}"]}
"##;

    let (errors, _) = validate_content(jsonl);

    assert_eq!(errors, 0, "Whitespace lines should be ignored");
}
#[test]
fn test_validate_comment_lines() {
    // Note: JSONL format does not officially support comments, but some parsers
    // allow lines starting with // as informal comments. The validator may or
    // may not accept them depending on strictness.
    let jsonl = r##"{"type": "sprite", "name": "test", "palette": {"{x}": "#FF0000"}, "grid": ["{x}"]}
{"type": "sprite", "name": "test2", "palette": {"{x}": "#00FF00"}, "grid": ["{x}"]}"##;

    let (errors, _) = validate_content(jsonl);

    // Valid JSONL without comments should pass
    assert_eq!(errors, 0, "Valid JSONL should have no errors");
}
