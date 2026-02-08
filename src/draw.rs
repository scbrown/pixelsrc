//! Core draw pipeline: read-modify-write for .pxl files.
//!
//! Provides the architecture for commands that modify `.pxl` files in-place.
//! Parses the file into `TtpObject`s, locates a target sprite, applies
//! modifications to its regions, reformats, and writes back.

use crate::fmt::format_pixelsrc;
use crate::models::{RegionDef, TtpObject};
use crate::parser::parse_stream;
use std::collections::HashMap;
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

/// A draw operation to apply to a sprite's regions.
#[derive(Debug, Clone)]
pub enum DrawOp {
    /// Set a single pixel: `--set x,y="{token}"`
    Set { x: u32, y: u32, token: String },
    /// Erase a single pixel (set to transparent): `--erase x,y`
    Erase { x: u32, y: u32 },
    /// Fill a rectangle: `--rect x,y,w,h="{token}"`
    Rect { x: u32, y: u32, w: u32, h: u32, token: String },
    /// Draw a line between two points: `--line x1,y1,x2,y2="{token}"`
    Line { x0: u32, y0: u32, x1: u32, y1: u32, token: String },
    /// Flood fill from a seed point: `--flood x,y="{token}"`
    Flood { x: u32, y: u32, token: String },
}

/// Editor that applies draw operations to a sprite's regions.
///
/// Modifies region definitions in-place. When a token already has a region:
/// - Same shape type (points+points, line+line) → appends to existing array
/// - Different shape type → wraps existing and new in a `union`
/// - New token → creates region directly
pub struct RegionEditor<'a> {
    regions: &'a mut HashMap<String, RegionDef>,
}

impl<'a> RegionEditor<'a> {
    pub fn new(regions: &'a mut HashMap<String, RegionDef>) -> Self {
        RegionEditor { regions }
    }

    /// Apply a single draw operation, returning warnings.
    pub fn apply(&mut self, op: &DrawOp) -> Vec<String> {
        match op {
            DrawOp::Set { x, y, token } => self.merge_point(*x, *y, token),
            DrawOp::Erase { x, y } => {
                // Compute max z before mutating
                let max_z = self.regions.values().filter_map(|r| r.z).max().unwrap_or(0);
                // Erase = add point to "_" (transparent) at high z-order
                let warnings = self.merge_point(*x, *y, &"_".to_string());
                // Ensure the _ region has high z-order so it covers other tokens
                if let Some(region) = self.regions.get_mut("_") {
                    if region.z.unwrap_or(0) <= max_z {
                        region.z = Some(max_z + 100);
                    }
                }
                warnings
            }
            DrawOp::Rect { x, y, w, h, token } => {
                let new_region = RegionDef { rect: Some([*x, *y, *w, *h]), ..Default::default() };
                self.merge_shape(token, new_region)
            }
            DrawOp::Line { x0, y0, x1, y1, token } => {
                let new_region =
                    RegionDef { line: Some(vec![[*x0, *y0], [*x1, *y1]]), ..Default::default() };
                self.merge_shape(token, new_region)
            }
            DrawOp::Flood { x, y, token } => {
                // Region-based flood: set seed point (full BFS requires grid)
                self.merge_point(*x, *y, token)
            }
        }
    }

    /// Merge a single point into the token's region.
    fn merge_point(&mut self, x: u32, y: u32, token: &str) -> Vec<String> {
        let new_point = [x, y];
        match self.regions.get_mut(token) {
            Some(existing) if is_points_only(existing) => {
                // Smart merge: append to existing points array
                existing.points.as_mut().unwrap().push(new_point);
            }
            Some(existing) => {
                // Different shape type: wrap in union
                let old = std::mem::take(existing);
                *existing = RegionDef {
                    union: Some(vec![
                        old,
                        RegionDef { points: Some(vec![new_point]), ..Default::default() },
                    ]),
                    ..Default::default()
                };
            }
            None => {
                self.regions.insert(
                    token.to_string(),
                    RegionDef { points: Some(vec![new_point]), ..Default::default() },
                );
            }
        }
        Vec::new()
    }

