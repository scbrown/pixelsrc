//! Typo suggestions using Levenshtein distance
//!
//! Also provides comprehensive suggestion analysis for pixelsrc files,
//! including missing token detection.

use crate::build::project_registry::ProjectRegistry;
use crate::models::{PaletteRef, Sprite, TtpObject};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

/// Type of suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    /// A token is used but not defined in the palette
    MissingToken,
    /// An @include: reference can be migrated to an import declaration
    IncludeToImport,
    /// A cross-file reference would benefit from an explicit import
    AddExplicitImport,
}

impl std::fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionType::MissingToken => write!(f, "missing_token"),
            SuggestionType::IncludeToImport => write!(f, "include_to_import"),
            SuggestionType::AddExplicitImport => write!(f, "add_explicit_import"),
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
    /// The fix to apply
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
    /// Replace @include: with an import declaration
    UseImport {
        /// The @include: reference to replace
        include_ref: String,
        /// The suggested import declaration JSON
        import_json: String,
    },
    /// Add an explicit import declaration
    AddImport {
        /// The suggested import declaration JSON
        import_json: String,
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
        self.suggestions.iter().filter(|s| s.suggestion_type == suggestion_type).collect()
    }

    /// Count suggestions of a specific type
    pub fn count_by_type(&self, suggestion_type: SuggestionType) -> usize {
        self.suggestions.iter().filter(|s| s.suggestion_type == suggestion_type).count()
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
        let builtin_palettes: HashSet<String> =
            crate::palettes::list_builtins().into_iter().map(|s| format!("@{}", s)).collect();

        Self { report: SuggestionReport::new(), palettes: HashMap::new(), builtin_palettes }
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
            TtpObject::Sprite(ref sprite) => {
                // Check for @include: references and suggest migration
                if let PaletteRef::Named(ref name) = sprite.palette {
                    if name.starts_with("@include:") {
                        self.suggest_include_migration(line_number, sprite, name);
                    }
                }
                self.analyze_sprite(line_number, sprite);
            }
            _ => {}
        }
    }

    /// Suggest migrating an @include: reference to an import declaration
    fn suggest_include_migration(
        &mut self,
        line_number: usize,
        sprite: &Sprite,
        include_ref: &str,
    ) {
        let after_prefix = include_ref.strip_prefix("@include:").unwrap_or(include_ref);

        // Parse optional #name suffix
        let (path_part, palette_name) = if let Some(hash_pos) = after_prefix.rfind('#') {
            (&after_prefix[..hash_pos], Some(&after_prefix[hash_pos + 1..]))
        } else {
            (after_prefix, None)
        };

        // Strip extension for import path
        let import_path = path_part
            .strip_suffix(".pxl")
            .or_else(|| path_part.strip_suffix(".jsonl"))
            .unwrap_or(path_part);

        // Build the suggested import JSON
        let import_json = if let Some(name) = palette_name {
            format!(r#"{{"type": "import", "from": "{}", "palettes": ["{}"]}}"#, import_path, name)
        } else {
            format!(r#"{{"type": "import", "from": "{}"}}"#, import_path)
        };

        self.report.suggestions.push(Suggestion {
            suggestion_type: SuggestionType::IncludeToImport,
            line: line_number,
            sprite: sprite.name.clone(),
            message: "Consider migrating @include: to an import declaration for cleaner dependency tracking".to_string(),
            fix: SuggestionFix::UseImport {
                include_ref: include_ref.to_string(),
                import_json,
            },
        });
    }

    /// Analyze a sprite for suggestions
    fn analyze_sprite(&mut self, line_number: usize, sprite: &Sprite) {
        self.report.sprites_analyzed += 1;

        // Get palette tokens
        let palette_tokens = self.get_palette_tokens(&sprite.palette);

        // Analyze regions - collect all token names used as region keys
        let mut all_tokens_used: HashSet<String> = HashSet::new();

        if let Some(regions) = &sprite.regions {
            for token in regions.keys() {
                all_tokens_used.insert(token.clone());
            }
        }

        // Check for missing tokens
        // Region keys are without braces (e.g., "x") while palette tokens have braces (e.g., "{x}")
        if let Some(ref defined_tokens) = palette_tokens {
            for token in &all_tokens_used {
                // Check both with and without braces for compatibility
                let braced_token = format!("{{{}}}", token);
                if !defined_tokens.contains(token) && !defined_tokens.contains(&braced_token) {
                    self.suggest_missing_token(line_number, sprite, token, defined_tokens);
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

    /// Suggest explicit imports for cross-file references in a project context.
    ///
    /// When a sprite references a palette by name and that palette exists in another
    /// file within the project (resolved via ProjectRegistry), suggest adding an
    /// explicit import declaration.
    pub fn suggest_explicit_imports(
        &mut self,
        registry: &ProjectRegistry,
        file_path: &std::path::Path,
        content: &str,
    ) {
        // Parse the content to find sprites and their palette refs
        for (line_idx, line) in content.lines().enumerate() {
            let line_number = line_idx + 1;
            if line.trim().is_empty() {
                continue;
            }

            let obj: TtpObject = match serde_json::from_str(line) {
                Ok(obj) => obj,
                Err(_) => continue,
            };

            if let TtpObject::Sprite(ref sprite) = obj {
                if let PaletteRef::Named(ref palette_name) = sprite.palette {
                    // Skip @include: references (handled separately)
                    if palette_name.starts_with("@include:") || palette_name.starts_with('@') {
                        continue;
                    }

                    // Skip if palette is defined locally in this file
                    if self.palettes.contains_key(palette_name) {
                        continue;
                    }

                    // Check if palette exists in project registry
                    if let Some(_resolved) = registry.resolve_palette_name(palette_name) {
                        // Get the location to construct import path
                        if let Some(canonical) = registry
                            .palette_names()
                            .find(|c| c.ends_with(&format!(":{}", palette_name)))
                        {
                            if let Some(loc) = registry.palette_location(canonical) {
                                // Check this isn't from the same file
                                if loc.source_file != file_path {
                                    let import_json = format!(
                                        r#"{{"type": "import", "from": "{}", "palettes": ["{}"]}}"#,
                                        loc.file_path, palette_name
                                    );

                                    self.report.suggestions.push(Suggestion {
                                        suggestion_type: SuggestionType::AddExplicitImport,
                                        line: line_number,
                                        sprite: sprite.name.clone(),
                                        message: format!(
                                            "Palette \"{}\" is defined in \"{}\". Consider adding an explicit import for cleaner dependency tracking.",
                                            palette_name, loc.file_path
                                        ),
                                        fix: SuggestionFix::AddImport { import_json },
                                    });
                                }
                            }
                        }
                    }
                }
            }
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
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
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
        2 => Some(format!("Did you mean '{}' or '{}'?", suggestions[0], suggestions[1])),
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
        assert_eq!(result, Some("Did you mean 'character' or 'item'?".to_string()));
    }

    #[test]
    fn test_format_suggestion_three() {
        let result = format_suggestion(&["character", "item", "tileset"]);
        assert_eq!(result, Some("Did you mean 'character', 'item', or 'tileset'?".to_string()));
    }

    // Suggester tests

    #[test]
    fn test_suggester_missing_token_typo() {
        let mut suggester = Suggester::new();
        // Palette with skin and hair
        suggester.analyze_line(
            1,
            r##"{"type": "palette", "name": "char", "colors": {"skin": "#FFCC99", "hair": "#8B4513"}}"##,
        );
        // Sprite using skni (typo for skin) as region name
        suggester.analyze_line(
            2,
            r#"{"type": "sprite", "name": "test", "size": [4, 4], "palette": "char", "regions": {"skni": {"rect": [0, 0, 4, 4]}}}"#,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        let missing_token = report.filter_by_type(SuggestionType::MissingToken);
        assert_eq!(missing_token.len(), 1);

        // Should suggest replacing skni with skin
        match &missing_token[0].fix {
            SuggestionFix::ReplaceToken { from, to } => {
                assert_eq!(from, "skni");
                assert_eq!(to, "skin");
            }
            _ => panic!("Expected ReplaceToken fix"),
        }
    }

    #[test]
    fn test_suggester_missing_token_add_to_palette() {
        let mut suggester = Suggester::new();
        // Palette with _ and x - using unknown_color which is very different from both
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "size": [4, 4], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"unknown_color": {"rect": [0, 0, 4, 4]}}}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        let missing_token = report.filter_by_type(SuggestionType::MissingToken);
        assert_eq!(missing_token.len(), 1);

        // unknown_color is too different from _ and x to be a typo, so suggest adding it
        match &missing_token[0].fix {
            SuggestionFix::AddToPalette { token, .. } => {
                assert_eq!(token, "unknown_color");
            }
            _ => panic!("Expected AddToPalette fix"),
        }
    }

    #[test]
    fn test_suggester_no_suggestions_for_valid_sprite() {
        let mut suggester = Suggester::new();
        // Valid sprite with regions - all region names defined in palette
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "valid", "size": [4, 4], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"rect": [0, 0, 4, 4]}}}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(!report.has_suggestions());
    }

    #[test]
    fn test_suggester_inline_palette() {
        let mut suggester = Suggester::new();
        // Sprite with inline palette - region name not in palette
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "size": [4, 4], "palette": {"a": "#FF0000", "b": "#00FF00"}, "regions": {"c": {"rect": [0, 0, 4, 4]}}}"##,
        );

        let report = suggester.into_report();
        assert_eq!(report.sprites_analyzed, 1);
        assert!(report.has_suggestions());

        // c is undefined
        let missing_token = report.filter_by_type(SuggestionType::MissingToken);
        assert_eq!(missing_token.len(), 1);
        assert!(missing_token[0].message.contains("c"));
    }

    #[test]
    fn test_suggester_include_to_import_basic() {
        let mut suggester = Suggester::new();
        // Sprite using @include: reference
        suggester.analyze_line(
            1,
            r#"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "@include:../palettes/brand.pxl", "regions": {"a": {"rect": [0, 0, 4, 4]}}}"#,
        );

        let report = suggester.into_report();
        let include_suggestions = report.filter_by_type(SuggestionType::IncludeToImport);
        assert_eq!(include_suggestions.len(), 1, "Expected 1 include migration suggestion");

        match &include_suggestions[0].fix {
            SuggestionFix::UseImport { include_ref, import_json } => {
                assert!(include_ref.contains("@include:"));
                assert!(import_json.contains("../palettes/brand"));
                assert!(!import_json.contains(".pxl")); // Extension should be stripped
            }
            _ => panic!("Expected UseImport fix"),
        }
    }

    #[test]
    fn test_suggester_include_to_import_with_name() {
        let mut suggester = Suggester::new();
        // Sprite using @include: with #name syntax
        suggester.analyze_line(
            1,
            r#"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "@include:palettes.pxl#gameboy", "regions": {"a": {"rect": [0, 0, 4, 4]}}}"#,
        );

        let report = suggester.into_report();
        let include_suggestions = report.filter_by_type(SuggestionType::IncludeToImport);
        assert_eq!(include_suggestions.len(), 1);

        match &include_suggestions[0].fix {
            SuggestionFix::UseImport { import_json, .. } => {
                assert!(import_json.contains("palettes"));
                assert!(import_json.contains("gameboy"));
            }
            _ => panic!("Expected UseImport fix"),
        }
    }

    #[test]
    fn test_suggester_no_include_suggestion_for_normal_palette() {
        let mut suggester = Suggester::new();
        // Normal palette reference - no include migration needed
        suggester.analyze_line(
            1,
            r##"{"type": "palette", "name": "colors", "colors": {"x": "#FF0000"}}"##,
        );
        suggester.analyze_line(
            2,
            r#"{"type": "sprite", "name": "hero", "size": [4, 4], "palette": "colors", "regions": {"x": {"rect": [0, 0, 4, 4]}}}"#,
        );

        let report = suggester.into_report();
        let include_suggestions = report.filter_by_type(SuggestionType::IncludeToImport);
        assert!(
            include_suggestions.is_empty(),
            "No include migration should be suggested for normal refs"
        );
    }

    #[test]
    fn test_suggestion_report_filter() {
        let mut suggester = Suggester::new();
        // Sprite with missing token in region
        suggester.analyze_line(
            1,
            r##"{"type": "sprite", "name": "test", "size": [4, 4], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"y": {"rect": [0, 0, 4, 4]}}}"##,
        );

        let report = suggester.into_report();

        // Should have missing token
        assert!(report.count_by_type(SuggestionType::MissingToken) > 0);
    }
}
