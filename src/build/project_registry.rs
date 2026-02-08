//! Project-wide registry that aggregates items from all `.pxl` files.
//!
//! Parses all source files under the project's `src/pxl/` directory and registers
//! items with fully-qualified canonical names (`project_name/path/to/file:item_name`).
//!
//! This is the foundation for the import system (IMP-1), enabling cross-file
//! references by maintaining a shared namespace across the entire project.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::build::discover_files;
use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::registry::{CompositionRegistry, PaletteRegistry, SpriteRegistry, TransformRegistry};

/// Error type for project registry operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProjectRegistryError {
    /// IO error reading a source file
    #[error("Failed to read '{path}': {source}")]
    Io { path: PathBuf, source: std::io::Error },
    /// Item name contains reserved characters
    #[error("Item name '{name}' in '{file}' contains reserved character '{ch}' (names cannot contain ':' or '/')")]
    InvalidName { name: String, file: PathBuf, ch: char },
    /// Name collision in strict mode
    #[error(
        "Name collision: '{item_type}' named '{name}' defined in both '{existing}' and '{new}'"
    )]
    NameCollision { item_type: String, name: String, existing: String, new: String },
    /// Discovery error
    #[error("Source discovery error: {0}")]
    Discovery(#[from] crate::build::DiscoveryError),
}

/// Warning from project registry loading (lenient mode).
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectRegistryWarning {
    pub message: String,
}

/// Tracks the source file for a registered item.
#[derive(Debug, Clone)]
pub struct ItemLocation {
    /// Canonical name: `project_name/path/to/file:item_name`
    pub canonical_name: String,
    /// Short name (just the item name without path qualification)
    pub short_name: String,
    /// The file-relative path component (e.g., `characters/hero/base`)
    pub file_path: String,
    /// Absolute path to the source file
    pub source_file: PathBuf,
}

/// Project-wide registry that wraps individual registries and provides
/// qualified name resolution across all project files.
#[derive(Debug)]
pub struct ProjectRegistry {
    /// Project name from `pxl.toml`
    project_name: String,
    /// Project source root directory
    src_root: PathBuf,
    /// Palette registry (all palettes from all files)
    pub palettes: PaletteRegistry,
    /// Sprite registry (all sprites/variants from all files)
    pub sprites: SpriteRegistry,
    /// Transform registry (all transforms from all files)
    pub transforms: TransformRegistry,
    /// Composition registry (all compositions from all files)
    pub compositions: CompositionRegistry,
    /// Canonical name → location for palettes
    palette_locations: HashMap<String, ItemLocation>,
    /// Canonical name → location for sprites
    sprite_locations: HashMap<String, ItemLocation>,
    /// Canonical name → location for variants
    variant_locations: HashMap<String, ItemLocation>,
    /// Canonical name → location for transforms
    transform_locations: HashMap<String, ItemLocation>,
    /// Canonical name → location for compositions
    composition_locations: HashMap<String, ItemLocation>,
    /// Short name → canonical name for palettes (first-wins)
    palette_short_names: HashMap<String, String>,
    /// Short name → canonical name for sprites (first-wins)
    sprite_short_names: HashMap<String, String>,
    /// Short name → canonical name for variants (first-wins)
    variant_short_names: HashMap<String, String>,
    /// Short name → canonical name for transforms (first-wins)
    transform_short_names: HashMap<String, String>,
    /// Short name → canonical name for compositions (first-wins)
    composition_short_names: HashMap<String, String>,
    /// Warnings accumulated during loading
    warnings: Vec<ProjectRegistryWarning>,
    /// Files that were loaded
    loaded_files: Vec<PathBuf>,
}

impl ProjectRegistry {
    /// Create a new empty project registry.
    pub fn new(project_name: String, src_root: PathBuf) -> Self {
        Self {
            project_name,
            src_root,
            palettes: PaletteRegistry::new(),
            sprites: SpriteRegistry::new(),
            transforms: TransformRegistry::new(),
            compositions: CompositionRegistry::new(),
            palette_locations: HashMap::new(),
            sprite_locations: HashMap::new(),
            variant_locations: HashMap::new(),
            transform_locations: HashMap::new(),
            composition_locations: HashMap::new(),
            palette_short_names: HashMap::new(),
            sprite_short_names: HashMap::new(),
            variant_short_names: HashMap::new(),
            transform_short_names: HashMap::new(),
            composition_short_names: HashMap::new(),
            warnings: Vec::new(),
            loaded_files: Vec::new(),
        }
    }

