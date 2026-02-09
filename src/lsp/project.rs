//! Cross-file project support for the LSP server.
//!
//! Provides project discovery and registry loading from `pxl.toml`,
//! cross-file completions, go-to-definition, and hover information
//! using the `ProjectRegistry`.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::*;

use crate::build::project_registry::ProjectRegistry;
use crate::config::loader::{find_config_from, load_config};

/// Cached project context for the LSP server.
#[derive(Debug)]
pub struct ProjectContext {
    /// The loaded project registry with all items across files.
    pub registry: ProjectRegistry,
    /// Path to the pxl.toml config file.
    pub config_path: PathBuf,
    /// Project source root directory.
    pub src_root: PathBuf,
}

impl ProjectContext {
    /// Try to discover and load a project context from a file path.
    ///
    /// Walks up from the file's directory looking for `pxl.toml`,
    /// then loads the project registry from the configured source directory.
    pub fn from_file(file_path: &Path) -> Option<Self> {
        let start_dir = file_path.parent()?.to_path_buf();
        let config_path = find_config_from(start_dir)?;
        let config = load_config(Some(&config_path)).ok()?;

        let project_root = config_path.parent()?;
        let src_dir = project_root.join(&config.project.src);

        if !src_dir.exists() {
            return None;
        }

        let mut registry = ProjectRegistry::new(config.project.name.clone(), src_dir.clone());
        registry.load_all(false).ok()?;

        Some(ProjectContext { registry, config_path: config_path.clone(), src_root: src_dir })
    }

    /// Reload the project registry (e.g., after a file change).
    pub fn reload(&mut self) -> bool {
        let config = match load_config(Some(&self.config_path)) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let project_root = match self.config_path.parent() {
            Some(p) => p,
            None => return false,
        };

        let src_dir = project_root.join(&config.project.src);
        if !src_dir.exists() {
            return false;
        }

        let mut registry = ProjectRegistry::new(config.project.name.clone(), src_dir.clone());
        match registry.load_all(false) {
            Ok(()) => {
                self.registry = registry;
                self.src_root = src_dir;
                true
            }
            Err(_) => false,
        }
    }
}

/// A resolved cross-file item with its location and type.
#[derive(Debug, Clone)]
pub struct CrossFileItem {
    pub name: String,
    pub item_type: &'static str,
    pub canonical_name: String,
    pub file_path: String,
    pub source_file: PathBuf,
}

/// Get cross-file completion items from the project registry.
pub fn get_cross_file_completions(registry: &ProjectRegistry) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Add palette completions
    for canonical in registry.palette_names() {
        if let Some(loc) = registry.palette_location(canonical) {
            completions.push(CompletionItem {
                label: loc.short_name.clone(),
                detail: Some(format!("palette ({})", loc.file_path)),
                kind: Some(CompletionItemKind::CONSTANT),
                documentation: Some(Documentation::String(format!(
                    "Palette from `{}`\nCanonical: `{}`",
                    loc.file_path, loc.canonical_name
                ))),
                ..Default::default()
            });
        }
    }

    // Add sprite completions
    for canonical in registry.sprite_names() {
        if let Some(loc) = registry.sprite_location(canonical) {
            completions.push(CompletionItem {
                label: loc.short_name.clone(),
                detail: Some(format!("sprite ({})", loc.file_path)),
                kind: Some(CompletionItemKind::CLASS),
                documentation: Some(Documentation::String(format!(
                    "Sprite from `{}`\nCanonical: `{}`",
                    loc.file_path, loc.canonical_name
                ))),
                ..Default::default()
            });
        }
    }

    // Add transform completions
    for canonical in registry.transform_names() {
        if let Some(loc) = registry.transform_location(canonical) {
            completions.push(CompletionItem {
                label: loc.short_name.clone(),
                detail: Some(format!("transform ({})", loc.file_path)),
                kind: Some(CompletionItemKind::FUNCTION),
                documentation: Some(Documentation::String(format!(
                    "Transform from `{}`\nCanonical: `{}`",
                    loc.file_path, loc.canonical_name
                ))),
                ..Default::default()
            });
        }
    }

    // Add composition completions
    for canonical in registry.composition_names() {
        if let Some(loc) = registry.composition_location(canonical) {
            completions.push(CompletionItem {
                label: loc.short_name.clone(),
                detail: Some(format!("composition ({})", loc.file_path)),
                kind: Some(CompletionItemKind::MODULE),
                documentation: Some(Documentation::String(format!(
                    "Composition from `{}`\nCanonical: `{}`",
                    loc.file_path, loc.canonical_name
                ))),
                ..Default::default()
            });
        }
    }

    completions
}

