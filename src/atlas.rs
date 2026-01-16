//! Atlas packing - combines multiple sprites into a texture atlas with metadata
//!
//! Implements shelf bin packing for efficient sprite arrangement.

use image::{Rgba, RgbaImage};
use serde::Serialize;
use std::collections::HashMap;

/// Configuration for atlas packing
#[derive(Debug, Clone)]
pub struct AtlasConfig {
    /// Maximum atlas dimensions (width, height)
    pub max_size: (u32, u32),
    /// Padding between sprites in pixels
    pub padding: u32,
    /// Force power-of-two dimensions
    pub power_of_two: bool,
}

impl Default for AtlasConfig {
    fn default() -> Self {
        Self {
            max_size: (4096, 4096),
            padding: 0,
            power_of_two: false,
        }
    }
}

/// A collision box in atlas export format
#[derive(Debug, Clone, Serialize)]
pub struct AtlasBox {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// A sprite's position and size within an atlas
#[derive(Debug, Clone, Serialize)]
pub struct AtlasFrame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// Sprite origin point (for positioning/rotation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<[i32; 2]>,
    /// Collision boxes (hit, hurt, collide, trigger, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boxes: Option<HashMap<String, AtlasBox>>,
}

/// Animation metadata for atlas export
#[derive(Debug, Clone, Serialize)]
pub struct AtlasAnimation {
    pub frames: Vec<String>,
    pub fps: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, AtlasTag>>,
}

/// A tag within an animation (frame range)
#[derive(Debug, Clone, Serialize)]
pub struct AtlasTag {
    pub from: u32,
    pub to: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#loop: Option<bool>,
}

/// Complete atlas metadata
#[derive(Debug, Clone, Serialize)]
pub struct AtlasMetadata {
    pub image: String,
    pub size: [u32; 2],
    pub frames: HashMap<String, AtlasFrame>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub animations: HashMap<String, AtlasAnimation>,
}

/// Result of atlas packing - may produce multiple atlases
#[derive(Debug)]
pub struct AtlasResult {
    /// List of (image, metadata) pairs for each atlas
    pub atlases: Vec<(RgbaImage, AtlasMetadata)>,
}

/// A sprite to be packed into an atlas
#[derive(Debug)]
pub struct SpriteInput {
    pub name: String,
    pub image: RgbaImage,
    /// Optional origin point for this sprite
    pub origin: Option<[i32; 2]>,
    /// Optional collision boxes for this sprite
    pub boxes: Option<HashMap<String, AtlasBox>>,
}

/// A shelf in the shelf packing algorithm
#[derive(Debug)]
struct Shelf {
    y: u32,
    height: u32,
    width_used: u32,
}

