//! Validation logic for Pixelsrc files
//!
//! Provides semantic validation beyond basic JSON parsing, checking for
//! common mistakes like undefined tokens, row mismatches, and invalid colors.

use crate::color::parse_color;
use crate::models::{Particle, PaletteRef, TtpObject};
use crate::tokenizer::tokenize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Severity of a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARNING"),
        }
    }
}

/// Type of validation issue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueType {
    /// Invalid JSON syntax on a line
    JsonSyntax,
    /// Line is valid JSON but missing the "type" field
    MissingType,
    /// Line has a "type" field but value is not recognized
    UnknownType,
    /// Token used in grid but not defined in palette
    UndefinedToken,
    /// Rows in a sprite have different token counts
    RowLengthMismatch,
    /// Sprite references a palette that doesn't exist
    MissingPalette,
    /// Color value is not valid hex format
    InvalidColor,
    /// Grid dimensions don't match declared size
    SizeMismatch,
    /// Sprite has no grid rows
    EmptyGrid,
    /// Multiple objects with the same name
    DuplicateName,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::JsonSyntax => write!(f, "json_syntax"),
            IssueType::MissingType => write!(f, "missing_type"),
            IssueType::UnknownType => write!(f, "unknown_type"),
            IssueType::UndefinedToken => write!(f, "undefined_token"),
            IssueType::RowLengthMismatch => write!(f, "row_length"),
            IssueType::MissingPalette => write!(f, "missing_palette"),
            IssueType::InvalidColor => write!(f, "invalid_color"),
            IssueType::SizeMismatch => write!(f, "size_mismatch"),
            IssueType::EmptyGrid => write!(f, "empty_grid"),
            IssueType::DuplicateName => write!(f, "duplicate_name"),
        }
    }
}

/// A validation issue found in the input
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Line number (1-indexed) where the issue was found
    pub line: usize,
    /// Severity of the issue
    pub severity: Severity,
    /// Type of issue
    pub issue_type: IssueType,
    /// Human-readable message describing the issue
    pub message: String,
    /// Optional suggestion for fixing the issue (e.g., "did you mean?")
    pub suggestion: Option<String>,
    /// Additional context (e.g., sprite name, palette name)
    pub context: Option<String>,
}

impl ValidationIssue {
    /// Create a new error
    pub fn error(line: usize, issue_type: IssueType, message: impl Into<String>) -> Self {
        Self {
            line,
            severity: Severity::Error,
            issue_type,
            message: message.into(),
            suggestion: None,
            context: None,
        }
    }

    /// Create a new warning
    pub fn warning(line: usize, issue_type: IssueType, message: impl Into<String>) -> Self {
        Self {
            line,
            severity: Severity::Warning,
            issue_type,
            message: message.into(),
            suggestion: None,
            context: None,
        }
    }

    /// Add a suggestion to this issue
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add context to this issue
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Validator for Pixelsrc files
pub struct Validator {
    /// Collected validation issues
    issues: Vec<ValidationIssue>,
    /// Known palette names -> set of defined tokens
    palettes: HashMap<String, HashSet<String>>,
    /// Built-in palette names
    builtin_palettes: HashSet<String>,
    /// Known sprite names (for duplicate detection)
    sprite_names: HashSet<String>,
    /// Known animation names
    animation_names: HashSet<String>,
    /// Known composition names
    composition_names: HashSet<String>,
    /// Known variant names
    variant_names: HashSet<String>,
    /// Known palette names (for duplicate detection)
    palette_names: HashSet<String>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        // Initialize with built-in palette names
        let builtin_palettes: HashSet<String> = crate::palettes::list_builtins()
            .into_iter()
            .map(|s| format!("@{}", s))
            .collect();

        Self {
            issues: Vec::new(),
            palettes: HashMap::new(),
            builtin_palettes,
            sprite_names: HashSet::new(),
            animation_names: HashSet::new(),
            composition_names: HashSet::new(),
            variant_names: HashSet::new(),
            palette_names: HashSet::new(),
        }
    }

