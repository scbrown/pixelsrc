//! WASM API module for browser/JS interop
//!
//! Provides WebAssembly bindings for rendering pixelsrc JSONL to images.

use std::collections::HashMap;
use std::io::Cursor;

use image::RgbaImage;
use wasm_bindgen::prelude::*;

use crate::composition::render_composition;
use crate::models::{Composition, Sprite, TtpObject};
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, SpriteRegistry};
use crate::renderer::{render_resolved, render_sprite};

/// Initialize panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
}

/// Result of rendering a sprite to RGBA pixels.
#[wasm_bindgen]
pub struct RenderResult {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl RenderResult {
    /// Width of the rendered image in pixels
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the rendered image in pixels
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGBA pixel data (4 bytes per pixel)
    #[wasm_bindgen(getter)]
    pub fn pixels(&self) -> Vec<u8> {
        self.pixels.clone()
    }

    /// Any warnings generated during rendering
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

/// Parse JSONL and build registry, returning (registry, sprites, warnings)
fn parse_and_prepare(jsonl: &str) -> (PaletteRegistry, Vec<crate::models::Sprite>, Vec<String>) {
    let parse_result = parse_stream(Cursor::new(jsonl));
    let mut registry = PaletteRegistry::new();
    let mut sprites = Vec::new();
    let warnings: Vec<String> =
        parse_result.warnings.iter().map(|w| format!("line {}: {}", w.line, w.message)).collect();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(p) => {
                registry.register(p);
            }
            TtpObject::Sprite(s) => {
                sprites.push(s);
            }
            _ => {
                // Skip compositions and animations for now
            }
        }
    }

    (registry, sprites, warnings)
}

/// Render the first sprite in a JSONL string to PNG bytes.
///
/// # Arguments
/// * `jsonl` - JSONL string containing palette and sprite definitions
///
/// # Returns
/// PNG image data as a byte array, or an empty array if no sprites found
#[wasm_bindgen]
pub fn render_to_png(jsonl: &str) -> Vec<u8> {
    let (registry, sprites, _warnings) = parse_and_prepare(jsonl);

    if sprites.is_empty() {
        return Vec::new();
    }

    let sprite = &sprites[0];
    let resolved = registry.resolve_lenient(sprite);
    let (image, _render_warnings) = render_sprite(sprite, &resolved.palette.colors);

    // Encode to PNG
    let mut png_data = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder
            .write_image(image.as_raw(), image.width(), image.height(), image::ColorType::Rgba8)
            .ok();
    }

    png_data
}

/// Render the first sprite in a JSONL string to RGBA pixels.
///
/// # Arguments
/// * `jsonl` - JSONL string containing palette and sprite definitions
///
/// # Returns
/// RenderResult containing width, height, pixels (RGBA), and warnings
#[wasm_bindgen]
pub fn render_to_rgba(jsonl: &str) -> RenderResult {
    let (registry, sprites, mut warnings) = parse_and_prepare(jsonl);

    if sprites.is_empty() {
        warnings.push("No sprites found in input".to_string());
        return RenderResult { width: 0, height: 0, pixels: Vec::new(), warnings };
    }

    let sprite = &sprites[0];
    let resolved = registry.resolve_lenient(sprite);

    if let Some(w) = &resolved.warning {
        warnings.push(w.message.clone());
    }

    let (image, render_warnings) = render_sprite(sprite, &resolved.palette.colors);

    for w in render_warnings {
        warnings.push(w.message);
    }

    RenderResult {
        width: image.width(),
        height: image.height(),
        pixels: image.into_raw(),
        warnings,
    }
}