    /// Load all `.pxl` and `.jsonl` files from the project source directory.
    ///
    /// In strict mode, name collisions are errors. In lenient mode, they produce
    /// warnings and the first definition wins.
    pub fn load_all(&mut self, strict: bool) -> Result<(), ProjectRegistryError> {
        let files = discover_files(&self.src_root, "**/*.pxl")?;
        let jsonl_files = discover_files(&self.src_root, "**/*.jsonl")?;

        let mut all_files = files;
        all_files.extend(jsonl_files);
        all_files.sort();
        all_files.dedup();

        for file_path in all_files {
            self.load_file(&file_path, strict)?;
        }

        Ok(())
    }

    /// Load a single file and register all its items.
    pub fn load_file(
        &mut self,
        file_path: &Path,
        strict: bool,
    ) -> Result<(), ProjectRegistryError> {
        let file = File::open(file_path)
            .map_err(|e| ProjectRegistryError::Io { path: file_path.to_path_buf(), source: e })?;
        let reader = BufReader::new(file);
        let parse_result = parse_stream(reader);

        let file_module = self.file_to_module_path(file_path);

        for obj in parse_result.objects {
            match obj {
                TtpObject::Palette(p) => {
                    validate_name(&p.name, file_path)?;
                    let canonical = format!("{}/{}:{}", self.project_name, file_module, p.name);
                    self.register_palette_location(
                        &canonical,
                        &p.name,
                        &file_module,
                        file_path,
                        strict,
                    )?;
                    self.palettes.register(p);
                }
                TtpObject::Sprite(s) => {
                    validate_name(&s.name, file_path)?;
                    let canonical = format!("{}/{}:{}", self.project_name, file_module, s.name);
                    self.register_sprite_location(
                        &canonical,
                        &s.name,
                        &file_module,
                        file_path,
                        strict,
                    )?;
                    self.sprites.register_sprite(s);
                }
                TtpObject::Variant(v) => {
                    validate_name(&v.name, file_path)?;
                    let canonical = format!("{}/{}:{}", self.project_name, file_module, v.name);
                    self.register_variant_location(
                        &canonical,
                        &v.name,
                        &file_module,
                        file_path,
                        strict,
                    )?;
                    self.sprites.register_variant(v);
                }
                TtpObject::Transform(t) => {
                    validate_name(&t.name, file_path)?;
                    let canonical = format!("{}/{}:{}", self.project_name, file_module, t.name);
                    self.register_transform_location(
                        &canonical,
                        &t.name,
                        &file_module,
                        file_path,
                        strict,
                    )?;
                    self.transforms.register(t);
                }
                TtpObject::Composition(c) => {
                    validate_name(&c.name, file_path)?;
                    let canonical = format!("{}/{}:{}", self.project_name, file_module, c.name);
                    self.register_composition_location(
                        &canonical,
                        &c.name,
                        &file_module,
                        file_path,
                        strict,
                    )?;
                    self.compositions.register(c);
                }
                // Animation, Particle, StateRules — not yet indexed in project registry
                _ => {}
            }
        }

        self.loaded_files.push(file_path.to_path_buf());
        Ok(())
    }

    /// Get the project name.
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    /// Get the source root directory.
    pub fn src_root(&self) -> &Path {
        &self.src_root
    }

    /// Get warnings accumulated during loading.
    pub fn warnings(&self) -> &[ProjectRegistryWarning] {
        &self.warnings
    }

    /// Get the list of loaded files.
    pub fn loaded_files(&self) -> &[PathBuf] {
        &self.loaded_files
    }

    /// Resolve a palette by name, checking canonical name, file:item, and short name.
    ///
    /// Returns the short name (registry key) for the resolved palette.
    ///
    /// Resolution order:
    /// 1. Exact canonical name (`project/path/file:item`)
    /// 2. File-qualified name (`path/file:item` or `file:item`)
    /// 3. Short name (bare `item`)
    pub fn resolve_palette_name(&self, name: &str) -> Option<&str> {
        // 1. Exact canonical name
        if let Some(loc) = self.palette_locations.get(name) {
            return Some(&loc.short_name);
        }

        // 2. File-qualified without project prefix: try prepending project name
        if name.contains(':') {
            let canonical = format!("{}/{}", self.project_name, name);
            if let Some(loc) = self.palette_locations.get(&canonical) {
                return Some(&loc.short_name);
            }
        }

        // 3. Short name lookup
        if let Some(canonical) = self.palette_short_names.get(name) {
            return self.palette_locations.get(canonical).map(|l| l.short_name.as_str());
        }

        None
    }

