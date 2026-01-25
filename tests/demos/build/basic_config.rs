//! Basic Configuration Demo Tests
//!
//! Demonstrates minimal pxl.toml project configuration.

use pixelsrc::config::{load_config, PxlConfig};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Get the fixture TOML content for basic configuration.
fn basic_toml() -> &'static str {
    include_str!("../../../examples/demos/build/basic.toml")
}

/// Get the fixture TOML content for full configuration.
fn full_toml() -> &'static str {
    include_str!("../../../examples/demos/build/full.toml")
}

/// @demo build/config#minimal
/// @title Minimal pxl.toml Configuration
/// @description A minimal valid pxl.toml with only the required project name field.
#[test]
fn test_basic_config_minimal() {
    let toml_content = basic_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse basic config");

    // Verify required field
    assert_eq!(config.project.name, "basic-demo", "Project name should match");

    // Verify defaults are applied
    assert_eq!(config.project.version, "0.1.0", "Default version should be 0.1.0");
    assert_eq!(config.project.src, PathBuf::from("src/pxl"), "Default src should be src/pxl");
    assert_eq!(config.project.out, PathBuf::from("build"), "Default out should be build");

    // Verify defaults section gets default values
    assert_eq!(config.defaults.scale, 1, "Default scale should be 1");
    assert_eq!(config.defaults.padding, 1, "Default padding should be 1");

    // Verify config is valid
    let errors = config.validate();
    assert!(errors.is_empty(), "Basic config should have no validation errors");
}

/// @demo build/config#full
/// @title Full pxl.toml Configuration
/// @description Comprehensive configuration showing all available options.
#[test]
fn test_basic_config_full() {
    let toml_content = full_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse full config");

    // Verify project section
    assert_eq!(config.project.name, "full-demo");
    assert_eq!(config.project.version, "1.0.0");
    assert_eq!(config.project.src, PathBuf::from("assets/pxl"));
    assert_eq!(config.project.out, PathBuf::from("dist"));

    // Verify defaults section
    assert_eq!(config.defaults.scale, 2);
    assert_eq!(config.defaults.padding, 4);

    // Verify atlases section
    assert!(config.atlases.contains_key("characters"), "Should have characters atlas");
    assert!(config.atlases.contains_key("ui"), "Should have ui atlas");

    let chars = config.atlases.get("characters").unwrap();
    assert_eq!(chars.sources.len(), 2);
    assert!(chars.sources.contains(&"sprites/player/**".to_string()));
    assert!(chars.sources.contains(&"sprites/enemies/**".to_string()));
    assert_eq!(chars.max_size, [2048, 2048]);
    assert_eq!(chars.padding, Some(2));
    assert!(chars.power_of_two);

    // Verify animations section
    assert!(config.animations.preview);
    assert_eq!(config.animations.preview_scale, 4);

    // Verify exports
    assert!(config.exports.godot.enabled);
    assert_eq!(config.exports.godot.resource_path, "res://sprites");
    assert!(config.exports.unity.enabled);
    assert_eq!(config.exports.unity.pixels_per_unit, 32);
    assert!(!config.exports.libgdx.enabled);

    // Verify validation settings
    assert!(config.validate.strict);

    // Verify watch settings
    assert_eq!(config.watch.debounce_ms, 200);
    assert!(!config.watch.clear_screen);

    // Verify config is valid
    let errors = config.validate();
    assert!(errors.is_empty(), "Full config should have no validation errors");
}

/// @demo build/config#load_file
/// @title Load Configuration from File
/// @description Demonstrates loading pxl.toml from filesystem using load_config().
#[test]
fn test_basic_config_load_file() {
    let temp = TempDir::new().expect("Should create temp dir");
    let config_path = temp.path().join("pxl.toml");

    // Write the basic config fixture to the temp directory
    let mut file = fs::File::create(&config_path).expect("Should create file");
    file.write_all(basic_toml().as_bytes()).expect("Should write file");

    // Load using the load_config function
    let config = load_config(Some(&config_path)).expect("Should load config from file");

    assert_eq!(config.project.name, "basic-demo");
    assert!(config.is_valid(), "Loaded config should be valid");
}

/// @demo build/config#validation
/// @title Configuration Validation
/// @description Verifies that invalid configurations produce appropriate errors.
#[test]
fn test_basic_config_validation_errors() {
    // Empty project name
    let invalid_name = r#"
[project]
name = ""
"#;
    let config: PxlConfig = toml::from_str(invalid_name).expect("Should parse");
    let errors = config.validate();
    assert!(!errors.is_empty(), "Empty name should produce error");
    assert!(errors.iter().any(|e| e.field == "project.name"), "Should have project.name error");

    // Zero scale
    let invalid_scale = r#"
[project]
name = "test"

[defaults]
scale = 0
"#;
    let config: PxlConfig = toml::from_str(invalid_scale).expect("Should parse");
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.field == "defaults.scale"), "Should have defaults.scale error");

    // Empty atlas sources
    let invalid_atlas = r#"
[project]
name = "test"

[atlases.empty]
sources = []
"#;
    let config: PxlConfig = toml::from_str(invalid_atlas).expect("Should parse");
    let errors = config.validate();
    assert!(
        errors.iter().any(|e| e.field.contains("atlases.empty.sources")),
        "Should have atlas sources error"
    );
}

/// @demo build/config#defaults
/// @title Default Values Applied
/// @description Verifies all default values are correctly applied when not specified.
#[test]
fn test_basic_config_defaults_applied() {
    let minimal = r#"
[project]
name = "minimal-test"
"#;
    let config: PxlConfig = toml::from_str(minimal).expect("Should parse");

    // Project defaults
    assert_eq!(config.project.version, "0.1.0");
    assert_eq!(config.project.src, PathBuf::from("src/pxl"));
    assert_eq!(config.project.out, PathBuf::from("build"));

    // Defaults section
    assert_eq!(config.defaults.scale, 1);
    assert_eq!(config.defaults.padding, 1);

    // No atlases by default
    assert!(config.atlases.is_empty());

    // Animations defaults
    assert!(!config.animations.preview);
    assert_eq!(config.animations.preview_scale, 1);

    // Export defaults
    assert!(config.exports.generic.enabled);
    assert!(!config.exports.godot.enabled);
    assert!(!config.exports.unity.enabled);
    assert!(!config.exports.libgdx.enabled);

    // Validation defaults
    assert!(!config.validate.strict);

    // Watch defaults
    assert_eq!(config.watch.debounce_ms, 100);
    assert!(config.watch.clear_screen);
}
