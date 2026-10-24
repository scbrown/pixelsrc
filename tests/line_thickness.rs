//! The CLI renderer and mask reader must agree on semantic line coverage.
use image::Rgba;
use serde_json::json;
use std::process::Command;

#[test]
fn thick_line_png_mask_union_and_z_order() {
    let dir = tempfile::tempdir().expect("temporary fixture directory");
    for (name, shape, skin_z, count, sample) in [
        ("thin", json!({"line": [[2, 4], [5, 4]], "thickness": 1}), 0, 4, "skin"),
        ("thick", json!({"line": [[2, 4], [5, 4]], "thickness": 3}), 0, 18, "hair"),
        ("nested", json!({"union": [{"line": [[2, 4], [5, 4]], "thickness": 3}]}), 0, 18, "hair"),
        ("occluded", json!({"line": [[2, 4], [5, 4]], "thickness": 3}), 20, 0, "skin"),
        ("clipped", json!({"line": [[0, 0], [1, 0]], "thickness": 3}), 0, 6, "skin"),
    ] {
        let source = dir.path().join(format!("{name}.pxl"));
        let png = dir.path().join(format!("{name}.png"));
        let mut hair = shape;
        hair["z"] = json!(10);
        let palette = json!({"type":"palette", "name":"p",
            "colors":{"skin":"#bb937a", "hair":"#302e31"}});
        let sprite = json!({"type":"sprite", "name":name, "size":[8,8], "palette":"p",
            "regions":{"skin":{"rect":[0,0,8,8], "z":skin_z}, "hair":hair}});
        std::fs::write(&source, format!("{palette}\n{sprite}\n")).expect("write fixture");
        let validate = Command::new(env!("CARGO_BIN_EXE_pxl"))
            .args(["validate", "--strict"])
            .arg(&source)
            .output()
            .expect("run validator");
        assert!(validate.status.success(), "{name}: {:?}", validate);
        let render = Command::new(env!("CARGO_BIN_EXE_pxl"))
            .arg("render")
            .arg(&source)
            .args(["--strict", "--scale", "1", "-o"])
            .arg(&png)
            .output()
            .expect("run renderer");
        assert!(render.status.success(), "{name}: {:?}", render);
        let image = image::open(png).expect("load rendered PNG").into_rgba8();
        assert_eq!(image.dimensions(), (8, 8));
        assert_eq!(
            image.pixels().filter(|&&p| p == Rgba([48, 46, 49, 255])).count(),
            count,
            "{name}"
        );
        let mask = Command::new(env!("CARGO_BIN_EXE_pxl"))
            .arg("mask")
            .arg(&source)
            .args(["--sprite", name, "--sample", "3,3", "--json"])
            .output()
            .expect("run mask reader");
        assert!(mask.status.success(), "{name}: {:?}", mask);
        let value: serde_json::Value = serde_json::from_slice(&mask.stdout).expect("mask JSON");
        assert_eq!(value["token"], format!("{{{sample}}}"), "{name}");
    }
}
