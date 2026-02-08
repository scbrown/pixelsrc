//! Core mask logic: read-only sprite state queries.
//!
//! Provides token grid extraction from region-based sprites.
//! The token grid is a 2D array where each cell contains the token name
//! at that pixel position, enabling structured queries without visual parsing.

use crate::draw::DrawError;
use crate::models::{RegionDef, Sprite, TtpObject};
use crate::renderer::Warning;
use crate::structured::rasterize_region;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// A read-only view of a parsed .pxl file for mask queries.
pub struct MaskPipeline {
    /// All parsed objects from the file.
    objects: Vec<TtpObject>,
    /// Index of the target sprite.
    sprite_index: Option<usize>,
}

impl MaskPipeline {
    /// Load a .pxl file for read-only queries.
    pub fn load(path: &Path, sprite_name: Option<&str>) -> Result<Self, DrawError> {
        let content = std::fs::read_to_string(path).map_err(DrawError::Io)?;
        Self::load_from_string(&content, sprite_name)
    }

    /// Load from a string (for testing).
    pub fn load_from_string(content: &str, sprite_name: Option<&str>) -> Result<Self, DrawError> {
        use crate::parser::parse_stream;
        use std::io::Cursor;

        let reader = Cursor::new(content);
        let parse_result = parse_stream(reader);

        if parse_result.objects.is_empty() && !parse_result.warnings.is_empty() {
            return Err(DrawError::ParseError(
                parse_result.warnings.first().map(|w| w.message.clone()).unwrap_or_default(),
            ));
        }

        let sprite_index = if let Some(name) = sprite_name {
            let idx = parse_result.objects.iter().position(|obj| match obj {
                TtpObject::Sprite(s) => s.name == name,
                _ => false,
            });
            match idx {
                Some(i) => Some(i),
                None => return Err(DrawError::SpriteNotFound(name.to_string())),
            }
        } else {
            None
        };

        Ok(MaskPipeline { objects: parse_result.objects, sprite_index })
    }

