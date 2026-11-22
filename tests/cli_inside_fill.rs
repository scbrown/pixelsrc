//! Inside fills stay within concave boundaries and validate coordinate seeds.
use std::process::Command;

const INPUT: &str = r##"{type:"palette",name:"p",colors:{_:"transparent",bone:"#e6dccd",outline:"#000000"}}
{type:"sprite",name:"cup",size:[10,10],palette:"p",regions:{
outline:{z:1,line:[[0,0],[3,0],[3,6],[6,6],[6,0],[9,0],[9,9],[0,9],[0,0]]},
bone:{z:0,fill:"inside(outline)" SEED}}}
"##;

#[test]
fn concave_fill_and_explicit_seed_render_the_same_interior() {
    let dir = tempfile::tempdir().unwrap();
    for (name, seed) in [("auto", ""), ("explicit", ",seed:[1,1]")] {
        let input = dir.path().join(format!("{name}.pxl"));
        let output = dir.path().join(format!("{name}.png"));
        std::fs::write(&input, INPUT.replace(" SEED", seed)).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
            .args(["render", "--strict", "-s", "cup"])
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .output()
            .unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let image = image::open(output).unwrap().to_rgba8();
        assert_eq!(image.get_pixel(1, 1).0, [230, 220, 205, 255]);
        assert_eq!(image.get_pixel(8, 1).0, [230, 220, 205, 255]);
        assert_eq!(image.get_pixel(4, 0).0[3], 0);
        assert_eq!(image.get_pixel(4, 4).0[3], 0);
    }
}

#[test]
fn exterior_seed_is_rejected_in_strict_mode() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("outside.pxl");
    std::fs::write(&input, INPUT.replace(" SEED", ",seed:[4,4]")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(["render", "--strict", "-s", "cup"])
        .arg(input)
        .arg("-o")
        .arg(dir.path().join("outside.png"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("No enclosed interior"));
}