/// Transparent color for atlas background
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Pack sprites into texture atlases.
///
/// Uses a simple shelf-based bin packing algorithm. Sprites are sorted by height
/// (tallest first) and placed into horizontal shelves.
///
/// # Arguments
///
/// * `sprites` - Slice of sprites to pack (name + image pairs)
/// * `config` - Atlas configuration (max size, padding, etc.)
/// * `base_image_name` - Base filename for the atlas image(s)
///
/// # Returns
///
/// AtlasResult containing one or more (image, metadata) pairs
pub fn pack_atlas(
    sprites: &[SpriteInput],
    config: &AtlasConfig,
    base_image_name: &str,
) -> AtlasResult {
    if sprites.is_empty() {
        return AtlasResult { atlases: vec![] };
    }

    // Sort sprites by height descending (better packing)
    let mut sorted_sprites: Vec<(usize, &SpriteInput)> = sprites.iter().enumerate().collect();
    sorted_sprites.sort_by(|a, b| b.1.image.height().cmp(&a.1.image.height()));

    let mut atlases: Vec<(RgbaImage, AtlasMetadata, Vec<Shelf>)> = vec![];
    let mut sprite_to_atlas: HashMap<String, (usize, AtlasFrame)> = HashMap::new();

    for (_, sprite) in sorted_sprites {
        let sprite_w = sprite.image.width();
        let sprite_h = sprite.image.height();
        let padded_w = sprite_w + config.padding;
        let padded_h = sprite_h + config.padding;

        // Try to fit in existing atlases
        let mut placed = false;
        for (atlas_idx, (_, _, shelves)) in atlases.iter_mut().enumerate() {
            if let Some(pos) = try_place_in_shelves(
                shelves,
                padded_w,
                padded_h,
                sprite_w,
                sprite_h,
                config.max_size,
            ) {
                sprite_to_atlas.insert(
                    sprite.name.clone(),
                    (
                        atlas_idx,
                        AtlasFrame {
                            x: pos.0,
                            y: pos.1,
                            w: sprite_w,
                            h: sprite_h,
                            origin: sprite.origin,
                            boxes: sprite.boxes.clone(),
                        },
                    ),
                );
                placed = true;
                break;
            }
        }

        // Create new atlas if needed
        if !placed {
            let mut shelves = vec![];
            if let Some(pos) = try_place_in_shelves(
                &mut shelves,
                padded_w,
                padded_h,
                sprite_w,
                sprite_h,
                config.max_size,
            ) {
                let atlas_idx = atlases.len();
                let image_name = if atlas_idx == 0 && sprites.len() <= config.max_size.0 as usize {
                    format!("{}.png", base_image_name)
                } else {
                    format!("{}_{}.png", base_image_name, atlas_idx)
                };

                let metadata = AtlasMetadata {
                    image: image_name,
                    size: [0, 0], // Will be calculated later
                    frames: HashMap::new(),
                    animations: HashMap::new(),
                };

                atlases.push((RgbaImage::new(1, 1), metadata, shelves));

                sprite_to_atlas.insert(
                    sprite.name.clone(),
                    (
                        atlas_idx,
                        AtlasFrame {
                            x: pos.0,
                            y: pos.1,
                            w: sprite_w,
                            h: sprite_h,
                            origin: sprite.origin,
                            boxes: sprite.boxes.clone(),
                        },
                    ),
                );
            }
        }
    }

    // Now calculate actual atlas sizes and create images
    let mut result_atlases = vec![];

    for (atlas_idx, (_, mut metadata, shelves)) in atlases.into_iter().enumerate() {
        // Calculate atlas size
        let (atlas_w, atlas_h) = calculate_atlas_size(&shelves, config);
        metadata.size = [atlas_w, atlas_h];

        // Update image name for single atlas case
        if result_atlases.is_empty() && sprite_to_atlas.values().all(|(idx, _)| *idx == 0) {
            metadata.image = format!("{}.png", base_image_name);
        }

        // Create the atlas image
        let mut atlas_image = RgbaImage::from_pixel(atlas_w, atlas_h, TRANSPARENT);

        // Copy sprites into atlas
        for sprite in sprites {
            if let Some((idx, frame)) = sprite_to_atlas.get(&sprite.name) {
                if *idx == atlas_idx {
                    copy_sprite_to_atlas(&mut atlas_image, &sprite.image, frame.x, frame.y);
                    metadata.frames.insert(sprite.name.clone(), frame.clone());
                }
            }
        }

        result_atlases.push((atlas_image, metadata));
    }

    // Rename single atlas if there's only one
    if result_atlases.len() == 1 {
        result_atlases[0].1.image = format!("{}.png", base_image_name);
    }

    AtlasResult {
        atlases: result_atlases,
    }
}

/// Try to place a sprite in the given shelves
fn try_place_in_shelves(
    shelves: &mut Vec<Shelf>,
    padded_w: u32,
    padded_h: u32,
    _sprite_w: u32,
    sprite_h: u32,
    max_size: (u32, u32),
) -> Option<(u32, u32)> {
    // Try to fit in existing shelf
    for shelf in shelves.iter_mut() {
        if sprite_h <= shelf.height && shelf.width_used + padded_w <= max_size.0 {
            let x = shelf.width_used;
            let y = shelf.y;
            shelf.width_used += padded_w;
            return Some((x, y));
        }
    }

    // Try to create new shelf
    let new_shelf_y = shelves.last().map(|s| s.y + s.height).unwrap_or(0);
    if new_shelf_y + padded_h <= max_size.1 && padded_w <= max_size.0 {
        shelves.push(Shelf {
            y: new_shelf_y,
            height: padded_h,
            width_used: padded_w,
        });
        return Some((0, new_shelf_y));
    }

    None
}

