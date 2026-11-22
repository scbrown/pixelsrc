//! Loaded in-memory projects for browser callers sharing several source files.
use std::collections::HashMap;
use std::io::Cursor;

use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::include::parse_include_ref;
use crate::models::{Palette, PaletteRef, TtpObject};
use crate::parser::parse_stream;

use super::{prepare_scene, render_scene_to_rgba, RenderResult, Scene};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    path: String,
    text: String,
}

/// A set of browser-provided source files, parsed once for named rendering.
#[wasm_bindgen]
pub struct Project {
    scene: Scene,
    source_count: u32,
}

#[wasm_bindgen]
impl Project {
    /// Load a JSON array of `{path, text}` files. Paths are relative virtual paths.
    /// All parsing and palette includes resolve here; rendering does no file IO.
    pub fn load(sources_json: &str) -> Result<Project, JsValue> {
        load_project(sources_json).map_err(|message| JsValue::from_str(&message))
    }

    /// Render a named sprite, variant or composition using the loaded registries.
    pub fn render(&self, name: &str) -> RenderResult {
        render_scene_to_rgba(&self.scene, name)
    }

    /// Return renderable names in source/definition order, prefixed by kind.
    pub fn list_renderables(&self) -> Vec<String> {
        self.scene.names.iter().map(|(kind, name)| format!("{}:{}", kind, name)).collect()
    }

    /// Number of files parsed when this project was loaded.
    #[wasm_bindgen(getter)]
    pub fn source_count(&self) -> u32 {
        self.source_count
    }
}

fn virtual_path(path: &str) -> Result<String, String> {
    if path.starts_with('/') || path.contains(['\\', ':']) {
        return Err(format!("Expected a relative virtual source path, got '{}'", path));
    }
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(format!("Source path escapes project: '{}'", path));
                }
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        return Err("Source path must name a file".into());
    }
    Ok(parts.join("/"))
}

