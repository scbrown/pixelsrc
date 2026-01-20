//! Composition rendering - layering sprites onto a canvas

use crate::models::{Composition, VarOr};
use crate::variables::VariableRegistry;
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ============================================================================
// Render Context (NC-3)
// ============================================================================

/// Context for rendering operations with caching support.
///
/// The RenderContext stores rendered compositions to avoid redundant rendering
/// when the same composition is referenced multiple times (e.g., in nested
/// compositions or tiled layouts).
///
/// # Example
///
/// ```ignore
/// use pixelsrc::composition::RenderContext;
/// use image::RgbaImage;
///
/// let mut ctx = RenderContext::new();
///
/// // First render - compute and cache
/// if ctx.get_cached("scene").is_none() {
///     let rendered = render_composition(/* ... */);
///     ctx.cache("scene".to_string(), rendered);
/// }
///
/// // Second reference - get from cache
/// let cached = ctx.get_cached("scene").unwrap();
/// ```
#[derive(Debug, Default, Clone)]
pub struct RenderContext {
    /// Cache of rendered compositions by name
    composition_cache: HashMap<String, RgbaImage>,
    /// Stack of composition names currently being rendered (for cycle detection)
    render_stack: Vec<String>,
}

impl RenderContext {
    /// Create a new empty render context.
    pub fn new() -> Self {
        Self { composition_cache: HashMap::new(), render_stack: Vec::new() }
    }

    /// Get a cached rendered composition by name.
    ///
    /// Returns `None` if the composition has not been cached yet.
    pub fn get_cached(&self, name: &str) -> Option<&RgbaImage> {
        self.composition_cache.get(name)
    }

    /// Cache a rendered composition.
    ///
    /// If a composition with the same name was already cached, it will be replaced.
    pub fn cache(&mut self, name: String, image: RgbaImage) {
        self.composition_cache.insert(name, image);
    }

    /// Check if a composition is already cached.
    pub fn is_cached(&self, name: &str) -> bool {
        self.composition_cache.contains_key(name)
    }

    /// Clear all cached compositions.
    pub fn clear(&mut self) {
        self.composition_cache.clear();
    }

    /// Get the number of cached compositions.
    pub fn len(&self) -> usize {
        self.composition_cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.composition_cache.is_empty() && self.render_stack.is_empty()
    }

    /// Push a composition onto the render stack.
    ///
    /// Returns `Ok(())` if successful, or `Err(CompositionError::CycleDetected)`
    /// if the composition is already on the stack (indicating a cycle).
    pub fn push(&mut self, name: impl Into<String>) -> Result<(), CompositionError> {
        let name = name.into();
        if self.render_stack.contains(&name) {
            let mut cycle_path: Vec<String> =
                self.render_stack.iter().skip_while(|n| *n != &name).cloned().collect();
            cycle_path.push(name);
            return Err(CompositionError::CycleDetected { cycle_path });
        }
        self.render_stack.push(name);
        Ok(())
    }

    /// Pop a composition from the render stack.
    pub fn pop(&mut self) -> Option<String> {
        self.render_stack.pop()
    }

    /// Check if a composition is currently being rendered.
    pub fn contains(&self, name: &str) -> bool {
        self.render_stack.iter().any(|n| n == name)
    }

    /// Get the current depth of the render stack.
    pub fn depth(&self) -> usize {
        self.render_stack.len()
    }

    /// Get the current render path as a slice.
    pub fn path(&self) -> &[String] {
        &self.render_stack
    }
}

// ============================================================================
// Blend Modes (ATF-10)
// ============================================================================

/// Blend modes for composition layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    /// Standard alpha compositing (source over destination)
    #[default]
    Normal,
    /// Darkens underlying colors: result = base * blend
    Multiply,
    /// Lightens underlying colors: result = 1 - (1 - base) * (1 - blend)
    Screen,
    /// Combines multiply/screen based on base brightness
    Overlay,
    /// Additive blending: result = min(1, base + blend)
    Add,
    /// Subtractive blending: result = max(0, base - blend)
    Subtract,
    /// Color difference: result = abs(base - blend)
    Difference,
    /// Keeps darker color: result = min(base, blend)
    Darken,
    /// Keeps lighter color: result = max(base, blend)
    Lighten,
}

impl BlendMode {
    /// Parse a blend mode from string
    pub fn from_str(s: &str) -> Option<BlendMode> {
        match s.to_lowercase().as_str() {
            "normal" => Some(BlendMode::Normal),
            "multiply" => Some(BlendMode::Multiply),
            "screen" => Some(BlendMode::Screen),
            "overlay" => Some(BlendMode::Overlay),
            "add" | "additive" => Some(BlendMode::Add),
            "subtract" | "subtractive" => Some(BlendMode::Subtract),
            "difference" => Some(BlendMode::Difference),
            "darken" => Some(BlendMode::Darken),
            "lighten" => Some(BlendMode::Lighten),
            _ => None,
        }
    }

    /// Apply blend mode to a single color channel (values are 0.0-1.0)
    fn blend_channel(&self, base: f32, blend: f32) -> f32 {
        match self {
            BlendMode::Normal => blend,
            BlendMode::Multiply => base * blend,
            BlendMode::Screen => 1.0 - (1.0 - base) * (1.0 - blend),
            BlendMode::Overlay => {
                if base < 0.5 {
                    2.0 * base * blend
                } else {
                    1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
                }
            }
            BlendMode::Add => (base + blend).min(1.0),
            BlendMode::Subtract => (base - blend).max(0.0),
            BlendMode::Difference => (base - blend).abs(),
            BlendMode::Darken => base.min(blend),
            BlendMode::Lighten => base.max(blend),
        }
    }
}

// ============================================================================
// CSS Variable Resolution (CSS-9)
// ============================================================================

/// Resolve a blend mode string, potentially containing a var() reference.
///
/// Returns the resolved blend mode and any warning if resolution failed.
pub fn resolve_blend_mode(
    blend: Option<&str>,
    registry: Option<&VariableRegistry>,
) -> (BlendMode, Option<Warning>) {
    let Some(blend_str) = blend else {
        return (BlendMode::Normal, None);
    };

    // Check if it contains var() and we have a registry
    let resolved = if blend_str.contains("var(") {
        if let Some(reg) = registry {
            match reg.resolve(blend_str) {
                Ok(resolved) => resolved,
                Err(e) => {
                    return (
                        BlendMode::Normal,
                        Some(Warning::new(format!(
                            "Failed to resolve blend mode variable '{}': {}, using normal",
                            blend_str, e
                        ))),
                    );
                }
            }
        } else {
            // No registry provided but var() used - warn and use default
            return (
                BlendMode::Normal,
                Some(Warning::new(format!(
                    "Blend mode '{}' contains var() but no variable registry provided, using normal",
                    blend_str
                ))),
            );
        }
    } else {
        blend_str.to_string()
    };

    // Parse the resolved string
    match BlendMode::from_str(&resolved) {
        Some(mode) => (mode, None),
        None => (
            BlendMode::Normal,
            Some(Warning::new(format!("Unknown blend mode '{}', using normal", resolved))),
        ),
    }
}

/// Resolve an opacity value, potentially containing a var() reference.
///
/// Returns the resolved opacity (clamped to 0.0-1.0) and any warning if resolution failed.
pub fn resolve_opacity(
    opacity: Option<&VarOr<f64>>,
    registry: Option<&VariableRegistry>,
) -> (f64, Option<Warning>) {
    let Some(opacity_val) = opacity else {
        return (1.0, None);
    };

    match opacity_val {
        VarOr::Value(v) => (*v, None),
        VarOr::Var(var_str) => {
            if let Some(reg) = registry {
                match reg.resolve(var_str) {
                    Ok(resolved) => {
                        // Try to parse the resolved string as f64
                        match resolved.trim().parse::<f64>() {
                            Ok(v) => (v.clamp(0.0, 1.0), None),
                            Err(_) => (
                                1.0,
                                Some(Warning::new(format!(
                                    "Opacity variable '{}' resolved to '{}' which is not a valid number, using 1.0",
                                    var_str, resolved
                                ))),
                            ),
                        }
                    }
                    Err(e) => (
                        1.0,
                        Some(Warning::new(format!(
                            "Failed to resolve opacity variable '{}': {}, using 1.0",
                            var_str, e
                        ))),
                    ),
                }
            } else {
                (
                    1.0,
                    Some(Warning::new(format!(
                        "Opacity '{}' contains var() but no variable registry provided, using 1.0",
                        var_str
                    ))),
                )
            }
        }
    }
}

/// A warning generated during composition rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

/// Error when rendering a composition in strict mode.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompositionError {
    /// Sprite dimensions exceed cell size
    #[error("Sprite '{sprite_name}' ({sprite_w}x{sprite_h}) exceeds cell size ({cell_w}x{cell_h}) in composition '{composition_name}'", sprite_w = sprite_size.0, sprite_h = sprite_size.1, cell_w = cell_size.0, cell_h = cell_size.1)]
    SizeMismatch {
        sprite_name: String,
        sprite_size: (u32, u32),
        cell_size: (u32, u32),
        composition_name: String,
    },
    /// Canvas size is not divisible by cell_size
    #[error("Size ({size_w}x{size_h}) is not divisible by cell_size ({cell_w}x{cell_h}) in composition '{composition_name}'", size_w = size.0, size_h = size.1, cell_w = cell_size.0, cell_h = cell_size.1)]
    SizeNotDivisible { size: (u32, u32), cell_size: (u32, u32), composition_name: String },
    /// Map dimensions don't match expected grid size
    #[error("Map dimensions ({actual_w}x{actual_h}) don't match expected grid size ({expected_w}x{expected_h}) for {layer_desc} in composition '{composition_name}'", actual_w = actual_dimensions.0, actual_h = actual_dimensions.1, expected_w = expected_dimensions.0, expected_h = expected_dimensions.1, layer_desc = layer_name.as_ref().map(|n| format!("layer '{}'", n)).unwrap_or_else(|| "unnamed layer".to_string()))]
    MapDimensionMismatch {
        layer_name: Option<String>,
        actual_dimensions: (usize, usize),
        expected_dimensions: (u32, u32),
        composition_name: String,
    },
    /// Cycle detected in nested composition references
    #[error("Cycle detected in composition rendering: {}", cycle_path.join(" -> "))]
    CycleDetected {
        /// The composition names forming the cycle (e.g., ["A", "B", "A"])
        cycle_path: Vec<String>,
    },
}

