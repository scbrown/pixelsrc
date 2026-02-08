//! CLI integration tests for the `pxl draw` command.
//!
//! Tests the full command-line interface for coordinate-based sprite editing.
//! Covers all draw operations (set, erase, rect, line, flood), error handling,
//! dry-run mode, output redirection, and round-trip validation.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the pxl binary.
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

/// Run pxl draw with the given arguments and return (stdout, stderr, success).
fn run_draw(args: &[&str]) -> (String, String, bool) {
    let output =
        Command::new(pxl_binary()).arg("draw").args(args).output().expect("Failed to execute pxl");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

/// Run pxl validate on a file and return (stdout, stderr, success).
fn run_validate(file: &str, args: &[&str]) -> (String, String, bool) {
    let output = Command::new(pxl_binary())
        .arg("validate")
        .arg(file)
        .args(args)
        .output()
        .expect("Failed to execute pxl");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

/// Create a temporary .pxl file with a simple sprite and return the path.
/// Uses a 4x4 sprite named "dot" with a single "x" rect region.
fn create_test_file(dir: &tempfile::TempDir) -> PathBuf {
    let path = dir.path().join("test.pxl");
    let content = r##"{"type":"palette","name":"colors","colors":{"_":"#00000000","x":"#FF0000","y":"#00FF00","z":"#0000FF"}}
{"type":"sprite","name":"dot","size":[4,4],"palette":"colors","regions":{"x":{"rect":[1,1,2,2],"z":0}}}"##;
    std::fs::write(&path, content).unwrap();
    path
}

/// Create a temp file with a 1x1 sprite.
fn create_1x1_file(dir: &tempfile::TempDir) -> PathBuf {
    let path = dir.path().join("tiny.pxl");
    let content = r##"{"type":"palette","name":"p","colors":{"_":"#00000000","x":"#FF0000"}}
{"type":"sprite","name":"pixel","size":[1,1],"palette":"p","regions":{}}"##;
    std::fs::write(&path, content).unwrap();
    path
}

/// Create a temp file with two sprites.
fn create_two_sprite_file(dir: &tempfile::TempDir) -> PathBuf {
    let path = dir.path().join("multi.pxl");
    let content = r##"{"type":"palette","name":"p","colors":{"_":"#00000000","x":"#FF0000","y":"#00FF00"}}
{"type":"sprite","name":"first","size":[4,4],"palette":"p","regions":{"x":{"rect":[0,0,4,4],"z":0}}}
{"type":"sprite","name":"second","size":[8,8],"palette":"p","regions":{"y":{"rect":[0,0,8,8],"z":0}}}"##;
    std::fs::write(&path, content).unwrap();
    path
}

// ============================================================================
// Basic set operation
// ============================================================================

#[test]
fn test_draw_set_pixel() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--set", "0,0={mark}"]);
    assert!(ok, "draw --set should succeed: {}", stderr);
    assert!(stderr.contains("Wrote"), "should confirm write");

    // Verify the file was modified: re-parse and check for "mark" region
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("mark"), "output should contain 'mark' token");
}

#[test]
fn test_draw_set_with_braces_and_quotes() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // Various token formats should all work
    let (_, _, ok) = run_draw(&[path_str, "--sprite", "dot", "--set", "0,0=\"{eye}\""]);
    assert!(ok, "quoted braces format should work");
}

#[test]
fn test_draw_set_bare_token() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, _, ok) = run_draw(&[path_str, "--sprite", "dot", "--set", "0,0=eye"]);
    assert!(ok, "bare token format should work");

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("eye"), "output should contain 'eye' token");
}

// ============================================================================
// Erase operation
// ============================================================================

#[test]
fn test_draw_erase_pixel() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--erase", "1,1"]);
    assert!(ok, "draw --erase should succeed: {}", stderr);

    // Erase adds a "_" region at high z
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"_\""), "should have transparent region");
}

// ============================================================================
// Rectangle operation
// ============================================================================

#[test]
fn test_draw_rect() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--rect", "0,0,4,2={sky}"]);
    assert!(ok, "draw --rect should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("sky"), "output should contain 'sky' token");
}

// ============================================================================
// Line operation
// ============================================================================

#[test]
fn test_draw_line() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--line", "0,0,3,3={rope}"]);
    assert!(ok, "draw --line should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("rope"), "output should contain 'rope' token");
}