fn load_project(sources_json: &str) -> Result<Project, String> {
    let sources: Vec<SourceFile> = serde_json::from_str(sources_json)
        .map_err(|error| format!("Invalid project sources: {}", error))?;
    if sources.is_empty() {
        return Err("Project requires at least one source file".into());
    }
    let source_count = u32::try_from(sources.len()).map_err(|_| "Too many project sources")?;
    let mut files = Vec::new();
    let mut palettes: HashMap<String, Vec<Palette>> = HashMap::new();
    for source in sources {
        let path = virtual_path(&source.path)?;
        if palettes.contains_key(&path) {
            return Err(format!("Duplicate source path '{}'", path));
        }
        let parsed = parse_stream(Cursor::new(source.text));
        if !parsed.warnings.is_empty() {
            return Err(parsed
                .warnings
                .iter()
                .map(|w| format!("{}:{}: {}", path, w.line, w.message))
                .collect::<Vec<_>>()
                .join("; "));
        }
        palettes.insert(
            path.clone(),
            parsed
                .objects
                .iter()
                .filter_map(|object| {
                    if let TtpObject::Palette(palette) = object {
                        Some(palette.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        );
        files.push((path, parsed.objects));
    }
    let mut objects = Vec::new();
    for (path, file_objects) in files {
        for mut object in file_objects {
            if let TtpObject::Sprite(sprite) = &mut object {
                if let PaletteRef::Named(reference) = &sprite.palette {
                    if let Some((include, name)) = parse_include_ref(reference) {
                        let base = path.rsplit_once('/').map_or("", |(base, _)| base);
                        let included = virtual_path(&if base.is_empty() {
                            include.to_string()
                        } else {
                            format!("{}/{}", base, include)
                        })?;
                        let candidates = [
                            included.clone(),
                            format!("{}.pxl", included),
                            format!("{}.jsonl", included),
                        ];
                        let available = candidates
                            .iter()
                            .find_map(|candidate| palettes.get(candidate))
                            .ok_or_else(|| {
                                format!(
                                    "{}: @include '{}' is not a supplied source file",
                                    path, include
                                )
                            })?;
                        let palette = available
                            .iter()
                            .find(|palette| name.is_none_or(|name| palette.name == name))
                            .ok_or_else(|| {
                                format!(
                                    "{}: @include '{}' has no matching palette",
                                    path, reference
                                )
                            })?;
                        sprite.palette = PaletteRef::Inline(palette.colors.clone());
                    }
                }
            }
            objects.push(object);
        }
    }
    Ok(Project { scene: prepare_scene(objects, Vec::new()), source_count })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHARED: &str = r##"{type:"palette",name:"p",colors:{x:"#ff0000"}}
{type:"sprite",name:"dot",size:[2,2],palette:"p",regions:{x:{points:[[0,0]]}}}"##;
    const FIGURE: &str = r#"{type:"composition",name:"figure",size:[2,2],cell_size:[2,2],sprites:{D:"dot"},layers:[{map:["D"]}]}"#;

    #[test]
    fn shared_project_matches_concatenated_source_and_reuses_parsed_scene() {
        let sources=serde_json::json!([{ "path":"shared.pxl", "text":SHARED }, {"path":"figure.pxl","text":FIGURE}]).to_string();
        let project = load_project(&sources).unwrap();
        assert_eq!(project.source_count(), 2);
        let expected =
            super::super::render_named_to_rgba(&format!("{}\n{}", SHARED, FIGURE), "figure");
        for _ in 0..3 {
            let actual = project.render("figure");
            assert_eq!(actual.pixels(), expected.pixels());
            assert_eq!((actual.width(), actual.height()), (2, 2));
            assert!(actual.warnings().is_empty(), "{:?}", actual.warnings());
        }
    }

    #[test]
    fn virtual_palette_includes_and_missing_source_diagnostics() {
        let sources=serde_json::json!([
            {"path":"art/palettes/base.pxl","text":SHARED.lines().next().unwrap()},
            {"path":"art/scenes/figure.pxl","text":SHARED.lines().nth(1).unwrap().replace("palette:\"p\"","palette:\"@include:../palettes/base.pxl#p\"")}
        ]).to_string();
        let project = load_project(&sources).unwrap();
        assert_eq!(&project.render("dot").pixels()[..4], &[255, 0, 0, 255]);
        assert!(load_project(&sources.replace("base.pxl#p", "missing.pxl#p"))
            .err()
            .unwrap()
            .contains("not a supplied source"));
        assert!(load_project(&sources.replace("base.pxl#p", "base.pxl#missing"))
            .err()
            .unwrap()
            .contains("no matching palette"));
    }

    #[test]
    fn nested_compositions_render_and_cycles_report_without_recursing_forever() {
        let nested = format!("{}\n{{type:'composition',name:'page',size:[2,2],cell_size:[2,2],sprites:{{F:'figure'}},layers:[{{map:['F']}}]}}", FIGURE);
        let sources = serde_json::json!([{"path":"shared.pxl","text":SHARED},{"path":"nested.pxl","text":nested}]).to_string();
        let project = load_project(&sources).unwrap();
        let image = project.render("page");
        assert_eq!(&image.pixels()[..4], &[255, 0, 0, 255]);
        assert!(image.warnings().is_empty(), "{:?}", image.warnings());
        let cycle = sources.replace("F:'figure'", "F:'page'");
        let image = load_project(&cycle).unwrap().render("page");
        assert!(
            image.warnings().iter().any(|w| w.to_lowercase().contains("cycle")),
            "{:?}",
            image.warnings()
        );
    }

    #[test]
    fn loaded_project_collects_direct_positioned_and_nested_layer_references() {
        let figures = r#"{type:"composition",name:"figure",size:[4,4],layers:[{sprite:"dot",x:1,y:1}]}
{type:"composition",name:"page",size:[6,6],layers:[{sprite:"figure",x:1,y:1}]}"#;
        let sources = serde_json::json!([
            {"path":"shared.pxl","text":SHARED},
            {"path":"figures.pxl","text":figures}
        ])
        .to_string();
        let project = load_project(&sources).unwrap();
        for _ in 0..2 {
            let image = project.render("page");
            assert_eq!((image.width(), image.height()), (6, 6));
            assert!(image.warnings().is_empty(), "{:?}", image.warnings());
            for (index, pixel) in image.pixels().as_chunks::<4>().0.iter().enumerate() {
                assert_eq!(
                    pixel,
                    if index == 2 * 6 + 2 { &[255, 0, 0, 255] } else { &[0, 0, 0, 0] }
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_paths_duplicate_paths_and_invalid_source() {
        for sources in [
            "[]".to_string(),
            serde_json::json!([{"path":"../escape.pxl","text":SHARED}]).to_string(),
            serde_json::json!([{"path":"a.pxl","text":SHARED},{"path":"./a.pxl","text":FIGURE}])
                .to_string(),
            serde_json::json!([{"path":"bad.pxl","text":"{oops"}]).to_string(),
        ] {
            assert!(load_project(&sources).is_err());
        }
    }
}
