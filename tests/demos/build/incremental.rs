//! Incremental Build Demo Tests
//!
//! Demonstrates configuration optimized for incremental rebuilds.
//! Tests verify atlas partitioning strategies that minimize rebuild scope.

use pixelsrc::build::BuildContext;
use pixelsrc::config::PxlConfig;
use std::path::PathBuf;

/// Get the fixture TOML content for incremental build configuration.
fn incremental_toml() -> &'static str {
    include_str!("../../../examples/demos/build/incremental.toml")
}

/// @demo build/incremental#partitioning
/// @title Atlas Partitioning for Incremental Builds
/// @description Split atlases by category to minimize rebuild scope when sources change.
#[test]
fn test_incremental_partitioning() {
    let toml_content = incremental_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse incremental config");

    // Verify project basics
    assert_eq!(config.project.name, "incremental-demo");

    // Verify we have 4 separate atlases for different sprite categories
    assert_eq!(config.atlases.len(), 4, "Should have 4 partitioned atlases");
    assert!(config.atlases.contains_key("player"), "Should have player atlas");
    assert!(config.atlases.contains_key("enemies"), "Should have enemies atlas");
    assert!(config.atlases.contains_key("environment"), "Should have environment atlas");
    assert!(config.atlases.contains_key("items"), "Should have items atlas");

    // Verify each atlas has its own isolated source
    let player = config.atlases.get("player").unwrap();
    assert_eq!(player.sources.len(), 1);
    assert!(player.sources[0].contains("player"));

    let enemies = config.atlases.get("enemies").unwrap();
    assert_eq!(enemies.sources.len(), 1);
    assert!(enemies.sources[0].contains("enemies"));

    let env = config.atlases.get("environment").unwrap();
    assert_eq!(env.sources.len(), 2, "Environment can have multiple sources");

    let items = config.atlases.get("items").unwrap();
    assert_eq!(items.sources.len(), 2);
}

/// @demo build/incremental#scope_isolation
/// @title Isolated Rebuild Scope
/// @description Changes to player sprites only trigger player atlas rebuild.
#[test]
fn test_incremental_scope_isolation() {
    let toml_content = incremental_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse incremental config");

    // Verify sources don't overlap between atlases
    // This ensures changing one category only rebuilds its atlas

    let player_sources: Vec<&str> =
        config.atlases.get("player").unwrap().sources.iter().map(|s| s.as_str()).collect();

    let enemies_sources: Vec<&str> =
        config.atlases.get("enemies").unwrap().sources.iter().map(|s| s.as_str()).collect();

    // No source from player should match enemies pattern
    for player_src in &player_sources {
        for enemy_src in &enemies_sources {
            assert_ne!(player_src, enemy_src, "Player and enemies sources should not overlap");
        }
    }

    // Verify validation is not strict (faster iteration)
    assert!(
        !config.validate.strict,
        "Incremental config should use non-strict validation for faster iteration"
    );
}

/// @demo build/incremental#build_context
/// @title Build Context Setup
/// @description Create a BuildContext from incremental configuration.
#[test]
fn test_incremental_build_context() {
    let toml_content = incremental_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse incremental config");

    // Create build context
    let project_root = PathBuf::from("/test/project");
    let ctx = BuildContext::new(config, project_root.clone());

    // Verify context properties
    assert_eq!(ctx.project_root(), &project_root);
    assert_eq!(ctx.src_dir(), project_root.join("src/pxl"));
    assert_eq!(ctx.out_dir(), project_root.join("build"));

    // Verify strict mode is off
    assert!(!ctx.is_strict(), "Should not be strict for incremental builds");

    // Verify defaults
    assert_eq!(ctx.default_scale(), 1);
    assert_eq!(ctx.default_padding(), 1);
}

/// @demo build/incremental#target_filter
/// @title Build Target Filtering
/// @description Build context supports filtering to rebuild specific atlases.
#[test]
fn test_incremental_target_filter() {
    let toml_content = incremental_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse incremental config");

    let project_root = PathBuf::from("/test/project");
    let ctx = BuildContext::new(config, project_root).with_filter(vec!["player".to_string()]);

    // Verify filter is set
    let filter = ctx.target_filter();
    assert!(filter.is_some(), "Filter should be set");
    assert_eq!(filter.unwrap(), &["player"]);
}

/// @demo build/incremental#validation_relaxed
/// @title Relaxed Validation for Development
/// @description Non-strict validation allows faster iteration during development.
#[test]
fn test_incremental_relaxed_validation() {
    let toml_content = incremental_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse incremental config");

    // Verify relaxed validation settings
    assert!(!config.validate.strict, "Should not be strict");

    // Unused palettes should warn, not error
    assert_eq!(
        config.validate.unused_palettes,
        pixelsrc::config::ValidationLevel::Warn,
        "Unused palettes should warn"
    );

    // Missing refs should still error (important for correctness)
    assert_eq!(
        config.validate.missing_refs,
        pixelsrc::config::ValidationLevel::Error,
        "Missing refs should error"
    );
}

/// @demo build/incremental#atlas_sizing
/// @title Atlas Sizing by Category
/// @description Atlases sized appropriately for their expected content.
#[test]
fn test_incremental_atlas_sizing() {
    let toml_content = incremental_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse incremental config");

    // Player atlas - smallest (single character)
    let player = config.atlases.get("player").unwrap();
    assert_eq!(player.max_size, [512, 512], "Player atlas should be 512x512");

    // Enemies atlas - medium (multiple enemies)
    let enemies = config.atlases.get("enemies").unwrap();
    assert_eq!(enemies.max_size, [1024, 1024], "Enemies atlas should be 1024x1024");

    // Environment atlas - largest (tiles, backgrounds)
    let env = config.atlases.get("environment").unwrap();
    assert_eq!(env.max_size, [2048, 2048], "Environment atlas should be 2048x2048");

    // Items atlas - small
    let items = config.atlases.get("items").unwrap();
    assert_eq!(items.max_size, [512, 512], "Items atlas should be 512x512");

    // All should use power-of-two for GPU compatibility
    for (name, atlas) in &config.atlases {
        assert!(atlas.power_of_two, "Atlas '{}' should use power-of-two dimensions", name);
    }
}
