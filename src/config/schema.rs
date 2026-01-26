//! Configuration schema types for `pxl.toml`
//!
//! Defines the structure and validation rules for pixelsrc project configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Format version configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Format version (default: 2 for structured regions)
    #[serde(default = "default_format_version")]
    pub version: u32,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { version: default_format_version() }
    }
}

fn default_format_version() -> u32 {
    2
}

/// Import settings for PNG-to-pxl conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    /// Confidence threshold for color clustering (0.0-1.0, default: 0.7)
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
    /// Enable automatic role inference from region analysis
    #[serde(default = "default_role_inference")]
    pub role_inference: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: default_confidence_threshold(),
            role_inference: default_role_inference(),
        }
    }
}

fn default_confidence_threshold() -> f64 {
    0.7
}

fn default_role_inference() -> bool {
    true
}

/// Telemetry settings for error collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable local error collection
    #[serde(default)]
    pub collect_errors: bool,
    /// Path to error log file (default: .pxl-errors.jsonl)
    #[serde(default = "default_error_log")]
    pub error_log: PathBuf,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { collect_errors: false, error_log: default_error_log() }
    }
}

fn default_error_log() -> PathBuf {
    PathBuf::from(".pxl-errors.jsonl")
}

/// Validation severity level for config issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ValidationLevel {
    /// Treat as error, fail build
    Error,
    /// Emit warning, continue build
    #[default]
    Warn,
    /// Silently ignore
    Ignore,
}

/// Sprite sheet layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SheetLayout {
    /// Frames arranged left to right
    #[default]
    Horizontal,
    /// Frames arranged top to bottom
    Vertical,
    /// Frames arranged in a grid
    Grid,
}

/// Texture filter mode for exports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FilterMode {
    /// Nearest-neighbor (pixel-perfect)
    #[default]
    Point,
    /// Bilinear interpolation
    Bilinear,
}

/// Antialiasing mode for pixel art scaling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AntialiasMode {
    /// No antialiasing
    #[default]
    None,
    /// Basic edge antialiasing
    Edge,
    /// Semantic-aware antialiasing (uses region roles)
    Semantic,
}

/// Antialiasing configuration for scaled output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntialiasConfig {
    /// Enable antialiasing
    #[serde(default)]
    pub enabled: bool,
    /// Antialiasing mode
    #[serde(default)]
    pub mode: AntialiasMode,
    /// Edge detection threshold (0.0-1.0, default: 0.5)
    #[serde(default = "default_edge_threshold")]
    pub edge_threshold: f64,
}

impl Default for AntialiasConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: AntialiasMode::None,
            edge_threshold: default_edge_threshold(),
        }
    }
}

fn default_edge_threshold() -> f64 {
    0.5
}

/// Project metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name (required)
    pub name: String,
    /// Project version
    #[serde(default = "default_version")]
    pub version: String,
    /// Source directory for .pxl files
    #[serde(default = "default_src")]
    pub src: PathBuf,
    /// Build output directory
    #[serde(default = "default_out")]
    pub out: PathBuf,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_src() -> PathBuf {
    PathBuf::from("src/pxl")
}

fn default_out() -> PathBuf {
    PathBuf::from("build")
}

/// Default settings applied to all outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default scale factor
    #[serde(default = "default_scale")]
    pub scale: u32,
    /// Default padding between sprites
    #[serde(default = "default_padding")]
    pub padding: u32,
    /// Default antialiasing settings
    #[serde(default)]
    pub antialias: AntialiasConfig,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            scale: default_scale(),
            padding: default_padding(),
            antialias: AntialiasConfig::default(),
        }
    }
}

fn default_scale() -> u32 {
    1
}

fn default_padding() -> u32 {
    1
}

/// Atlas configuration for sprite packing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasConfig {
    /// Glob patterns for sprite sources
    pub sources: Vec<String>,
    /// Maximum atlas dimensions [width, height]
    #[serde(default = "default_max_size")]
    pub max_size: [u32; 2],
    /// Padding between sprites (overrides defaults)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    /// Constrain to power-of-two dimensions
    #[serde(default)]
    pub power_of_two: bool,
    /// Preserve nine-slice metadata
    #[serde(default)]
    pub nine_slice: bool,
    /// Antialiasing settings (overrides defaults)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antialias: Option<AntialiasConfig>,
}

