//! Composition rendering - layering sprites onto a canvas

use crate::models::Composition;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;
use std::fmt;

/// A warning generated during composition rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Error when rendering a composition in strict mode.
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionError {
    /// Sprite dimensions exceed cell size
    SizeMismatch {
        sprite_name: String,
        sprite_size: (u32, u32),
        cell_size: (u32, u32),
        composition_name: String,
    },
}

impl fmt::Display for CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositionError::SizeMismatch {
                sprite_name,
                sprite_size,
                cell_size,
                composition_name,
            } => write!(
                f,
                "Sprite '{}' ({}x{}) exceeds cell size ({}x{}) in composition '{}'",
                sprite_name,
                sprite_size.0,
                sprite_size.1,
                cell_size.0,
                cell_size.1,
                composition_name
            ),
        }
    }
}

impl std::error::Error for CompositionError {}

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
/// ```ignore
/// use pixelsrc::composition::render_composition;
/// use pixelsrc::models::Composition;
/// use std::collections::HashMap;
/// use image::RgbaImage;
///
/// let comp = Composition { /* ... */ };
/// let sprites: HashMap<String, RgbaImage> = HashMap::new();
/// // Lenient mode
/// let result = render_composition(&comp, &sprites, false);
/// assert!(result.is_ok());
/// let (image, warnings) = result.unwrap();
/// ```
pub fn render_composition(
    comp: &Composition,
    sprites: &HashMap<String, RgbaImage>,
    strict: bool,
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

    // Create canvas (transparent by default)
    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    // Render base sprite first if present
    if let Some(base_img) = base_sprite {
        blit_sprite(&mut canvas, base_img, 0, 0);
    }

    // Render each layer (bottom to top)
    for layer in &comp.layers {
        if let Some(ref map) = layer.map {
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

                    // Blit sprite onto canvas (anchors top-left, overwrites adjacent cells)
                    blit_sprite(&mut canvas, sprite_image, x, y);
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
    let canvas_width = canvas.width();
    let canvas_height = canvas.height();

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

            // Alpha blend
            let src = pixel;
            if src[3] == 0 {
                // Fully transparent, skip
                continue;
            } else if src[3] == 255 {
                // Fully opaque, overwrite
                canvas.put_pixel(dest_x, dest_y, *src);
            } else {
                // Partial transparency, blend
                let dst = canvas.get_pixel(dest_x, dest_y);
                let blended = alpha_blend(src, dst);
                canvas.put_pixel(dest_x, dest_y, blended);
            }
        }
    }
}

