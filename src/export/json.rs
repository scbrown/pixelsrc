//! Generic JSON export format.
//!
//! Exports atlas metadata to a standard JSON format that can be used
//! by any game engine or tool that supports JSON parsing.
//!
//! # Output Format
//!
//! The JSON output contains:
//! - `image`: Path to the atlas image file
//! - `size`: Atlas dimensions `[width, height]`
//! - `frames`: Map of sprite names to frame data
//!   - `x`, `y`: Position in atlas
//!   - `w`, `h`: Frame dimensions
//!   - `origin`: Optional pivot point `[x, y]`
//!   - `boxes`: Optional collision/hit boxes
//! - `animations`: Map of animation names to animation data
//!   - `frames`: List of frame names
//!   - `fps`: Frames per second
//!   - `tags`: Optional sub-animation tags
//!
//! # Example Output
//!
//! ```json
//! {
//!   "image": "sprites.png",
//!   "size": [256, 256],
//!   "frames": {
//!     "player_idle": {
//!       "x": 0,
//!       "y": 0,
//!       "w": 32,
//!       "h": 32,
//!       "origin": [16, 32]
//!     }
//!   },
//!   "animations": {
//!     "walk": {
//!       "frames": ["walk_1", "walk_2", "walk_3"],
//!       "fps": 10
//!     }
//!   }
//! }
//! ```

