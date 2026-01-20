//! Watch Mode Demo Tests
//!
//! Demonstrates configuration for file watching and automatic rebuilds.
//! Watch mode allows real-time updates during sprite development.

use pixelsrc::config::{PxlConfig, WatchConfig};

/// Get the fixture TOML content for watch mode configuration.
fn watch_mode_toml() -> &'static str {
    include_str!("../../../examples/demos/build/watch_mode.toml")
}

/// @demo build/watch#configuration
/// @title Watch Mode Configuration
/// @description Configure file watching with debounce and screen clearing options.
#[test]
fn test_watch_mode_configuration() {
    let toml_content = watch_mode_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse watch mode config");

    // Verify project basics
    assert_eq!(config.project.name, "watch-demo");

    // Verify watch settings
    assert_eq!(
        config.watch.debounce_ms, 50,
        "Debounce should be 50ms for responsive feedback"
    );
    assert!(
        config.watch.clear_screen,
        "Should clear screen between rebuilds"
    );
}

/// @demo build/watch#debounce
/// @title Debounce Timing
/// @description Debounce prevents excessive rebuilds during rapid file saves.
#[test]
fn test_watch_mode_debounce() {
    // Test default debounce
    let default = WatchConfig::default();
    assert_eq!(default.debounce_ms, 100, "Default debounce should be 100ms");

    // Test custom debounce from config
    let toml_content = watch_mode_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse config");
    assert_eq!(
        config.watch.debounce_ms, 50,
        "Custom debounce should be 50ms"
    );

    // Verify debounce is reasonable (not too fast, not too slow)
    assert!(
        config.watch.debounce_ms >= 10,
        "Debounce should be at least 10ms to batch rapid changes"
    );
    assert!(
        config.watch.debounce_ms <= 500,
        "Debounce should be at most 500ms for responsiveness"
    );
}

/// @demo build/watch#clear_screen
/// @title Screen Clearing Options
/// @description Clear terminal between rebuilds for clean output.
#[test]
fn test_watch_mode_clear_screen() {
    // Test default clear screen
    let default = WatchConfig::default();
    assert!(default.clear_screen, "Default should clear screen");

    // Test custom clear screen from config
    let toml_content = watch_mode_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse config");
    assert!(
        config.watch.clear_screen,
        "Watch mode config should clear screen"
    );

    // Test disabled clear screen
    let no_clear_toml = r#"
[project]
name = "test"

[watch]
clear_screen = false
"#;
    let no_clear: PxlConfig = toml::from_str(no_clear_toml).expect("Should parse");
    assert!(
        !no_clear.watch.clear_screen,
        "Should respect clear_screen = false"
    );
}

/// @demo build/watch#relaxed_validation
/// @title Relaxed Validation for Development
/// @description Watch mode uses relaxed validation for faster iteration.
#[test]
fn test_watch_mode_relaxed_validation() {
    let toml_content = watch_mode_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse config");

    // Verify relaxed validation
    assert!(
        !config.validate.strict,
        "Watch mode should not use strict validation"
    );

    // Unused palettes should be ignored during development
    assert_eq!(
        config.validate.unused_palettes,
        pixelsrc::config::ValidationLevel::Ignore,
        "Unused palettes should be ignored in watch mode"
    );

    // Missing refs should warn, not error
    assert_eq!(
        config.validate.missing_refs,
        pixelsrc::config::ValidationLevel::Warn,
        "Missing refs should warn in watch mode"
    );
}

/// @demo build/watch#preview_generation
/// @title Preview Generation During Watch
/// @description Enable preview GIFs for animations during watch mode.
#[test]
fn test_watch_mode_preview_generation() {
    let toml_content = watch_mode_toml();
    let config: PxlConfig = toml::from_str(toml_content).expect("Should parse config");

    // Verify preview is enabled for quick visual feedback
    assert!(
        config.animations.preview,
        "Watch mode should enable animation previews"
    );

    // Preview scale should be reasonable for quick rendering
    assert_eq!(
        config.animations.preview_scale, 2,
        "Preview scale should be 2x for watch mode"
    );

    // Horizontal layout is faster to render
    assert_eq!(
        config.animations.sheet_layout,
        pixelsrc::config::SheetLayout::Horizontal,
        "Should use horizontal layout for faster spritesheet generation"
    );
}

/// @demo build/watch#default_watch_config
/// @title Default Watch Configuration
/// @description Verify default WatchConfig values are sensible.
#[test]
fn test_watch_mode_defaults() {
    let default = WatchConfig::default();

    // Check all defaults
    assert_eq!(
        default.debounce_ms, 100,
        "Default debounce should be 100ms"
    );
    assert!(
        default.clear_screen,
        "Default should clear screen"
    );
}

/// @demo build/watch#serde_roundtrip
/// @title Watch Config Serialization
/// @description Verify watch configuration serializes and deserializes correctly.
#[test]
fn test_watch_mode_serde_roundtrip() {
    let original = WatchConfig {
        debounce_ms: 75,
        clear_screen: false,
    };

    // Serialize to TOML
    let toml_str = toml::to_string(&original).expect("Should serialize");

    // Verify expected fields are present
    assert!(toml_str.contains("debounce_ms = 75"));
    assert!(toml_str.contains("clear_screen = false"));

    // Deserialize back
    let restored: WatchConfig = toml::from_str(&toml_str).expect("Should deserialize");
    assert_eq!(restored.debounce_ms, 75);
    assert!(!restored.clear_screen);
}

/// Test watch mode with different debounce values
#[test]
fn test_watch_mode_debounce_variations() {
    // Fast debounce for responsive feedback
    let fast_toml = r#"
[project]
name = "fast"

[watch]
debounce_ms = 25
"#;
    let fast: PxlConfig = toml::from_str(fast_toml).expect("Should parse");
    assert_eq!(fast.watch.debounce_ms, 25);

    // Slow debounce for low-resource systems
    let slow_toml = r#"
[project]
name = "slow"

[watch]
debounce_ms = 500
"#;
    let slow: PxlConfig = toml::from_str(slow_toml).expect("Should parse");
    assert_eq!(slow.watch.debounce_ms, 500);
}
