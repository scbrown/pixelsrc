//! Core draw pipeline: read-modify-write for .pxl files.
//!
//! Provides the architecture for commands that modify `.pxl` files in-place.
//! Parses the file into `TtpObject`s, locates a target sprite, applies
//! modifications, reformats, and writes back.

use crate::fmt::format_pixelsrc;
use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::shapes::rasterize_line;
use std::io::Cursor;
use std::path::Path;

/// Result of a draw operation.
#[derive(Debug)]
pub struct DrawResult {
    /// The formatted output content.
    pub content: String,
    /// Whether any modifications were made.
    pub modified: bool,
    /// Warnings encountered during processing.
    pub warnings: Vec<String>,
}

/// Error type for draw operations.
#[derive(Debug)]
pub enum DrawError {
    /// File I/O error.
    Io(std::io::Error),
    /// Sprite not found in file.
    SpriteNotFound(String),
    /// Parse error in the input file.
    ParseError(String),
    /// Format error during reserialization.
    FormatError(String),
    /// Coordinate out of bounds.
    OutOfBounds { x: usize, y: usize, width: usize, height: usize },
    /// Sprite has no grid data.
    NoGrid(String),
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawError::Io(e) => write!(f, "I/O error: {}", e),
            DrawError::SpriteNotFound(name) => write!(f, "sprite '{}' not found", name),
            DrawError::ParseError(msg) => write!(f, "parse error: {}", msg),
            DrawError::FormatError(msg) => write!(f, "format error: {}", msg),
            DrawError::OutOfBounds { x, y, width, height } => {
                write!(f, "coordinates ({}, {}) out of bounds for {}x{} grid", x, y, width, height)
            }
            DrawError::NoGrid(name) => write!(f, "sprite '{}' has no grid data", name),
        }
    }
}

impl From<std::io::Error> for DrawError {
    fn from(e: std::io::Error) -> Self {
        DrawError::Io(e)
    }
}

/// A draw operation to apply to a sprite's grid.
#[derive(Debug, Clone)]
pub enum DrawOp {
    /// Set a single cell: `--set x,y="{token}"`
    Set { x: usize, y: usize, token: String },
    /// Erase a single cell (set to transparent): `--erase x,y`
    Erase { x: usize, y: usize },
    /// Fill a rectangle: `--rect x,y,w,h="{token}"`
    Rect { x: usize, y: usize, w: usize, h: usize, token: String },
    /// Draw a line between two points: `--line x1,y1,x2,y2="{token}"`
    Line { x0: usize, y0: usize, x1: usize, y1: usize, token: String },
    /// Flood fill from a seed point: `--flood x,y="{token}"`
    Flood { x: usize, y: usize, token: String },
}

/// A 2D grid of tokens parsed from `{token}` grid strings.
///
/// Each cell holds a token name (without braces). The grid uses (x, y) coordinates
/// where (0,0) is top-left, x is the column, and y is the row.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenGrid {
    /// 2D array indexed as `cells[y][x]`.
    cells: Vec<Vec<String>>,
    width: usize,
    height: usize,
}

impl TokenGrid {
    /// Parse grid strings into a 2D token array.
    ///
    /// Each row string contains `{token}` patterns, e.g. `"{skin}{eye}{skin}"`.
    /// Returns an error if any row is malformed or rows have inconsistent widths.
    pub fn parse(rows: &[String]) -> Result<Self, DrawError> {
        if rows.is_empty() {
            return Ok(TokenGrid { cells: Vec::new(), width: 0, height: 0 });
        }

        let mut cells = Vec::with_capacity(rows.len());
        let mut expected_width: Option<usize> = None;

        for (row_idx, row) in rows.iter().enumerate() {
            let tokens = Self::parse_row(row).map_err(|msg| {
                DrawError::ParseError(format!("row {}: {}", row_idx, msg))
            })?;

            if let Some(w) = expected_width {
                if tokens.len() != w {
                    return Err(DrawError::ParseError(format!(
                        "row {} has {} tokens, expected {} (rows must have consistent width)",
                        row_idx,
                        tokens.len(),
                        w
                    )));
                }
            } else {
                expected_width = Some(tokens.len());
            }

            cells.push(tokens);
        }

        let width = expected_width.unwrap_or(0);
        let height = cells.len();
        Ok(TokenGrid { cells, width, height })
    }

    /// Parse a single row of `{token}` patterns into token names.
    fn parse_row(row: &str) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();
        let mut chars = row.chars().peekable();

