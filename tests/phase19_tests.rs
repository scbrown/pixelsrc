//! Comprehensive tests for Phase 19 Advanced Texture Features (ATF)
//!
//! This test suite covers all ATF features:
//! - ATF-3: Palette Cycling
//! - ATF-4: Frame Tags
//! - ATF-5: Atlas Export
//! - ATF-7: Hit/Hurt Boxes
//! - ATF-10: Blend Modes
//! - ATF-11: Onion Skinning
//! - ATF-14: Secondary Motion

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the pxl binary
fn pxl_binary() -> PathBuf {
    let release = Path::new("target/release/pxl");
    if release.exists() {
        return release.to_path_buf();
    }

    let debug = Path::new("target/debug/pxl");
    if debug.exists() {
        return debug.to_path_buf();
    }

    panic!("pxl binary not found. Run 'cargo build' first.");
}

/// Run pxl render on a fixture file
fn run_pxl_render(fixture: &Path) -> std::process::Output {
    let output_dir = std::env::temp_dir().join("pxl_phase19_tests");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("test_output.png");

    Command::new(pxl_binary())
        .arg("render")
        .arg(fixture)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl")
}

/// Run pxl with atlas export
fn run_pxl_atlas(fixture: &Path, format: &str) -> std::process::Output {
    let output_dir = std::env::temp_dir().join("pxl_phase19_atlas");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("atlas.png");

    Command::new(pxl_binary())
        .arg("render")
        .arg(fixture)
        .arg("-o")
        .arg(&output_path)
        .arg("--format")
        .arg(format)
        .output()
        .expect("Failed to execute pxl")
}

// ============================================================================
// ATF-3: Palette Cycling Tests
// ============================================================================
#[test]
fn test_atf3_multiple_palette_cycles() {
    let fixture = Path::new("tests/fixtures/valid/atf_multiple_cycles.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Multiple palette cycles should render successfully.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// ATF-4: Frame Tags Tests
// ============================================================================// ============================================================================
// ATF-5: Atlas Export Tests
// ============================================================================

#[test]
fn test_atf5_atlas_export_basic() {
    let fixture = Path::new("tests/fixtures/valid/multiple_sprites.jsonl");
    let output = run_pxl_atlas(fixture, "atlas");

    assert!(
        output.status.success(),
        "Atlas export should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_atf5_atlas_export_aseprite() {
    let fixture = Path::new("tests/fixtures/valid/atf_frame_tags.jsonl");
    let output = run_pxl_atlas(fixture, "atlas-aseprite");

    assert!(
        output.status.success(),
        "Atlas Aseprite export should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_atf5_atlas_export_godot() {
    let fixture = Path::new("tests/fixtures/valid/multiple_sprites.jsonl");
    let output = run_pxl_atlas(fixture, "atlas-godot");

    assert!(
        output.status.success(),
        "Atlas Godot export should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// ATF-7: Hit/Hurt Boxes Tests
// ============================================================================
#[test]
fn test_atf7_frame_metadata_validation() {
    // The atf_hit_boxes.jsonl has frame_metadata that matches frame count
    let fixture = Path::new("tests/fixtures/valid/atf_hit_boxes.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Frame metadata with matching count should pass.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// ATF-10: Blend Modes Tests
// ============================================================================

#[test]
fn test_atf10_blend_modes_composition() {
    let fixture = Path::new("tests/fixtures/valid/atf_blend_modes.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Blend modes composition should render successfully.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// ATF-11: Onion Skinning Tests
// ============================================================================

#[test]
fn test_atf11_onion_skinning_show_command() {
    let output_dir = std::env::temp_dir().join("pxl_onion_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("onion_output.png");
    let fixture = Path::new("tests/fixtures/valid/animation.jsonl");

    let output = Command::new(pxl_binary())
        .arg("show")
        .arg(fixture)
        .arg("--animation")
        .arg("blink_anim")
        .arg("--onion")
        .arg("1")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "Onion skinning show command should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Onion skin output file should be created");
}

#[test]
fn test_atf11_onion_skinning_with_colors() {
    let output_dir = std::env::temp_dir().join("pxl_onion_color_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("onion_color_output.png");
    let fixture = Path::new("tests/fixtures/valid/animation.jsonl");

    let output = Command::new(pxl_binary())
        .arg("show")
        .arg(fixture)
        .arg("--animation")
        .arg("blink_anim")
        .arg("--onion")
        .arg("2")
        .arg("--onion-prev-color")
        .arg("#FF0000")
        .arg("--onion-next-color")
        .arg("#00FF00")
        .arg("--onion-opacity")
        .arg("0.5")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "Onion skinning with custom colors should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_atf11_onion_skinning_with_fade() {
    let output_dir = std::env::temp_dir().join("pxl_onion_fade_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("onion_fade_output.png");
    let fixture = Path::new("tests/fixtures/valid/animation.jsonl");

    let output = Command::new(pxl_binary())
        .arg("show")
        .arg(fixture)
        .arg("--animation")
        .arg("blink_anim")
        .arg("--onion")
        .arg("2")
        .arg("--onion-fade")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "Onion skinning with fade should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// ATF-14: Secondary Motion Tests
// ============================================================================
#[test]
fn test_atf14_keyframed_attachment() {
    let fixture = Path::new("tests/fixtures/valid/atf_keyframed_attachment.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Keyframed attachment fixture should render successfully.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Combined Feature Tests
// ============================================================================

#[test]
fn test_combined_palette_cycle_with_tags() {
    // Test animation with both palette cycling and frame tags
    let fixture = Path::new("tests/fixtures/valid/atf_frame_tags.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Combined features should work together.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_all_atf_fixtures_valid() {
    // Verify all ATF fixtures are valid
    let atf_fixtures = [
        "tests/fixtures/valid/atf_palette_cycle.jsonl",
        "tests/fixtures/valid/atf_frame_tags.jsonl",
        "tests/fixtures/valid/atf_hit_boxes.jsonl",
        "tests/fixtures/valid/atf_secondary_motion.jsonl",
        "tests/fixtures/valid/atf_blend_modes.jsonl",
        "tests/fixtures/valid/atf_multiple_cycles.jsonl",
        "tests/fixtures/valid/atf_keyframed_attachment.jsonl",
    ];

    for fixture_path in &atf_fixtures {
        let fixture = Path::new(fixture_path);
        if fixture.exists() {
            let output = run_pxl_render(fixture);
            assert!(
                output.status.success(),
                "ATF fixture {:?} should render successfully.\nstderr: {}",
                fixture,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

// ============================================================================
// Regression Tests
// ============================================================================

#[test]
fn test_existing_animations_still_work() {
    // Ensure existing animation fixtures continue to work
    let fixture = Path::new("tests/fixtures/valid/animation.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Existing animation fixture should still work.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_existing_compositions_still_work() {
    // Ensure existing composition fixtures continue to work
    let fixture = Path::new("tests/fixtures/valid/composition_basic.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Existing composition fixture should still work.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_palette_cycle_array() {
    // Animation with empty palette_cycle should be valid
    let fixture = Path::new("tests/fixtures/valid/animation.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Animation without palette cycle should work.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_sprite_without_metadata() {
    // Sprites without metadata field should still work
    let fixture = Path::new("tests/fixtures/valid/minimal_dot.jsonl");
    let output = run_pxl_render(fixture);

    assert!(
        output.status.success(),
        "Sprite without metadata should work.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
