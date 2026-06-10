//! CLI integration tests for particle baking (`pxl render --frames`).
//!
//! The particle engine is seeded and deterministic; these tests exercise the
//! CLI plumbing that bakes a particle system to a PNG frame-set or a GIF, and
//! confirm the output is reproducible run-to-run.

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

const PARTICLE_PXL: &str = r##"{ type: "palette", name: "ember_pal", colors: { _: "transparent", ember: "#FFB23E" } }
{ type: "sprite", name: "ember", size: [2, 2], palette: "ember_pal", regions: { ember: { rect: [0, 0, 2, 2] } } }
{ type: "particle", name: "embers", sprite: "ember", emitter: { rate: 1.5, lifetime: [6, 10], velocity: { x: [-0.4, 0.4], y: [-1.2, -0.6] }, gravity: -0.02, fade: true, seed: 7 } }
"##;

fn write_input(dir: &Path) -> PathBuf {
    let path = dir.join("embers.pxl");
    fs::write(&path, PARTICLE_PXL).unwrap();
    path
}

fn run_render(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(pxl_binary())
        .arg("render")
        .args(args)
        .output()
        .expect("Failed to execute pxl");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

#[test]
fn test_particle_bake_png_frames() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path());
    let out = dir.path().join("frames/");
    fs::create_dir_all(&out).unwrap();

    let (_stdout, stderr, ok) = run_render(&[
        input.to_str().unwrap(),
        "--frames",
        "6",
        "--canvas",
        "16x16",
        "--origin",
        "8,15",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "particle bake should succeed; stderr: {}", stderr);

    // Six numbered PNGs, zero-padded to the frame-set convention.
    for n in 0..6 {
        let frame = out.join(format!("embers_{:02}.png", n));
        assert!(frame.exists(), "missing frame {}", frame.display());
    }
}

#[test]
fn test_particle_bake_is_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path());

    let bake = |sub: &str| -> Vec<u8> {
        let out = dir.path().join(sub);
        fs::create_dir_all(&out).unwrap();
        let (_o, stderr, ok) = run_render(&[
            input.to_str().unwrap(),
            "--frames",
            "5",
            "--canvas",
            "16x16",
            "--origin",
            "8,15",
            "-o",
            out.to_str().unwrap(),
        ]);
        assert!(ok, "bake failed; stderr: {}", stderr);
        fs::read(out.join("embers_03.png")).unwrap()
    };

    assert_eq!(bake("a/"), bake("b/"), "same seed must produce identical frames");
}

#[test]
fn test_particle_bake_gif() {
    let dir = tempfile::tempdir().unwrap();
    let input = write_input(dir.path());
    let gif = dir.path().join("embers.gif");

    let (stdout, stderr, ok) = run_render(&[
        input.to_str().unwrap(),
        "--frames",
        "12",
        "--canvas",
        "16x16",
        "--gif",
        "-o",
        gif.to_str().unwrap(),
    ]);
    assert!(ok, "gif bake should succeed; stderr: {}", stderr);
    assert!(gif.exists(), "gif not written");
    assert!(stdout.contains("12 frames"), "should report frame count");
}

#[test]
fn test_particle_bake_no_particle_errors() {
    let dir = tempfile::tempdir().unwrap();
    // A sprite-only file: --frames has nothing to bake.
    let input = dir.path().join("plain.pxl");
    fs::write(
        &input,
        r##"{ type: "sprite", name: "dot", size: [2, 2], palette: {_: "#0000", x: "#FFF"}, regions: { x: { rect: [0,0,2,2] } } }"##,
    )
    .unwrap();

    let (_stdout, stderr, ok) =
        run_render(&[input.to_str().unwrap(), "--frames", "4"]);
    assert!(!ok, "should fail when no particle is present");
    assert!(stderr.contains("no particle systems"), "stderr: {}", stderr);
}
