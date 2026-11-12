//! Nested references share validated sprite rendering and a cycle/cache context.
use std::process::Command;

#[test]
fn nested_composition_renders_every_tile_without_sprite_warnings() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("scene.png");
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args([
            "render",
            "tests/fixtures/valid/nested_structured.pxl",
            "-c",
            "scene",
            "--strict",
            "-o",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(result.stderr.is_empty());
    let actual = image::open(output).unwrap().to_rgba8();
    let mut expected = image::RgbaImage::new(8, 8);
    for y in [0, 4] {
        for x in [0, 4] {
            for dy in 0..2 {
                for dx in 0..2 {
                    expected.put_pixel(x + dx, y + dy, image::Rgba([255, 0, 0, 255]));
                }
            }
        }
    }
    assert_eq!(actual, expected);
}

#[test]
fn nested_composition_cycle_is_an_error_even_when_lenient() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("cycle.pxl");
    std::fs::write(&input, r#"{ type: "composition", name: "a", size: [1,1], sprites: { b: "b" }, layers: [{ map: ["b"] }] }
{ type: "composition", name: "b", size: [1,1], sprites: { a: "a" }, layers: [{ map: ["a"] }] }"#).unwrap();
    let output = dir.path().join("cycle.png");
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["-c", "a", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr).to_lowercase();
    assert!(stderr.contains("cycl"), "{stderr}");
    assert!(!output.exists());
}

#[test]
fn strict_flat_size_validation_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("oversized.pxl");
    let fixture = std::fs::read_to_string("tests/fixtures/valid/nested_structured.pxl").unwrap();
    // The pair's 4x4 sprites cannot fit 1x1 cells.
    std::fs::write(&input, fixture.replace("cell_size: [4, 4]", "cell_size: [1, 1]")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["-c", "pair", "--strict", "-o"])
        .arg(dir.path().join("oversized.png"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!dir.path().join("oversized.png").exists());
}

#[test]
fn nested_composition_can_be_a_base_and_an_animation_frame() {
    use image::AnimationDecoder;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("animation.pxl");
    let fixture = std::fs::read_to_string("tests/fixtures/valid/nested_structured.pxl").unwrap();
    std::fs::write(&input, format!("{fixture}\n{{ type: 'composition', name: 'copy', base: 'scene', sprites: {{}}, layers: [] }}\n{{ type: 'animation', name: 'poses', frames: ['copy', 'scene'], duration: 100 }}\n")).unwrap();
    let output = dir.path().join("poses.gif");
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["--gif", "--animation", "poses", "--strict", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let decoder = image::codecs::gif::GifDecoder::new(std::io::BufReader::new(
        std::fs::File::open(output).unwrap(),
    ))
    .unwrap();
    let frames = decoder.into_frames().collect_frames().unwrap();
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].buffer(), frames[1].buffer());
    assert!(frames[0].buffer().pixels().any(|p| p.0 == [255, 0, 0, 255]));
}

#[test]
fn sprite_keeps_precedence_over_composition_with_same_name() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("precedence.pxl");
    let fixture = std::fs::read_to_string("tests/fixtures/valid/nested_structured.pxl").unwrap();
    std::fs::write(&input, format!("{fixture}\n{{ type: 'sprite', name: 'pair', size: [8,4], palette: 'p', regions: {{ a: {{ rect: [0,0,8,4] }} }} }}\n")).unwrap();
    let output = dir.path().join("scene.png");
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["-c", "scene", "--strict", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let image = image::open(output).unwrap().to_rgba8();
    assert!(image.pixels().all(|p| p.0 == [255, 0, 0, 255]));
}
