//! The region-level `transform` string is applied to the rasterized pixels:
//! integer translate, nearest-neighbour integer scale, and exact quarter-turn
//! rotation about the region's top-left pivot. Any other angle is refused by
//! `pxl validate` and `pxl render` instead of being resampled (aegis-1agh.1).
use std::collections::BTreeSet;
use std::process::{Command, Output};

const PALETTE: &str = r##"{type:"palette",name:"p",colors:{_:"transparent",a:"#ff0000"}}"##;
const RED: [u8; 4] = [255, 0, 0, 255];

fn sprite(region: &str) -> String {
    format!(
        "{PALETTE}\n{{type:\"sprite\",name:\"s\",size:[8,8],palette:\"p\",regions:{{a:{{{region},z:0}}}}}}\n"
    )
}

fn run(source: &str, args: &[&str]) -> (Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.pxl");
    std::fs::write(&input, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_pxl"));
    command.args(args).arg(&input);
    if args[0] == "render" {
        command.arg("-o").arg(dir.path().join("out.png"));
    }
    (command.output().unwrap(), dir)
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned() + &String::from_utf8_lossy(&output.stderr)
}

/// Render under `--strict` and return the golden pixel set of red pixels.
fn red_pixels(region: &str) -> BTreeSet<(u32, u32)> {
    let (output, dir) = run(&sprite(region), &["render", "--strict"]);
    assert!(output.status.success(), "{}", text(&output));
    let image = image::open(dir.path().join("out.png")).unwrap().to_rgba8();
    for p in image.pixels() {
        assert!(p.0 == RED || p.0[3] == 0, "only exact red or transparent pixels: {:?}", p.0);
    }
    image.enumerate_pixels().filter(|(_, _, p)| p.0 == RED).map(|(x, y, _)| (x, y)).collect()
}

fn set(pixels: &[(u32, u32)]) -> BTreeSet<(u32, u32)> {
    pixels.iter().copied().collect()
}

/// An L of three pixels whose top-left pivot is (4, 4).
const L: &str = "points:[[4,4],[4,5],[5,5]]";

#[test]
fn translate_moves_by_whole_pixels() {
    assert_eq!(red_pixels("rect:[0,0,2,1],transform:\"translate(3, 2)\""), set(&[(3, 2), (4, 2)]));
    assert_eq!(red_pixels("rect:[0,0,1,1],transform:\"translate(5)\""), set(&[(5, 0)]));
}

#[test]
fn scale_is_nearest_neighbour_about_the_pivot() {
    assert_eq!(
        red_pixels("rect:[1,1,2,1],transform:\"scale(2)\""),
        set(&[(1, 1), (2, 1), (3, 1), (4, 1), (1, 2), (2, 2), (3, 2), (4, 2)])
    );
    assert_eq!(
        red_pixels("rect:[2,1,1,1],transform:\"scale(1, 3)\""),
        set(&[(2, 1), (2, 2), (2, 3)])
    );
}

#[test]
fn rotate_quarter_turns_clockwise_about_the_pivot() {
    let r90 = set(&[(4, 4), (3, 4), (3, 5)]);
    let r180 = set(&[(4, 4), (4, 3), (3, 3)]);
    let r270 = set(&[(4, 4), (5, 4), (5, 3)]);
    assert_eq!(red_pixels(&format!("{L},transform:\"rotate(90deg)\"")), r90);
    assert_eq!(red_pixels(&format!("{L},transform:\"rotate(180deg)\"")), r180);
    assert_eq!(red_pixels(&format!("{L},transform:\"rotate(270deg)\"")), r270);
    assert_eq!(red_pixels(&format!("{L},transform:\"rotate(-90deg)\"")), r270);
    // Radians within 1e-9 of k*pi/2 count.
    assert_eq!(red_pixels(&format!("{L},transform:\"rotate(1.5707963267948966rad)\"")), r90);
    assert_eq!(red_pixels(&format!("{L},transform:\"rotate(3.141592653589793rad)\"")), r180);
    assert_eq!(
        red_pixels(&format!("{L},transform:\"rotate(0deg)\"")),
        set(&[(4, 4), (4, 5), (5, 5)])
    );
}

#[test]
fn composed_string_applies_left_to_right_and_order_matters() {
    let line = "points:[[2,0],[2,1],[2,2]]";
    assert_eq!(
        red_pixels(&format!("{line},transform:\"rotate(90deg) translate(2,0)\"")),
        set(&[(2, 0), (3, 0), (4, 0)])
    );
    assert_eq!(
        red_pixels(&format!("{line},transform:\"translate(2,0) rotate(90deg)\"")),
        set(&[(0, 2), (1, 2), (2, 2)])
    );
    assert_eq!(
        red_pixels("rect:[0,0,2,1],transform:\"scale(2) rotate(90deg) translate(1, 0)\""),
        set(&[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2), (0, 3), (1, 3)])
    );
}

#[test]
fn rotate_45deg_is_refused_by_validate_and_render() {
    let source = sprite(&format!("{L},transform:\"rotate(45deg)\""));
    for args in [&["validate", "--strict"][..], &["validate"][..]] {
        let (output, _dir) = run(&source, args);
        assert!(!output.status.success(), "validate must fail: {}", text(&output));
        assert!(text(&output).contains("multiples of 90deg"), "{}", text(&output));
    }
    for args in [&["render", "--strict"][..], &["render"][..]] {
        let (output, dir) = run(&source, args);
        assert!(!output.status.success(), "render must fail: {}", text(&output));
        assert!(text(&output).contains("multiples of 90deg"), "{}", text(&output));
        assert!(!dir.path().join("out.png").exists(), "no resampled output is written");
    }
}

#[test]
fn an_invalid_transform_on_a_nested_region_is_refused() {
    let source = sprite("union:[{rect:[0,0,2,2],transform:\"rotate(30deg)\"}]");
    let (output, _dir) = run(&source, &["validate"]);
    assert!(!output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("Region a has an invalid transform"), "{}", text(&output));
    let (output, _dir) = run(&source, &["render"]);
    assert!(!output.status.success(), "{}", text(&output));
}

#[test]
fn non_integer_scale_and_offset_are_refused() {
    for transform in ["scale(1.5)", "scale(0)", "translate(0.5, 0)", "rotate(1rad)", "skew(5deg)"] {
        let source = sprite(&format!("{L},transform:\"{transform}\""));
        let (output, _dir) = run(&source, &["validate", "--strict"]);
        assert!(!output.status.success(), "{transform}: {}", text(&output));
        let (output, _dir) = run(&source, &["render"]);
        assert!(!output.status.success(), "{transform}: {}", text(&output));
    }
}

#[test]
fn a_valid_transform_is_no_longer_reported_as_ignored() {
    let (output, _dir) =
        run(&sprite(&format!("{L},transform:\"rotate(90deg)\"")), &["validate", "--strict"]);
    assert!(output.status.success(), "{}", text(&output));
    assert!(!text(&output).contains("ignored"), "{}", text(&output));
}
