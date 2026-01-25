//! New Command Demo Tests
//!
//! Demonstrates the `pxl new` command functionality for scaffolding
//! new sprites, animations, and palettes within a pixelsrc project.

use pixelsrc::scaffold::{new_animation, new_palette, new_sprite, ScaffoldError};
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a minimal project structure for testing
fn setup_test_project(temp: &TempDir) -> PathBuf {
    let project_path = temp.path().to_path_buf();

    // Create pxl.toml
    let config = r#"[project]
name = "test"
version = "0.1.0"
"#;
    fs::write(project_path.join("pxl.toml"), config).unwrap();

    // Create directory structure
    fs::create_dir_all(project_path.join("src/pxl/sprites")).unwrap();
    fs::create_dir_all(project_path.join("src/pxl/animations")).unwrap();
    fs::create_dir_all(project_path.join("src/pxl/palettes")).unwrap();

    project_path
}

/// Run a test function in a project directory context
fn in_project<F, R>(temp: &TempDir, f: F) -> R
where
    F: FnOnce() -> R,
{
    let project_path = setup_test_project(temp);
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project_path).unwrap();

    let result = f();

    std::env::set_current_dir(original_dir).unwrap();
    result
}

// ============================================================================
// New Sprite Tests
// ============================================================================

/// @demo cli/new#sprite_basic
/// @title Create New Sprite
/// @description Create a new sprite with `pxl new sprite hero`.
#[test]
#[serial]
fn test_new_sprite_basic() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_sprite("hero", None));

    assert!(result.is_ok(), "Should create sprite: {:?}", result.err());
    let path = result.unwrap();
    assert!(path.exists(), "Sprite file should exist");
    assert!(
        path.to_string_lossy().contains("sprites/hero.pxl"),
        "Path should contain sprites/hero.pxl"
    );

    // Verify content
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"type\": \"sprite\""));
    assert!(content.contains("\"name\": \"hero\""));
    assert!(content.contains("\"regions\""));
}

/// @demo cli/new#sprite_with_palette
/// @title Create Sprite with Custom Palette
/// @description Create a sprite referencing a specific palette with `pxl new sprite enemy --palette enemies`.
#[test]
#[serial]
fn test_new_sprite_with_palette() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_sprite("enemy", Some("enemies")));

    assert!(result.is_ok(), "Should create sprite: {:?}", result.err());
    let path = result.unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"palette\": \"enemies\""), "Should reference enemies palette");
}

// ============================================================================
// New Animation Tests
// ============================================================================

/// @demo cli/new#animation_basic
/// @title Create New Animation
/// @description Create a new animation with `pxl new animation walk`.
#[test]
#[serial]
fn test_new_animation_basic() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_animation("walk", None));

    assert!(result.is_ok(), "Should create animation: {:?}", result.err());
    let path = result.unwrap();
    assert!(path.exists(), "Animation file should exist");
    assert!(
        path.to_string_lossy().contains("animations/walk.pxl"),
        "Path should contain animations/walk.pxl"
    );

    // Verify content includes frames and animation
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"type\": \"animation\""));
    assert!(content.contains("\"name\": \"walk\""));
    assert!(content.contains("\"frames\""));
    assert!(content.contains("\"duration\""));
}

/// @demo cli/new#animation_frames
/// @title Animation Includes Frame Sprites
/// @description Animation template includes frame sprites (e.g., walk_1, walk_2).
#[test]
#[serial]
fn test_new_animation_includes_frames() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_animation("run", Some("player")));

    assert!(result.is_ok());
    let path = result.unwrap();
    let content = fs::read_to_string(&path).unwrap();

    // Should include frame sprites
    assert!(content.contains("\"name\": \"run_1\""), "Should include run_1 frame");
    assert!(content.contains("\"name\": \"run_2\""), "Should include run_2 frame");

    // Animation should reference frames
    assert!(content.contains("\"run_1\""));
    assert!(content.contains("\"run_2\""));
}

// ============================================================================
// New Palette Tests
// ============================================================================

