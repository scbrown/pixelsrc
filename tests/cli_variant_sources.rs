//! Variants are addressable by name and as derived sprite sources.
use std::process::Command;

const SOURCE: &str = r##"{type:"palette",name:"p",colors:{_:"transparent",a:"#ff0000"}}
{type:"sprite",name:"dot",size:[4,4],palette:"p",regions:{a:{rect:[0,0,2,2]}}}
{type:"variant",name:"dot_blue",base:"dot",palette:{a:"#0000ff"}}
{type:"sprite",name:"flip",source:"dot_blue",transform:["mirror-h"]}
"##;

fn render(source: &str, name: &str) -> (std::process::Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.pxl");
    std::fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(["render", "--strict", "-s", name])
        .arg(input)
        .arg("-o")
        .arg(dir.path().join("out.png"))
        .output()
        .unwrap();
    (output, dir)
}

fn image(output: &std::process::Output, dir: &tempfile::TempDir) -> image::RgbaImage {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    image::open(dir.path().join("out.png")).unwrap().to_rgba8()
}

#[test]
fn variant_is_renderable_by_sprite_name() {
    let (output, dir) = render(SOURCE, "dot_blue");
    let image = image(&output, &dir);
    assert_eq!(image.dimensions(), (4, 4));
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(3, 0).0, [0, 0, 0, 0]);
}

#[test]
fn derived_sprite_inherits_variant_colors_and_geometry() {
    let (output, dir) = render(SOURCE, "flip");
    let image = image(&output, &dir);
    assert_eq!(image.dimensions(), (4, 4));
    assert_eq!(image.get_pixel(3, 0).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 0, 0]);
}

#[test]
fn explicit_source_palette_still_overrides_variant_colors() {
    let source = SOURCE.replace("name:\"flip\",source", "name:\"flip\",palette:\"p\",source");
    let (output, dir) = render(&source, "flip");
    assert_eq!(image(&output, &dir).get_pixel(3, 0).0, [255, 0, 0, 255]);
}

#[test]
fn variant_base_can_itself_inherit_or_be_a_variant() {
    let inherited = SOURCE.replace("base:\"dot\"", "base:\"inherited\"")
        + "\n{type:\"sprite\",name:\"inherited\",source:\"dot\"}\n";
    let (output, dir) = render(&inherited, "dot_blue");
    assert_eq!(image(&output, &dir).get_pixel(0, 0).0, [0, 0, 255, 255]);

    let nested = SOURCE.to_string()
        + "\n{type:\"variant\",name:\"dot_green\",base:\"dot_blue\",palette:{a:\"#00ff00\"}}\n";
    let (output, dir) = render(&nested, "dot_green");
    assert_eq!(image(&output, &dir).get_pixel(0, 0).0, [0, 255, 0, 255]);
}

#[test]
fn mixed_sprite_variant_cycle_is_diagnosed() {
    let source = r##"{type:"sprite",name:"dot",source:"dot_blue",palette:{a:"#ff0000"}}
{type:"variant",name:"dot_blue",base:"dot",palette:{a:"#0000ff"}}"##;
    let (output, _) = render(source, "dot");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Circular reference"));
}
