//! Composition rendering functions

use image::{Rgba, RgbaImage};
use std::collections::HashMap;

use crate::models::Composition;
use crate::registry::CompositionRegistry;
use crate::variables::VariableRegistry;

use super::blend::{blit_sprite, blit_sprite_blended};
use super::context::RenderContext;
use super::error::{CompositionError, Warning};
use super::resolve::{resolve_blend_mode, resolve_opacity};

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

/// Render a composition with support for nested composition references (NC-4).
///
/// This function extends `render_composition` to support compositions that reference
/// other compositions in their sprite maps. When a sprite name is not found in the
/// `sprites` HashMap, the function checks the `composition_registry` for a composition
/// with that name and recursively renders it.
///
/// # Cycle Detection
/// The function uses `RenderContext` to track which compositions are currently being
/// rendered. If a composition attempts to reference itself (directly or indirectly),
/// a `CycleDetected` error is returned.
///
/// # Caching
/// Rendered compositions are cached in the `RenderContext`. When the same composition
/// is referenced multiple times, the cached result is reused.
pub fn render_composition_nested(
    comp: &Composition,
    sprites: &HashMap<String, RgbaImage>,
    composition_registry: Option<&CompositionRegistry>,
    ctx: &mut RenderContext,
    strict: bool,
    variables: Option<&VariableRegistry>,
) -> Result<(RgbaImage, Vec<Warning>), CompositionError> {
    // Push this composition onto the render stack for cycle detection
    ctx.push(&comp.name)?;

    let result =
        render_composition_inner(comp, sprites, composition_registry, ctx, strict, variables);

    // Pop from render stack when done
    ctx.pop();

    result
}

/// Internal implementation of nested composition rendering.
fn render_composition_inner(
    comp: &Composition,
    sprites: &HashMap<String, RgbaImage>,
    composition_registry: Option<&CompositionRegistry>,
    ctx: &mut RenderContext,
    strict: bool,
    variables: Option<&VariableRegistry>,
) -> Result<(RgbaImage, Vec<Warning>), CompositionError> {
    let mut warnings = Vec::new();

    // Determine cell size (default to [1, 1])
    let cell_size = comp.cell_size.unwrap_or([1, 1]);

    // Look up base sprite/composition if specified (NC-4: supports nested compositions)
    let base_image: Option<std::borrow::Cow<'_, RgbaImage>> = if let Some(ref base_name) = comp.base
    {
        if let Some(img) = sprites.get(base_name) {
            Some(std::borrow::Cow::Borrowed(img))
        } else if let Some(reg) = composition_registry {
            if let Some(nested_comp) = reg.get(base_name) {
                if let Some(cached) = ctx.get_cached(base_name) {
                    Some(std::borrow::Cow::Owned(cached.clone()))
                } else {
                    let (rendered, nested_warnings) = render_composition_nested(
                        nested_comp,
                        sprites,
                        composition_registry,
                        ctx,
                        strict,
                        variables,
                    )?;
                    warnings.extend(nested_warnings);
                    ctx.cache(base_name.to_string(), rendered.clone());
                    Some(std::borrow::Cow::Owned(rendered))
                }
            } else {
                warnings.push(Warning::new(format!(
                    "Base '{}' not found for composition '{}'",
                    base_name, comp.name
                )));
                None
            }
        } else {
            warnings.push(Warning::new(format!(
                "Base sprite '{}' not found for composition '{}'",
                base_name, comp.name
            )));
            None
        }
    } else {
        None
    };

    // Determine canvas size
    let (width, height) = if let Some([w, h]) = comp.size {
        (w, h)
    } else if let Some(ref base_img) = base_image {
        (base_img.width(), base_img.height())
    } else {
        let (w, h) = infer_size_from_layers(&comp.layers, cell_size);
        if w == 0 || h == 0 {
            (1, 1)
        } else {
            (w, h)
        }
    };

    // Create canvas
    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    // Render base first
    if let Some(ref base_img) = base_image {
        blit_sprite(&mut canvas, base_img, 0, 0);
    }

    // Render each layer
    for layer in &comp.layers {
        let (blend_mode, blend_warning) = resolve_blend_mode(layer.blend.as_deref(), variables);
        if let Some(w) = blend_warning {
            warnings.push(w);
        }

        let (opacity, opacity_warning) = resolve_opacity(layer.opacity.as_ref(), variables);
        if let Some(w) = opacity_warning {
            warnings.push(w);
        }

        if let Some(ref map) = layer.map {
            for (row_idx, row) in map.iter().enumerate() {
                for (col_idx, char_key) in row.chars().enumerate() {
                    let key = char_key.to_string();

                    let sprite_name = match comp.sprites.get(&key) {
                        Some(Some(name)) => name,
                        Some(None) => continue,
                        None => {
                            warnings.push(Warning::new(format!(
                                "Unknown sprite key '{}' in composition '{}'",
                                key, comp.name
                            )));
                            continue;
                        }
                    };

                    // Get sprite/composition image (NC-4: check compositions too)
                    let sprite_image: std::borrow::Cow<'_, RgbaImage> =
                        if let Some(img) = sprites.get(sprite_name) {
                            std::borrow::Cow::Borrowed(img)
                        } else if let Some(reg) = composition_registry {
                            if let Some(nested_comp) = reg.get(sprite_name) {
                                if let Some(cached) = ctx.get_cached(sprite_name) {
                                    std::borrow::Cow::Owned(cached.clone())
                                } else {
                                    let (rendered, nested_warnings) = render_composition_nested(
                                        nested_comp,
                                        sprites,
                                        composition_registry,
                                        ctx,
                                        strict,
                                        variables,
                                    )?;
                                    warnings.extend(nested_warnings);
                                    ctx.cache(sprite_name.to_string(), rendered.clone());
                                    std::borrow::Cow::Owned(rendered)
                                }
                            } else {
                                warnings.push(Warning::new(format!(
                                    "Sprite '{}' not found for composition '{}'",
                                    sprite_name, comp.name
                                )));
                                continue;
                            }
                        } else {
                            warnings.push(Warning::new(format!(
                                "Sprite '{}' not found for composition '{}'",
                                sprite_name, comp.name
                            )));
                            continue;
                        };

                    let x = (col_idx as u32) * cell_size[0];
                    let y = (row_idx as u32) * cell_size[1];

                    blit_sprite_blended(&mut canvas, &sprite_image, x, y, blend_mode, opacity);
                }
            }
        }
    }

    Ok((canvas, warnings))
}

/// Infer canvas size from layer maps and cell size
pub(crate) fn infer_size_from_layers(
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
