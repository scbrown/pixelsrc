//! Composition transforms must agree with direct sprite rendering in PNG and GIF.

use image::AnimationDecoder;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

const INPUT: &str = "tests/fixtures/valid/composition_transforms.pxl";

fn render(output: &Path, options: &[&str]) {
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(["render", INPUT, "--strict"])
        .args(options)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}

fn expected(x: u32, y: u32) -> image::RgbaImage {
    let mut image = image::RgbaImage::new(8, 8);
    for dy in 0..2 {
        for dx in 0..3 {
            image.put_pixel(x + dx, y + dy, image::Rgba([227, 218, 201, 255]));
        }
    }
    image
}

#[test]
fn png_compositions_match_direct_transformed_sprites() {
    let dir = tempfile::tempdir().unwrap();
    render(dir.path(), &[]);
    let shifted = image::open(dir.path().join("jaw_shift.png")).unwrap().to_rgba8();
    assert_eq!(shifted, expected(2, 4));
    for name in ["layer_shift", "source_shift", "base_shift"] {
        let actual = image::open(dir.path().join(format!("{name}.png"))).unwrap().to_rgba8();
        assert_eq!(actual, shifted, "{name}");
    }
    let combined = image::open(dir.path().join("combined.png")).unwrap().to_rgba8();
    assert_eq!(combined, expected(3, 4));
    assert_eq!(combined, image::open(dir.path().join("jaw_combined.png")).unwrap().to_rgba8());
}

#[test]
fn gif_composition_frames_apply_source_then_layer_transforms() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("poses.gif");
    render(&path, &["--gif", "--animation", "poses"]);
    let decoder =
        image::codecs::gif::GifDecoder::new(BufReader::new(File::open(path).unwrap())).unwrap();
    let frames = decoder.into_frames().collect_frames().unwrap();
    assert_eq!(frames.len(), 3);
    for (frame, x) in frames.iter().zip([2, 2, 3]) {
        assert_eq!(frame.buffer(), &expected(x, 4));
    }
}
