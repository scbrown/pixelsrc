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
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawError::Io(e) => write!(f, "I/O error: {}", e),
            DrawError::SpriteNotFound(name) => write!(f, "sprite '{}' not found", name),
            DrawError::ParseError(msg) => write!(f, "parse error: {}", msg),
            DrawError::FormatError(msg) => write!(f, "format error: {}", msg),
        }
    }
}

impl From<std::io::Error> for DrawError {
    fn from(e: std::io::Error) -> Self {
        DrawError::Io(e)
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
}
