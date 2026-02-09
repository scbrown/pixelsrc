//! Suggest Command Demo Tests
//!
//! Demonstrates the `pxl suggest` command functionality for finding
//! potential improvements and fixes in pixelsrc files.

use pixelsrc::suggest::{format_suggestion, suggest, Suggester, SuggestionFix, SuggestionType};
use std::io::Cursor;

// ============================================================================
// Typo Suggestion Tests
// ============================================================================
/// @demo cli/analysis#suggest
/// @demo cli/suggest#typo_correction
/// @title Typo Correction Suggestion
/// @description Suggests similar tokens for likely typos.
#[test]
fn test_suggest_typo_correction() {
    let candidates = vec!["skin", "hair", "shirt", "shadow"];

    // "shin" is close to "skin"
    let suggestions = suggest("shin", &candidates, 2);

    assert!(suggestions.contains(&"skin"), "Should suggest 'skin' for typo 'shin'");
}

/// @demo cli/suggest#distance_threshold
/// @title Suggestion Distance Threshold
/// @description Only suggests within specified edit distance.
#[test]
fn test_suggest_distance_threshold() {
    let candidates = vec!["red", "green", "blue", "yellow"];

    // "reed" is distance 1 from "red"
    let close_suggestions = suggest("reed", &candidates, 1);
    assert!(close_suggestions.contains(&"red"), "Should suggest 'red' within distance 1");

    // "xyz" is far from all candidates
    let far_suggestions = suggest("xyz", &candidates, 2);
    assert!(far_suggestions.is_empty(), "Should not suggest anything for very different input");
}

/// @demo cli/suggest#format
/// @title Format Suggestions
/// @description Formats suggestions for human-readable output.
#[test]
fn test_format_suggestions() {
    let candidates = vec!["skin", "shadow"];
    let suggestions = suggest("skn", &candidates, 2);

    let formatted = format_suggestion(&suggestions);

    assert!(formatted.is_some(), "Should have formatted output");
    let output = formatted.unwrap();
    assert!(output.contains("skin"), "Formatted output should mention suggested token");
}

// ============================================================================
// Report Tests
// ============================================================================
/// @demo cli/suggest#filter_by_type
/// @title Filter Suggestions by Type
/// @description Can filter suggestions to only show specific types.
#[test]
fn test_suggest_filter_by_type() {
    // Sprite with missing token in region
    let jsonl = r##"{"type": "sprite", "name": "test", "size": [2, 2], "palette": {"x": "#FF0000"}, "regions": {"y": {"rect": [0, 0, 2, 2]}}}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let missing_only = report.filter_by_type(SuggestionType::MissingToken);

    // All filtered suggestions should be of the requested type
    for suggestion in missing_only {
        assert_eq!(suggestion.suggestion_type, SuggestionType::MissingToken);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================
/// @demo cli/suggest#no_suggestions
/// @title No Suggestions for Valid File
/// @description Valid file produces no suggestions.
#[test]
fn test_suggest_no_suggestions_valid_file() {
    let jsonl = r##"{"type": "palette", "name": "colors", "colors": {"{x}": "#FF0000", "{y}": "#00FF00"}}
{"type": "sprite", "name": "test", "palette": "colors", "size": [2, 2], "regions": {"x": {"points": [[0,0],[1,1]]}, "y": {"points": [[1,0],[0,1]]}}}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    assert!(!report.has_suggestions(), "Valid file should have no suggestions");
}

// ============================================================================
// Import Suggestion Tests
// ============================================================================
/// @demo cli/suggest#include_to_import
/// @title Include to Import Migration
/// @description Suggests migrating @include: references to import declarations.
#[test]
fn test_suggest_include_to_import() {
    let jsonl = r#"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "@include:../palettes/brand.pxl", "regions": {"a": {"rect": [0, 0, 4, 4]}}}"#;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let include_suggestions = report.filter_by_type(SuggestionType::IncludeToImport);
    assert_eq!(include_suggestions.len(), 1, "Should suggest include migration");
    assert_eq!(include_suggestions[0].sprite, "hero");
    assert!(include_suggestions[0].message.contains("@include:"));
}

/// @demo cli/suggest#include_with_name
/// @title Include with Name Suffix Migration
/// @description Suggests migration for @include: with #name syntax.
#[test]
fn test_suggest_include_with_name_suffix() {
    let jsonl = r#"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "@include:palettes.pxl#gameboy", "regions": {"a": {"rect": [0, 0, 4, 4]}}}"#;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let include_suggestions = report.filter_by_type(SuggestionType::IncludeToImport);
    assert_eq!(include_suggestions.len(), 1);

    // The import JSON should reference the palette name
    match &include_suggestions[0].fix {
        SuggestionFix::UseImport { import_json, .. } => {
            assert!(import_json.contains("gameboy"), "Import should reference palette name");
            assert!(!import_json.contains(".pxl"), "Extension should be stripped");
        }
        _ => panic!("Expected UseImport fix"),
    }
}

/// @demo cli/suggest#no_include_for_normal_ref
/// @title No Include Suggestion for Normal Palette
/// @description Normal palette references don't trigger include migration suggestions.
#[test]
fn test_suggest_no_include_for_normal_palette() {
    let jsonl = r##"{"type": "palette", "name": "colors", "colors": {"x": "#FF0000"}}
{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "colors", "regions": {"x": {"rect": [0, 0, 4, 4]}}}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let include_suggestions = report.filter_by_type(SuggestionType::IncludeToImport);
    assert!(include_suggestions.is_empty(), "Normal refs should not trigger include migration");
}

/// @demo cli/suggest#filter_include_type
/// @title Filter Suggestions by Include Type
/// @description Can filter suggestions to only include migration suggestions.
#[test]
fn test_suggest_filter_by_include_type() {
    // Content with both missing token and include migration
    let jsonl = r##"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "@include:shared.pxl", "regions": {"a": {"rect": [0, 0, 4, 4]}}}
{"type": "sprite", "name": "test", "size": [2, 2], "palette": {"x": "#FF0000"}, "regions": {"y": {"rect": [0, 0, 2, 2]}}}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    // Should have both types of suggestions
    let include_only = report.filter_by_type(SuggestionType::IncludeToImport);
    let missing_only = report.filter_by_type(SuggestionType::MissingToken);

    assert_eq!(include_only.len(), 1, "Should have 1 include migration suggestion");
    assert_eq!(missing_only.len(), 1, "Should have 1 missing token suggestion");

    // Filtered lists should only contain their type
    for s in include_only {
        assert_eq!(s.suggestion_type, SuggestionType::IncludeToImport);
    }
    for s in missing_only {
        assert_eq!(s.suggestion_type, SuggestionType::MissingToken);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================
/// @demo cli/suggest#line_numbers
/// @title Suggestion Line Numbers
/// @description Suggestions include accurate line numbers for locating issues.
#[test]
fn test_suggest_line_numbers() {
    let jsonl = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "test", "palette": "p", "size": [2, 1], "regions": {"x": {"points": [[0,0]]}, "y": {"points": [[1,0]]}}}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    if !report.suggestions.is_empty() {
        let suggestion = &report.suggestions[0];
        assert!(suggestion.line > 0, "Line number should be 1-indexed and positive");
        assert_eq!(suggestion.line, 2, "Missing token should be on line 2 (sprite line)");
    }
}
