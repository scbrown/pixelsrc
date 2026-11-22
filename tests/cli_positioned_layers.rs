use std::process::Command;
fn render(layer: &str, size: &str) -> image::RgbaImage {
    let dir = tempfile::tempdir().unwrap();
    let source = format!(
        r##"{{type:"palette",name:"p",colors:{{x:"#ff0000"}}}}
{{type:"sprite",name:"dot",size:[4,4],palette:"p",regions:{{x:{{points:[[0,0],[2,2]]}}}}}}
{{type:"composition",name:"inner",size:[6,6],layers:[{{sprite:"dot",x:1,y:1}}]}}
{{type:"composition",name:"scene",{size}layers:[{layer}]}}
"##
    );
    let input = dir.path().join("input.pxl");
    std::fs::write(&input, source).unwrap();
    let output = dir.path().join("out.png");
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .arg("render")
        .arg(input)
        .args(["-c", "scene", "--strict", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    image::open(output).unwrap().to_rgba8()
}
#[test]
fn positioned_sprite_works_without_cell_size_or_sprite_map() {
    let image = render(r#"{sprite:"dot",x:2,y:1}"#, "size:[8,8],");
    assert_eq!(image.dimensions(), (8, 8));
    assert_eq!(image.get_pixel(2, 1).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(4, 3).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 0, 0]);
}
#[test]
fn negative_offsets_clip_and_size_can_be_inferred() {
    let image = render(r#"{sprite:"dot",x:-1,y:-1}"#, "size:[4,4],");
    assert_eq!(image.get_pixel(1, 1).0, [255, 0, 0, 255]);
    let image = render(r#"{sprite:"dot",x:2,y:3}"#, "");
    assert_eq!(image.dimensions(), (6, 7));
}

#[test]
fn nested_positioned_layer_uses_transforms_and_blending() {
    let image = render(r#"{sprite:"inner",x:1,y:2,opacity:0.5}"#, "size:[10,10],");
    assert_eq!(image.get_pixel(2, 3).0, [255, 0, 0, 128]);
    let image = render(r#"{sprite:"dot",x:1,y:1,transform:["mirror-h"]}"#, "size:[8,8],");
    assert_eq!(image.get_pixel(4, 1).0, [255, 0, 0, 255]);
}

#[test]
fn formatter_preserves_position_and_effects() {
    let source = r#"{"type":"composition","name":"scene","layers":[{"sprite":"dot","x":-2,"y":3,"transform":["mirror-h"],"opacity":0.5,"blend":"multiply"}]}"#;
    let formatted = pixelsrc::fmt::format_pixelsrc(source).unwrap();
    let before: serde_json::Value = serde_json::from_str(source).unwrap();
    let after: serde_json::Value = serde_json::from_str(&formatted).unwrap();
    assert_eq!(before["layers"], after["layers"]);
}