    /// Merge a shape into the token's region. Uses smart extend for compatible
    /// shapes, or union-wraps for different shapes.
    fn merge_shape(&mut self, token: &str, new_region: RegionDef) -> Vec<String> {
        match self.regions.get_mut(token) {
            Some(existing) if can_extend(existing, &new_region) => {
                extend_region(existing, &new_region);
            }
            Some(existing) => {
                let old = std::mem::take(existing);
                *existing = RegionDef { union: Some(vec![old, new_region]), ..Default::default() };
            }
            None => {
                self.regions.insert(token.to_string(), new_region);
            }
        }
        Vec::new()
    }
}

/// Check if a region has only `points` set (no other shape primitives or compounds).
fn is_points_only(r: &RegionDef) -> bool {
    r.points.is_some()
        && r.rect.is_none()
        && r.line.is_none()
        && r.stroke.is_none()
        && r.ellipse.is_none()
        && r.circle.is_none()
        && r.polygon.is_none()
        && r.path.is_none()
        && r.fill.is_none()
        && r.union.is_none()
        && r.base.is_none()
        && r.subtract.is_none()
        && r.intersect.is_none()
}

/// Check if a region has only `line` set (no other shape primitives or compounds).
fn is_line_only(r: &RegionDef) -> bool {
    r.line.is_some()
        && r.points.is_none()
        && r.rect.is_none()
        && r.stroke.is_none()
        && r.ellipse.is_none()
        && r.circle.is_none()
        && r.polygon.is_none()
        && r.path.is_none()
        && r.fill.is_none()
        && r.union.is_none()
        && r.base.is_none()
        && r.subtract.is_none()
        && r.intersect.is_none()
}

/// Check if we can smart-extend existing region with same-shape optimization.
fn can_extend(existing: &RegionDef, new: &RegionDef) -> bool {
    // points + points → append
    if is_points_only(existing) && new.points.is_some() {
        return true;
    }
    // line + line → append segments
    if is_line_only(existing) && new.line.is_some() {
        return true;
    }
    false
}