fn default_max_size() -> [u32; 2] {
    [1024, 1024]
}

/// Animation output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationsConfig {
    /// Glob patterns for animation files
    #[serde(default = "default_animation_sources")]
    pub sources: Vec<String>,
    /// Generate preview GIFs
    #[serde(default)]
    pub preview: bool,
    /// Scale factor for previews
    #[serde(default = "default_scale")]
    pub preview_scale: u32,
    /// Layout direction for sprite sheets
    #[serde(default)]
    pub sheet_layout: SheetLayout,
}

impl Default for AnimationsConfig {
    fn default() -> Self {
        Self {
            sources: default_animation_sources(),
            preview: false,
            preview_scale: default_scale(),
            sheet_layout: SheetLayout::default(),
        }
    }
}

fn default_animation_sources() -> Vec<String> {
    vec!["animations/**".to_string()]
}

/// Generic JSON export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericExportConfig {
    /// Enable this export
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output format identifier
    #[serde(default = "default_json_format")]
    pub atlas_format: String,
}

fn default_true() -> bool {
    true
}

fn default_json_format() -> String {
    "json".to_string()
}

impl Default for GenericExportConfig {
    fn default() -> Self {
        Self { enabled: true, atlas_format: "json".to_string() }
    }
}

/// Godot export configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GodotExportConfig {
    /// Enable Godot export
    #[serde(default)]
    pub enabled: bool,
    /// Output format identifier
    #[serde(default = "default_godot_format")]
    pub atlas_format: String,
    /// Godot resource path prefix
    #[serde(default = "default_godot_resource_path")]
    pub resource_path: String,
    /// Generate AnimationPlayer resources
    #[serde(default = "default_true")]
    pub animation_player: bool,
    /// Generate SpriteFrames resources
    #[serde(default = "default_true")]
    pub sprite_frames: bool,
}

fn default_godot_format() -> String {
    "godot".to_string()
}

fn default_godot_resource_path() -> String {
    "res://assets/sprites".to_string()
}

/// Unity export configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnityExportConfig {
    /// Enable Unity export
    #[serde(default)]
    pub enabled: bool,
    /// Output format identifier
    #[serde(default = "default_unity_format")]
    pub atlas_format: String,
    /// Pixels per unit setting
    #[serde(default = "default_pixels_per_unit")]
    pub pixels_per_unit: u32,
    /// Texture filter mode
    #[serde(default)]
    pub filter_mode: FilterMode,
    /// Generate texture .meta file with sprite slice data
    #[serde(default = "default_true")]
    pub generate_meta: bool,
    /// Generate .anim files for animations
    #[serde(default = "default_true")]
    pub generate_anim: bool,
    /// Generate JSON metadata (for custom import scripts)
    #[serde(default = "default_true")]
    pub generate_json: bool,
}

fn default_unity_format() -> String {
    "unity".to_string()
}

fn default_pixels_per_unit() -> u32 {
    16
}

/// libGDX export configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibGdxExportConfig {
    /// Enable libGDX export
    #[serde(default)]
    pub enabled: bool,
    /// Output format identifier
    #[serde(default = "default_libgdx_format")]
    pub atlas_format: String,
}

fn default_libgdx_format() -> String {
    "libgdx".to_string()
}

/// Export configurations container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportsConfig {
    /// Generic JSON export
    #[serde(default)]
    pub generic: GenericExportConfig,
    /// Godot export
    #[serde(default)]
    pub godot: GodotExportConfig,
    /// Unity export
    #[serde(default)]
    pub unity: UnityExportConfig,
    /// libGDX export
    #[serde(default)]
    pub libgdx: LibGdxExportConfig,
}

