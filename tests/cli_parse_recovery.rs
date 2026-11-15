//! A malformed object must not hide later valid render targets.
use std::process::Command;

const INPUT: &str = r##"{ type: "palette", name: "p", colors: { _: "transparent", a: "#ff0000" } }
{ type: "sprite", name: "dot", size: [4,4], palette: "p", regions: { a: { rect: [0,0,4,4] } } }
{ type: "composition", name: "bad", size: [4,4], layers: [{ sprite: "dot", x: 0, y: 0 }] }
{ type: "composition", name: "good", size: [4,4], sprites: { d: "dot" }, layers: [{ map: ["d"] }] }
"##;

fn render(composition: &str, strict: bool) -> (std::process::Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("recovery.pxl");
    std::fs::write(&input, INPUT).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_pxl"));
    command
        .arg("render")
        .arg(input)
        .args(["-c", composition, "-o"])
        .arg(dir.path().join("result.png"));
    if strict {
        command.arg("--strict");
    }
    (command.output().unwrap(), dir)
}

#[test]
fn bad_object_does_not_hide_later_composition() {
    let (result, dir) = render("good", false);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(result.status.success(), "{stderr}");
    assert!(stderr.contains("Failed to parse composition") && stderr.contains("bad"), "{stderr}");
    assert!(stderr.contains("line 3") && stderr.contains("missing field `sprites`"), "{stderr}");
    let image = image::open(dir.path().join("result.png")).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (4, 4));
    assert!(image.pixels().all(|p| p.0 == [255, 0, 0, 255]));
}

#[test]
fn strict_still_refuses_parse_failure() {
    let (result, dir) = render("good", true);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("line 3"));
    assert!(!dir.path().join("result.png").exists());
}

#[test]
fn missing_target_error_includes_parse_failure() {
    let (result, _) = render("absent", false);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!result.status.success());
    assert!(stderr.contains("No composition named 'absent'"), "{stderr}");
    assert!(stderr.contains("line 3") && stderr.contains("missing field `sprites`"), "{stderr}");
}

#[test]
fn missing_sprite_error_includes_parse_failure() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("recovery.pxl");
    std::fs::write(&input, INPUT).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["--sprite", "absent"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!result.status.success());
    assert!(stderr.contains("No sprite named 'absent'"), "{stderr}");
    assert!(
        stderr.contains("line 3") && stderr.contains("Failed to parse composition"),
        "{stderr}"
    );
}
