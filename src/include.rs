//! External palette file inclusion support
//!
//! Supports the `@include:path` syntax for including palettes from external files.
//! Paths are resolved relative to the including file's directory.

use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::models::{Palette, TtpObject};
use crate::parser::parse_stream;

/// Error type for include resolution failures.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum IncludeError {
    /// Circular include detected
    #[error("Circular include detected: {}", .0.display())]
    CircularInclude(PathBuf),
    /// File not found
    #[error("Include file not found '{}': {1}", .0.display())]
    FileNotFound(PathBuf, String),
    /// No palette found in included file
    #[error("No palette found in included file: {}", .0.display())]
    NoPaletteFound(PathBuf),
    /// IO error reading file
    #[error("Error reading include file '{}': {1}", .0.display())]
    IoError(PathBuf, String),
}

/// The prefix for include syntax
pub const INCLUDE_PREFIX: &str = "@include:";

/// Check if a palette reference is an include reference.
pub fn is_include_ref(palette_ref: &str) -> bool {
    palette_ref.starts_with(INCLUDE_PREFIX)
}

/// Extract the path from an include reference.
///
/// Returns `None` if the reference is not an include reference.
pub fn extract_include_path(palette_ref: &str) -> Option<&str> {
    palette_ref.strip_prefix(INCLUDE_PREFIX)
}

/// Resolve an include reference to a Palette.
///
/// # Arguments
///
/// * `include_path` - The path string after `@include:` (e.g., "./shared/colors.jsonl")
/// * `base_path` - The directory containing the file with the @include reference
///
/// # Returns
///
/// The first palette found in the included file, or an error.
///
/// # Example
///
/// ```ignore
/// // Given: file at /project/sprites.jsonl with "@include:shared/palette.jsonl"
/// // base_path would be /project/
/// // include_path would be "shared/palette.jsonl"
/// // Resolves to: /project/shared/palette.jsonl
/// ```
pub fn resolve_include(include_path: &str, base_path: &Path) -> Result<Palette, IncludeError> {
    let mut visited = HashSet::new();
    resolve_include_with_detection(include_path, base_path, &mut visited)
}

/// Resolve a path, trying alternate extensions if the exact path doesn't exist.
///
/// Tries paths in order:
/// 1. Exact path as specified
/// 2. Path with .pxl extension
/// 3. Path with .jsonl extension
fn resolve_path_with_extensions(path: &Path) -> Option<PathBuf> {
    // Try exact path first
    if path.exists() {
        return Some(path.to_path_buf());
    }

    // Try alternate extensions
    let alternates = [path.with_extension("pxl"), path.with_extension("jsonl")];

    for alt in &alternates {
        if alt.exists() {
            return Some(alt.clone());
        }
    }

    None
}