/// Try to resolve a name to a cross-file item location.
///
/// Searches palettes, sprites, transforms, and compositions in order.
pub fn resolve_cross_file_reference(
    registry: &ProjectRegistry,
    name: &str,
) -> Option<CrossFileItem> {
    // Check palettes
    if let Some(short) = registry.resolve_palette_name(name) {
        // Find the canonical name via short name
        for canonical in registry.palette_names() {
            if let Some(loc) = registry.palette_location(canonical) {
                if loc.short_name == short {
                    return Some(CrossFileItem {
                        name: loc.short_name.clone(),
                        item_type: "palette",
                        canonical_name: loc.canonical_name.clone(),
                        file_path: loc.file_path.clone(),
                        source_file: loc.source_file.clone(),
                    });
                }
            }
        }
    }

    // Check sprites
    if let Some(short) = registry.resolve_sprite_name(name) {
        for canonical in registry.sprite_names() {
            if let Some(loc) = registry.sprite_location(canonical) {
                if loc.short_name == short {
                    return Some(CrossFileItem {
                        name: loc.short_name.clone(),
                        item_type: "sprite",
                        canonical_name: loc.canonical_name.clone(),
                        file_path: loc.file_path.clone(),
                        source_file: loc.source_file.clone(),
                    });
                }
            }
        }
    }

    // Check transforms
    if let Some(short) = registry.resolve_transform_name(name) {
        for canonical in registry.transform_names() {
            if let Some(loc) = registry.transform_location(canonical) {
                if loc.short_name == short {
                    return Some(CrossFileItem {
                        name: loc.short_name.clone(),
                        item_type: "transform",
                        canonical_name: loc.canonical_name.clone(),
                        file_path: loc.file_path.clone(),
                        source_file: loc.source_file.clone(),
                    });
                }
            }
        }
    }

    // Check compositions
    if let Some(short) = registry.resolve_composition_name(name) {
        for canonical in registry.composition_names() {
            if let Some(loc) = registry.composition_location(canonical) {
                if loc.short_name == short {
                    return Some(CrossFileItem {
                        name: loc.short_name.clone(),
                        item_type: "composition",
                        canonical_name: loc.canonical_name.clone(),
                        file_path: loc.file_path.clone(),
                        source_file: loc.source_file.clone(),
                    });
                }
            }
        }
    }

    None
}

/// Find the line number where an item is defined in a source file.
///
/// Scans the file looking for a JSON object with matching `"name"` field.
/// Returns the 0-indexed line number.
pub fn find_item_line_in_file(source_file: &Path, item_name: &str) -> Option<u32> {
    let content = std::fs::read_to_string(source_file).ok()?;
    let search_pattern = "\"name\":".to_string();

    for (line_num, line) in content.lines().enumerate() {
        if !line.contains(&search_pattern) {
            continue;
        }
        // Check if this line has the specific name we're looking for
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                if name == item_name {
                    return Some(line_num as u32);
                }
            }
        }
    }

    None
}

/// Build a go-to-definition response for a cross-file reference.
pub fn goto_cross_file_definition(
    registry: &ProjectRegistry,
    name: &str,
) -> Option<GotoDefinitionResponse> {
    let item = resolve_cross_file_reference(registry, name)?;
    let line = find_item_line_in_file(&item.source_file, &item.name).unwrap_or(0);

    let uri = Url::from_file_path(&item.source_file).ok()?;

    Some(GotoDefinitionResponse::Scalar(Location {
        uri,
        range: Range {
            start: Position { line, character: 0 },
            end: Position { line, character: 0 },
        },
    }))
}

/// Build hover content for a cross-file reference.
pub fn hover_cross_file_reference(registry: &ProjectRegistry, name: &str) -> Option<String> {
    let item = resolve_cross_file_reference(registry, name)?;

    Some(format!(
        "**{}** `{}`\n\n\
         **Source**: `{}`\n\n\
         **Canonical**: `{}`",
        capitalize(item.item_type),
        item.name,
        item.file_path,
        item.canonical_name,
    ))
}