/// Everything a stream defines, indexed for rendering by name.
struct Scene {
    palettes: PaletteRegistry,
    registry: SpriteRegistry,
    sprites: HashMap<String, Sprite>,
    compositions: HashMap<String, Composition>,
    /// Sprites, variants and compositions in definition order.
    names: Vec<(&'static str, String)>,
    warnings: Vec<String>,
}

fn load_scene(jsonl: &str) -> Scene {
    let parsed = parse_stream(Cursor::new(jsonl));
    let mut scene = Scene {
        palettes: PaletteRegistry::new(),
        registry: SpriteRegistry::new(),
        sprites: HashMap::new(),
        compositions: HashMap::new(),
        names: Vec::new(),
        warnings: parsed
            .warnings
            .iter()
            .map(|w| format!("line {}: {}", w.line, w.message))
            .collect(),
    };
    for obj in parsed.objects {
        match obj {
            TtpObject::Palette(p) => scene.palettes.register(p),
            TtpObject::Sprite(s) => {
                scene.names.push(("sprite", s.name.clone()));
                scene.registry.register_sprite(s.clone());
                scene.sprites.insert(s.name.clone(), s);
            }
            TtpObject::Variant(v) => {
                scene.names.push(("variant", v.name.clone()));
                scene.registry.register_variant(v);
            }
            TtpObject::Composition(c) => {
                scene.names.push(("composition", c.name.clone()));
                scene.compositions.insert(c.name.clone(), c);
            }
            _ => {}
        }
    }
    scene
}

/// Render one sprite or variant, applying a derived sprite's image transforms.
fn render_sprite_named(scene: &Scene, name: &str, warnings: &mut Vec<String>) -> Option<RgbaImage> {
    let resolved = match scene.registry.resolve(name, &scene.palettes, false) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("sprite '{}': {}", name, e));
            return None;
        }
    };
    for w in &resolved.warnings {
        warnings.push(format!("sprite '{}': {}", name, w.message));
    }
    let (mut image, render_warnings) = render_resolved(&resolved);
    for w in render_warnings {
        warnings.push(format!("sprite '{}': {}", name, w.message));
    }
    if let Some(specs) = scene.sprites.get(name).and_then(|s| s.transform.as_deref()) {
        let (transformed, tw) =
            crate::transforms::apply_image_transform_specs(image, specs, Some(&resolved.palette));
        image = transformed;
        for w in tw {
            warnings.push(format!("sprite '{}': {}", name, w));
        }
    }
    Some(image)
}

/// Render a composition, sprite or variant by name. Compositions take precedence.
fn render_named(scene: &Scene, name: &str, warnings: &mut Vec<String>) -> Option<RgbaImage> {
    let Some(comp) = scene.compositions.get(name) else {
        if scene.registry.contains(name) {
            return render_sprite_named(scene, name, warnings);
        }
        warnings.push(format!("No sprite, variant or composition named '{}'", name));
        return None;
    };
    let mut rendered: HashMap<String, RgbaImage> = HashMap::new();
    let wanted = comp.base.iter().chain(comp.sprites.values().flatten());
    for sprite_name in wanted {
        if rendered.contains_key(sprite_name) {
            continue;
        }
        if let Some(image) = render_sprite_named(scene, sprite_name, warnings) {
            rendered.insert(sprite_name.clone(), image);
        }
    }
    match render_composition(comp, &rendered, false, None) {
        Ok((image, comp_warnings)) => {
            for w in comp_warnings {
                warnings.push(format!("composition '{}': {}", name, w.message));
            }
            Some(image)
        }
        Err(e) => {
            warnings.push(format!("composition '{}': {}", name, e));
            None
        }
    }
}

fn encode_png(image: &RgbaImage) -> Vec<u8> {
    use image::ImageEncoder;
    let mut png_data = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_data)
        .write_image(image.as_raw(), image.width(), image.height(), image::ColorType::Rgba8)
        .ok();
    png_data
}

