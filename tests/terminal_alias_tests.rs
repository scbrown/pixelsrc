//! Comprehensive tests for terminal display, alias, and grid commands
//!
//! Tests the following CLI commands:
//! - pxl show: Display sprites with ANSI colored terminal output
//! - pxl grid: Display sprites with row/column coordinate headers
//! - pxl inline: Display grids with column-aligned spacing
//! - pxl alias: Extract repeated patterns into single-letter aliases
//! - pxl sketch: Create sprites from simple text grid input

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

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

// =============================================================================
// pxl show command tests
// =============================================================================

#[test]
fn test_show_basic() {
    let output = Command::new(pxl_binary())
        .arg("show")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "pxl show should succeed on valid file\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain ANSI escape sequences for colored output
    assert!(
        stdout.contains("\x1b["),
        "Output should contain ANSI escape sequences"
    );
    // Should contain Legend section
    assert!(stdout.contains("Legend:"), "Output should contain a legend");
}

#[test]
fn test_show_with_sprite_filter() {
    let output = Command::new(pxl_binary())
        .arg("show")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("green_dot")
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "pxl show --sprite should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show the sprite name
    assert!(
        stdout.contains("green_dot"),
        "Output should mention the sprite name"
    );
}

#[test]
fn test_show_nonexistent_sprite() {
    let output = Command::new(pxl_binary())
        .arg("show")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("nonexistent_sprite")
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        !output.status.success(),
        "pxl show should fail for non-existent sprite"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No sprite named"),
        "Should report sprite not found"
    );
}

#[test]
fn test_show_missing_file() {
    let output = Command::new(pxl_binary())
        .arg("show")
        .arg("nonexistent_file.jsonl")
        .output()
        .expect("Failed to execute pxl show");

    assert!(!output.status.success(), "pxl show should fail for missing file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cannot open") || stderr.contains("Error"),
        "Should report file not found"
    );
}

#[test]
fn test_show_alias_test_fixture() {
    let output = Command::new(pxl_binary())
        .arg("show")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "pxl show should succeed on alias_test fixture\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify legend contains color tokens
    assert!(
        stdout.contains("Legend:"),
        "Output should contain Legend section"
    );
}

// =============================================================================
// pxl grid command tests
// =============================================================================