/// Get import path completions for the `from` field of import declarations.
///
/// Lists available file paths relative to the project source root.
pub fn get_import_path_completions(registry: &ProjectRegistry) -> Vec<CompletionItem> {
    let mut paths: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Collect unique file paths from all item locations
    for canonical in registry.palette_names() {
        if let Some(loc) = registry.palette_location(canonical) {
            if seen.insert(loc.file_path.clone()) {
                paths.push(loc.file_path.clone());
            }
        }
    }
    for canonical in registry.sprite_names() {
        if let Some(loc) = registry.sprite_location(canonical) {
            if seen.insert(loc.file_path.clone()) {
                paths.push(loc.file_path.clone());
            }
        }
    }
    for canonical in registry.transform_names() {
        if let Some(loc) = registry.transform_location(canonical) {
            if seen.insert(loc.file_path.clone()) {
                paths.push(loc.file_path.clone());
            }
        }
    }
    for canonical in registry.composition_names() {
        if let Some(loc) = registry.composition_location(canonical) {
            if seen.insert(loc.file_path.clone()) {
                paths.push(loc.file_path.clone());
            }
        }
    }

    paths.sort();

    paths
        .into_iter()
        .map(|p| CompletionItem {
            label: p.clone(),
            detail: Some("import path".to_string()),
            kind: Some(CompletionItemKind::FILE),
            insert_text: Some(p),
            ..Default::default()
        })
        .collect()
}