/// Resolve an include reference with circular include detection.
///
/// This is the internal implementation that tracks visited files to detect cycles.
pub fn resolve_include_with_detection(
    include_path: &str,
    base_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Palette, IncludeError> {
    // Resolve the include path relative to the base path
    let resolved_path = base_path.join(include_path);

    // Try to find the file, using extension fallback if needed
    let found_path = resolve_path_with_extensions(&resolved_path).ok_or_else(|| {
        IncludeError::FileNotFound(
            resolved_path.clone(),
            "file not found (tried .pxl and .jsonl extensions)".to_string(),
        )
    })?;

    // Canonicalize for consistent comparison (handles .., symlinks, etc.)
    let canonical_path = found_path
        .canonicalize()
        .map_err(|e| IncludeError::FileNotFound(found_path.clone(), e.to_string()))?;

    // Check for circular includes
    if visited.contains(&canonical_path) {
        return Err(IncludeError::CircularInclude(canonical_path));
    }

    // Mark this file as being processed
    visited.insert(canonical_path.clone());

    // Open and parse the included file
    let file = File::open(&canonical_path)
        .map_err(|e| IncludeError::IoError(canonical_path.clone(), e.to_string()))?;

    let reader = BufReader::new(file);
    let parse_result = parse_stream(reader);

    // Get the directory of the included file for nested includes
    // (reserved for future nested include support)
    let _include_dir = canonical_path.parent().unwrap_or(Path::new("."));

    // Find the first palette in the included file
    for obj in parse_result.objects {
        if let TtpObject::Palette(palette) = obj {
            return Ok(palette);
        }
        // Handle sprites that might have @include palette refs
        // (but we're looking for palette objects, not sprite palette refs)
    }

    Err(IncludeError::NoPaletteFound(canonical_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_is_include_ref() {
        assert!(is_include_ref("@include:path/to/file.jsonl"));
        assert!(is_include_ref("@include:./relative.jsonl"));
        assert!(!is_include_ref("@gameboy"));
        assert!(!is_include_ref("regular_palette"));
        assert!(!is_include_ref(""));
    }

    #[test]
    fn test_extract_include_path() {
        assert_eq!(extract_include_path("@include:path/to/file.jsonl"), Some("path/to/file.jsonl"));
        assert_eq!(extract_include_path("@include:./relative.jsonl"), Some("./relative.jsonl"));
        assert_eq!(extract_include_path("@gameboy"), None);
        assert_eq!(extract_include_path("regular"), None);
    }

    #[test]
    fn test_resolve_include_simple() {
        let temp_dir = TempDir::new().unwrap();
        let palette_path = temp_dir.path().join("palette.jsonl");

        // Create a palette file
        let mut file = fs::File::create(&palette_path).unwrap();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{_}": "#00000000", "{x}": "#FF0000"}}"##;
        writeln!(file, "{}", content).unwrap();

        let result = resolve_include("palette.jsonl", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "test");
        assert!(palette.colors.contains_key("{x}"));
    }

    #[test]
    fn test_resolve_include_file_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let result = resolve_include("nonexistent.jsonl", temp_dir.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            IncludeError::FileNotFound(_, _) => {}
            other => panic!("Expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_include_no_palette() {
        let temp_dir = TempDir::new().unwrap();
        let sprite_path = temp_dir.path().join("sprite.jsonl");

        // Create a file with only a sprite (no palette)
        let mut file = fs::File::create(&sprite_path).unwrap();
        let content = r##"{"type": "sprite", "name": "test", "size": [1, 1], "palette": {"x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;
        writeln!(file, "{}", content).unwrap();

        let result = resolve_include("sprite.jsonl", temp_dir.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            IncludeError::NoPaletteFound(_) => {}
            other => panic!("Expected NoPaletteFound, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_include_circular_detection() {
        let temp_dir = TempDir::new().unwrap();

        // For this test, we simulate circular detection by checking the same file twice
        let palette_path = temp_dir.path().join("palette.jsonl");
        let mut file = fs::File::create(&palette_path).unwrap();
        let content = r##"{"type": "palette", "name": "test", "colors": {"{x}": "#FF0000"}}"##;
        writeln!(file, "{}", content).unwrap();

        let mut visited = HashSet::new();

        // First resolution should succeed
        let result1 =
            resolve_include_with_detection("palette.jsonl", temp_dir.path(), &mut visited);
        assert!(result1.is_ok());

        // Second resolution of same file should detect circular include
        let result2 =
            resolve_include_with_detection("palette.jsonl", temp_dir.path(), &mut visited);
        assert!(result2.is_err());

        match result2.unwrap_err() {
            IncludeError::CircularInclude(_) => {}
            other => panic!("Expected CircularInclude, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_include_relative_path() {
        let temp_dir = TempDir::new().unwrap();

        // Create a subdirectory
        let sub_dir = temp_dir.path().join("shared");
        fs::create_dir(&sub_dir).unwrap();

        let palette_path = sub_dir.join("colors.jsonl");
        let mut file = fs::File::create(&palette_path).unwrap();
        let content =
            r##"{"type": "palette", "name": "shared_colors", "colors": {"{a}": "#AA0000"}}"##;
        writeln!(file, "{}", content).unwrap();

        // Resolve from parent directory
        let result = resolve_include("shared/colors.jsonl", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "shared_colors");
    }

    #[test]
    fn test_resolve_include_pxl_extension() {
        let temp_dir = TempDir::new().unwrap();
        let palette_path = temp_dir.path().join("palette.pxl");

        // Create a palette file with .pxl extension
        let mut file = fs::File::create(&palette_path).unwrap();
        let content = r##"{"type": "palette", "name": "pxl_palette", "colors": {"{_}": "#00000000", "{x}": "#00FF00"}}"##;
        writeln!(file, "{}", content).unwrap();

        // Resolve with explicit .pxl extension
        let result = resolve_include("palette.pxl", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "pxl_palette");
        assert!(palette.colors.contains_key("{x}"));
    }

    #[test]
    fn test_resolve_include_extension_auto_detect_pxl() {
        let temp_dir = TempDir::new().unwrap();
        let palette_path = temp_dir.path().join("colors.pxl");

        // Create a palette file with .pxl extension
        let mut file = fs::File::create(&palette_path).unwrap();
        let content = r##"{"type": "palette", "name": "auto_pxl", "colors": {"{a}": "#0000FF"}}"##;
        writeln!(file, "{}", content).unwrap();

        // Resolve without extension - should find .pxl
        let result = resolve_include("colors", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "auto_pxl");
    }

    #[test]
    fn test_resolve_include_extension_auto_detect_jsonl() {
        let temp_dir = TempDir::new().unwrap();
        let palette_path = temp_dir.path().join("colors.jsonl");

        // Create a palette file with .jsonl extension
        let mut file = fs::File::create(&palette_path).unwrap();
        let content =
            r##"{"type": "palette", "name": "auto_jsonl", "colors": {"{b}": "#FF00FF"}}"##;
        writeln!(file, "{}", content).unwrap();

        // Resolve without extension - should find .jsonl
        let result = resolve_include("colors", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "auto_jsonl");
    }

    #[test]
    fn test_resolve_include_extension_priority_pxl_first() {
        let temp_dir = TempDir::new().unwrap();

        // Create both .pxl and .jsonl files
        let pxl_path = temp_dir.path().join("colors.pxl");
        let mut pxl_file = fs::File::create(&pxl_path).unwrap();
        writeln!(pxl_file, r##"{{"type": "palette", "name": "pxl_wins", "colors": {{}}}}"##)
            .unwrap();

        let jsonl_path = temp_dir.path().join("colors.jsonl");
        let mut jsonl_file = fs::File::create(&jsonl_path).unwrap();
        writeln!(jsonl_file, r##"{{"type": "palette", "name": "jsonl_loses", "colors": {{}}}}"##)
            .unwrap();

        // Resolve without extension - .pxl should be preferred
        let result = resolve_include("colors", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "pxl_wins");
    }

    #[test]
    fn test_resolve_include_subdirectory_pxl() {
        let temp_dir = TempDir::new().unwrap();

        // Create a subdirectory
        let sub_dir = temp_dir.path().join("shared");
        fs::create_dir(&sub_dir).unwrap();

        let palette_path = sub_dir.join("colors.pxl");
        let mut file = fs::File::create(&palette_path).unwrap();
        let content =
            r##"{"type": "palette", "name": "shared_pxl", "colors": {"{c}": "#CCCCCC"}}"##;
        writeln!(file, "{}", content).unwrap();

        // Resolve with .pxl extension
        let result = resolve_include("shared/colors.pxl", temp_dir.path());
        assert!(result.is_ok());

        let palette = result.unwrap();
        assert_eq!(palette.name, "shared_pxl");

        // Also test without extension
        let result2 = resolve_include("shared/colors", temp_dir.path());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().name, "shared_pxl");
    }
}