#[test]
fn test_grid_basic() {
    let output = Command::new(pxl_binary())
        .arg("grid")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .output()
        .expect("Failed to execute pxl grid");

    assert!(
        output.status.success(),
        "pxl grid should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain coordinate header (column numbers)
    assert!(stdout.contains(" 0"), "Output should contain column header 0");
    // Should contain the grid border character
    assert!(
        stdout.contains("\u{250C}") || stdout.contains("\u{2502}"),
        "Output should contain grid border characters"
    );
}

#[test]
fn test_grid_multiple_sprites() {
    let output = Command::new(pxl_binary())
        .arg("grid")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .output()
        .expect("Failed to execute pxl grid");

    assert!(
        output.status.success(),
        "pxl grid should succeed with multiple sprites\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_grid_with_sprite_filter() {
    let output = Command::new(pxl_binary())
        .arg("grid")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("red_dot")
        .output()
        .expect("Failed to execute pxl grid");

    assert!(
        output.status.success(),
        "pxl grid --sprite should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("red_dot"),
        "Output should mention the sprite name"
    );
}

#[test]
fn test_grid_full_names() {
    let output = Command::new(pxl_binary())
        .arg("grid")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .arg("--full")
        .output()
        .expect("Failed to execute pxl grid --full");

    assert!(
        output.status.success(),
        "pxl grid --full should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Full names mode should show full token names including braces
    assert!(
        stdout.contains("{_}") || stdout.contains("{a}") || stdout.contains("{b}"),
        "Full names mode should show complete token names"
    );
}

#[test]
fn test_grid_nonexistent_sprite() {
    let output = Command::new(pxl_binary())
        .arg("grid")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("nonexistent")
        .output()
        .expect("Failed to execute pxl grid");

    assert!(
        !output.status.success(),
        "pxl grid should fail for non-existent sprite"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No sprite named"),
        "Should report sprite not found"
    );
}

#[test]
fn test_grid_missing_file() {
    let output = Command::new(pxl_binary())
        .arg("grid")
        .arg("nonexistent.jsonl")
        .output()
        .expect("Failed to execute pxl grid");

    assert!(!output.status.success(), "pxl grid should fail for missing file");
}

// =============================================================================
// pxl inline command tests
// =============================================================================

#[test]
fn test_inline_basic() {
    let output = Command::new(pxl_binary())
        .arg("inline")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .output()
        .expect("Failed to execute pxl inline");

    assert!(
        output.status.success(),
        "pxl inline should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain tokens in brace format
    assert!(
        stdout.contains("{_}") || stdout.contains("{a}") || stdout.contains("{b}"),
        "Inline output should contain tokenized grid"
    );
    // Column alignment means multiple spaces between tokens
    assert!(
        stdout.contains("  "),
        "Inline output should have column alignment (multiple spaces)"
    );
}

#[test]
fn test_inline_multiple_sprites() {
    let output = Command::new(pxl_binary())
        .arg("inline")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .output()
        .expect("Failed to execute pxl inline");

    assert!(
        output.status.success(),
        "pxl inline should succeed with multiple sprites\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // When multiple sprites, should show sprite names as comments
    assert!(
        stdout.contains("# green_dot") || stdout.contains("# yellow_dot") || stdout.contains("# red_dot"),
        "Multiple sprites should be labeled with comments"
    );
}

#[test]
fn test_inline_with_sprite_filter() {
    let output = Command::new(pxl_binary())
        .arg("inline")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("yellow_dot")
        .output()
        .expect("Failed to execute pxl inline");

    assert!(
        output.status.success(),
        "pxl inline --sprite should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // When filtering to single sprite, should NOT show comment header
    assert!(
        !stdout.contains("# green_dot") && !stdout.contains("# red_dot"),
        "Filtered output should not show other sprites"
    );
}

#[test]
fn test_inline_nonexistent_sprite() {
    let output = Command::new(pxl_binary())
        .arg("inline")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("missing_sprite")
        .output()
        .expect("Failed to execute pxl inline");

    assert!(
        !output.status.success(),
        "pxl inline should fail for non-existent sprite"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No sprite named"),
        "Should report sprite not found"
    );
}

#[test]
fn test_inline_missing_file() {
    let output = Command::new(pxl_binary())
        .arg("inline")
        .arg("nonexistent.jsonl")
        .output()
        .expect("Failed to execute pxl inline");

    assert!(
        !output.status.success(),
        "pxl inline should fail for missing file"
    );
}

// =============================================================================
// pxl alias command tests
// =============================================================================

#[test]
fn test_alias_basic() {
    let output = Command::new(pxl_binary())
        .arg("alias")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(
        output.status.success(),
        "pxl alias should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("pxl alias should output valid JSON");

    // Should have aliases and grid keys
    assert!(
        json.get("aliases").is_some(),
        "Output should contain 'aliases' key"
    );
    assert!(json.get("grid").is_some(), "Output should contain 'grid' key");
}

#[test]
fn test_alias_json_structure() {
    let output = Command::new(pxl_binary())
        .arg("alias")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify aliases structure
    let aliases = json.get("aliases").expect("aliases key should exist");
    assert!(aliases.is_object(), "aliases should be an object");

    // Should have underscore mapping for transparent
    let aliases_obj = aliases.as_object().unwrap();
    assert!(
        aliases_obj.contains_key("_"),
        "Aliases should contain '_' for transparent"
    );
    assert_eq!(
        aliases_obj.get("_").unwrap(),
        "_",
        "'_' alias should map to '_'"
    );

    // Verify grid structure
    let grid = json.get("grid").expect("grid key should exist");
    assert!(grid.is_array(), "grid should be an array");
    let grid_arr = grid.as_array().unwrap();
    assert!(!grid_arr.is_empty(), "grid should not be empty");

    // Each grid row should be a string with alias tokens
    for row in grid_arr {
        let row_str = row.as_str().expect("grid row should be string");
        assert!(
            row_str.contains("{") && row_str.contains("}"),
            "Grid rows should contain tokenized aliases"
        );
    }
}

#[test]
fn test_alias_multiple_sprites() {
    let output = Command::new(pxl_binary())
        .arg("alias")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(
        output.status.success(),
        "pxl alias should succeed with multiple sprites\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Multiple sprites means multiple JSON objects in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Count the number of "aliases" keys (one per sprite)
    let alias_count = stdout.matches("\"aliases\"").count();
    assert!(
        alias_count >= 3,
        "Should output alias JSON for each sprite (expected 3, got {})",
        alias_count
    );
}

#[test]
fn test_alias_with_sprite_filter() {
    let output = Command::new(pxl_binary())
        .arg("alias")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("green_dot")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(
        output.status.success(),
        "pxl alias --sprite should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output single JSON object
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert!(json.get("aliases").is_some());
}

#[test]
fn test_alias_nonexistent_sprite() {
    let output = Command::new(pxl_binary())
        .arg("alias")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("--sprite")
        .arg("nonexistent")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(
        !output.status.success(),
        "pxl alias should fail for non-existent sprite"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No sprite named"),
        "Should report sprite not found"
    );
}

#[test]
fn test_alias_missing_file() {
    let output = Command::new(pxl_binary())
        .arg("alias")
        .arg("nonexistent.jsonl")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(
        !output.status.success(),
        "pxl alias should fail for missing file"
    );
}

// =============================================================================
// pxl sketch command tests
// =============================================================================

#[test]
fn test_sketch_from_stdin() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    // Write simple grid to stdin
    let stdin = cmd.stdin.as_mut().expect("Failed to get stdin");
    stdin
        .write_all(b"_ _ b b\n_ b c b\nb b b _\n")
        .expect("Failed to write to stdin");

    let output = cmd.wait_with_output().expect("Failed to wait for command");

    assert!(
        output.status.success(),
        "pxl sketch should succeed with stdin input\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("pxl sketch should output valid JSON");

    // Verify sprite structure
    assert_eq!(json["type"], "sprite", "Should be a sprite type");
    assert_eq!(json["name"], "sketch", "Default name should be 'sketch'");
    assert_eq!(json["size"], serde_json::json!([4, 3]), "Size should be [4, 3]");

    // Verify grid
    let grid = json["grid"].as_array().expect("grid should be array");
    assert_eq!(grid.len(), 3, "Grid should have 3 rows");
    assert_eq!(grid[0], "{_}{_}{b}{b}");
}

#[test]
fn test_sketch_from_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_file = temp_dir.path().join("input.txt");

    fs::write(&input_file, "a b\nc d\n").expect("Failed to write input file");

    let output = Command::new(pxl_binary())
        .arg("sketch")
        .arg(&input_file)
        .output()
        .expect("Failed to execute pxl sketch");

    assert!(
        output.status.success(),
        "pxl sketch should succeed with file input\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["size"], serde_json::json!([2, 2]));
}

#[test]
fn test_sketch_with_name() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .arg("--name")
        .arg("my_sprite")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"x x\nx x\n").unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["name"], "my_sprite", "Custom name should be used");
}

#[test]
fn test_sketch_with_palette_ref() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .arg("--palette")
        .arg("@synthwave")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"a b\n").unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["palette"], "@synthwave",
        "Palette reference should be used"
    );
}

#[test]
fn test_sketch_inline_palette() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"_ a\na _\n").unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have inline palette object
    let palette = json.get("palette").expect("Should have palette");
    assert!(palette.is_object(), "Palette should be an object (inline)");

    let palette_obj = palette.as_object().unwrap();
    // Transparent token should be transparent
    assert_eq!(palette_obj.get("{_}").unwrap(), "#00000000");
    // Other tokens should have placeholder color
    assert_eq!(palette_obj.get("{a}").unwrap(), "#000000");
}

#[test]
fn test_sketch_output_to_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("output.json");

    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .arg("--output")
        .arg(&output_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"x y\nz w\n").unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "pxl sketch --output should succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify file was created
    assert!(output_file.exists(), "Output file should be created");

    // Verify content
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Output should be valid JSON");
    assert_eq!(json["type"], "sprite");
}