// ============================================================================
// Flood fill operation
// ============================================================================

#[test]
fn test_draw_flood() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--flood", "0,0={water}"]);
    assert!(ok, "draw --flood should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("water"), "output should contain 'water' token");
}

// ============================================================================
// Multiple operations in one invocation
// ============================================================================

#[test]
fn test_draw_multiple_ops() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[
        path_str,
        "--sprite",
        "dot",
        "--rect",
        "0,0,4,2={sky}",
        "--rect",
        "0,2,4,2={ground}",
        "--set",
        "2,1={sun}",
    ]);
    assert!(ok, "multiple ops should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("sky"), "should have sky");
    assert!(content.contains("ground"), "should have ground");
    assert!(content.contains("sun"), "should have sun");
}

// ============================================================================
// Dry-run mode
// ============================================================================

#[test]
fn test_draw_dry_run_does_not_modify() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let original = std::fs::read_to_string(&path).unwrap();

    let (stdout, _, ok) =
        run_draw(&[path_str, "--sprite", "dot", "--set", "0,0={mark}", "--dry-run"]);
    assert!(ok, "dry-run should succeed");

    // File should be unchanged
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(original, after, "dry-run should not modify the file");

    // Output should show a diff
    assert!(stdout.contains("---") || stdout.contains("+++"), "should show diff output");
}

#[test]
fn test_draw_dry_run_no_changes() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // First, format the file so it's in canonical form
    let _ = Command::new(pxl_binary()).arg("fmt").arg(path_str).output().expect("fmt should work");

    // Now dry-run with no operations should show "No changes"
    let (stdout, _, ok) = run_draw(&[path_str, "--sprite", "dot", "--dry-run"]);
    assert!(ok, "dry-run with no ops should succeed");
    assert!(stdout.contains("No changes"), "should report no changes");
}

// ============================================================================
// Output redirection
// ============================================================================

#[test]
fn test_draw_output_to_different_file() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();
    let output_path = dir.path().join("output.pxl");
    let output_str = output_path.to_str().unwrap();

    let original = std::fs::read_to_string(&path).unwrap();

    let (_, stderr, ok) =
        run_draw(&[path_str, "--sprite", "dot", "--set", "0,0={mark}", "--output", output_str]);
    assert!(ok, "output redirection should succeed: {}", stderr);

    // Original file should be unchanged
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(original, after, "original should be unchanged");

    // Output file should contain the modification
    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("mark"), "output file should have 'mark' token");
}

// ============================================================================
// Error handling
// ============================================================================

#[test]
fn test_draw_missing_sprite_flag() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--set", "0,0={mark}"]);
    assert!(!ok, "should fail without --sprite");
    assert!(
        stderr.contains("--sprite") || stderr.contains("Available sprites"),
        "should mention --sprite requirement: {}",
        stderr
    );
}

#[test]
fn test_draw_sprite_not_found() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "nonexistent", "--set", "0,0={mark}"]);
    assert!(!ok, "should fail for unknown sprite");
    assert!(stderr.contains("not found"), "should report sprite not found: {}", stderr);
    assert!(stderr.contains("dot"), "should suggest available sprites: {}", stderr);
}

#[test]
fn test_draw_file_not_found() {
    let (_, stderr, ok) =
        run_draw(&["/nonexistent/file.pxl", "--sprite", "x", "--set", "0,0={mark}"]);
    assert!(!ok, "should fail for missing file");
    assert!(!stderr.is_empty(), "should have error message");
}

#[test]
fn test_draw_invalid_set_format() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--set", "bad"]);
    assert!(!ok, "invalid --set format should fail");
    assert!(!stderr.is_empty(), "should have error message: {}", stderr);
}

#[test]
fn test_draw_invalid_rect_format() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--rect", "1,2,3={x}"]);
    assert!(!ok, "rect with 3 coords should fail");
    assert!(
        stderr.contains("4 values") || stderr.contains("x,y,w,h"),
        "should explain format: {}",
        stderr
    );
}

#[test]
fn test_draw_invalid_line_format() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--line", "1,2={x}"]);
    assert!(!ok, "line with 2 coords should fail");
    assert!(
        stderr.contains("4 values") || stderr.contains("x1,y1,x2,y2"),
        "should explain format: {}",
        stderr
    );
}