/// Calculate the final atlas dimensions
fn calculate_atlas_size(shelves: &[Shelf], config: &AtlasConfig) -> (u32, u32) {
    if shelves.is_empty() {
        return (1, 1);
    }

    // Find the maximum width used and total height
    let max_width = shelves.iter().map(|s| s.width_used).max().unwrap_or(1);
    let total_height = shelves
        .last()
        .map(|s| s.y + s.height)
        .unwrap_or(1);

    // Remove padding from edges (padding is between sprites, not on edges)
    let width = if config.padding > 0 && max_width > config.padding {
        max_width - config.padding
    } else {
        max_width.max(1)
    };
    let height = if config.padding > 0 && total_height > config.padding {
        total_height - config.padding
    } else {
        total_height.max(1)
    };

    if config.power_of_two {
        (next_power_of_two(width), next_power_of_two(height))
    } else {
        (width, height)
    }
}

/// Get the next power of two >= n
fn next_power_of_two(n: u32) -> u32 {
    if n == 0 {
        return 1;
    }
    let mut p = 1u32;
    while p < n {
        p *= 2;
    }
    p
}

/// Copy a sprite image to the atlas at the given position
fn copy_sprite_to_atlas(atlas: &mut RgbaImage, sprite: &RgbaImage, x: u32, y: u32) {
    for sy in 0..sprite.height() {
        for sx in 0..sprite.width() {
            let pixel = *sprite.get_pixel(sx, sy);
            if x + sx < atlas.width() && y + sy < atlas.height() {
                atlas.put_pixel(x + sx, y + sy, pixel);
            }
        }
    }
}