/// Render a sprite, variant or composition by name to RGBA pixels.
///
/// Unlike `render_to_rgba`, which renders the first sprite only, this resolves
/// variants, derived sprites and compositions the way `pxl render` does. Rendering
/// is lenient: problems are reported in `warnings`, and a name that cannot be
/// rendered gives a 0x0 result.
///
/// # Arguments
/// * `jsonl` - Pixelsrc source: palettes, sprites, variants, compositions
/// * `name` - The sprite, variant or composition to render
#[wasm_bindgen]
pub fn render_named_to_rgba(jsonl: &str, name: &str) -> RenderResult {
    let scene = load_scene(jsonl);
    let mut warnings = scene.warnings.clone();
    match render_named(&scene, name, &mut warnings) {
        Some(image) => RenderResult {
            width: image.width(),
            height: image.height(),
            pixels: image.into_raw(),
            warnings,
        },
        None => RenderResult { width: 0, height: 0, pixels: Vec::new(), warnings },
    }
}

/// Render a sprite, variant or composition by name to PNG bytes.
///
/// Returns an empty array if the name cannot be rendered.
#[wasm_bindgen]
pub fn render_named_to_png(jsonl: &str, name: &str) -> Vec<u8> {
    let scene = load_scene(jsonl);
    let mut warnings = Vec::new();
    render_named(&scene, name, &mut warnings).map(|image| encode_png(&image)).unwrap_or_default()
}

/// List everything that can be rendered by name, in definition order.
///
/// Each entry is `"<kind>:<name>"`, where kind is `sprite`, `variant` or
/// `composition`.
#[wasm_bindgen]
pub fn list_renderables(jsonl: &str) -> Vec<String> {
    load_scene(jsonl).names.into_iter().map(|(kind, name)| format!("{}:{}", kind, name)).collect()
}

/// List all sprite names in a JSONL string.
///
/// # Arguments
/// * `jsonl` - JSONL string containing sprite definitions
///
/// # Returns
/// Array of sprite names
#[wasm_bindgen]
pub fn list_sprites(jsonl: &str) -> Vec<String> {
    let (_registry, sprites, _warnings) = parse_and_prepare(jsonl);
    sprites.into_iter().map(|s| s.name).collect()
}

