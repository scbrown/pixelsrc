//! Tests for IMP-3: Project Context for pxl render
//!
//! Verifies that `pxl render` auto-detects `pxl.toml` in parent directories,
//! loads ProjectRegistry for cross-file reference resolution, and respects
//! the `--no-project` flag.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Create a project with pxl.toml, a palette file, and a sprite file that references it.
fn create_cross_file_project(temp: &TempDir) {
    let root = temp.path();

    // Create pxl.toml
    fs::write(root.join("pxl.toml"), "[project]\nname = \"test-project\"\nsrc = \"src/pxl\"\n")
        .unwrap();

    // Create src/pxl directory structure
    let src_dir = root.join("src/pxl");
    fs::create_dir_all(src_dir.join("palettes")).unwrap();
    fs::create_dir_all(src_dir.join("sprites")).unwrap();

    // Create a palette in palettes/mono.pxl
    fs::write(
        src_dir.join("palettes/mono.pxl"),
        r##"{"type": "palette", "name": "mono", "colors": {"{_}": "#000000", "{on}": "#FFFFFF"}}"##,
    )
    .unwrap();

    // Create a sprite in sprites/hero.pxl that references "mono" palette by name
    fs::write(
        src_dir.join("sprites/hero.pxl"),
        r##"{"type": "sprite", "name": "hero", "palette": "mono", "size": [2, 2], "regions": {"{on}": {"points": [[0,0],[1,1]]}, "{_}": {"points": [[1,0],[0,1]]}}}"##,
    )
    .unwrap();
}

#[test]
fn test_render_with_project_context_resolves_cross_file_palette() {
    let temp = TempDir::new().unwrap();
    create_cross_file_project(&temp);

    let sprite_file = temp.path().join("src/pxl/sprites/hero.pxl");
    let output_file = temp.path().join("hero.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(&sprite_file)
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute pxl");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "pxl render should succeed with project context.\nstdout: {}\nstderr: {}",
        stdout,
        stderr,
    );
    assert!(output_file.exists(), "Output PNG should be created");
}

#[test]
fn test_render_no_project_flag_disables_project_context() {
    let temp = TempDir::new().unwrap();
    create_cross_file_project(&temp);

    let sprite_file = temp.path().join("src/pxl/sprites/hero.pxl");
    let output_file = temp.path().join("hero_noproject.png");

    // With --no-project, the sprite's "mono" palette reference won't resolve
    // from the project registry. It should still render (lenient mode) but
    // the palette won't be found cross-file.
    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(&sprite_file)
        .arg("--no-project")
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute pxl");

    // In lenient mode, missing palette produces a warning but still renders
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "pxl render --no-project should succeed in lenient mode.\nstderr: {}",
        stderr,
    );
}

#[test]
fn test_render_standalone_file_without_pxl_toml() {
    let temp = TempDir::new().unwrap();

    // Create a standalone file with inline palette (no pxl.toml anywhere above)
    let sprite_file = temp.path().join("standalone.pxl");
    fs::write(
        &sprite_file,
        r##"{"type": "sprite", "name": "dot", "palette": {"{on}": "#FF0000"}, "size": [1, 1], "pixels": "{on}"}"##,
    )
    .unwrap();

    let output_file = temp.path().join("dot.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(&sprite_file)
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute pxl");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "Standalone render should succeed without pxl.toml.\nstdout: {}\nstderr: {}",
        stdout,
        stderr,
    );
    assert!(output_file.exists(), "Output PNG should be created");
}

#[test]
fn test_render_strict_with_project_context() {
    let temp = TempDir::new().unwrap();
    create_cross_file_project(&temp);

    let sprite_file = temp.path().join("src/pxl/sprites/hero.pxl");
    let output_file = temp.path().join("hero_strict.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(&sprite_file)
        .arg("--strict")
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute pxl");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "Strict render should succeed when project context resolves palette.\nstdout: {}\nstderr: {}",
        stdout,
        stderr,
    );
    assert!(output_file.exists(), "Output PNG should be created");
}

#[test]
fn test_render_with_project_context_local_palette_still_works() {
    let temp = TempDir::new().unwrap();

    // Create pxl.toml (project exists but palette is inline in the file)
    fs::write(
        temp.path().join("pxl.toml"),
        "[project]\nname = \"test-project\"\nsrc = \"src/pxl\"\n",
    )
    .unwrap();

    let src_dir = temp.path().join("src/pxl");
    fs::create_dir_all(&src_dir).unwrap();

    // Sprite with inline palette should work regardless of project context
    fs::write(
        src_dir.join("inline.pxl"),
        concat!(
            r##"{"type": "palette", "name": "local", "colors": {"{x}": "#FF0000"}}"##,
            "\n",
            r##"{"type": "sprite", "name": "pixel", "palette": "local", "size": [1, 1], "pixels": "{x}"}"##,
        ),
    )
    .unwrap();

    let output_file = temp.path().join("pixel.png");

    let output = Command::new(pxl_binary())
        .arg("render")
        .arg(src_dir.join("inline.pxl"))
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to execute pxl");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "Inline palette should work with project context.\nstdout: {}\nstderr: {}",
        stdout,
        stderr,
    );
    assert!(output_file.exists(), "Output PNG should be created");
}
