//! CLI integration tests for the --scale option
//!
//! These tests verify end-to-end behavior of the --scale flag by running the
//! binary and checking output image dimensions.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the pxl binary
fn pxl_binary() -> PathBuf {
    // Try release first, then debug
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

/// Get image dimensions from a PNG file
fn get_image_dimensions(path: &Path) -> (u32, u32) {
    let img = image::open(path).expect("Failed to open output image");
    (img.width(), img.height())
}

/// Test that default scale (1) produces original-size output
#[test]
fn test_scale_default() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_default.png");

    // Render minimal_dot.jsonl (should be 1x1)
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success(), "Render failed: {}", String::from_utf8_lossy(&output.stderr));

    let (w, h) = get_image_dimensions(&output_path);
    assert_eq!((w, h), (1, 1), "Default scale should produce 1x1 output");
}

/// Test --scale 2 doubles the output dimensions
#[test]
fn test_scale_2x() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_2x.png");

    // Render minimal_dot.jsonl with --scale 2
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--scale")
        .arg("2")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Render with --scale 2 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (w, h) = get_image_dimensions(&output_path);
    assert_eq!((w, h), (2, 2), "Scale 2 should produce 2x2 from 1x1");
}

/// Test --scale 4 quadruples the output dimensions
#[test]
fn test_scale_4x() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_4x.png");

    // Render a 2x2 sprite with --scale 4
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/include_palette.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--scale")
        .arg("4")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Render with --scale 4 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (w, h) = get_image_dimensions(&output_path);
    // include_palette.jsonl has a 2x2 sprite
    assert_eq!((w, h), (8, 8), "Scale 4 should produce 8x8 from 2x2");
}

/// Test large scale factor (8x)
#[test]
fn test_scale_8x() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_8x.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--scale")
        .arg("8")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Render with --scale 8 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (w, h) = get_image_dimensions(&output_path);
    assert_eq!((w, h), (8, 8), "Scale 8 should produce 8x8 from 1x1");
}

/// Test maximum scale factor (16x)
#[test]
fn test_scale_max_16x() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_16x.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--scale")
        .arg("16")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Render with --scale 16 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (w, h) = get_image_dimensions(&output_path);
    assert_eq!((w, h), (16, 16), "Scale 16 should produce 16x16 from 1x1");
}

/// Test that scale factor above 16 is rejected
#[test]
fn test_scale_invalid_too_high() {
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--scale")
        .arg("17")
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Scale 17 should be rejected");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("17") || stderr.contains("invalid") || stderr.contains("error"),
        "Expected error message about invalid scale value, got: {stderr}"
    );
}

/// Test that scale factor 0 is rejected
#[test]
fn test_scale_invalid_zero() {
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--scale")
        .arg("0")
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Scale 0 should be rejected");
}

/// Test scale with spritesheet output
#[test]
fn test_scale_with_spritesheet() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_spritesheet.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("examples/walk_cycle.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--spritesheet")
        .arg("--scale")
        .arg("2")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Render spritesheet with --scale 2 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The spritesheet should have scaled dimensions
    let (w, h) = get_image_dimensions(&output_path);
    // Original frames are 8x8, scale 2 makes 16x16
    // 4 frames horizontal = 64 wide, 16 tall
    assert_eq!(h, 16, "Scaled spritesheet height should be 16 (8x8 frames * scale 2)");
    assert!(w > 16, "Scaled spritesheet width should be wider than single frame");
}

/// Test scale with GIF output
#[test]
fn test_scale_with_gif() {
    let output_dir = std::env::temp_dir().join("pxl_scale_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("scale_test.gif");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("examples/walk_cycle.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--gif")
        .arg("--scale")
        .arg("2")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Render GIF with --scale 2 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify GIF was created
    assert!(output_path.exists(), "GIF output file should exist at {output_path:?}");
}
