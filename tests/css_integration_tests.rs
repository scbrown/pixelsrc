//! CSS Integration Tests (CSS-18)
//!
//! End-to-end tests for CSS features:
//! - CSS-12: color-mix() support
//! - CSS-13: @keyframes animation model
//! - CSS-14: CSS transforms (translate, rotate, scale, flip)
//!
//! Tests verify that fixtures in tests/fixtures/css/ render successfully
//! through the CLI pipeline.

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

/// Run pxl render on a fixture file and return the output
fn run_pxl_render(fixture: &Path) -> std::process::Output {
    let output_dir = std::env::temp_dir().join("pxl_css_integration_tests");
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

/// Helper to assert a fixture renders successfully
fn assert_renders_successfully(fixture_path: &str) {
    let fixture = Path::new(fixture_path);
    assert!(fixture.exists(), "Fixture not found: {}", fixture_path);

    let output = run_pxl_render(fixture);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Failed to render {}: exit code {:?}\nstderr: {}",
        fixture_path,
        output.status.code(),
        stderr
    );

    assert!(!stderr.contains("Warning:"), "Unexpected warnings for {}: {}", fixture_path, stderr);
}

// ============================================================================
// CSS-12: color-mix() Tests
// ============================================================================

#[test]
fn test_color_mix_fixture_renders() {
    assert_renders_successfully("tests/fixtures/css/color_mix.jsonl");
}

#[test]
fn test_color_mix_produces_valid_output() {
    let fixture = Path::new("tests/fixtures/css/color_mix.jsonl");
    let output_dir = std::env::temp_dir().join("pxl_color_mix_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("color_mix.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(fixture)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    assert!(output.status.success());
    assert!(output_path.exists(), "Output PNG not created");

    // Verify output is valid PNG by checking file size
    let metadata = fs::metadata(&output_path).expect("Cannot read output file");
    assert!(metadata.len() > 0, "Output file is empty");
}

// ============================================================================
// CSS-13: @keyframes Animation Tests
// ============================================================================

#[test]
fn test_keyframes_fixture_renders() {
    assert_renders_successfully("tests/fixtures/css/keyframes.jsonl");
}

#[test]
fn test_keyframes_animation_output() {
    let fixture = Path::new("tests/fixtures/css/keyframes.jsonl");
    let output_dir = std::env::temp_dir().join("pxl_keyframes_test");
    fs::create_dir_all(&output_dir).ok();

    let output_path = output_dir.join("keyframes.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(fixture)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute pxl");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Keyframes render failed: {}", stderr);
}

// ============================================================================
// CSS-14: Transform Tests
// ============================================================================

#[test]
fn test_transforms_jsonl_fixture_renders() {
    assert_renders_successfully("tests/fixtures/css/transforms.jsonl");
}

#[test]
fn test_transforms_pxl_fixture_renders() {
    assert_renders_successfully("tests/fixtures/css/transforms.pxl");
}

#[test]
fn test_transforms_all_rotations() {
    // Verify all rotation variants render correctly
    let fixture = Path::new("tests/fixtures/css/transforms.jsonl");
    let output_dir = std::env::temp_dir().join("pxl_transforms_test");
    fs::create_dir_all(&output_dir).ok();

    // Render each sprite variant
    for sprite in
        &["arrow", "arrow_rot90", "arrow_rot180", "arrow_rot270", "arrow_flip_h", "arrow_flip_v"]
    {
        let output_path = output_dir.join(format!("{}.png", sprite));

        let output = Command::new(pxl_binary())
            .arg("render")
            .arg(fixture)
            .arg("-o")
            .arg(&output_path)
            .arg("--sprite")
            .arg(sprite)
            .output()
            .expect("Failed to execute pxl");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "Failed to render sprite '{}': {}", sprite, stderr);
    }
}

// ============================================================================
// Integrated CSS Features Tests
// ============================================================================

#[test]
fn test_integrated_css_fixture_renders() {
    assert_renders_successfully("tests/fixtures/css/integrated.jsonl");
}

#[test]
fn test_integrated_variables_and_color_mix() {
    // Tests that CSS variables work correctly with color-mix()
    let fixture = Path::new("tests/fixtures/css/integrated.jsonl");
    let output_dir = std::env::temp_dir().join("pxl_integrated_test");
    fs::create_dir_all(&output_dir).ok();

    // Render specific sprites that use the combined features
    for sprite in &["button_normal", "button_hover", "icon_base", "icon_muted", "panel_bg"] {
        let output_path = output_dir.join(format!("{}.png", sprite));

        let output = Command::new(pxl_binary())
            .arg("render")
            .arg(fixture)
            .arg("-o")
            .arg(&output_path)
            .arg("--sprite")
            .arg(sprite)
            .output()
            .expect("Failed to execute pxl");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "Failed to render integrated sprite '{}': {}",
            sprite,
            stderr
        );
    }
}

// ============================================================================
// All CSS Fixtures Pass Test
// ============================================================================

#[test]
fn test_all_css_fixtures_render() {
    let css_dir = Path::new("tests/fixtures/css");

    if !css_dir.exists() {
        panic!("CSS fixtures directory not found: {:?}", css_dir);
    }

    let mut files_tested = 0;

    for entry in fs::read_dir(css_dir).expect("Cannot read CSS fixtures directory") {
        let entry = entry.expect("Cannot read directory entry");
        let path = entry.path();

        // Test both .jsonl and .pxl files
        if path.extension().map_or(false, |e| e == "jsonl" || e == "pxl") {
            let output = run_pxl_render(&path);
            let stderr = String::from_utf8_lossy(&output.stderr);

            assert!(output.status.success(), "CSS fixture {:?} failed to render: {}", path, stderr);

            assert!(
                !stderr.contains("Error:"),
                "CSS fixture {:?} produced errors: {}",
                path,
                stderr
            );

            files_tested += 1;
        }
    }

    assert!(files_tested >= 4, "Expected at least 4 CSS fixtures, found {}", files_tested);

    println!("All {} CSS fixtures in tests/fixtures/css/ rendered successfully", files_tested);
}

// ============================================================================
// Strict Mode Tests
// ============================================================================

#[test]
fn test_css_fixtures_pass_strict_mode() {
    let css_dir = Path::new("tests/fixtures/css");

    for entry in fs::read_dir(css_dir).expect("Cannot read CSS fixtures directory") {
        let entry = entry.expect("Cannot read directory entry");
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "jsonl" || e == "pxl") {
            let output_dir = std::env::temp_dir().join("pxl_css_strict_test");
            fs::create_dir_all(&output_dir).ok();

            let output_path = output_dir.join("strict_test.png");

            let output = Command::new(pxl_binary())
                .arg("render")
                .arg(&path)
                .arg("-o")
                .arg(&output_path)
                .arg("--strict")
                .output()
                .expect("Failed to execute pxl");

            let stderr = String::from_utf8_lossy(&output.stderr);

            assert!(
                output.status.success(),
                "CSS fixture {:?} failed in strict mode: {}",
                path,
                stderr
            );
        }
    }
}
