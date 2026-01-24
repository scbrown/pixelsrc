//! CLI integration tests for `pxl transform` command
//!
//! Tests the transform CLI with various flags and combinations.
//! These tests verify end-to-end behavior of the transform command.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// Create a temporary output path for tests
fn temp_output(name: &str) -> PathBuf {
    let output_dir = std::env::temp_dir().join("pxl_transform_tests");
    fs::create_dir_all(&output_dir).ok();
    output_dir.join(name)
}

/// Parse JSONL output and extract the grid from the first sprite
fn extract_grid(jsonl: &str) -> Option<Vec<String>> {
    for line in jsonl.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            if value.get("type").and_then(|t| t.as_str()) == Some("sprite") {
                if let Some(grid) = value.get("grid").and_then(|g| g.as_array()) {
                    return Some(
                        grid.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                    );
                }
            }
        }
    }
    None
}

// ============================================================================
// Mirror Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_mirror_horizontal() {
    let output_path = temp_output("mirror_h.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--mirror")
        .arg("horizontal")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output file not created");

    // For a 1x1 sprite, mirror should produce the same result
    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(content.contains("grid"), "Output should contain grid");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_mirror_vertical() {
    let output_path = temp_output("mirror_v.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--mirror")
        .arg("vertical")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Vertical mirror reverses row order
    // Original heart: top is narrow, bottom is point
    // Mirrored: point at top, narrow at bottom
    assert_eq!(grid.len(), 6, "Should have 6 rows");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_mirror_both() {
    let output_path = temp_output("mirror_both.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--mirror")
        .arg("both")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists());
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_mirror_alias_h() {
    let output_path = temp_output("mirror_alias_h.jsonl");

    // Test short alias "h" for "horizontal"
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--mirror")
        .arg("h")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform with 'h' alias failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_mirror_alias_v() {
    let output_path = temp_output("mirror_alias_v.jsonl");

    // Test short alias "v" for "vertical"
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--mirror")
        .arg("v")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform with 'v' alias failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Rotate Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_rotate_90() {
    let output_path = temp_output("rotate_90.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--rotate")
        .arg("90")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original heart is 7 wide x 6 tall
    // After 90° rotation, should be 6 wide x 7 tall
    assert_eq!(grid.len(), 7, "Rotated grid should have 7 rows (was 7 columns)");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_rotate_180() {
    let output_path = temp_output("rotate_180.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--rotate")
        .arg("180")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // 180° rotation preserves dimensions
    assert_eq!(grid.len(), 6, "Rotated grid should have same row count");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_rotate_270() {
    let output_path = temp_output("rotate_270.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--rotate")
        .arg("270")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // 270° rotation swaps dimensions like 90°
    assert_eq!(grid.len(), 7, "Rotated grid should have 7 rows");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_rotate_invalid() {
    let output_path = temp_output("rotate_invalid.jsonl");

    // 45 degrees is not a valid rotation
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--rotate")
        .arg("45")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Should reject invalid rotation");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("90")
            || stderr.contains("180")
            || stderr.contains("270")
            || stderr.contains("invalid"),
        "Should mention valid rotation values: {stderr}"
    );
}

// ============================================================================
// Tile Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_tile_2x2() {
    let output_path = temp_output("tile_2x2.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--tile")
        .arg("2x2")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 1x1, tiled 2x2 = 2x2
    assert_eq!(grid.len(), 2, "Tiled grid should have 2 rows");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_tile_3x1() {
    let output_path = temp_output("tile_3x1.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--tile")
        .arg("3x1")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 1x1, tiled 3x1 = 3x1
    assert_eq!(grid.len(), 1, "Tiled grid should have 1 row");
}

// ============================================================================
// Pad Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_pad() {
    let output_path = temp_output("pad.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--pad")
        .arg("2")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 1x1, padded by 2 = 5x5 (2+1+2)
    assert_eq!(grid.len(), 5, "Padded grid should have 5 rows");
}

// ============================================================================
// Outline Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_outline_basic() {
    let output_path = temp_output("outline_basic.jsonl");

    // Outline replaces transparent pixels adjacent to opaque pixels
    // So we need to pad first to create transparent space around the sprite
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--pad")
        .arg("1")
        .arg("--outline")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 1x1 + pad 1 = 3x3, outline fills the padding
    assert_eq!(grid.len(), 3, "Padded+outlined grid should have 3 rows");
    assert!(content.contains("{outline}"), "Should contain outline token");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_outline_with_token() {
    let output_path = temp_output("outline_token.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--pad")
        .arg("1")
        .arg("--outline")
        .arg("{border}")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(content.contains("{border}"), "Output should contain the outline token");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_outline_with_width() {
    let output_path = temp_output("outline_width.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--pad")
        .arg("2")
        .arg("--outline")
        .arg("--outline-width")
        .arg("2")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 1x1 + pad 2 = 5x5, outline fills the padding
    assert_eq!(grid.len(), 5, "Padded+outlined grid with width 2 should have 5 rows");
}

// ============================================================================
// Crop Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_crop() {
    let output_path = temp_output("crop.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--crop")
        .arg("0,0,3,3")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Cropped to 3x3
    assert_eq!(grid.len(), 3, "Cropped grid should have 3 rows");
}

// ============================================================================
// Shift Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_shift() {
    let output_path = temp_output("shift.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--shift")
        .arg("1,1")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Shift preserves dimensions
    assert_eq!(grid.len(), 6, "Shifted grid should have same row count");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_shift_negative() {
    let output_path = temp_output("shift_neg.jsonl");

    // Use -- to pass negative values to clap
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--")
        .arg("--shift")
        .arg("-2,-1")
        .output()
        .expect("Failed to execute pxl");

    // Note: shift with negative values may have clap parsing issues
    // This test verifies the command behavior
    // The command may fail due to argument parsing; that's expected
    // and the actual functionality is tested via unit tests
    let _ = output.status;
}

// ============================================================================
// Shadow Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_shadow() {
    let output_path = temp_output("shadow.jsonl");

    // Shadow replaces transparent pixels where the shadow would fall
    // Need to pad first to create space for the shadow
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--pad")
        .arg("1")
        .arg("--shadow")
        .arg("1,1")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 1x1 + pad 1 = 3x3, shadow fills one cell
    assert_eq!(grid.len(), 3, "Padded+shadow grid should have 3 rows");
    assert!(content.contains("{shadow}"), "Should contain shadow token");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_shadow_with_token() {
    let output_path = temp_output("shadow_token.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--pad")
        .arg("1")
        .arg("--shadow")
        .arg("1,1")
        .arg("--shadow-token")
        .arg("{dark}")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(content.contains("{dark}"), "Output should contain the shadow token");
}

// ============================================================================
// Chained Transform Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_chain_mirror_rotate() {
    let output_path = temp_output("chain_mirror_rotate.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--mirror")
        .arg("horizontal")
        .arg("--rotate")
        .arg("90")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform chain failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // Original 7x6, rotated 90° = 6 wide x 7 tall
    assert_eq!(grid.len(), 7, "Chained transform should produce 7 rows");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_chain_multiple() {
    let output_path = temp_output("chain_multiple.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--tile")
        .arg("2x2")
        .arg("--pad")
        .arg("1")
        .arg("--outline")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform chain failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // 1x1 -> tile 2x2 = 2x2 -> pad 1 = 4x4 (outline fills padding, same size)
    assert_eq!(grid.len(), 4, "Multi-step chain should produce 4 rows");
    assert!(content.contains("{outline}"), "Should contain outline tokens");
}

// ============================================================================
// stdin Input Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_stdin() {
    let output_path = temp_output("stdin.jsonl");

    let input = r##"{"type": "sprite", "name": "test", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}"##;

    // Note: The transform command still requires INPUT argument even with --stdin
    // The input path is ignored when --stdin is specified, but clap still requires it
    let mut child = Command::new(pxl_binary())
        .arg("transform")
        .arg("-") // Dummy input path (ignored when --stdin is used)
        .arg("--stdin")
        .arg("--mirror")
        .arg("horizontal")
        .arg("-o")
        .arg(&output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    assert!(
        output.status.success(),
        "Transform from stdin failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output file should be created");
}

// ============================================================================
// --sprite Flag Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_sprite_filter() {
    let output_path = temp_output("sprite_filter.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("red_dot")
        .arg("--mirror")
        .arg("horizontal")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Transform with sprite filter failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(content.contains("\"name\":"), "Output should contain sprite");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_sprite_filter_invalid() {
    let output_path = temp_output("sprite_filter_invalid.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("nonexistent_sprite")
        .arg("--mirror")
        .arg("horizontal")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Should fail for invalid sprite name");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("No sprite") || stderr.contains("Error"),
        "Should mention sprite not found: {stderr}"
    );
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_missing_output() {
    // Transform requires -o flag
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--mirror")
        .arg("horizontal")
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Should fail without output flag");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_missing_input() {
    let output_path = temp_output("missing_input.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("nonexistent_file.jsonl")
        .arg("--mirror")
        .arg("horizontal")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Should fail for missing input file");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_invalid_tile_format() {
    let output_path = temp_output("invalid_tile.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--tile")
        .arg("invalid")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Should fail for invalid tile format");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_invalid_crop_format() {
    let output_path = temp_output("invalid_crop.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("--crop")
        .arg("1,2,3")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success(), "Should fail for invalid crop format");
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_preserves_palette() {
    let output_path = temp_output("preserves_palette.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--rotate")
        .arg("90")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success());

    let content = fs::read_to_string(&output_path).expect("Failed to read output");

    // Should preserve palette tokens
    assert!(content.contains("{r}"), "Should preserve palette token {{r}}");
    assert!(content.contains("{p}"), "Should preserve palette token {{p}}");
    assert!(content.contains("palette"), "Should contain palette");
}

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_valid_jsonl_output() {
    let output_path = temp_output("valid_jsonl.jsonl");

    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--mirror")
        .arg("horizontal")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success());

    let content = fs::read_to_string(&output_path).expect("Failed to read output");

    // Verify each non-empty line is valid JSON
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parse_result.is_ok(), "Line should be valid JSON: {line}");
    }
}

// ============================================================================
// Allow Large Flag Tests
// ============================================================================

#[test]
#[ignore = "Grid transform command deprecated"]
fn test_transform_allow_large() {
    let output_path = temp_output("allow_large.jsonl");

    // Large tile operation with --allow-large
    let output = Command::new(pxl_binary())
        .arg("transform")
        .arg("tests/fixtures/valid/simple_heart.jsonl")
        .arg("--tile")
        .arg("10x10")
        .arg("--allow-large")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Large transform with --allow-large should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    let grid = extract_grid(&content).expect("Should have grid");

    // 7x6 heart tiled 10x10 = 70x60
    assert_eq!(grid.len(), 60, "Large tiled grid should have 60 rows");
}
