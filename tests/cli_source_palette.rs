//! Source sprites inherit omitted palettes through chains, while explicit palettes win.
use std::process::Command;

const SOURCE: &str = r##"{type:"palette",name:"p",colors:{_:"transparent",x:"#ff0000"}}
{type:"sprite",name:"base",size:[4,4],palette:"p",regions:{x:{points:[[0,0]]}}}
{type:"sprite",name:"derived",source:"base"}
{type:"sprite",name:"chain",source:"derived"}
{type:"composition",name:"scene",size:[4,4],cell_size:[4,4],sprites:{D:"chain"},layers:[{map:["D"]}]}
"##;

fn probe(text: &str, command: &[&str]) -> (std::process::Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.pxl");
    std::fs::write(&input, text).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pxl"));
    cmd.args(command).arg(input);
    if command[0] == "render" {
        cmd.arg("-o").arg(dir.path().join("out.png"));
    }
    (cmd.output().unwrap(), dir)
}

fn rendered_image(dir: &tempfile::TempDir) -> image::RgbaImage {
    let path = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "png"))
        .unwrap();
    image::open(path).unwrap().to_rgba8()
}

#[test]
fn omitted_source_palette_renders_and_validates() {
    for args in
        [&["render", "--strict", "-s", "chain"][..], &["render", "--strict", "-c", "scene"][..]]
    {
        let (result, dir) = probe(SOURCE, args);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let image = rendered_image(&dir);
        assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 255]);
    }
    let (result, _) = probe(SOURCE, &["validate"]);
    let report = String::from_utf8_lossy(&result.stdout);
    assert!(result.status.success(), "{report} {}", String::from_utf8_lossy(&result.stderr));
    assert!(!report.contains("has no palette"), "{report}");
    assert!(!report.contains("Unused token x"), "{report}");
}

#[test]
fn source_explicit_palette_overrides_inherited_colors() {
    let text = SOURCE
        .replace("name:\"derived\",source", "name:\"derived\",palette:{x:\"#0000ff\"},source");
    let (result, dir) = probe(&text, &["render", "--strict", "-s", "chain"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(rendered_image(&dir).get_pixel(0, 0).0, [0, 0, 255, 255]);
}

#[test]
fn inherited_palette_works_with_direct_source_transform() {
    let text = SOURCE.replace(
        "name:\"derived\",source:\"base\"",
        "name:\"derived\",source:\"base\",transform:[\"mirror-h\"]",
    );
    let (result, dir) = probe(&text, &["render", "--strict", "-s", "derived"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(rendered_image(&dir).get_pixel(3, 0).0, [255, 0, 0, 255]);
}

#[test]
fn direct_sprite_still_needs_a_palette() {
    let (result, _) =
        probe("{type:'sprite',name:'bad',size:[1,1],regions:{x:{points:[[0,0]]}}}", &["validate"]);
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(report.contains("has no palette"), "{report}");
}