/// Extend an existing region by appending same-shape data.
fn extend_region(existing: &mut RegionDef, new: &RegionDef) {
    if let (Some(pts), Some(new_pts)) = (&mut existing.points, &new.points) {
        for pt in new_pts {
            pts.push(*pt);
        }
    } else if let (Some(segs), Some(new_segs)) = (&mut existing.line, &new.line) {
        for seg in new_segs {
            segs.push(*seg);
        }
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

        let warnings: Vec<String> = parse_result
            .warnings
            .iter()
            .map(|w| format!("line {}: {}", w.line, w.message))
            .collect();

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
        let mut json_lines = Vec::new();
        for obj in &self.objects {
            let json = serde_json::to_string(obj)
                .map_err(|e| DrawError::FormatError(format!("serialization failed: {}", e)))?;
            json_lines.push(json);
        }
        let raw_content = json_lines.join("\n");

        let formatted = format_pixelsrc(&raw_content).map_err(DrawError::FormatError)?;

        Ok(DrawResult { content: formatted, modified: true, warnings: self.warnings.clone() })
    }

    /// Apply a sequence of draw operations to the target sprite's regions.
    pub fn apply_ops(&mut self, ops: &[DrawOp]) -> Result<(), DrawError> {
        let idx = self
            .sprite_index
            .ok_or_else(|| DrawError::ParseError("no sprite selected".to_string()))?;

        let sprite = match &mut self.objects[idx] {
            TtpObject::Sprite(s) => s,
            _ => return Err(DrawError::ParseError("no sprite selected".to_string())),
        };

        // Initialize regions if not present
        if sprite.regions.is_none() {
            sprite.regions = Some(HashMap::new());
        }

        let regions = sprite.regions.as_mut().unwrap();
        let mut editor = RegionEditor::new(regions);
        let mut all_warnings = Vec::new();
        for op in ops {
            let op_warnings = editor.apply(op);
            all_warnings.extend(op_warnings);
        }
        self.warnings.extend(all_warnings);
        Ok(())
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

    // =========================================================================
    // DrawPipeline scaffolding tests (unchanged from original)
    // =========================================================================

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

        let pipeline2 = DrawPipeline::load_from_string(&result.content, None).unwrap();
        assert_eq!(pipeline2.objects.len(), 3);
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
    // RegionEditor tests
    // =========================================================================

    #[test]
    fn test_set_creates_region() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline.apply_ops(&[DrawOp::Set { x: 3, y: 3, token: "mark".to_string() }]).unwrap();

        let sprite = pipeline.sprite().unwrap();
        let regions = sprite.regions.as_ref().unwrap();
        assert!(regions.contains_key("mark"));
        let region = &regions["mark"];
        assert_eq!(region.points, Some(vec![[3, 3]]));
    }

    #[test]
    fn test_set_appends_to_existing_points() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[
                DrawOp::Set { x: 1, y: 1, token: "mark".to_string() },
                DrawOp::Set { x: 2, y: 2, token: "mark".to_string() },
                DrawOp::Set { x: 3, y: 0, token: "mark".to_string() },
            ])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let region = &sprite.regions.as_ref().unwrap()["mark"];
        assert_eq!(region.points, Some(vec![[1, 1], [2, 2], [3, 0]]));
        // No union wrapping — points just appended
        assert!(region.union.is_none());
    }

    #[test]
    fn test_rect_creates_region() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[DrawOp::Rect { x: 0, y: 0, w: 4, h: 2, token: "sky".to_string() }])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let region = &sprite.regions.as_ref().unwrap()["sky"];
        assert_eq!(region.rect, Some([0, 0, 4, 2]));
    }

    #[test]
    fn test_rect_over_existing_points_creates_union() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[
                DrawOp::Set { x: 5, y: 5, token: "mark".to_string() },
                DrawOp::Rect { x: 0, y: 0, w: 3, h: 3, token: "mark".to_string() },
            ])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let region = &sprite.regions.as_ref().unwrap()["mark"];
        assert!(region.union.is_some());
        let parts = region.union.as_ref().unwrap();
        assert_eq!(parts.len(), 2);
        // First part is the original points
        assert!(parts[0].points.is_some());
        // Second part is the rect
        assert_eq!(parts[1].rect, Some([0, 0, 3, 3]));
    }

    #[test]
    fn test_line_creates_region() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[DrawOp::Line { x0: 0, y0: 0, x1: 4, y1: 4, token: "rope".to_string() }])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let region = &sprite.regions.as_ref().unwrap()["rope"];
        assert_eq!(region.line, Some(vec![[0, 0], [4, 4]]));
    }

    #[test]
    fn test_line_extends_existing_line() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[
                DrawOp::Line { x0: 0, y0: 0, x1: 4, y1: 4, token: "rope".to_string() },
                DrawOp::Line { x0: 4, y0: 4, x1: 8, y1: 0, token: "rope".to_string() },
            ])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let region = &sprite.regions.as_ref().unwrap()["rope"];
        // Lines extend — all segments in one array
        assert_eq!(region.line, Some(vec![[0, 0], [4, 4], [4, 4], [8, 0]]));
        assert!(region.union.is_none());
    }

    #[test]
    fn test_erase_creates_transparent_region() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline.apply_ops(&[DrawOp::Erase { x: 2, y: 2 }]).unwrap();

        let sprite = pipeline.sprite().unwrap();
        let regions = sprite.regions.as_ref().unwrap();
        assert!(regions.contains_key("_"));
        let region = &regions["_"];
        assert_eq!(region.points, Some(vec![[2, 2]]));
        // z should be high to cover other regions
        assert!(region.z.unwrap_or(0) > 0);
    }

    #[test]
    fn test_set_preserves_existing_regions() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        // Original sprite has "x" region with rect [1,1,2,2]
        pipeline.apply_ops(&[DrawOp::Set { x: 0, y: 0, token: "mark".to_string() }]).unwrap();

        let sprite = pipeline.sprite().unwrap();
        let regions = sprite.regions.as_ref().unwrap();
        // Original "x" region still present
        assert!(regions.contains_key("x"));
        assert_eq!(regions["x"].rect, Some([1, 1, 2, 2]));
        // New "mark" region added
        assert!(regions.contains_key("mark"));
    }

    #[test]
    fn test_roundtrip_with_draw_ops() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[
                DrawOp::Set { x: 0, y: 0, token: "mark".to_string() },
                DrawOp::Rect { x: 0, y: 2, w: 4, h: 2, token: "floor".to_string() },
            ])
            .unwrap();
        let result = pipeline.serialize().unwrap();

        // Re-parse and verify regions survived
        let pipeline2 = DrawPipeline::load_from_string(&result.content, Some("dot")).unwrap();
        let sprite = pipeline2.sprite().unwrap();
        let regions = sprite.regions.as_ref().unwrap();
        assert!(regions.contains_key("mark"));
        assert!(regions.contains_key("floor"));
        assert_eq!(regions["floor"].rect, Some([0, 2, 4, 2]));
    }

    #[test]
    fn test_multiple_ops_different_tokens() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline
            .apply_ops(&[
                DrawOp::Rect { x: 0, y: 0, w: 4, h: 2, token: "sky".to_string() },
                DrawOp::Rect { x: 0, y: 2, w: 4, h: 2, token: "ground".to_string() },
                DrawOp::Set { x: 2, y: 1, token: "sun".to_string() },
            ])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let regions = sprite.regions.as_ref().unwrap();
        assert_eq!(regions["sky"].rect, Some([0, 0, 4, 2]));
        assert_eq!(regions["ground"].rect, Some([0, 2, 4, 2]));
        assert_eq!(regions["sun"].points, Some(vec![[2, 1]]));
    }

    #[test]
    fn test_apply_ops_initializes_empty_regions() {
        // Sprite with no regions at all
        let content = r##"{"type": "palette", "name": "p", "colors": {"x": "#FF0000"}}
{"type": "sprite", "name": "empty", "size": [4, 4], "palette": "p"}"##;

        let mut pipeline = DrawPipeline::load_from_string(content, Some("empty")).unwrap();
        pipeline.apply_ops(&[DrawOp::Set { x: 0, y: 0, token: "x".to_string() }]).unwrap();

        let sprite = pipeline.sprite().unwrap();
        assert!(sprite.regions.is_some());
        assert!(sprite.regions.as_ref().unwrap().contains_key("x"));
    }

    #[test]
    fn test_rect_over_existing_rect_creates_union() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        // "x" already has rect [1,1,2,2]. Add another rect for "x".
        pipeline
            .apply_ops(&[DrawOp::Rect { x: 0, y: 0, w: 1, h: 1, token: "x".to_string() }])
            .unwrap();

        let sprite = pipeline.sprite().unwrap();
        let region = &sprite.regions.as_ref().unwrap()["x"];
        // Should be wrapped in union (rect + rect = union)
        assert!(region.union.is_some());
        let parts = region.union.as_ref().unwrap();
        assert_eq!(parts.len(), 2);
    }

    // =========================================================================
    // Flood fill (region-based) tests
    // =========================================================================

    #[test]
    fn test_flood_creates_region() {
        let mut pipeline = DrawPipeline::load_from_string(SIMPLE_PXL, Some("dot")).unwrap();
        pipeline.apply_ops(&[DrawOp::Flood { x: 1, y: 1, token: "water".to_string() }]).unwrap();

        let sprite = pipeline.sprite().unwrap();
        let regions = sprite.regions.as_ref().unwrap();
        assert!(regions.contains_key("water"));
        let region = &regions["water"];
        assert_eq!(region.points, Some(vec![[1, 1]]));
    }
}
