//! Typo suggestions using Levenshtein distance
//!
//! Also provides comprehensive suggestion analysis for pixelsrc files,
//! including missing token detection and row completion suggestions.

use crate::models::{PaletteRef, Sprite, TtpObject};
use crate::tokenizer::tokenize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

/// Type of suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    /// A token is used but not defined in the palette
    MissingToken,
    /// A row is shorter than others and could be completed
    RowCompletion,
}

impl std::fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionType::MissingToken => write!(f, "missing_token"),
            SuggestionType::RowCompletion => write!(f, "row_completion"),
        }
    }
}

/// A suggestion for fixing an issue in a pixelsrc file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Type of suggestion
    #[serde(rename = "type")]
    pub suggestion_type: SuggestionType,
    /// Line number (1-indexed) where the issue was found
    pub line: usize,
    /// Name of the sprite or object affected
    pub sprite: String,
    /// Human-readable message describing the issue
    pub message: String,
    /// The fix to apply (e.g., token to add, row to extend)
    pub fix: SuggestionFix,
}

/// The suggested fix for an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SuggestionFix {
    /// Replace a token with another (typo correction)
    ReplaceToken {
        /// The token to replace
        from: String,
        /// The suggested replacement
        to: String,
    },
    /// Add a token to the palette
    AddToPalette {
        /// The token to add
        token: String,
        /// Suggested color (hex)
        suggested_color: String,
    },
    /// Extend a row to match expected width
    ExtendRow {
        /// The row index (0-indexed)
        row_index: usize,
        /// Current row content
        current: String,
        /// Suggested extended row content
        suggested: String,
        /// Token used for padding
        pad_token: String,
        /// Number of tokens to add
        tokens_to_add: usize,
    },
}

/// Result of suggestion analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuggestionReport {
    /// All suggestions found
    pub suggestions: Vec<Suggestion>,
    /// Number of sprites analyzed
    pub sprites_analyzed: usize,
}

impl SuggestionReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there are any suggestions
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }

    /// Filter suggestions by type
    pub fn filter_by_type(&self, suggestion_type: SuggestionType) -> Vec<&Suggestion> {
        self.suggestions
            .iter()
            .filter(|s| s.suggestion_type == suggestion_type)
            .collect()
    }

    /// Count suggestions of a specific type
    pub fn count_by_type(&self, suggestion_type: SuggestionType) -> usize {
        self.suggestions
            .iter()
            .filter(|s| s.suggestion_type == suggestion_type)
            .count()
    }
}

/// Analyzer for generating suggestions from pixelsrc content
pub struct Suggester {
    /// Report being built
    report: SuggestionReport,
    /// Known palette names -> set of defined tokens
    palettes: HashMap<String, HashSet<String>>,
    /// Built-in palette names
    builtin_palettes: HashSet<String>,
}

impl Default for Suggester {
    fn default() -> Self {
        Self::new()
    }
}

impl Suggester {
    /// Create a new suggester
    pub fn new() -> Self {
        // Initialize with built-in palette names
        let builtin_palettes: HashSet<String> = crate::palettes::list_builtins()
            .into_iter()
            .map(|s| format!("@{}", s))
            .collect();

        Self {
            report: SuggestionReport::new(),
            palettes: HashMap::new(),
            builtin_palettes,
        }
    }

    /// Analyze content from a reader
    pub fn analyze_reader<R: BufRead>(&mut self, reader: R) -> Result<(), std::io::Error> {
        for (line_idx, line_result) in reader.lines().enumerate() {
            let line_number = line_idx + 1;
            let line = line_result?;
            self.analyze_line(line_number, &line);
        }
        Ok(())
    }

    /// Analyze a single line
    pub fn analyze_line(&mut self, line_number: usize, content: &str) {
        // Skip empty lines
        if content.trim().is_empty() {
            return;
        }

        // Try to parse as TtpObject
        let obj: TtpObject = match serde_json::from_str(content) {
            Ok(obj) => obj,
            Err(_) => return, // Skip lines that don't parse
        };

        match obj {
            TtpObject::Palette(palette) => {
                // Register palette tokens
                let tokens: HashSet<String> = palette.colors.keys().cloned().collect();
                self.palettes.insert(palette.name, tokens);
            }
            TtpObject::Sprite(sprite) => {
                self.analyze_sprite(line_number, &sprite);
            }
            _ => {} // Ignore other types for now
        }
    }

