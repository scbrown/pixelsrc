//! Import resolution for cross-file references.
//!
//! Resolves `{"type": "import", ...}` declarations into actual items from target files.
//! Handles path resolution, circular detection, diamond imports, selective filtering,
//! alias naming, and collision handling.
//!
//! # Path Resolution
//!
//! | Path form | Resolves relative to | Requires project? |
//! |-----------|---------------------|-------------------|
//! | `./path` or `../path` | Current file's directory | No |
//! | `path` (bare) | Project root (`src/pxl/`) | Yes |
//!
//! # Import Variants
//!
//! - **Unfiltered**: imports all items from a file
//! - **Selective**: imports specific items by type (`sprites`, `palettes`, etc.)
//! - **Aliased**: creates namespace prefixed names (`alias:item_name`)
//! - **Directory**: imports all files from a directory (trailing `/`)

use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::models::{
    Animation, Composition, Import, Palette, Sprite, TransformDef, TtpObject, Variant,
};
use crate::parser::parse_stream;

/// Error type for import resolution failures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ImportError {
    /// Circular import detected (file imports itself or cycle detected).
    #[error("Circular import detected: {cycle}")]
    CircularImport { cycle: String },

    /// Import target file not found.
    #[error("Import file not found: '{path}' (tried .pxl and .jsonl extensions)")]
    FileNotFound { path: String },

    /// Root-relative import used without a project context.
    #[error("Root-relative import '{from}' requires pxl.toml project configuration")]
    NoProjectContext { from: String },

    /// Import alias shadows a name defined locally in the importing file.
    #[error("Import alias '{alias}' shadows local name '{name}'")]
    AliasShadowsLocal { alias: String, name: String },

    /// Name collision in strict mode.
    #[error("Import collision in strict mode: {item_type} '{name}' already imported")]
    Collision { item_type: String, name: String },

    /// IO error reading an import target.
    #[error("IO error reading '{path}': {message}")]
    Io { path: String, message: String },

    /// Selectively imported item not found in target file.
    #[error("Imported {item_type} '{name}' not found in '{from}'")]
    ItemNotFound { item_type: String, name: String, from: String },
}

/// Warning from import resolution (lenient mode).
#[derive(Debug, Clone)]
pub struct ImportWarning {
    pub message: String,
}

/// Items parsed from a single file (internal use).
#[derive(Debug, Default)]
struct FileItems {
    palettes: Vec<Palette>,
    sprites: Vec<Sprite>,
    variants: Vec<Variant>,
    transforms: Vec<TransformDef>,
    compositions: Vec<Composition>,
    animations: Vec<Animation>,
}

/// Result of resolving all import declarations in a file.
///
/// Contains items with their names already set to effective names
/// (original for non-aliased, `alias:name` for aliased imports).
#[derive(Debug, Default)]
pub struct ResolvedImports {
    pub palettes: Vec<Palette>,
    pub sprites: Vec<Sprite>,
    pub variants: Vec<Variant>,
    pub transforms: Vec<TransformDef>,
    pub compositions: Vec<Composition>,
    pub animations: Vec<Animation>,
    pub warnings: Vec<ImportWarning>,
}

/// Resolves import declarations to actual items from target files.
///
/// Handles path resolution, circular/diamond detection, selective filtering,
/// alias naming, and collision handling.
pub struct ImportResolver {
    /// Project source root directory (for root-relative imports).
    src_root: Option<PathBuf>,
    /// Canonical path of the file doing the importing (for circular detection).
    importing_file: Option<PathBuf>,
    /// Canonical paths of all files parsed during resolution (for diamond detection).
    visited: HashSet<PathBuf>,
    /// Names already imported per type (collision detection).
    imported_palettes: HashSet<String>,
    imported_sprites: HashSet<String>,
    imported_variants: HashSet<String>,
    imported_transforms: HashSet<String>,
    imported_compositions: HashSet<String>,
    imported_animations: HashSet<String>,
    /// Names defined locally in the importing file.
    local_names: HashSet<String>,
    /// Strict mode: collisions are errors.
    strict: bool,
    /// Accumulated warnings.
    warnings: Vec<ImportWarning>,
}

