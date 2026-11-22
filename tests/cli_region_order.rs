//! Equal-z regions paint in source order, consistently across separate processes.
use std::process::Command;
const INPUT: &str = r##"{type:"palette",name:"p",colors:{z:"#ff0000",y:"#0000ff",x:"#00ff00",a:"#ffff00"}}
{type:"sprite",name:"order",size:[4,4],palette:"p",regions:{
z:{rect:[0,0,4,4] ZHIGH},y:{rect:[1,1,3,3]},x:{rect:[2,2,2,2]},a:{points:[[3,3]]}}}
"##;
#[test]
fn default_and_explicit_z_ties_follow_definition_order() {
    for (extra, expected) in [("", [255, 255, 0, 255]), (",z:9", [255, 0, 0, 255])] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("order.pxl");
        std::fs::write(&input, INPUT.replace(" ZHIGH", extra)).unwrap();
        for _ in 0..16 {
            let out = dir.path().join("out.png");
            let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
                .args(["render", "--strict"])
                .arg(&input)
                .arg("-o")
                .arg(&out)
                .output()
                .unwrap();
            assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
            let image = image::open(out).unwrap().to_rgba8();
            assert_eq!(image.get_pixel(3, 3).0, expected, "unstable or non-source paint order");
        }
    }
}
