//! Suggest Command Demo Tests
//!
//! Demonstrates the `pxl suggest` command functionality for finding
//! potential improvements and fixes in pixelsrc files.

use pixelsrc::suggest::{format_suggestion, suggest, Suggester, SuggestionFix, SuggestionType};
use std::io::Cursor;

// ============================================================================
// Typo Suggestion Tests
// ============================================================================

/// @demo cli/suggest#typo_basic
/// @title Suggest Typo Corrections
/// @description `pxl suggest` finds similar tokens when an unknown token is used.
#[test]
fn test_suggest_typo_correction() {
    let candidates = vec!["skin", "hair", "shirt", "shadow"];

    // "shin" is close to "skin"
    let suggestions = suggest("shin", &candidates, 2);

    assert!(suggestions.contains(&"skin"), "Should suggest 'skin' for typo 'shin'");
}

/// @demo cli/suggest#typo_distance
/// @title Suggestion Distance Threshold
/// @description Only tokens within the edit distance threshold are suggested.
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

/// @demo cli/suggest#format_suggestions
/// @title Format Suggestions
/// @description Suggestions can be formatted as user-friendly text.
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
// Missing Token Detection Tests
// ============================================================================

/// @demo cli/suggest#missing_token
/// @title Detect Missing Tokens
/// @description Finds tokens used in grids that are not defined in palettes.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_suggest_missing_token() {
    let jsonl = r##"{"type": "palette", "name": "colors", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "name": "test", "palette": "colors", "grid": ["{a}{b}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let missing_token_suggestions: Vec<_> = report
        .suggestions
        .iter()
        .filter(|s| s.suggestion_type == SuggestionType::MissingToken)
        .collect();

    assert!(!missing_token_suggestions.is_empty(), "Should detect missing token {{b}}");
    assert!(
        missing_token_suggestions
            .iter()
            .any(|s| s.message.contains("{b}") || s.message.contains("b")),
        "Should mention the missing token {{b}}"
    );
}

/// @demo cli/suggest#missing_token_fix
/// @title Missing Token Fix Suggestion
/// @description Suggests adding the missing token to the palette with a color.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_suggest_missing_token_fix() {
    // Use a token that is very different from {x} to trigger AddToPalette instead of ReplaceToken
    let jsonl = r##"{"type": "sprite", "name": "test", "palette": {"{x}": "#FF0000"}, "grid": ["{x}{unknown_color}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let missing_suggestions: Vec<_> = report
        .suggestions
        .iter()
        .filter(|s| s.suggestion_type == SuggestionType::MissingToken)
        .collect();

    assert!(!missing_suggestions.is_empty());

    // Check that fix is AddToPalette (for very different token names)
    let fix = &missing_suggestions[0].fix;
    match fix {
        SuggestionFix::AddToPalette { token, suggested_color } => {
            assert!(token.contains("unknown_color"), "Should suggest adding {{unknown_color}}");
            assert!(!suggested_color.is_empty(), "Should suggest a color");
        }
        _ => panic!("Expected AddToPalette fix for token very different from existing tokens"),
    }
}

// ============================================================================
// Row Completion Tests
// ============================================================================

/// @demo cli/suggest#row_completion
/// @title Suggest Row Completion
/// @description Detects rows that are shorter than others and suggests padding.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_suggest_row_completion() {
    let jsonl = r##"{"type": "sprite", "name": "test", "palette": {"{_}": "#0000", "{x}": "#FF0000"}, "grid": ["{x}{x}{x}", "{x}{x}", "{x}{x}{x}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let row_completion_suggestions: Vec<_> = report
        .suggestions
        .iter()
        .filter(|s| s.suggestion_type == SuggestionType::RowCompletion)
        .collect();

    assert!(!row_completion_suggestions.is_empty(), "Should detect short row needing completion");
}

/// @demo cli/suggest#row_completion_fix
/// @title Row Completion Fix
/// @description Suggests extending short rows with appropriate padding token.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_suggest_row_completion_fix() {
    let jsonl = r##"{"type": "sprite", "name": "test", "palette": {"{_}": "#0000", "{x}": "#FF0000"}, "grid": ["{x}{x}{x}{x}", "{x}{x}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let row_suggestions: Vec<_> = report
        .suggestions
        .iter()
        .filter(|s| s.suggestion_type == SuggestionType::RowCompletion)
        .collect();

    if !row_suggestions.is_empty() {
        let fix = &row_suggestions[0].fix;
        match fix {
            SuggestionFix::ExtendRow { row_index, tokens_to_add, pad_token, .. } => {
                assert_eq!(row_index, &1, "Should target row 1 (0-indexed)");
                assert_eq!(tokens_to_add, &2, "Should add 2 tokens to match width 4");
                assert!(pad_token.contains("_"), "Should pad with transparent token");
            }
            _ => panic!("Expected ExtendRow fix for row completion"),
        }
    }
}