use crate::atlas::AtlasMetadata;
use crate::export::{ExportError, ExportOptions, Exporter, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// JSON format exporter.
///
/// Exports atlas metadata to a generic JSON format suitable for
/// use with any game engine or custom tooling.
#[derive(Debug, Default)]
pub struct JsonExporter;

impl JsonExporter {
    /// Create a new JSON exporter.
    pub fn new() -> Self {
        Self
    }

    /// Export atlas metadata to a JSON string.
    ///
    /// This is useful for testing or when you need the JSON as a string
    /// rather than writing directly to a file.
    pub fn export_to_string(
        &self,
        metadata: &AtlasMetadata,
        options: &ExportOptions,
    ) -> Result<String> {
        let json = if options.pretty {
            serde_json::to_string_pretty(metadata)?
        } else {
            serde_json::to_string(metadata)?
        };
        Ok(json)
    }
}

impl Exporter for JsonExporter {
    fn export(
        &self,
        metadata: &AtlasMetadata,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<()> {
        let json = self.export_to_string(metadata, options)?;

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "json"
    }

    fn extension(&self) -> &'static str {
        "json"
    }
}

/// Export atlas metadata to JSON file.
///
/// Convenience function for simple export use cases.
///
/// # Arguments
///
/// * `metadata` - The atlas metadata to export
/// * `output_path` - Path to write the JSON file
/// * `pretty` - Whether to pretty-print the output
///
/// # Example
///
/// ```ignore
/// use pixelsrc::export::json::export_json;
/// use pixelsrc::atlas::AtlasMetadata;
///
/// let metadata = AtlasMetadata { /* ... */ };
/// export_json(&metadata, "atlas.json", true)?;
/// ```
pub fn export_json(metadata: &AtlasMetadata, output_path: &Path, pretty: bool) -> Result<()> {
    let exporter = JsonExporter::new();
    let options = ExportOptions { pretty, ..Default::default() };
    exporter.export(metadata, output_path, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::{AtlasAnimation, AtlasBox, AtlasFrame};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_metadata() -> AtlasMetadata {
        AtlasMetadata {
            image: "test.png".to_string(),
            size: [64, 64],
            frames: HashMap::from([
                (
                    "sprite1".to_string(),
                    AtlasFrame { x: 0, y: 0, w: 16, h: 16, origin: None, boxes: None },
                ),
                (
                    "sprite2".to_string(),
                    AtlasFrame {
                        x: 16,
                        y: 0,
                        w: 16,
                        h: 16,
                        origin: Some([8, 16]),
                        boxes: Some(HashMap::from([(
                            "hit".to_string(),
                            AtlasBox { x: 2, y: 2, w: 12, h: 12 },
                        )])),
                    },
                ),
            ]),
            animations: HashMap::from([(
                "idle".to_string(),
                AtlasAnimation {
                    frames: vec!["sprite1".to_string(), "sprite2".to_string()],
                    fps: 10,
                    tags: None,
                },
            )]),
        }
    }

    #[test]
    fn test_json_exporter_new() {
        let exporter = JsonExporter::new();
        assert_eq!(exporter.format_name(), "json");
        assert_eq!(exporter.extension(), "json");
    }

    #[test]
    fn test_export_to_string_pretty() {
        let exporter = JsonExporter::new();
        let metadata = create_test_metadata();
        let options = ExportOptions { pretty: true, ..Default::default() };

        let json = exporter.export_to_string(&metadata, &options).unwrap();

        // Pretty printed should have newlines
        assert!(json.contains('\n'));
        assert!(json.contains("  ")); // Indentation
        assert!(json.contains("\"image\""));
        assert!(json.contains("\"test.png\""));
        assert!(json.contains("\"frames\""));
        assert!(json.contains("\"sprite1\""));
        assert!(json.contains("\"sprite2\""));
        assert!(json.contains("\"animations\""));
        assert!(json.contains("\"idle\""));
    }

    #[test]
    fn test_export_to_string_compact() {
        let exporter = JsonExporter::new();
        let metadata = create_test_metadata();
        let options = ExportOptions { pretty: false, ..Default::default() };

        let json = exporter.export_to_string(&metadata, &options).unwrap();

        // Compact should not have indentation newlines (may have \n in strings)
        assert!(!json.contains("  ")); // No indentation
        assert!(json.contains("\"image\":\"test.png\""));
    }

    #[test]
    fn test_export_to_file() {
        let temp = TempDir::new().unwrap();
        let output_path = temp.path().join("atlas.json");

        let exporter = JsonExporter::new();
        let metadata = create_test_metadata();
        let options = ExportOptions::default();

        exporter.export(&metadata, &output_path, &options).unwrap();

        // Verify file exists and contains valid JSON
        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["image"], "test.png");
        assert_eq!(parsed["size"][0], 64);
        assert_eq!(parsed["size"][1], 64);
    }

    #[test]
    fn test_export_creates_directories() {
        let temp = TempDir::new().unwrap();
        let output_path = temp.path().join("nested/dir/atlas.json");

        let exporter = JsonExporter::new();
        let metadata = create_test_metadata();
        let options = ExportOptions::default();

        exporter.export(&metadata, &output_path, &options).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_export_json_convenience_function() {
        let temp = TempDir::new().unwrap();
        let output_path = temp.path().join("atlas.json");

        let metadata = create_test_metadata();
        export_json(&metadata, &output_path, true).unwrap();

        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains('\n')); // Pretty printed
    }

    #[test]
    fn test_export_preserves_optional_fields() {
        let exporter = JsonExporter::new();
        let metadata = create_test_metadata();
        let options = ExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();

        // sprite2 has origin and boxes - should be included
        assert!(json.contains("\"origin\""));
        assert!(json.contains("\"boxes\""));
        assert!(json.contains("\"hit\""));

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let sprite2 = &parsed["frames"]["sprite2"];
        assert_eq!(sprite2["origin"][0], 8);
        assert_eq!(sprite2["origin"][1], 16);
        assert!(sprite2["boxes"]["hit"].is_object());
    }

    #[test]
    fn test_export_skips_none_fields() {
        let exporter = JsonExporter::new();
        let metadata = AtlasMetadata {
            image: "simple.png".to_string(),
            size: [32, 32],
            frames: HashMap::from([(
                "sprite".to_string(),
                AtlasFrame { x: 0, y: 0, w: 32, h: 32, origin: None, boxes: None },
            )]),
            animations: HashMap::new(),
        };

        let options = ExportOptions::default();
        let json = exporter.export_to_string(&metadata, &options).unwrap();

        // Verify that None fields are not included
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let sprite = &parsed["frames"]["sprite"];

        // origin and boxes should not be present when None
        assert!(sprite.get("origin").is_none());
        assert!(sprite.get("boxes").is_none());
        // animations should not be present when empty
        assert!(parsed.get("animations").is_none());
    }

    #[test]
    fn test_export_empty_metadata() {
        let exporter = JsonExporter::new();
        let metadata = AtlasMetadata {
            image: "empty.png".to_string(),
            size: [0, 0],
            frames: HashMap::new(),
            animations: HashMap::new(),
        };

        let options = ExportOptions::default();
        let result = exporter.export_to_string(&metadata, &options);
        assert!(result.is_ok());

        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["image"], "empty.png");
        assert!(parsed["frames"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_roundtrip_json() {
        let exporter = JsonExporter::new();
        let original = create_test_metadata();
        let options = ExportOptions::default();

        let json = exporter.export_to_string(&original, &options).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify structure matches original
        assert_eq!(parsed["image"], original.image);
        assert_eq!(parsed["size"][0], original.size[0]);
        assert_eq!(parsed["size"][1], original.size[1]);
        assert_eq!(parsed["frames"].as_object().unwrap().len(), original.frames.len());
        assert_eq!(parsed["animations"].as_object().unwrap().len(), original.animations.len());
    }
}