/// Add animation metadata to an atlas
pub fn add_animation_to_atlas(
    metadata: &mut AtlasMetadata,
    name: &str,
    frame_names: &[String],
    fps: u32,
) {
    metadata.animations.insert(
        name.to_string(),
        AtlasAnimation {
            frames: frame_names.to_vec(),
            fps,
            tags: None,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solid_sprite(name: &str, width: u32, height: u32, color: Rgba<u8>) -> SpriteInput {
        SpriteInput {
            name: name.to_string(),
            image: RgbaImage::from_pixel(width, height, color),
            origin: None,
            boxes: None,
        }
    }

    #[test]
    fn test_empty_sprites() {
        let result = pack_atlas(&[], &AtlasConfig::default(), "test");
        assert!(result.atlases.is_empty());
    }

    #[test]
    fn test_single_sprite() {
        let red = Rgba([255, 0, 0, 255]);
        let sprites = vec![make_solid_sprite("red", 16, 16, red)];
        let result = pack_atlas(&sprites, &AtlasConfig::default(), "test");

        assert_eq!(result.atlases.len(), 1);
        let (image, metadata) = &result.atlases[0];

        assert_eq!(metadata.image, "test.png");
        assert!(metadata.frames.contains_key("red"));

        let frame = &metadata.frames["red"];
        assert_eq!(frame.x, 0);
        assert_eq!(frame.y, 0);
        assert_eq!(frame.w, 16);
        assert_eq!(frame.h, 16);

        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 16);
        assert_eq!(*image.get_pixel(0, 0), red);
    }

    #[test]
    fn test_multiple_sprites() {
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);

        let sprites = vec![
            make_solid_sprite("red", 16, 16, red),
            make_solid_sprite("green", 16, 16, green),
            make_solid_sprite("blue", 16, 16, blue),
        ];

        let result = pack_atlas(&sprites, &AtlasConfig::default(), "test");

        assert_eq!(result.atlases.len(), 1);
        let (image, metadata) = &result.atlases[0];

        assert_eq!(metadata.frames.len(), 3);
        assert!(metadata.frames.contains_key("red"));
        assert!(metadata.frames.contains_key("green"));
        assert!(metadata.frames.contains_key("blue"));

        // Check that sprites don't overlap
        let positions: Vec<(u32, u32, u32, u32)> = metadata
            .frames
            .values()
            .map(|f| (f.x, f.y, f.w, f.h))
            .collect();

        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let (x1, y1, w1, h1) = positions[i];
                let (x2, y2, w2, h2) = positions[j];
                // Check no overlap
                let overlap = x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
                assert!(!overlap, "Sprites overlap");
            }
        }

        // Verify colors are correctly placed
        for (name, frame) in &metadata.frames {
            let expected_color = match name.as_str() {
                "red" => red,
                "green" => green,
                "blue" => blue,
                _ => panic!("Unknown sprite"),
            };
            assert_eq!(*image.get_pixel(frame.x, frame.y), expected_color);
        }
    }

    #[test]
    fn test_padding() {
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);

        let sprites = vec![
            make_solid_sprite("red", 8, 8, red),
            make_solid_sprite("green", 8, 8, green),
        ];

        let config = AtlasConfig {
            padding: 2,
            ..Default::default()
        };

        let result = pack_atlas(&sprites, &config, "test");
        assert_eq!(result.atlases.len(), 1);
        let (_, metadata) = &result.atlases[0];

        // With padding of 2, sprites should be at least 2 pixels apart
        let frames: Vec<&AtlasFrame> = metadata.frames.values().collect();
        assert_eq!(frames.len(), 2);

        // One should be at (0,0), other at (10,0) or (0,10) with padding
        let mut x_positions: Vec<u32> = frames.iter().map(|f| f.x).collect();
        x_positions.sort();

        // Second sprite should start after first sprite + padding
        if x_positions[0] == 0 && x_positions[1] > 0 {
            assert!(x_positions[1] >= 8 + 2, "Padding not applied correctly");
        }
    }

    #[test]
    fn test_power_of_two() {
        let red = Rgba([255, 0, 0, 255]);
        let sprites = vec![make_solid_sprite("red", 10, 10, red)];

        let config = AtlasConfig {
            power_of_two: true,
            ..Default::default()
        };

        let result = pack_atlas(&sprites, &config, "test");
        assert_eq!(result.atlases.len(), 1);
        let (image, metadata) = &result.atlases[0];

        // 10x10 should become 16x16 (next power of two)
        assert_eq!(image.width(), 16);
        assert_eq!(image.height(), 16);
        assert_eq!(metadata.size, [16, 16]);
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(4), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(9), 16);
        assert_eq!(next_power_of_two(100), 128);
        assert_eq!(next_power_of_two(256), 256);
        assert_eq!(next_power_of_two(257), 512);
    }

    #[test]
    fn test_different_sized_sprites() {
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);

        let sprites = vec![
            make_solid_sprite("big", 32, 32, red),
            make_solid_sprite("medium", 16, 16, green),
            make_solid_sprite("small", 8, 8, blue),
        ];

        let result = pack_atlas(&sprites, &AtlasConfig::default(), "test");
        assert_eq!(result.atlases.len(), 1);
        let (_, metadata) = &result.atlases[0];

        assert_eq!(metadata.frames["big"].w, 32);
        assert_eq!(metadata.frames["big"].h, 32);
        assert_eq!(metadata.frames["medium"].w, 16);
        assert_eq!(metadata.frames["medium"].h, 16);
        assert_eq!(metadata.frames["small"].w, 8);
        assert_eq!(metadata.frames["small"].h, 8);
    }

    #[test]
    fn test_max_size_creates_multiple_atlases() {
        let red = Rgba([255, 0, 0, 255]);

        // Create sprites that won't fit in a small atlas
        let sprites = vec![
            make_solid_sprite("a", 32, 32, red),
            make_solid_sprite("b", 32, 32, red),
            make_solid_sprite("c", 32, 32, red),
            make_solid_sprite("d", 32, 32, red),
        ];

        let config = AtlasConfig {
            max_size: (64, 64), // Only fits 4 sprites in 2x2
            ..Default::default()
        };

        let result = pack_atlas(&sprites, &config, "test");
        // Should fit all 4 in one 64x64 atlas (2x2 grid)
        assert_eq!(result.atlases.len(), 1);

        // Now try with even smaller
        let config_tiny = AtlasConfig {
            max_size: (32, 32), // Only fits 1 sprite
            ..Default::default()
        };

        let result_tiny = pack_atlas(&sprites, &config_tiny, "test");
        // Should create multiple atlases
        assert!(result_tiny.atlases.len() > 1);
    }

    #[test]
    fn test_atlas_metadata_serialization() {
        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [64, 64],
            frames: HashMap::from([
                (
                    "sprite1".to_string(),
                    AtlasFrame {
                        x: 0,
                        y: 0,
                        w: 16,
                        h: 16,
                        origin: None,
                        boxes: None,
                    },
                ),
                (
                    "sprite2".to_string(),
                    AtlasFrame {
                        x: 16,
                        y: 0,
                        w: 16,
                        h: 16,
                        origin: None,
                        boxes: None,
                    },
                ),
            ]),
            animations: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("\"image\""));
        assert!(json.contains("\"size\""));
        assert!(json.contains("\"frames\""));
        assert!(json.contains("\"sprite1\""));
        assert!(json.contains("\"sprite2\""));

        // Should not include empty animations
        assert!(!json.contains("\"animations\""));
        // Should not include None origin/boxes
        assert!(!json.contains("\"origin\""));
        assert!(!json.contains("\"boxes\""));
    }

    #[test]
    fn test_add_animation_to_atlas() {
        let mut metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [64, 64],
            frames: HashMap::new(),
            animations: HashMap::new(),
        };

        add_animation_to_atlas(
            &mut metadata,
            "walk",
            &["walk_1".to_string(), "walk_2".to_string()],
            10,
        );

        assert!(metadata.animations.contains_key("walk"));
        let anim = &metadata.animations["walk"];
        assert_eq!(anim.frames, vec!["walk_1", "walk_2"]);
        assert_eq!(anim.fps, 10);
    }

    #[test]
    fn test_sprite_input_with_metadata() {
        // Test that sprite metadata (origin and boxes) is preserved in atlas packing
        let red = Rgba([255, 0, 0, 255]);
        let sprite = SpriteInput {
            name: "player".to_string(),
            image: RgbaImage::from_pixel(32, 32, red),
            origin: Some([16, 32]),
            boxes: Some(HashMap::from([
                (
                    "hurt".to_string(),
                    AtlasBox {
                        x: 4,
                        y: 0,
                        w: 24,
                        h: 32,
                    },
                ),
                (
                    "hit".to_string(),
                    AtlasBox {
                        x: 20,
                        y: 8,
                        w: 20,
                        h: 16,
                    },
                ),
            ])),
        };

        let result = pack_atlas(&[sprite], &AtlasConfig::default(), "test");

        assert_eq!(result.atlases.len(), 1);
        let (_, metadata) = &result.atlases[0];

        let frame = &metadata.frames["player"];
        assert_eq!(frame.w, 32);
        assert_eq!(frame.h, 32);
        assert_eq!(frame.origin, Some([16, 32]));

        let boxes = frame.boxes.as_ref().unwrap();
        assert_eq!(boxes.len(), 2);
        assert!(boxes.contains_key("hurt"));
        assert!(boxes.contains_key("hit"));

        let hurt_box = &boxes["hurt"];
        assert_eq!(hurt_box.x, 4);
        assert_eq!(hurt_box.y, 0);
        assert_eq!(hurt_box.w, 24);
        assert_eq!(hurt_box.h, 32);
    }

    #[test]
    fn test_atlas_frame_metadata_serialization() {
        // Test that metadata is correctly serialized in atlas JSON
        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [32, 32],
            frames: HashMap::from([(
                "player_attack".to_string(),
                AtlasFrame {
                    x: 0,
                    y: 0,
                    w: 32,
                    h: 32,
                    origin: Some([16, 32]),
                    boxes: Some(HashMap::from([
                        (
                            "hurt".to_string(),
                            AtlasBox {
                                x: 4,
                                y: 0,
                                w: 24,
                                h: 32,
                            },
                        ),
                        (
                            "hit".to_string(),
                            AtlasBox {
                                x: 20,
                                y: 8,
                                w: 20,
                                h: 16,
                            },
                        ),
                    ])),
                },
            )]),
            animations: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();

        // Check that origin is included
        assert!(json.contains("\"origin\""));
        // JSON pretty-print puts spaces after commas in arrays
        assert!(json.contains("16") && json.contains("32"));

        // Check that boxes are included
        assert!(json.contains("\"boxes\""));
        assert!(json.contains("\"hurt\""));
        assert!(json.contains("\"hit\""));
    }
}