    /// Resolve a sprite by name, checking canonical name, file:item, and short name.
    ///
    /// Returns the short name (registry key) for the resolved sprite.
    pub fn resolve_sprite_name(&self, name: &str) -> Option<&str> {
        if let Some(loc) = self.sprite_locations.get(name) {
            return Some(&loc.short_name);
        }

        if name.contains(':') {
            let canonical = format!("{}/{}", self.project_name, name);
            if let Some(loc) = self.sprite_locations.get(&canonical) {
                return Some(&loc.short_name);
            }
        }

        if let Some(canonical) = self.sprite_short_names.get(name) {
            return self.sprite_locations.get(canonical).map(|l| l.short_name.as_str());
        }

        None
    }

    /// Resolve a transform by name, checking canonical name, file:item, and short name.
    ///
    /// Returns the short name (registry key) for the resolved transform.
    pub fn resolve_transform_name(&self, name: &str) -> Option<&str> {
        if let Some(loc) = self.transform_locations.get(name) {
            return Some(&loc.short_name);
        }

        if name.contains(':') {
            let canonical = format!("{}/{}", self.project_name, name);
            if let Some(loc) = self.transform_locations.get(&canonical) {
                return Some(&loc.short_name);
            }
        }

        if let Some(canonical) = self.transform_short_names.get(name) {
            return self.transform_locations.get(canonical).map(|l| l.short_name.as_str());
        }

        None
    }

    /// Resolve a composition by name.
    ///
    /// Returns the short name (registry key) for the resolved composition.
    pub fn resolve_composition_name(&self, name: &str) -> Option<&str> {
        if let Some(loc) = self.composition_locations.get(name) {
            return Some(&loc.short_name);
        }

        if name.contains(':') {
            let canonical = format!("{}/{}", self.project_name, name);
            if let Some(loc) = self.composition_locations.get(&canonical) {
                return Some(&loc.short_name);
            }
        }

        if let Some(canonical) = self.composition_short_names.get(name) {
            return self.composition_locations.get(canonical).map(|l| l.short_name.as_str());
        }

        None
    }

    /// Get the location info for a palette by canonical name.
    pub fn palette_location(&self, canonical: &str) -> Option<&ItemLocation> {
        self.palette_locations.get(canonical)
    }

    /// Get the location info for a sprite by canonical name.
    pub fn sprite_location(&self, canonical: &str) -> Option<&ItemLocation> {
        self.sprite_locations.get(canonical)
    }

    /// Get all palette canonical names.
    pub fn palette_names(&self) -> impl Iterator<Item = &String> {
        self.palette_locations.keys()
    }

    /// Get all sprite canonical names.
    pub fn sprite_names(&self) -> impl Iterator<Item = &String> {
        self.sprite_locations.keys()
    }

    /// Get all transform canonical names.
    pub fn transform_names(&self) -> impl Iterator<Item = &String> {
        self.transform_locations.keys()
    }

    /// Get all composition canonical names.
    pub fn composition_names(&self) -> impl Iterator<Item = &String> {
        self.composition_locations.keys()
    }

    /// Total number of items across all registries.
    pub fn total_items(&self) -> usize {
        self.palette_locations.len()
            + self.sprite_locations.len()
            + self.variant_locations.len()
            + self.transform_locations.len()
            + self.composition_locations.len()
    }

    /// Convert an absolute file path to a module path relative to src_root.
    ///
    /// Example: `/project/src/pxl/characters/hero/base.pxl` → `characters/hero/base`
    fn file_to_module_path(&self, file_path: &Path) -> String {
        let relative = file_path.strip_prefix(&self.src_root).unwrap_or(file_path);

        let stem = relative.with_extension("");
        stem.to_string_lossy().replace('\\', "/") // Normalize Windows paths
    }

    // --- Internal registration helpers ---