#[test]
fn test_draw_empty_token() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--set", "0,0="]);
    assert!(!ok, "empty token should fail");
    assert!(stderr.contains("empty token"), "should report empty token: {}", stderr);
}

// ============================================================================
// Sprite selection with multiple sprites
// ============================================================================

#[test]
fn test_draw_select_second_sprite() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_two_sprite_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "second", "--set", "0,0={mark}"]);
    assert!(ok, "should work with second sprite: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("mark"), "should have mark token");
    // Both sprites should still be present
    assert!(content.contains("first"), "first sprite preserved");
    assert!(content.contains("second"), "second sprite preserved");
}

#[test]
fn test_draw_lists_sprites_without_sprite_flag() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_two_sprite_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--set", "0,0={mark}"]);
    assert!(!ok, "should fail without --sprite");
    assert!(stderr.contains("first"), "should list first sprite: {}", stderr);
    assert!(stderr.contains("second"), "should list second sprite: {}", stderr);
}

// ============================================================================
// Edge cases: 1x1 sprite
// ============================================================================

#[test]
fn test_draw_1x1_sprite_set() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_1x1_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "pixel", "--set", "0,0={dot}"]);
    assert!(ok, "1x1 set should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("dot"), "should have dot token");
}

#[test]
fn test_draw_1x1_sprite_rect() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_1x1_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "pixel", "--rect", "0,0,1,1={fill}"]);
    assert!(ok, "1x1 rect should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("fill"), "should have fill token");
}

#[test]
fn test_draw_1x1_sprite_erase() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_1x1_file(&dir);
    let path_str = path.to_str().unwrap();

    // First set a pixel, then erase it
    let (_, _, ok) = run_draw(&[path_str, "--sprite", "pixel", "--set", "0,0={x}"]);
    assert!(ok, "set should succeed");

    let (_, _, ok) = run_draw(&[path_str, "--sprite", "pixel", "--erase", "0,0"]);
    assert!(ok, "erase should succeed");

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"_\""), "should have transparent region after erase");
}

// ============================================================================
// Boundary draws
// ============================================================================

#[test]
fn test_draw_at_max_bounds() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // Set pixel at corner (3,3) of 4x4 sprite
    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--set", "3,3={corner}"]);
    assert!(ok, "setting at max bounds should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("corner"), "should have corner token");
}

#[test]
fn test_draw_rect_at_boundary() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // Rect filling bottom-right corner of 4x4 sprite
    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--rect", "2,2,2,2={corner}"]);
    assert!(ok, "rect at boundary should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("corner"), "should have corner token");
}

#[test]
fn test_draw_line_diagonal() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // Diagonal line across full 4x4 sprite
    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--line", "0,0,3,3={diag}"]);
    assert!(ok, "diagonal line should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("diag"), "should have diag token");
}

#[test]
fn test_draw_line_horizontal() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--line", "0,0,3,0={h}"]);
    assert!(ok, "horizontal line should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"h\""), "should have h token");
}

#[test]
fn test_draw_line_vertical() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--line", "0,0,0,3={v}"]);
    assert!(ok, "vertical line should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"v\""), "should have v token");
}

#[test]
fn test_draw_single_pixel_line() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // Line from (1,1) to (1,1) = single pixel
    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "dot", "--line", "1,1,1,1={dot}"]);
    assert!(ok, "single-pixel line should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"dot\""), "should have dot token");
}

// ============================================================================
// Round-trip validation: draw then validate
// ============================================================================

#[test]
fn test_draw_roundtrip_validates() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    // Apply multiple operations
    let (_, _, ok) = run_draw(&[
        path_str,
        "--sprite",
        "dot",
        "--rect",
        "0,0,4,1={sky}",
        "--set",
        "2,2={gem}",
        "--erase",
        "3,3",
    ]);
    assert!(ok, "draw should succeed");

    // Validate the output
    let (_, stderr, ok) = run_validate(path_str, &[]);
    assert!(ok, "modified file should pass validation: {}", stderr);
}

// ============================================================================
// Draw on example files (read-only via --output)
// ============================================================================

#[test]
fn test_draw_on_heart_dry_run() {
    let (stdout, _, ok) =
        run_draw(&["examples/heart.pxl", "--sprite", "heart", "--set", "3,2={eye}", "--dry-run"]);
    assert!(ok, "dry-run on heart should succeed");
    // Should show a diff since we're adding a new region
    assert!(
        stdout.contains("---") || stdout.contains("+++") || stdout.contains("No changes"),
        "should show diff output"
    );
}