/// Validation settings for the build process
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidateConfig {
    /// Treat warnings as errors
    #[serde(default)]
    pub strict: bool,
    /// Allow region overflow beyond declared sprite size
    #[serde(default)]
    pub allow_overflow: bool,
    /// Allow orphan tokens (defined but unused)
    #[serde(default)]
    pub allow_orphans: bool,
    /// Allow circular dependencies in relationships
    #[serde(default)]
    pub allow_cycles: bool,
    /// How to handle unused palettes
    #[serde(default)]
    pub unused_palettes: ValidationLevel,
    /// How to handle missing references
    #[serde(default = "default_missing_refs")]
    pub missing_refs: ValidationLevel,
}

fn default_missing_refs() -> ValidationLevel {
    ValidationLevel::Error
}

/// Watch mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Debounce delay in milliseconds
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u32,
    /// Clear terminal between rebuilds
    #[serde(default = "default_true")]
    pub clear_screen: bool,
}

fn default_debounce_ms() -> u32 {
    100
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self { debounce_ms: 100, clear_screen: true }
    }
}

/// Complete pxl.toml configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PxlConfig {
    /// Project metadata (required)
    pub project: ProjectConfig,
    /// Format version settings
    #[serde(default)]
    pub format: FormatConfig,
    /// Import settings
    #[serde(default, rename = "import")]
    pub import_config: ImportConfig,
    /// Telemetry settings
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    /// Default settings
    #[serde(default)]
    pub defaults: DefaultsConfig,
    /// Atlas definitions
    #[serde(default)]
    pub atlases: HashMap<String, AtlasConfig>,
    /// Animation settings
    #[serde(default)]
    pub animations: AnimationsConfig,
    /// Export configurations
    #[serde(default, rename = "export")]
    pub exports: ExportsConfig,
    /// Validation settings
    #[serde(default)]
    pub validate: ValidateConfig,
    /// Watch mode settings
    #[serde(default)]
    pub watch: WatchConfig,
}

/// Configuration validation error
#[derive(Debug, Clone)]
pub struct ConfigValidationError {
    /// Path to the invalid field (e.g., "atlases.characters.max_size")
    pub field: String,
    /// Error message
    pub message: String,
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pxl.toml: '{}' {}", self.field, self.message)
    }
}

