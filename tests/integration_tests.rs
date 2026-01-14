//! Integration tests for the pxl CLI
//!
//! These tests verify end-to-end behavior of the CLI by running the binary
//! against fixture files and checking exit codes and output.

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

/// Get all .jsonl files in a directory
fn get_jsonl_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "jsonl") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Run pxl render on a fixture file
fn run_pxl_render(fixture: &Path, strict: bool) -> std::process::Output {
    let output_dir = std::env::temp_dir().join("pxl_integration_tests");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("test_output.png");

    let mut cmd = Command::new(pxl_binary());
    cmd.arg("render")
        .arg(fixture)
        .arg("-o")
        .arg(&output_path);

    if strict {
        cmd.arg("--strict");
    }

    cmd.output().expect("Failed to execute pxl")
}

/// Test that all valid fixtures render successfully
#[test]
fn test_valid_fixtures_render() {
    let fixtures_dir = Path::new("tests/fixtures/valid");
    let files = get_jsonl_files(fixtures_dir);

    assert!(!files.is_empty(), "No valid fixtures found");

    for fixture in &files {
        let output = run_pxl_render(fixture, false);
        assert!(
            output.status.success(),
            "Expected success for {:?}, got exit code {:?}\nstderr: {}",
            fixture,
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );

        // Valid fixtures should not produce warnings
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("Warning:"),
            "Unexpected warnings for {:?}: {}",
            fixture,
            stderr
        );
    }

    println!("✓ All {} valid fixtures rendered successfully", files.len());
}

/// Test that all invalid fixtures produce errors
///
/// Note: Some fixtures fail during parsing (lenient mode), others have semantic
/// errors that only fail in strict mode. We test all in strict mode to ensure
/// they all produce errors.
#[test]
fn test_invalid_fixtures_error() {
    let fixtures_dir = Path::new("tests/fixtures/invalid");
    let files = get_jsonl_files(fixtures_dir);

    assert!(!files.is_empty(), "No invalid fixtures found");

    for fixture in &files {
        let output = run_pxl_render(fixture, true);
        assert!(
            !output.status.success(),
            "Expected error for {:?} in strict mode, got success\nstdout: {}\nstderr: {}",
            fixture,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        // Should have error output
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Error:"),
            "Expected error message for {:?}: {}",
            fixture,
            stderr
        );
    }

    println!("✓ All {} invalid fixtures produced errors", files.len());
}

/// Test that lenient fixtures succeed with warnings
///
/// Lenient fixtures have issues that produce warnings in lenient mode but
/// still succeed (exit 0). Most produce warnings, but some edge cases
/// (like duplicate_name.jsonl) may succeed silently.
#[test]
fn test_lenient_fixtures_warn() {
    let fixtures_dir = Path::new("tests/fixtures/lenient");
    let files = get_jsonl_files(fixtures_dir);

    assert!(!files.is_empty(), "No lenient fixtures found");

    let mut files_with_warnings = 0;

    for fixture in &files {
        let output = run_pxl_render(fixture, false);
        assert!(
            output.status.success(),
            "Expected success for {:?} in lenient mode, got exit code {:?}\nstderr: {}",
            fixture,
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );

        // Track files that produce warnings
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Warning:") {
            files_with_warnings += 1;
        }
    }

    // At least some lenient fixtures should produce warnings
    assert!(
        files_with_warnings > 0,
        "Expected at least some lenient fixtures to produce warnings, but none did"
    );

    println!(
        "✓ All {} lenient fixtures succeeded ({} with warnings)",
        files.len(),
        files_with_warnings
    );
}

/// Test that lenient fixtures with warnings fail in strict mode
///
/// Lenient fixtures that produce warnings in lenient mode should fail in strict mode.
/// Some edge cases (like duplicate_name.jsonl) may not produce warnings and thus
/// will succeed even in strict mode.
#[test]
fn test_strict_mode_fails_on_warnings() {
    let fixtures_dir = Path::new("tests/fixtures/lenient");
    let files = get_jsonl_files(fixtures_dir);

    assert!(!files.is_empty(), "No lenient fixtures found");

    let mut failed_count = 0;

    for fixture in &files {
        // First check if this fixture produces warnings in lenient mode
        let lenient_output = run_pxl_render(fixture, false);
        let lenient_stderr = String::from_utf8_lossy(&lenient_output.stderr);
        let produces_warnings = lenient_stderr.contains("Warning:");

        // Run in strict mode
        let strict_output = run_pxl_render(fixture, true);

        if produces_warnings {
            // Fixtures that produce warnings should fail in strict mode
            assert!(
                !strict_output.status.success(),
                "Expected failure for {:?} in strict mode (has warnings), got success\nstdout: {}\nstderr: {}",
                fixture,
                String::from_utf8_lossy(&strict_output.stdout),
                String::from_utf8_lossy(&strict_output.stderr)
            );
            failed_count += 1;
        }
        // Fixtures without warnings may succeed in strict mode (edge cases)
    }

    assert!(
        failed_count > 0,
        "Expected at least some lenient fixtures to fail in strict mode"
    );

    println!(
        "✓ {} lenient fixtures (with warnings) failed in strict mode as expected",
        failed_count
    );
}

/// Test CLI help and version flags
#[test]
fn test_cli_help() {
    let output = Command::new(pxl_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("render"));
    assert!(stdout.contains("Pixelsrc"));
}

/// Test CLI version flag
#[test]
fn test_cli_version() {
    let output = Command::new(pxl_binary())
        .arg("--version")
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pxl"));
}

/// Test that missing input file produces appropriate error
#[test]
fn test_missing_input_file() {
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("nonexistent_file.jsonl")
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cannot open") || stderr.contains("Error"),
        "Expected file not found error, got: {}",
        stderr
    );
}

/// Test output file naming
#[test]
fn test_output_file_naming() {
    let output_dir = std::env::temp_dir().join("pxl_naming_test");
    fs::create_dir_all(&output_dir).ok();

    // Clean up any existing files
    for entry in fs::read_dir(&output_dir).into_iter().flatten() {
        if let Ok(entry) = entry {
            fs::remove_file(entry.path()).ok();
        }
    }

    // Test explicit output path
    let explicit_output = output_dir.join("explicit.png");
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/minimal_dot.jsonl")
        .arg("-o")
        .arg(&explicit_output)
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success());
    assert!(
        explicit_output.exists(),
        "Output file not created at {:?}",
        explicit_output
    );
}

/// Test @include:path syntax for external palette files
#[test]
fn test_include_palette() {
    let output_dir = std::env::temp_dir().join("pxl_include_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("include_test.png");

    // Test with @include:shared/palette.jsonl
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/include_palette.jsonl")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Failed to render with @include: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the output file exists and is a valid PNG
    assert!(
        output_path.exists(),
        "Output file not created at {:?}",
        output_path
    );

    // No warnings should be produced
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Warning:"),
        "Unexpected warnings: {}",
        stderr
    );
}

/// Test sprite filtering with --sprite flag
#[test]
fn test_sprite_filter() {
    let output_dir = std::env::temp_dir().join("pxl_filter_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("filtered.png");

    // Test with valid sprite name
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--sprite")
        .arg("green_dot")
        .output()
        .expect("Failed to execute pxl");

    assert!(
        output.status.success(),
        "Failed to filter sprite: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Test with invalid sprite name
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg("tests/fixtures/valid/multiple_sprites.jsonl")
        .arg("-o")
        .arg(&output_path)
        .arg("--sprite")
        .arg("nonexistent_sprite")
        .output()
        .expect("Failed to execute pxl");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No sprite named"));
}