/// Alpha blend source over destination
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

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
            }],
        };

        // Create a 1x1 red sprite
        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("red_pixel".to_string(), red_sprite)]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false).unwrap();

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
            }],
        };

        // Empty sprites map - sprite not provided
        let (_, warnings) = render_composition(&comp, &HashMap::new(), false).unwrap();

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
            }],
        };

        let mut pixel = RgbaImage::new(1, 1);
        pixel.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (image, _) = render_composition(&comp, &sprites, false).unwrap();

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
                },
                CompositionLayer {
                    name: Some("top".to_string()),
                    fill: None,
                    map: Some(vec!["B.".to_string(), "..".to_string()]),
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

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                },
                CompositionLayer {
                    name: Some("top".to_string()),
                    fill: None,
                    map: Some(vec!["..".to_string(), ".B".to_string()]),
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

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                },
                CompositionLayer {
                    name: Some("layer2".to_string()),
                    fill: None,
                    map: Some(vec!["GG".to_string(), "..".to_string()]),
                },
                CompositionLayer {
                    name: Some("layer3".to_string()),
                    fill: None,
                    map: Some(vec!["B.".to_string(), "..".to_string()]),
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

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                },
                CompositionLayer {
                    name: Some("empty".to_string()),
                    fill: None,
                    map: Some(vec!["..".to_string(), "..".to_string()]),
                },
            ],
        };

        let mut red_sprite = RgbaImage::new(1, 1);
        red_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("red_pixel".to_string(), red_sprite)]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
            }],
        };

        // 2x2 sprite exactly fits 2x2 cell
        let mut pixel = RgbaImage::new(2, 2);
        pixel.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        pixel.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
        pixel.put_pixel(0, 1, Rgba([255, 0, 0, 255]));
        pixel.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (_, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
            }],
        };

        // 2x2 sprite fits in 4x4 cell
        let mut pixel = RgbaImage::new(2, 2);
        pixel.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (_, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                map: Some(vec!["X...".to_string(), "....".to_string(), "....".to_string(), "....".to_string()]),
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

        let result = render_composition(&comp, &sprites, false);

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
                map: Some(vec!["X...".to_string(), "....".to_string(), "....".to_string(), "....".to_string()]),
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

        let result = render_composition(&comp, &sprites, true);

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
                },
                // Second layer: big red sprite at (0,0)
                CompositionLayer {
                    name: Some("foreground".to_string()),
                    fill: None,
                    map: Some(vec!["X..".to_string(), "...".to_string(), "...".to_string()]),
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

        let sprites = HashMap::from([
            ("big_sprite".to_string(), big_sprite),
            ("blue".to_string(), blue),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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

        let (_, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                map: Some(vec!["X.".to_string(), "..".to_string(), "..".to_string(), "..".to_string()]),
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

        let (_, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                map: Some(vec!["A...".to_string(), "....".to_string(), "..B.".to_string(), "....".to_string()]),
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

        let sprites = HashMap::from([
            ("big_a".to_string(), big_a),
            ("big_b".to_string(), big_b),
        ]);

        let (_, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
            }],
        };

        // 1x1 pixel sprites
        let mut red = RgbaImage::new(1, 1);
        red.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let mut green = RgbaImage::new(1, 1);
        green.put_pixel(0, 0, Rgba([0, 255, 0, 255]));

        let sprites = HashMap::from([
            ("red".to_string(), red),
            ("green".to_string(), green),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 4);

        // Check diagonal pattern
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255])); // R at (0,0)
        assert_eq!(*image.get_pixel(2, 0), Rgba([0, 255, 0, 255]));  // G at (2,0)
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255])); // R at (1,1)
        assert_eq!(*image.get_pixel(2, 1), Rgba([0, 255, 0, 255]));  // G at (2,1)
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

        let sprites = HashMap::from([
            ("tile_a".to_string(), tile_a),
            ("tile_b".to_string(), tile_b),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                map: Some(vec![
                    "GGW".to_string(),
                    "GWW".to_string(),
                ]),
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

        let sprites = HashMap::from([
            ("grass".to_string(), grass),
            ("water".to_string(), water),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

        assert!(warnings.is_empty());
        assert_eq!(image.width(), 48);
        assert_eq!(image.height(), 32);

        // Row 0: G at (0,0), G at (16,0), W at (32,0)
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 128, 0, 255]));   // Grass
        assert_eq!(*image.get_pixel(16, 0), Rgba([0, 128, 0, 255]));  // Grass
        assert_eq!(*image.get_pixel(32, 0), Rgba([0, 0, 200, 255]));  // Water

        // Row 1: G at (0,16), W at (16,16), W at (32,16)
        assert_eq!(*image.get_pixel(0, 16), Rgba([0, 128, 0, 255]));  // Grass
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
                map: Some(vec![
                    "X.X".to_string(),
                    "...".to_string(),
                    "X.X".to_string(),
                ]),
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

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
                map: Some(vec!["H.".to_string(), "..".to_string()]),
            }],
        };

        // Base sprite is 20x24
        let mut hero = RgbaImage::new(20, 24);
        for y in 0..24 {
            for x in 0..20 {
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

        let sprites = HashMap::from([
            ("hero".to_string(), hero),
            ("hat".to_string(), hat),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

        assert!(warnings.is_empty());
        // Canvas size should be inferred from base sprite (20x24)
        assert_eq!(image.width(), 20);
        assert_eq!(image.height(), 24);

        // Base sprite should be rendered first
        assert_eq!(*image.get_pixel(10, 12), Rgba([100, 100, 100, 255]));

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

        let (image, _) = render_composition(&comp, &sprites, false).unwrap();

        // Should use explicit size, not base size
        assert_eq!(image.width(), 10);
        assert_eq!(image.height(), 10);
    }

    #[test]
    fn test_size_inference_priority_base_over_layers() {
        // Base sprite size should take priority over layer inference
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
                map: Some(vec!["X.".to_string(), ".X".to_string()]),
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

        let sprites = HashMap::from([
            ("background".to_string(), background),
            ("tile".to_string(), tile),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

        assert!(warnings.is_empty());
        // Should use base sprite size (16x20), not layer-inferred (8x8)
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 20);
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
            }],
        };

        let mut tile = RgbaImage::new(8, 8);
        tile.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("tile".to_string(), tile)]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
            }],
        };

        let mut tile = RgbaImage::new(2, 2);
        tile.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let sprites = HashMap::from([("tile".to_string(), tile)]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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

        let sprites = HashMap::from([
            ("bg".to_string(), bg),
            ("overlay".to_string(), overlay),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
        use crate::models::{Sprite, Variant, PaletteRef};
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
            grid: vec![
                "{_}{skin}".to_string(),
                "{skin}{_}".to_string(),
            ],
        };

        let variant = Variant {
            name: "hero_red".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([
                ("{skin}".to_string(), "#FF0000".to_string()), // Red skin
            ]),
        };

        // Build registries and resolve
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(base_sprite);
        sprite_registry.register_variant(variant);

        // Render both base and variant
        let hero_resolved = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        let variant_resolved = sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();

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
            }],
        };

        // Provide both the base sprite and variant as rendered images
        let sprites = HashMap::from([
            ("hero".to_string(), hero_img),
            ("hero_red".to_string(), variant_img),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false).unwrap();

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
}
