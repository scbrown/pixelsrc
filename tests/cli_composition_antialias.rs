//! Composition AA runs on the composed bitmap before additional nearest scaling.
use std::process::Command;
const INPUT: &str = r##"{type:"palette",name:"p",colors:{bg:"#000000",edge:"#ffffff"}}
{type:"sprite",name:"diagonal",size:[4,4],palette:"p",regions:{bg:{rect:[0,0,4,4],z:0},edge:{points:[[0,0],[0,1],[1,1],[0,2],[1,2],[2,2]],z:1}}}
{type:"composition",name:"scene",size:[4,4],cell_size:[4,4],sprites:{D:"diagonal"},layers:[{map:["D"]}]}
{type:"animation",name:"an",frames:["scene"],duration:100}
"##;
fn render(args: &[&str], filtered: bool) -> image::RgbaImage {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("aa.pxl");
    std::fs::write(&input, INPUT).unwrap();
    let output = dir.path().join("out.png");
    let mut command = Command::new(env!("CARGO_BIN_EXE_pxl"));
    command.arg("render").arg(input).arg("-o").arg(output);
    if filtered {
        command.args(["-c", "scene"]);
    }
    command.args(args);
    let result = command.output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    image::open(dir.path().join(if filtered { "out.png" } else { "out_scene.png" }))
        .unwrap()
        .to_rgba8()
}
#[test]
fn selected_and_batch_compositions_apply_intrinsic_aa_scaling() {
    let nearest = render(&["--scale", "4"], true);
    for algorithm in ["xbr4x", "hq4x"] {
        let actual = render(&["--antialias", algorithm], true);
        assert_eq!(actual.dimensions(), (16, 16), "{algorithm} intrinsic scale ignored");
        let source = render(&[], true);
        let context = pixelsrc::antialias::SemanticContext::empty();
        let config =
            pixelsrc::antialias::AntialiasConfig::with_algorithm(if algorithm == "xbr4x" {
                pixelsrc::antialias::AAAlgorithm::Xbr4x
            } else {
                pixelsrc::antialias::AAAlgorithm::Hq4x
            });
        let expected = if algorithm == "xbr4x" {
            pixelsrc::antialias::xbr4x(&source, &context, &config)
        } else {
            pixelsrc::antialias::hq4x(&source, &context, &config)
        };
        assert_eq!(actual, expected, "CLI must dispatch the selected algorithm");
        if algorithm == "xbr4x" {
            assert!(actual.as_raw() != nearest.as_raw(), "xbr4x did not smooth diagonal");
        }
        let batch = render(&["--antialias", algorithm], false);
        assert_eq!(batch, actual);
    }
    let scale2 = render(&["--antialias", "scale2x"], true);
    assert_eq!(scale2.dimensions(), (8, 8));
    assert_ne!(scale2, render(&["--scale", "2"], true));
    assert_eq!(render(&["--antialias", "xbr4x", "--scale", "4"], true).dimensions(), (64, 64));
    assert_eq!(render(&["--antialias", "nearest", "--scale", "4"], true), nearest);
}
#[test]
fn unsupported_animation_aa_is_diagnosed() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("aa.pxl");
    std::fs::write(&input, INPUT).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["--gif", "--animation", "an", "--antialias", "xbr4x", "-o"])
        .arg(dir.path().join("out.gif"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--antialias"));
}
