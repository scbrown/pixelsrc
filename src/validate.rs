//! Validation logic for Pixelsrc files
//!
//! Provides semantic validation beyond basic JSON parsing, checking for
//! common mistakes like undefined tokens, row mismatches, and invalid colors.

use crate::color::parse_color;
use crate::models::{PaletteRef, Particle, RegionDef, Relationship, RelationshipType, TtpObject};
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
            IssueType::InvalidWithinReference => write!(f, "invalid_within_ref"),
            IssueType::InvalidAdjacentReference => write!(f, "invalid_adjacent_ref"),
            IssueType::InvalidRelationshipReference => write!(f, "invalid_relationship_ref"),
            IssueType::CircularRelationship => write!(f, "circular_relationship"),
            IssueType::UncertainConstraint => write!(f, "uncertain_constraint"),
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
                self.validate_palette_full(line_number, &palette);
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
                        IssueType::EmptyGrid,
                        "Transform has 0 frames".to_string(),
                    )
                    .with_context(format!("transform \"{}\"", transform.name)),
                );
            }
        }
    }

    /// Validate a full palette definition including relationships
    fn validate_palette_full(&mut self, line_number: usize, palette: &crate::models::Palette) {
        // First validate colors using the basic method
        self.validate_palette(line_number, &palette.name, &palette.colors);

        // Then validate relationships if present
        if let Some(ref relationships) = palette.relationships {
            self.validate_relationships(
                line_number,
                &palette.name,
                relationships,
                &palette.colors,
            );
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

    /// Validate palette relationships
    fn validate_relationships(
        &mut self,
        line_number: usize,
        palette_name: &str,
        relationships: &HashMap<String, Relationship>,
        colors: &HashMap<String, String>,
    ) {
        // Build a graph for circular dependency detection
        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();

        for (source_token, relationship) in relationships {
            // Check that source token exists in colors
            if !colors.contains_key(source_token) {
                self.issues.push(
                    ValidationIssue::error(
                        line_number,
                        IssueType::InvalidRelationshipReference,
                        format!(
                            "Relationship source token \"{}\" not defined in palette colors",
                            source_token
                        ),
                    )
                    .with_context(format!("palette \"{}\"", palette_name)),
                );
            }

            // Check that target token exists in colors
            if !colors.contains_key(&relationship.target) {
                self.issues.push(
                    ValidationIssue::error(
                        line_number,
                        IssueType::InvalidRelationshipReference,
                        format!(
                            "Relationship target token \"{}\" not defined in palette colors",
                            relationship.target
                        ),
                    )
                    .with_context(format!("palette \"{}\"", palette_name)),
                );
            }

            // Add to dependency graph for circular detection
            // Only certain relationship types create dependencies
            match relationship.relationship_type {
                RelationshipType::DerivesFrom | RelationshipType::ContainedWithin => {
                    dependency_graph
                        .entry(source_token.clone())
                        .or_default()
                        .push(relationship.target.clone());
                }
                _ => {}
            }
        }

        // Detect circular relationships
        self.detect_circular_relationships(line_number, palette_name, &dependency_graph);
    }

    /// Detect circular relationships using DFS
    fn detect_circular_relationships(
        &mut self,
        line_number: usize,
        palette_name: &str,
        graph: &HashMap<String, Vec<String>>,
    ) {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) =
                    Self::dfs_find_cycle(node, graph, &mut visited, &mut rec_stack, &mut path)
                {
                    self.issues.push(
                        ValidationIssue::error(
                            line_number,
                            IssueType::CircularRelationship,
                            format!("Circular relationship detected: {}", cycle.join(" -> ")),
                        )
                        .with_context(format!("palette \"{}\"", palette_name)),
                    );
                }
            }
        }
    }

    /// DFS helper to find cycles in the dependency graph
    fn dfs_find_cycle(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) =
                        Self::dfs_find_cycle(neighbor, graph, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle - build the cycle path
                    let start_idx = path.iter().position(|n| n == neighbor).unwrap_or(0);
                    let mut cycle: Vec<String> = path[start_idx..].to_vec();
                    cycle.push(neighbor.clone());
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }

    /// Validate structured regions in a sprite
    fn validate_regions(
        &mut self,
        line_number: usize,
        sprite_name: &str,
        regions: &HashMap<String, RegionDef>,
        palette_tokens: &Option<HashSet<String>>,
    ) {
        // Collect all region token names for cross-referencing
        let region_tokens: HashSet<String> = regions.keys().cloned().collect();

        // Build a dependency graph for circular detection
        let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();

        for (token_name, region_def) in regions {
            // Check that the token exists in palette (if we have palette info)
            if let Some(ref tokens) = palette_tokens {
                if !tokens.contains(token_name) {
                    self.issues.push(
                        ValidationIssue::warning(
                            line_number,
                            IssueType::UndefinedToken,
                            format!("Region token \"{}\" not defined in palette", token_name),
                        )
                        .with_context(format!("sprite \"{}\"", sprite_name)),
                    );
                }
            }

            // Validate 'within' constraint
            if let Some(ref within_ref) = region_def.within {
                if !region_tokens.contains(within_ref) {
                    // Check if it's in the palette tokens instead
                    let in_palette = palette_tokens
                        .as_ref()
                        .is_some_and(|t| t.contains(within_ref));
                    if !in_palette {
                        self.issues.push(
                            ValidationIssue::error(
                                line_number,
                                IssueType::InvalidWithinReference,
                                format!(
                                    "Region \"{}\" has 'within' constraint referencing unknown token \"{}\"",
                                    token_name, within_ref
                                ),
                            )
                            .with_context(format!("sprite \"{}\"", sprite_name)),
                        );
                    }
                }

                // Add to dependency graph for circular detection
                dependency_graph
                    .entry(token_name.clone())
                    .or_default()
                    .push(within_ref.clone());

                // Warn about uncertain validation when regions might overlap
                if region_tokens.contains(within_ref) {
                    self.issues.push(
                        ValidationIssue::warning(
                            line_number,
                            IssueType::UncertainConstraint,
                            format!(
                                "Region \"{}\" 'within' constraint for \"{}\" - containment check requires rendered pixels",
                                token_name, within_ref
                            ),
                        )
                        .with_context(format!("sprite \"{}\"", sprite_name)),
                    );
                }
            }

            // Validate 'adjacent-to' constraint
            if let Some(ref adjacent_ref) = region_def.adjacent_to {
                if !region_tokens.contains(adjacent_ref) {
                    // Check if it's in the palette tokens instead
                    let in_palette = palette_tokens
                        .as_ref()
                        .is_some_and(|t| t.contains(adjacent_ref));
                    if !in_palette {
                        self.issues.push(
                            ValidationIssue::error(
                                line_number,
                                IssueType::InvalidAdjacentReference,
                                format!(
                                    "Region \"{}\" has 'adjacent-to' constraint referencing unknown token \"{}\"",
                                    token_name, adjacent_ref
                                ),
                            )
                            .with_context(format!("sprite \"{}\"", sprite_name)),
                        );
                    }
                }

                // Warn about uncertain validation
                if region_tokens.contains(adjacent_ref) {
                    self.issues.push(
                        ValidationIssue::warning(
                            line_number,
                            IssueType::UncertainConstraint,
                            format!(
                                "Region \"{}\" 'adjacent-to' constraint for \"{}\" - adjacency check requires rendered pixels",
                                token_name, adjacent_ref
                            ),
                        )
                        .with_context(format!("sprite \"{}\"", sprite_name)),
                    );
                }
            }

            // Recursively validate nested regions (union, subtract, intersect, base)
            self.validate_nested_regions(
                line_number,
                sprite_name,
                token_name,
                region_def,
                &region_tokens,
                palette_tokens,
                &mut dependency_graph,
            );
        }

        // Detect circular dependencies in regions
        self.detect_circular_relationships(line_number, sprite_name, &dependency_graph);
    }

    /// Validate nested region definitions (compound operations)
    #[allow(clippy::too_many_arguments)]
    fn validate_nested_regions(
        &mut self,
        line_number: usize,
        sprite_name: &str,
        parent_token: &str,
        region_def: &RegionDef,
        region_tokens: &HashSet<String>,
        palette_tokens: &Option<HashSet<String>>,
        dependency_graph: &mut HashMap<String, Vec<String>>,
    ) {
        // Validate union regions
        if let Some(ref union_regions) = region_def.union {
            for nested in union_regions {
                self.validate_region_constraints(
                    line_number,
                    sprite_name,
                    parent_token,
                    nested,
                    region_tokens,
                    palette_tokens,
                    dependency_graph,
                );
            }
        }

        // Validate base region
        if let Some(ref base) = region_def.base {
            self.validate_region_constraints(
                line_number,
                sprite_name,
                parent_token,
                base,
                region_tokens,
                palette_tokens,
                dependency_graph,
            );
        }

        // Validate subtract regions
        if let Some(ref subtract_regions) = region_def.subtract {
            for nested in subtract_regions {
                self.validate_region_constraints(
                    line_number,
                    sprite_name,
                    parent_token,
                    nested,
                    region_tokens,
                    palette_tokens,
                    dependency_graph,
                );
            }
        }

        // Validate intersect regions
        if let Some(ref intersect_regions) = region_def.intersect {
            for nested in intersect_regions {
                self.validate_region_constraints(
                    line_number,
                    sprite_name,
                    parent_token,
                    nested,
                    region_tokens,
                    palette_tokens,
                    dependency_graph,
                );
            }
        }
    }

    /// Validate constraints in a single region definition
    #[allow(clippy::too_many_arguments)]
    fn validate_region_constraints(
        &mut self,
        line_number: usize,
        sprite_name: &str,
        parent_token: &str,
        region_def: &RegionDef,
        region_tokens: &HashSet<String>,
        palette_tokens: &Option<HashSet<String>>,
        dependency_graph: &mut HashMap<String, Vec<String>>,
    ) {
        // Validate 'within' constraint
        if let Some(ref within_ref) = region_def.within {
            if !region_tokens.contains(within_ref) {
                let in_palette = palette_tokens
                    .as_ref()
                    .is_some_and(|t| t.contains(within_ref));
                if !in_palette {
                    self.issues.push(
                        ValidationIssue::error(
                            line_number,
                            IssueType::InvalidWithinReference,
                            format!(
                                "Nested region in \"{}\" has 'within' constraint referencing unknown token \"{}\"",
                                parent_token, within_ref
                            ),
                        )
                        .with_context(format!("sprite \"{}\"", sprite_name)),
                    );
                }
            }
            dependency_graph
                .entry(parent_token.to_string())
                .or_default()
                .push(within_ref.clone());
        }

        // Validate 'adjacent-to' constraint
        if let Some(ref adjacent_ref) = region_def.adjacent_to {
            if !region_tokens.contains(adjacent_ref) {
                let in_palette = palette_tokens
                    .as_ref()
                    .is_some_and(|t| t.contains(adjacent_ref));
                if !in_palette {
                    self.issues.push(
                        ValidationIssue::error(
                            line_number,
                            IssueType::InvalidAdjacentReference,
                            format!(
                                "Nested region in \"{}\" has 'adjacent-to' constraint referencing unknown token \"{}\"",
                                parent_token, adjacent_ref
                            ),
                        )
                        .with_context(format!("sprite \"{}\"", sprite_name)),
                    );
                }
            }
        }

        // Recursively validate nested regions
        self.validate_nested_regions(
            line_number,
            sprite_name,
            parent_token,
            region_def,
            region_tokens,
            palette_tokens,
            dependency_graph,
        );
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

        // Get palette tokens early - needed for both grid and regions validation
        let palette_tokens = self.get_palette_tokens(&sprite.palette, line_number, name);

        // If sprite has regions, validate them instead of grid
        if let Some(ref regions) = sprite.regions {
            self.validate_regions(line_number, name, regions, &palette_tokens);
            return;
        }

        // Check for empty grid (only if no regions)
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

        // Validate grid rows
        let mut first_row_count: Option<usize> = None;
        let mut all_tokens_used: HashSet<String> = HashSet::new();

        for (row_idx, row) in sprite.grid.iter().enumerate() {
            let (tokens, _warnings) = tokenize(row);

            // Check row length consistency
            match first_row_count {
                None => first_row_count = Some(tokens.len()),
                Some(expected) if tokens.len() != expected => {
                    let actual = tokens.len();
                    let message = format!(
                        "Row {} length mismatch: expected {} tokens, found {}",
                        row_idx + 1,
                        expected,
                        actual
                    );

                    let mut issue = ValidationIssue::warning(
                        line_number,
                        IssueType::RowLengthMismatch,
                        message,
                    )
                    .with_context(format!("sprite \"{}\"", name));

                    // Add padding suggestion for short rows
                    if actual < expected {
                        let padding_needed = expected - actual;
                        let padding = "{_}".repeat(padding_needed);
                        issue = issue.with_suggestion(format!(
                            "add {} padding token{}: {}",
                            padding_needed,
                            if padding_needed == 1 { "" } else { "s" },
                            padding
                        ));
                    }

                    self.issues.push(issue);
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
    fn test_row_length_message_format() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": ["{a}{a}{a}{a}", "{a}{a}"]}"#,
        );

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert_eq!(row_mismatch_issues.len(), 1);

        let issue = row_mismatch_issues[0];
        // Check message format: "Row X length mismatch: expected Y tokens, found Z"
        assert!(
            issue.message.contains("expected 4 tokens"),
            "Message should contain 'expected 4 tokens': {}",
            issue.message
        );
        assert!(
            issue.message.contains("found 2"),
            "Message should contain 'found 2': {}",
            issue.message
        );
    }

    #[test]
    fn test_row_length_padding_suggestion() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": ["{a}{a}{a}{a}", "{a}"]}"#,
        );

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert_eq!(row_mismatch_issues.len(), 1);

        let issue = row_mismatch_issues[0];
        // Check padding suggestion for short row (1 token vs expected 4)
        assert!(issue.suggestion.is_some(), "Short row should have padding suggestion");
        let suggestion = issue.suggestion.as_ref().unwrap();
        assert!(
            suggestion.contains("{_}{_}{_}"),
            "Should suggest 3 padding tokens: {}",
            suggestion
        );
        assert!(
            suggestion.contains("add 3 padding tokens"),
            "Should mention adding 3 tokens: {}",
            suggestion
        );
    }

    #[test]
    fn test_row_length_single_padding_suggestion() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": ["{a}{a}", "{a}"]}"#,
        );

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert_eq!(row_mismatch_issues.len(), 1);

        let issue = row_mismatch_issues[0];
        let suggestion = issue.suggestion.as_ref().unwrap();
        // Should say "token" (singular) not "tokens"
        assert!(
            suggestion.contains("add 1 padding token:"),
            "Should use singular 'token': {}",
            suggestion
        );
    }

    #[test]
    fn test_row_length_no_padding_for_long_rows() {
        let mut validator = Validator::new();
        validator.validate_line(
            1,
            r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
        );
        // Row 2 is LONGER than row 1 (5 tokens vs 3)
        validator.validate_line(
            2,
            r#"{"type": "sprite", "name": "test", "palette": "test", "grid": ["{a}{a}{a}", "{a}{a}{a}{a}{a}"]}"#,
        );

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert_eq!(row_mismatch_issues.len(), 1);

        let issue = row_mismatch_issues[0];
        // Long rows should NOT have padding suggestion (can't "pad" to make shorter)
        assert!(
            issue.suggestion.is_none(),
            "Long rows should not have padding suggestion, but got: {:?}",
            issue.suggestion
        );
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

        let size_mismatch_issues: Vec<_> =
            validator.issues().iter().filter(|i| i.issue_type == IssueType::SizeMismatch).collect();
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

        let empty_grid_issues: Vec<_> =
            validator.issues().iter().filter(|i| i.issue_type == IssueType::EmptyGrid).collect();
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
    #[serial]
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
        assert!(!undefined_token_issues.is_empty(), "Expected undefined token warning for {{b}}");

        let row_mismatch_issues: Vec<_> = validator
            .issues()
            .iter()
            .filter(|i| i.issue_type == IssueType::RowLengthMismatch)
            .collect();
        assert!(!row_mismatch_issues.is_empty(), "Expected row length mismatch warning");
    }

    #[test]
    #[serial]
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

    // ========================================================================
    // Constraint Validation Tests (TTP-oghw)
    // ========================================================================

    #[test]
    fn test_validate_relationship_valid() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00", "{b}": "#0F0"}, "relationships": {"{a}": {"type": "derives-from", "target": "{b}"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, content);
        let issues = validator.issues();

        let relationship_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRelationshipReference)
            .collect();
        assert!(
            relationship_errors.is_empty(),
            "Should not have relationship errors for valid references"
        );
    }

    #[test]
    fn test_validate_relationship_invalid_source() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00"}, "relationships": {"{missing}": {"type": "derives-from", "target": "{a}"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, content);
        let issues = validator.issues();

        let relationship_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRelationshipReference)
            .collect();
        assert_eq!(
            relationship_errors.len(),
            1,
            "Should have 1 error for missing source token"
        );
        assert!(relationship_errors[0].message.contains("missing"));
    }

    #[test]
    fn test_validate_relationship_invalid_target() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00"}, "relationships": {"{a}": {"type": "derives-from", "target": "{nonexistent}"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, content);
        let issues = validator.issues();

        let relationship_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidRelationshipReference)
            .collect();
        assert_eq!(
            relationship_errors.len(),
            1,
            "Should have 1 error for missing target token"
        );
        assert!(relationship_errors[0].message.contains("nonexistent"));
    }

    #[test]
    fn test_validate_circular_relationship() {
        // A derives from B, B derives from A = circular
        let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00", "{b}": "#0F0"}, "relationships": {"{a}": {"type": "derives-from", "target": "{b}"}, "{b}": {"type": "derives-from", "target": "{a}"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, content);
        let issues = validator.issues();

        let circular_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularRelationship)
            .collect();
        assert!(
            !circular_errors.is_empty(),
            "Should detect circular relationship"
        );
    }

    #[test]
    fn test_validate_circular_relationship_chain() {
        // A -> B -> C -> A = circular chain
        let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00", "{b}": "#0F0", "{c}": "#00F"}, "relationships": {"{a}": {"type": "contained-within", "target": "{b}"}, "{b}": {"type": "contained-within", "target": "{c}"}, "{c}": {"type": "contained-within", "target": "{a}"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, content);
        let issues = validator.issues();

        let circular_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularRelationship)
            .collect();
        assert!(
            !circular_errors.is_empty(),
            "Should detect circular relationship chain"
        );
    }

    #[test]
    fn test_validate_no_circular_for_paired_with() {
        // paired-with doesn't create dependency chains
        let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00", "{b}": "#0F0"}, "relationships": {"{a}": {"type": "paired-with", "target": "{b}"}, "{b}": {"type": "paired-with", "target": "{a}"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, content);
        let issues = validator.issues();

        let circular_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularRelationship)
            .collect();
        assert!(
            circular_errors.is_empty(),
            "paired-with should not cause circular dependency errors"
        );
    }

    #[test]
    fn test_validate_region_within_valid() {
        let palette = r##"{"type": "palette", "name": "p", "colors": {"{o}": "#000", "{f}": "#F00"}}"##;
        let sprite = r##"{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"o": {"rect": [0, 0, 8, 8]}, "f": {"rect": [2, 2, 4, 4], "within": "o"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, palette);
        validator.validate_line(2, sprite);
        let issues = validator.issues();

        let within_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidWithinReference)
            .collect();
        assert!(
            within_errors.is_empty(),
            "Should not have errors for valid within reference"
        );
    }

    #[test]
    fn test_validate_region_within_invalid_reference() {
        let palette = r##"{"type": "palette", "name": "p", "colors": {"{o}": "#000", "{f}": "#F00"}}"##;
        let sprite = r##"{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"f": {"rect": [2, 2, 4, 4], "within": "nonexistent"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, palette);
        validator.validate_line(2, sprite);
        let issues = validator.issues();

        let within_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidWithinReference)
            .collect();
        assert_eq!(
            within_errors.len(),
            1,
            "Should have error for invalid within reference"
        );
        assert!(within_errors[0].message.contains("nonexistent"));
    }

    #[test]
    fn test_validate_region_adjacent_invalid_reference() {
        let palette = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00"}}"##;
        let sprite = r##"{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4], "adjacent-to": "missing"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, palette);
        validator.validate_line(2, sprite);
        let issues = validator.issues();

        let adjacent_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::InvalidAdjacentReference)
            .collect();
        assert_eq!(
            adjacent_errors.len(),
            1,
            "Should have error for invalid adjacent-to reference"
        );
    }

    #[test]
    fn test_validate_region_circular_within() {
        // A within B, B within A = circular
        let palette = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00", "{b}": "#0F0"}}"##;
        let sprite = r##"{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4], "within": "b"}, "b": {"rect": [0, 0, 8, 8], "within": "a"}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, palette);
        validator.validate_line(2, sprite);
        let issues = validator.issues();

        let circular_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::CircularRelationship)
            .collect();
        assert!(
            !circular_errors.is_empty(),
            "Should detect circular within dependency"
        );
    }

    #[test]
    fn test_validate_region_uncertain_constraint() {
        // When both regions exist, we can't verify containment without rendering
        let palette = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00", "{b}": "#0F0"}}"##;
        let sprite = r##"{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"a": {"rect": [2, 2, 4, 4], "within": "b"}, "b": {"rect": [0, 0, 8, 8]}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, palette);
        validator.validate_line(2, sprite);
        let issues = validator.issues();

        let uncertain_warnings: Vec<_> = issues
            .iter()
            .filter(|i| i.issue_type == IssueType::UncertainConstraint)
            .collect();
        assert!(
            !uncertain_warnings.is_empty(),
            "Should warn about uncertain constraint validation"
        );
    }

    #[test]
    fn test_validate_region_token_not_in_palette() {
        let palette = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#F00"}}"##;
        // Region "b" is not in palette colors
        let sprite = r##"{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4]}, "b": {"rect": [4, 4, 4, 4]}}}"##;

        let mut validator = Validator::new();
        validator.validate_line(1, palette);
        validator.validate_line(2, sprite);
        let issues = validator.issues();

        let undefined_warnings: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.issue_type == IssueType::UndefinedToken && i.message.contains("\"b\"")
            })
            .collect();
        assert!(
            !undefined_warnings.is_empty(),
            "Should warn about region token not in palette"
        );
    }
}