    /// Validate a single line of input
    pub fn validate_line(&mut self, line_number: usize, content: &str) {
        // Skip empty lines
        if content.trim().is_empty() {
            return;
        }

        // Check 1: JSON syntax
        let json_value: Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                self.issues.push(ValidationIssue::error(
                    line_number,
                    IssueType::JsonSyntax,
                    format!("Invalid JSON: {}", e),
                ));
                return;
            }
        };

        // Check 2: Missing type field
        let obj = match json_value.as_object() {
            Some(obj) => obj,
            None => {
                self.issues.push(ValidationIssue::error(
                    line_number,
                    IssueType::JsonSyntax,
                    "Line must be a JSON object",
                ));
                return;
            }
        };

        let type_value = match obj.get("type") {
            Some(t) => t,
            None => {
                self.issues.push(ValidationIssue::error(
                    line_number,
                    IssueType::MissingType,
                    "Missing required \"type\" field",
                ));
                return;
            }
        };

        let type_str = match type_value.as_str() {
            Some(s) => s,
            None => {
                self.issues.push(ValidationIssue::error(
                    line_number,
                    IssueType::MissingType,
                    "\"type\" field must be a string",
                ));
                return;
            }
        };

        // Check 3: Unknown type
        let valid_types = ["palette", "sprite", "animation", "composition", "variant"];
        if !valid_types.contains(&type_str) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::UnknownType,
                    format!("Unknown type \"{}\"", type_str),
                )
                .with_suggestion(format!("Valid types are: {}", valid_types.join(", "))),
            );
            return;
        }

        // Now parse as TtpObject for semantic validation
        let ttp_obj: TtpObject = match serde_json::from_str(content) {
            Ok(obj) => obj,
            Err(e) => {
                // This shouldn't happen if type is valid, but handle gracefully
                self.issues.push(ValidationIssue::error(
                    line_number,
                    IssueType::JsonSyntax,
                    format!("Failed to parse {}: {}", type_str, e),
                ));
                return;
            }
        };

        // Validate based on object type
        match ttp_obj {
            TtpObject::Palette(palette) => {
                self.validate_palette(line_number, &palette.name, &palette.colors);
            }
            TtpObject::Sprite(sprite) => {
                self.validate_sprite(line_number, &sprite);
            }
            TtpObject::Animation(animation) => {
                self.validate_animation(line_number, &animation.name);
            }
            TtpObject::Composition(composition) => {
                self.validate_composition(line_number, &composition.name);
            }
            TtpObject::Variant(variant) => {
                self.validate_variant(line_number, &variant.name, &variant.palette);
            }
            TtpObject::Particle(particle) => {
                self.validate_particle(line_number, &particle);
            }
        }
    }

    /// Validate a palette definition
    fn validate_palette(
        &mut self,
        line_number: usize,
        name: &str,
        colors: &HashMap<String, String>,
    ) {
        // Check for duplicate name
        if !self.palette_names.insert(name.to_string()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate palette name \"{}\"", name),
                )
                .with_context(format!("palette \"{}\"", name)),
            );
        }

        // Validate each color
        let mut defined_tokens = HashSet::new();
        for (token, color) in colors {
            defined_tokens.insert(token.clone());

            // Check color format
            if let Err(e) = parse_color(color) {
                self.issues.push(
                    ValidationIssue::error(
                        line_number,
                        IssueType::InvalidColor,
                        format!("Invalid color \"{}\" for token {}: {}", color, token, e),
                    )
                    .with_context(format!("palette \"{}\"", name)),
                );
            }
        }

        // Register palette tokens
        self.palettes.insert(name.to_string(), defined_tokens);
    }

    /// Validate a sprite definition
    fn validate_sprite(&mut self, line_number: usize, sprite: &crate::models::Sprite) {
        let name = &sprite.name;

        // Check for duplicate name
        if !self.sprite_names.insert(name.to_string()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate sprite name \"{}\"", name),
                )
                .with_context(format!("sprite \"{}\"", name)),
            );
        }

        // Check for empty grid
        if sprite.grid.is_empty() {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::EmptyGrid,
                    format!("Sprite \"{}\" has no grid rows", name),
                )
                .with_context(format!("sprite \"{}\"", name)),
            );
            return;
        }

        // Get palette tokens
        let palette_tokens = self.get_palette_tokens(&sprite.palette, line_number, name);

        // Validate grid rows
        let mut first_row_count: Option<usize> = None;
        let mut all_tokens_used: HashSet<String> = HashSet::new();

        for (row_idx, row) in sprite.grid.iter().enumerate() {
            let (tokens, _warnings) = tokenize(row);

            // Check row length consistency
            match first_row_count {
                None => first_row_count = Some(tokens.len()),
                Some(expected) if tokens.len() != expected => {
                    self.issues.push(
                        ValidationIssue::warning(
                            line_number,
                            IssueType::RowLengthMismatch,
                            format!(
                                "Row {} has {} tokens, expected {} (row 1 has {})",
                                row_idx + 1,
                                tokens.len(),
                                expected,
                                expected
                            ),
                        )
                        .with_context(format!("sprite \"{}\"", name)),
                    );
                }
                _ => {}
            }

            // Collect all tokens used
            for token in tokens {
                all_tokens_used.insert(token);
            }
        }

        // Check size mismatch
        if let Some(declared_size) = sprite.size {
            let actual_width = first_row_count.unwrap_or(0) as u32;
            let actual_height = sprite.grid.len() as u32;

            if declared_size[0] != actual_width || declared_size[1] != actual_height {
                self.issues.push(
                    ValidationIssue::warning(
                        line_number,
                        IssueType::SizeMismatch,
                        format!(
                            "Declared size [{}x{}] doesn't match grid [{}x{}]",
                            declared_size[0], declared_size[1], actual_width, actual_height
                        ),
                    )
                    .with_context(format!("sprite \"{}\"", name)),
                );
            }
        }

        // Check for undefined tokens (only if we have palette info)
        if let Some(ref defined_tokens) = palette_tokens {
            for token in &all_tokens_used {
                if !defined_tokens.contains(token) {
                    let mut issue = ValidationIssue::warning(
                        line_number,
                        IssueType::UndefinedToken,
                        format!("Undefined token {}", token),
                    )
                    .with_context(format!("sprite \"{}\"", name));

                    // Try to suggest a correction
                    let known: Vec<&str> = defined_tokens.iter().map(|s| s.as_str()).collect();
                    if let Some(suggestion) = suggest_token(token, &known) {
                        issue = issue.with_suggestion(format!("did you mean {}?", suggestion));
                    }

                    self.issues.push(issue);
                }
            }
        }
    }

    /// Get tokens defined in a palette reference
    fn get_palette_tokens(
        &mut self,
        palette_ref: &PaletteRef,
        line_number: usize,
        sprite_name: &str,
    ) -> Option<HashSet<String>> {
        match palette_ref {
            PaletteRef::Named(name) => {
                // Check for @include: syntax
                if name.starts_with("@include:") {
                    // Include files are not validated here
                    return None;
                }

                // Check for built-in palettes
                if self.builtin_palettes.contains(name) {
                    // Get tokens from built-in palette
                    let palette_name = name.strip_prefix('@').unwrap_or(name);
                    if let Some(palette) = crate::palettes::get_builtin(palette_name) {
                        return Some(palette.colors.keys().cloned().collect());
                    }
                    return None;
                }

                // Check if palette is defined
                if let Some(tokens) = self.palettes.get(name) {
                    return Some(tokens.clone());
                }

                // Palette not found
                self.issues.push(
                    ValidationIssue::warning(
                        line_number,
                        IssueType::MissingPalette,
                        format!("Palette \"{}\" not defined", name),
                    )
                    .with_context(format!("sprite \"{}\"", sprite_name)),
                );
                None
            }
            PaletteRef::Inline(colors) => {
                // Validate inline palette colors
                for (token, color) in colors {
                    if let Err(e) = parse_color(color) {
                        self.issues.push(
                            ValidationIssue::error(
                                line_number,
                                IssueType::InvalidColor,
                                format!("Invalid color \"{}\" for token {}: {}", color, token, e),
                            )
                            .with_context(format!("sprite \"{}\" inline palette", sprite_name)),
                        );
                    }
                }
                Some(colors.keys().cloned().collect())
            }
        }
    }

    /// Validate an animation definition
    fn validate_animation(&mut self, line_number: usize, name: &str) {
        // Check for duplicate name
        if !self.animation_names.insert(name.to_string()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate animation name \"{}\"", name),
                )
                .with_context(format!("animation \"{}\"", name)),
            );
        }
    }

    /// Validate a composition definition
    fn validate_composition(&mut self, line_number: usize, name: &str) {
        // Check for duplicate name
        if !self.composition_names.insert(name.to_string()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate composition name \"{}\"", name),
                )
                .with_context(format!("composition \"{}\"", name)),
            );
        }
    }

    /// Validate a variant definition
    fn validate_variant(
        &mut self,
        line_number: usize,
        name: &str,
        palette: &HashMap<String, String>,
    ) {
        // Check for duplicate name
        if !self.variant_names.insert(name.to_string()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate variant name \"{}\"", name),
                )
                .with_context(format!("variant \"{}\"", name)),
            );
        }

        // Validate palette override colors
        for (token, color) in palette {
            if let Err(e) = parse_color(color) {
                self.issues.push(
                    ValidationIssue::error(
                        line_number,
                        IssueType::InvalidColor,
                        format!("Invalid color \"{}\" for token {}: {}", color, token, e),
                    )
                    .with_context(format!("variant \"{}\"", name)),
                );
            }
        }
    }

    /// Validate a particle system definition
    fn validate_particle(&mut self, line_number: usize, particle: &Particle) {
        // Check for empty name
        if particle.name.is_empty() {
            self.issues.push(
                ValidationIssue::error(
                    line_number,
                    IssueType::DuplicateName, // Reusing for empty name validation
                    "Particle system has empty name".to_string(),
                )
                .with_context("particle".to_string()),
            );
        }

        // Check for empty sprite reference
        if particle.sprite.is_empty() {
            self.issues.push(
                ValidationIssue::error(
                    line_number,
                    IssueType::MissingPalette, // Reusing for missing sprite reference
                    "Particle system has empty sprite reference".to_string(),
                )
                .with_context(format!("particle \"{}\"", particle.name)),
            );
        }

        // Validate emitter lifetime range
        if particle.emitter.lifetime[0] > particle.emitter.lifetime[1] {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::SizeMismatch, // Reusing for range validation
                    format!(
                        "Particle lifetime min ({}) > max ({})",
                        particle.emitter.lifetime[0], particle.emitter.lifetime[1]
                    ),
                )
                .with_context(format!("particle \"{}\"", particle.name)),
            );
        }
    }

    /// Validate a file
    pub fn validate_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_idx, line_result) in reader.lines().enumerate() {
            let line_number = line_idx + 1;
            match line_result {
                Ok(line) => self.validate_line(line_number, &line),
                Err(e) => {
                    self.issues.push(ValidationIssue::error(
                        line_number,
                        IssueType::JsonSyntax,
                        format!("IO error reading line: {}", e),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get all collected issues
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Consume the validator and return all issues
    pub fn into_issues(self) -> Vec<ValidationIssue> {
        self.issues
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Error))
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Warning))
    }

    /// Count errors
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .count()
    }

    /// Count warnings
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Warning))
            .count()
    }
}