    fn register_palette_location(
        &mut self,
        canonical: &str,
        short_name: &str,
        file_path: &str,
        source_file: &Path,
        strict: bool,
    ) -> Result<(), ProjectRegistryError> {
        let location = ItemLocation {
            canonical_name: canonical.to_string(),
            short_name: short_name.to_string(),
            file_path: file_path.to_string(),
            source_file: source_file.to_path_buf(),
        };

        self.palette_locations.insert(canonical.to_string(), location);

        if let Some(existing_canonical) = self.palette_short_names.get(short_name) {
            let existing_loc = &self.palette_locations[existing_canonical];
            if strict {
                return Err(ProjectRegistryError::NameCollision {
                    item_type: "palette".to_string(),
                    name: short_name.to_string(),
                    existing: existing_loc.source_file.display().to_string(),
                    new: source_file.display().to_string(),
                });
            } else {
                self.warnings.push(ProjectRegistryWarning {
                    message: format!(
                        "Palette '{}' defined in both '{}' and '{}'; first definition wins",
                        short_name,
                        existing_loc.source_file.display(),
                        source_file.display(),
                    ),
                });
            }
        } else {
            self.palette_short_names.insert(short_name.to_string(), canonical.to_string());
        }

        Ok(())
    }

    fn register_sprite_location(
        &mut self,
        canonical: &str,
        short_name: &str,
        file_path: &str,
        source_file: &Path,
        strict: bool,
    ) -> Result<(), ProjectRegistryError> {
        let location = ItemLocation {
            canonical_name: canonical.to_string(),
            short_name: short_name.to_string(),
            file_path: file_path.to_string(),
            source_file: source_file.to_path_buf(),
        };

        self.sprite_locations.insert(canonical.to_string(), location);

        if let Some(existing_canonical) = self.sprite_short_names.get(short_name) {
            let existing_loc = &self.sprite_locations[existing_canonical];
            if strict {
                return Err(ProjectRegistryError::NameCollision {
                    item_type: "sprite".to_string(),
                    name: short_name.to_string(),
                    existing: existing_loc.source_file.display().to_string(),
                    new: source_file.display().to_string(),
                });
            } else {
                self.warnings.push(ProjectRegistryWarning {
                    message: format!(
                        "Sprite '{}' defined in both '{}' and '{}'; first definition wins",
                        short_name,
                        existing_loc.source_file.display(),
                        source_file.display(),
                    ),
                });
            }
        } else {
            self.sprite_short_names.insert(short_name.to_string(), canonical.to_string());
        }

        Ok(())
    }

    fn register_variant_location(
        &mut self,
        canonical: &str,
        short_name: &str,
        file_path: &str,
        source_file: &Path,
        strict: bool,
    ) -> Result<(), ProjectRegistryError> {
        let location = ItemLocation {
            canonical_name: canonical.to_string(),
            short_name: short_name.to_string(),
            file_path: file_path.to_string(),
            source_file: source_file.to_path_buf(),
        };

        self.variant_locations.insert(canonical.to_string(), location);

        if let Some(existing_canonical) = self.variant_short_names.get(short_name) {
            let existing_loc = &self.variant_locations[existing_canonical];
            if strict {
                return Err(ProjectRegistryError::NameCollision {
                    item_type: "variant".to_string(),
                    name: short_name.to_string(),
                    existing: existing_loc.source_file.display().to_string(),
                    new: source_file.display().to_string(),
                });
            } else {
                self.warnings.push(ProjectRegistryWarning {
                    message: format!(
                        "Variant '{}' defined in both '{}' and '{}'; first definition wins",
                        short_name,
                        existing_loc.source_file.display(),
                        source_file.display(),
                    ),
                });
            }
        } else {
            self.variant_short_names.insert(short_name.to_string(), canonical.to_string());
        }

        Ok(())
    }

