//! Configuration loading and discovery for `pxl.toml`
//!
//! Provides functions to find, load, and merge configuration.

use super::schema::{
    AnimationsConfig, DefaultsConfig, ExportsConfig, FormatConfig, ImportConfig, ProjectConfig,
    PxlConfig, TelemetryConfig, ValidateConfig, WatchConfig,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration loading error
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// File I/O error
    #[error("Failed to read config: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing error
    #[error("Failed to parse pxl.toml: {0}")]
    Parse(#[from] toml::de::Error),
    /// Validation error
    #[error("Config validation failed:\n{}", .0.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n"))]
    Validation(Vec<String>),
}

/// CLI arguments that can override config values
#[derive(Debug, Default, Clone)]
pub struct CliOverrides {
    /// Override output directory
    pub out: Option<PathBuf>,
    /// Override source directory
    pub src: Option<PathBuf>,
    /// Override scale factor
    pub scale: Option<u32>,
    /// Override padding
    pub padding: Option<u32>,
    /// Build specific atlas only
    pub atlas: Option<String>,
    /// Build specific export format only
    pub export: Option<String>,
    /// Enable strict validation
    pub strict: Option<bool>,
    /// Allow region overflow
    pub allow_overflow: Option<bool>,
    /// Allow orphan tokens
    pub allow_orphans: Option<bool>,
    /// Allow circular dependencies
    pub allow_cycles: Option<bool>,
    /// Enable error collection
    pub collect_errors: Option<bool>,
    /// Number of parallel jobs
    pub jobs: Option<usize>,
}

/// Find pxl.toml by walking up from the current working directory.
///
/// Search order:
/// 1. Walk up from current directory looking for pxl.toml
/// 2. Check XDG_CONFIG_HOME/pixelsrc/pxl.toml (or ~/.config/pixelsrc/pxl.toml)
///
/// # Returns
/// - `Some(path)` if a pxl.toml file is found
/// - `None` if no config file is found
///
/// # Example
/// ```ignore
/// if let Some(config_path) = find_config() {
///     println!("Found config at: {}", config_path.display());
/// }
/// ```
pub fn find_config() -> Option<PathBuf> {
    // First try walking up from current directory
    if let Ok(cwd) = env::current_dir() {
        if let Some(path) = find_config_from(cwd) {
            return Some(path);
        }
    }

    // Fall back to XDG config
    find_xdg_config()
}

/// Find pxl.toml in XDG config directory.
///
/// Checks XDG_CONFIG_HOME/pixelsrc/pxl.toml or ~/.config/pixelsrc/pxl.toml
pub fn find_xdg_config() -> Option<PathBuf> {
    let xdg_config = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok()?;

    let config_path = xdg_config.join("pixelsrc").join("pxl.toml");
    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
}

/// Find pxl.toml by walking up from a specific directory.
///
/// This is the internal implementation that allows specifying the start directory,
/// useful for testing.
pub fn find_config_from(start: PathBuf) -> Option<PathBuf> {
    let mut current = start;

    loop {
        let config_path = current.join("pxl.toml");
        if config_path.exists() {
            return Some(config_path);
        }

        // Move to parent directory
        if !current.pop() {
            // Reached root, no config found
            return None;
        }
    }
}

/// Load configuration from a pxl.toml file.
///
/// If a path is provided, loads from that file. Otherwise, uses `find_config()`
/// to locate the config file. If no config file is found, returns a default
/// configuration.
///
/// # Arguments
/// - `path` - Optional path to a pxl.toml file
///
/// # Returns
/// - `Ok(PxlConfig)` on success
/// - `Err(ConfigError)` if the file cannot be read or parsed
///
/// # Example
/// ```ignore
/// // Load from discovered config
/// let config = load_config(None)?;
///
/// // Load from specific path
/// let config = load_config(Some(Path::new("my-project/pxl.toml")))?;
/// ```
pub fn load_config(path: Option<&Path>) -> Result<PxlConfig, ConfigError> {
    let config_path = match path {
        Some(p) => Some(p.to_path_buf()),
        None => find_config(),
    };

    match config_path {
        Some(p) => load_config_file(&p),
        None => Ok(default_config()),
    }
}

/// Load configuration from a specific file path.
fn load_config_file(path: &Path) -> Result<PxlConfig, ConfigError> {
    let contents = fs::read_to_string(path)?;
    let config: PxlConfig = toml::from_str(&contents)?;

    // Validate the config
    let errors = config.validate();
    if !errors.is_empty() {
        return Err(ConfigError::Validation(errors.into_iter().map(|e| e.to_string()).collect()));
    }

    Ok(config)
}

/// Create a default configuration when no pxl.toml is found.
///
/// Returns a minimal valid configuration with the project name set to
/// the current directory name.
pub fn default_config() -> PxlConfig {
    let project_name = env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unnamed".to_string());

    PxlConfig {
        project: ProjectConfig {
            name: project_name,
            version: "0.1.0".to_string(),
            src: PathBuf::from("src/pxl"),
            out: PathBuf::from("build"),
        },
        format: FormatConfig::default(),
        import_config: ImportConfig::default(),
        telemetry: TelemetryConfig::default(),
        defaults: DefaultsConfig::default(),
        atlases: HashMap::new(),
        animations: AnimationsConfig::default(),
        exports: ExportsConfig::default(),
        validate: ValidateConfig::default(),
        watch: WatchConfig::default(),
        dependencies: HashMap::new(),
    }
}

/// Merge CLI overrides into a configuration.
///
/// CLI arguments take precedence over config file values.
///
/// # Arguments
/// - `config` - The configuration to modify
/// - `overrides` - CLI overrides to apply
///
/// # Example
/// ```ignore
/// let mut config = load_config(None)?;
/// let overrides = CliOverrides {
///     out: Some(PathBuf::from("dist")),
///     strict: Some(true),
///     ..Default::default()
/// };
/// merge_cli_overrides(&mut config, &overrides);
/// ```
pub fn merge_cli_overrides(config: &mut PxlConfig, overrides: &CliOverrides) {
    // Override output directory
    if let Some(ref out) = overrides.out {
        config.project.out = out.clone();
    }

    // Override source directory
    if let Some(ref src) = overrides.src {
        config.project.src = src.clone();
    }

    // Override scale
    if let Some(scale) = overrides.scale {
        config.defaults.scale = scale;
    }

    // Override padding
    if let Some(padding) = overrides.padding {
        config.defaults.padding = padding;
    }

    // Override strict mode
    if let Some(strict) = overrides.strict {
        config.validate.strict = strict;
    }

    // Override allow flags (these override strict mode for specific checks)
    if let Some(allow_overflow) = overrides.allow_overflow {
        config.validate.allow_overflow = allow_overflow;
    }
    if let Some(allow_orphans) = overrides.allow_orphans {
        config.validate.allow_orphans = allow_orphans;
    }
    if let Some(allow_cycles) = overrides.allow_cycles {
        config.validate.allow_cycles = allow_cycles;
    }

    // Override error collection
    if let Some(collect_errors) = overrides.collect_errors {
        config.telemetry.collect_errors = collect_errors;
    }
}

/// Get the project root directory from a config file path.
///
/// Returns the parent directory of the pxl.toml file.
pub fn project_root(config_path: &Path) -> Option<&Path> {
    config_path.parent()
}

/// Resolve a path relative to the project root.
///
/// If the path is absolute, returns it unchanged.
/// If relative, joins it with the project root.
pub fn resolve_path(project_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_find_config_in_current_dir() {
        let temp = TempDir::new().expect("should create temp dir");
        let config_path = temp.path().join("pxl.toml");
        File::create(&config_path)
            .expect("should create config file")
            .write_all(b"[project]\nname = \"test\"")
            .expect("should write config content");

        let found = find_config_from(temp.path().to_path_buf());
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_config_in_parent_dir() {
        let temp = TempDir::new().expect("should create temp dir");
        let config_path = temp.path().join("pxl.toml");
        File::create(&config_path)
            .expect("should create config file")
            .write_all(b"[project]\nname = \"test\"")
            .expect("should write config content");

        // Create a subdirectory
        let subdir = temp.path().join("src").join("sprites");
        fs::create_dir_all(&subdir).expect("should create subdirectories");

        let found = find_config_from(subdir);
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_find_config_not_found() {
        let temp = TempDir::new().expect("should create temp dir");
        let found = find_config_from(temp.path().to_path_buf());
        assert_eq!(found, None);
    }

    #[test]
    fn test_load_config_from_file() {
        let temp = TempDir::new().expect("should create temp dir");
        let config_path = temp.path().join("pxl.toml");
        File::create(&config_path)
            .expect("should create config file")
            .write_all(
                br#"
[project]
name = "test-project"
version = "2.0.0"

[defaults]
scale = 3
padding = 2

[atlases.main]
sources = ["sprites/**"]
max_size = [512, 512]
"#,
            )
            .expect("should write config content");

        let config = load_config(Some(&config_path)).expect("should load valid config");
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.project.version, "2.0.0");
        assert_eq!(config.defaults.scale, 3);
        assert_eq!(config.defaults.padding, 2);
        assert!(config.atlases.contains_key("main"));
    }

    #[test]
    fn test_load_config_missing_file_uses_defaults() {
        let temp = TempDir::new().expect("should create temp dir");
        let config_path = temp.path().join("nonexistent.toml");

        // When file doesn't exist, load_config with explicit path should error
        let result = load_config(Some(&config_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_no_path_no_file_uses_defaults() {
        // When no config is found via find_config_from, default_config() is used
        let temp = TempDir::new().expect("should create temp dir");

        // find_config_from returns None when no pxl.toml exists
        let found = find_config_from(temp.path().to_path_buf());
        assert!(found.is_none());

        // default_config should return sensible defaults
        let config = default_config();
        assert_eq!(config.project.src, PathBuf::from("src/pxl"));
        assert_eq!(config.project.out, PathBuf::from("build"));
        assert_eq!(config.defaults.scale, 1);
        assert_eq!(config.defaults.padding, 1);
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let temp = TempDir::new().expect("should create temp dir");
        let config_path = temp.path().join("pxl.toml");
        File::create(&config_path)
            .expect("should create config file")
            .write_all(b"this is not valid toml {{{")
            .expect("should write invalid config");

        let result = load_config(Some(&config_path));
        assert!(matches!(result, Err(ConfigError::Parse(_))));
    }

    #[test]
    fn test_load_config_validation_error() {
        let temp = TempDir::new().expect("should create temp dir");
        let config_path = temp.path().join("pxl.toml");
        File::create(&config_path)
            .expect("should create config file")
            .write_all(
                br#"
[project]
name = ""

[defaults]
scale = 0
"#,
            )
            .expect("should write invalid config");

        let result = load_config(Some(&config_path));
        assert!(matches!(result, Err(ConfigError::Validation(_))));
    }

    #[test]
    fn test_merge_cli_overrides_out() {
        let mut config = default_config();
        let overrides = CliOverrides { out: Some(PathBuf::from("dist")), ..Default::default() };

        merge_cli_overrides(&mut config, &overrides);
        assert_eq!(config.project.out, PathBuf::from("dist"));
    }

    #[test]
    fn test_merge_cli_overrides_src() {
        let mut config = default_config();
        let overrides =
            CliOverrides { src: Some(PathBuf::from("assets/pxl")), ..Default::default() };

        merge_cli_overrides(&mut config, &overrides);
        assert_eq!(config.project.src, PathBuf::from("assets/pxl"));
    }

    #[test]
    fn test_merge_cli_overrides_scale() {
        let mut config = default_config();
        let overrides = CliOverrides { scale: Some(4), ..Default::default() };

        merge_cli_overrides(&mut config, &overrides);
        assert_eq!(config.defaults.scale, 4);
    }

    #[test]
    fn test_merge_cli_overrides_strict() {
        let mut config = default_config();
        assert!(!config.validate.strict);

        let overrides = CliOverrides { strict: Some(true), ..Default::default() };

        merge_cli_overrides(&mut config, &overrides);
        assert!(config.validate.strict);
    }

    #[test]
    fn test_merge_cli_overrides_multiple() {
        let mut config = default_config();
        let overrides = CliOverrides {
            out: Some(PathBuf::from("output")),
            scale: Some(2),
            padding: Some(4),
            strict: Some(true),
            ..Default::default()
        };

        merge_cli_overrides(&mut config, &overrides);
        assert_eq!(config.project.out, PathBuf::from("output"));
        assert_eq!(config.defaults.scale, 2);
        assert_eq!(config.defaults.padding, 4);
        assert!(config.validate.strict);
    }

    #[test]
    fn test_resolve_path_absolute() {
        let root = Path::new("/project");
        let absolute = Path::new("/other/path");
        assert_eq!(resolve_path(root, absolute), PathBuf::from("/other/path"));
    }

    #[test]
    fn test_resolve_path_relative() {
        let root = Path::new("/project");
        let relative = Path::new("src/pxl");
        assert_eq!(resolve_path(root, relative), PathBuf::from("/project/src/pxl"));
    }

    #[test]
    fn test_project_root() {
        let config_path = Path::new("/project/pxl.toml");
        assert_eq!(project_root(config_path), Some(Path::new("/project")));
    }

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert!(!config.project.name.is_empty());
        assert_eq!(config.project.version, "0.1.0");
        assert_eq!(config.project.src, PathBuf::from("src/pxl"));
        assert_eq!(config.project.out, PathBuf::from("build"));
    }
}
