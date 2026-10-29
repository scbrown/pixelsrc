//! Keep the editable portrait sources and published renders in agreement.
use std::path::Path;
use std::process::Command;

#[test]
fn stiwi_sources_validate_and_match_saved_renders() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/stiwi");
    let output = tempfile::tempdir().expect("temporary render directory");
    for (stem, size, preview_scale) in [
        ("stiwi-hand-48", 48, 11),
        ("stiwi-hand-32", 32, 16),
        ("stiwi-hand-128", 128, 8),
        ("stiwi-hand-256", 256, 4),
    ] {
        let source = examples.join(format!("{stem}.pxl"));
        let validate = Command::new(env!("CARGO_BIN_EXE_pxl"))
            .args(["validate", "--strict"])
            .arg(&source)
            .output()
            .expect("run strict validator");
        assert!(validate.status.success(), "{stem}: {validate:?}");
        for (scale, suffix) in [(1, ".png"), (preview_scale, "-preview.png")] {
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