impl ImportResolver {
    /// Create a new import resolver.
    ///
    /// # Arguments
    /// * `src_root` - Project source root (e.g., `src/pxl/`). `None` for standalone files.
    /// * `strict` - If `true`, collisions are errors. If `false`, first import wins with warning.
    pub fn new(src_root: Option<PathBuf>, strict: bool) -> Self {
        Self {
            src_root,
            importing_file: None,
            visited: HashSet::new(),
            imported_palettes: HashSet::new(),
            imported_sprites: HashSet::new(),
            imported_variants: HashSet::new(),
            imported_transforms: HashSet::new(),
            imported_compositions: HashSet::new(),
            imported_animations: HashSet::new(),
            local_names: HashSet::new(),
            strict,
            warnings: Vec::new(),
        }
    }

    /// Mark a file as the importing file (for circular detection) and add it to visited set.
    pub fn mark_visited(&mut self, path: &Path) {
        if let Ok(canonical) = path.canonicalize() {
            self.importing_file = Some(canonical.clone());
            self.visited.insert(canonical);
        }
    }

    /// Resolve all import declarations from a file.
    ///
    /// # Arguments
    /// * `imports` - Import declarations to resolve
    /// * `base_dir` - Directory of the importing file (for relative path resolution)
    /// * `local_names` - Names defined locally in the importing file (for alias shadow checking)
    pub fn resolve_all(
        &mut self,
        imports: &[Import],
        base_dir: &Path,
        local_names: &HashSet<String>,
    ) -> Result<ResolvedImports, ImportError> {
        self.local_names = local_names.clone();
        let mut result = ResolvedImports::default();

        for import in imports {
            self.resolve_single(import, base_dir, &mut result)?;
        }

        result.warnings.append(&mut self.warnings);
        Ok(result)
    }

    /// Resolve a single import declaration.
    fn resolve_single(
        &mut self,
        import: &Import,
        base_dir: &Path,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        // Check alias doesn't shadow local names
        if let Some(alias) = &import.alias {
            if self.local_names.contains(alias) {
                return Err(ImportError::AliasShadowsLocal {
                    alias: alias.clone(),
                    name: alias.clone(),
                });
            }
        }

        if import.is_directory_import() {
            self.resolve_directory_import(import, base_dir, result)
        } else {
            self.resolve_file_import(import, base_dir, result)
        }
    }

    /// Resolve a file import (non-directory).
    fn resolve_file_import(
        &mut self,
        import: &Import,
        base_dir: &Path,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let file_path = self.resolve_import_path(&import.from, base_dir)?;
        let canonical = file_path.canonicalize().map_err(|e| ImportError::Io {
            path: file_path.display().to_string(),
            message: e.to_string(),
        })?;

        // Circular detection: importing the file that's doing the importing
        if self.importing_file.as_ref() == Some(&canonical) {
            return Err(ImportError::CircularImport { cycle: import.from.to_string() });
        }

        // Diamond detection: file already imported by a previous import declaration
        if self.visited.contains(&canonical) {
            return Ok(());
        }

        // Mark visited and parse
        self.visited.insert(canonical.clone());
        let items = self.parse_file_items(&canonical)?;

        self.add_items_to_result(import, &items, result)
    }

    /// Resolve a directory import (trailing `/` in path).
    fn resolve_directory_import(
        &mut self,
        import: &Import,
        base_dir: &Path,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let dir_path = self.resolve_dir_path(&import.from, base_dir)?;

        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(ImportError::FileNotFound { path: import.from.clone() });
        }