/// Render a composition to an RGBA image buffer.
///
/// Takes a composition and a map of sprite name -> rendered image.
/// Returns the rendered composition and any warnings generated.
///
/// # Basic Rendering (Task 2.1)
///
/// This implementation supports:
/// - Single layer rendering
/// - cell_size [1, 1] (pixel-perfect placement)
/// - Size inference from layers
///
/// # Cell Size Scaling (Task 2.3)
///
/// The `cell_size` field determines how many pixels each grid character represents:
/// - `cell_size: [4, 4]` means each character in the layer map represents a 4x4 pixel area
/// - Sprites are placed at positions calculated as `(col * cell_size[0], row * cell_size[1])`
/// - Sprite top-left aligns to cell top-left
///
/// # Size Inference
///
/// Canvas size is determined by (in order of priority):
/// 1. `composition.size` if explicitly set
/// 2. `composition.base` sprite dimensions (if base is set and found)
/// 3. Inferred from layer maps and cell_size
///
/// # Size Mismatch Handling (Task 2.5)
///
/// When a sprite's dimensions exceed the cell size:
/// - In lenient mode (strict=false): Emits a warning, sprite anchors top-left and overwrites adjacent cells
/// - In strict mode (strict=true): Returns an error
///
/// # Examples
///
/// # CSS Variable Support (CSS-9)
///
/// Layer `blend` and `opacity` properties can use CSS variable syntax:
/// ```json
/// {
///   "blend": "var(--layer-blend)",
///   "opacity": "var(--layer-opacity, 0.8)"
/// }
/// ```
///
/// Pass a `VariableRegistry` to resolve these references before rendering.
///
/// ```ignore
/// use pixelsrc::composition::render_composition;
/// use pixelsrc::models::Composition;
/// use pixelsrc::variables::VariableRegistry;
/// use std::collections::HashMap;
/// use image::RgbaImage;
///
/// let comp = Composition { /* ... */ };
/// let sprites: HashMap<String, RgbaImage> = HashMap::new();
/// let mut registry = VariableRegistry::new();
/// registry.define("--layer-opacity", "0.5");
///
/// // Lenient mode with variable registry
/// let result = render_composition(&comp, &sprites, false, Some(&registry));
/// assert!(result.is_ok());
/// let (image, warnings) = result.unwrap();
/// ```
pub fn render_composition(
    comp: &Composition,
    sprites: &HashMap<String, RgbaImage>,
    strict: bool,
    variables: Option<&VariableRegistry>,
) -> Result<(RgbaImage, Vec<Warning>), CompositionError> {
    let mut warnings = Vec::new();

    // Determine cell size (default to [1, 1])
    let cell_size = comp.cell_size.unwrap_or([1, 1]);

    // Look up base sprite if specified
    let base_sprite = if let Some(ref base_name) = comp.base {
        match sprites.get(base_name) {
            Some(img) => Some(img),
            None => {
                warnings.push(Warning::new(format!(
                    "Base sprite '{}' not found for composition '{}'",
                    base_name, comp.name
                )));
                None
            }
        }
    } else {
        None
    };

    // Determine canvas size with priority:
    // 1. Explicit size
    // 2. Base sprite dimensions
    // 3. Inferred from layers + cell_size
    let (width, height) = if let Some([w, h]) = comp.size {
        (w, h)
    } else if let Some(base_img) = base_sprite {
        // Infer from base sprite dimensions
        (base_img.width(), base_img.height())
    } else {
        // Infer from layers
        let (inferred_w, inferred_h) = infer_size_from_layers(&comp.layers, cell_size);
        if inferred_w == 0 || inferred_h == 0 {
            warnings.push(Warning::new(format!(
                "Could not infer size for composition '{}', using 1x1",
                comp.name
            )));
            (1, 1)
        } else {
            (inferred_w, inferred_h)
        }
    };

    // Validate size is divisible by cell_size (only meaningful when cell_size > [1,1])
    if cell_size[0] > 1 || cell_size[1] > 1 {
        let width_divisible = width % cell_size[0] == 0;
        let height_divisible = height % cell_size[1] == 0;

        if !width_divisible || !height_divisible {
            if strict {
                return Err(CompositionError::SizeNotDivisible {
                    size: (width, height),
                    cell_size: (cell_size[0], cell_size[1]),
                    composition_name: comp.name.clone(),
                });
            } else {
                warnings.push(Warning::new(format!(
                    "Size ({}x{}) is not divisible by cell_size ({}x{}) in composition '{}'",
                    width, height, cell_size[0], cell_size[1], comp.name
                )));
            }
        }
    }

    // Calculate expected grid dimensions for map validation
    let expected_cols = if cell_size[0] > 0 { width / cell_size[0] } else { width };
    let expected_rows = if cell_size[1] > 0 { height / cell_size[1] } else { height };

    // Create canvas (transparent by default)
    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    // Render base sprite first if present
    if let Some(base_img) = base_sprite {
        blit_sprite(&mut canvas, base_img, 0, 0);
    }

    // Render each layer (bottom to top)
    for layer in &comp.layers {
        // Parse layer blend mode and opacity with CSS variable resolution (ATF-10, CSS-9)
        let (blend_mode, blend_warning) = resolve_blend_mode(layer.blend.as_deref(), variables);
        if let Some(w) = blend_warning {
            warnings.push(w);
        }

        let (opacity, opacity_warning) = resolve_opacity(layer.opacity.as_ref(), variables);
        if let Some(w) = opacity_warning {
            warnings.push(w);
        }

        if let Some(ref map) = layer.map {
            // Validate map dimensions match expected grid (only when cell_size > [1,1])
            if cell_size[0] > 1 || cell_size[1] > 1 {
                let actual_rows = map.len();
                let actual_cols = map.iter().map(|r| r.chars().count()).max().unwrap_or(0);

                if actual_rows != expected_rows as usize || actual_cols != expected_cols as usize {
                    if strict {
                        return Err(CompositionError::MapDimensionMismatch {
                            layer_name: layer.name.clone(),
                            actual_dimensions: (actual_cols, actual_rows),
                            expected_dimensions: (expected_cols, expected_rows),
                            composition_name: comp.name.clone(),
                        });
                    } else {
                        let layer_desc = layer
                            .name
                            .as_ref()
                            .map(|n| format!("layer '{}'", n))
                            .unwrap_or_else(|| "unnamed layer".to_string());
                        warnings.push(Warning::new(format!(
                            "Map dimensions ({}x{}) don't match expected grid size ({}x{}) for {} in composition '{}'",
                            actual_cols, actual_rows, expected_cols, expected_rows, layer_desc, comp.name
                        )));
                    }
                }
            }

            for (row_idx, row) in map.iter().enumerate() {
                for (col_idx, char_key) in row.chars().enumerate() {
                    let key = char_key.to_string();

                    // Look up sprite name from sprites map
                    let sprite_name = match comp.sprites.get(&key) {
                        Some(Some(name)) => name,
                        Some(None) => continue, // null means transparent/skip
                        None => {
                            warnings.push(Warning::new(format!(
                                "Unknown sprite key '{}' in composition '{}'",
                                key, comp.name
                            )));
                            continue;
                        }
                    };

                    // Get the rendered sprite image
                    let sprite_image = match sprites.get(sprite_name) {
                        Some(img) => img,
                        None => {
                            warnings.push(Warning::new(format!(
                                "Sprite '{}' not found for composition '{}'",
                                sprite_name, comp.name
                            )));
                            continue;
                        }
                    };

                    // Check for size mismatch (Task 2.5)
                    let sprite_width = sprite_image.width();
                    let sprite_height = sprite_image.height();
                    if sprite_width > cell_size[0] || sprite_height > cell_size[1] {
                        if strict {
                            return Err(CompositionError::SizeMismatch {
                                sprite_name: sprite_name.clone(),
                                sprite_size: (sprite_width, sprite_height),
                                cell_size: (cell_size[0], cell_size[1]),
                                composition_name: comp.name.clone(),
                            });
                        } else {
                            warnings.push(Warning::new(format!(
                                "Sprite '{}' ({}x{}) exceeds cell size ({}x{}) in composition '{}', anchoring from top-left",
                                sprite_name, sprite_width, sprite_height, cell_size[0], cell_size[1], comp.name
                            )));
                        }
                    }

                    // Calculate position
                    let x = (col_idx as u32) * cell_size[0];
                    let y = (row_idx as u32) * cell_size[1];

                    // Blit sprite onto canvas with blend mode and opacity (ATF-10)
                    blit_sprite_blended(&mut canvas, sprite_image, x, y, blend_mode, opacity);
                }
            }
        }
    }

    Ok((canvas, warnings))
}

/// Infer canvas size from layer maps and cell size
fn infer_size_from_layers(
    layers: &[crate::models::CompositionLayer],
    cell_size: [u32; 2],
) -> (u32, u32) {
    let mut max_cols = 0u32;
    let mut max_rows = 0u32;

    for layer in layers {
        if let Some(ref map) = layer.map {
            let rows = map.len() as u32;
            let cols = map.iter().map(|r| r.chars().count() as u32).max().unwrap_or(0);
            max_rows = max_rows.max(rows);
            max_cols = max_cols.max(cols);
        }
    }

    (max_cols * cell_size[0], max_rows * cell_size[1])
}

/// Blit a sprite onto the canvas at the given position.
/// Uses alpha blending for transparent pixels.
fn blit_sprite(canvas: &mut RgbaImage, sprite: &RgbaImage, x: u32, y: u32) {
    blit_sprite_blended(canvas, sprite, x, y, BlendMode::Normal, 1.0);
}

/// Blit a sprite onto the canvas with blend mode and opacity.
///
/// # Arguments
/// * `canvas` - The destination image
/// * `sprite` - The sprite to blit
/// * `x`, `y` - Position on canvas
/// * `blend_mode` - How to blend colors (ATF-10)
/// * `opacity` - Layer opacity (0.0-1.0)
fn blit_sprite_blended(
    canvas: &mut RgbaImage,
    sprite: &RgbaImage,
    x: u32,
    y: u32,
    blend_mode: BlendMode,
    opacity: f64,
) {
    let canvas_width = canvas.width();
    let canvas_height = canvas.height();
    let opacity = opacity.clamp(0.0, 1.0) as f32;

    for (sy, row) in sprite.rows().enumerate() {
        let dest_y = y + sy as u32;
        if dest_y >= canvas_height {
            break;
        }

        for (sx, pixel) in row.enumerate() {
            let dest_x = x + sx as u32;
            if dest_x >= canvas_width {
                break;
            }

            let src = pixel;
            // Fully transparent source, skip
            if src[3] == 0 {
                continue;
            }

            // Apply layer opacity to source alpha
            let src_alpha = (src[3] as f32 / 255.0) * opacity;
            if src_alpha == 0.0 {
                continue;
            }

            let dst = canvas.get_pixel(dest_x, dest_y);
            let blended = blend_pixels(src, dst, blend_mode, src_alpha);
            canvas.put_pixel(dest_x, dest_y, blended);
        }
    }
}