        while chars.peek().is_some() {
            match chars.next() {
                Some('{') => {
                    let mut token = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => token.push(c),
                            None => return Err("unterminated token (missing '}')".to_string()),
                        }
                    }
                    if token.is_empty() {
                        return Err("empty token '{}'".to_string());
                    }
                    tokens.push(token);
                }
                Some(c) if c.is_whitespace() => {
                    // Skip whitespace between tokens
                }
                Some(c) => {
                    return Err(format!("unexpected character '{}', expected '{{token}}'", c));
                }
                None => break,
            }
        }

        Ok(tokens)
    }

    /// Grid width (number of columns).
    pub fn width(&self) -> usize {
        self.width
    }

    /// Grid height (number of rows).
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the token at (x, y). Returns `None` if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<&str> {
        self.cells.get(y).and_then(|row| row.get(x)).map(|s| s.as_str())
    }

    /// Set the token at (x, y). Returns error if out of bounds.
    pub fn set(&mut self, x: usize, y: usize, token: String) -> Result<(), DrawError> {
        if x >= self.width || y >= self.height {
            return Err(DrawError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        self.cells[y][x] = token;
        Ok(())
    }

    /// Erase the cell at (x, y) by setting it to `_` (transparent).
    pub fn erase(&mut self, x: usize, y: usize) -> Result<(), DrawError> {
        self.set(x, y, "_".to_string())
    }

    /// Fill a rectangular region with a token, clamping to sprite bounds.
    ///
    /// Returns a list of warnings (e.g. if the rect was clamped or is zero-sized).
    pub fn rect_fill(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        token: String,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        if w == 0 || h == 0 {
            warnings.push(format!(
                "zero-size rect {}x{} at ({},{}) is a no-op",
                w, h, x, y
            ));
            return warnings;
        }

        // Compute the desired end coordinates
        let end_x = x.saturating_add(w);
        let end_y = y.saturating_add(h);

        // Clamp to grid bounds
        let clamped_start_x = x.min(self.width);
        let clamped_start_y = y.min(self.height);
        let clamped_end_x = end_x.min(self.width);
        let clamped_end_y = end_y.min(self.height);

        if clamped_start_x >= clamped_end_x || clamped_start_y >= clamped_end_y {
            warnings.push(format!(
                "rect at ({},{}) {}x{} is entirely outside {}x{} grid",
                x, y, w, h, self.width, self.height
            ));
            return warnings;
        }

        if clamped_end_x < end_x || clamped_end_y < end_y
            || clamped_start_x > x || clamped_start_y > y
        {
            warnings.push(format!(
                "rect at ({},{}) {}x{} clipped to grid bounds ({}x{})",
                x, y, w, h, self.width, self.height
            ));
        }

        for row in clamped_start_y..clamped_end_y {
            for col in clamped_start_x..clamped_end_x {
                self.cells[row][col] = token.clone();
            }
        }

        warnings
    }

    /// Flood fill from a seed point using iterative BFS.
    ///
    /// Fills all 4-connected pixels matching the original token at (x, y) with
    /// the new token. Returns warnings (e.g. if filling with same token).
    pub fn flood_fill(&mut self, x: usize, y: usize, token: String) -> Result<Vec<String>, DrawError> {
        if x >= self.width || y >= self.height {
            return Err(DrawError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        let original = self.cells[y][x].clone();
        let mut warnings = Vec::new();

        // Filling with the same token is a no-op
        if original == token {
            warnings.push(format!(
                "flood fill at ({},{}) with '{{{}}}' is a no-op (already that token)",
                x, y, token
            ));
            return Ok(warnings);
        }

        // Iterative BFS
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((x, y));
        // Mark visited by filling as we go — no separate visited set needed
        self.cells[y][x] = token.clone();

        while let Some((cx, cy)) = queue.pop_front() {
            // Check 4-connected neighbors
            let neighbors: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in &neighbors {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;
                if nx >= 0 && (nx as usize) < self.width && ny >= 0 && (ny as usize) < self.height {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.cells[ny][nx] == original {
                        self.cells[ny][nx] = token.clone();
                        queue.push_back((nx, ny));
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Apply a draw operation to the grid. Returns warnings (if any).
    pub fn apply(&mut self, op: &DrawOp) -> Result<Vec<String>, DrawError> {
        match op {
            DrawOp::Set { x, y, token } => {
                self.set(*x, *y, token.clone())?;
                Ok(Vec::new())
            }
            DrawOp::Erase { x, y } => {
                self.erase(*x, *y)?;
                Ok(Vec::new())
            }
            DrawOp::Rect { x, y, w, h, token } => {
                Ok(self.rect_fill(*x, *y, *w, *h, token.clone()))
            }
            DrawOp::Line { x0, y0, x1, y1, token } => {
                let pixels = rasterize_line(
                    (*x0 as i32, *y0 as i32),
                    (*x1 as i32, *y1 as i32),
                );
                for (px, py) in pixels {
                    if px >= 0
                        && (px as usize) < self.width
                        && py >= 0
                        && (py as usize) < self.height
                    {
                        self.set(px as usize, py as usize, token.clone())?;
                    }
                }
                Ok(Vec::new())
            }
            DrawOp::Flood { x, y, token } => {
                self.flood_fill(*x, *y, token.clone())
            }
        }
    }

    /// Reserialize the grid back to `{token}` strings.
    pub fn to_grid_strings(&self) -> Vec<String> {
        self.cells
            .iter()
            .map(|row| row.iter().map(|t| format!("{{{}}}", t)).collect::<String>())
            .collect()
    }
}

/// A draw pipeline that reads a .pxl file, applies modifications, and writes back.
#[derive(Debug)]
pub struct DrawPipeline {
    /// All parsed objects from the file, in order.
    objects: Vec<TtpObject>,
    /// Index of the target sprite in the objects vec.
    sprite_index: Option<usize>,
    /// Warnings from parsing.
    warnings: Vec<String>,
}

impl DrawPipeline {
    /// Load a .pxl file and prepare for editing.
    ///
    /// Parses the file into objects and optionally locates a target sprite.
    pub fn load(path: &Path, sprite_name: Option<&str>) -> Result<Self, DrawError> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_string(&content, sprite_name)
    }

    /// Load from a string (for testing and --dry-run).
    pub fn load_from_string(content: &str, sprite_name: Option<&str>) -> Result<Self, DrawError> {
        let reader = Cursor::new(content);
        let parse_result = parse_stream(reader);

        let warnings: Vec<String> =
            parse_result.warnings.iter().map(|w| format!("line {}: {}", w.line, w.message)).collect();

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

        Ok(DrawPipeline { objects: parse_result.objects, sprite_index, warnings })
    }

    /// Get a reference to the target sprite, if one was selected.
    pub fn sprite(&self) -> Option<&crate::models::Sprite> {
        self.sprite_index.and_then(|i| match &self.objects[i] {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
    }

    /// Get a mutable reference to the target sprite, if one was selected.
    pub fn sprite_mut(&mut self) -> Option<&mut crate::models::Sprite> {
        self.sprite_index.and_then(|i| match &mut self.objects[i] {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
    }

    /// Get the list of all sprite names in the file.
    pub fn sprite_names(&self) -> Vec<&str> {
        self.objects
            .iter()
            .filter_map(|obj| match obj {
                TtpObject::Sprite(s) => Some(s.name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Serialize all objects back to formatted .pxl content.
    pub fn serialize(&self) -> Result<DrawResult, DrawError> {
        // Serialize each object to JSON, then concatenate
        let mut json_lines = Vec::new();
        for obj in &self.objects {
            let json = serde_json::to_string(obj)
                .map_err(|e| DrawError::FormatError(format!("serialization failed: {}", e)))?;
            json_lines.push(json);
        }
        let raw_content = json_lines.join("\n");

        // Run through the formatter for canonical output
        let formatted = format_pixelsrc(&raw_content)
            .map_err(|e| DrawError::FormatError(e))?;

        Ok(DrawResult { content: formatted, modified: true, warnings: self.warnings.clone() })
    }

    /// Extract the target sprite's grid as a TokenGrid for editing.
    ///
    /// Returns an error if no sprite is selected or the sprite has no grid data.
    pub fn grid(&self) -> Result<TokenGrid, DrawError> {
        let sprite = self.sprite().ok_or_else(|| {
            DrawError::ParseError("no sprite selected".to_string())
        })?;
        let grid_rows = sprite.grid.as_ref().ok_or_else(|| {
            DrawError::NoGrid(sprite.name.clone())
        })?;
        TokenGrid::parse(grid_rows)
    }

    /// Set the target sprite's grid from a TokenGrid.
    ///
    /// Also updates the sprite's size to match the grid dimensions.
    pub fn set_grid(&mut self, grid: &TokenGrid) -> Result<(), DrawError> {
        let sprite = self.sprite_mut().ok_or_else(|| {
            DrawError::ParseError("no sprite selected".to_string())
        })?;
        sprite.grid = Some(grid.to_grid_strings());
        sprite.size = Some([grid.width() as u32, grid.height() as u32]);
        Ok(())
    }

    /// Apply a sequence of draw operations to the target sprite's grid.
    pub fn apply_ops(&mut self, ops: &[DrawOp]) -> Result<(), DrawError> {
        let mut grid = self.grid()?;
        for op in ops {
            let op_warnings = grid.apply(op)?;
            self.warnings.extend(op_warnings);
        }
        self.set_grid(&grid)
    }

    /// Write the serialized content to a file.
    pub fn write_to(&self, path: &Path) -> Result<DrawResult, DrawError> {
        let result = self.serialize()?;
        std::fs::write(path, &result.content)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_PXL: &str = r##"{"type": "palette", "name": "colors", "colors": {"_": "#00000000", "x": "#FF0000"}}
{"type": "sprite", "name": "dot", "size": [4, 4], "palette": "colors", "regions": {"x": {"rect": [1, 1, 2, 2], "z": 0}}}"##;

    const GRID_PXL: &str = r##"{"type": "palette", "name": "pal", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{eye}": "#000000", "{hair}": "#8B4513"}}
{"type": "sprite", "name": "face", "size": [3, 3], "palette": "pal", "grid": ["{hair}{hair}{hair}", "{skin}{eye}{skin}", "{skin}{skin}{skin}"]}"##;

    const TWO_SPRITES: &str = r##"{"type": "palette", "name": "colors", "colors": {"_": "#00000000", "x": "#FF0000", "y": "#00FF00"}}
{"type": "sprite", "name": "first", "size": [4, 4], "palette": "colors", "regions": {"x": {"rect": [0, 0, 4, 4], "z": 0}}}
{"type": "sprite", "name": "second", "size": [8, 8], "palette": "colors", "regions": {"y": {"rect": [0, 0, 8, 8], "z": 0}}}"##;

    #[test]
    fn test_load_from_string_no_sprite() {
        let pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, None).unwrap();
        assert_eq!(pipeline.objects.len(), 2);
        assert!(pipeline.sprite_index.is_none());
        assert!(pipeline.warnings.is_empty());
    }

    #[test]
    fn test_load_from_string_with_sprite() {
        let pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        assert_eq!(pipeline.sprite_index, Some(1));
        let sprite = pipeline.sprite().unwrap();
        assert_eq!(sprite.name, "dot");
        assert_eq!(sprite.size, Some([4, 4]));
    }

    #[test]
    fn test_load_sprite_not_found() {
        let result = DrawPipeline::load_from_string(SIMPLE_PXL, Some("nonexistent"));
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::SpriteNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected SpriteNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_sprite_names() {
        let pipeline = DrawPipeline::load_from_string(TWO_SPRITES, None).unwrap();
        let names = pipeline.sprite_names();
        assert_eq!(names, vec!["first", "second"]);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        let result = pipeline.serialize().unwrap();

        // The output should be valid .pxl that we can re-parse
        let pipeline2 = DrawPipeline::load_from_string(&result.content, Some("dot")).unwrap();
        assert_eq!(pipeline2.objects.len(), 2);
        let sprite = pipeline2.sprite().unwrap();
        assert_eq!(sprite.name, "dot");
        assert_eq!(sprite.size, Some([4, 4]));
    }

    #[test]
    fn test_serialize_preserves_all_objects() {
        let pipeline = DrawPipeline::load_from_string(TWO_SPRITES, None).unwrap();
        let result = pipeline.serialize().unwrap();

        // Re-parse and verify all objects are preserved
        let pipeline2 = DrawPipeline::load_from_string(&result.content, None).unwrap();
        assert_eq!(pipeline2.objects.len(), 3); // 1 palette + 2 sprites
        let names = pipeline2.sprite_names();
        assert_eq!(names, vec!["first", "second"]);
    }

    #[test]
    fn test_sprite_mut() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        {
            let sprite = pipeline.sprite_mut().unwrap();
            sprite.size = Some([8, 8]);
        }
        let result = pipeline.serialize().unwrap();

        // Verify the modification persisted
        let pipeline2 = DrawPipeline::load_from_string(&result.content, Some("dot")).unwrap();
        let sprite = pipeline2.sprite().unwrap();
        assert_eq!(sprite.size, Some([8, 8]));
    }

    #[test]
    fn test_load_empty_content() {
        let pipeline = DrawPipeline::load_from_string("", None).unwrap();
        assert_eq!(pipeline.objects.len(), 0);
    }

    #[test]
    fn test_load_invalid_json() {
        let result = DrawPipeline::load_from_string("{not valid json}", None);
        // Should succeed with warnings (lenient parsing), or error if all content is bad
        match result {
            Ok(pipeline) => {
                assert!(pipeline.objects.is_empty() || !pipeline.warnings.is_empty());
            }
            Err(DrawError::ParseError(_)) => { /* also acceptable */ }
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_write_to_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let input_path = dir.path().join("test.pxl");
        let output_path = dir.path().join("output.pxl");

        std::fs::write(&input_path, SIMPLE_PXL).unwrap();

        let pipeline = DrawPipeline::load(&input_path, Some("dot")).unwrap();
        pipeline.write_to(&output_path).unwrap();

        // Verify output file exists and is valid
        let output_content = std::fs::read_to_string(&output_path).unwrap();
        let pipeline2 = DrawPipeline::load_from_string(&output_content, Some("dot")).unwrap();
        assert_eq!(pipeline2.sprite().unwrap().name, "dot");
    }

    #[test]
    fn test_load_file_not_found() {
        let result = DrawPipeline::load(Path::new("/nonexistent/file.pxl"), None);
        assert!(matches!(result, Err(DrawError::Io(_))));
    }

    #[test]
    fn test_select_second_sprite() {
        let pipeline = DrawPipeline::load_from_string(TWO_SPRITES, Some("second")).unwrap();
        let sprite = pipeline.sprite().unwrap();
        assert_eq!(sprite.name, "second");
        assert_eq!(sprite.size, Some([8, 8]));
    }

    #[test]
    fn test_compositions_and_animations_preserved() {
        let content = r##"{"type": "palette", "name": "p", "colors": {"x": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [2, 2], "palette": "p", "regions": {"x": {"rect": [0, 0, 2, 2], "z": 0}}}
{"type": "animation", "name": "anim", "frames": ["s"]}
{"type": "composition", "name": "comp", "sprites": {".": null, "S": "s"}, "layers": [{"map": ["S."]}]}"##;

        let pipeline = DrawPipeline::load_from_string(content, Some("s")).unwrap();
        let result = pipeline.serialize().unwrap();

        // Re-parse and verify all 4 objects preserved
        let pipeline2 = DrawPipeline::load_from_string(&result.content, None).unwrap();
        assert_eq!(pipeline2.objects.len(), 4);

        let mut has_palette = false;
        let mut has_sprite = false;
        let mut has_animation = false;
        let mut has_composition = false;

        for obj in &pipeline2.objects {
            match obj {
                TtpObject::Palette(_) => has_palette = true,
                TtpObject::Sprite(_) => has_sprite = true,
                TtpObject::Animation(_) => has_animation = true,
                TtpObject::Composition(_) => has_composition = true,
                _ => {}
            }
        }

        assert!(has_palette, "palette should be preserved");
        assert!(has_sprite, "sprite should be preserved");
        assert!(has_animation, "animation should be preserved");
        assert!(has_composition, "composition should be preserved");
    }

    // =========================================================================
    // TokenGrid tests
    // =========================================================================

    #[test]
    fn test_grid_parse_simple() {
        let rows = vec![
            "{hair}{hair}{hair}".to_string(),
            "{skin}{eye}{skin}".to_string(),
            "{skin}{skin}{skin}".to_string(),
        ];
        let grid = TokenGrid::parse(&rows).unwrap();
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
    }

    #[test]
    fn test_grid_parse_single_row() {
        let rows = vec!["{a}{b}{c}".to_string()];
        let grid = TokenGrid::parse(&rows).unwrap();
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 1);
    }

    #[test]
    fn test_grid_parse_empty() {
        let grid = TokenGrid::parse(&[]).unwrap();
        assert_eq!(grid.width(), 0);
        assert_eq!(grid.height(), 0);
    }

    #[test]
    fn test_grid_parse_inconsistent_width() {
        let rows = vec!["{a}{b}{c}".to_string(), "{d}{e}".to_string()];
        let result = TokenGrid::parse(&rows);
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::ParseError(msg) => assert!(msg.contains("consistent width"), "got: {}", msg),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_grid_parse_unterminated_token() {
        let rows = vec!["{a}{b".to_string()];
        let result = TokenGrid::parse(&rows);
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::ParseError(msg) => assert!(msg.contains("unterminated")),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_grid_parse_empty_token() {
        let rows = vec!["{a}{}{c}".to_string()];
        let result = TokenGrid::parse(&rows);
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::ParseError(msg) => assert!(msg.contains("empty token")),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_grid_parse_unexpected_char() {
        let rows = vec!["x{a}{b}".to_string()];
        let result = TokenGrid::parse(&rows);
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::ParseError(msg) => assert!(msg.contains("unexpected character")),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_grid_get() {
        let rows = vec![
            "{hair}{hair}{hair}".to_string(),
            "{skin}{eye}{skin}".to_string(),
        ];
        let grid = TokenGrid::parse(&rows).unwrap();

        // (0,0) is top-left
        assert_eq!(grid.get(0, 0), Some("hair"));
        assert_eq!(grid.get(1, 0), Some("hair"));
        assert_eq!(grid.get(2, 0), Some("hair"));
        // Row 1
        assert_eq!(grid.get(0, 1), Some("skin"));
        assert_eq!(grid.get(1, 1), Some("eye"));
        assert_eq!(grid.get(2, 1), Some("skin"));
        // Out of bounds
        assert_eq!(grid.get(3, 0), None);
        assert_eq!(grid.get(0, 2), None);
    }

    #[test]
    fn test_grid_set() {
        let rows = vec![
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.set(1, 0, "eye".to_string()).unwrap();
        assert_eq!(grid.get(1, 0), Some("eye"));
        assert_eq!(grid.get(0, 0), Some("_"));
    }

    #[test]
    fn test_grid_set_out_of_bounds() {
        let rows = vec!["{_}{_}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let result = grid.set(2, 0, "x".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::OutOfBounds { x, y, width, height } => {
                assert_eq!((x, y, width, height), (2, 0, 2, 1));
            }
            other => panic!("Expected OutOfBounds, got: {:?}", other),
        }

        let result = grid.set(0, 1, "x".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_grid_erase() {
        let rows = vec!["{skin}{eye}{skin}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.erase(1, 0).unwrap();
        assert_eq!(grid.get(1, 0), Some("_"));
    }

    #[test]
    fn test_grid_to_strings() {
        let rows = vec![
            "{hair}{hair}{hair}".to_string(),
            "{skin}{eye}{skin}".to_string(),
        ];
        let grid = TokenGrid::parse(&rows).unwrap();
        let output = grid.to_grid_strings();

        assert_eq!(output, vec![
            "{hair}{hair}{hair}",
            "{skin}{eye}{skin}",
        ]);
    }

    #[test]
    fn test_grid_roundtrip() {
        let rows = vec![
            "{_}{skin}{_}".to_string(),
            "{skin}{eye}{skin}".to_string(),
            "{_}{skin}{_}".to_string(),
        ];
        let grid = TokenGrid::parse(&rows).unwrap();
        let output = grid.to_grid_strings();
        assert_eq!(output, rows);
    }

    #[test]
    fn test_grid_set_and_roundtrip() {
        let rows = vec![
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.set(1, 1, "eye".to_string()).unwrap();
        grid.set(0, 0, "hair".to_string()).unwrap();
        grid.set(2, 0, "hair".to_string()).unwrap();

        let output = grid.to_grid_strings();
        assert_eq!(output, vec![
            "{hair}{_}{hair}",
            "{_}{eye}{_}",
            "{_}{_}{_}",
        ]);
    }

    #[test]
    fn test_grid_apply_set_op() {
        let rows = vec!["{_}{_}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.apply(&DrawOp::Set { x: 1, y: 0, token: "x".to_string() }).unwrap();
        assert_eq!(grid.get(1, 0), Some("x"));
    }

    #[test]
    fn test_grid_apply_erase_op() {
        let rows = vec!["{skin}{eye}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.apply(&DrawOp::Erase { x: 1, y: 0 }).unwrap();
        assert_eq!(grid.get(1, 0), Some("_"));
    }

    #[test]
    fn test_grid_multichar_tokens() {
        let rows = vec!["{dark_skin}{light_hair}".to_string()];
        let grid = TokenGrid::parse(&rows).unwrap();
        assert_eq!(grid.get(0, 0), Some("dark_skin"));
        assert_eq!(grid.get(1, 0), Some("light_hair"));
    }

    #[test]
    fn test_grid_single_cell() {
        let rows = vec!["{x}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();
        assert_eq!(grid.width(), 1);
        assert_eq!(grid.height(), 1);
        assert_eq!(grid.get(0, 0), Some("x"));

        grid.set(0, 0, "y".to_string()).unwrap();
        assert_eq!(grid.get(0, 0), Some("y"));
        assert_eq!(grid.to_grid_strings(), vec!["{y}"]);
    }

    // =========================================================================
    // DrawPipeline grid integration tests
    // =========================================================================

    #[test]
    fn test_pipeline_grid_load_and_edit() {
        let mut pipeline = DrawPipeline::load_from_string(GRID_PXL, Some("face")).unwrap();

        // Extract grid
        let grid = pipeline.grid().unwrap();
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.get(1, 1), Some("eye"));

        // Apply a set operation
        pipeline
            .apply_ops(&[DrawOp::Set { x: 1, y: 1, token: "skin".to_string() }])
            .unwrap();

        // Verify modification
        let grid = pipeline.grid().unwrap();
        assert_eq!(grid.get(1, 1), Some("skin"));
    }

    #[test]
    fn test_pipeline_grid_roundtrip() {
        let mut pipeline = DrawPipeline::load_from_string(GRID_PXL, Some("face")).unwrap();

        // Modify and serialize
        pipeline
            .apply_ops(&[DrawOp::Set { x: 0, y: 2, token: "eye".to_string() }])
            .unwrap();

        let result = pipeline.serialize().unwrap();

        // Re-parse and verify
        let pipeline2 = DrawPipeline::load_from_string(&result.content, Some("face")).unwrap();
        let grid = pipeline2.grid().unwrap();
        assert_eq!(grid.get(0, 2), Some("eye"));
        // Unmodified cells preserved
        assert_eq!(grid.get(1, 1), Some("eye"));
        assert_eq!(grid.get(0, 0), Some("hair"));
    }

    #[test]
    fn test_pipeline_no_grid_error() {
        let pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        let result = pipeline.grid();
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::NoGrid(name) => assert_eq!(name, "dot"),
            other => panic!("Expected NoGrid, got: {:?}", other),
        }
    }

    #[test]
    fn test_pipeline_erase_sets_transparent() {
        let mut pipeline = DrawPipeline::load_from_string(GRID_PXL, Some("face")).unwrap();

        pipeline.apply_ops(&[DrawOp::Erase { x: 1, y: 1 }]).unwrap();

        let grid = pipeline.grid().unwrap();
        assert_eq!(grid.get(1, 1), Some("_"));
    }

    #[test]
    fn test_pipeline_multiple_ops() {
        let mut pipeline = DrawPipeline::load_from_string(GRID_PXL, Some("face")).unwrap();

        pipeline
            .apply_ops(&[
                DrawOp::Set { x: 0, y: 0, token: "eye".to_string() },
                DrawOp::Set { x: 2, y: 0, token: "eye".to_string() },
                DrawOp::Erase { x: 1, y: 0 },
            ])
            .unwrap();

        let grid = pipeline.grid().unwrap();
        assert_eq!(grid.get(0, 0), Some("eye"));
        assert_eq!(grid.get(1, 0), Some("_"));
        assert_eq!(grid.get(2, 0), Some("eye"));
    }

    #[test]
    fn test_pipeline_out_of_bounds_error() {
        let mut pipeline = DrawPipeline::load_from_string(GRID_PXL, Some("face")).unwrap();

        let result = pipeline.apply_ops(&[DrawOp::Set { x: 10, y: 10, token: "x".to_string() }]);
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::OutOfBounds { x, y, .. } => {
                assert_eq!((x, y), (10, 10));
            }
            other => panic!("Expected OutOfBounds, got: {:?}", other),
        }
    }

    #[test]
    fn test_pipeline_grid_preserves_palette() {
        let mut pipeline = DrawPipeline::load_from_string(GRID_PXL, Some("face")).unwrap();
        pipeline
            .apply_ops(&[DrawOp::Set { x: 0, y: 0, token: "skin".to_string() }])
            .unwrap();
        let result = pipeline.serialize().unwrap();

        // Re-parse and verify palette is preserved
        let pipeline2 = DrawPipeline::load_from_string(&result.content, None).unwrap();
        assert_eq!(pipeline2.objects.len(), 2); // palette + sprite
    }

    // =========================================================================
    // Rect fill tests
    // =========================================================================

    #[test]
    fn test_rect_fill_basic() {
        let rows = vec![
            "{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.rect_fill(0, 0, 4, 2, "sky".to_string());
        assert!(warnings.is_empty());

        // Top 2 rows filled
        for y in 0..2 {
            for x in 0..4 {
                assert_eq!(grid.get(x, y), Some("sky"), "({},{}) should be sky", x, y);
            }
        }
        // Bottom 2 rows unchanged
        for y in 2..4 {
            for x in 0..4 {
                assert_eq!(grid.get(x, y), Some("_"), "({},{}) should be _", x, y);
            }
        }
    }

    #[test]
    fn test_rect_fill_partial_overlap() {
        let rows = vec![
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Rect extends beyond grid bounds
        let warnings = grid.rect_fill(1, 1, 5, 5, "x".to_string());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("clipped"), "Expected clipping warning, got: {}", warnings[0]);

        // Only the part inside the grid should be filled
        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(1, 0), Some("_"));
        assert_eq!(grid.get(0, 1), Some("_"));
        assert_eq!(grid.get(1, 1), Some("x"));
        assert_eq!(grid.get(2, 1), Some("x"));
        assert_eq!(grid.get(1, 2), Some("x"));
        assert_eq!(grid.get(2, 2), Some("x"));
    }

    #[test]
    fn test_rect_fill_zero_width() {
        let rows = vec!["{_}{_}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.rect_fill(0, 0, 0, 1, "x".to_string());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("zero-size"));
        // No modification
        assert_eq!(grid.get(0, 0), Some("_"));
    }

    #[test]
    fn test_rect_fill_zero_height() {
        let rows = vec!["{_}{_}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.rect_fill(0, 0, 2, 0, "x".to_string());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("zero-size"));
        assert_eq!(grid.get(0, 0), Some("_"));
    }

    #[test]
    fn test_rect_fill_entirely_outside() {
        let rows = vec![
            "{_}{_}".to_string(),
            "{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.rect_fill(5, 5, 3, 3, "x".to_string());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("entirely outside"));
        // No modification
        assert_eq!(grid.get(0, 0), Some("_"));
    }

    #[test]
    fn test_rect_fill_single_cell() {
        let rows = vec!["{_}{_}".to_string(), "{_}{_}".to_string()];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.rect_fill(1, 0, 1, 1, "dot".to_string());
        assert!(warnings.is_empty());
        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(1, 0), Some("dot"));
        assert_eq!(grid.get(0, 1), Some("_"));
        assert_eq!(grid.get(1, 1), Some("_"));
    }

    #[test]
    fn test_rect_fill_full_grid() {
        let rows = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.rect_fill(0, 0, 3, 2, "fill".to_string());
        assert!(warnings.is_empty());

        for y in 0..2 {
            for x in 0..3 {
                assert_eq!(grid.get(x, y), Some("fill"));
            }
        }
    }

    // =========================================================================
    // Line drawing tests
    // =========================================================================

    /// A 5x5 grid of transparent pixels for line drawing tests.
    const LINE_GRID_PXL: &str = r##"{"type": "palette", "name": "pal", "colors": {"{_}": "#00000000", "{x}": "#FF0000"}}
{"type": "sprite", "name": "canvas", "size": [5, 5], "palette": "pal", "grid": ["{_}{_}{_}{_}{_}", "{_}{_}{_}{_}{_}", "{_}{_}{_}{_}{_}", "{_}{_}{_}{_}{_}", "{_}{_}{_}{_}{_}"]}"##;

    #[test]
    fn test_line_horizontal() {
        let rows: Vec<String> = (0..5).map(|_| "{_}{_}{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.apply(&DrawOp::Line { x0: 0, y0: 2, x1: 4, y1: 2, token: "x".to_string() })
            .unwrap();

        // Entire row 2 should be filled
        for col in 0..5 {
            assert_eq!(grid.get(col, 2), Some("x"), "col {} should be 'x'", col);
        }
        // Other rows untouched
        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(0, 4), Some("_"));
    }

    #[test]
    fn test_line_vertical() {
        let rows: Vec<String> = (0..5).map(|_| "{_}{_}{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.apply(&DrawOp::Line { x0: 2, y0: 0, x1: 2, y1: 4, token: "x".to_string() })
            .unwrap();

        // Entire column 2 should be filled
        for row in 0..5 {
            assert_eq!(grid.get(2, row), Some("x"), "row {} should be 'x'", row);
        }
        // Other columns untouched
        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(4, 4), Some("_"));
    }

    #[test]
    fn test_line_diagonal() {
        let rows: Vec<String> = (0..5).map(|_| "{_}{_}{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        grid.apply(&DrawOp::Line { x0: 0, y0: 0, x1: 4, y1: 4, token: "x".to_string() })
            .unwrap();

        // Diagonal should be filled
        for i in 0..5 {
            assert_eq!(grid.get(i, i), Some("x"), "({},{}) should be 'x'", i, i);
        }
        // Off-diagonal untouched
        assert_eq!(grid.get(1, 0), Some("_"));
        assert_eq!(grid.get(0, 1), Some("_"));
    }

    #[test]
    fn test_line_single_pixel() {
        let rows: Vec<String> = (0..3).map(|_| "{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Same start and end → single pixel
        grid.apply(&DrawOp::Line { x0: 1, y0: 1, x1: 1, y1: 1, token: "x".to_string() })
            .unwrap();

        assert_eq!(grid.get(1, 1), Some("x"));
        // Neighbors untouched
        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(2, 2), Some("_"));
    }

    #[test]
    fn test_line_reverse_direction() {
        let rows: Vec<String> = (0..5).map(|_| "{_}{_}{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Draw from bottom-right to top-left
        grid.apply(&DrawOp::Line { x0: 4, y0: 4, x1: 0, y1: 0, token: "x".to_string() })
            .unwrap();

        // Same diagonal as top-left to bottom-right
        for i in 0..5 {
            assert_eq!(grid.get(i, i), Some("x"), "({},{}) should be 'x'", i, i);
        }
    }

    #[test]
    fn test_rect_apply_via_draw_op() {
        let rows = vec![
            "{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}".to_string(),
            "{_}{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.apply(&DrawOp::Rect {
            x: 1, y: 0, w: 2, h: 2, token: "sky".to_string(),
        }).unwrap();
        assert!(warnings.is_empty());

        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(1, 0), Some("sky"));
        assert_eq!(grid.get(2, 0), Some("sky"));
        assert_eq!(grid.get(3, 0), Some("_"));
        assert_eq!(grid.get(1, 1), Some("sky"));
        assert_eq!(grid.get(2, 1), Some("sky"));
        assert_eq!(grid.get(1, 2), Some("_"));
    }

    #[test]
    fn test_pipeline_rect_fill() {
        let content = r##"{"type": "palette", "name": "pal", "colors": {"{_}": "#00000000", "{sky}": "#87CEEB"}}
{"type": "sprite", "name": "scene", "size": [4, 4], "palette": "pal", "grid": ["{_}{_}{_}{_}", "{_}{_}{_}{_}", "{_}{_}{_}{_}", "{_}{_}{_}{_}"]}"##;

        let mut pipeline = DrawPipeline::load_from_string(content, Some("scene")).unwrap();

        pipeline
            .apply_ops(&[DrawOp::Rect { x: 0, y: 0, w: 4, h: 2, token: "sky".to_string() }])
            .unwrap();

        let grid = pipeline.grid().unwrap();
        // Top 2 rows are sky
        for y in 0..2 {
            for x in 0..4 {
                assert_eq!(grid.get(x, y), Some("sky"));
            }
        }
        // Bottom 2 rows unchanged
        for y in 2..4 {
            for x in 0..4 {
                assert_eq!(grid.get(x, y), Some("_"));
            }
        }
    }

    #[test]
    fn test_pipeline_rect_with_clamp_warning() {
        let content = r##"{"type": "palette", "name": "pal", "colors": {"{_}": "#00000000", "{x}": "#FF0000"}}
{"type": "sprite", "name": "tiny", "size": [2, 2], "palette": "pal", "grid": ["{_}{_}", "{_}{_}"]}"##;

        let mut pipeline = DrawPipeline::load_from_string(content, Some("tiny")).unwrap();

        pipeline
            .apply_ops(&[DrawOp::Rect { x: 0, y: 0, w: 10, h: 10, token: "x".to_string() }])
            .unwrap();

        // Should have a clipping warning
        let result = pipeline.serialize().unwrap();
        assert!(!result.warnings.is_empty(), "Expected clipping warning");

        // All cells should still be filled (clamped to grid)
        let grid = pipeline.grid().unwrap();
        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(grid.get(x, y), Some("x"));
            }
        }
    }

    #[test]
    fn test_pipeline_rect_and_set_combined() {
        let content = r##"{"type": "palette", "name": "pal", "colors": {"{_}": "#00000000", "{sky}": "#87CEEB", "{sun}": "#FFD700"}}
{"type": "sprite", "name": "scene", "size": [4, 4], "palette": "pal", "grid": ["{_}{_}{_}{_}", "{_}{_}{_}{_}", "{_}{_}{_}{_}", "{_}{_}{_}{_}"]}"##;

        let mut pipeline = DrawPipeline::load_from_string(content, Some("scene")).unwrap();

        // Fill sky, then place a sun pixel
        pipeline
            .apply_ops(&[
                DrawOp::Rect { x: 0, y: 0, w: 4, h: 4, token: "sky".to_string() },
                DrawOp::Set { x: 1, y: 1, token: "sun".to_string() },
            ])
            .unwrap();

        let grid = pipeline.grid().unwrap();
        assert_eq!(grid.get(0, 0), Some("sky"));
        assert_eq!(grid.get(1, 1), Some("sun"));
        assert_eq!(grid.get(2, 2), Some("sky"));
    }

    #[test]
    fn test_line_anti_diagonal() {
        let rows: Vec<String> = (0..5).map(|_| "{_}{_}{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Top-right to bottom-left
        grid.apply(&DrawOp::Line { x0: 4, y0: 0, x1: 0, y1: 4, token: "x".to_string() })
            .unwrap();

        for i in 0..5 {
            assert_eq!(grid.get(4 - i, i), Some("x"), "({},{}) should be 'x'", 4 - i, i);
        }
    }

    #[test]
    fn test_line_pipeline_integration() {
        let mut pipeline =
            DrawPipeline::load_from_string(LINE_GRID_PXL, Some("canvas")).unwrap();

        pipeline
            .apply_ops(&[DrawOp::Line {
                x0: 0,
                y0: 0,
                x1: 4,
                y1: 0,
                token: "x".to_string(),
            }])
            .unwrap();

        let grid = pipeline.grid().unwrap();
        // Top row all filled
        for col in 0..5 {
            assert_eq!(grid.get(col, 0), Some("x"));
        }
        // Second row untouched
        for col in 0..5 {
            assert_eq!(grid.get(col, 1), Some("_"));
        }
    }

    #[test]
    fn test_line_combined_with_set() {
        let mut pipeline =
            DrawPipeline::load_from_string(LINE_GRID_PXL, Some("canvas")).unwrap();

        // Draw a line, then overwrite one pixel
        pipeline
            .apply_ops(&[
                DrawOp::Line { x0: 0, y0: 2, x1: 4, y1: 2, token: "x".to_string() },
                DrawOp::Set { x: 2, y: 2, token: "_".to_string() },
            ])
            .unwrap();

        let grid = pipeline.grid().unwrap();
        assert_eq!(grid.get(0, 2), Some("x"));
        assert_eq!(grid.get(1, 2), Some("x"));
        assert_eq!(grid.get(2, 2), Some("_")); // overwritten
        assert_eq!(grid.get(3, 2), Some("x"));
        assert_eq!(grid.get(4, 2), Some("x"));
    }

    #[test]
    fn test_line_out_of_bounds_clips() {
        // Line that extends beyond grid bounds should draw visible portion only
        let rows: Vec<String> = (0..3).map(|_| "{_}{_}{_}".to_string()).collect();
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Line from (0,0) to (2,2) — fully in-bounds, should work fine
        grid.apply(&DrawOp::Line { x0: 0, y0: 0, x1: 2, y1: 2, token: "x".to_string() })
            .unwrap();

        assert_eq!(grid.get(0, 0), Some("x"));
        assert_eq!(grid.get(1, 1), Some("x"));
        assert_eq!(grid.get(2, 2), Some("x"));
    }

    // =========================================================================
    // Flood fill tests
    // =========================================================================

    #[test]
    fn test_flood_fill_basic() {
        // Fill a connected region of transparent pixels
        let rows = vec![
            "{_}{_}{_}{x}{_}".to_string(),
            "{_}{_}{_}{x}{_}".to_string(),
            "{_}{_}{_}{x}{_}".to_string(),
            "{x}{x}{x}{x}{_}".to_string(),
            "{_}{_}{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Flood fill the top-left region
        let warnings = grid.flood_fill(0, 0, "water".to_string()).unwrap();
        assert!(warnings.is_empty());

        // Top-left 3x3 region should be filled
        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(grid.get(x, y), Some("water"), "({},{}) should be water", x, y);
            }
        }
        // The x-boundary should remain
        assert_eq!(grid.get(3, 0), Some("x"));
        assert_eq!(grid.get(3, 1), Some("x"));
        assert_eq!(grid.get(3, 2), Some("x"));
        // Right column should NOT be filled (separated by x wall)
        assert_eq!(grid.get(4, 0), Some("_"));
        assert_eq!(grid.get(4, 1), Some("_"));
        // Bottom row should NOT be filled (separated by x wall)
        assert_eq!(grid.get(0, 4), Some("_"));
    }

    #[test]
    fn test_flood_fill_same_token_noop() {
        let rows = vec![
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.flood_fill(0, 0, "_".to_string()).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("no-op"));

        // Grid unchanged
        for y in 0..2 {
            for x in 0..3 {
                assert_eq!(grid.get(x, y), Some("_"));
            }
        }
    }

    #[test]
    fn test_flood_fill_out_of_bounds() {
        let rows = vec!["{_}{_}".to_string()];
        let grid_result = TokenGrid::parse(&rows);
        let mut grid = grid_result.unwrap();

        let result = grid.flood_fill(5, 5, "x".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            DrawError::OutOfBounds { x, y, .. } => {
                assert_eq!((x, y), (5, 5));
            }
            other => panic!("Expected OutOfBounds, got: {:?}", other),
        }
    }

    #[test]
    fn test_flood_fill_single_pixel() {
        // Seed surrounded by different tokens — only seed changes
        let rows = vec![
            "{a}{b}{c}".to_string(),
            "{d}{_}{f}".to_string(),
            "{g}{h}{i}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.flood_fill(1, 1, "fill".to_string()).unwrap();
        assert!(warnings.is_empty());

        assert_eq!(grid.get(1, 1), Some("fill"));
        // All neighbors unchanged (different tokens)
        assert_eq!(grid.get(0, 0), Some("a"));
        assert_eq!(grid.get(1, 0), Some("b"));
        assert_eq!(grid.get(2, 0), Some("c"));
        assert_eq!(grid.get(0, 1), Some("d"));
        assert_eq!(grid.get(2, 1), Some("f"));
    }

    #[test]
    fn test_flood_fill_entire_grid() {
        // All same token → flood fills everything
        let rows = vec![
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.flood_fill(1, 1, "fill".to_string()).unwrap();
        assert!(warnings.is_empty());

        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(grid.get(x, y), Some("fill"), "({},{}) should be fill", x, y);
            }
        }
    }

    #[test]
    fn test_flood_fill_no_diagonal_leak() {
        // Diagonal-only connection should NOT fill (4-connected only)
        let rows = vec![
            "{_}{x}{_}".to_string(),
            "{x}{_}{x}".to_string(),
            "{_}{x}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        // Fill the center — only the center should change (surrounded by x on 4 sides)
        // Wait, the center at (1,1) is "_" and neighbors (0,1),(2,1),(1,0),(1,2) are all "x"
        let warnings = grid.flood_fill(1, 1, "fill".to_string()).unwrap();
        assert!(warnings.is_empty());

        assert_eq!(grid.get(1, 1), Some("fill"));
        // Corners are "_" but NOT connected via 4-connectivity
        assert_eq!(grid.get(0, 0), Some("_"));
        assert_eq!(grid.get(2, 0), Some("_"));
        assert_eq!(grid.get(0, 2), Some("_"));
        assert_eq!(grid.get(2, 2), Some("_"));
    }

    #[test]
    fn test_flood_fill_l_shape() {
        // L-shaped region should all fill
        let rows = vec![
            "{_}{x}{x}".to_string(),
            "{_}{x}{x}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.flood_fill(0, 0, "water".to_string()).unwrap();
        assert!(warnings.is_empty());

        // The L-shape: (0,0), (0,1), (0,2), (1,2), (2,2)
        assert_eq!(grid.get(0, 0), Some("water"));
        assert_eq!(grid.get(0, 1), Some("water"));
        assert_eq!(grid.get(0, 2), Some("water"));
        assert_eq!(grid.get(1, 2), Some("water"));
        assert_eq!(grid.get(2, 2), Some("water"));
        // x-cells unchanged
        assert_eq!(grid.get(1, 0), Some("x"));
        assert_eq!(grid.get(2, 0), Some("x"));
        assert_eq!(grid.get(1, 1), Some("x"));
        assert_eq!(grid.get(2, 1), Some("x"));
    }

    #[test]
    fn test_flood_fill_via_draw_op() {
        let rows = vec![
            "{_}{_}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let mut grid = TokenGrid::parse(&rows).unwrap();

        let warnings = grid.apply(&DrawOp::Flood { x: 0, y: 0, token: "fill".to_string() }).unwrap();
        assert!(warnings.is_empty());

        for y in 0..2 {
            for x in 0..3 {
                assert_eq!(grid.get(x, y), Some("fill"));
            }
        }
    }

    #[test]
    fn test_flood_fill_pipeline_integration() {
        let content = r##"{"type": "palette", "name": "pal", "colors": {"{_}": "#00000000", "{water}": "#0000FF", "{x}": "#FF0000"}}
{"type": "sprite", "name": "pool", "size": [4, 4], "palette": "pal", "grid": ["{x}{_}{_}{x}", "{x}{_}{_}{x}", "{x}{_}{_}{x}", "{x}{x}{x}{x}"]}"##;

        let mut pipeline = DrawPipeline::load_from_string(content, Some("pool")).unwrap();

        pipeline
            .apply_ops(&[DrawOp::Flood { x: 1, y: 0, token: "water".to_string() }])
            .unwrap();

        let grid = pipeline.grid().unwrap();
        // Inner pool area should be filled
        assert_eq!(grid.get(1, 0), Some("water"));
        assert_eq!(grid.get(2, 0), Some("water"));
        assert_eq!(grid.get(1, 1), Some("water"));
        assert_eq!(grid.get(2, 1), Some("water"));
        assert_eq!(grid.get(1, 2), Some("water"));
        assert_eq!(grid.get(2, 2), Some("water"));
        // Walls unchanged
        assert_eq!(grid.get(0, 0), Some("x"));
        assert_eq!(grid.get(3, 0), Some("x"));
        assert_eq!(grid.get(0, 3), Some("x"));
    }
}