        // Collect all .pxl and .jsonl files in the directory
        let mut files: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "pxl" || ext == "jsonl" {
                            files.push(path);
                        }
                    }
                }
            }
        }
        files.sort();

        for file_path in &files {
            let canonical = match file_path.canonicalize() {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Skip already-visited files (diamond)
            if self.visited.contains(&canonical) {
                continue;
            }

            self.visited.insert(canonical.clone());
            let items = self.parse_file_items(&canonical)?;

            let file_stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

            // For directory imports, each file's items are namespaced by filename
            let dir_import = Import {
                from: import.from.clone(),
                alias: Some(file_stem.to_string()),
                sprites: import.sprites.clone(),
                palettes: import.palettes.clone(),
                transforms: import.transforms.clone(),
                animations: import.animations.clone(),
            };

            self.add_items_to_result(&dir_import, &items, result)?;
        }

        Ok(())
    }

    /// Resolve an import path to an absolute file path.
    fn resolve_import_path(&self, from: &str, base_dir: &Path) -> Result<PathBuf, ImportError> {
        if from.starts_with("./") || from.starts_with("../") {
            // Relative import: resolve relative to base_dir
            let path = base_dir.join(from);
            self.find_file_with_extensions(&path)
        } else {
            // Root-relative import: resolve relative to src_root
            let src_root = self
                .src_root
                .as_ref()
                .ok_or_else(|| ImportError::NoProjectContext { from: from.to_string() })?;
            let path = src_root.join(from);
            self.find_file_with_extensions(&path)
        }
    }

    /// Resolve a directory path from an import.
    fn resolve_dir_path(&self, from: &str, base_dir: &Path) -> Result<PathBuf, ImportError> {
        let from_trimmed = from.trim_end_matches('/');
        if from_trimmed.starts_with("./") || from_trimmed.starts_with("../") {
            Ok(base_dir.join(from_trimmed))
        } else {
            let src_root = self
                .src_root
                .as_ref()
                .ok_or_else(|| ImportError::NoProjectContext { from: from.to_string() })?;
            Ok(src_root.join(from_trimmed))
        }
    }

    /// Find a file, trying alternate extensions if the exact path doesn't exist.
    ///
    /// Resolution order: exact path, then `.pxl`, then `.jsonl`.
    fn find_file_with_extensions(&self, path: &Path) -> Result<PathBuf, ImportError> {
        if path.exists() && path.is_file() {
            return Ok(path.to_path_buf());
        }

        let pxl_path = path.with_extension("pxl");
        if pxl_path.exists() && pxl_path.is_file() {
            return Ok(pxl_path);
        }

        let jsonl_path = path.with_extension("jsonl");
        if jsonl_path.exists() && jsonl_path.is_file() {
            return Ok(jsonl_path);
        }

        Err(ImportError::FileNotFound { path: path.display().to_string() })
    }

    /// Parse a file and extract items by type.
    fn parse_file_items(&self, file_path: &Path) -> Result<FileItems, ImportError> {
        let file = File::open(file_path).map_err(|e| ImportError::Io {
            path: file_path.display().to_string(),
            message: e.to_string(),
        })?;
        let reader = BufReader::new(file);
        let parse_result = parse_stream(reader);

        let mut items = FileItems::default();
        for obj in parse_result.objects {
            match obj {
                TtpObject::Palette(p) => items.palettes.push(p),
                TtpObject::Sprite(s) => items.sprites.push(s),
                TtpObject::Variant(v) => items.variants.push(v),
                TtpObject::Transform(t) => items.transforms.push(t),
                TtpObject::Composition(c) => items.compositions.push(c),
                TtpObject::Animation(a) => items.animations.push(a),
                // Import declarations in the target file are not transitively
                // resolved (no re-exports). See design doc: "direct imports only."
                _ => {}
            }
        }

        Ok(items)
    }

    /// Check for circular references by inspecting import declarations in
    /// a parsed file (without resolving them).
    /// Add items from a parsed file to the result, applying filtering and aliases.
    fn add_items_to_result(
        &mut self,
        import: &Import,
        items: &FileItems,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let alias = import.alias.as_deref();

        if import.is_selective() {
            // Selective import: only import explicitly requested items
            if let Some(palette_names) = &import.palettes {
                for name in palette_names {
                    let palette =
                        items.palettes.iter().find(|p| p.name == *name).ok_or_else(|| {
                            ImportError::ItemNotFound {
                                item_type: "palette".to_string(),
                                name: name.clone(),
                                from: import.from.clone(),
                            }
                        })?;
                    self.add_palette(palette, alias, result)?;
                }
            }

            if let Some(sprite_names) = &import.sprites {
                for name in sprite_names {
                    let sprite =
                        items.sprites.iter().find(|s| s.name == *name).ok_or_else(|| {
                            ImportError::ItemNotFound {
                                item_type: "sprite".to_string(),
                                name: name.clone(),
                                from: import.from.clone(),
                            }
                        })?;
                    self.add_sprite(sprite, alias, result)?;
                }
            }

            if let Some(transform_names) = &import.transforms {
                for name in transform_names {
                    let transform =
                        items.transforms.iter().find(|t| t.name == *name).ok_or_else(|| {
                            ImportError::ItemNotFound {
                                item_type: "transform".to_string(),
                                name: name.clone(),
                                from: import.from.clone(),
                            }
                        })?;
                    self.add_transform(transform, alias, result)?;
                }
            }

            if let Some(animation_names) = &import.animations {
                for name in animation_names {
                    let animation =
                        items.animations.iter().find(|a| a.name == *name).ok_or_else(|| {
                            ImportError::ItemNotFound {
                                item_type: "animation".to_string(),
                                name: name.clone(),
                                from: import.from.clone(),
                            }
                        })?;
                    self.add_animation(animation, alias, result)?;
                }
            }
        } else {
            // Unfiltered or aliased-only: import all items
            for palette in &items.palettes {
                self.add_palette(palette, alias, result)?;
            }
            for sprite in &items.sprites {
                self.add_sprite(sprite, alias, result)?;
            }
            for variant in &items.variants {
                self.add_variant(variant, alias, result)?;
            }
            for transform in &items.transforms {
                self.add_transform(transform, alias, result)?;
            }
            for composition in &items.compositions {
                self.add_composition(composition, alias, result)?;
            }
            for animation in &items.animations {
                self.add_animation(animation, alias, result)?;
            }
        }

        Ok(())
    }

    /// Compute the effective name for an item (with optional alias prefix).
    fn effective_name(name: &str, alias: Option<&str>) -> String {
        match alias {
            Some(a) => format!("{}:{}", a, name),
            None => name.to_string(),
        }
    }

    /// Add a palette to the result, handling collision detection.
    fn add_palette(
        &mut self,
        palette: &Palette,
        alias: Option<&str>,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let eff_name = Self::effective_name(&palette.name, alias);

        // Local names always win — skip silently
        if self.local_names.contains(&eff_name) {
            return Ok(());
        }

        // Collision with previously imported item
        if !self.imported_palettes.insert(eff_name.clone()) {
            if self.strict {
                return Err(ImportError::Collision {
                    item_type: "palette".to_string(),
                    name: eff_name,
                });
            }
            self.warnings.push(ImportWarning {
                message: format!("Palette '{}' already imported; first import wins", eff_name),
            });
            return Ok(());
        }

        let mut imported = palette.clone();
        imported.name = eff_name;
        result.palettes.push(imported);
        Ok(())
    }

    /// Add a sprite to the result, handling collision detection.
    fn add_sprite(
        &mut self,
        sprite: &Sprite,
        alias: Option<&str>,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let eff_name = Self::effective_name(&sprite.name, alias);

        if self.local_names.contains(&eff_name) {
            return Ok(());
        }

        if !self.imported_sprites.insert(eff_name.clone()) {
            if self.strict {
                return Err(ImportError::Collision {
                    item_type: "sprite".to_string(),
                    name: eff_name,
                });
            }
            self.warnings.push(ImportWarning {
                message: format!("Sprite '{}' already imported; first import wins", eff_name),
            });
            return Ok(());
        }

        let mut imported = sprite.clone();
        imported.name = eff_name;
        result.sprites.push(imported);
        Ok(())
    }

    /// Add a variant to the result, handling collision detection.
    fn add_variant(
        &mut self,
        variant: &Variant,
        alias: Option<&str>,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let eff_name = Self::effective_name(&variant.name, alias);

        if self.local_names.contains(&eff_name) {
            return Ok(());
        }

        if !self.imported_variants.insert(eff_name.clone()) {
            if self.strict {
                return Err(ImportError::Collision {
                    item_type: "variant".to_string(),
                    name: eff_name,
                });
            }
            self.warnings.push(ImportWarning {
                message: format!("Variant '{}' already imported; first import wins", eff_name),
            });
            return Ok(());
        }

        let mut imported = variant.clone();
        imported.name = eff_name;
        result.variants.push(imported);
        Ok(())
    }

    /// Add a transform to the result, handling collision detection.
    fn add_transform(
        &mut self,
        transform: &TransformDef,
        alias: Option<&str>,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let eff_name = Self::effective_name(&transform.name, alias);

        if self.local_names.contains(&eff_name) {
            return Ok(());
        }

        if !self.imported_transforms.insert(eff_name.clone()) {
            if self.strict {
                return Err(ImportError::Collision {
                    item_type: "transform".to_string(),
                    name: eff_name,
                });
            }
            self.warnings.push(ImportWarning {
                message: format!("Transform '{}' already imported; first import wins", eff_name),
            });
            return Ok(());
        }

        let mut imported = transform.clone();
        imported.name = eff_name;
        result.transforms.push(imported);
        Ok(())
    }

    /// Add a composition to the result, handling collision detection.
    fn add_composition(
        &mut self,
        composition: &Composition,
        alias: Option<&str>,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let eff_name = Self::effective_name(&composition.name, alias);

        if self.local_names.contains(&eff_name) {
            return Ok(());
        }

        if !self.imported_compositions.insert(eff_name.clone()) {
            if self.strict {
                return Err(ImportError::Collision {
                    item_type: "composition".to_string(),
                    name: eff_name,
                });
            }
            self.warnings.push(ImportWarning {
                message: format!("Composition '{}' already imported; first import wins", eff_name),
            });
            return Ok(());
        }

        let mut imported = composition.clone();
        imported.name = eff_name;
        result.compositions.push(imported);
        Ok(())
    }

    /// Add an animation to the result, handling collision detection.
    fn add_animation(
        &mut self,
        animation: &Animation,
        alias: Option<&str>,
        result: &mut ResolvedImports,
    ) -> Result<(), ImportError> {
        let eff_name = Self::effective_name(&animation.name, alias);

        if self.local_names.contains(&eff_name) {
            return Ok(());
        }

        if !self.imported_animations.insert(eff_name.clone()) {
            if self.strict {
                return Err(ImportError::Collision {
                    item_type: "animation".to_string(),
                    name: eff_name,
                });
            }
            self.warnings.push(ImportWarning {
                message: format!("Animation '{}' already imported; first import wins", eff_name),
            });
            return Ok(());
        }

        let mut imported = animation.clone();
        imported.name = eff_name;
        result.animations.push(imported);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a file in a temp directory with the given content.
    fn create_file(dir: &Path, relative_path: &str, content: &str) -> PathBuf {
        let path = dir.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    // ========================================================================
    // Basic file import
    // ========================================================================

    #[test]
    fn test_basic_unfiltered_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        // Target file with a palette and a sprite
        create_file(
            &src,
            "shared/colors.pxl",
            concat!(
                r##"{"type": "palette", "name": "gameboy", "colors": {"{bg}": "#0F380F"}}"##,
                "\n",
                r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "gameboy"}"##,
            ),
        );

        let imports = vec![Import {
            from: "shared/colors".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "gameboy");
        assert_eq!(result.sprites.len(), 1);
        assert_eq!(result.sprites[0].name, "dot");
    }

    // ========================================================================
    // Selective import
    // ========================================================================

    #[test]
    fn test_selective_palette_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "palettes.pxl",
            concat!(
                r##"{"type": "palette", "name": "gameboy", "colors": {"{bg}": "#0F380F"}}"##,
                "\n",
                r##"{"type": "palette", "name": "nes", "colors": {"{bg}": "#000000"}}"##,
                "\n",
                r##"{"type": "sprite", "name": "icon", "size": [1, 1], "palette": "gameboy"}"##,
            ),
        );

        let imports = vec![Import {
            from: "palettes".to_string(),
            alias: None,
            sprites: None,
            palettes: Some(vec!["gameboy".to_string()]),
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        // Only the requested palette should be imported
        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "gameboy");
        // Sprites should NOT be imported (selective import only requested palettes)
        assert_eq!(result.sprites.len(), 0);
    }

    #[test]
    fn test_selective_sprite_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "chars.pxl",
            concat!(
                r##"{"type": "sprite", "name": "idle", "size": [8, 8], "palette": {"r": "#F00"}}"##,
                "\n",
                r##"{"type": "sprite", "name": "walk", "size": [8, 8], "palette": {"r": "#F00"}}"##,
                "\n",
                r##"{"type": "sprite", "name": "run", "size": [8, 8], "palette": {"r": "#F00"}}"##,
            ),
        );

        let imports = vec![Import {
            from: "chars".to_string(),
            alias: None,
            sprites: Some(vec!["idle".to_string(), "walk".to_string()]),
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.sprites.len(), 2);
        let names: Vec<&str> = result.sprites.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"idle"));
        assert!(names.contains(&"walk"));
        assert!(!names.contains(&"run"));
    }

    // ========================================================================
    // Aliased import
    // ========================================================================

    #[test]
    fn test_aliased_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "characters/hero/base.pxl",
            concat!(
                r##"{"type": "sprite", "name": "idle", "size": [8, 8], "palette": {"r": "#F00"}}"##,
                "\n",
                r##"{"type": "palette", "name": "skin", "colors": {"{tone}": "#FFCC99"}}"##,
            ),
        );

        let imports = vec![Import {
            from: "characters/hero/base".to_string(),
            alias: Some("hero".to_string()),
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.sprites.len(), 1);
        assert_eq!(result.sprites[0].name, "hero:idle");
        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "hero:skin");
    }

    // ========================================================================
    // Relative imports
    // ========================================================================

    #[test]
    fn test_relative_import_dot_slash() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // File in a subdirectory
        create_file(
            base,
            "sprites/palettes/brand.pxl",
            r##"{"type": "palette", "name": "brand", "colors": {"{accent}": "#FF6600"}}"##,
        );

        let imports = vec![Import {
            from: "./palettes/brand".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        // Relative imports work without src_root
        let mut resolver = ImportResolver::new(None, false);
        let base_dir = base.join("sprites");
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &base_dir, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "brand");
    }

    #[test]
    fn test_relative_import_dot_dot_slash() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        create_file(
            base,
            "shared/colors.pxl",
            r##"{"type": "palette", "name": "shared_pal", "colors": {"{x}": "#FF0000"}}"##,
        );

        let imports = vec![Import {
            from: "../shared/colors".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(None, false);
        let base_dir = base.join("sprites");
        fs::create_dir_all(&base_dir).unwrap();
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &base_dir, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "shared_pal");
    }

    // ========================================================================
    // Root-relative imports
    // ========================================================================

    #[test]
    fn test_root_relative_import_requires_project() {
        let temp = TempDir::new().unwrap();
        let base_dir = temp.path();

        let imports = vec![Import {
            from: "palettes/shared".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        // No src_root -> error
        let mut resolver = ImportResolver::new(None, false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, base_dir, &local_names);

        assert!(result.is_err());
        match result.unwrap_err() {
            ImportError::NoProjectContext { from } => {
                assert_eq!(from, "palettes/shared");
            }
            e => panic!("Expected NoProjectContext, got {:?}", e),
        }
    }

    // ========================================================================
    // Circular import detection
    // ========================================================================

    #[test]
    fn test_circular_import_self() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        let file_path = create_file(
            &src,
            "self_ref.pxl",
            r##"{"type": "palette", "name": "pal", "colors": {"{x}": "#F00"}}"##,
        );

        let imports = vec![Import {
            from: "self_ref".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        // Mark the importing file as visited (simulates: this file imports itself)
        resolver.mark_visited(&file_path);

        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names);

        assert!(result.is_err());
        match result.unwrap_err() {
            ImportError::CircularImport { cycle } => {
                assert!(cycle.contains("self_ref"));
            }
            e => panic!("Expected CircularImport, got {:?}", e),
        }
    }

    // ========================================================================
    // Diamond imports
    // ========================================================================

    #[test]
    fn test_diamond_import_no_duplication() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        // Shared file imported by two separate imports
        create_file(
            &src,
            "shared.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#F00"}}"##,
        );

        let imports = vec![
            Import {
                from: "shared".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
            Import {
                from: "shared".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
        ];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        // Should only have one palette (diamond detection prevents re-parsing)
        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "shared");
    }

    // ========================================================================
    // Import collision handling
    // ========================================================================

    #[test]
    fn test_collision_lenient_first_wins() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "a.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#FF0000"}}"##,
        );
        create_file(
            &src,
            "b.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#00FF00"}}"##,
        );

        let imports = vec![
            Import {
                from: "a".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
            Import {
                from: "b".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
        ];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        // First import wins
        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "shared");
        assert_eq!(result.palettes[0].colors.get("{x}"), Some(&"#FF0000".to_string()));

        // Warning should be produced
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("already imported"));
    }

    #[test]
    fn test_collision_strict_errors() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "a.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#FF0000"}}"##,
        );
        create_file(
            &src,
            "b.pxl",
            r##"{"type": "palette", "name": "shared", "colors": {"{x}": "#00FF00"}}"##,
        );

        let imports = vec![
            Import {
                from: "a".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
            Import {
                from: "b".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
        ];

        let mut resolver = ImportResolver::new(Some(src.clone()), true);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names);

        assert!(result.is_err());
        match result.unwrap_err() {
            ImportError::Collision { item_type, name } => {
                assert_eq!(item_type, "palette");
                assert_eq!(name, "shared");
            }
            e => panic!("Expected Collision, got {:?}", e),
        }
    }

    // ========================================================================
    // Alias cannot shadow local name
    // ========================================================================

    #[test]
    fn test_alias_shadows_local_error() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "other.pxl",
            r##"{"type": "sprite", "name": "idle", "size": [1, 1], "palette": {"x": "#F00"}}"##,
        );

        let imports = vec![Import {
            from: "other".to_string(),
            alias: Some("hero".to_string()),
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let mut local_names = HashSet::new();
        local_names.insert("hero".to_string()); // Local name "hero" exists

        let result = resolver.resolve_all(&imports, &src, &local_names);

        assert!(result.is_err());
        match result.unwrap_err() {
            ImportError::AliasShadowsLocal { alias, .. } => {
                assert_eq!(alias, "hero");
            }
            e => panic!("Expected AliasShadowsLocal, got {:?}", e),
        }
    }

    // ========================================================================
    // Local names win over imported names
    // ========================================================================

    #[test]
    fn test_local_name_wins_over_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "external.pxl",
            r##"{"type": "palette", "name": "colors", "colors": {"{x}": "#00FF00"}}"##,
        );

        let imports = vec![Import {
            from: "external".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let mut local_names = HashSet::new();
        local_names.insert("colors".to_string()); // Same name defined locally

        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        // Imported palette skipped because local name wins
        assert_eq!(result.palettes.len(), 0);
        assert!(result.warnings.is_empty());
    }

    // ========================================================================
    // Missing item error (selective import)
    // ========================================================================

    #[test]
    fn test_selective_missing_item_error() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "file.pxl",
            r##"{"type": "palette", "name": "exists", "colors": {"{x}": "#F00"}}"##,
        );

        let imports = vec![Import {
            from: "file".to_string(),
            alias: None,
            sprites: None,
            palettes: Some(vec!["nonexistent".to_string()]),
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names);

        assert!(result.is_err());
        match result.unwrap_err() {
            ImportError::ItemNotFound { item_type, name, from } => {
                assert_eq!(item_type, "palette");
                assert_eq!(name, "nonexistent");
                assert_eq!(from, "file");
            }
            e => panic!("Expected ItemNotFound, got {:?}", e),
        }
    }

    // ========================================================================
    // File not found error
    // ========================================================================

    #[test]
    fn test_file_not_found_error() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        let imports = vec![Import {
            from: "nonexistent/file".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names);

        assert!(result.is_err());
        matches!(result.unwrap_err(), ImportError::FileNotFound { .. });
    }

    // ========================================================================
    // Extension auto-detection
    // ========================================================================

    #[test]
    fn test_extension_auto_detect_pxl() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "colors.pxl",
            r##"{"type": "palette", "name": "auto_pxl", "colors": {"{x}": "#F00"}}"##,
        );

        let imports = vec![Import {
            from: "colors".to_string(), // No extension
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "auto_pxl");
    }

    #[test]
    fn test_extension_auto_detect_jsonl() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "colors.jsonl",
            r##"{"type": "palette", "name": "auto_jsonl", "colors": {"{x}": "#0F0"}}"##,
        );

        let imports = vec![Import {
            from: "colors".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "auto_jsonl");
    }

    // ========================================================================
    // Directory import
    // ========================================================================

    #[test]
    fn test_directory_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "characters/hero/base.pxl",
            r##"{"type": "sprite", "name": "idle", "size": [8, 8], "palette": {"r": "#F00"}}"##,
        );
        create_file(
            &src,
            "characters/hero/attack.pxl",
            r##"{"type": "sprite", "name": "slash", "size": [8, 8], "palette": {"r": "#F00"}}"##,
        );

        let imports = vec![Import {
            from: "characters/hero/".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        // Directory import namespaces items by file stem
        assert_eq!(result.sprites.len(), 2);
        let names: Vec<&str> = result.sprites.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"attack:slash"));
        assert!(names.contains(&"base:idle"));
    }

    // ========================================================================
    // Mixed selective + aliased
    // ========================================================================

    #[test]
    fn test_selective_with_alias() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "palettes/retro.pxl",
            concat!(
                r##"{"type": "palette", "name": "gameboy", "colors": {"{bg}": "#0F380F"}}"##,
                "\n",
                r##"{"type": "palette", "name": "nes", "colors": {"{bg}": "#000000"}}"##,
            ),
        );

        let imports = vec![Import {
            from: "palettes/retro".to_string(),
            alias: Some("retro".to_string()),
            sprites: None,
            palettes: Some(vec!["gameboy".to_string()]),
            transforms: None,
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 1);
        assert_eq!(result.palettes[0].name, "retro:gameboy");
    }

    // ========================================================================
    // Transform import
    // ========================================================================

    #[test]
    fn test_transform_import() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "transforms/motion.pxl",
            r#"{"type": "transform", "name": "bounce", "ops": [{"op": "translate", "y": -4}]}"#,
        );

        let imports = vec![Import {
            from: "transforms/motion".to_string(),
            alias: None,
            sprites: None,
            palettes: None,
            transforms: Some(vec!["bounce".to_string()]),
            animations: None,
        }];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.transforms.len(), 1);
        assert_eq!(result.transforms[0].name, "bounce");
    }

    // ========================================================================
    // Multiple imports from different files
    // ========================================================================

    #[test]
    fn test_multiple_imports() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/pxl");
        fs::create_dir_all(&src).unwrap();

        create_file(
            &src,
            "palettes/dark.pxl",
            r##"{"type": "palette", "name": "dark", "colors": {"{bg}": "#111"}}"##,
        );
        create_file(
            &src,
            "palettes/light.pxl",
            r##"{"type": "palette", "name": "light", "colors": {"{bg}": "#EEE"}}"##,
        );
        create_file(
            &src,
            "sprites/hero.pxl",
            r##"{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "dark"}"##,
        );

        let imports = vec![
            Import {
                from: "palettes/dark".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
            Import {
                from: "palettes/light".to_string(),
                alias: None,
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
            Import {
                from: "sprites/hero".to_string(),
                alias: Some("hero".to_string()),
                sprites: None,
                palettes: None,
                transforms: None,
                animations: None,
            },
        ];

        let mut resolver = ImportResolver::new(Some(src.clone()), false);
        let local_names = HashSet::new();
        let result = resolver.resolve_all(&imports, &src, &local_names).unwrap();

        assert_eq!(result.palettes.len(), 2);
        assert_eq!(result.sprites.len(), 1);
        assert_eq!(result.sprites[0].name, "hero:hero");
    }
}