// ============================================================================
// Report Tests
// ============================================================================

/// @demo cli/suggest#report_counts
/// @title Suggestion Report Counts
/// @description Report tracks number of suggestions by type.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_suggest_report_counts() {
    let jsonl = r##"{"type": "sprite", "name": "test1", "palette": {"{x}": "#FF0000"}, "grid": ["{x}{y}"]}
{"type": "sprite", "name": "test2", "palette": {"{a}": "#00FF00"}, "grid": ["{a}{a}{a}", "{a}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    assert!(report.has_suggestions(), "Should have suggestions for both issues");

    let missing_count = report.count_by_type(SuggestionType::MissingToken);
    let completion_count = report.count_by_type(SuggestionType::RowCompletion);

    assert!(missing_count > 0, "Should count missing token suggestions");
    assert!(completion_count > 0, "Should count row completion suggestions");
}

/// @demo cli/suggest#filter_by_type
/// @title Filter Suggestions by Type
/// @description Can filter suggestions to only show specific types.
#[test]
fn test_suggest_filter_by_type() {
    let jsonl = r##"{"type": "sprite", "name": "test", "palette": {"{x}": "#FF0000"}, "grid": ["{x}{y}", "{x}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    let missing_only = report.filter_by_type(SuggestionType::MissingToken);
    let completion_only = report.filter_by_type(SuggestionType::RowCompletion);

    // All filtered suggestions should be of the requested type
    for suggestion in missing_only {
        assert_eq!(suggestion.suggestion_type, SuggestionType::MissingToken);
    }
    for suggestion in completion_only {
        assert_eq!(suggestion.suggestion_type, SuggestionType::RowCompletion);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

/// @demo cli/suggest#no_suggestions
/// @title No Suggestions for Valid File
/// @description Valid files with no issues produce empty suggestion reports.
#[test]
fn test_suggest_no_suggestions_valid_file() {
    let jsonl = r##"{"type": "palette", "name": "colors", "colors": {"{x}": "#FF0000", "{y}": "#00FF00"}}
{"type": "sprite", "name": "test", "palette": "colors", "grid": ["{x}{y}", "{y}{x}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    assert!(!report.has_suggestions(), "Valid file should have no suggestions");
}

/// @demo cli/suggest#multiple_sprites
/// @title Suggestions Across Multiple Sprites
/// @description Analyzes all sprites in a file and tracks which sprite each suggestion relates to.
#[test]
    #[ignore = "Grid format deprecated"]
fn test_suggest_multiple_sprites() {
    let jsonl = r##"{"type": "sprite", "name": "sprite_a", "palette": {"{x}": "#FF0000"}, "grid": ["{x}{y}"]}
{"type": "sprite", "name": "sprite_b", "palette": {"{a}": "#00FF00"}, "grid": ["{a}{b}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    assert_eq!(report.sprites_analyzed, 2, "Should analyze both sprites");

    // Each sprite should have a missing token suggestion
    let sprite_names: Vec<_> = report.suggestions.iter().map(|s| s.sprite.clone()).collect();
    assert!(sprite_names.contains(&"sprite_a".to_string()), "Should have suggestion for sprite_a");
    assert!(sprite_names.contains(&"sprite_b".to_string()), "Should have suggestion for sprite_b");
}

/// @demo cli/suggest#line_numbers
/// @title Suggestion Line Numbers
/// @description Suggestions include accurate line numbers for locating issues.
#[test]
fn test_suggest_line_numbers() {
    let jsonl = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "test", "palette": "p", "grid": ["{x}{y}"]}"##;

    let mut suggester = Suggester::new();
    suggester.analyze_reader(Cursor::new(jsonl)).unwrap();
    let report = suggester.into_report();

    if !report.suggestions.is_empty() {
        let suggestion = &report.suggestions[0];
        assert!(suggestion.line > 0, "Line number should be 1-indexed and positive");
        assert_eq!(suggestion.line, 2, "Missing token should be on line 2 (sprite line)");
    }
}