/// Blend source pixel over destination using the specified blend mode and opacity.
fn blend_pixels(src: &Rgba<u8>, dst: &Rgba<u8>, mode: BlendMode, src_alpha: f32) -> Rgba<u8> {
    let dst_alpha = dst[3] as f32 / 255.0;

    // Normalize colors to 0.0-1.0
    let src_r = src[0] as f32 / 255.0;
    let src_g = src[1] as f32 / 255.0;
    let src_b = src[2] as f32 / 255.0;

    let dst_r = dst[0] as f32 / 255.0;
    let dst_g = dst[1] as f32 / 255.0;
    let dst_b = dst[2] as f32 / 255.0;

    // Apply blend mode to get the blended color (before alpha compositing)
    let blended_r = mode.blend_channel(dst_r, src_r);
    let blended_g = mode.blend_channel(dst_g, src_g);
    let blended_b = mode.blend_channel(dst_b, src_b);

    // Now composite the blended color over destination using porter-duff "source over"
    // out_alpha = src_alpha + dst_alpha * (1 - src_alpha)
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

    if out_alpha == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }

    // Composite each channel:
    // out_color = (blended_color * src_alpha + dst_color * dst_alpha * (1 - src_alpha)) / out_alpha
    let composite = |blended: f32, dst: f32| -> u8 {
        let result = (blended * src_alpha + dst * dst_alpha * (1.0 - src_alpha)) / out_alpha;
        (result.clamp(0.0, 1.0) * 255.0).round() as u8
    };

    Rgba([
        composite(blended_r, dst_r),
        composite(blended_g, dst_g),
        composite(blended_b, dst_b),
        (out_alpha * 255.0).round() as u8,
    ])
}