    /// Get a reference to the target sprite.
    pub fn sprite(&self) -> Option<&Sprite> {
        self.sprite_index.and_then(|i| match &self.objects[i] {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
    }

    /// Get all sprite names in the file.
    pub fn sprite_names(&self) -> Vec<&str> {
        self.objects
            .iter()
            .filter_map(|obj| match obj {
                TtpObject::Sprite(s) => Some(s.name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Get all objects (for --list metadata queries).
    pub fn objects(&self) -> &[TtpObject] {
        &self.objects
    }
}

/// A 2D token grid extracted from a sprite's regions.
///
/// Each cell contains the token name (e.g. "skin", "eye", "_") at that position.
/// The grid dimensions match the sprite's size.
pub struct TokenGrid {
    /// 2D grid indexed as grid[y][x].
    pub grid: Vec<Vec<String>>,
    /// Width of the grid.
    pub width: u32,
    /// Height of the grid.
    pub height: u32,
}

impl TokenGrid {
    /// Build a token grid from a sprite's regions.
    ///
    /// Rasterizes all regions and assigns each pixel to the token that
    /// renders last (highest z-order), matching the visual rendering order.
    pub fn from_sprite(sprite: &Sprite) -> Result<Self, String> {
        let [width, height] =
            sprite.size.ok_or_else(|| format!("sprite '{}' has no size defined", sprite.name))?;

        if width == 0 || height == 0 {
            return Err(format!("sprite '{}' has invalid size: {}x{}", sprite.name, width, height));
        }

        // Initialize grid with transparent token
        let mut grid = vec![vec!["_".to_string(); width as usize]; height as usize];

        let regions = match &sprite.regions {
            Some(r) => r,
            None => return Ok(TokenGrid { grid, width, height }),
        };

        // Rasterize all regions using the same two-pass approach as render_structured
        let mut rasterized: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
        let mut pending: Vec<(String, &RegionDef)> = Vec::new();
        let mut warnings: Vec<Warning> = Vec::new();

        for (token, region) in regions {
            if region.fill.is_some() || region.auto_shadow.is_some() {
                pending.push((token.clone(), region));
            } else {
                let pixels = rasterize_region(
                    region,
                    &rasterized,
                    width as i32,
                    height as i32,
                    &mut warnings,
                );
                rasterized.insert(token.clone(), pixels);
            }
        }

        for (token, region) in pending {
            let pixels =
                rasterize_region(region, &rasterized, width as i32, height as i32, &mut warnings);
            rasterized.insert(token, pixels);
        }

        // Sort by z-order (lowest first, so highest z renders last / wins)
        let mut region_order: Vec<(String, i32)> = regions
            .iter()
            .map(|(token, region)| {
                let z = region.z.unwrap_or_else(|| default_z_for_role(region.role.as_ref()));
                (token.clone(), z)
            })
            .collect();
        region_order.sort_by_key(|(_, z)| *z);

        // Fill grid in z-order (last writer wins, matching render_structured)
        for (token, _z) in &region_order {
            if let Some(pixels) = rasterized.get(token) {
                for &(x, y) in pixels {
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        grid[y as usize][x as usize] = token.clone();
                    }
                }
            }
        }

        Ok(TokenGrid { grid, width, height })
    }
}

/// Default z-order for semantic roles (mirrors structured.rs logic).
fn default_z_for_role(role: Option<&crate::models::Role>) -> i32 {
    use crate::models::Role;
    match role {
        Some(Role::Anchor) => 100,
        Some(Role::Boundary) => 80,
        Some(Role::Shadow) | Some(Role::Highlight) => 60,
        Some(Role::Fill) => 40,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SPRITE: &str = r##"{"type": "palette", "name": "colors", "colors": {"_": "#00000000", "x": "#FF0000"}}
{"type": "sprite", "name": "dot", "size": [4, 4], "palette": "colors", "regions": {"x": {"rect": [1, 1, 2, 2], "z": 0}}}"##;

    const MULTI_REGION: &str = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000", "o": "#000", "f": "#F00"}}
{"type": "sprite", "name": "s", "size": [8, 8], "palette": "p", "regions": {"o": {"stroke": [0, 0, 8, 8]}, "f": {"fill": "inside(o)", "z": 1}}}"##;

    const TWO_SPRITES: &str = r##"{"type": "palette", "name": "colors", "colors": {"_": "#00000000", "x": "#FF0000", "y": "#00FF00"}}
{"type": "sprite", "name": "first", "size": [4, 4], "palette": "colors", "regions": {"x": {"rect": [0, 0, 4, 4], "z": 0}}}
{"type": "sprite", "name": "second", "size": [8, 8], "palette": "colors", "regions": {"y": {"rect": [0, 0, 8, 8], "z": 0}}}"##;

    #[test]
    fn test_mask_pipeline_load() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        assert_eq!(sprite.name, "dot");
        assert_eq!(sprite.size, Some([4, 4]));
    }

    #[test]
    fn test_mask_pipeline_sprite_not_found() {
        let result = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_mask_pipeline_sprite_names() {
        let pipeline = MaskPipeline::load_from_string(TWO_SPRITES, None).unwrap();
        let names = pipeline.sprite_names();
        assert_eq!(names, vec!["first", "second"]);
    }

    #[test]
    fn test_token_grid_simple_rect() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 4);

        // Corners should be transparent
        assert_eq!(grid.grid[0][0], "_");
        assert_eq!(grid.grid[0][3], "_");
        assert_eq!(grid.grid[3][0], "_");
        assert_eq!(grid.grid[3][3], "_");

        // Interior rect [1,1,2,2] should be "x"
        assert_eq!(grid.grid[1][1], "x");
        assert_eq!(grid.grid[1][2], "x");
        assert_eq!(grid.grid[2][1], "x");
        assert_eq!(grid.grid[2][2], "x");
    }

    #[test]
    fn test_token_grid_with_fill() {
        let pipeline = MaskPipeline::load_from_string(MULTI_REGION, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        assert_eq!(grid.width, 8);
        assert_eq!(grid.height, 8);

        // Border should be "o" (stroke)
        assert_eq!(grid.grid[0][0], "o");
        assert_eq!(grid.grid[0][7], "o");

        // Interior should be "f" (fill with higher z)
        assert_eq!(grid.grid[2][2], "f");
        assert_eq!(grid.grid[5][5], "f");
    }

    #[test]
    fn test_token_grid_no_regions() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000"}}
{"type": "sprite", "name": "empty", "size": [3, 3], "palette": "p"}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("empty")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // All cells should be transparent
        for row in &grid.grid {
            for cell in row {
                assert_eq!(cell, "_");
            }
        }
    }

    #[test]
    fn test_token_grid_no_size() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000"}}
{"type": "sprite", "name": "nosize", "palette": "p"}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("nosize")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let result = TokenGrid::from_sprite(sprite);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_grid_z_ordering() {
        // Two overlapping regions — higher z wins
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000", "a": "#F00", "b": "#0F0"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4], "z": 0}, "b": {"rect": [1, 1, 2, 2], "z": 1}}}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Outer area should be "a"
        assert_eq!(grid.grid[0][0], "a");
        // Inner area should be "b" (higher z)
        assert_eq!(grid.grid[1][1], "b");
        assert_eq!(grid.grid[2][2], "b");
    }
}
