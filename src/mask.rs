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

/// Result of a --query operation: all coordinates where a token appears.
#[derive(Debug)]
pub struct QueryResult {
    /// The token that was queried (bare name, no braces).
    pub token: String,
    /// All (x, y) coordinates where this token appears, sorted.
    pub coords: Vec<(u32, u32)>,
}

/// Result of a --bounds operation: bounding box of a token's extent.
#[derive(Debug)]
pub struct BoundsResult {
    /// The token that was queried (bare name, no braces).
    pub token: String,
    /// Bounding box as [x, y, w, h], or None if token not found.
    pub bounds: Option<[u32; 4]>,
    /// Number of pixels containing this token.
    pub pixel_count: u32,
}

/// Result of a neighbor query at a coordinate.
pub struct NeighborResult {
    /// The token at the queried coordinate.
    pub token: String,
    /// Token above (y-1), if in bounds.
    pub up: Option<String>,
    /// Token below (y+1), if in bounds.
    pub down: Option<String>,
    /// Token to the left (x-1), if in bounds.
    pub left: Option<String>,
    /// Token to the right (x+1), if in bounds.
    pub right: Option<String>,
}

/// Result of a --region operation: tokens in a rectangular slice.
#[derive(Debug)]
pub struct RegionResult {
    /// Origin x of the region.
    pub x: u32,
    /// Origin y of the region.
    pub y: u32,
    /// Width of the region (clamped to grid bounds).
    pub width: u32,
    /// Height of the region (clamped to grid bounds).
    pub height: u32,
    /// 2D grid of tokens in the region, indexed as grid[y][x].
    pub grid: Vec<Vec<String>>,
    /// Whether the requested region was clamped to fit the sprite.
    pub clamped: bool,
}

/// Result of a --count operation: token frequency map.
#[derive(Debug)]
pub struct CountResult {
    /// Token counts sorted by count descending.
    pub tokens: Vec<(String, u32)>,
    /// Total number of pixels.
    pub total: u32,
}

impl TokenGrid {
    /// Return the token at the given coordinate.
    ///
    /// Returns an error if the coordinate is out of bounds.
    pub fn sample(&self, x: u32, y: u32) -> Result<&str, String> {
        if x >= self.width || y >= self.height {
            return Err(format!(
                "coordinate ({}, {}) is out of bounds for {}x{} sprite",
                x, y, self.width, self.height
            ));
        }
        Ok(&self.grid[y as usize][x as usize])
    }

    /// Return the token and its 4-connected neighbors at the given coordinate.
    ///
    /// Out-of-bounds neighbors are omitted (None).
    pub fn neighbors(&self, x: u32, y: u32) -> Result<NeighborResult, String> {
        if x >= self.width || y >= self.height {
            return Err(format!(
                "coordinate ({}, {}) is out of bounds for {}x{} sprite",
                x, y, self.width, self.height
            ));
        }

        let token = self.grid[y as usize][x as usize].clone();
        let up = if y > 0 { Some(self.grid[(y - 1) as usize][x as usize].clone()) } else { None };
        let down = if y + 1 < self.height {
            Some(self.grid[(y + 1) as usize][x as usize].clone())
        } else {
            None
        };
        let left = if x > 0 { Some(self.grid[y as usize][(x - 1) as usize].clone()) } else { None };
        let right = if x + 1 < self.width {
            Some(self.grid[y as usize][(x + 1) as usize].clone())
        } else {
            None
        };

        Ok(NeighborResult { token, up, down, left, right })
    }

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

