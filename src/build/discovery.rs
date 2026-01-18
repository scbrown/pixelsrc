//! Source file discovery for the build system.
//!
//! Discovers `.pxl` and `.jsonl` source files based on glob patterns
//! from the configuration.

use crate::build::{BuildContext, BuildPlan, BuildTarget};
use crate::config::AtlasConfig;
use glob::glob;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Error during source discovery.
#[derive(Debug)]
pub enum DiscoveryError {
    /// Invalid glob pattern
    InvalidPattern(String, glob::PatternError),
    /// IO error during file enumeration
    Io(std::io::Error),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::InvalidPattern(pattern, err) => {
                write!(f, "Invalid glob pattern '{}': {}", pattern, err)
            }
            DiscoveryError::Io(err) => write!(f, "IO error during discovery: {}", err),
        }
    }
}

impl std::error::Error for DiscoveryError {}

impl From<std::io::Error> for DiscoveryError {
    fn from(err: std::io::Error) -> Self {
        DiscoveryError::Io(err)
    }
}

/// Discover source files matching a glob pattern.
///
/// # Arguments
/// - `base_dir` - Base directory to resolve patterns from
/// - `pattern` - Glob pattern to match
///
/// # Returns
/// List of matching file paths.
pub fn discover_files(base_dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, DiscoveryError> {
    let full_pattern = base_dir.join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    let paths =
        glob(&pattern_str).map_err(|e| DiscoveryError::InvalidPattern(pattern.to_string(), e))?;

    let mut files = Vec::new();
    for entry in paths {
        match entry {
            Ok(path) => {
                if path.is_file() && is_pxl_file(&path) {
                    files.push(path);
                }
            }
            Err(e) => {
                // Log but continue on glob errors
                eprintln!("Warning: error reading path: {}", e);
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Check if a path is a pixelsrc source file.
fn is_pxl_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("pxl") | Some("jsonl") => true,
        _ => false,
    }
}

/// Discover all source files from config patterns.
///
/// Returns a deduplicated list of all source files found.
pub fn discover_all_sources(ctx: &BuildContext) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut all_files = HashSet::new();
    let src_dir = ctx.src_dir();

    // Discover files from atlas configs
    for atlas_config in ctx.config().atlases.values() {
        for pattern in &atlas_config.sources {
            let files = discover_files(&src_dir, pattern)?;
            all_files.extend(files);
        }
    }

    // Discover files from animation config
    for pattern in &ctx.config().animations.sources {
        let files = discover_files(&src_dir, pattern)?;
        all_files.extend(files);
    }

    // If no patterns defined, discover all pxl files in src
    if all_files.is_empty() {
        let files = discover_files(&src_dir, "**/*.pxl")?;
        all_files.extend(files);
        let files = discover_files(&src_dir, "**/*.jsonl")?;
        all_files.extend(files);
    }

    let mut result: Vec<_> = all_files.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Create a build plan from the configuration and discovered sources.
///
/// Analyzes the configuration to create build targets for:
/// - Atlases defined in `[atlases.*]`
/// - Animations matching animation patterns
/// - Exports for enabled export targets
pub fn create_build_plan(ctx: &BuildContext) -> Result<BuildPlan, DiscoveryError> {
    let mut plan = BuildPlan::new();
    let src_dir = ctx.src_dir();
    let out_dir = ctx.out_dir();

    // Create atlas targets
    for (name, atlas_config) in &ctx.config().atlases {
        let sources = discover_atlas_sources(&src_dir, atlas_config)?;
        if !sources.is_empty() {
            let output = out_dir.join(format!("{}.png", name));
            plan.add_target(BuildTarget::atlas(name.clone(), sources, output));

            // Add export targets for this atlas
            add_export_targets(&mut plan, ctx, name);
        }
    }

    // Create animation targets
    let anim_sources = discover_animation_sources(ctx)?;
    for source in anim_sources {
        let name = source.file_stem().and_then(|s| s.to_str()).unwrap_or("unnamed").to_string();

        let output = out_dir.join("animations").join(format!("{}.png", name));
        plan.add_target(BuildTarget::animation(name.clone(), source.clone(), output.clone()));

        // Add preview target if enabled
        if ctx.config().animations.preview {
            let preview_output = out_dir.join("animations").join(format!("{}.gif", name));
            plan.add_target(BuildTarget::animation_preview(name, source, preview_output));
        }
    }

    Ok(plan)
}

/// Discover sources for an atlas configuration.
fn discover_atlas_sources(
    src_dir: &Path,
    config: &AtlasConfig,
) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut all_files = HashSet::new();

    for pattern in &config.sources {
        let files = discover_files(src_dir, pattern)?;
        all_files.extend(files);
    }

    let mut result: Vec<_> = all_files.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Discover animation source files.
fn discover_animation_sources(ctx: &BuildContext) -> Result<Vec<PathBuf>, DiscoveryError> {
    let src_dir = ctx.src_dir();
    let mut all_files = HashSet::new();

    for pattern in &ctx.config().animations.sources {
        let files = discover_files(&src_dir, pattern)?;
        all_files.extend(files);
    }

    let mut result: Vec<_> = all_files.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Add export targets for an atlas.
fn add_export_targets(plan: &mut BuildPlan, ctx: &BuildContext, atlas_name: &str) {
    let out_dir = ctx.out_dir();
    let exports = &ctx.config().exports;

    // Generic JSON export
    if exports.generic.enabled {
        let output = out_dir.join(format!("{}.json", atlas_name));
        let target = BuildTarget::export(atlas_name.to_string(), "json".to_string(), output)
            .with_dependency(format!("atlas:{}", atlas_name));
        plan.add_target(target);
    }

    // Godot export
    if exports.godot.enabled {
        let output = out_dir.join("godot").join(format!("{}.tres", atlas_name));
        let target = BuildTarget::export(atlas_name.to_string(), "godot".to_string(), output)
            .with_dependency(format!("atlas:{}", atlas_name));
        plan.add_target(target);
    }

    // Unity export
    if exports.unity.enabled {
        let output = out_dir.join("unity").join(format!("{}.asset", atlas_name));
        let target = BuildTarget::export(atlas_name.to_string(), "unity".to_string(), output)
            .with_dependency(format!("atlas:{}", atlas_name));
        plan.add_target(target);
    }

    // libGDX export
    if exports.libgdx.enabled {
        let output = out_dir.join("libgdx").join(format!("{}.atlas", atlas_name));
        let target = BuildTarget::export(atlas_name.to_string(), "libgdx".to_string(), output)
            .with_dependency(format!("atlas:{}", atlas_name));
        plan.add_target(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        File::create(&path).unwrap().write_all(b"{}").unwrap();
        path
    }

    #[test]
    fn test_discover_files_simple() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), "sprite.pxl");
        create_test_file(temp.path(), "other.txt");

        let files = discover_files(temp.path(), "*.pxl").unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("sprite.pxl"));
    }

    #[test]
    fn test_discover_files_recursive() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), "a.pxl");
        create_test_file(temp.path(), "sub/b.pxl");
        create_test_file(temp.path(), "sub/deep/c.pxl");

        let files = discover_files(temp.path(), "**/*.pxl").unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_discover_files_jsonl() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), "sprite.jsonl");

        let files = discover_files(temp.path(), "*.jsonl").unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_discover_files_no_match() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), "sprite.png");

        let files = discover_files(temp.path(), "*.pxl").unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_is_pxl_file() {
        assert!(is_pxl_file(Path::new("sprite.pxl")));
        assert!(is_pxl_file(Path::new("sprite.jsonl")));
        assert!(!is_pxl_file(Path::new("sprite.png")));
        assert!(!is_pxl_file(Path::new("sprite.json")));
    }

    #[test]
    fn test_build_plan_creation() {
        use crate::config::default_config;

        let temp = TempDir::new().unwrap();
        let config = default_config();
        let ctx = BuildContext::new(config, temp.path().to_path_buf());

        // Create source directory
        let src_dir = temp.path().join("src/pxl");
        fs::create_dir_all(&src_dir).unwrap();

        let plan = create_build_plan(&ctx).unwrap();
        // With default config and no sources, plan should be empty
        assert!(plan.is_empty());
    }
}
