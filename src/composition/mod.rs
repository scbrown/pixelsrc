//! Composition rendering - layering sprites onto a canvas

mod blend;
mod context;
mod error;
mod render;
mod resolve;

// Re-export public API
pub use blend::BlendMode;
pub use context::RenderContext;
pub use error::{CompositionError, Warning};
pub use render::{render_composition, render_composition_nested};
pub use resolve::{resolve_blend_mode, resolve_opacity};

/// Result type alias for composition operations.
pub type Result<T> = std::result::Result<T, CompositionError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Composition, CompositionLayer};
    use image::{Rgba, RgbaImage};
    use std::collections::HashMap;

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
        use render::infer_size_from_layers;

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

    /// Alpha blend source over destination (test helper)
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
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("big_sprite".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X.".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        // 2x2 sprite doesn't fit in 1x1 cell
        let mut big_sprite = RgbaImage::new(2, 2);
        big_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        big_sprite.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
        big_sprite.put_pixel(0, 1, Rgba([255, 0, 0, 255]));
        big_sprite.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("big_sprite".to_string(), big_sprite)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        // Should have size mismatch warning
        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("exceeds cell size"));
        assert!(warnings[0].message.contains("anchoring from top-left"));

        // Sprite should still render (anchored at top-left, overflowing to adjacent cells)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255])); // Overflow to right
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255])); // Overflow down
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255])); // Overflow diagonal
    }

    #[test]
    fn test_sprite_larger_than_cell_strict_error() {
        // Sprite larger than cell - error in strict mode
        let comp = Composition {
            name: "oversized_strict".to_string(),
            base: None,
            size: Some([2, 2]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                (".".to_string(), None),
                ("X".to_string(), Some("big_sprite".to_string())),
            ]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["X.".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        // 2x2 sprite doesn't fit in 1x1 cell
        let mut big_sprite = RgbaImage::new(2, 2);
        big_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let sprites = HashMap::from([("big_sprite".to_string(), big_sprite)]);

        let result = render_composition(&comp, &sprites, true, None);

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
                assert_eq!(sprite_size, (2, 2));
                assert_eq!(cell_size, (1, 1));
                assert_eq!(composition_name, "oversized_strict");
            }
            _ => panic!("Expected SizeMismatch error"),
        }
    }

    // Task 2.4: Size Validation and Map Dimension Tests

    #[test]
    fn test_size_not_divisible_warning() {
        // Size not divisible by cell_size - warning in lenient mode
        let comp = Composition {
            name: "bad_size".to_string(),
            base: None,
            size: Some([5, 5]), // Not divisible by 2x2
            cell_size: Some([2, 2]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["..".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("not divisible"));
    }

    #[test]
    fn test_size_not_divisible_strict_error() {
        // Size not divisible by cell_size - error in strict mode
        let comp = Composition {
            name: "bad_size_strict".to_string(),
            base: None,
            size: Some([5, 5]), // Not divisible by 2x2
            cell_size: Some([2, 2]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["..".to_string(), "..".to_string()]),
                ..Default::default()
            }],
        };

        let result = render_composition(&comp, &HashMap::new(), true, None);

        assert!(result.is_err());
        match result.unwrap_err() {
            CompositionError::SizeNotDivisible { size, cell_size, composition_name } => {
                assert_eq!(size, (5, 5));
                assert_eq!(cell_size, (2, 2));
                assert_eq!(composition_name, "bad_size_strict");
            }
            _ => panic!("Expected SizeNotDivisible error"),
        }
    }

    #[test]
    fn test_map_dimension_mismatch_warning() {
        // Map dimensions don't match expected grid - warning in lenient mode
        let comp = Composition {
            name: "bad_map".to_string(),
            base: None,
            size: Some([4, 4]), // 2x2 grid with 2x2 cells
            cell_size: Some([2, 2]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: Some("bad_layer".to_string()),
                fill: None,
                map: Some(vec!["...".to_string(), "...".to_string(), "...".to_string()]), // 3x3 instead of 2x2
                ..Default::default()
            }],
        };

        let (_, warnings) = render_composition(&comp, &HashMap::new(), false, None).unwrap();

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("Map dimensions"));
        assert!(warnings[0].message.contains("don't match"));
    }

    #[test]
    fn test_map_dimension_mismatch_strict_error() {
        // Map dimensions don't match expected grid - error in strict mode
        let comp = Composition {
            name: "bad_map_strict".to_string(),
            base: None,
            size: Some([4, 4]), // 2x2 grid with 2x2 cells
            cell_size: Some([2, 2]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: Some("bad_layer".to_string()),
                fill: None,
                map: Some(vec!["...".to_string(), "...".to_string(), "...".to_string()]), // 3x3 instead of 2x2
                ..Default::default()
            }],
        };

        let result = render_composition(&comp, &HashMap::new(), true, None);

        assert!(result.is_err());
        match result.unwrap_err() {
            CompositionError::MapDimensionMismatch {
                layer_name,
                actual_dimensions,
                expected_dimensions,
                composition_name,
            } => {
                assert_eq!(layer_name, Some("bad_layer".to_string()));
                assert_eq!(actual_dimensions, (3, 3));
                assert_eq!(expected_dimensions, (2, 2));
                assert_eq!(composition_name, "bad_map_strict");
            }
            _ => panic!("Expected MapDimensionMismatch error"),
        }
    }

    #[test]
    fn test_cell_size_1x1_no_validation() {
        // When cell_size is [1, 1], no size/map validation is performed
        let comp = Composition {
            name: "pixel_perfect".to_string(),
            base: None,
            size: Some([3, 3]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([(".".to_string(), None)]),
            layers: vec![CompositionLayer {
                name: None,
                fill: None,
                map: Some(vec!["....".to_string(), "....".to_string()]), // Doesn't match 3x3
                ..Default::default()
            }],
        };

        // In strict mode, no errors for [1, 1] cell size
        let result = render_composition(&comp, &HashMap::new(), true, None);
        assert!(result.is_ok());
        assert!(result.unwrap().1.is_empty()); // No warnings either
    }

    // Task 2.3: Cell Size Scaling tests

    #[test]
    fn test_cell_size_2x2_placement() {
        // Sprites placed at 2x2 grid positions
        let comp = Composition {
            name: "scaled".to_string(),
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

        // 2x2 sprite fills exactly one cell
        let mut pixel = RgbaImage::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                pixel.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let sprites = HashMap::from([("pixel".to_string(), pixel)]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();

        assert!(warnings.is_empty());

        // X at grid (0,0) -> pixels (0,0), (1,0), (0,1), (1,1)
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));

        // . at grid (1,0) -> pixels (2,0), (3,0), (2,1), (3,1) should be transparent
        assert_eq!(*image.get_pixel(2, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*image.get_pixel(3, 0), Rgba([0, 0, 0, 0]));

        // X at grid (1,1) -> pixels (2,2), (3,2), (2,3), (3,3)
        assert_eq!(*image.get_pixel(2, 2), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(3, 2), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(2, 3), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(3, 3), Rgba([255, 0, 0, 255]));
    }

    // Blend mode tests (ATF-10)

    #[test]
    fn test_blend_mode_parsing() {
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
        assert_eq!(BlendMode::from_str("unknown"), None);

        // Case insensitive
        assert_eq!(BlendMode::from_str("NORMAL"), Some(BlendMode::Normal));
        assert_eq!(BlendMode::from_str("Multiply"), Some(BlendMode::Multiply));
    }

    #[test]
    fn test_blend_mode_multiply() {
        // Multiply: result = base * blend
        let comp = Composition {
            name: "multiply_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                ("B".to_string(), Some("base".to_string())),
                ("O".to_string(), Some("overlay".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("base".to_string()),
                    fill: None,
                    map: Some(vec!["B".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("overlay".to_string()),
                    fill: None,
                    blend: Some("multiply".to_string()),
                    map: Some(vec!["O".to_string()]),
                    ..Default::default()
                },
            ],
        };

        let mut base_sprite = RgbaImage::new(1, 1);
        base_sprite.put_pixel(0, 0, Rgba([200, 100, 50, 255]));

        let mut overlay_sprite = RgbaImage::new(1, 1);
        overlay_sprite.put_pixel(0, 0, Rgba([128, 128, 128, 255])); // 50% gray

        let sprites = HashMap::from([
            ("base".to_string(), base_sprite),
            ("overlay".to_string(), overlay_sprite),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();
        assert!(warnings.is_empty());

        let pixel = image.get_pixel(0, 0);
        // Multiply: (200 * 128/255) ≈ 100, (100 * 128/255) ≈ 50, (50 * 128/255) ≈ 25
        assert!(pixel[0] > 90 && pixel[0] < 110); // Red
        assert!(pixel[1] > 45 && pixel[1] < 55); // Green
        assert!(pixel[2] > 20 && pixel[2] < 30); // Blue
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn test_layer_opacity() {
        let comp = Composition {
            name: "opacity_test".to_string(),
            base: None,
            size: Some([1, 1]),
            cell_size: Some([1, 1]),
            sprites: HashMap::from([
                ("B".to_string(), Some("base".to_string())),
                ("O".to_string(), Some("overlay".to_string())),
            ]),
            layers: vec![
                CompositionLayer {
                    name: Some("base".to_string()),
                    fill: None,
                    map: Some(vec!["B".to_string()]),
                    ..Default::default()
                },
                CompositionLayer {
                    name: Some("overlay".to_string()),
                    fill: None,
                    opacity: Some(crate::models::VarOr::Value(0.5)),
                    map: Some(vec!["O".to_string()]),
                    ..Default::default()
                },
            ],
        };

        let mut base_sprite = RgbaImage::new(1, 1);
        base_sprite.put_pixel(0, 0, Rgba([0, 0, 255, 255])); // Blue

        let mut overlay_sprite = RgbaImage::new(1, 1);
        overlay_sprite.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Red, but 50% opacity

        let sprites = HashMap::from([
            ("base".to_string(), base_sprite),
            ("overlay".to_string(), overlay_sprite),
        ]);

        let (image, warnings) = render_composition(&comp, &sprites, false, None).unwrap();
        assert!(warnings.is_empty());

        let pixel = image.get_pixel(0, 0);
        // With 50% opacity red over blue, should be roughly purple
        assert!(pixel[0] > 100 && pixel[0] < 150); // Some red
        assert!(pixel[2] > 100 && pixel[2] < 150); // Some blue
        assert_eq!(pixel[3], 255);
    }

    // CSS variable resolution tests (CSS-9)

    #[test]
    fn test_blend_mode_var_resolution() {
        use crate::variables::VariableRegistry;

        let mut registry = VariableRegistry::new();
        registry.define("--blend-mode", "multiply");

        let (mode, warning) = resolve_blend_mode(Some("var(--blend-mode)"), Some(&registry));
        assert_eq!(mode, BlendMode::Multiply);
        assert!(warning.is_none());
    }

    #[test]
    fn test_blend_mode_var_fallback() {
        use crate::variables::VariableRegistry;

        let registry = VariableRegistry::new(); // Empty - no definitions

        // Should use fallback
        let (mode, warning) = resolve_blend_mode(Some("var(--missing, screen)"), Some(&registry));
        assert_eq!(mode, BlendMode::Screen);
        assert!(warning.is_none());
    }

    #[test]
    fn test_blend_mode_var_no_registry_warning() {
        let (mode, warning) = resolve_blend_mode(Some("var(--blend-mode)"), None);
        assert_eq!(mode, BlendMode::Normal);
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("no variable registry"));
    }

    #[test]
    fn test_opacity_var_resolution() {
        use crate::models::VarOr;
        use crate::variables::VariableRegistry;

        let mut registry = VariableRegistry::new();
        registry.define("--layer-opacity", "0.75");

        let (opacity, warning) =
            resolve_opacity(Some(&VarOr::Var("var(--layer-opacity)".to_string())), Some(&registry));
        assert!((opacity - 0.75).abs() < 0.01);
        assert!(warning.is_none());
    }

    #[test]
    fn test_opacity_direct_value() {
        use crate::models::VarOr;

        let (opacity, warning) = resolve_opacity(Some(&VarOr::Value(0.5)), None);
        assert!((opacity - 0.5).abs() < 0.01);
        assert!(warning.is_none());
    }

    // Nested composition tests (NC-3, NC-4)

    mod nested_tests {
        use super::*;
        use crate::registry::CompositionRegistry;

        fn make_composition(
            name: &str,
            sprites: HashMap<String, Option<String>>,
            layer_maps: Vec<Vec<String>>,
            size: Option<[u32; 2]>,
        ) -> Composition {
            let layers: Vec<CompositionLayer> = layer_maps
                .into_iter()
                .map(|map| CompositionLayer {
                    name: None,
                    fill: None,
                    map: Some(map),
                    ..Default::default()
                })
                .collect();

            Composition {
                name: name.to_string(),
                base: None,
                size,
                cell_size: Some([2, 2]),
                sprites,
                layers,
            }
        }

        fn make_sprite_image(r: u8, g: u8, b: u8) -> RgbaImage {
            let mut img = RgbaImage::new(2, 2);
            for y in 0..2 {
                for x in 0..2 {
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            img
        }

        #[test]
        fn test_render_context_caching() {
            let mut ctx = RenderContext::new();

            assert!(!ctx.is_cached("test"));
            assert_eq!(ctx.len(), 0);
            assert!(ctx.is_empty());

            let img = RgbaImage::new(1, 1);
            ctx.cache("test".to_string(), img);

            assert!(ctx.is_cached("test"));
            assert_eq!(ctx.len(), 1);
            assert!(!ctx.is_empty());

            let cached = ctx.get_cached("test");
            assert!(cached.is_some());
            assert_eq!(cached.unwrap().width(), 1);
        }

        #[test]
        fn test_render_context_cycle_detection() {
            let mut ctx = RenderContext::new();

            // Push A -> OK
            assert!(ctx.push("A").is_ok());
            assert!(ctx.contains("A"));
            assert_eq!(ctx.depth(), 1);

            // Push B -> OK
            assert!(ctx.push("B").is_ok());
            assert!(ctx.contains("B"));
            assert_eq!(ctx.depth(), 2);

            // Push A again -> Error (cycle)
            let result = ctx.push("A");
            assert!(result.is_err());
            match result.unwrap_err() {
                CompositionError::CycleDetected { cycle_path } => {
                    assert_eq!(cycle_path, vec!["A", "B", "A"]);
                }
                _ => panic!("Expected CycleDetected error"),
            }

            // Pop should work
            assert_eq!(ctx.pop(), Some("B".to_string()));
            assert_eq!(ctx.depth(), 1);
            assert!(!ctx.contains("B"));
        }

        #[test]
        fn test_nested_composition_rendering() {
            // Create sprites
            let mut sprites = HashMap::new();
            sprites.insert("red".to_string(), make_sprite_image(255, 0, 0));
            sprites.insert("blue".to_string(), make_sprite_image(0, 0, 255));

            // Create a sub-composition (red square)
            let sub_sprites: HashMap<String, Option<String>> =
                [("R".to_string(), Some("red".to_string()))].into_iter().collect();

            let sub_comp =
                make_composition("sub", sub_sprites, vec![vec!["R".to_string()]], Some([2, 2]));

            // Create main composition that references sub
            let main_sprites: HashMap<String, Option<String>> =
                [("S".to_string(), Some("sub".to_string()))].into_iter().collect();

            let main_comp = make_composition(
                "main",
                main_sprites,
                vec![vec!["SS".to_string(), "SS".to_string()]],
                Some([4, 4]),
            );

            // Register compositions
            let mut composition_registry = CompositionRegistry::new();
            composition_registry.register(sub_comp);
            composition_registry.register(main_comp.clone());

            let mut ctx = RenderContext::new();

            // Render main composition (which references sub)
            let result = render_composition_nested(
                &main_comp,
                &sprites,
                Some(&composition_registry),
                &mut ctx,
                false,
                None,
            );

            assert!(result.is_ok());
            let (image, warnings) = result.unwrap();
            assert!(warnings.is_empty());
            assert_eq!(image.width(), 4);
            assert_eq!(image.height(), 4);

            // Verify the cache was used - "sub" should be cached
            assert!(ctx.is_cached("sub"));

            // All 4 cells should be red (from the cached sub composition)
            assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
            assert_eq!(*image.get_pixel(2, 0), Rgba([255, 0, 0, 255]));
            assert_eq!(*image.get_pixel(0, 2), Rgba([255, 0, 0, 255]));
            assert_eq!(*image.get_pixel(2, 2), Rgba([255, 0, 0, 255]));
        }

        #[test]
        fn test_nested_base_composition() {
            // Test: base can also be a composition reference
            let mut sprites = HashMap::new();
            sprites.insert("blue".to_string(), make_sprite_image(0, 0, 255));
            sprites.insert("red".to_string(), make_sprite_image(255, 0, 0));

            // background composition is all blue
            let bg_sprites: HashMap<String, Option<String>> =
                [("B".to_string(), Some("blue".to_string()))].into_iter().collect();

            let bg_comp = make_composition(
                "background",
                bg_sprites,
                vec![vec!["BB".to_string(), "BB".to_string()]],
                Some([4, 4]),
            );

            // main composition has background as base, with red overlay
            let main_sprites: HashMap<String, Option<String>> =
                [("R".to_string(), Some("red".to_string())), (".".to_string(), None)]
                    .into_iter()
                    .collect();

            let mut main_comp = make_composition(
                "main",
                main_sprites,
                vec![vec!["R.".to_string(), ".R".to_string()]],
                Some([4, 4]),
            );
            main_comp.base = Some("background".to_string());

            let mut composition_registry = CompositionRegistry::new();
            composition_registry.register(bg_comp);
            composition_registry.register(main_comp.clone());

            let mut ctx = RenderContext::new();

            let result = render_composition_nested(
                &main_comp,
                &sprites,
                Some(&composition_registry),
                &mut ctx,
                false,
                None,
            );

            assert!(result.is_ok());
            let (image, warnings) = result.unwrap();
            assert!(warnings.is_empty());

            // (0,0) should be red (overlay)
            assert_eq!(*image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
            // (2,0) should be blue (from background)
            assert_eq!(*image.get_pixel(2, 0), Rgba([0, 0, 255, 255]));
            // (2,2) should be red (overlay)
            assert_eq!(*image.get_pixel(2, 2), Rgba([255, 0, 0, 255]));
        }

        #[test]
        fn test_nested_missing_composition_warning() {
            // Test: referencing non-existent composition emits warning
            let sprites = HashMap::new();

            let main_sprites: HashMap<String, Option<String>> =
                [("X".to_string(), Some("nonexistent".to_string()))].into_iter().collect();

            let main_comp =
                make_composition("main", main_sprites, vec![vec!["X".to_string()]], Some([2, 2]));

            let composition_registry = CompositionRegistry::new();
            let mut ctx = RenderContext::new();

            let result = render_composition_nested(
                &main_comp,
                &sprites,
                Some(&composition_registry),
                &mut ctx,
                false,
                None,
            );

            assert!(result.is_ok());
            let (_, warnings) = result.unwrap();
            assert!(!warnings.is_empty());
            assert!(warnings[0].message.contains("not found"));
        }
    }
}