    /// Find all coordinates where a token appears.
    ///
    /// The `token` parameter is the bare token name (e.g. "skin", "_").
    /// Coordinates are returned sorted by (y, x) order.
    pub fn query(&self, token: &str) -> QueryResult {
        let mut coords = Vec::new();
        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell == token {
                    coords.push((x as u32, y as u32));
                }
            }
        }
        QueryResult { token: token.to_string(), coords }
    }

    /// Extract a rectangular region of tokens from the grid.
    ///
    /// Clamps the requested rectangle to the grid bounds. If the region
    /// extends beyond the sprite, the result contains only the portion
    /// that fits and `clamped` is set to true.
    pub fn region(&self, x: u32, y: u32, w: u32, h: u32) -> RegionResult {
        let clamp_x = x.min(self.width);
        let clamp_y = y.min(self.height);
        let clamp_w = w.min(self.width.saturating_sub(clamp_x));
        let clamp_h = h.min(self.height.saturating_sub(clamp_y));
        let clamped = clamp_x != x || clamp_y != y || clamp_w != w || clamp_h != h;

        let mut grid = Vec::with_capacity(clamp_h as usize);
        for row_idx in clamp_y..(clamp_y + clamp_h) {
            let row = &self.grid[row_idx as usize];
            let slice: Vec<String> = row[clamp_x as usize..(clamp_x + clamp_w) as usize].to_vec();
            grid.push(slice);
        }

        RegionResult { x: clamp_x, y: clamp_y, width: clamp_w, height: clamp_h, grid, clamped }
    }

    /// Build a frequency map of all tokens in the grid.
    ///
    /// Returns tokens sorted by count descending. The total equals
    /// width * height.
    pub fn count(&self) -> CountResult {
        let mut freq: HashMap<String, u32> = HashMap::new();
        for row in &self.grid {
            for cell in row {
                *freq.entry(cell.clone()).or_insert(0) += 1;
            }
        }

        let mut tokens: Vec<(String, u32)> = freq.into_iter().collect();
        tokens.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let total = self.width * self.height;
        CountResult { tokens, total }
    }

    /// Compute the bounding box of a token's extent.
    ///
    /// Returns [x, y, w, h] where (x,y) is the top-left corner
    /// and (w,h) are the dimensions. Returns None bounds if the token
    /// is not found anywhere in the grid.
    pub fn bounds(&self, token: &str) -> BoundsResult {
        let mut min_x: Option<u32> = None;
        let mut min_y: Option<u32> = None;
        let mut max_x: u32 = 0;
        let mut max_y: u32 = 0;
        let mut pixel_count: u32 = 0;

        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell == token {
                    let x = x as u32;
                    let y = y as u32;
                    pixel_count += 1;
                    min_x = Some(min_x.map_or(x, |m: u32| m.min(x)));
                    min_y = Some(min_y.map_or(y, |m: u32| m.min(y)));
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        let bounds = min_x.map(|mx| {
            let my = min_y.unwrap();
            [mx, my, max_x - mx + 1, max_y - my + 1]
        });

        BoundsResult { token: token.to_string(), bounds, pixel_count }
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

    // --- Query tests ---

    #[test]
    fn test_query_finds_all_coords() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.query("x");
        assert_eq!(result.token, "x");
        assert_eq!(result.coords.len(), 4);
        assert!(result.coords.contains(&(1, 1)));
        assert!(result.coords.contains(&(2, 1)));
        assert!(result.coords.contains(&(1, 2)));
        assert!(result.coords.contains(&(2, 2)));
    }

    #[test]
    fn test_query_transparent_token() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // 4x4 grid with 4 "x" pixels = 12 "_" pixels
        let result = grid.query("_");
        assert_eq!(result.coords.len(), 12);
    }

    #[test]
    fn test_query_not_found() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.query("nonexistent");
        assert_eq!(result.coords.len(), 0);
    }

    #[test]
    fn test_query_sorted_by_y_then_x() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.query("x");
        // Should be in row-major order: (1,1), (2,1), (1,2), (2,2)
        assert_eq!(result.coords, vec![(1, 1), (2, 1), (1, 2), (2, 2)]);
    }

    // --- Bounds tests ---

    #[test]
    fn test_bounds_simple_rect() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.bounds("x");
        assert_eq!(result.token, "x");
        assert_eq!(result.bounds, Some([1, 1, 2, 2])); // x=1, y=1, w=2, h=2
        assert_eq!(result.pixel_count, 4);
    }

    #[test]
    fn test_bounds_not_found() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.bounds("nonexistent");
        assert_eq!(result.bounds, None);
        assert_eq!(result.pixel_count, 0);
    }

    #[test]
    fn test_bounds_full_grid() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.bounds("_");
        assert_eq!(result.bounds, Some([0, 0, 4, 4])); // fills entire 4x4 grid
        assert_eq!(result.pixel_count, 12);
    }

    #[test]
    fn test_bounds_with_z_ordering() {
        // Two overlapping regions — higher z wins
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000", "a": "#F00", "b": "#0F0"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4], "z": 0}, "b": {"rect": [1, 1, 2, 2], "z": 1}}}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // "b" occupies 2x2 at (1,1)
        let result = grid.bounds("b");
        assert_eq!(result.bounds, Some([1, 1, 2, 2]));
        assert_eq!(result.pixel_count, 4);

        // "a" occupies the L-shaped border (16 - 4 = 12 pixels)
        let result = grid.bounds("a");
        assert_eq!(result.bounds, Some([0, 0, 4, 4]));
        assert_eq!(result.pixel_count, 12);
    }

    #[test]
    fn test_bounds_single_pixel() {
        // Single pixel token should have w=1, h=1
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000", "x": "#F00"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"x": {"rect": [2, 3, 1, 1], "z": 0}}}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.bounds("x");
        assert_eq!(result.bounds, Some([2, 3, 1, 1]));
        assert_eq!(result.pixel_count, 1);
    }

    // --- Sample tests ---

    #[test]
    fn test_sample_returns_correct_token() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Transparent corner
        assert_eq!(grid.sample(0, 0).unwrap(), "_");
        // Interior rect
        assert_eq!(grid.sample(1, 1).unwrap(), "x");
        assert_eq!(grid.sample(2, 2).unwrap(), "x");
        // Transparent edge
        assert_eq!(grid.sample(3, 0).unwrap(), "_");
    }

    #[test]
    fn test_sample_out_of_bounds() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        assert!(grid.sample(4, 0).is_err());
        assert!(grid.sample(0, 4).is_err());
        assert!(grid.sample(100, 100).is_err());
    }

    // --- Neighbors tests ---

    #[test]
    fn test_neighbors_interior() {
        // Use z-ordering test sprite: "a" fills all, "b" fills [1,1,2,2]
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000", "a": "#F00", "b": "#0F0"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4], "z": 0}, "b": {"rect": [1, 1, 2, 2], "z": 1}}}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Center-ish point (1,1) is "b", neighbors are mixed
        let result = grid.neighbors(1, 1).unwrap();
        assert_eq!(result.token, "b");
        assert_eq!(result.up.as_deref(), Some("a")); // (1,0) = "a"
        assert_eq!(result.down.as_deref(), Some("b")); // (1,2) = "b"
        assert_eq!(result.left.as_deref(), Some("a")); // (0,1) = "a"
        assert_eq!(result.right.as_deref(), Some("b")); // (2,1) = "b"
    }

    #[test]
    fn test_neighbors_corner_top_left() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Top-left corner (0,0): only down and right exist
        let result = grid.neighbors(0, 0).unwrap();
        assert_eq!(result.token, "_");
        assert!(result.up.is_none());
        assert!(result.left.is_none());
        assert!(result.down.is_some());
        assert!(result.right.is_some());
    }

    #[test]
    fn test_neighbors_corner_bottom_right() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Bottom-right corner (3,3): only up and left exist
        let result = grid.neighbors(3, 3).unwrap();
        assert_eq!(result.token, "_");
        assert!(result.up.is_some());
        assert!(result.left.is_some());
        assert!(result.down.is_none());
        assert!(result.right.is_none());
    }

    #[test]
    fn test_neighbors_edge_top() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Top edge (1,0): no up neighbor
        let result = grid.neighbors(1, 0).unwrap();
        assert!(result.up.is_none());
        assert!(result.down.is_some());
        assert!(result.left.is_some());
        assert!(result.right.is_some());
    }

    #[test]
    fn test_neighbors_out_of_bounds() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        assert!(grid.neighbors(4, 0).is_err());
        assert!(grid.neighbors(0, 4).is_err());
    }

    // --- Region tests ---

    #[test]
    fn test_region_full_grid() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.region(0, 0, 4, 4);
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
        assert!(!result.clamped);
        assert_eq!(result.grid.len(), 4);
        assert_eq!(result.grid[1][1], "x");
        assert_eq!(result.grid[0][0], "_");
    }

    #[test]
    fn test_region_subslice() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Slice only the inner 2x2 rect
        let result = grid.region(1, 1, 2, 2);
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
        assert!(!result.clamped);
        assert_eq!(result.grid, vec![vec!["x", "x"], vec!["x", "x"]]);
    }

    #[test]
    fn test_region_clamped() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Request extends beyond sprite bounds
        let result = grid.region(2, 2, 10, 10);
        assert_eq!(result.x, 2);
        assert_eq!(result.y, 2);
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
        assert!(result.clamped);
    }

    #[test]
    fn test_region_out_of_bounds_origin() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        // Origin beyond sprite
        let result = grid.region(10, 10, 2, 2);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
        assert!(result.clamped);
        assert!(result.grid.is_empty());
    }

    // --- Count tests ---

    #[test]
    fn test_count_simple() {
        let pipeline = MaskPipeline::load_from_string(SIMPLE_SPRITE, Some("dot")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.count();
        assert_eq!(result.total, 16);
        // Sorted by count descending: "_" (12) then "x" (4)
        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0], ("_".to_string(), 12));
        assert_eq!(result.tokens[1], ("x".to_string(), 4));
    }

    #[test]
    fn test_count_with_z_ordering() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000", "a": "#F00", "b": "#0F0"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"a": {"rect": [0, 0, 4, 4], "z": 0}, "b": {"rect": [1, 1, 2, 2], "z": 1}}}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.count();
        assert_eq!(result.total, 16);
        // "a" covers 16 but "b" overwrites 4 → a=12, b=4
        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0], ("a".to_string(), 12));
        assert_eq!(result.tokens[1], ("b".to_string(), 4));
    }

    #[test]
    fn test_count_empty_sprite() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"_": "#0000"}}
{"type": "sprite", "name": "empty", "size": [3, 3], "palette": "p"}"##;

        let pipeline = MaskPipeline::load_from_string(content, Some("empty")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.count();
        assert_eq!(result.total, 9);
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0], ("_".to_string(), 9));
    }

    #[test]
    fn test_count_percentages_sum() {
        let pipeline = MaskPipeline::load_from_string(MULTI_REGION, Some("s")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        let grid = TokenGrid::from_sprite(sprite).unwrap();

        let result = grid.count();
        let sum: u32 = result.tokens.iter().map(|(_, c)| c).sum();
        assert_eq!(sum, result.total);
    }
}
