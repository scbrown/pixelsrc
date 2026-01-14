//! Composition rendering - layering sprites onto a canvas

use crate::models::Composition;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;

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
/// # Size Inference
///
/// Canvas size is determined by (in order of priority):
/// 1. `composition.size` if explicitly set
/// 2. Inferred from layer maps and cell_size
///
/// # Examples
///
/// ```ignore
/// use pxl::composition::render_composition;
/// use pxl::models::Composition;
/// use std::collections::HashMap;
/// use image::RgbaImage;
///
/// let comp = Composition { /* ... */ };
/// let sprites: HashMap<String, RgbaImage> = HashMap::new();
/// let (image, warnings) = render_composition(&comp, &sprites);
/// ```
pub fn render_composition(
    comp: &Composition,
    sprites: &HashMap<String, RgbaImage>,
) -> (RgbaImage, Vec<Warning>) {
    let mut warnings = Vec::new();

    // Determine cell size (default to [1, 1])
    let cell_size = comp.cell_size.unwrap_or([1, 1]);

    // Determine canvas size
    let (width, height) = if let Some([w, h]) = comp.size {
        (w, h)
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

                    // Calculate position
                    let x = (col_idx as u32) * cell_size[0];
                    let y = (row_idx as u32) * cell_size[1];

                    // Blit sprite onto canvas
                    blit_sprite(&mut canvas, sprite_image, x, y);
                }
            }
        }
    }

    (canvas, warnings)
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

        let (image, warnings) = render_composition(&comp, &sprites);

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

        let (image, warnings) = render_composition(&comp, &sprites);

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

        let (_, warnings) = render_composition(&comp, &HashMap::new());

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
        let (_, warnings) = render_composition(&comp, &HashMap::new());

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

        let (image, _) = render_composition(&comp, &sprites);

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

        let (image, warnings) = render_composition(&comp, &sprites);

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

        let (image, warnings) = render_composition(&comp, &sprites);

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

        let (image, warnings) = render_composition(&comp, &sprites);

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

        let (image, warnings) = render_composition(&comp, &sprites);

        assert!(warnings.is_empty());
        // All pixels should still be red (empty layer didn't erase anything)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
    }
}
