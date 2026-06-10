//! CLI integration tests for `pxl show` on structured-region sprites.
//!
//! `show` used to reject the structured format with a "grid format is
//! deprecated" error; it now renders an ANSI half-block color preview that
//! resolves `extends`/`source`/`translate` like `pxl render`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

const SPRITES: &str = r##"{ type: "palette", name: "p", colors: { _: "transparent", body: "#CC4444" } }
{ type: "sprite", name: "blob", size: [8, 8], palette: "p", regions: { body: { rect: [2, 2, 4, 4], z: 0 } } }
{ type: "sprite", name: "blob_bob", extends: "blob", translate: { by: [0, -1], regions: ["body"] } }
"##;

fn write_input(dir: &Path) -> PathBuf {
    let path = dir.join("blob.pxl");
    fs::write(&path, SPRITES).unwrap();
    path
}

fn run_show(args: &[&str]) -> (String, String, bool) {
    let output =
        Command::new(pxl_binary()).arg("show").args(args).output().expect("Failed to execute pxl");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

#[test]
fn test_show_structured_sprite_renders() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path());
    let (stdout, stderr, ok) = run_show(&[input.to_str().unwrap(), "--sprite", "blob"]);
    assert!(ok, "show should render a structured sprite; stderr: {}", stderr);
    assert!(stdout.contains("Sprite: blob (8x8)"), "should print the sprite header: {}", stdout);
    // ANSI half-block output uses escape sequences.
    assert!(stdout.contains('\u{1b}'), "should emit ANSI color output");
    assert!(!stderr.contains("deprecated"), "must not report the old deprecation error");
}

#[test]
fn test_show_resolves_extends_and_translate() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path());
    let (stdout, stderr, ok) = run_show(&[input.to_str().unwrap(), "--sprite", "blob_bob"]);
    assert!(ok, "show should resolve an extends+translate frame; stderr: {}", stderr);
    assert!(stdout.contains("Sprite: blob_bob (8x8)"), "size inherited from base: {}", stdout);
}
