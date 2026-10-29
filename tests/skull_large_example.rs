//! Keep the large skull sources and their published renders in agreement.
use std::path::Path;
use std::process::Command;

#[test]
fn large_skull_sources_validate_and_match_saved_renders() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/skull");
    let output = tempfile::tempdir().expect("temporary render directory");
    for (stem, size, preview_scale) in [("skull-128", 128, 8), ("skull-256", 256, 4)] {
        let source = examples.join(format!("{stem}.pxl"));
        let validate = Command::new(env!("CARGO_BIN_EXE_pxl"))
            .args(["validate", "--strict"])
            .arg(&source)
            .output()
            .expect("run strict validator");
        assert!(validate.status.success(), "{stem}: {validate:?}");
        for (scale, suffix) in
            [(1, ".png".to_string()), (preview_scale, format!("_{preview_scale}x.png"))]
        {
            let name = format!("{stem}{suffix}");
            let rendered = output.path().join(&name);
            let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
                .arg("render")
                .arg(&source)
                .args(["--strict", "--scale", &scale.to_string(), "-o"])
                .arg(&rendered)
                .output()
                .expect("run renderer");
            assert!(result.status.success(), "{name}: {result:?}");
            let actual = image::open(rendered).expect("read new PNG").into_rgba8();
            let expected = image::open(examples.join(&name)).expect("read saved PNG").into_rgba8();
            assert_eq!(actual.dimensions(), (size * scale, size * scale), "{name}");
            assert_eq!(actual, expected, "{name}: source and saved pixels differ");
        }
    }
}
