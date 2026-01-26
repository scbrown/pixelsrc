//! Build Variants Demo Tests
//!
//! Demonstrates debug/release/custom build variants through CLI overrides.
//! Build variants allow different configurations for development vs production.

use pixelsrc::config::{merge_cli_overrides, CliOverrides, PxlConfig};
use std::path::PathBuf;

/// Get the fixture TOML content for variants configuration.
fn variants_toml() -> &'static str {
    include_str!("../../../examples/demos/build/variants.toml")
}

/// @demo build/variants#base_config
/// @title Base Configuration (Debug)
/// @description Default configuration optimized for fast iteration during development.
#[test]
fn test_build_variants_base_config() {
    let toml_content = variants_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Verify project basics
    assert_eq!(config.project.name, "variants-demo");
    assert_eq!(config.project.out, PathBuf::from("build"));

    // Debug variant: 1x scale for fast iteration
    assert_eq!(config.defaults.scale, 1, "Debug should use 1x scale");
    assert_eq!(config.defaults.padding, 1);

    // Debug validation: relaxed for faster feedback
    assert!(!config.validate.strict, "Debug should not use strict validation");

    // Preview enabled for development
    assert!(config.animations.preview);
    assert_eq!(config.animations.preview_scale, 1);

    // Watch enabled for development
    assert!(config.watch.clear_screen);
}

/// @demo build/variants#release_override
/// @title Release Build via CLI Override
/// @description Use CLI overrides to create a release build configuration.
#[test]
fn test_build_variants_release_override() {
    let toml_content = variants_toml();
    let mut config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Apply release overrides via CLI
    let release_overrides = CliOverrides {
        out: Some(PathBuf::from("dist")),
        scale: Some(4),     // 4x scale for release
        strict: Some(true), // Strict validation for release
        ..Default::default()
    };

    merge_cli_overrides(&mut config, &release_overrides);

    // Verify release overrides applied
    assert_eq!(config.project.out, PathBuf::from("dist"), "Release output should be dist/");
    assert_eq!(config.defaults.scale, 4, "Release should use 4x scale");
    assert!(config.validate.strict, "Release should use strict validation");
}

/// @demo build/variants#cli_overrides
/// @title CLI Override Precedence
/// @description CLI arguments take precedence over config file values.
#[test]
fn test_build_variants_cli_precedence() {
    let toml_content = variants_toml();
    let mut config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Verify original values
    assert_eq!(config.defaults.scale, 1);
    assert_eq!(config.defaults.padding, 1);

    // Apply partial overrides
    let overrides = CliOverrides { scale: Some(2), padding: Some(4), ..Default::default() };

    merge_cli_overrides(&mut config, &overrides);

    // Verify only overridden values changed
    assert_eq!(config.defaults.scale, 2, "Scale should be overridden to 2");
    assert_eq!(config.defaults.padding, 4, "Padding should be overridden to 4");

    // Non-overridden values should remain unchanged
    assert_eq!(config.project.name, "variants-demo");
    assert_eq!(config.project.out, PathBuf::from("build"));
}

/// @demo build/variants#selective_exports
/// @title Selective Export Formats
/// @description Enable different export formats for different build variants.
#[test]
fn test_build_variants_selective_exports() {
    let toml_content = variants_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Base config only has generic enabled
    assert!(config.exports.generic.enabled, "Generic should be enabled");
    assert!(!config.exports.godot.enabled, "Godot disabled by default");
    assert!(!config.exports.unity.enabled, "Unity disabled by default");

    // Release would enable all exports via separate config or CLI
    // (In practice, this would be a separate pxl.release.toml or CLI flags)
}

/// @demo build/variants#multiple_overrides
/// @title Multiple CLI Overrides
/// @description Apply multiple overrides for complex build configurations.
#[test]
fn test_build_variants_multiple_overrides() {
    let toml_content = variants_toml();
    let mut config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Apply comprehensive release overrides
    let overrides = CliOverrides {
        out: Some(PathBuf::from("release/sprites")),
        src: Some(PathBuf::from("assets")),
        scale: Some(4),
        padding: Some(2),
        strict: Some(true),
        allow_overflow: None,
        allow_orphans: None,
        allow_cycles: None,
        collect_errors: None,
        atlas: Some("main".to_string()),
        export: Some("godot".to_string()),
        jobs: Some(4),
    };

    merge_cli_overrides(&mut config, &overrides);

    // Verify all applicable overrides
    assert_eq!(config.project.out, PathBuf::from("release/sprites"));
    assert_eq!(config.project.src, PathBuf::from("assets"));
    assert_eq!(config.defaults.scale, 4);
    assert_eq!(config.defaults.padding, 2);
    assert!(config.validate.strict);

    // Note: atlas, export, and jobs are stored in CliOverrides
    // but don't modify PxlConfig directly (they're build-time filters)
}

/// @demo build/variants#default_overrides
/// @title Default CLI Overrides
/// @description Verify CliOverrides defaults to None for all fields.
#[test]
fn test_build_variants_default_overrides() {
    let overrides = CliOverrides::default();

    assert!(overrides.out.is_none());
    assert!(overrides.src.is_none());
    assert!(overrides.scale.is_none());
    assert!(overrides.padding.is_none());
    assert!(overrides.atlas.is_none());
    assert!(overrides.export.is_none());
    assert!(overrides.strict.is_none());
    assert!(overrides.jobs.is_none());
}

/// @demo build/variants#partial_overrides
/// @title Partial Override Application
/// @description Only specified overrides are applied, others remain unchanged.
#[test]
fn test_build_variants_partial_overrides() {
    let toml_content = variants_toml();
    let mut config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Save original values
    let original_name = config.project.name.clone();
    let original_version = config.project.version.clone();
    let original_src = config.project.src.clone();

    // Apply only scale override
    let overrides = CliOverrides { scale: Some(8), ..Default::default() };

    merge_cli_overrides(&mut config, &overrides);

    // Verify scale changed
    assert_eq!(config.defaults.scale, 8);

    // Verify other values unchanged
    assert_eq!(config.project.name, original_name);
    assert_eq!(config.project.version, original_version);
    assert_eq!(config.project.src, original_src);
}

/// @demo build/variants#validation
/// @title Build Variant Validation
/// @description Verify configurations remain valid after applying CLI overrides.
#[test]
fn test_build_variants_validation() {
    let toml_content = variants_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse variants config");

    // Base config should be valid
    let errors = config.validate();
    assert!(errors.is_empty(), "Variants config should have no validation errors");

    // Config remains valid after overrides
    let mut modified = config;
    merge_cli_overrides(
        &mut modified,
        &CliOverrides { scale: Some(4), strict: Some(true), ..Default::default() },
    );

    let errors = modified.validate();
    assert!(errors.is_empty(), "Config should remain valid after overrides");
}