impl PxlConfig {
    /// Validate the configuration and return any errors
    pub fn validate(&self) -> Vec<ConfigValidationError> {
        let mut errors = Vec::new();

        // Validate project.name is non-empty
        if self.project.name.is_empty() {
            errors.push(ConfigValidationError {
                field: "project.name".to_string(),
                message: "must be a non-empty string".to_string(),
            });
        }

        // Validate defaults
        if self.defaults.scale == 0 {
            errors.push(ConfigValidationError {
                field: "defaults.scale".to_string(),
                message: "must be a positive integer".to_string(),
            });
        }

        // Validate defaults.antialias
        if !(0.0..=1.0).contains(&self.defaults.antialias.edge_threshold) {
            errors.push(ConfigValidationError {
                field: "defaults.antialias.edge_threshold".to_string(),
                message: "must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate atlases
        for (name, atlas) in &self.atlases {
            if atlas.sources.is_empty() {
                errors.push(ConfigValidationError {
                    field: format!("atlases.{}.sources", name),
                    message: "must contain at least one glob pattern".to_string(),
                });
            }

            if atlas.max_size[0] == 0 || atlas.max_size[1] == 0 {
                errors.push(ConfigValidationError {
                    field: format!("atlases.{}.max_size", name),
                    message: "dimensions must be positive".to_string(),
                });
            }

            if let Some(ref aa) = atlas.antialias {
                if !(0.0..=1.0).contains(&aa.edge_threshold) {
                    errors.push(ConfigValidationError {
                        field: format!("atlases.{}.antialias.edge_threshold", name),
                        message: "must be between 0.0 and 1.0".to_string(),
                    });
                }
            }
        }

        // Validate animations
        if self.animations.preview_scale == 0 {
            errors.push(ConfigValidationError {
                field: "animations.preview_scale".to_string(),
                message: "must be a positive integer".to_string(),
            });
        }

        // Validate unity export
        if self.exports.unity.enabled && self.exports.unity.pixels_per_unit == 0 {
            errors.push(ConfigValidationError {
                field: "export.unity.pixels_per_unit".to_string(),
                message: "must be a positive integer".to_string(),
            });
        }

        errors
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Get effective padding for an atlas (atlas-specific or default)
    pub fn effective_padding(&self, atlas: &AtlasConfig) -> u32 {
        atlas.padding.unwrap_or(self.defaults.padding)
    }

    /// Get effective antialias config for an atlas (atlas-specific or default)
    pub fn effective_antialias<'a>(&'a self, atlas: &'a AtlasConfig) -> &'a AntialiasConfig {
        atlas.antialias.as_ref().unwrap_or(&self.defaults.antialias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config_parse() {
        let toml = r#"
[project]
name = "test-project"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("minimal config should parse");
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.project.version, "0.1.0");
        assert_eq!(config.project.src, PathBuf::from("src/pxl"));
        assert_eq!(config.project.out, PathBuf::from("build"));
    }

    #[test]
    fn test_full_config_parse() {
        let toml = r#"
[project]
name = "full-project"
version = "1.0.0"
src = "assets/pxl"
out = "dist"

[defaults]
scale = 2
padding = 4

[atlases.characters]
sources = ["sprites/player/**", "sprites/enemies/**"]
max_size = [2048, 2048]
padding = 2
power_of_two = true

[animations]
sources = ["anims/**"]
preview = true
preview_scale = 4
sheet_layout = "vertical"

[export.godot]
enabled = true
resource_path = "res://sprites"

[export.unity]
enabled = true
pixels_per_unit = 32
filter_mode = "bilinear"

[validate]
strict = true
unused_palettes = "error"
missing_refs = "warn"

[watch]
debounce_ms = 200
clear_screen = false
"#;
        let config: PxlConfig = toml::from_str(toml).expect("full config should parse");

        assert_eq!(config.project.name, "full-project");
        assert_eq!(config.project.version, "1.0.0");
        assert_eq!(config.defaults.scale, 2);
        assert_eq!(config.defaults.padding, 4);

        let chars_atlas = config.atlases.get("characters").expect("characters atlas should exist");
        assert_eq!(chars_atlas.sources.len(), 2);
        assert_eq!(chars_atlas.max_size, [2048, 2048]);
        assert_eq!(chars_atlas.padding, Some(2));
        assert!(chars_atlas.power_of_two);

        assert!(config.animations.preview);
        assert_eq!(config.animations.preview_scale, 4);
        assert_eq!(config.animations.sheet_layout, SheetLayout::Vertical);

        assert!(config.exports.godot.enabled);
        assert_eq!(config.exports.godot.resource_path, "res://sprites");

        assert!(config.exports.unity.enabled);
        assert_eq!(config.exports.unity.pixels_per_unit, 32);
        assert_eq!(config.exports.unity.filter_mode, FilterMode::Bilinear);

        assert!(config.validate.strict);
        assert_eq!(config.validate.unused_palettes, ValidationLevel::Error);
        assert_eq!(config.validate.missing_refs, ValidationLevel::Warn);

        assert_eq!(config.watch.debounce_ms, 200);
        assert!(!config.watch.clear_screen);
    }

    #[test]
    fn test_validation_empty_name() {
        let toml = r#"
[project]
name = ""
"#;
        let config: PxlConfig = toml::from_str(toml).expect("empty name config should parse");
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "project.name"));
    }

    #[test]
    fn test_validation_zero_scale() {
        let toml = r#"
[project]
name = "test"

[defaults]
scale = 0
"#;
        let config: PxlConfig = toml::from_str(toml).expect("zero scale config should parse");
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.field == "defaults.scale"));
    }

    #[test]
    fn test_validation_empty_atlas_sources() {
        let toml = r#"
[project]
name = "test"

[atlases.empty]
sources = []
"#;
        let config: PxlConfig = toml::from_str(toml).expect("empty sources config should parse");
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.field == "atlases.empty.sources"));
    }

    #[test]
    fn test_validation_zero_max_size() {
        let toml = r#"
[project]
name = "test"

[atlases.bad]
sources = ["sprites/**"]
max_size = [0, 1024]
"#;
        let config: PxlConfig = toml::from_str(toml).expect("zero max_size config should parse");
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.field == "atlases.bad.max_size"));
    }

    #[test]
    fn test_effective_padding() {
        let toml = r#"
[project]
name = "test"

[defaults]
padding = 4

[atlases.with_padding]
sources = ["a/**"]
padding = 2

[atlases.without_padding]
sources = ["b/**"]
"#;
        let config: PxlConfig = toml::from_str(toml).expect("padding config should parse");

        let with = config.atlases.get("with_padding").expect("with_padding atlas should exist");
        let without = config.atlases.get("without_padding").expect("without_padding atlas should exist");

        assert_eq!(config.effective_padding(with), 2);
        assert_eq!(config.effective_padding(without), 4);
    }

    #[test]
    fn test_validation_level_serde() {
        let toml = r#"
[project]
name = "test"

[validate]
unused_palettes = "error"
missing_refs = "ignore"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("validation level config should parse");
        assert_eq!(config.validate.unused_palettes, ValidationLevel::Error);
        assert_eq!(config.validate.missing_refs, ValidationLevel::Ignore);
    }

    #[test]
    fn test_sheet_layout_serde() {
        let toml = r#"
[project]
name = "test"

[animations]
sheet_layout = "grid"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("sheet layout config should parse");
        assert_eq!(config.animations.sheet_layout, SheetLayout::Grid);
    }

    #[test]
    fn test_filter_mode_serde() {
        let toml = r#"
[project]
name = "test"

[export.unity]
enabled = true
filter_mode = "bilinear"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("filter mode config should parse");
        assert_eq!(config.exports.unity.filter_mode, FilterMode::Bilinear);
    }

    #[test]
    fn test_format_config_defaults() {
        let toml = r#"
[project]
name = "test"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("format defaults config should parse");
        assert_eq!(config.format.version, 2);
    }

    #[test]
    fn test_format_config_explicit() {
        let toml = r#"
[project]
name = "test"

[format]
version = 3
"#;
        let config: PxlConfig = toml::from_str(toml).expect("explicit format config should parse");
        assert_eq!(config.format.version, 3);
    }

    #[test]
    fn test_import_config_defaults() {
        let toml = r#"
[project]
name = "test"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("import defaults config should parse");
        assert!((config.import_config.confidence_threshold - 0.7).abs() < 0.001);
        assert!(config.import_config.role_inference);
    }

    #[test]
    fn test_import_config_explicit() {
        let toml = r#"
[project]
name = "test"

[import]
confidence_threshold = 0.9
role_inference = false
"#;
        let config: PxlConfig = toml::from_str(toml).expect("explicit import config should parse");
        assert!((config.import_config.confidence_threshold - 0.9).abs() < 0.001);
        assert!(!config.import_config.role_inference);
    }

    #[test]
    fn test_telemetry_config_defaults() {
        let toml = r#"
[project]
name = "test"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("telemetry defaults config should parse");
        assert!(!config.telemetry.collect_errors);
        assert_eq!(config.telemetry.error_log, PathBuf::from(".pxl-errors.jsonl"));
    }

    #[test]
    fn test_telemetry_config_explicit() {
        let toml = r#"
[project]
name = "test"

[telemetry]
collect_errors = true
error_log = "errors/pxl.jsonl"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("explicit telemetry config should parse");
        assert!(config.telemetry.collect_errors);
        assert_eq!(config.telemetry.error_log, PathBuf::from("errors/pxl.jsonl"));
    }

    #[test]
    fn test_validate_config_allow_flags() {
        let toml = r#"
[project]
name = "test"

[validate]
strict = true
allow_overflow = true
allow_orphans = true
allow_cycles = true
"#;
        let config: PxlConfig = toml::from_str(toml).expect("validate flags config should parse");
        assert!(config.validate.strict);
        assert!(config.validate.allow_overflow);
        assert!(config.validate.allow_orphans);
        assert!(config.validate.allow_cycles);
    }

    #[test]
    fn test_validate_config_allow_flags_default_false() {
        let toml = r#"
[project]
name = "test"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("validate defaults config should parse");
        assert!(!config.validate.allow_overflow);
        assert!(!config.validate.allow_orphans);
        assert!(!config.validate.allow_cycles);
    }

    #[test]
    fn test_antialias_config_defaults() {
        let toml = r#"
[project]
name = "test"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("antialias defaults config should parse");
        assert!(!config.defaults.antialias.enabled);
        assert_eq!(config.defaults.antialias.mode, AntialiasMode::None);
        assert!((config.defaults.antialias.edge_threshold - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_antialias_config_explicit() {
        let toml = r#"
[project]
name = "test"

[defaults.antialias]
enabled = true
mode = "semantic"
edge_threshold = 0.75
"#;
        let config: PxlConfig = toml::from_str(toml).expect("explicit antialias config should parse");
        assert!(config.defaults.antialias.enabled);
        assert_eq!(config.defaults.antialias.mode, AntialiasMode::Semantic);
        assert!((config.defaults.antialias.edge_threshold - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_antialias_mode_serde() {
        let toml = r#"
[project]
name = "test"

[defaults.antialias]
mode = "edge"
"#;
        let config: PxlConfig = toml::from_str(toml).expect("antialias mode config should parse");
        assert_eq!(config.defaults.antialias.mode, AntialiasMode::Edge);
    }

    #[test]
    fn test_atlas_antialias_config() {
        let toml = r#"
[project]
name = "test"

[atlases.sprites]
sources = ["sprites/**"]

[atlases.sprites.antialias]
enabled = true
mode = "semantic"
edge_threshold = 0.6
"#;
        let config: PxlConfig = toml::from_str(toml).expect("atlas antialias config should parse");
        let atlas = config.atlases.get("sprites").expect("sprites atlas should exist");
        let aa = atlas.antialias.as_ref().expect("atlas should have antialias config");
        assert!(aa.enabled);
        assert_eq!(aa.mode, AntialiasMode::Semantic);
        assert!((aa.edge_threshold - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_effective_antialias() {
        let toml = r#"
[project]
name = "test"

[defaults.antialias]
enabled = true
mode = "edge"
edge_threshold = 0.4

[atlases.with_aa]
sources = ["a/**"]

[atlases.with_aa.antialias]
enabled = true
mode = "semantic"
edge_threshold = 0.8

[atlases.without_aa]
sources = ["b/**"]
"#;
        let config: PxlConfig = toml::from_str(toml).expect("effective antialias config should parse");

        let with = config.atlases.get("with_aa").expect("with_aa atlas should exist");
        let without = config.atlases.get("without_aa").expect("without_aa atlas should exist");

        let aa_with = config.effective_antialias(with);
        assert_eq!(aa_with.mode, AntialiasMode::Semantic);
        assert!((aa_with.edge_threshold - 0.8).abs() < 0.001);

        let aa_without = config.effective_antialias(without);
        assert_eq!(aa_without.mode, AntialiasMode::Edge);
        assert!((aa_without.edge_threshold - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_validation_invalid_edge_threshold_defaults() {
        let toml = r#"
[project]
name = "test"

[defaults.antialias]
edge_threshold = 1.5
"#;
        let config: PxlConfig = toml::from_str(toml).expect("invalid edge threshold config should parse");
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.field == "defaults.antialias.edge_threshold"));
    }

    #[test]
    fn test_validation_invalid_edge_threshold_atlas() {
        let toml = r#"
[project]
name = "test"

[atlases.bad]
sources = ["sprites/**"]

[atlases.bad.antialias]
edge_threshold = -0.1
"#;
        let config: PxlConfig = toml::from_str(toml).expect("invalid atlas edge threshold config should parse");
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.field == "atlases.bad.antialias.edge_threshold"));
    }
}