/// Extract a reference name at the cursor position from a JSON line.
///
/// Looks for patterns like `"palette": "name"`, `"source": "name"`,
/// `"base": "name"`, etc. Returns the referenced name if the cursor
/// is positioned on it.
pub fn extract_reference_at_position(line: &str, char_pos: u32) -> Option<String> {
    let pos = char_pos as usize;
    if pos >= line.len() {
        return None;
    }

    // Reference fields that point to other items
    let ref_fields = ["palette", "source", "base", "sprite", "from"];

    // Try to parse the line as JSON to find reference fields
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
        if let Some(obj) = obj.as_object() {
            for field in &ref_fields {
                if let Some(val) = obj.get(*field) {
                    if let Some(name) = val.as_str() {
                        // Check if cursor is within this value's position in the line
                        if let Some(value_start) = find_value_position(line, field, name) {
                            let value_end = value_start + name.len();
                            if pos >= value_start && pos <= value_end {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }

            // Check array fields like "sprites" in compositions
            if let Some(sprites_obj) = obj.get("sprites").and_then(|s| s.as_object()) {
                for (_key, val) in sprites_obj {
                    if let Some(name) = val.as_str() {
                        // Check for aliased refs like "hero:idle" → look for just the part
                        let lookup_name = if name.contains(':') {
                            name.split(':').next_back().unwrap_or(name)
                        } else {
                            name
                        };
                        if let Some(start) = line.find(&format!("\"{}\"", name)) {
                            let value_start = start + 1;
                            let value_end = value_start + name.len();
                            if pos >= value_start && pos <= value_end {
                                return Some(lookup_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Find the position of a JSON string value for a given key in a line.
fn find_value_position(line: &str, key: &str, value: &str) -> Option<usize> {
    // Look for "key": "value" pattern
    let key_pattern = format!("\"{}\"", key);
    let key_pos = line.find(&key_pattern)?;

    // Find the value after the key
    let after_key = &line[key_pos + key_pattern.len()..];
    let value_pattern = format!("\"{}\"", value);
    let value_offset = after_key.find(&value_pattern)?;

    // +1 for the opening quote
    Some(key_pos + key_pattern.len() + value_offset + 1)
}

/// Detect if cursor is in an import `from` field value.
pub fn is_import_from_context(line: &str, char_pos: u32) -> bool {
    let pos = char_pos as usize;

    // Quick check: must have "type": "import" and "from"
    if !line.contains("\"import\"") || !line.contains("\"from\"") {
        return false;
    }

    // Find the "from" value and check if cursor is inside it
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
        if let Some(obj) = obj.as_object() {
            let is_import = obj.get("type").and_then(|t| t.as_str()) == Some("import");
            if !is_import {
                return false;
            }
            if let Some(from_val) = obj.get("from").and_then(|f| f.as_str()) {
                if let Some(start) = find_value_position(line, "from", from_val) {
                    let end = start + from_val.len();
                    return pos >= start && pos <= end;
                }
            }
            // Also return true if the from value is being typed (empty or partial)
            // Check if cursor is after "from": " and before closing quote
            if let Some(from_key) = line.find("\"from\"") {
                let after = &line[from_key..];
                if let Some(colon) = after.find(':') {
                    let after_colon = &after[colon + 1..].trim_start();
                    if after_colon.starts_with('"') {
                        let abs_start = from_key
                            + colon
                            + 1
                            + (after.len()
                                - after[colon + 1..].len()
                                - after[colon + 1..].trim_start().len())
                            + 1;
                        if pos >= abs_start {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Collect unresolved cross-file references from document content.
///
/// Returns diagnostics for references that can't be found in the registry.
pub fn check_cross_file_references(content: &str, registry: &ProjectRegistry) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        let obj = match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(obj) => obj,
            Err(_) => continue,
        };

        let obj = match obj.as_object() {
            Some(o) => o,
            None => continue,
        };

        let obj_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");

        // Skip import objects (they have their own resolution)
        if obj_type == "import" {
            continue;
        }

        // Check palette reference
        if let Some(palette_name) = obj.get("palette").and_then(|p| p.as_str()) {
            // Skip @include references and inline palettes
            if !palette_name.starts_with('@')
                && !palette_name.is_empty()
                && registry.resolve_palette_name(palette_name).is_none()
            {
                // Only warn if the reference looks like a cross-file ref (not a local def)
                if let Some(start) = find_value_position(line, "palette", palette_name) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_num as u32, character: start as u32 },
                            end: Position {
                                line: line_num as u32,
                                character: (start + palette_name.len()) as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        source: Some("pixelsrc".to_string()),
                        message: format!(
                            "Palette '{}' not found in project registry",
                            palette_name
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        // Check source/base reference for variants
        if obj_type == "variant" {
            if let Some(base_name) = obj.get("base").and_then(|b| b.as_str()) {
                if !base_name.is_empty() && registry.resolve_sprite_name(base_name).is_none() {
                    if let Some(start) = find_value_position(line, "base", base_name) {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: line_num as u32, character: start as u32 },
                                end: Position {
                                    line: line_num as u32,
                                    character: (start + base_name.len()) as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            source: Some("pixelsrc".to_string()),
                            message: format!(
                                "Base sprite '{}' not found in project registry",
                                base_name
                            ),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Check source reference for transforms/variants
        if let Some(source_name) = obj.get("source").and_then(|s| s.as_str()) {
            if !source_name.is_empty() && registry.resolve_sprite_name(source_name).is_none() {
                if let Some(start) = find_value_position(line, "source", source_name) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_num as u32, character: start as u32 },
                            end: Position {
                                line: line_num as u32,
                                character: (start + source_name.len()) as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        source: Some("pixelsrc".to_string()),
                        message: format!("Source '{}' not found in project registry", source_name),
                        ..Default::default()
                    });
                }
            }
        }
    }

    diagnostics
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_project(dir: &Path) -> PathBuf {
        let src = dir.join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        // Create pxl.toml
        let mut config = fs::File::create(dir.join("pxl.toml")).unwrap();
        config
            .write_all(
                b"[project]\nname = \"test-project\"\nversion = \"1.0.0\"\nsrc = \"src/pxl\"\n",
            )
            .unwrap();

        // Create palette file
        let palette_dir = src.join("palettes");
        fs::create_dir_all(&palette_dir).unwrap();
        let mut palette_file = fs::File::create(palette_dir.join("mono.pxl")).unwrap();
        palette_file
            .write_all(
                br##"{"type": "palette", "name": "mono", "colors": {"{_}": "#000", "{on}": "#FFF"}}"##,
            )
            .unwrap();

        // Create sprite file
        let sprite_dir = src.join("characters");
        fs::create_dir_all(&sprite_dir).unwrap();
        let mut sprite_file = fs::File::create(sprite_dir.join("hero.pxl")).unwrap();
        sprite_file
            .write_all(br#"{"type": "sprite", "name": "idle", "palette": "mono", "size": [8, 8]}"#)
            .unwrap();

        // Create transform file
        let mut transform_file = fs::File::create(sprite_dir.join("motion.pxl")).unwrap();
        transform_file
            .write_all(
                br#"{"type": "transform", "name": "bounce", "ops": [{"op": "translate", "y": -4}]}"#,
            )
            .unwrap();

        src
    }

    #[test]
    fn test_project_context_from_file() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path);
        assert!(ctx.is_some());

        let ctx = ctx.unwrap();
        assert_eq!(ctx.registry.project_name(), "test-project");
        assert!(ctx.registry.total_items() > 0);
    }

    #[test]
    fn test_project_context_reload() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let mut ctx = ProjectContext::from_file(&file_path).unwrap();
        let initial_items = ctx.registry.total_items();

        // Add a new file
        let mut new_file = fs::File::create(src.join("characters/enemy.pxl")).unwrap();
        new_file
            .write_all(
                br#"{"type": "sprite", "name": "goblin", "palette": "mono", "size": [8, 8]}"#,
            )
            .unwrap();

        assert!(ctx.reload());
        assert!(ctx.registry.total_items() > initial_items);
    }

    #[test]
    fn test_cross_file_completions() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path).unwrap();
        let completions = get_cross_file_completions(&ctx.registry);

        assert!(!completions.is_empty());
        // Should have palette, sprite, and transform completions
        assert!(completions.iter().any(|c| c.label == "mono"));
        assert!(completions.iter().any(|c| c.label == "idle"));
        assert!(completions.iter().any(|c| c.label == "bounce"));
    }

    #[test]
    fn test_resolve_cross_file_reference() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path).unwrap();

        // Resolve palette
        let item = resolve_cross_file_reference(&ctx.registry, "mono");
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.item_type, "palette");
        assert_eq!(item.name, "mono");

        // Resolve sprite
        let item = resolve_cross_file_reference(&ctx.registry, "idle");
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.item_type, "sprite");

        // Resolve nonexistent
        let item = resolve_cross_file_reference(&ctx.registry, "nonexistent");
        assert!(item.is_none());
    }

    #[test]
    fn test_hover_cross_file_reference() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path).unwrap();

        let hover = hover_cross_file_reference(&ctx.registry, "mono");
        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contains("Palette"));
        assert!(hover.contains("mono"));
        assert!(hover.contains("palettes/mono"));
    }

    #[test]
    fn test_goto_cross_file_definition() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path).unwrap();

        let response = goto_cross_file_definition(&ctx.registry, "idle");
        assert!(response.is_some());

        match response.unwrap() {
            GotoDefinitionResponse::Scalar(loc) => {
                assert!(loc.uri.to_string().contains("hero.pxl"));
                assert_eq!(loc.range.start.line, 0);
            }
            _ => panic!("Expected Scalar response"),
        }
    }

    #[test]
    fn test_find_item_line_in_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.pxl");
        let mut f = fs::File::create(&file_path).unwrap();
        f.write_all(
            br##"{"type": "palette", "name": "colors", "colors": {}}
{"type": "sprite", "name": "hero", "size": [8, 8]}
{"type": "sprite", "name": "enemy", "size": [8, 8]}"##,
        )
        .unwrap();

        assert_eq!(find_item_line_in_file(&file_path, "colors"), Some(0));
        assert_eq!(find_item_line_in_file(&file_path, "hero"), Some(1));
        assert_eq!(find_item_line_in_file(&file_path, "enemy"), Some(2));
        assert_eq!(find_item_line_in_file(&file_path, "missing"), None);
    }

    #[test]
    fn test_extract_reference_at_position() {
        let line = r#"{"type": "sprite", "name": "hero", "palette": "mono", "size": [8, 8]}"#;
        let palette_pos = line.find("\"mono\"").unwrap() + 1;
        let result = extract_reference_at_position(line, palette_pos as u32);
        assert_eq!(result, Some("mono".to_string()));

        // Position not on a reference
        let result = extract_reference_at_position(line, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_reference_source_field() {
        let line =
            r#"{"type": "sprite", "name": "flipped", "source": "hero", "transform": ["mirror-h"]}"#;
        let source_pos = line.find("\"hero\"").unwrap() + 1;
        let result = extract_reference_at_position(line, source_pos as u32);
        assert_eq!(result, Some("hero".to_string()));
    }

    #[test]
    fn test_is_import_from_context() {
        let line = r#"{"type": "import", "from": "characters/hero"}"#;
        let from_pos = line.find("characters").unwrap();
        assert!(is_import_from_context(line, from_pos as u32));

        // Not on from field
        assert!(!is_import_from_context(line, 5));

        // Not an import
        let line2 = r#"{"type": "sprite", "name": "hero"}"#;
        assert!(!is_import_from_context(line2, 10));
    }

    #[test]
    fn test_import_path_completions() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path).unwrap();
        let completions = get_import_path_completions(&ctx.registry);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "palettes/mono"));
        assert!(completions.iter().any(|c| c.label == "characters/hero"));
    }

    #[test]
    fn test_check_cross_file_references() {
        let temp = TempDir::new().unwrap();
        let src = create_test_project(temp.path());
        let file_path = src.join("characters/hero.pxl");

        let ctx = ProjectContext::from_file(&file_path).unwrap();

        // Content with valid reference
        let content = r#"{"type": "sprite", "name": "test", "palette": "mono", "size": [8, 8]}"#;
        let diags = check_cross_file_references(content, &ctx.registry);
        assert!(diags.is_empty(), "mono should resolve: {:?}", diags);

        // Content with invalid reference
        let content =
            r#"{"type": "sprite", "name": "test", "palette": "nonexistent", "size": [8, 8]}"#;
        let diags = check_cross_file_references(content, &ctx.registry);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("nonexistent"));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("palette"), "Palette");
        assert_eq!(capitalize("sprite"), "Sprite");
        assert_eq!(capitalize(""), "");
    }
}