/// @demo cli/new#palette_basic
/// @title Create New Palette
/// @description Create a new palette with `pxl new palette forest`.
#[test]
#[serial]
fn test_new_palette_basic() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_palette("forest"));

    assert!(result.is_ok(), "Should create palette: {:?}", result.err());
    let path = result.unwrap();
    assert!(path.exists(), "Palette file should exist");
    assert!(
        path.to_string_lossy().contains("palettes/forest.pxl"),
        "Path should contain palettes/forest.pxl"
    );

    // Verify content
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"type\": \"palette\""));
    assert!(content.contains("\"name\": \"forest\""));
    assert!(content.contains("\"colors\""));
}

/// @demo cli/new#palette_starter_colors
/// @title Palette Includes Starter Colors
/// @description Palette template includes transparent, black, white, and starter colors.
#[test]
#[serial]
fn test_new_palette_starter_colors() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_palette("ui"));

    assert!(result.is_ok());
    let path = result.unwrap();
    let content = fs::read_to_string(&path).unwrap();

    // Should include starter colors
    assert!(content.contains("\"{_}\""), "Should include transparent");
    assert!(content.contains("\"{black}\""), "Should include black");
    assert!(content.contains("\"{white}\""), "Should include white");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// @demo cli/new#error_not_in_project
/// @title Error: Not in Project
/// @description Fail gracefully when not in a pixelsrc project.
#[test]
#[serial]
fn test_new_not_in_project() {
    let temp = TempDir::new().unwrap();
    // Don't create pxl.toml

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    let result = new_sprite("test", None);

    std::env::set_current_dir(original_dir).unwrap();

    match result {
        Err(ScaffoldError::NotInProject) => {}
        other => panic!("Expected NotInProject error, got: {:?}", other),
    }
}

/// @demo cli/new#error_file_exists
/// @title Error: File Already Exists
/// @description Fail gracefully when file already exists.
#[test]
#[serial]
fn test_new_file_exists() {
    let temp = TempDir::new().unwrap();
    let project_path = setup_test_project(&temp);

    // Create existing file
    let sprite_path = project_path.join("src/pxl/sprites/existing.pxl");
    fs::write(&sprite_path, "existing content").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project_path).unwrap();

    let result = new_sprite("existing", None);

    std::env::set_current_dir(original_dir).unwrap();

    match result {
        Err(ScaffoldError::FileExists(_)) => {}
        other => panic!("Expected FileExists error, got: {:?}", other),
    }
}

/// @demo cli/new#error_invalid_name
/// @title Error: Invalid Asset Name
/// @description Fail gracefully when asset name contains invalid characters.
#[test]
#[serial]
fn test_new_invalid_name() {
    let temp = TempDir::new().unwrap();

    let result = in_project(&temp, || new_sprite("Invalid-Name", None));

    match result {
        Err(ScaffoldError::InvalidName(name)) => {
            assert_eq!(name, "Invalid-Name");
        }
        other => panic!("Expected InvalidName error, got: {:?}", other),
    }
}

// ============================================================================
// Name Validation Tests
// ============================================================================

/// @demo cli/new#valid_names
/// @title Valid Asset Names
/// @description Asset names use lowercase letters, numbers, and underscores.
#[test]
#[serial]
fn test_valid_asset_names() {
    // Various valid names
    let valid_names = ["hero", "player_idle", "sprite1", "a", "walk_cycle_1"];

    for name in valid_names.iter() {
        let temp = TempDir::new().unwrap();
        let result = in_project(&temp, || new_sprite(name, None));
        assert!(result.is_ok(), "Name '{}' should be valid: {:?}", name, result.err());
    }
}

/// @demo cli/new#invalid_names
/// @title Invalid Asset Names
/// @description Names must start with lowercase letter, no spaces or special chars.
#[test]
#[serial]
fn test_invalid_asset_names() {
    let invalid_names = [
        ("", "empty"),
        ("Hero", "uppercase"),
        ("1sprite", "starts with number"),
        ("_underscore", "starts with underscore"),
        ("my-sprite", "contains hyphen"),
        ("my sprite", "contains space"),
    ];

    for (name, reason) in invalid_names.iter() {
        let temp = TempDir::new().unwrap();
        let result = in_project(&temp, || new_sprite(name, None));
        assert!(
            matches!(result, Err(ScaffoldError::InvalidName(_))),
            "Name '{}' ({}) should be invalid",
            name,
            reason
        );
    }
}
