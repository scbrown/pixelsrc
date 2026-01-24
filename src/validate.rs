//! Validation logic for Pixelsrc files
//!
//! Provides semantic validation beyond basic JSON parsing, checking for
//! common mistakes like undefined tokens and invalid colors.

use crate::color::parse_color;
use crate::models::{Palette, PaletteRef, Particle, Relationship, RelationshipType, TtpObject};
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
    /// Token used in region but not defined in palette
    UndefinedToken,
    /// Sprite references a palette that doesn't exist
    MissingPalette,
    /// Color value is not valid hex format
    InvalidColor,
    /// Multiple objects with the same name
    DuplicateName,
    /// Role references a token not defined in palette colors
    InvalidRoleToken,
    /// Relationship references a token that doesn't exist
    InvalidRelationshipTarget,
    /// Circular dependency in derives-from relationships
    CircularDependency,
    /// Region 'within' constraint references non-existent token
    InvalidWithinReference,
    /// Region 'adjacent-to' constraint references non-existent token
    InvalidAdjacentReference,
    /// Relationship references a non-existent token
    InvalidRelationshipReference,
    /// Circular relationship detected (e.g., A within B, B within A)
    CircularRelationship,
    /// Constraint validation is uncertain (e.g., overlapping regions)
    UncertainConstraint,
    /// Range validation (reused for constraint checking)
    RangeValidation,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::JsonSyntax => write!(f, "json_syntax"),
            IssueType::MissingType => write!(f, "missing_type"),
            IssueType::UnknownType => write!(f, "unknown_type"),
            IssueType::UndefinedToken => write!(f, "undefined_token"),
            IssueType::MissingPalette => write!(f, "missing_palette"),
            IssueType::InvalidColor => write!(f, "invalid_color"),
            IssueType::DuplicateName => write!(f, "duplicate_name"),
            IssueType::InvalidRoleToken => write!(f, "invalid_role_token"),
            IssueType::InvalidRelationshipTarget => write!(f, "invalid_relationship_target"),
            IssueType::CircularDependency => write!(f, "circular_dependency"),
            IssueType::InvalidWithinReference => write!(f, "invalid_within_ref"),
            IssueType::InvalidAdjacentReference => write!(f, "invalid_adjacent_ref"),
            IssueType::InvalidRelationshipReference => write!(f, "invalid_relationship_ref"),
            IssueType::CircularRelationship => write!(f, "circular_relationship"),
            IssueType::UncertainConstraint => write!(f, "uncertain_constraint"),
            IssueType::RangeValidation => write!(f, "range_validation"),
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
        let builtin_palettes: HashSet<String> =
            crate::palettes::list_builtins().into_iter().map(|s| format!("@{}", s)).collect();

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

        // Check 1: JSON5 syntax
        let json_value: Value = match json5::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                self.issues.push(ValidationIssue::error(
                    line_number,
                    IssueType::JsonSyntax,
                    format!("Invalid JSON5: {}", e),
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
        let ttp_obj: TtpObject = match json5::from_str(content) {
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
                self.validate_palette(line_number, &palette);
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
            TtpObject::Transform(transform) => {
                self.validate_transform(line_number, &transform);
            }
            TtpObject::StateRules(state_rules) => {
                self.validate_state_rules(line_number, &state_rules);
            }
        }
    }

    /// Validate a user-defined transform
    fn validate_transform(&mut self, line_number: usize, transform: &crate::models::TransformDef) {
        // Check for duplicate name - transforms share namespace with other named objects
        if !self.sprite_names.insert(transform.name.clone()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate transform name \"{}\"", transform.name),
                )
                .with_context(format!("transform \"{}\"", transform.name)),
            );
        }

        // Validate keyframe frames if animation
        if let Some(frames) = transform.frames {
            if frames == 0 {
                self.issues.push(
                    ValidationIssue::warning(
                        line_number,
                        IssueType::RangeValidation,
                        "Transform has 0 frames".to_string(),
                    )
                    .with_context(format!("transform \"{}\"", transform.name)),
                );
            }
        }
    }

    /// Validate a state rules definition
    fn validate_state_rules(
        &mut self,
        line_number: usize,
        state_rules: &crate::state::StateRules,
    ) {
        // Check for duplicate name - state rules share namespace with other named objects
        if !self.sprite_names.insert(state_rules.name.clone()) {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::DuplicateName,
                    format!("Duplicate state-rules name \"{}\"", state_rules.name),
                )
                .with_context(format!("state-rules \"{}\"", state_rules.name)),
            );
        }

        // Warn if no rules defined
        if state_rules.rules.is_empty() {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::EmptyGrid,
                    "State rules has no rules defined".to_string(),
                )
                .with_context(format!("state-rules \"{}\"", state_rules.name)),
            );
        }
    }

    /// Validate a palette definition
    fn validate_palette(&mut self, line_number: usize, palette: &Palette) {
        let name = &palette.name;
        let colors = &palette.colors;
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

        // Validate role token references
        if let Some(roles) = &palette.roles {
            for (token, role) in roles {
                if !defined_tokens.contains(token) {
                    self.issues.push(
                        ValidationIssue::error(
                            line_number,
                            IssueType::InvalidRoleToken,
                            format!(
                                "Role \"{}\" references undefined token {}",
                                role, token
                            ),
                        )
                        .with_context(format!("palette \"{}\"", name)),
                    );
                }
            }
        }

        // Validate relationships
        if let Some(rels) = palette.relationships.as_ref() {
            self.validate_relationships(line_number, name, &defined_tokens, rels);
        }

        // Register palette tokens
        self.palettes.insert(name.to_string(), defined_tokens);
    }

    /// Validate palette relationships
    fn validate_relationships(
        &mut self,
        line_number: usize,
        palette_name: &str,
        defined_tokens: &HashSet<String>,
        relationships: &HashMap<String, Relationship>,
    ) {
        // Build derives-from graph for cycle detection
        let mut derives_from: HashMap<&str, &str> = HashMap::new();

        for (source_token, relationship) in relationships {
            // Check that source token exists
            if !defined_tokens.contains(source_token) {
                self.issues.push(
                    ValidationIssue::error(
                        line_number,
                        IssueType::InvalidRelationshipTarget,
                        format!(
                            "Relationship source token {} is not defined in palette",
                            source_token
                        ),
                    )
                    .with_context(format!("palette \"{}\"", palette_name)),
                );
            }

            // Check that target token exists
            if !defined_tokens.contains(&relationship.target) {
                self.issues.push(
                    ValidationIssue::error(
                        line_number,
                        IssueType::InvalidRelationshipTarget,
                        format!(
                            "Relationship target token {} is not defined in palette",
                            relationship.target
                        ),
                    )
                    .with_context(format!("palette \"{}\"", palette_name)),
                );
            }

            // Collect derives-from edges for cycle detection
            if relationship.relationship_type == RelationshipType::DerivesFrom {
                derives_from.insert(source_token.as_str(), relationship.target.as_str());
            }
        }

        // Check for circular dependencies in derives-from chains
        for start_token in derives_from.keys() {
            let mut visited = HashSet::new();
            let mut current = *start_token;

            while let Some(&next) = derives_from.get(current) {
                if !visited.insert(current) {
                    // We've seen this token before - cycle detected
                    self.issues.push(
                        ValidationIssue::error(
                            line_number,
                            IssueType::CircularDependency,
                            format!(
                                "Circular dependency in derives-from chain involving token {}",
                                current
                            ),
                        )
                        .with_context(format!("palette \"{}\"", palette_name)),
                    );
                    break;
                }
                current = next;
            }
        }
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

        // Get palette tokens for validation
        let palette_tokens = self.get_palette_tokens(&sprite.palette, line_number, name);

        // Validate sprites have regions defined
        if sprite.regions.is_none() {
            self.issues.push(
                ValidationIssue::warning(
                    line_number,
                    IssueType::EmptyGrid,
                    format!("Sprite \"{}\" has no regions defined", name),
                )
                .with_context(format!("sprite \"{}\"", name))
                .with_suggestion("add structured regions format to the sprite".to_string()),
            );
        }

        // Collect all tokens used in regions
        let mut all_tokens_used: HashSet<String> = HashSet::new();
        if let Some(regions) = &sprite.regions {
            for token in regions.keys() {
                all_tokens_used.insert(token.clone());
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
                    IssueType::RangeValidation,
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
    ///
    /// Supports both single-line JSONL and multi-line JSON5 formats.
    pub fn validate_file(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut accumulator = String::new();
        let mut start_line = 1;
        let mut current_line = 0;
        let mut brace_depth = 0;
        let mut bracket_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut in_single_line_comment = false;
        let mut in_multi_line_comment = false;
        let mut prev_char: Option<char> = None;

        for line_result in reader.lines() {
            current_line += 1;

            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    self.issues.push(ValidationIssue::error(
                        current_line,
                        IssueType::JsonSyntax,
                        format!("IO error reading line: {}", e),
                    ));
                    continue;
                }
            };

            // Reset single-line comment flag at start of new line
            in_single_line_comment = false;

            // Skip empty lines when not accumulating
            if accumulator.is_empty() && line.trim().is_empty() {
                continue;
            }

            // Skip standalone comment lines when not accumulating
            if accumulator.is_empty() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    continue;
                }
            }

            // Start tracking from this line if accumulator was empty
            if accumulator.is_empty() {
                start_line = current_line;
            }

            // Add line to accumulator
            if !accumulator.is_empty() {
                accumulator.push('\n');
            }
            accumulator.push_str(&line);

            // Track brace/bracket depth to detect complete objects
            for ch in line.chars() {
                // Handle multi-line comment end
                if in_multi_line_comment {
                    if prev_char == Some('*') && ch == '/' {
                        in_multi_line_comment = false;
                    }
                    prev_char = Some(ch);
                    continue;
                }

                // Check for comment starts
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

                // Skip if in comment
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

            // Reset prev_char at end of line (for comment detection across lines)
            prev_char = None;

            // Try to validate when braces are balanced
            if brace_depth == 0 && bracket_depth == 0 && !accumulator.trim().is_empty() {
                self.validate_line(start_line, &accumulator);

                accumulator.clear();
                in_string = false;
                escape_next = false;
                in_single_line_comment = false;
            }
        }

        // Handle any remaining accumulated content
        if !accumulator.trim().is_empty() {
            self.validate_line(start_line, &accumulator);
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
        self.issues.iter().any(|i| matches!(i.severity, Severity::Error))
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|i| matches!(i.severity, Severity::Warning))
    }

    /// Count errors
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| matches!(i.severity, Severity::Error)).count()
    }

    /// Count warnings
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| matches!(i.severity, Severity::Warning)).count()
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
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
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
    use serial_test::serial;

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
    fn test_validate_no_regions() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test"}"#,
        );

        let empty_issues: Vec<_> =
            validator.issues().iter().filter(|i| i.issue_type == IssueType::EmptyGrid).collect();
        assert_eq!(empty_issues.len(), 1);
    }

    #[test]
    fn test_validate_undefined_token() {
        let mut validator = Validator::new();
        // First define a palette
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"a": "#FF0000"}}"##,
        );
        // Then a sprite using undefined token (b is not in palette)
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "size": [4, 4], "palette": "test", "regions": {"b": {"rect": [0, 0, 4, 4]}}}"#,
        );
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::UndefinedToken);
        assert_eq!(validator.issues()[0].line, 2);
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
            r#"{"type": "sprite", "name": "test", "size": [4, 4], "palette": "nonexistent", "regions": {"a": {"rect": [0, 0, 4, 4]}}}"#,
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
            r##"{"type": "sprite", "name": "test", "size": [4, 4], "palette": {"a": "#FF0000"}, "regions": {"a": {"rect": [0, 0, 4, 4]}}}"##,
        );
        assert!(validator.issues().is_empty());
    }

    #[test]
    fn test_validate_inline_palette_invalid_color() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "sprite", "name": "test", "size": [4, 4], "palette": {"a": "#INVALID"}, "regions": {"a": {"rect": [0, 0, 4, 4]}}}"##,
        );
        assert_eq!(validator.issues().len(), 1);
        assert_eq!(validator.issues()[0].issue_type, IssueType::InvalidColor);
    }

    #[test]
    #[serial]
    #[ignore = "Grid format deprecated"]
    fn test_validate_file_errors() {
        use std::path::Path;

        let fixture_path = Path::new("tests/fixtures/invalid/validate_errors.jsonl");
        if !fixture_path.exists() {
            return; // Skip if fixture not available
        }

        let mut validator = Validator::new();
        validator.validate_file(fixture_path).unwrap();

        // Should have warnings for undefined tokens
        let undefined_token_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::UndefinedToken)
            .collect();
        assert!(!undefined_token_issues.is_empty(), "Expected undefined token warning");
    }

    #[test]
    #[serial]
    #[ignore = "Grid format deprecated"]
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
        assert_eq!(undefined_token_issues.len(), 2, "Expected 2 undefined token warnings");

        // Check that suggestions are provided
        let has_skin_suggestion = undefined_token_issues
            .iter()
            .any(|i| i.suggestion.as_ref().is_some_and(|s| s.contains("{skin}")));
        let has_hair_suggestion = undefined_token_issues
            .iter()
            .any(|i| i.suggestion.as_ref().is_some_and(|s| s.contains("{hair}")));

        assert!(has_skin_suggestion, "Expected suggestion for {{skin}}");
        assert!(has_hair_suggestion, "Expected suggestion for {{hair}}");
    }

    #[test]
    fn test_validate_palette_with_valid_roles() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00"}, "roles": {"{a}": "boundary", "{b}": "fill"}}"##,
        );
        assert!(validator.issues().is_empty(), "Valid roles should not produce issues");
    }

    #[test]
    fn test_validate_palette_with_invalid_role_token() {
        let mut validator = Validator::new();
        // Role references {c} which is not defined in colors
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}, "roles": {"{c}": "boundary"}}"##,
        );

        let invalid_role_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRoleToken)
            .collect();
        assert_eq!(invalid_role_issues.len(), 1);
        assert!(invalid_role_issues[0].message.contains("{c}"));
        assert!(invalid_role_issues[0].message.contains("boundary"));
    }

    #[test]
    fn test_validate_palette_with_multiple_invalid_role_tokens() {
        let mut validator = Validator::new();
        // Roles reference {b} and {c} which are not defined in colors
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}, "roles": {"{b}": "shadow", "{c}": "highlight"}}"##,
        );

        let invalid_role_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRoleToken)
            .collect();
        assert_eq!(invalid_role_issues.len(), 2);
    }

    #[test]
    fn test_validate_palette_with_all_role_types() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00", "{c}": "#0000FF", "{d}": "#FFFF00", "{e}": "#FF00FF"}, "roles": {"{a}": "boundary", "{b}": "anchor", "{c}": "fill", "{d}": "shadow", "{e}": "highlight"}}"##,
        );
        assert!(validator.issues().is_empty(), "All valid role types should be accepted");
    }

    #[test]
    fn test_validate_relationship_valid() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{base}": "#FF0000", "{shadow}": "#AA0000"}, "relationships": {"{shadow}": {"type": "derives-from", "target": "{base}"}}}"##,
        );
        assert!(validator.issues().is_empty(), "Expected no issues for valid relationship");
    }

    #[test]
    fn test_validate_relationship_invalid_source() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{base}": "#FF0000"}, "relationships": {"{undefined}": {"type": "derives-from", "target": "{base}"}}}"##,
        );
        let issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRelationshipTarget)
            .collect();
        assert_eq!(issues.len(), 1, "Expected 1 invalid relationship target issue");
        assert!(issues[0].message.contains("{undefined}"));
    }

    #[test]
    fn test_validate_relationship_invalid_target() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{shadow}": "#AA0000"}, "relationships": {"{shadow}": {"type": "derives-from", "target": "{undefined}"}}}"##,
        );
        let issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRelationshipTarget)
            .collect();
        assert_eq!(issues.len(), 1, "Expected 1 invalid relationship target issue");
        assert!(issues[0].message.contains("{undefined}"));
    }

    #[test]
    fn test_validate_relationship_circular_dependency() {
        let mut validator = Validator::new();
        // Create a cycle: a -> b -> c -> a
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00", "{c}": "#0000FF"}, "relationships": {"{a}": {"type": "derives-from", "target": "{b}"}, "{b}": {"type": "derives-from", "target": "{c}"}, "{c}": {"type": "derives-from", "target": "{a}"}}}"##,
        );
        let issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularDependency)
            .collect();
        assert!(!issues.is_empty(), "Expected circular dependency issue");
    }

    #[test]
    fn test_validate_relationship_types() {
        let mut validator = Validator::new();
        // Test all relationship types
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00", "{c}": "#0000FF", "{d}": "#FFFF00", "{e}": "#FF00FF"}, "relationships": {"{a}": {"type": "derives-from", "target": "{b}"}, "{b}": {"type": "contained-within", "target": "{c}"}, "{c}": {"type": "adjacent-to", "target": "{d}"}, "{d}": {"type": "paired-with", "target": "{e}"}}}"##,
        );
        assert!(validator.issues().is_empty(), "Expected no issues for valid relationship types");
    }

    #[test]
    fn test_validate_relationship_invalid_type() {
        let mut validator = Validator::new();
        // Invalid relationship type should fail JSON parsing
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00"}, "relationships": {"{a}": {"type": "invalid-type", "target": "{b}"}}}"##,
        );
        // This should result in a JSON syntax error because the enum doesn't include "invalid-type"
        let issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::JsonSyntax)
            .collect();
        assert_eq!(issues.len(), 1, "Expected JSON syntax error for invalid relationship type");
    }

    #[test]
    fn test_validate_relationship_no_cycle_in_chain() {
        let mut validator = Validator::new();
        // Linear chain without cycle: a -> b -> c (no cycle)
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00", "{c}": "#0000FF"}, "relationships": {"{a}": {"type": "derives-from", "target": "{b}"}, "{b}": {"type": "derives-from", "target": "{c}"}}}"##,
        );
        let cycle_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularDependency)
            .collect();
        assert!(cycle_issues.is_empty(), "Expected no circular dependency issues for linear chain");
    }

    #[test]
    fn test_validate_relationship_self_reference() {
        let mut validator = Validator::new();
        // Self-reference: a -> a (single node cycle)
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}, "relationships": {"{a}": {"type": "derives-from", "target": "{a}"}}}"##,
        );
        let cycle_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularDependency)
            .collect();
        assert!(!cycle_issues.is_empty(), "Expected circular dependency issue for self-reference");
    }
}