    fn register_transform_location(
        &mut self,
        canonical: &str,
        short_name: &str,
        file_path: &str,
        source_file: &Path,
        strict: bool,
    ) -> Result<(), ProjectRegistryError> {
        let location = ItemLocation {
            canonical_name: canonical.to_string(),
            short_name: short_name.to_string(),
            file_path: file_path.to_string(),
            source_file: source_file.to_path_buf(),
        };

        self.transform_locations.insert(canonical.to_string(), location);

        if let Some(existing_canonical) = self.transform_short_names.get(short_name) {
            let existing_loc = &self.transform_locations[existing_canonical];
            if strict {
                return Err(ProjectRegistryError::NameCollision {
                    item_type: "transform".to_string(),
                    name: short_name.to_string(),
                    existing: existing_loc.source_file.display().to_string(),
                    new: source_file.display().to_string(),
                });
            } else {
                self.warnings.push(ProjectRegistryWarning {
                    message: format!(
                        "Transform '{}' defined in both '{}' and '{}'; first definition wins",
                        short_name,
                        existing_loc.source_file.display(),
                        source_file.display(),
                    ),
                });
            }
        } else {
            self.transform_short_names.insert(short_name.to_string(), canonical.to_string());
        }

        Ok(())
    }

    fn register_composition_location(
        &mut self,
        canonical: &str,
        short_name: &str,
        file_path: &str,
        source_file: &Path,
        strict: bool,
    ) -> Result<(), ProjectRegistryError> {
        let location = ItemLocation {
            canonical_name: canonical.to_string(),
            short_name: short_name.to_string(),
            file_path: file_path.to_string(),
            source_file: source_file.to_path_buf(),
        };

        self.composition_locations.insert(canonical.to_string(), location);

        if let Some(existing_canonical) = self.composition_short_names.get(short_name) {
            let existing_loc = &self.composition_locations[existing_canonical];
            if strict {
                return Err(ProjectRegistryError::NameCollision {
                    item_type: "composition".to_string(),
                    name: short_name.to_string(),
                    existing: existing_loc.source_file.display().to_string(),
                    new: source_file.display().to_string(),
                });
            } else {
                self.warnings.push(ProjectRegistryWarning {
                    message: format!(
                        "Composition '{}' defined in both '{}' and '{}'; first definition wins",
                        short_name,
                        existing_loc.source_file.display(),
                        source_file.display(),
                    ),
                });
            }
        } else {
            self.composition_short_names.insert(short_name.to_string(), canonical.to_string());
        }

        Ok(())
    }
}

