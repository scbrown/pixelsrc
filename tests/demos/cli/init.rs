//! Init Command Demo Tests
//!
//! Demonstrates the `pxl init` command functionality for initializing
//! new pixelsrc projects with various presets.

use pixelsrc::init::{init_project, InitError, Preset};
use std::fs;
use tempfile::TempDir;

/// Create a temporary directory for testing
fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

// ============================================================================
// Preset Tests
// ============================================================================

/// @demo cli/project#init
/// @title Init Command
/// @description The pxl init command initializes a new pixelsrc project.
#[test]
fn test_init_preset_minimal() {
    let temp = temp_dir();
    let project_path = temp.path().join("minimal_project");

    let result = init_project(&project_path, "minimal_project", "minimal");
    assert!(result.is_ok(), "Minimal preset should succeed: {:?}", result.err());

    // Verify minimal structure
    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.join("pxl.toml").exists(), "pxl.toml should exist");
}

/// @demo cli/init#preset_artist
/// @title Artist Preset Initialization
/// @description Initialize an artist project with `pxl init --preset artist`.
#[test]
fn test_init_preset_artist() {
    let temp = temp_dir();
    let project_path = temp.path().join("artist_project");

    let result = init_project(&project_path, "artist_project", "artist");
    assert!(result.is_ok(), "Artist preset should succeed: {:?}", result.err());

    // Verify artist structure includes palette directory
    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.join("pxl.toml").exists(), "pxl.toml should exist");
}

/// @demo cli/init#preset_animator
/// @title Animator Preset Initialization
/// @description Initialize an animator project with `pxl init --preset animator`.
#[test]
fn test_init_preset_animator() {
    let temp = temp_dir();
    let project_path = temp.path().join("animator_project");

    let result = init_project(&project_path, "animator_project", "animator");
    assert!(result.is_ok(), "Animator preset should succeed: {:?}", result.err());

    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.join("pxl.toml").exists(), "pxl.toml should exist");
}

/// @demo cli/init#preset_game
/// @title Game Preset Initialization
/// @description Initialize a full game project with `pxl init --preset game`.
#[test]
fn test_init_preset_game() {
    let temp = temp_dir();
    let project_path = temp.path().join("game_project");

    let result = init_project(&project_path, "game_project", "game");
    assert!(result.is_ok(), "Game preset should succeed: {:?}", result.err());

    // Game preset should include all directories
    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.join("pxl.toml").exists(), "pxl.toml should exist");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// @demo cli/init#error_existing_dir
/// @title Error: Directory Already Exists
/// @description Fail gracefully when directory already exists with files.
#[test]
fn test_init_error_existing_directory() {
    let temp = temp_dir();
    let project_path = temp.path().join("existing_project");

    // Create directory with content
    fs::create_dir_all(&project_path).unwrap();
    fs::write(project_path.join("some_file.txt"), "existing content").unwrap();

    let result = init_project(&project_path, "existing_project", "minimal");

    match result {
        Err(InitError::DirectoryExists(_)) => {}
        other => panic!("Expected DirectoryExists error, got: {:?}", other),
    }
}

/// @demo cli/init#error_unknown_preset
/// @title Error: Unknown Preset
/// @description Fail gracefully when preset name is invalid.
#[test]
fn test_init_error_unknown_preset() {
    let temp = temp_dir();
    let project_path = temp.path().join("unknown_preset_project");

    let result = init_project(&project_path, "unknown_preset_project", "invalid_preset");

    match result {
        Err(InitError::UnknownPreset(name)) => {
            assert_eq!(name, "invalid_preset", "Should report the invalid preset name");
        }
        other => panic!("Expected UnknownPreset error, got: {:?}", other),
    }
}

// ============================================================================
// Preset Parsing Tests
// ============================================================================

/// @demo cli/init#preset_parsing
/// @title Preset Name Parsing
/// @description Presets are case-insensitive.
#[test]
fn test_preset_parsing() {
    assert_eq!(Preset::from_str("minimal"), Some(Preset::Minimal));
    assert_eq!(Preset::from_str("MINIMAL"), Some(Preset::Minimal));
    assert_eq!(Preset::from_str("Minimal"), Some(Preset::Minimal));

    assert_eq!(Preset::from_str("artist"), Some(Preset::Artist));
    assert_eq!(Preset::from_str("animator"), Some(Preset::Animator));
    assert_eq!(Preset::from_str("game"), Some(Preset::Game));

    assert_eq!(Preset::from_str("unknown"), None);
}

/// @demo cli/init#empty_directory
/// @title Initialize in Empty Existing Directory
/// @description Allow initialization in empty existing directory.
#[test]
fn test_init_empty_existing_directory() {
    let temp = temp_dir();
    let project_path = temp.path().join("empty_project");

    // Create empty directory
    fs::create_dir_all(&project_path).unwrap();

    // Should succeed in empty directory
    let result = init_project(&project_path, "empty_project", "minimal");
    assert!(result.is_ok(), "Should succeed in empty directory: {:?}", result.err());
}