#[test]
fn test_draw_on_coin_to_output() {
    let dir = tempfile::TempDir::new().unwrap();
    let output_path = dir.path().join("modified_coin.pxl");
    let output_str = output_path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[
        "examples/coin.pxl",
        "--sprite",
        "coin",
        "--set",
        "3,3={star}",
        "--output",
        output_str,
    ]);
    assert!(ok, "draw on coin should succeed: {}", stderr);

    // Verify output file exists and contains the new token
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("star"), "output should have star token");
    // Original should be unchanged
    let original = std::fs::read_to_string("examples/coin.pxl").unwrap();
    assert!(!original.contains("star"), "original coin should be unchanged");
}

// ============================================================================
// Multiple same-type operations (merge behavior)
// ============================================================================

#[test]
fn test_draw_multiple_sets_same_token() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[
        path_str,
        "--sprite",
        "dot",
        "--set",
        "0,0={mark}",
        "--set",
        "1,1={mark}",
        "--set",
        "2,2={mark}",
    ]);
    assert!(ok, "multiple sets of same token should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("mark"), "should have mark token");
}

#[test]
fn test_draw_multiple_lines_same_token() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) = run_draw(&[
        path_str,
        "--sprite",
        "dot",
        "--line",
        "0,0,3,0={border}",
        "--line",
        "0,3,3,3={border}",
    ]);
    assert!(ok, "multiple lines of same token should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("border"), "should have border token");
}

// ============================================================================
// Complex workflow: scaffold-like → draw → validate
// ============================================================================

#[test]
fn test_draw_build_scene_from_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("scene.pxl");
    let path_str = path.to_str().unwrap();

    // Create a minimal sprite with no regions
    let content = r##"{"type":"palette","name":"scene","colors":{"_":"#00000000","sky":"#87CEEB","grass":"#228B22","sun":"#FFD700"}}
{"type":"sprite","name":"landscape","size":[8,8],"palette":"scene","regions":{}}"##;
    std::fs::write(&path, content).unwrap();

    // Build the scene with multiple draw ops
    let (_, stderr, ok) = run_draw(&[
        path_str,
        "--sprite",
        "landscape",
        "--rect",
        "0,0,8,4={sky}",
        "--rect",
        "0,4,8,4={grass}",
        "--set",
        "6,1={sun}",
    ]);
    assert!(ok, "scene building should succeed: {}", stderr);

    // Validate the result
    let (_, val_stderr, val_ok) = run_validate(path_str, &[]);
    assert!(val_ok, "built scene should validate: {}", val_stderr);

    // Verify all tokens are present
    let output = std::fs::read_to_string(&path).unwrap();
    assert!(output.contains("sky"), "should have sky");
    assert!(output.contains("grass"), "should have grass");
    assert!(output.contains("sun"), "should have sun");
}

// ============================================================================
// Erase multiple pixels
// ============================================================================

#[test]
fn test_draw_erase_multiple() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = create_test_file(&dir);
    let path_str = path.to_str().unwrap();

    let (_, stderr, ok) =
        run_draw(&[path_str, "--sprite", "dot", "--erase", "1,1", "--erase", "2,2"]);
    assert!(ok, "multiple erases should succeed: {}", stderr);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("\"_\""), "should have transparent region");
}

// ============================================================================
// Non-sprite objects preserved after draw
// ============================================================================

#[test]
fn test_draw_preserves_animations() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("animated.pxl");
    let path_str = path.to_str().unwrap();

    let content = r##"{"type":"palette","name":"p","colors":{"_":"#00000000","x":"#FF0000"}}
{"type":"sprite","name":"s","size":[4,4],"palette":"p","regions":{"x":{"rect":[0,0,4,4],"z":0}}}
{"type":"animation","name":"anim","frames":["s"]}"##;
    std::fs::write(&path, content).unwrap();

    let (_, stderr, ok) = run_draw(&[path_str, "--sprite", "s", "--set", "0,0={mark}"]);
    assert!(ok, "draw should succeed: {}", stderr);

    let output = std::fs::read_to_string(&path).unwrap();
    assert!(output.contains("animation"), "animation should be preserved");
    assert!(output.contains("anim"), "animation name should be preserved");
}