/// Validate that an item name does not contain reserved characters.
fn validate_name(name: &str, file_path: &Path) -> Result<(), ProjectRegistryError> {
    for ch in [':', '/'] {
        if name.contains(ch) {
            return Err(ProjectRegistryError::InvalidName {
                name: name.to_string(),
                file: file_path.to_path_buf(),
                ch,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a .pxl file in a temp directory with the given content.
    fn create_pxl_file(dir: &Path, relative_path: &str, content: &str) -> PathBuf {
        let path = dir.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_empty_project() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        let mut registry = ProjectRegistry::new("test-project".to_string(), src);
        registry.load_all(false).unwrap();

        assert_eq!(registry.total_items(), 0);
        assert!(registry.loaded_files().is_empty());
        assert!(registry.warnings().is_empty());
    }

    #[test]
    fn test_single_file_with_palette() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "palettes/mono.pxl",
            r##"{"type": "palette", "name": "mono", "colors": {"{_}": "#000", "{on}": "#FFF"}}"##,
        );

        let mut registry = ProjectRegistry::new("my-game".to_string(), src);
        registry.load_all(false).unwrap();

        assert_eq!(registry.total_items(), 1);
        assert_eq!(registry.loaded_files().len(), 1);
        assert!(registry.palettes.contains("mono"));

        // Check canonical name exists
        assert!(registry.palette_locations.contains_key("my-game/palettes/mono:mono"));

        // Short name resolution
        assert_eq!(registry.resolve_palette_name("mono"), Some("mono"));

        // File-qualified resolution
        assert_eq!(registry.resolve_palette_name("palettes/mono:mono"), Some("mono"));
    }

    #[test]
    fn test_single_file_with_sprite() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "characters/hero.pxl",
            r##"{"type": "sprite", "name": "idle", "palette": {"_{_}": "#000"}, "size": [4, 4]}"##,
        );

        let mut registry = ProjectRegistry::new("my-game".to_string(), src);
        registry.load_all(false).unwrap();

        assert_eq!(registry.total_items(), 1);
        assert!(registry.sprites.contains("idle"));

        assert!(registry.sprite_locations.contains_key("my-game/characters/hero:idle"));
        assert_eq!(registry.resolve_sprite_name("idle"), Some("idle"));
        assert_eq!(registry.resolve_sprite_name("characters/hero:idle"), Some("idle"));
    }

    #[test]
    fn test_multiple_files_no_collision() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "palettes/dark.pxl",
            r##"{"type": "palette", "name": "dark", "colors": {"{bg}": "#111"}}"##,
        );
        create_pxl_file(
            &src,
            "palettes/light.pxl",
            r##"{"type": "palette", "name": "light", "colors": {"{bg}": "#EEE"}}"##,
        );
        create_pxl_file(
            &src,
            "sprites/hero.pxl",
            r#"{"type": "sprite", "name": "hero", "palette": "dark", "size": [8, 8]}"#,
        );

        let mut registry = ProjectRegistry::new("rpg".to_string(), src);
        registry.load_all(false).unwrap();

        assert_eq!(registry.total_items(), 3);
        assert!(registry.palettes.contains("dark"));
        assert!(registry.palettes.contains("light"));
        assert!(registry.sprites.contains("hero"));
        assert!(registry.warnings().is_empty());
    }

    #[test]
    fn test_name_collision_lenient() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "a.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#F00"}}"##,
        );
        create_pxl_file(
            &src,
            "b.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#0F0"}}"##,
        );

        let mut registry = ProjectRegistry::new("test".to_string(), src);
        registry.load_all(false).unwrap();

        // Both canonical names exist
        assert!(registry.palette_locations.contains_key("test/a:shared"));
        assert!(registry.palette_locations.contains_key("test/b:shared"));

        // Short name resolves to first (a.pxl comes first alphabetically)
        assert_eq!(registry.resolve_palette_name("shared"), Some("shared"));

        // Warning was produced
        assert_eq!(registry.warnings().len(), 1);
        assert!(registry.warnings()[0].message.contains("shared"));
    }

    #[test]
    fn test_name_collision_strict() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "a.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#F00"}}"##,
        );
        create_pxl_file(
            &src,
            "b.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#0F0"}}"##,
        );

        let mut registry = ProjectRegistry::new("test".to_string(), src);
        let result = registry.load_all(true);

        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectRegistryError::NameCollision { item_type, name, .. } => {
                assert_eq!(item_type, "palette");
                assert_eq!(name, "shared");
            }
            e => panic!("Expected NameCollision, got {:?}", e),
        }
    }

    #[test]
    fn test_invalid_name_with_colon() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "bad.pxl",
            r#"{"type": "palette", "name": "bad:name", "colors": {}}"#,
        );

        let mut registry = ProjectRegistry::new("test".to_string(), src);
        let result = registry.load_all(false);

        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectRegistryError::InvalidName { name, ch, .. } => {
                assert_eq!(name, "bad:name");
                assert_eq!(ch, ':');
            }
            e => panic!("Expected InvalidName, got {:?}", e),
        }
    }

    #[test]
    fn test_invalid_name_with_slash() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "bad.pxl",
            r#"{"type": "sprite", "name": "bad/name", "palette": {}, "size": [4, 4]}"#,
        );

        let mut registry = ProjectRegistry::new("test".to_string(), src);
        let result = registry.load_all(false);

        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectRegistryError::InvalidName { name, ch, .. } => {
                assert_eq!(name, "bad/name");
                assert_eq!(ch, '/');
            }
            e => panic!("Expected InvalidName, got {:?}", e),
        }
    }

    #[test]
    fn test_file_to_module_path() {
        let src = PathBuf::from("/project/src/pxl");
        let registry = ProjectRegistry::new("test".to_string(), src);

        assert_eq!(
            registry.file_to_module_path(Path::new("/project/src/pxl/sprites/hero.pxl")),
            "sprites/hero"
        );
        assert_eq!(
            registry.file_to_module_path(Path::new("/project/src/pxl/simple.pxl")),
            "simple"
        );
        assert_eq!(
            registry.file_to_module_path(Path::new("/project/src/pxl/deep/nested/path/item.jsonl")),
            "deep/nested/path/item"
        );
    }

    #[test]
    fn test_canonical_name_format() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "characters/hero/base.pxl",
            r#"{"type": "sprite", "name": "idle", "palette": {}, "size": [8, 8]}"#,
        );

        let mut registry = ProjectRegistry::new("my-rpg".to_string(), src);
        registry.load_all(false).unwrap();

        let expected_canonical = "my-rpg/characters/hero/base:idle";
        assert!(
            registry.sprite_locations.contains_key(expected_canonical),
            "Expected canonical name: {}",
            expected_canonical
        );
    }

    #[test]
    fn test_mixed_object_types() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "game.pxl",
            concat!(
                r##"{"type": "palette", "name": "mono", "colors": {"{_}": "#000", "{on}": "#FFF"}}"##,
                "\n",
                r#"{"type": "sprite", "name": "dot", "palette": "mono", "size": [1, 1]}"#,
                "\n",
                r##"{"type": "variant", "name": "dot_red", "base": "dot", "palette": {"{on}": "#F00"}}"##,
            ),
        );

        let mut registry = ProjectRegistry::new("test".to_string(), src);
        registry.load_all(false).unwrap();

        assert_eq!(registry.palette_locations.len(), 1);
        assert_eq!(registry.sprite_locations.len(), 1);
        assert_eq!(registry.variant_locations.len(), 1);
        assert_eq!(registry.total_items(), 3);

        assert!(registry.palettes.contains("mono"));
        assert!(registry.sprites.contains("dot"));
        assert!(registry.sprites.contains("dot_red"));
    }

    #[test]
    fn test_type_aware_resolution() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        // Same name "forest" for both palette and sprite — no collision because type-aware
        create_pxl_file(
            &src,
            "a.pxl",
            r##"{"type": "palette", "name": "forest", "colors": {"{leaf}": "#0A0"}}"##,
        );
        create_pxl_file(
            &src,
            "b.pxl",
            r#"{"type": "sprite", "name": "forest", "palette": "forest", "size": [16, 16]}"#,
        );

        let mut registry = ProjectRegistry::new("game".to_string(), src);
        registry.load_all(false).unwrap();

        // Both resolve independently — no collision
        assert_eq!(registry.resolve_palette_name("forest"), Some("forest"));
        assert_eq!(registry.resolve_sprite_name("forest"), Some("forest"));
        assert!(registry.warnings().is_empty());
    }

    #[test]
    fn test_resolve_nonexistent_returns_none() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        let registry = ProjectRegistry::new("test".to_string(), src);

        assert_eq!(registry.resolve_palette_name("nope"), None);
        assert_eq!(registry.resolve_sprite_name("nope"), None);
        assert_eq!(registry.resolve_transform_name("nope"), None);
        assert_eq!(registry.resolve_composition_name("nope"), None);
    }

    #[test]
    fn test_validate_name_ok() {
        assert!(validate_name("idle", Path::new("test.pxl")).is_ok());
        assert!(validate_name("hero_red", Path::new("test.pxl")).is_ok());
        assert!(validate_name("my-palette-2", Path::new("test.pxl")).is_ok());
    }

    #[test]
    fn test_validate_name_reserved_chars() {
        assert!(validate_name("bad:name", Path::new("test.pxl")).is_err());
        assert!(validate_name("bad/name", Path::new("test.pxl")).is_err());
    }

    #[test]
    fn test_transform_registration() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "transforms/motion.pxl",
            r#"{"type": "transform", "name": "bounce", "ops": [{"op": "translate", "y": -4}]}"#,
        );

        let mut registry = ProjectRegistry::new("game".to_string(), src);
        registry.load_all(false).unwrap();

        assert!(registry.transforms.contains("bounce"));
        assert!(registry.transform_locations.contains_key("game/transforms/motion:bounce"));
        assert_eq!(registry.resolve_transform_name("bounce"), Some("bounce"));
    }

    #[test]
    fn test_composition_registration() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        create_pxl_file(
            &src,
            "scenes/battle.pxl",
            r#"{"type": "composition", "name": "arena", "size": [64, 64], "sprites": {}, "layers": []}"#,
        );

        let mut registry = ProjectRegistry::new("game".to_string(), src);
        registry.load_all(false).unwrap();

        assert!(registry.compositions.contains("arena"));
        assert!(registry.composition_locations.contains_key("game/scenes/battle:arena"));
        assert_eq!(registry.resolve_composition_name("arena"), Some("arena"));
    }

    #[test]
    fn test_load_nonexistent_file_errors() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        let mut registry = ProjectRegistry::new("test".to_string(), src);
        let result = registry.load_file(Path::new("/nonexistent/file.pxl"), false);

        assert!(result.is_err());
        matches!(result.unwrap_err(), ProjectRegistryError::Io { .. });
    }
}