#[test]
fn test_sketch_empty_input() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    {
        let stdin = cmd.stdin.as_mut().unwrap();
        stdin.write_all(b"").unwrap();
    }

    let output = cmd.wait_with_output().unwrap();
    assert!(
        !output.status.success(),
        "pxl sketch should fail for empty input"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Empty input"),
        "Should report empty input error"
    );
}

#[test]
fn test_sketch_missing_input_file() {
    let output = Command::new(pxl_binary())
        .arg("sketch")
        .arg("nonexistent_file.txt")
        .output()
        .expect("Failed to execute pxl sketch");

    assert!(
        !output.status.success(),
        "pxl sketch should fail for missing file"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cannot read") || stderr.contains("Error"),
        "Should report file not found"
    );
}

#[test]
fn test_sketch_preserves_underscore_transparency() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"_ x _\nx _ x\n").unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Grid should preserve underscore as {_}
    let grid = json["grid"].as_array().unwrap();
    assert_eq!(grid[0], "{_}{x}{_}");
    assert_eq!(grid[1], "{x}{_}{x}");

    // Palette should have transparent for {_}
    let palette = json["palette"].as_object().unwrap();
    assert_eq!(palette.get("{_}").unwrap(), "#00000000");
}

// =============================================================================
// Cross-command integration tests
// =============================================================================

