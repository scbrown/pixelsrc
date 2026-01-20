//! Multiple Output Targets Demo Tests
//!
//! Demonstrates configuration with multiple atlases and export formats.

use pixelsrc::config::PxlConfig;
use std::path::PathBuf;

/// Get the fixture TOML content for multi-target configuration.
fn multi_target_toml() -> &'static str {
    include_str!("../../../examples/demos/build/multi_target.toml")
}

/// @demo build/targets#multiple_atlases
/// @title Multiple Atlas Definitions
/// @description Configuration with separate atlases for different sprite categories.
#[test]
fn test_multi_target_atlases() {
    let toml_content = multi_target_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse multi-target config");

    // Verify project basics
    assert_eq!(config.project.name, "multi-target-demo");
    assert_eq!(config.project.src, PathBuf::from("sprites"));
    assert_eq!(config.project.out, PathBuf::from("build"));

    // Verify we have 3 atlases defined
    assert_eq!(config.atlases.len(), 3, "Should have 3 atlases defined");
    assert!(config.atlases.contains_key("game"), "Should have game atlas");
    assert!(config.atlases.contains_key("ui"), "Should have ui atlas");
    assert!(config.atlases.contains_key("effects"), "Should have effects atlas");

    // Verify game atlas settings
    let game = config.atlases.get("game").unwrap();
    assert_eq!(game.sources.len(), 3);
    assert!(game.sources.contains(&"characters/**".to_string()));
    assert!(game.sources.contains(&"enemies/**".to_string()));
    assert!(game.sources.contains(&"items/**".to_string()));
    assert_eq!(game.max_size, [2048, 2048]);
    assert_eq!(game.padding, Some(2));
    assert!(game.power_of_two);

    // Verify UI atlas settings
    let ui = config.atlases.get("ui").unwrap();
    assert_eq!(ui.sources.len(), 2);
    assert!(ui.sources.contains(&"ui/**".to_string()));
    assert!(ui.sources.contains(&"hud/**".to_string()));
    assert_eq!(ui.max_size, [1024, 1024]);
    assert!(ui.power_of_two);

    // Verify effects atlas settings (smaller, no POT)
    let effects = config.atlases.get("effects").unwrap();
    assert_eq!(effects.sources.len(), 2);
    assert_eq!(effects.max_size, [512, 512]);
    assert_eq!(effects.padding, Some(0));
    assert!(!effects.power_of_two, "Effects atlas should not require power-of-two");
}

/// @demo build/targets#multiple_exports
/// @title Multiple Export Formats
/// @description Configuration with all export formats enabled.
#[test]
fn test_multi_target_exports() {
    let toml_content = multi_target_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse multi-target config");

    // Verify all exports are enabled
    assert!(config.exports.generic.enabled, "Generic export should be enabled");
    assert!(config.exports.godot.enabled, "Godot export should be enabled");
    assert!(config.exports.unity.enabled, "Unity export should be enabled");
    assert!(config.exports.libgdx.enabled, "libGDX export should be enabled");

    // Verify Godot settings
    assert_eq!(config.exports.godot.resource_path, "res://assets/sprites");
    assert!(config.exports.godot.animation_player);
    assert!(config.exports.godot.sprite_frames);

    // Verify Unity settings
    assert_eq!(config.exports.unity.pixels_per_unit, 16);

    // Verify config is valid
    assert!(config.is_valid(), "Multi-target config should be valid");
}

/// @demo build/targets#atlas_isolation
/// @title Atlas Source Isolation
/// @description Each atlas has separate sources to prevent overlapping sprite assignments.
#[test]
fn test_multi_target_atlas_isolation() {
    let toml_content = multi_target_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse multi-target config");

    // Collect all source patterns across atlases
    let mut all_sources: Vec<&String> = Vec::new();
    for atlas in config.atlases.values() {
        all_sources.extend(&atlas.sources);
    }

    // Verify no duplicate patterns (sources should be mutually exclusive)
    let unique_count = {
        let mut unique: Vec<&String> = all_sources.clone();
        unique.sort();
        unique.dedup();
        unique.len()
    };

    assert_eq!(
        unique_count,
        all_sources.len(),
        "All atlas sources should be unique (no overlapping patterns)"
    );

    // Verify each atlas has at least one source
    for (name, atlas) in &config.atlases {
        assert!(
            !atlas.sources.is_empty(),
            "Atlas '{}' should have at least one source pattern",
            name
        );
    }
}

/// @demo build/targets#size_hierarchy
/// @title Atlas Size Hierarchy
/// @description Atlases sized appropriately for their content categories.
#[test]
fn test_multi_target_size_hierarchy() {
    let toml_content = multi_target_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse multi-target config");

    let game = config.atlases.get("game").unwrap();
    let ui = config.atlases.get("ui").unwrap();
    let effects = config.atlases.get("effects").unwrap();

    // Game atlas should be largest (most sprites)
    assert!(
        game.max_size[0] >= ui.max_size[0] && game.max_size[0] >= effects.max_size[0],
        "Game atlas should be largest"
    );

    // Effects atlas should be smallest (fewer sprites)
    assert!(
        effects.max_size[0] <= ui.max_size[0],
        "Effects atlas should be smaller than UI"
    );

    // Verify all sizes are valid
    for (name, atlas) in &config.atlases {
        assert!(
            atlas.max_size[0] > 0 && atlas.max_size[1] > 0,
            "Atlas '{}' should have positive dimensions",
            name
        );
    }
}