/// Suggest a similar token using Levenshtein distance
pub fn suggest_token(unknown: &str, known: &[&str]) -> Option<String> {
    // Only consider tokens with distance <= 2
    const MAX_DISTANCE: usize = 2;

    let mut best_match: Option<(&str, usize)> = None;

    for candidate in known {
        let distance = levenshtein_distance(unknown, candidate);
        if distance <= MAX_DISTANCE {
            match best_match {
                None => best_match = Some((candidate, distance)),
                Some((_, best_dist)) if distance < best_dist => {
                    best_match = Some((candidate, distance))
                }
                _ => {}
            }
        }
    }

    best_match.map(|(s, _)| s.to_string())
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Quick checks
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // DP table
    let mut dp = vec![vec![0usize; b_len + 1]; a_len + 1];

    // Initialize base cases
    for i in 0..=a_len {
        dp[i][0] = i;
    }
    for j in 0..=b_len {
        dp[0][j] = j;
    }

    // Fill table
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1) // deletion
                .min(dp[i][j - 1] + 1) // insertion
                .min(dp[i - 1][j - 1] + cost); // substitution
        }
    }

    dp[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("test", "test"), 0);
        assert_eq!(levenshtein_distance("{skin}", "{skin}"), 0);
    }

    #[test]
    fn test_levenshtein_one_char_diff() {
        assert_eq!(levenshtein_distance("{skni}", "{skin}"), 2); // transposition = 2 ops
        assert_eq!(levenshtein_distance("{hiar}", "{hair}"), 2); // transposition = 2 ops
        assert_eq!(levenshtein_distance("{skinx}", "{skin}"), 1); // deletion
        assert_eq!(levenshtein_distance("{skin}", "{skinx}"), 1); // insertion
    }

    #[test]
    fn test_levenshtein_distant() {
        assert!(levenshtein_distance("{xyz}", "{abc}") > 2);
        assert!(levenshtein_distance("{completely}", "{different}") > 2);
    }

    #[test]
    fn test_suggest_token_typo() {
        let known = vec!["{skin}", "{hair}", "{outline}"];
        assert_eq!(suggest_token("{skni}", &known), Some("{skin}".to_string()));
        assert_eq!(suggest_token("{hiar}", &known), Some("{hair}".to_string()));
    }

    #[test]
    fn test_suggest_token_no_match() {
        let known = vec!["{skin}", "{hair}"];
        assert_eq!(suggest_token("{xyz123456}", &known), None);
    }

    #[test]
    fn test_validate_valid_json() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        assert!(validator.issues().is_empty());
    }

    #[test]
    fn test_validate_invalid_json() {
        let mut validator = Validator::new();
        validator.validate_line(1, "{not valid json}");
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::JsonSyntax);
        assert!(validator.has_errors());
    }

    #[test]
    fn test_validate_missing_type() {
        let mut validator = Validator::new();
        validator.validate_line(1, r#"{"name": "test"}"#);
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::MissingType);
    }

    #[test]
    fn test_validate_unknown_type() {
        let mut validator = Validator::new();
        validator.validate_line(1, r#"{"type": "unknown", "name": "test"}"#);
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::UnknownType);
        assert!(validator.has_warnings());
    }

    #[test]
    fn test_validate_invalid_color() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#GGG"}}"##,
        );
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::InvalidColor);
    }

    #[test]
    fn test_validate_undefined_token() {
        let mut validator = Validator::new();
        // First define a palette
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        // Then a sprite using undefined token
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": ["{a}{b}"]}"#,
        );
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::UndefinedToken);
        assert_eq!(validator.issues()[0].line, 2);
    }

    #[test]
    fn test_validate_row_length_mismatch() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": ["{a}{a}{a}{a}", "{a}{a}{a}"]}"#,
        );

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert_eq!(row_mismatch_issues.len(), 1);
    }

    #[test]
    fn test_validate_size_mismatch() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "size": [10, 10], "palette": "test", "grid": ["{a}{a}"]}"#,
        );

        let size_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::SizeMismatch)
            .collect();
        assert_eq!(size_mismatch_issues.len(), 1);
    }

    #[test]
    fn test_validate_empty_grid() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": []}"#,
        );

        let empty_grid_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::EmptyGrid)
            .collect();
        assert_eq!(empty_grid_issues.len(), 1);
    }

    #[test]
    fn test_validate_duplicate_name() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r##"{"type": "palette", "name": "test", "colors": {"{b}": "#00FF00"}}"##,
        );

        let duplicate_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::DuplicateName)
            .collect();
        assert_eq!(duplicate_issues.len(), 1);
    }

    #[test]
    fn test_validate_missing_palette() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r#"{"type": "sprite", "name": "test", "palette": "nonexistent", "grid": ["{a}"]}"#,
        );

        let missing_palette_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::MissingPalette)
            .collect();
        assert_eq!(missing_palette_issues.len(), 1);
    }

    #[test]
    fn test_validate_inline_palette() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "sprite", "name": "test", "palette": {"{a}": "#FF0000"}, "grid": ["{a}"]}"##,
        );
        assert!(validator.issues().is_empty());
    }

    #[test]
    fn test_validate_inline_palette_invalid_color() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "sprite", "name": "test", "palette": {"{a}": "#INVALID"}, "grid": ["{a}"]}"##,
        );
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::InvalidColor);
    }

    #[test]
    fn test_validate_file_errors() {
        use std::path::Path;

        let fixture_path = Path::new("tests/fixtures/invalid/validate_errors.jsonl");
        if !fixture_path.exists() {
            return; // Skip if fixture not available
        }

        let mut validator = Validator::new();
        validator.validate_file(fixture_path).unwrap();

        // Should have warnings for undefined token {b} and row length mismatch
        let undefined_token_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::UndefinedToken)
            .collect();
        assert!(
            !undefined_token_issues.is_empty(),
            "Expected undefined token warning for {{b}}"
        );

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert!(
            !row_mismatch_issues.is_empty(),
            "Expected row length mismatch warning"
        );
    }

    #[test]
    fn test_validate_file_typos() {
        use std::path::Path;

        let fixture_path = Path::new("tests/fixtures/invalid/validate_typo.jsonl");
        if !fixture_path.exists() {
            return; // Skip if fixture not available
        }

        let mut validator = Validator::new();
        validator.validate_file(fixture_path).unwrap();

        // Should have warnings for undefined tokens with suggestions
        let undefined_token_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::UndefinedToken)
            .collect();

        // Should find {skni} and {hiar} as undefined
        assert_eq!(
            undefined_token_issues.len(),
            2,
            "Expected 2 undefined token warnings"
        );

        // Check that suggestions are provided
        let has_skin_suggestion = undefined_token_issues.iter().any(|i| {
            i.suggestion
                .as_ref()
                .map_or(false, |s| s.contains("{skin}"))
        });
        let has_hair_suggestion = undefined_token_issues.iter().any(|i| {
            i.suggestion
                .as_ref()
                .map_or(false, |s| s.contains("{hair}"))
        });

        assert!(has_skin_suggestion, "Expected suggestion for {{skin}}");
        assert!(has_hair_suggestion, "Expected suggestion for {{hair}}");
    }
}
