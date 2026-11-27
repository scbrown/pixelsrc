//! Region tiling, jitter, `round` and `x`/`y` range modifiers take effect, and
//! modifiers that still have no effect are reported instead of being dropped
//! silently (GH #15).
use std::process::{Command, Output};

const PALETTE: &str =
    r##"{type:"palette",name:"p",colors:{_:"transparent",a:"#ff0000",b:"#0000ff"}}"##;

fn sprite(size: [u32; 2], regions: &str) -> String {
    format!(
        "{PALETTE}\n{{type:\"sprite\",name:\"s\",size:[{},{}],palette:\"p\",regions:{{{regions}}}}}\n",
        size[0], size[1]
    )
}

fn run(source: &str, args: &[&str]) -> (Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.pxl");
    std::fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(args)
        .arg(&input)
        .arg("-o")
        .arg(dir.path().join("out.png"))
        .output()
        .unwrap();
    (output, dir)
}

fn render(source: &str) -> image::RgbaImage {
    let (output, dir) = run(source, &["render", "--strict"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    image::open(dir.path().join("out.png")).unwrap().to_rgba8()
}

fn opaque(image: &image::RgbaImage, color: [u8; 4]) -> Vec<(u32, u32)> {
    image.enumerate_pixels().filter(|(_, _, p)| p.0 == color).map(|(x, y, _)| (x, y)).collect()
}

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

/// Strict validation: (succeeded, combined stdout and stderr).
fn validate(source: &str) -> (bool, String) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.pxl");
    std::fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(["validate", "--strict"])
        .arg(input)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    (output.status.success(), text)
}

#[test]
fn repeat_with_spacing_tiles_the_shape() {
    // The GH #15 owner fixture: a 3x2 brick, 4 across and 2 down, 1px mortar.
    let image = render(&sprite([16, 8], "a:{rect:[0,0,3,2],repeat:[4,2],spacing:[1,1],z:0}"));
    let red = opaque(&image, RED);
    assert_eq!(red.len(), 8 * 6, "eight bricks of six pixels");
    for (column, row) in [(0, 0), (3, 0), (0, 1), (3, 1)] {
        let (x, y) = (column * 4, row * 3);
        assert!(red.contains(&(x, y)) && red.contains(&(x + 2, y + 1)), "brick at {x},{y}");
    }
    assert!(!red.contains(&(3, 0)), "mortar column stays empty");
    assert!(!red.contains(&(0, 2)), "mortar row stays empty");
}

#[test]
fn offset_alternate_shifts_odd_rows_by_half_a_step() {
    let image = render(&sprite(
        [16, 8],
        "a:{rect:[0,0,3,2],repeat:[2,2],spacing:[1,1],\"offset-alternate\":true,z:0}",
    ));
    let red = opaque(&image, RED);
    // Step is 4 across, so the second row starts 2 pixels right.
    assert!(red.contains(&(0, 0)) && red.contains(&(4, 0)));
    assert!(!red.contains(&(0, 3)) && !red.contains(&(1, 3)));
    assert!(red.contains(&(2, 3)) && red.contains(&(6, 3)));
}

#[test]
fn jitter_moves_each_point_within_its_range_deterministically() {
    let source =
        sprite([16, 16], "a:{points:[[0,15],[4,15],[8,15],[12,15]],jitter:{y:[-3,0]},seed:42,z:0}");
    let first = opaque(&render(&source), RED);
    assert_eq!(first.len(), 4);
    for (x, y) in &first {
        assert!([0, 4, 8, 12].contains(x), "x is not jittered: {x}");
        assert!((12..=15).contains(y), "y within [-3, 0] of 15: {y}");
    }
    assert!(first.iter().any(|(_, y)| *y != 15), "seed 42 moves at least one point");
    assert_eq!(first, opaque(&render(&source), RED), "same seed, same output");
    let other =
        sprite([16, 16], "a:{points:[[0,15],[4,15],[8,15],[12,15]],jitter:{y:[-3,0]},seed:7,z:0}");
    assert_ne!(first, opaque(&render(&other), RED), "a different seed changes the result");
}

#[test]
fn jitter_moves_whole_tiles_of_a_repeat() {
    let image = render(&sprite(
        [16, 16],
        "a:{rect:[0,0,2,2],repeat:[3,1],spacing:[2,0],jitter:{y:[0,4]},seed:3,z:0}",
    ));
    let red = opaque(&image, RED);
    assert_eq!(red.len(), 12, "three intact 2x2 tiles");
    for column in 0..3u32 {
        let tile: Vec<_> = red.iter().filter(|(x, _)| x / 4 == column).collect();
        assert_eq!(tile.len(), 4, "tile {column} stays whole");
        let top = tile.iter().map(|(_, y)| *y).min().unwrap();
        assert!(top <= 4, "tile {column} offset within range");
        assert!(tile.iter().all(|(_, y)| *y == top || *y == top + 1));
    }
}

#[test]
fn unsupported_region_fields_warn_and_fail_strict_validation() {
    let source = sprite(
        [8, 8],
        "a:{rect:[0,0,8,8],spacing:[1,1],z:0},b:{union:[{rect:[0,0,2,2],spacing:[1,1]}],z:1}",
    );
    let (ok, stdout) = validate(&source);
    assert!(!ok, "strict validation fails: {stdout}");
    assert!(stdout.contains("Region a uses spacing which is not supported"), "{stdout}");
    assert!(stdout.contains("Region b uses spacing"), "nested field reported: {stdout}");

    let (output, _dir) = run(&source, &["render", "--strict"]);
    assert!(!output.status.success(), "strict render refuses ignored fields");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Region field 'spacing' is not supported and was ignored"), "{stderr}");
}

/// The red pixels of `image` as rows of `#` (painted) and `.` (not painted).
fn grid(image: &image::RgbaImage) -> String {
    let mut out = String::new();
    for y in 0..image.height() {
        for x in 0..image.width() {
            out.push(if image.get_pixel(x, y).0 == RED { '#' } else { '.' });
        }
        out.push('\n');
    }
    out
}

fn golden(rows: &[&str]) -> String {
    rows.iter().map(|row| format!("{row}\n")).collect()
}

#[test]
fn round_cuts_the_corners_of_a_rect() {
    let image = render(&sprite([8, 6], "a:{rect:[0,0,8,6],round:2,z:0}"));
    assert_eq!(
        grid(&image),
        golden(&[".######.", "########", "########", "########", "########", ".######."])
    );
}

#[test]
fn round_larger_than_half_the_side_clamps() {
    // min(99, 8/2, 8/2) = 4.
    let clamped = render(&sprite([8, 8], "a:{rect:[0,0,8,8],round:99,z:0}"));
    let expected = golden(&[
        "..####..", ".######.", "########", "########", "########", "########", ".######.",
        "..####..",
    ]);
    assert_eq!(grid(&clamped), expected);
    let exact = render(&sprite([8, 8], "a:{rect:[0,0,8,8],round:4,z:0}"));
    assert_eq!(grid(&exact), expected, "round:99 renders as round:4");
    // On 8x6 the radius clamps to 3.
    let short = render(&sprite([8, 6], "a:{rect:[0,0,8,6],round:99,z:0}"));
    let three = render(&sprite([8, 6], "a:{rect:[0,0,8,6],round:3,z:0}"));
    assert_eq!(grid(&short), grid(&three));
}

#[test]
fn round_on_a_stroke_rounds_the_outline() {
    let image = render(&sprite([8, 8], "a:{stroke:[0,0,8,8],round:4,z:0}"));
    assert_eq!(
        grid(&image),
        golden(&[
            "..####..", ".#....#.", "#......#", "#......#", "#......#", "#......#", ".#....#.",
            "..####..",
        ])
    );
    // Thickness is unchanged: straight edges stay two pixels wide.
    let thick = render(&sprite([8, 8], "a:{stroke:[0,0,8,8],round:2,thickness:2,z:0}"));
    assert_eq!(
        grid(&thick),
        golden(&[
            ".######.", "########", "##....##", "##....##", "##....##", "##....##", "########",
            ".######.",
        ])
    );
}

#[test]
fn x_range_limits_the_columns() {
    let image = render(&sprite([8, 4], "a:{rect:[0,0,8,4],x:[2,5],z:0}"));
    assert_eq!(grid(&image), golden(&["..####..", "..####..", "..####..", "..####.."]));
}

#[test]
fn y_range_limits_the_rows() {
    let image = render(&sprite([8, 4], "a:{rect:[0,0,8,4],y:[1,2],z:0}"));
    assert_eq!(grid(&image), golden(&["........", "########", "########", "........"]));
}

#[test]
fn x_and_y_ranges_limit_together() {
    let image = render(&sprite([8, 4], "a:{rect:[0,0,8,4],x:[2,5],y:[1,2],z:0}"));
    assert_eq!(grid(&image), golden(&["........", "..####..", "..####..", "........"]));
}

#[test]
fn range_limits_clamp_to_the_canvas_and_apply_after_repeat() {
    // A bound past the canvas edge clamps instead of erroring.
    let image = render(&sprite([8, 2], "a:{rect:[0,0,8,2],x:[6,100],z:0}"));
    assert_eq!(grid(&image), golden(&["......##", "......##"]));
    // The limit clips the tiled result, not the shape before tiling.
    let tiled = render(&sprite([8, 1], "a:{rect:[0,0,1,1],repeat:[8,1],x:[3,4],z:0}"));
    assert_eq!(grid(&tiled), golden(&["...##..."]));
}

#[test]
fn spacing_without_repeat_is_reported() {
    let (ok, stdout) =
        validate(&sprite([8, 8], "a:{rect:[0,0,2,2],spacing:[1,1],z:0},b:{rect:[4,4,1,1],z:0}"));
    assert!(!ok, "{stdout}");
    assert!(stdout.contains("Region a uses spacing"), "{stdout}");
}

#[test]
fn supported_modifiers_still_validate_cleanly() {
    // 18 wide: the offset second row ends at x=17, so nothing is clipped.
    let (ok, text) = validate(&sprite(
        [18, 8],
        "a:{rect:[0,0,3,2],repeat:[4,2],spacing:[1,1],\"offset-alternate\":true,z:0},b:{points:[[0,7]],jitter:{y:[-3,0]},seed:4,translate:[1,0],z:1}",
    ));
    assert!(ok, "{text}");
    let _ = BLUE;
}