    /// Analyze a sprite for suggestions
    fn analyze_sprite(&mut self, line_number: usize, sprite: &Sprite) {
        self.report.sprites_analyzed += 1;

        // Get palette tokens
        let palette_tokens = self.get_palette_tokens(&sprite.palette);

        // Analyze grid
        let mut all_tokens_used: HashSet<String> = HashSet::new();
        let mut row_lengths: Vec<(usize, String, Vec<String>)> = Vec::new();

        for (_row_idx, row) in sprite.grid.iter().enumerate() {
            let (tokens, _warnings) = tokenize(row);
            all_tokens_used.extend(tokens.iter().cloned());
            row_lengths.push((tokens.len(), row.clone(), tokens));
        }

        // Check for missing tokens
        if let Some(ref defined_tokens) = palette_tokens {
            for token in &all_tokens_used {
                if !defined_tokens.contains(token) {
                    self.suggest_missing_token(line_number, sprite, token, defined_tokens);
                }
            }
        }

        // Check for row length mismatches
        if !row_lengths.is_empty() {
            let max_length = row_lengths.iter().map(|(len, _, _)| *len).max().unwrap_or(0);
            if max_length > 0 {
                // Find the most common token for padding (prefer {_} if present)
                let pad_token = self.find_pad_token(&all_tokens_used, palette_tokens.as_ref());

                for (row_idx, (length, row_content, _tokens)) in row_lengths.iter().enumerate() {
                    if *length < max_length {
                        self.suggest_row_completion(
                            line_number,
                            sprite,
                            row_idx,
                            row_content,
                            *length,
                            max_length,
                            &pad_token,
                        );
                    }
                }
            }
        }
    }

    /// Suggest a fix for a missing token
    fn suggest_missing_token(
        &mut self,
        line_number: usize,
        sprite: &Sprite,
        unknown_token: &str,
        defined_tokens: &HashSet<String>,
    ) {
        let known: Vec<&str> = defined_tokens.iter().map(|s| s.as_str()).collect();
        let suggestions = suggest(unknown_token, &known, 2);

        if !suggestions.is_empty() {
            // Typo correction
            self.report.suggestions.push(Suggestion {
                suggestion_type: SuggestionType::MissingToken,
                line: line_number,
                sprite: sprite.name.clone(),
                message: format!(
                    "Token {} is not defined in palette. Did you mean {}?",
                    unknown_token, suggestions[0]
                ),
                fix: SuggestionFix::ReplaceToken {
                    from: unknown_token.to_string(),
                    to: suggestions[0].to_string(),
                },
            });
        } else {
            // Suggest adding to palette
            self.report.suggestions.push(Suggestion {
                suggestion_type: SuggestionType::MissingToken,
                line: line_number,
                sprite: sprite.name.clone(),
                message: format!(
                    "Token {} is not defined in palette. Consider adding it.",
                    unknown_token
                ),
                fix: SuggestionFix::AddToPalette {
                    token: unknown_token.to_string(),
                    suggested_color: "#FF00FF".to_string(), // Magenta as placeholder
                },
            });
        }
    }

    /// Suggest extending a row to match expected width
    fn suggest_row_completion(
        &mut self,
        line_number: usize,
        sprite: &Sprite,
        row_idx: usize,
        current_row: &str,
        current_length: usize,
        expected_length: usize,
        pad_token: &str,
    ) {
        let tokens_to_add = expected_length - current_length;
        let padding = pad_token.repeat(tokens_to_add);
        let suggested_row = format!("{}{}", current_row, padding);

        self.report.suggestions.push(Suggestion {
            suggestion_type: SuggestionType::RowCompletion,
            line: line_number,
            sprite: sprite.name.clone(),
            message: format!(
                "Row {} has {} tokens, expected {}. Add {} {} token(s) to complete.",
                row_idx + 1,
                current_length,
                expected_length,
                tokens_to_add,
                pad_token
            ),
            fix: SuggestionFix::ExtendRow {
                row_index: row_idx,
                current: current_row.to_string(),
                suggested: suggested_row,
                pad_token: pad_token.to_string(),
                tokens_to_add,
            },
        });
    }

