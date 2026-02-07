//! Core draw pipeline: read-modify-write for .pxl files.
//!
//! Provides the architecture for commands that modify `.pxl` files in-place.
//! Parses the file into `TtpObject`s, locates a target sprite, applies
//! modifications, reformats, and writes back.

use crate::fmt::format_pixelsrc;
use crate::models::TtpObject;
use crate::parser::parse_stream;
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

    /// Apply a draw operation to the grid.
    pub fn apply(&mut self, op: &DrawOp) -> Result<(), DrawError> {
        match op {
            DrawOp::Set { x, y, token } => self.set(*x, *y, token.clone()),
            DrawOp::Erase { x, y } => self.erase(*x, *y),
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
            grid.apply(op)?;
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
}