/// Alpha blend source over destination (test helper)
#[cfg(test)]
fn alpha_blend(src: &Rgba<u8>, dst: &Rgba<u8>) -> Rgba<u8> {
    let src_a = src[3] as f32 / 255.0;
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);

    if out_a == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }

    let blend = |s: u8, d: u8| -> u8 {
        let s_f = s as f32 / 255.0;
        let d_f = d as f32 / 255.0;
        let out = (s_f * src_a + d_f * dst_a * (1.0 - src_a)) / out_a;
        (out * 255.0).round() as u8
    };

    Rgba([
        blend(src[0], dst[0]),
        blend(src[1], dst[1]),
        blend(src[2], dst[2]),
        (out_a * 255.0).round() as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Composition, CompositionLayer};

    #[test]
    fn test_render_empty_composition() {
        let comp = Composition {
            name: "empty".to_string(),
            base: None,
            size: Some([8, 8]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };
        let sprites = HashMap::new();

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert_eq!(image.width(), 8);
        assert_eq!(image.height(), 8);
        assert!(warnings.is_empty());
        // All pixels should be transparent
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_render_single_layer_composition() {
        let comp = Composition {
            name: "single_layer".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("red_pixel".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: Some("main".to_string()),
                fill: None,
                map: Some(vec!["X.".to_string(), ".X".to_string()]),
                ..Default::default()
            }],
        };

        // Create a 1x1 red sprite
        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("red_pixel".to_string(), red_sprite)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);

        // Check diagonal pattern: X at (0,0) and (1,1), transparent at (1,0) and (0,1)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255])); // X
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 0, 0, 0])); // .
        assert_eq!(*image.get_pixel(0, 1), Rgba([0, 0, 0, 0])); // .
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255])); // X
    }

    #[test]
    fn test_infer_size_from_layers() {
        let layers = vec![CompositionLayer {
            name: None,
            fill: None,
            map: Some(vec!["ABC".to_string(), "DEF".to_string()]),
            ..Default::default()
        }];

        let (width, height) = infer_size_from_layers(&layers, [1, 1]);
        assert_eq!(width, 3);
        assert_eq!(height, 2);

        // With cell_size [4, 4]
        let (width, height) = infer_size_from_layers(&layers, [4, 4]);
        assert_eq!(width, 12);
        assert_eq!(height, 8);
    }

    #[test]
    fn test_unknown_sprite_key_warning() {
        let comp = Composition {
            name: "test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: None,
            sprites: HashMap::new(), // Empty - no keys defined
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X".to_string()]),
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("Unknown sprite key"));
    }

    #[test]
    fn test_missing_sprite_warning() {
        let comp = Composition {
            name: "test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: None,
            sprites: HashMap::from([("X".to_string(), Some("missing_sprite".to_string()))]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X".to_string()]),
                ..Default::default()
            }],
        };

        // Empty sprites map - sprite not provided
        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("not found"));
    }

    #[test]
    fn test_alpha_blend() {
        // Opaque over transparent
        let src = Rgba([255, 0, 0, 255]);
        let dst = Rgba([0, 0, 0, 0]);
        let result = alpha_blend(&src, &dst);
        assert_eq!(result, Rgba([255, 0, 0, 255]));

        // Semi-transparent over opaque
        let src = Rgba([255, 0, 0, 128]); // ~50% red
        let dst = Rgba([0, 0, 255, 255]); // 100% blue
        let result = alpha_blend(&src, &dst);
        // Result should be roughly purple
        assert!(result[0] > 100); // Some red
        assert!(result[2] > 100); // Some blue
        assert_eq!(result[3], 255); // Fully opaque
    }

    #[test]
    fn test_cell_size_default() {
        let comp = Composition {
            name: "no_cell_size".to_string(),
            base: None,
            size: None,
            cell_size: None, // Should default to [1, 1]
            sprites: HashMap::from([("X".to_string(), Some("pixel".to_string()))]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["XX".to_string(), "XX".to_string()]),
                ..Default::default()
            }],
        };

        let mut pixel = RgbaImage::new(1, 1);
        pixel.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (image, _) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should infer 2x2 from map
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);
    }

    #[test]
    fn test_render_two_layers_stack() {
        // Layer 1: red at (0,0), transparent elsewhere
        // Layer 2: blue at (0,0), transparent elsewhere
        // Result: blue at (0,0) because layer 2 is on top
        let comp = Composition {
            name: "two_layers".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("R".to_string(), Some("red_pixel".to_string())),
                ("B".to_string(), Some("blue_pixel".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("bottom".to_string()),
                    fill: None,
                    map: Some(vec!["R.".to_string(), "..".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("top".to_string()),
                    fill: None,
                    map: Some(vec!["B.".to_string(), "..".to_string()]),
                    ..Default::default()
                },
            ],
        };

        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let mut blue_sprite = RgbaImage::new(1, 1);
        blue_sprite.put_pixel(0, 0, Rgba([0, 0, 255, 255]));

        let sprites = HashMap::from([
            ("red_pixel".to_string(), red_sprite),
            ("blue_pixel".to_string(), blue_sprite),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // (0,0) should be blue (layer 2 overwrites layer 1)
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        // Other pixels should be transparent
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(0, 1), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(1, 1), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_render_two_layers_different_positions() {
        // Layer 1: red at (0,0)
        // Layer 2: blue at (1,1)
        // Result: both visible at their respective positions
        let comp = Composition {
            name: "two_layers_positions".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("R".to_string(), Some("red_pixel".to_string())),
                ("B".to_string(), Some("blue_pixel".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("bottom".to_string()),
                    fill: None,
                    map: Some(vec!["R.".to_string(), "..".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("top".to_string()),
                    fill: None,
                    map: Some(vec!["..".to_string(), ".B".to_string()]),
                    ..Default::default()
                },
            ],
        };

        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let mut blue_sprite = RgbaImage::new(1, 1);
        blue_sprite.put_pixel(0, 0, Rgba([0, 0, 255, 255]));

        let sprites = HashMap::from([
            ("red_pixel".to_string(), red_sprite),
            ("blue_pixel".to_string(), blue_sprite),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // (0,0) should be red (from layer 1)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        // (1,1) should be blue (from layer 2)
        assert_eq!(*image.get_pixel(1, 1), Rgba([0, 0, 255, 255]));
        // Other pixels should be transparent
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(0, 1), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_render_three_layers_stack() {
        // Layer 1: red across all
        // Layer 2: green at (0,0) and (1,0)
        // Layer 3: blue at (0,0) only
        // Result: blue at (0,0), green at (1,0), red at (0,1) and (1,1)
        let comp = Composition {
            name: "three_layers".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("R".to_string(), Some("red_pixel".to_string())),
                ("G".to_string(), Some("green_pixel".to_string())),
                ("B".to_string(), Some("blue_pixel".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("layer1".to_string()),
                    fill: None,
                    map: Some(vec!["RR".to_string(), "RR".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("layer2".to_string()),
                    fill: None,
                    map: Some(vec!["GG".to_string(), "..".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("layer3".to_string()),
                    fill: None,
                    map: Some(vec!["B.".to_string(), "..".to_string()]),
                    ..Default::default()
                },
            ],
        };

        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let mut green_sprite = RgbaImage::new(1, 1);
        green_sprite.put_pixel(0, 0, Rgba([0, 255, 0, 255]));

        let mut blue_sprite = RgbaImage::new(1, 1);
        blue_sprite.put_pixel(0, 0, Rgba([0, 0, 255, 255]));

        let sprites = HashMap::from([
            ("red_pixel".to_string(), red_sprite),
            ("green_pixel".to_string(), green_sprite),
            ("blue_pixel".to_string(), blue_sprite),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // (0,0): red -> green -> blue = blue
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        // (1,0): red -> green = green
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 255, 0, 255]));
        // (0,1): red only
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
        // (1,1): red only
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_all_dots_layer_renders_nothing() {
        // Layer with all "." should not affect the canvas
        let comp = Composition {
            name: "dots_layer".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("R".to_string(), Some("red_pixel".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("background".to_string()),
                    fill: None,
                    map: Some(vec!["RR".to_string(), "RR".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("empty".to_string()),
                    fill: None,
                    map: Some(vec!["..".to_string(), "..".to_string()]),
                    ..Default::default()
                },
            ],
        };

        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("red_pixel".to_string(), red_sprite)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // All pixels should still be red (empty layer didn't erase anything)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
    }

    // Task 2.5: Size Mismatch Handling tests

    #[test]
    fn test_sprite_fits_cell_no_warning() {
        // Sprite exactly fits cell - no warning
        let comp = Composition {
            name: "exact_fit".to_string(),
            base: None,
            size: Some([4, 4]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("pixel".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X.".to_string(), ".X".to_string()]),
                ..Default::default()
            }],
        };

        // 2x2 sprite exactly fits 2x2 cell
        let mut pixel = RgbaImage::new(2, 2);
        pixel.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        pixel.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
        pixel.put_pixel(0, 1, Rgba([255, 0, 0, 255]));
        pixel.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (_, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // No size mismatch warnings
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_sprite_smaller_than_cell_no_warning() {
        // Sprite smaller than cell - no warning
        let comp = Composition {
            name: "small_fit".to_string(),
            base: None,
            size: Some([4, 4]),
            cell_size: Some([4, 4]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("pixel".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X".to_string()]),
                ..Default::default()
            }],
        };

        // 2x2 sprite fits in 4x4 cell
        let mut pixel = RgbaImage::new(2, 2);
        pixel.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (_, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
    }

    #[test]
    fn test_sprite_larger_than_cell_lenient_warning() {
        // Sprite larger than cell - warning in lenient mode
        let comp = Composition {
            name: "oversized".to_string(),
            base: None,
            size: Some([8, 8]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("big_sprite".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![
                    "X...".to_string(),
                    "....".to_string(),
                    "....".to_string(),
                    "....".to_string(),
                ]),
                ..Default::default()
            }],
        };

        // 4x4 sprite exceeds 2x2 cell
        let mut big_sprite = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                big_sprite.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let sprites = HashMap::from([("big_sprite".to_string(), big_sprite)]);

        let result = render_composition(&comp, &sprites, false, None);

        // Should succeed in lenient mode
        assert!(result.is_ok());
        let (image, warnings) = result.unwrap();

        // Should have a warning
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("exceeds cell size"));
        assert!(warnings[0].message.contains("big_sprite"));
        assert!(warnings[0].message.contains("4x4"));
        assert!(warnings[0].message.contains("2x2"));

        // Sprite should still render (anchored top-left, overwriting adjacent cells)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(3, 3), Rgba([255, 0, 0, 255])); // Overflows into adjacent cells
    }

    #[test]
    fn test_sprite_larger_than_cell_strict_error() {
        // Sprite larger than cell - error in strict mode
        let comp = Composition {
            name: "oversized_strict".to_string(),
            base: None,
            size: Some([8, 8]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("big_sprite".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![
                    "X...".to_string(),
                    "....".to_string(),
                    "....".to_string(),
                    "....".to_string(),
                ]),
                ..Default::default()
            }],
        };

        // 4x4 sprite exceeds 2x2 cell
        let mut big_sprite = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                big_sprite.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let sprites = HashMap::from([("big_sprite".to_string(), big_sprite)]);

        let result = render_composition(&comp, &sprites, true, None);

        // Should fail in strict mode
        assert!(result.is_err());
        let err = result.unwrap_err();

        match err {
            CompositionError::SizeMismatch {
                sprite_name,
                sprite_size,
                cell_size,
                composition_name,
            } => {
                assert_eq!(sprite_name, "big_sprite");
                assert_eq!(sprite_size, (4, 4));
                assert_eq!(cell_size, (2, 2));
                assert_eq!(composition_name, "oversized_strict");
            }
            _ => panic!("Expected SizeMismatch error, got {:?}", err),
        }
    }

    #[test]
    fn test_large_sprite_overwrites_from_topleft() {
        // Large sprite anchors from top-left and overwrites adjacent cells
        let comp = Composition {
            name: "topleft_anchor".to_string(),
            base: None,
            size: Some([6, 6]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("big_sprite".to_string())),
                ("B".to_string(), Some("blue".to_string())),
            ]),
            layers: vec![
                // First layer: blue everywhere
                CompositionLayer {
                    name: Some("background".to_string()),
                    fill: None,
                    map: Some(vec!["BBB".to_string(), "BBB".to_string(), "BBB".to_string()]),
                    ..Default::default()
                },
                // Second layer: big red sprite at (0,0)
                CompositionLayer {
                    name: Some("foreground".to_string()),
                    fill: None,
                    map: Some(vec!["X..".to_string(), "...".to_string(), "...".to_string()]),
                    ..Default::default()
                },
            ],
        };

        // 4x4 red sprite
        let mut big_sprite = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                big_sprite.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        // 2x2 blue sprite
        let mut blue = RgbaImage::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                blue.put_pixel(x, y, Rgba([0, 0, 255, 255]));
            }
        }

        let sprites =
            HashMap::from([("big_sprite".to_string(), big_sprite), ("blue".to_string(), blue)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should have 1 size mismatch warning
        assert_eq!(warnings.len(), 1);

        // Top-left 4x4 area should be red (big sprite)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(3, 3), Rgba([255, 0, 0, 255]));

        // Area beyond the big sprite should still be blue
        assert_eq!(*image.get_pixel(4, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(0, 4), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(5, 5), Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn test_width_only_exceeds_cell() {
        // Only width exceeds cell - should warn
        let comp = Composition {
            name: "wide".to_string(),
            base: None,
            size: Some([8, 4]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("wide_sprite".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X...".to_string(), "....".to_string()]),
                ..Default::default()
            }],
        };

        // 4x2 sprite (width exceeds, height fits)
        let mut wide_sprite = RgbaImage::new(4, 2);
        for y in 0..2 {
            for x in 0..4 {
                wide_sprite.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let sprites = HashMap::from([("wide_sprite".to_string(), wide_sprite)]);

        let (_, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("4x2"));
    }

    #[test]
    fn test_height_only_exceeds_cell() {
        // Only height exceeds cell - should warn
        let comp = Composition {
            name: "tall".to_string(),
            base: None,
            size: Some([4, 8]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("tall_sprite".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![
                    "X.".to_string(),
                    "..".to_string(),
                    "..".to_string(),
                    "..".to_string(),
                ]),
                ..Default::default()
            }],
        };

        // 2x4 sprite (width fits, height exceeds)
        let mut tall_sprite = RgbaImage::new(2, 4);
        for y in 0..4 {
            for x in 0..2 {
                tall_sprite.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let sprites = HashMap::from([("tall_sprite".to_string(), tall_sprite)]);

        let (_, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("2x4"));
    }

    #[test]
    fn test_multiple_size_mismatches_lenient() {
        // Multiple sprites with size mismatches - all should warn in lenient mode
        let comp = Composition {
            name: "multi_mismatch".to_string(),
            base: None,
            size: Some([8, 8]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("A".to_string(), Some("big_a".to_string())),
                ("B".to_string(), Some("big_b".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![
                    "A...".to_string(),
                    "....".to_string(),
                    "..B.".to_string(),
                    "....".to_string(),
                ]),
                ..Default::default()
            }],
        };

        let mut big_a = RgbaImage::new(3, 3);
        let mut big_b = RgbaImage::new(4, 4);

        for y in 0..3 {
            for x in 0..3 {
                big_a.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                big_b.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }

        let sprites = HashMap::from([("big_a".to_string(), big_a), ("big_b".to_string(), big_b)]);

        let (_, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should have 2 warnings
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_size_mismatch_error_display() {
        let err = CompositionError::SizeMismatch {
            sprite_name: "test_sprite".to_string(),
            sprite_size: (10, 20),
            cell_size: (5, 5),
            composition_name: "test_comp".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("test_sprite"));
        assert!(msg.contains("10x20"));
        assert!(msg.contains("5x5"));
        assert!(msg.contains("test_comp"));
    }

    // ========== Task 2.3: Cell Size Scaling Tests ==========

    #[test]
    fn test_cell_size_1x1_pixel_perfect_overlay() {
        // cell_size [1, 1] should place sprites at exact pixel positions
        // This is the pixel-perfect overlay mode
        let comp = Composition {
            name: "pixel_perfect".to_string(),
            base: None,
            size: Some([4, 4]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("R".to_string(), Some("red".to_string())),
                ("G".to_string(), Some("green".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: Some("overlay".to_string()),
                fill: None,
                map: Some(vec![
                    "R.G.".to_string(),
                    ".RG.".to_string(),
                    "..RG".to_string(),
                    "...R".to_string(),
                ]),
                ..Default::default()
            }],
        };

        // 1x1 pixel sprites
        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let mut green = RgbaImage::new(1, 1);
        green.put_pixel(0, 0, Rgba([0, 255, 0, 255]));

        let sprites = HashMap::from([("red".to_string(), red), ("green".to_string(), green)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 4);

        // Check diagonal pattern
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255])); // R at (0,0)
        assert_eq!(*image.get_pixel(2, 0), Rgba([0, 255, 0, 255])); // G at (2,0)
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255])); // R at (1,1)
        assert_eq!(*image.get_pixel(2, 1), Rgba([0, 255, 0, 255])); // G at (2,1)
        assert_eq!(*image.get_pixel(3, 3), Rgba([255, 0, 0, 255])); // R at (3,3)
                                                                    // Transparent pixels
        assert_eq!(*image.get_pixel(1, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(0, 3), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_cell_size_4x4_grid_cells() {
        // cell_size [4, 4] means each grid character = 4x4 pixel area
        let comp = Composition {
            name: "4x4_grid".to_string(),
            base: None,
            size: Some([16, 16]), // 4 cells x 4 cells = 16x16 pixels
            cell_size: Some([4, 4]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("A".to_string(), Some("tile_a".to_string())),
                ("B".to_string(), Some("tile_b".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: Some("tiles".to_string()),
                fill: None,
                map: Some(vec![
                    "AB..".to_string(),
                    "BA..".to_string(),
                    "....".to_string(),
                    "..AB".to_string(),
                ]),
                ..Default::default()
            }],
        };

        // 4x4 pixel tiles
        let mut tile_a = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                tile_a.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let mut tile_b = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                tile_b.put_pixel(x, y, Rgba([0, 0, 255, 255]));
            }
        }

        let sprites =
            HashMap::from([("tile_a".to_string(), tile_a), ("tile_b".to_string(), tile_b)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 16);

        // Row 0: A at (0,0), B at (4,0)
        // Tile A occupies pixels (0-3, 0-3)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(3, 3), Rgba([255, 0, 0, 255]));
        // Tile B occupies pixels (4-7, 0-3)
        assert_eq!(*image.get_pixel(4, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(7, 3), Rgba([0, 0, 255, 255]));

        // Row 1: B at (0,4), A at (4,4)
        assert_eq!(*image.get_pixel(0, 4), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(4, 4), Rgba([255, 0, 0, 255]));

        // Row 3: A at (8,12), B at (12,12)
        assert_eq!(*image.get_pixel(8, 12), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(12, 12), Rgba([0, 0, 255, 255]));

        // Empty cells should be transparent
        assert_eq!(*image.get_pixel(8, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(0, 8), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_cell_size_16x16_tile_based_scene() {
        // cell_size [16, 16] for tile-based game scenes
        let comp = Composition {
            name: "tile_scene".to_string(),
            base: None,
            size: Some([48, 32]), // 3x2 tiles = 48x32 pixels
            cell_size: Some([16, 16]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("G".to_string(), Some("grass".to_string())),
                ("W".to_string(), Some("water".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: Some("terrain".to_string()),
                fill: None,
                map: Some(vec!["GGW".to_string(), "GWW".to_string()]),
                ..Default::default()
            }],
        };

        // 16x16 pixel tiles
        let mut grass = RgbaImage::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                grass.put_pixel(x, y, Rgba([0, 128, 0, 255])); // Green
            }
        }

        let mut water = RgbaImage::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                water.put_pixel(x, y, Rgba([0, 0, 200, 255])); // Blue
            }
        }

        let sprites = HashMap::from([("grass".to_string(), grass), ("water".to_string(), water)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 48);
        assert_eq!(image.height(), 32);

        // Row 0: G at (0,0), G at (16,0), W at (32,0)
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 128, 0, 255])); // Grass
        assert_eq!(*image.get_pixel(16, 0), Rgba([0, 128, 0, 255])); // Grass
        assert_eq!(*image.get_pixel(32, 0), Rgba([0, 0, 200, 255])); // Water

        // Row 1: G at (0,16), W at (16,16), W at (32,16)
        assert_eq!(*image.get_pixel(0, 16), Rgba([0, 128, 0, 255])); // Grass
        assert_eq!(*image.get_pixel(16, 16), Rgba([0, 0, 200, 255])); // Water
        assert_eq!(*image.get_pixel(32, 16), Rgba([0, 0, 200, 255])); // Water

        // Check tile boundaries
        assert_eq!(*image.get_pixel(15, 15), Rgba([0, 128, 0, 255])); // Last pixel of (0,0) grass
        assert_eq!(*image.get_pixel(47, 31), Rgba([0, 0, 200, 255])); // Last pixel of (2,1) water
    }

    #[test]
    fn test_cell_size_asymmetric() {
        // Non-square cell size: [8, 4]
        let comp = Composition {
            name: "asymmetric".to_string(),
            base: None,
            size: Some([24, 12]), // 3 cols x 3 rows with asymmetric cells
            cell_size: Some([8, 4]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("wide_tile".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X.X".to_string(), "...".to_string(), "X.X".to_string()]),
                ..Default::default()
            }],
        };

        // 8x4 wide tile
        let mut wide_tile = RgbaImage::new(8, 4);
        for y in 0..4 {
            for x in 0..8 {
                wide_tile.put_pixel(x, y, Rgba([255, 128, 0, 255])); // Orange
            }
        }

        let sprites = HashMap::from([("wide_tile".to_string(), wide_tile)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 24);
        assert_eq!(image.height(), 12);

        // Tile at (0,0) - covers pixels (0-7, 0-3)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 128, 0, 255]));
        assert_eq!(*image.get_pixel(7, 3), Rgba([255, 128, 0, 255]));

        // Tile at (2,0) - covers pixels (16-23, 0-3)
        assert_eq!(*image.get_pixel(16, 0), Rgba([255, 128, 0, 255]));
        assert_eq!(*image.get_pixel(23, 3), Rgba([255, 128, 0, 255]));

        // Empty middle column at x=8-15
        assert_eq!(*image.get_pixel(8, 0), Rgba([0, 0, 0, 0]));

        // Tile at (0,2) - covers pixels (0-7, 8-11)
        assert_eq!(*image.get_pixel(0, 8), Rgba([255, 128, 0, 255]));
        assert_eq!(*image.get_pixel(7, 11), Rgba([255, 128, 0, 255]));
    }

    // ========== Size Inference Tests ==========

    #[test]
    fn test_size_inference_from_base_sprite() {
        // When size is not specified but base is, use base sprite dimensions
        let comp = Composition {
            name: "base_inference".to_string(),
            base: Some("hero".to_string()),
            size: None, // No explicit size
            cell_size: Some([4, 4]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("H".to_string(), Some("hat".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                // Map 2x2 cells = 8x8 pixels with cell_size 4x4
                map: Some(vec!["H.".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        // Base sprite is 8x8 (2x2 cells with cell_size 4x4)
        let mut hero = RgbaImage::new(8, 8);
        for y in 0..8 {
            for x in 0..8 {
                hero.put_pixel(x, y, Rgba([100, 100, 100, 255])); // Gray
            }
        }

        // Hat is 4x4
        let mut hat = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                hat.put_pixel(x, y, Rgba([255, 0, 0, 255])); // Red
            }
        }

        let sprites = HashMap::from([("hero".to_string(), hero), ("hat".to_string(), hat)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // Canvas size should be inferred from base sprite (8x8)
        assert_eq!(image.width(), 8);
        assert_eq!(image.height(), 8);

        // Base sprite should be rendered first
        assert_eq!(*image.get_pixel(4, 4), Rgba([100, 100, 100, 255]));

        // Hat should be overlaid at (0,0)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(3, 3), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_size_inference_priority_explicit_over_base() {
        // Explicit size should take priority over base sprite
        let comp = Composition {
            name: "explicit_priority".to_string(),
            base: Some("base".to_string()),
            size: Some([10, 10]), // Explicit size different from base
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };

        // Base sprite is 32x32, but explicit size is 10x10
        let mut base = RgbaImage::new(32, 32);
        base.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("base".to_string(), base)]);

        let (image, _) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should use explicit size, not base size
        assert_eq!(image.width(), 10);
        assert_eq!(image.height(), 10);
    }

    #[test]
    fn test_size_inference_priority_base_over_layers() {
        // Base sprite size should take priority over layer inference
        // This test uses a base sprite size that differs from what the layer map would suggest
        // With validation, this generates a map dimension warning (expected in lenient mode)
        let comp = Composition {
            name: "base_over_layers".to_string(),
            base: Some("background".to_string()),
            size: None, // No explicit size
            cell_size: Some([4, 4]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("tile".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                // Layer map would infer 8x8 (2x2 cells * 4x4 cell_size)
                // But base sprite is 16x20, so expected grid is 4x5
                map: Some(vec!["X.".to_string(), ".X".to_string()]),
                ..Default::default()
            }],
        };

        // Background sprite is 16x20 (different from layer inference of 8x8)
        let mut background = RgbaImage::new(16, 20);
        for y in 0..20 {
            for x in 0..16 {
                background.put_pixel(x, y, Rgba([50, 50, 50, 255]));
            }
        }

        let mut tile = RgbaImage::new(4, 4);
        tile.put_pixel(0, 0, Rgba([255, 255, 0, 255]));

        let sprites =
            HashMap::from([("background".to_string(), background), ("tile".to_string(), tile)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // The key assertion: base sprite size (16x20) takes priority over layer-inferred (8x8)
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 20);

        // With validation, we expect a map dimension warning since map is 2x2 but expected is 4x5
        let dim_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("don't match expected")).collect();
        assert_eq!(dim_warnings.len(), 1);
    }

    #[test]
    fn test_size_inference_from_layers_with_cell_size() {
        // When no explicit size and no base, infer from layers + cell_size
        let comp = Composition {
            name: "layer_inference".to_string(),
            base: None,
            size: None, // No explicit size
            cell_size: Some([8, 8]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("tile".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                // 3 cols x 2 rows = 24x16 with cell_size 8x8
                map: Some(vec!["X.X".to_string(), ".X.".to_string()]),
                ..Default::default()
            }],
        };

        let mut tile = RgbaImage::new(8, 8);
        tile.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("tile".to_string(), tile)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // Inferred: 3 cols * 8 = 24, 2 rows * 8 = 16
        assert_eq!(image.width(), 24);
        assert_eq!(image.height(), 16);
    }

    #[test]
    fn test_missing_base_sprite_warning() {
        // When base is specified but not found, should warn and continue
        let comp = Composition {
            name: "missing_base".to_string(),
            base: Some("nonexistent".to_string()),
            size: None,
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("tile".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["XX".to_string()]),
                ..Default::default()
            }],
        };

        let mut tile = RgbaImage::new(2, 2);
        tile.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("tile".to_string(), tile)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should have warning about missing base
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("Base sprite 'nonexistent' not found"));

        // Should still render with size inferred from layers
        assert_eq!(image.width(), 4); // 2 cells * 2 cell_size
        assert_eq!(image.height(), 2); // 1 row * 2 cell_size
    }

    #[test]
    fn test_base_sprite_rendered_as_background() {
        // Base sprite should be rendered first, then layers on top
        let comp = Composition {
            name: "base_background".to_string(),
            base: Some("bg".to_string()),
            size: Some([4, 4]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("overlay".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                // Overlay at (0,0) only
                map: Some(vec!["X.".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        // Blue 4x4 background
        let mut bg = RgbaImage::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                bg.put_pixel(x, y, Rgba([0, 0, 255, 255]));
            }
        }

        // Red 2x2 overlay
        let mut overlay = RgbaImage::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                overlay.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let sprites = HashMap::from([("bg".to_string(), bg), ("overlay".to_string(), overlay)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());

        // Top-left 2x2 should be red (overlay)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));

        // Rest should be blue (background showing through)
        assert_eq!(*image.get_pixel(2, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(0, 2), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(3, 3), Rgba([0, 0, 255, 255]));
    }

    // ========== Task 2.6: Variant in Composition Test ==========

    #[test]
    fn test_variant_usable_in_composition() {
        // Verify that a variant can be used in a composition's sprites map
        // just like a regular sprite
        use crate::models::{PaletteRef, Sprite, Variant};
        use crate::registry::{PaletteRegistry, SpriteRegistry};
        use crate::renderer::render_resolved;

        // Create base sprite and variant
        let base_sprite = Sprite {
            name: "hero".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{skin}".to_string(), "#FFCC99".to_string()), // Original skin
            ])),
            grid: vec!["{_}{skin}".to_string(), "{skin}{_}".to_string()],
            metadata: None,
            ..Default::default()
        };

        let variant = Variant {
            name: "hero_red".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([
                ("{skin}".to_string(), "#FF0000".to_string()), // Red skin
            ]),
            ..Default::default()
        };

        // Build registries and resolve
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(base_sprite);
        sprite_registry.register_variant(variant);

        // Render both base and variant
        let hero_resolved = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        let variant_resolved =
            sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();

        let (hero_img, _) = render_resolved(&hero_resolved);
        let (variant_img, _) = render_resolved(&variant_resolved);

        // Build the composition that uses both
        let comp = Composition {
            name: "scene".to_string(),
            base: None,
            size: Some([4, 4]),
            cell_size: Some([2, 2]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("H".to_string(), Some("hero".to_string())),
                ("R".to_string(), Some("hero_red".to_string())), // Variant reference
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["HR".to_string(), "RH".to_string()]),
                ..Default::default()
            }],
        };

        // Provide both the base sprite and variant as rendered images
        let sprites =
            HashMap::from([("hero".to_string(), hero_img), ("hero_red".to_string(), variant_img)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 4);

        // hero (original skin #FFCC99 = 255, 204, 153) at (0,0) and (2,2)
        // hero_red (red skin #FF0000) at (2,0) and (0,2)

        // (1, 0) is hero's skin pixel (from {_}{skin} grid, skin is at x=1)
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 204, 153, 255])); // Original skin

        // (3, 0) is hero_red's skin pixel
        assert_eq!(*image.get_pixel(3, 0), Rgba([255, 0, 0, 255])); // Red skin

        // (1, 2) is hero_red's skin pixel (at grid position (0, 1) * cell_size (2,2))
        assert_eq!(*image.get_pixel(1, 2), Rgba([255, 0, 0, 255])); // Red skin

        // (3, 2) is hero's skin pixel (at grid position (1, 1) * cell_size (2,2))
        assert_eq!(*image.get_pixel(3, 2), Rgba([255, 204, 153, 255])); // Original skin
    }

    // ========== Task 14.3: Tiling Validation Tests ==========

    #[test]
    fn test_size_divisible_by_cell_size_valid() {
        // Size 64x64 with cell_size 16x16 = 4x4 grid - valid
        let comp = Composition {
            name: "valid_grid".to_string(),
            base: None,
            size: Some([64, 64]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![
                    "....".to_string(),
                    "....".to_string(),
                    "....".to_string(),
                    "....".to_string(),
                ]),
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        // No divisibility warnings
        let div_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("not divisible")).collect();
        assert!(div_warnings.is_empty());
    }

    #[test]
    fn test_size_not_divisible_lenient_warning() {
        // Size 65x64 with cell_size 16x16 - width not divisible
        let comp = Composition {
            name: "invalid_width".to_string(),
            base: None,
            size: Some([65, 64]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["....".to_string()]),
                ..Default::default()
            }],
        };

        let result = render_composition(&comp, &HashMap::new(), false, None);

        // Should succeed in lenient mode
        assert!(result.is_ok());
        let (_, warnings) = result.unwrap();

        // Should have divisibility warning
        let div_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("not divisible")).collect();
        assert_eq!(div_warnings.len(), 1);
        assert!(div_warnings[0].message.contains("65x64"));
        assert!(div_warnings[0].message.contains("16x16"));
    }

    #[test]
    fn test_size_not_divisible_strict_error() {
        // Size 64x65 with cell_size 16x16 - height not divisible
        let comp = Composition {
            name: "invalid_height".to_string(),
            base: None,
            size: Some([64, 65]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["....".to_string()]),
                ..Default::default()
            }],
        };

        let result = render_composition(&comp, &HashMap::new(), true, None);

        // Should fail in strict mode
        assert!(result.is_err());
        let err = result.unwrap_err();

        match err {
            CompositionError::SizeNotDivisible { size, cell_size, composition_name } => {
                assert_eq!(size, (64, 65));
                assert_eq!(cell_size, (16, 16));
                assert_eq!(composition_name, "invalid_height");
            }
            _ => panic!("Expected SizeNotDivisible error"),
        }
    }

    #[test]
    fn test_map_dimensions_match_expected_grid() {
        // Size 32x32 with cell_size 16x16 = 2x2 grid
        // Map has 2x2 = valid
        let comp = Composition {
            name: "valid_map".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: Some("terrain".to_string()),
                fill: None,
                map: Some(vec!["..".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        // No dimension warnings
        let dim_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("don't match expected")).collect();
        assert!(dim_warnings.is_empty());
    }

    #[test]
    fn test_map_dimensions_mismatch_lenient_warning() {
        // Size 32x32 with cell_size 16x16 = expected 2x2 grid
        // Map has 3x2 = mismatch
        let comp = Composition {
            name: "map_mismatch".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: Some("terrain".to_string()),
                fill: None,
                map: Some(vec!["...".to_string(), "...".to_string()]),
                ..Default::default()
            }],
        };

        let result = render_composition(&comp, &HashMap::new(), false, None);

        // Should succeed in lenient mode
        assert!(result.is_ok());
        let (_, warnings) = result.unwrap();

        // Should have dimension warning
        let dim_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("don't match expected")).collect();
        assert_eq!(dim_warnings.len(), 1);
        assert!(dim_warnings[0].message.contains("3x2")); // actual
        assert!(dim_warnings[0].message.contains("2x2")); // expected
        assert!(dim_warnings[0].message.contains("layer 'terrain'"));
    }

    #[test]
    fn test_map_dimensions_mismatch_strict_error() {
        // Size 32x32 with cell_size 16x16 = expected 2x2 grid
        // Map has 2x3 = mismatch (too many rows)
        let comp = Composition {
            name: "map_rows_mismatch".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: Some("layer1".to_string()),
                fill: None,
                map: Some(vec!["..".to_string(), "..".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        let result = render_composition(&comp, &HashMap::new(), true, None);

        // Should fail in strict mode
        assert!(result.is_err());
        let err = result.unwrap_err();

        match err {
            CompositionError::MapDimensionMismatch {
                layer_name,
                actual_dimensions,
                expected_dimensions,
                composition_name,
            } => {
                assert_eq!(layer_name, Some("layer1".to_string()));
                assert_eq!(actual_dimensions, (2, 3)); // (cols, rows)
                assert_eq!(expected_dimensions, (2, 2));
                assert_eq!(composition_name, "map_rows_mismatch");
            }
            _ => panic!("Expected MapDimensionMismatch error"),
        }
    }

    #[test]
    fn test_map_dimensions_unnamed_layer() {
        // Unnamed layer should say "unnamed layer" in error/warning
        let comp = Composition {
            name: "unnamed_layer_test".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: Some([16, 16]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None, // No name
                fill: None,
                map: Some(vec!["...".to_string()]),
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        // Should have warning mentioning unnamed layer
        let dim_warnings: Vec<_> =
            warnings.iter().filter(|w| w.message.contains("unnamed layer")).collect();
        assert_eq!(dim_warnings.len(), 1);
    }

    #[test]
    fn test_cell_size_1x1_no_validation() {
        // With default cell_size [1,1], size divisibility doesn't apply
        // Size 65x65 with cell_size [1,1] should be fine
        let comp = Composition {
            name: "default_cell".to_string(),
            base: None,
            size: Some([65, 65]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![".".to_string()]), // Just 1 cell
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        // No divisibility or dimension warnings for cell_size [1,1]
        let validation_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| {
                w.message.contains("not divisible") || w.message.contains("don't match expected")
            })
            .collect();
        assert!(validation_warnings.is_empty());
    }

    #[test]
    fn test_cell_size_none_no_validation() {
        // When cell_size is None (defaults to [1,1]), no validation
        let comp = Composition {
            name: "no_cell_size".to_string(),
            base: None,
            size: Some([65, 65]),
            cell_size: None, // Defaults to [1,1]
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec![".".to_string()]),
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        // No validation warnings
        let validation_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| {
                w.message.contains("not divisible") || w.message.contains("don't match expected")
            })
            .collect();
        assert!(validation_warnings.is_empty());
    }

    #[test]
    fn test_size_not_divisible_error_display() {
        let err = CompositionError::SizeNotDivisible {
            size: (65, 64),
            cell_size: (16, 16),
            composition_name: "test_comp".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("65x64"));
        assert!(msg.contains("16x16"));
        assert!(msg.contains("test_comp"));
        assert!(msg.contains("not divisible"));
    }

    #[test]
    fn test_map_dimension_mismatch_error_display() {
        let err = CompositionError::MapDimensionMismatch {
            layer_name: Some("terrain".to_string()),
            actual_dimensions: (3, 2),
            expected_dimensions: (2, 2),
            composition_name: "test_comp".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("3x2"));
        assert!(msg.contains("2x2"));
        assert!(msg.contains("layer 'terrain'"));
        assert!(msg.contains("test_comp"));
    }

    #[test]
    fn test_map_dimension_mismatch_unnamed_error_display() {
        let err = CompositionError::MapDimensionMismatch {
            layer_name: None,
            actual_dimensions: (3, 2),
            expected_dimensions: (2, 2),
            composition_name: "test_comp".to_string(),
        };

        let msg = format!("{}", err);
        assert!(msg.contains("unnamed layer"));
    }

    // ========================================================================
    // Blend Mode Tests (ATF-10)
    // ========================================================================

    #[test]
    fn test_blend_mode_from_str() {
        assert_eq!(BlendMode::from_str("normal"), Some(BlendMode::Normal));
        assert_eq!(BlendMode::from_str("multiply"), Some(BlendMode::Multiply));
        assert_eq!(BlendMode::from_str("screen"), Some(BlendMode::Screen));
        assert_eq!(BlendMode::from_str("overlay"), Some(BlendMode::Overlay));
        assert_eq!(BlendMode::from_str("add"), Some(BlendMode::Add));
        assert_eq!(BlendMode::from_str("additive"), Some(BlendMode::Add));
        assert_eq!(BlendMode::from_str("subtract"), Some(BlendMode::Subtract));
        assert_eq!(BlendMode::from_str("difference"), Some(BlendMode::Difference));
        assert_eq!(BlendMode::from_str("darken"), Some(BlendMode::Darken));
        assert_eq!(BlendMode::from_str("lighten"), Some(BlendMode::Lighten));
        assert_eq!(BlendMode::from_str("NORMAL"), Some(BlendMode::Normal)); // case insensitive
        assert_eq!(BlendMode::from_str("unknown"), None);
    }

    #[test]
    fn test_blend_mode_multiply() {
        // Multiply: result = base * blend
        let mode = BlendMode::Multiply;
        // 0.5 * 0.5 = 0.25
        assert!((mode.blend_channel(0.5, 0.5) - 0.25).abs() < 0.01);
        // 1.0 * 0.5 = 0.5
        assert!((mode.blend_channel(1.0, 0.5) - 0.5).abs() < 0.01);
        // 0.0 * anything = 0.0
        assert!((mode.blend_channel(0.0, 1.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_screen() {
        // Screen: result = 1 - (1 - base) * (1 - blend)
        let mode = BlendMode::Screen;
        // 1 - (1 - 0.5) * (1 - 0.5) = 1 - 0.25 = 0.75
        assert!((mode.blend_channel(0.5, 0.5) - 0.75).abs() < 0.01);
        // Screen with white = white
        assert!((mode.blend_channel(0.5, 1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_add() {
        // Add: result = min(1, base + blend)
        let mode = BlendMode::Add;
        assert!((mode.blend_channel(0.3, 0.4) - 0.7).abs() < 0.01);
        // Clamped at 1.0
        assert!((mode.blend_channel(0.8, 0.5) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_subtract() {
        // Subtract: result = max(0, base - blend)
        let mode = BlendMode::Subtract;
        assert!((mode.blend_channel(0.7, 0.3) - 0.4).abs() < 0.01);
        // Clamped at 0.0
        assert!((mode.blend_channel(0.3, 0.7) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_difference() {
        // Difference: result = abs(base - blend)
        let mode = BlendMode::Difference;
        assert!((mode.blend_channel(0.7, 0.3) - 0.4).abs() < 0.01);
        assert!((mode.blend_channel(0.3, 0.7) - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_darken() {
        // Darken: result = min(base, blend)
        let mode = BlendMode::Darken;
        assert!((mode.blend_channel(0.7, 0.3) - 0.3).abs() < 0.01);
        assert!((mode.blend_channel(0.3, 0.7) - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_lighten() {
        // Lighten: result = max(base, blend)
        let mode = BlendMode::Lighten;
        assert!((mode.blend_channel(0.7, 0.3) - 0.7).abs() < 0.01);
        assert!((mode.blend_channel(0.3, 0.7) - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_blend_pixels_normal() {
        // Normal blend mode should be standard alpha compositing
        let src = Rgba([255, 0, 0, 255]); // Opaque red
        let dst = Rgba([0, 0, 255, 255]); // Opaque blue
        let result = blend_pixels(&src, &dst, BlendMode::Normal, 1.0);
        // Opaque source completely replaces
        assert_eq!(result, Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_blend_pixels_multiply() {
        // Multiply should darken
        let src = Rgba([255, 128, 0, 255]); // Orange
        let dst = Rgba([255, 255, 255, 255]); // White
        let result = blend_pixels(&src, &dst, BlendMode::Multiply, 1.0);
        // Multiplying with white gives the original color
        assert_eq!(result[0], 255); // R
        assert_eq!(result[1], 128); // G (approximately)
        assert_eq!(result[2], 0); // B
    }

    #[test]
    fn test_opacity_reduces_effect() {
        // At 50% opacity, the source color should be half-strength
        let src = Rgba([255, 0, 0, 255]); // Opaque red
        let dst = Rgba([0, 0, 255, 255]); // Opaque blue
        let result = blend_pixels(&src, &dst, BlendMode::Normal, 0.5);
        // Should be a blend of red and blue
        assert!(result[0] > 100 && result[0] < 200); // Some red
        assert!(result[2] > 100 && result[2] < 200); // Some blue
    }

    #[test]
    fn test_composition_with_multiply_blend() {
        // Test multiply blend mode in a composition
        let comp = Composition {
            name: "multiply_test".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None), // transparent
                ("W".to_string(), Some("white".to_string())),
                ("S".to_string(), Some("shadow".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("background".to_string()),
                    fill: None,
                    map: Some(vec!["WW".to_string(), "WW".to_string()]),
                    blend: None, // Normal
                    opacity: None,
                    transform: None,
                },
                CompositionLayer {
                    name: Some("shadow".to_string()),
                    fill: None,
                    map: Some(vec!["S.".to_string(), "..".to_string()]),
                    blend: Some("multiply".to_string()),
                    opacity: None,
                    transform: None,
                },
            ],
        };

        // White background
        let mut white = RgbaImage::new(1, 1);
        white.put_pixel(0, 0, Rgba([255, 255, 255, 255]));

        // Gray shadow
        let mut shadow = RgbaImage::new(1, 1);
        shadow.put_pixel(0, 0, Rgba([128, 128, 128, 255]));

        let sprites = HashMap::from([("white".to_string(), white), ("shadow".to_string(), shadow)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // (0,0) should be darkened (white * gray = gray)
        let pixel = image.get_pixel(0, 0);
        assert!(pixel[0] < 200); // Darkened red
        assert!(pixel[1] < 200); // Darkened green
        assert!(pixel[2] < 200); // Darkened blue
                                 // (1,0) should still be white
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_composition_with_opacity() {
        // Test layer opacity
        let comp = Composition {
            name: "opacity_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                ("B".to_string(), Some("blue".to_string())),
                ("R".to_string(), Some("red".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("base".to_string()),
                    fill: None,
                    map: Some(vec!["B".to_string()]),
                    blend: None,
                    opacity: None, // Full opacity
                    transform: None,
                },
                CompositionLayer {
                    name: Some("overlay".to_string()),
                    fill: None,
                    map: Some(vec!["R".to_string()]),
                    blend: None,
                    opacity: Some(VarOr::Value(0.5)), // 50% opacity
                    transform: None,
                },
            ],
        };

        let mut blue = RgbaImage::new(1, 1);
        blue.put_pixel(0, 0, Rgba([0, 0, 255, 255]));

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("blue".to_string(), blue), ("red".to_string(), red)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // Should be a mix of red and blue (purple-ish)
        let pixel = image.get_pixel(0, 0);
        assert!(pixel[0] > 100); // Has red
        assert!(pixel[2] > 100); // Has blue
        assert!(pixel[0] < 255); // Not fully red
        assert!(pixel[2] < 255); // Not fully blue
    }

    #[test]
    fn test_composition_with_add_blend() {
        // Test additive blend mode (good for glows)
        let comp = Composition {
            name: "add_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                ("B".to_string(), Some("blue".to_string())),
                ("R".to_string(), Some("red".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("base".to_string()),
                    fill: None,
                    map: Some(vec!["B".to_string()]),
                    blend: None,
                    opacity: None,
                    transform: None,
                },
                CompositionLayer {
                    name: Some("glow".to_string()),
                    fill: None,
                    map: Some(vec!["R".to_string()]),
                    blend: Some("add".to_string()),
                    opacity: None,
                    transform: None,
                },
            ],
        };

        let mut blue = RgbaImage::new(1, 1);
        blue.put_pixel(0, 0, Rgba([0, 0, 255, 255]));

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("blue".to_string(), blue), ("red".to_string(), red)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // Add mode: blue + red = magenta
        let pixel = image.get_pixel(0, 0);
        assert_eq!(pixel[0], 255); // Full red
        assert_eq!(pixel[2], 255); // Full blue
    }

    #[test]
    fn test_layer_default_blend_and_opacity() {
        // Test that layers without blend/opacity work correctly (use defaults)
        let comp = Composition {
            name: "defaults_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([("R".to_string(), Some("red".to_string()))]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["R".to_string()]),
                blend: None,   // Default: normal
                opacity: None, // Default: 1.0
                transform: None,
            }],
        };

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("red".to_string(), red)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());
        // Should be full red
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_unknown_blend_mode_uses_normal() {
        // Unknown blend mode should fall back to normal
        let comp = Composition {
            name: "unknown_blend".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([("R".to_string(), Some("red".to_string()))]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["R".to_string()]),
                blend: Some("invalid_blend_mode".to_string()),
                opacity: None,
                transform: None,
            }],
        };

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("red".to_string(), red)]);

        let (image, _) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should still work, falling back to normal blend
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    // ========================================================================
    // CSS Variable Tests (CSS-9)
    // ========================================================================

    #[test]
    fn test_resolve_blend_mode_with_var() {
        let mut registry = VariableRegistry::new();
        registry.define("--blend", "multiply");

        let (mode, warning) = resolve_blend_mode(Some("var(--blend)"), Some(&registry));
        assert_eq!(mode, BlendMode::Multiply);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_blend_mode_with_var_fallback() {
        let registry = VariableRegistry::new();

        // Missing variable with fallback
        let (mode, warning) = resolve_blend_mode(Some("var(--missing, screen)"), Some(&registry));
        assert_eq!(mode, BlendMode::Screen);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_blend_mode_undefined_var() {
        let registry = VariableRegistry::new();

        // Missing variable without fallback
        let (mode, warning) = resolve_blend_mode(Some("var(--undefined)"), Some(&registry));
        assert_eq!(mode, BlendMode::Normal);
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("Failed to resolve"));
    }

    #[test]
    fn test_resolve_blend_mode_no_registry() {
        // var() used but no registry provided
        let (mode, warning) = resolve_blend_mode(Some("var(--blend)"), None);
        assert_eq!(mode, BlendMode::Normal);
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("no variable registry"));
    }

    #[test]
    fn test_resolve_opacity_with_var() {
        let mut registry = VariableRegistry::new();
        registry.define("--opacity", "0.5");

        let var_opacity = VarOr::Var("var(--opacity)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), Some(&registry));
        assert!((opacity - 0.5).abs() < 0.001);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_opacity_with_var_fallback() {
        let registry = VariableRegistry::new();

        // Missing variable with numeric fallback
        let var_opacity = VarOr::Var("var(--missing, 0.75)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), Some(&registry));
        assert!((opacity - 0.75).abs() < 0.001);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_opacity_literal() {
        let registry = VariableRegistry::new();

        // Literal value (not var)
        let literal_opacity = VarOr::Value(0.3);
        let (opacity, warning) = resolve_opacity(Some(&literal_opacity), Some(&registry));
        assert!((opacity - 0.3).abs() < 0.001);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_opacity_clamps_values() {
        let mut registry = VariableRegistry::new();
        registry.define("--over", "2.0");
        registry.define("--under", "-0.5");

        // Value over 1.0 should clamp to 1.0
        let over = VarOr::Var("var(--over)".to_string());
        let (opacity, _) = resolve_opacity(Some(&over), Some(&registry));
        assert!((opacity - 1.0).abs() < 0.001);

        // Value under 0.0 should clamp to 0.0
        let under = VarOr::Var("var(--under)".to_string());
        let (opacity, _) = resolve_opacity(Some(&under), Some(&registry));
        assert!(opacity.abs() < 0.001);
    }

    #[test]
    fn test_resolve_opacity_invalid_number() {
        let mut registry = VariableRegistry::new();
        registry.define("--invalid", "not-a-number");

        let var_opacity = VarOr::Var("var(--invalid)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), Some(&registry));
        assert!((opacity - 1.0).abs() < 0.001); // Falls back to 1.0
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("not a valid number"));
    }

    #[test]
    fn test_resolve_opacity_no_registry() {
        let var_opacity = VarOr::Var("var(--opacity)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), None);
        assert!((opacity - 1.0).abs() < 0.001); // Falls back to 1.0
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("no variable registry"));
    }

    #[test]
    fn test_composition_with_var_opacity() {
        // Full integration test with composition rendering
        let mut registry = VariableRegistry::new();
        registry.define("--layer-opacity", "0.5");

        let comp = Composition {
            name: "var_opacity_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([("R".to_string(), Some("red".to_string()))]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["R".to_string()]),
                blend: None,
                opacity: Some(VarOr::Var("var(--layer-opacity)".to_string())),
                transform: None,
            }],
        };

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("red".to_string(), red)]);

        let (image, warnings) =
            render_composition(&comp, &sprites, false, Some(&registry)).unwrap();
        assert!(warnings.is_empty());

        // With 50% opacity on transparent background, the alpha should be 127/128
        let pixel = image.get_pixel(0, 0);
        assert_eq!(pixel[0], 255); // Red channel full
        assert!(pixel[3] < 200); // Alpha reduced due to opacity
    }

    #[test]
    fn test_composition_with_var_blend_mode() {
        let mut registry = VariableRegistry::new();
        registry.define("--layer-blend", "add");

        let comp = Composition {
            name: "var_blend_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([("R".to_string(), Some("red".to_string()))]),
            layers: vec![
                // Base layer
                CompositionLayer {
                    name: Some("base".to_string()),
                    fill: None,
                    map: Some(vec!["R".to_string()]),
                    blend: None,
                    opacity: Some(VarOr::Value(1.0)),
                    transform: None,
                },
                // Top layer with var blend
                CompositionLayer {
                    name: Some("top".to_string()),
                    fill: None,
                    map: Some(vec!["R".to_string()]),
                    blend: Some("var(--layer-blend)".to_string()),
                    opacity: Some(VarOr::Value(1.0)),
                    transform: None,
                },
            ],
        };

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([128, 0, 0, 255]));
        let sprites = HashMap::from([("red".to_string(), red)]);

        let (image, warnings) =
            render_composition(&comp, &sprites, false, Some(&registry)).unwrap();
        assert!(warnings.is_empty());

        // With additive blend mode, 128 + 128 = 255 (clamped)
        let pixel = image.get_pixel(0, 0);
        assert_eq!(pixel[0], 255); // Red should be clamped at 255
    }

    #[test]
    fn test_composition_var_without_registry_warns() {
        let comp = Composition {
            name: "no_registry_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([("R".to_string(), Some("red".to_string()))]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["R".to_string()]),
                blend: Some("var(--blend)".to_string()),
                opacity: Some(VarOr::Var("var(--opacity)".to_string())),
                transform: None,
            }],
        };

        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("red".to_string(), red)]);

        // No registry passed
        let (_, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should have warnings for both blend and opacity
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.message.contains("blend")));
        assert!(warnings.iter().any(|w| w.message.contains("Opacity")));
    }

    #[test]
    fn test_var_or_deserialization() {
        // Test that VarOr<f64> deserializes correctly from JSON
        use serde_json;

        // Number value
        let num: VarOr<f64> = serde_json::from_str("0.5").unwrap();
        assert!(matches!(num, VarOr::Value(v) if (v - 0.5).abs() < 0.001));

        // String var() reference
        let var: VarOr<f64> = serde_json::from_str(r#""var(--opacity)""#).unwrap();
        assert!(matches!(var, VarOr::Var(s) if s == "var(--opacity)"));

        // String var() with fallback
        let var_fb: VarOr<f64> = serde_json::from_str(r#""var(--opacity, 0.5)""#).unwrap();
        assert!(matches!(var_fb, VarOr::Var(s) if s == "var(--opacity, 0.5)"));
    }

    // ========== RenderContext Tests (NC-2, NC-3) ==========

    #[test]
    fn test_render_context_new() {
        let ctx = RenderContext::new();
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
        assert_eq!(ctx.depth(), 0);
        assert!(ctx.path().is_empty());
    }

    #[test]
    fn test_render_context_cache_and_get() {
        let mut ctx = RenderContext::new();

        // Create a test image
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        // Cache it
        ctx.cache("test_comp".to_string(), img);

        // Verify it's cached
        assert!(ctx.is_cached("test_comp"));
        assert_eq!(ctx.len(), 1);

        // Get it back
        let cached = ctx.get_cached("test_comp");
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.width(), 2);
        assert_eq!(cached.height(), 2);
        assert_eq!(*cached.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_context_get_cached_not_found() {
        let ctx = RenderContext::new();
        assert!(ctx.get_cached("nonexistent").is_none());
        assert!(!ctx.is_cached("nonexistent"));
    }

    #[test]
    fn test_render_context_cache_overwrites() {
        let mut ctx = RenderContext::new();

        // Cache first image (red)
        let mut img1 = RgbaImage::new(1, 1);
        img1.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        ctx.cache("comp".to_string(), img1);

        // Cache second image with same name (blue)
        let mut img2 = RgbaImage::new(1, 1);
        img2.put_pixel(0, 0, Rgba([0, 0, 255, 255]));
        ctx.cache("comp".to_string(), img2);

        // Should still have only one entry
        assert_eq!(ctx.len(), 1);

        // Should be the second (blue) image
        let cached = ctx.get_cached("comp").unwrap();
        assert_eq!(*cached.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn test_render_context_multiple_compositions() {
        let mut ctx = RenderContext::new();

        // Cache multiple compositions
        let mut img1 = RgbaImage::new(1, 1);
        img1.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        ctx.cache("red".to_string(), img1);

        let mut img2 = RgbaImage::new(1, 1);
        img2.put_pixel(0, 0, Rgba([0, 255, 0, 255]));
        ctx.cache("green".to_string(), img2);

        let mut img3 = RgbaImage::new(1, 1);
        img3.put_pixel(0, 0, Rgba([0, 0, 255, 255]));
        ctx.cache("blue".to_string(), img3);

        assert_eq!(ctx.len(), 3);
        assert!(ctx.is_cached("red"));
        assert!(ctx.is_cached("green"));
        assert!(ctx.is_cached("blue"));

        // Verify colors
        assert_eq!(*ctx.get_cached("red").unwrap().get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*ctx.get_cached("green").unwrap().get_pixel(0, 0), Rgba([0, 255, 0, 255]));
        assert_eq!(*ctx.get_cached("blue").unwrap().get_pixel(0, 0), Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn test_render_context_clear() {
        let mut ctx = RenderContext::new();

        // Cache some compositions
        ctx.cache("a".to_string(), RgbaImage::new(1, 1));
        ctx.cache("b".to_string(), RgbaImage::new(1, 1));
        assert_eq!(ctx.len(), 2);

        // Clear
        ctx.clear();
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
        assert!(!ctx.is_cached("a"));
        assert!(!ctx.is_cached("b"));
    }

    #[test]
    fn test_render_context_cache_hit_avoids_rerender() {
        // This test demonstrates the cache usage pattern
        let mut ctx = RenderContext::new();
        let mut render_count = 0;

        // Simulate rendering "scene" twice with cache
        for _ in 0..2 {
            if ctx.get_cached("scene").is_none() {
                // "Render" the composition (in reality this would call render_composition)
                render_count += 1;
                let mut img = RgbaImage::new(4, 4);
                img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
                ctx.cache("scene".to_string(), img);
            }
            // Use the cached version
            let _cached = ctx.get_cached("scene").unwrap();
        }

        // Should only have rendered once
        assert_eq!(render_count, 1);
    }

    #[test]
    fn test_render_context_push_pop_basic() {
        let mut ctx = RenderContext::new();

        // Push A
        assert!(ctx.push("A").is_ok());
        assert_eq!(ctx.depth(), 1);
        assert!(ctx.contains("A"));
        assert!(!ctx.is_empty());

        // Push B
        assert!(ctx.push("B").is_ok());
        assert_eq!(ctx.depth(), 2);
        assert!(ctx.contains("A"));
        assert!(ctx.contains("B"));

        // Push C
        assert!(ctx.push("C").is_ok());
        assert_eq!(ctx.depth(), 3);
        assert_eq!(ctx.path(), &["A", "B", "C"]);

        // Pop C
        assert_eq!(ctx.pop(), Some("C".to_string()));
        assert_eq!(ctx.depth(), 2);
        assert!(!ctx.contains("C"));

        // Pop B
        assert_eq!(ctx.pop(), Some("B".to_string()));
        assert_eq!(ctx.depth(), 1);

        // Pop A
        assert_eq!(ctx.pop(), Some("A".to_string()));
        assert!(ctx.is_empty());

        // Pop empty returns None
        assert_eq!(ctx.pop(), None);
    }

    #[test]
    fn test_render_context_a_b_c_renders_ok() {
        // A -> B -> C renders OK (no cycles)
        let mut ctx = RenderContext::new();

        assert!(ctx.push("A").is_ok());
        assert!(ctx.push("B").is_ok());
        assert!(ctx.push("C").is_ok());

        assert_eq!(ctx.depth(), 3);
        assert_eq!(ctx.path(), &["A", "B", "C"]);
    }

    #[test]
    fn test_render_context_cycle_a_b_a() {
        // A -> B -> A returns CycleDetected error
        let mut ctx = RenderContext::new();

        assert!(ctx.push("A").is_ok());
        assert!(ctx.push("B").is_ok());

        let result = ctx.push("A");
        assert!(result.is_err());

        match result {
            Err(CompositionError::CycleDetected { cycle_path }) => {
                assert_eq!(cycle_path, vec!["A", "B", "A"]);
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_render_context_self_reference_cycle() {
        // A -> A (self-reference) returns CycleDetected error
        let mut ctx = RenderContext::new();

        assert!(ctx.push("A").is_ok());

        let result = ctx.push("A");
        assert!(result.is_err());

        match result {
            Err(CompositionError::CycleDetected { cycle_path }) => {
                assert_eq!(cycle_path, vec!["A", "A"]);
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_render_context_longer_cycle() {
        // A -> B -> C -> D -> B returns CycleDetected error with path B -> C -> D -> B
        let mut ctx = RenderContext::new();

        assert!(ctx.push("A").is_ok());
        assert!(ctx.push("B").is_ok());
        assert!(ctx.push("C").is_ok());
        assert!(ctx.push("D").is_ok());

        let result = ctx.push("B");
        assert!(result.is_err());

        match result {
            Err(CompositionError::CycleDetected { cycle_path }) => {
                assert_eq!(cycle_path, vec!["B", "C", "D", "B"]);
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_render_context_cycle_error_message() {
        // Verify the error message format
        let mut ctx = RenderContext::new();
        ctx.push("scene").unwrap();
        ctx.push("background").unwrap();
        ctx.push("overlay").unwrap();

        let result = ctx.push("scene");
        let err = result.unwrap_err();
        let message = err.to_string();

        assert!(message.contains("Cycle detected"));
        assert!(message.contains("scene -> background -> overlay -> scene"));
    }

    #[test]
    fn test_render_context_reuse_after_pop() {
        // After popping, the same name can be pushed again
        let mut ctx = RenderContext::new();

        ctx.push("A").unwrap();
        ctx.push("B").unwrap();
        ctx.pop(); // Pop B

        // B can be pushed again
        assert!(ctx.push("B").is_ok());
        assert_eq!(ctx.path(), &["A", "B"]);
    }

    #[test]
    fn test_render_context_default() {
        // Default trait implementation
        let ctx: RenderContext = Default::default();
        assert!(ctx.is_empty());
        assert_eq!(ctx.depth(), 0);
    }

    #[test]
    fn test_render_context_clone() {
        let mut ctx = RenderContext::new();
        ctx.push("A").unwrap();
        ctx.push("B").unwrap();

        let cloned = ctx.clone();
        assert_eq!(cloned.depth(), 2);
        assert_eq!(cloned.path(), &["A", "B"]);

        // Original and clone are independent
        ctx.push("C").unwrap();
        assert_eq!(ctx.depth(), 3);
        assert_eq!(cloned.depth(), 2);
    }
}