    /// Find the best token to use for padding
    fn find_pad_token(
        &self,
        used_tokens: &HashSet<String>,
        defined_tokens: Option<&HashSet<String>>,
    ) -> String {
        // Prefer {_} if it's defined (conventional transparent token)
        if let Some(defined) = defined_tokens {
            if defined.contains("{_}") {
                return "{_}".to_string();
            }
        }
        if used_tokens.contains("{_}") {
            return "{_}".to_string();
        }

        // Otherwise, use the first defined token or a default
        if let Some(defined) = defined_tokens {
            if let Some(first) = defined.iter().next() {
                return first.clone();
            }
        }

        // Fallback
        "{_}".to_string()
    }

    /// Get tokens defined in a palette reference
    fn get_palette_tokens(&self, palette_ref: &PaletteRef) -> Option<HashSet<String>> {
        match palette_ref {
            PaletteRef::Named(name) => {
                // Check for @include: syntax
                if name.starts_with("@include:") {
                    return None;
                }

                // Check for built-in palettes
                if self.builtin_palettes.contains(name) {
                    let palette_name = name.strip_prefix('@').unwrap_or(name);
                    if let Some(palette) = crate::palettes::get_builtin(palette_name) {
                        return Some(palette.colors.keys().cloned().collect());
                    }
                    return None;
                }

                // Check defined palettes
                self.palettes.get(name).cloned()
            }
            PaletteRef::Inline(colors) => Some(colors.keys().cloned().collect()),
        }
    }

    /// Consume the suggester and return the report
    pub fn into_report(self) -> SuggestionReport {
        self.report
    }
}