/// Validate a JSONL string and return any errors/warnings.
///
/// # Arguments
/// * `jsonl` - JSONL string to validate
///
/// # Returns
/// Array of validation messages (empty if valid)
#[wasm_bindgen]
pub fn validate(jsonl: &str) -> Vec<String> {
    let (registry, sprites, mut warnings) = parse_and_prepare(jsonl);

    // Validate palette references
    for sprite in &sprites {
        let resolved = registry.resolve_lenient(sprite);
        if let Some(w) = &resolved.warning {
            warnings.push(format!("sprite '{}': {}", sprite.name, w.message));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_DOT: &str = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;

    const HEART_WITH_PALETTE: &str = r##"{"type": "palette", "name": "reds", "colors": {"_": "#00000000", "r": "#FF0000", "p": "#FF6B6B"}}
{"type": "sprite", "name": "heart", "size": [4, 4], "palette": "reds", "regions": {"r": {"points": [[1, 0], [2, 0], [0, 1], [1, 1], [2, 1], [3, 1], [1, 2], [2, 2], [2, 3]], "z": 0}}}"##;

    const SCENE: &str = r##"{"type": "palette", "name": "p", "colors": {"_": "#00000000", "a": "#FF0000"}}
{"type": "sprite", "name": "dot", "size": [2, 2], "palette": "p", "regions": {"a": {"rect": [0, 0, 1, 1], "z": 0}}}
{"type": "variant", "name": "dot_blue", "base": "dot", "palette": {"a": "#0000FF"}}
{"type": "sprite", "name": "dot_flip", "palette": "p", "source": "dot", "transform": ["mirror-h"]}
{"type": "composition", "name": "scene", "size": [4, 2], "cell_size": [2, 2], "sprites": {"D": "dot", "B": "dot_blue"}, "layers": [{"map": ["DB"]}]}"##;

    fn pixel(r: &RenderResult, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * r.width + x) * 4) as usize;
        [r.pixels[i], r.pixels[i + 1], r.pixels[i + 2], r.pixels[i + 3]]
    }

    #[test]
    fn test_list_renderables() {
        assert_eq!(
            list_renderables(SCENE),
            vec!["sprite:dot", "variant:dot_blue", "sprite:dot_flip", "composition:scene"]
        );
    }

    #[test]
    fn test_render_named_sprite_is_not_just_the_first() {
        let r = render_named_to_rgba(SCENE, "dot_flip");
        assert_eq!((r.width, r.height), (2, 2));
        assert_eq!(pixel(&r, 1, 0), [255, 0, 0, 255], "mirror-h moves the dot to the right");
        assert_eq!(pixel(&r, 0, 0)[3], 0);
    }

    #[test]
    fn test_render_named_variant() {
        let r = render_named_to_rgba(SCENE, "dot_blue");
        assert_eq!(pixel(&r, 0, 0), [0, 0, 255, 255]);
    }

    #[test]
    fn test_render_named_composition_places_sprite_and_variant() {
        let r = render_named_to_rgba(SCENE, "scene");
        assert_eq!((r.width, r.height), (4, 2));
        assert_eq!(pixel(&r, 0, 0), [255, 0, 0, 255], "dot in the first cell");
        assert_eq!(pixel(&r, 2, 0), [0, 0, 255, 255], "blue variant in the second cell");
        assert_eq!(pixel(&r, 1, 1)[3], 0);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }

    #[test]
    fn test_render_named_unknown_name_reports_it() {
        let r = render_named_to_rgba(SCENE, "nope");
        assert_eq!((r.width, r.height), (0, 0));
        assert!(r.warnings.iter().any(|w| w.contains("nope")));
        assert!(render_named_to_png(SCENE, "nope").is_empty());
    }

    #[test]
    fn test_render_named_png_magic() {
        let png = render_named_to_png(SCENE, "scene");
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_list_sprites() {
        let result = list_sprites(MINIMAL_DOT);
        assert_eq!(result, vec!["dot"]);
    }

    #[test]
    fn test_list_sprites_multiple() {
        let jsonl = r#"{"type": "sprite", "name": "a", "size": [1, 1], "palette": {}, "regions": {}}
{"type": "sprite", "name": "b", "size": [1, 1], "palette": {}, "regions": {}}"#;
        let result = list_sprites(jsonl);
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn test_validate_valid() {
        let result = validate(HEART_WITH_PALETTE);
        assert!(result.is_empty(), "Expected no warnings: {:?}", result);
    }

    #[test]
    fn test_validate_missing_palette() {
        let jsonl = r#"{"type": "sprite", "name": "bad", "size": [2, 1], "palette": "nonexistent", "regions": {"x": {"rect": [0, 0, 2, 1], "z": 0}}}"#;
        let result = validate(jsonl);
        assert!(!result.is_empty());
        assert!(result[0].contains("not found"));
    }

    #[test]
    fn test_render_to_rgba() {
        let result = render_to_rgba(MINIMAL_DOT);
        assert_eq!(result.width(), 1);
        assert_eq!(result.height(), 1);
        assert_eq!(result.pixels().len(), 4); // 1 pixel * 4 bytes (RGBA)
                                              // Red pixel: #FF0000 = [255, 0, 0, 255]
        assert_eq!(result.pixels()[0], 255); // R
        assert_eq!(result.pixels()[1], 0); // G
        assert_eq!(result.pixels()[2], 0); // B
        assert_eq!(result.pixels()[3], 255); // A
    }

    #[test]
    fn test_render_to_rgba_no_sprites() {
        let result = render_to_rgba(r#"{"type": "palette", "name": "empty", "colors": {}}"#);
        assert_eq!(result.width(), 0);
        assert_eq!(result.height(), 0);
        assert!(result.warnings().iter().any(|w| w.contains("No sprites")));
    }

    #[test]
    fn test_render_to_png() {
        let result = render_to_png(MINIMAL_DOT);
        assert!(!result.is_empty());
        // PNG magic bytes
        assert_eq!(&result[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_render_to_png_no_sprites() {
        let result = render_to_png(r#"{"type": "palette", "name": "empty", "colors": {}}"#);
        assert!(result.is_empty());
    }
}