#[test]
fn test_sketch_then_show() {
    // Create a sprite with sketch
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sprite_file = temp_dir.path().join("sprite.jsonl");

    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .arg("--output")
        .arg(&sprite_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"r g b\ng b r\nb r g\n").unwrap();
    let sketch_output = cmd.wait_with_output().unwrap();
    assert!(sketch_output.status.success());

    // Now show the created sprite
    let show_output = Command::new(pxl_binary())
        .arg("show")
        .arg(&sprite_file)
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        show_output.status.success(),
        "pxl show should work on sketch output\nstderr: {}",
        String::from_utf8_lossy(&show_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(stdout.contains("Legend:"));
}

#[test]
fn test_inline_then_alias_consistency() {
    // The inline command shows the original tokens
    let inline_output = Command::new(pxl_binary())
        .arg("inline")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .output()
        .expect("Failed to execute pxl inline");

    assert!(inline_output.status.success());

    // The alias command extracts aliases
    let alias_output = Command::new(pxl_binary())
        .arg("alias")
        .arg("tests/fixtures/valid/alias_test.jsonl")
        .output()
        .expect("Failed to execute pxl alias");

    assert!(alias_output.status.success());

    // Both should work on the same file without errors
    let inline_stdout = String::from_utf8_lossy(&inline_output.stdout);
    let alias_stdout = String::from_utf8_lossy(&alias_output.stdout);

    // Inline shows original tokens, alias shows transformed ones
    assert!(inline_stdout.contains("{_}") || inline_stdout.contains("{a}"));

    let alias_json: serde_json::Value = serde_json::from_str(&alias_stdout).unwrap();
    assert!(alias_json.get("aliases").is_some());
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_commands_with_empty_sprite_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let empty_file = temp_dir.path().join("empty.jsonl");
    fs::write(&empty_file, "").expect("Failed to write empty file");

    // All commands should handle empty files gracefully
    for cmd_name in &["show", "grid", "inline", "alias"] {
        let output = Command::new(pxl_binary())
            .arg(cmd_name)
            .arg(&empty_file)
            .output()
            .expect(&format!("Failed to execute pxl {}", cmd_name));

        assert!(
            !output.status.success(),
            "pxl {} should fail on empty file",
            cmd_name
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No sprite") || stderr.contains("Error"),
            "pxl {} should report no sprites found",
            cmd_name
        );
    }
}

#[test]
fn test_commands_with_palette_only_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let palette_file = temp_dir.path().join("palette_only.jsonl");
    fs::write(
        &palette_file,
        r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}"##,
    )
    .expect("Failed to write palette file");

    // Commands that require sprites should fail
    for cmd_name in &["show", "grid", "inline", "alias"] {
        let output = Command::new(pxl_binary())
            .arg(cmd_name)
            .arg(&palette_file)
            .output()
            .expect(&format!("Failed to execute pxl {}", cmd_name));

        assert!(
            !output.status.success(),
            "pxl {} should fail with palette-only file",
            cmd_name
        );
    }
}

#[test]
fn test_sketch_with_extra_whitespace() {
    let mut cmd = Command::new(pxl_binary())
        .arg("sketch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pxl sketch");

    // Input with various whitespace
    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"  a   b  \n\nc   d\n\n").unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should handle whitespace correctly (empty lines filtered out)
    let grid = json["grid"].as_array().unwrap();
    assert_eq!(grid.len(), 2, "Empty lines should be filtered out");
}

#[test]
fn test_show_large_sprite() {
    // Create a larger sprite to test display
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sprite_file = temp_dir.path().join("large.jsonl");

    // Create 16x16 sprite
    let mut grid_rows = Vec::new();
    for _ in 0..16 {
        let row: String = (0..16).map(|i| format!("{{t{}}}", i % 4)).collect();
        grid_rows.push(format!("\"{}\"", row));
    }

    let palette = r##"{"type": "palette", "name": "p", "colors": {"{t0}": "#FF0000", "{t1}": "#00FF00", "{t2}": "#0000FF", "{t3}": "#FFFF00"}}"##;
    let sprite = format!(
        r#"{{"type": "sprite", "name": "large", "palette": "p", "grid": [{}]}}"#,
        grid_rows.join(", ")
    );

    fs::write(&sprite_file, format!("{}\n{}\n", palette, sprite)).expect("Failed to write sprite");

    let output = Command::new(pxl_binary())
        .arg("show")
        .arg(&sprite_file)
        .output()
        .expect("Failed to execute pxl show");

    assert!(
        output.status.success(),
        "pxl show should handle larger sprites\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have many lines of output (at least 16 grid lines)
    assert!(
        stdout.lines().count() >= 16,
        "Large sprite should produce many output lines"
    );
}