/// Calculate the Levenshtein distance between two strings.
/// This measures the minimum number of single-character edits (insertions,
/// deletions, or substitutions) required to change one string into the other.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Handle empty strings
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use two rows instead of full matrix for O(min(m,n)) space
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr_row[j] = (prev_row[j] + 1) // deletion
                .min(curr_row[j - 1] + 1) // insertion
                .min(prev_row[j - 1] + cost); // substitution
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Find suggestions for a typo from a list of valid options.
/// Returns suggestions sorted by edit distance (closest first).
///
/// - `query`: The mistyped string
/// - `candidates`: List of valid options to compare against
/// - `max_distance`: Maximum edit distance to consider (default: 3)
///
/// Returns up to 3 closest matches within the max distance.
pub fn suggest<'a>(query: &str, candidates: &[&'a str], max_distance: usize) -> Vec<&'a str> {
    let query_lower = query.to_lowercase();

    let mut scored: Vec<(&str, usize)> = candidates
        .iter()
        .map(|&candidate| {
            let candidate_lower = candidate.to_lowercase();
            let distance = levenshtein_distance(&query_lower, &candidate_lower);
            (candidate, distance)
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .collect();

    // Sort by distance (ascending)
    scored.sort_by_key(|(_, distance)| *distance);

    // Return up to 3 closest matches
    scored.into_iter().take(3).map(|(s, _)| s).collect()
}

/// Format a "did you mean?" suggestion string.
/// Returns None if there are no suggestions.
pub fn format_suggestion(suggestions: &[&str]) -> Option<String> {
    match suggestions.len() {
        0 => None,
        1 => Some(format!("Did you mean '{}'?", suggestions[0])),
        2 => Some(format!(
            "Did you mean '{}' or '{}'?",
            suggestions[0], suggestions[1]
        )),
        _ => Some(format!(
            "Did you mean '{}', '{}', or '{}'?",
            suggestions[0], suggestions[1], suggestions[2]
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", "hello"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        // Substitution
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        // Insertion
        assert_eq!(levenshtein_distance("hello", "helllo"), 1);
        // Deletion
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_suggest_exact_match() {
        let candidates = &["character", "item", "tileset", "animation"];
        let suggestions = suggest("character", candidates, 3);
        assert_eq!(suggestions, vec!["character"]);
    }

    #[test]
    fn test_suggest_typo() {
        let candidates = &["character", "item", "tileset", "animation"];
        // "charactor" is 1 edit from "character"
        let suggestions = suggest("charactor", candidates, 3);
        assert_eq!(suggestions[0], "character");
    }

    #[test]
    fn test_suggest_case_insensitive() {
        let candidates = &["character", "Item", "TILESET"];
        let suggestions = suggest("CHARACTER", candidates, 3);
        assert_eq!(suggestions[0], "character");
    }

    #[test]
    fn test_suggest_no_match() {
        let candidates = &["character", "item", "tileset"];
        let suggestions = suggest("xyz", candidates, 2);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_format_suggestion_none() {
        assert_eq!(format_suggestion(&[]), None);
    }

    #[test]
    fn test_format_suggestion_one() {
        let result = format_suggestion(&["character"]);
        assert_eq!(result, Some("Did you mean 'character'?".to_string()));
    }

    #[test]
    fn test_format_suggestion_two() {
        let result = format_suggestion(&["character", "item"]);
        assert_eq!(
            result,
            Some("Did you mean 'character' or 'item'?".to_string())
        );
    }

    #[test]
    fn test_format_suggestion_three() {
        let result = format_suggestion(&["character", "item", "tileset"]);
        assert_eq!(
            result,
            Some("Did you mean 'character', 'item', or 'tileset'?".to_string())
        );
    }

    // Suggester tests

    #[test]
    fn test_suggester_missing_token_typo() {
        let mut suggester = Suggester::new();
        // Palette with {skin} and {hair}
        suggester.analyze_line(
            1,
            r##"{"type": "palette", "name": "char", "colors": {"{skin}": "#FFCC99", "{hair}": "#8B4513"}}"##,
        );
        // Sprite using {skni} (typo for {skin})
        suggester.analyze_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "char", "grid": ["{skni}"]}"#,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        let missing_token = report.filter_by_type(SuggestionType::MissingToken);
        assert_eq!(missing_token.len(), 1);

        // Should suggest replacing {skni} with {skin}
        match &missing_token[0].fix {
            SuggestionFix::ReplaceToken { from, to } => {
                assert_eq!(from, "{skni}");
                assert_eq!(to, "{skin}");
            }
            _ => panic!("Expected ReplaceToken fix"),
        }
    }

    #[test]
    fn test_suggester_missing_token_add_to_palette() {
        let mut suggester = Suggester::new();
        // Palette with {_} and {x} - using {unknown_color} which is very different from both
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}{unknown_color}{x}"]}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        let missing_token = report.filter_by_type(SuggestionType::MissingToken);
        assert_eq!(missing_token.len(), 1);

        // {unknown_color} is too different from {_} and {x} to be a typo, so suggest adding it
        match &missing_token[0].fix {
            SuggestionFix::AddToPalette { token, .. } => {
                assert_eq!(token, "{unknown_color}");
            }
            _ => panic!("Expected AddToPalette fix"),
        }
    }

    #[test]
    fn test_suggester_row_completion() {
        let mut suggester = Suggester::new();
        // Sprite with uneven row lengths
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}{x}", "{x}{x}{x}{x}"]}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        let row_completion = report.filter_by_type(SuggestionType::RowCompletion);
        assert_eq!(row_completion.len(), 1);

        // First row (2 tokens) should be extended to match second row (4 tokens)
        match &row_completion[0].fix {
            SuggestionFix::ExtendRow { row_index, tokens_to_add, pad_token, suggested, .. } => {
                assert_eq!(*row_index, 0);
                assert_eq!(*tokens_to_add, 2);
                assert_eq!(pad_token, "{_}");
                assert_eq!(suggested, "{x}{x}{_}{_}");
            }
            _ => panic!("Expected ExtendRow fix"),
        }
    }

    #[test]
    fn test_suggester_no_suggestions_for_valid_sprite() {
        let mut suggester = Suggester::new();
        // Valid sprite with no issues
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "valid", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}{x}{x}", "{x}{_}{x}", "{x}{x}{x}"]}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(!report.has_suggestions());
    }

    #[test]
    fn test_suggester_inline_palette() {
        let mut suggester = Suggester::new();
        // Sprite with inline palette
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "palette": {"{a}": "#FF0000", "{b}": "#00FF00"}, "grid": ["{a}{b}{c}"]}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        // {c} is undefined
        let missing_token = report.filter_by_type(SuggestionType::MissingToken);
        assert_eq!(missing_token.len(), 1);
        assert!(missing_token[0].message.contains("{c}"));
    }

    #[test]
    fn test_suggestion_report_filter() {
        let mut suggester = Suggester::new();
        // Multiple issues
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{y}{x}", "{x}{x}{x}{x}"]}"##,
        );

        let report = suggester.into_report();

        // Should have both missing token and row completion
        assert!(report.count_by_type(SuggestionType::MissingToken) > 0);
        assert!(report.count_by_type(SuggestionType::RowCompletion) > 0);
    }
}
