//! Unity engine export format.
//!
//! Exports atlas metadata to Unity-compatible formats for sprite sheet import.
//! Generates JSON metadata that can be used with Unity's sprite import system
//! or custom import scripts.
//!
//! # Output Format
//!
//! The Unity exporter generates a JSON file containing:
//!
//! ```json
//! {
//!   "texture": "atlas.png",
//!   "textureSize": { "w": 256, "h": 256 },
//!   "pixelsPerUnit": 16,
//!   "filterMode": "Point",
//!   "sprites": [
//!     {
//!       "name": "player_idle",
//!       "rect": { "x": 0, "y": 0, "w": 32, "h": 32 },
//!       "pivot": { "x": 0.5, "y": 0.0 },
//!       "border": { "x": 0, "y": 0, "z": 0, "w": 0 }
//!     }
//!   ],
//!   "animations": [
//!     {
//!       "name": "walk",
//!       "frameRate": 10,
//!       "sprites": ["walk_1", "walk_2", "walk_3"]
//!     }
//!   ]
//! }
//! ```
//!
//! This format can be imported using Unity's `TextureImporter` API or
//! custom editor scripts.

use crate::atlas::AtlasMetadata;
use crate::export::{ExportError, ExportOptions, Exporter};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Unity texture filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum UnityFilterMode {
    /// Point (nearest neighbor) filtering - pixel perfect
    #[default]
    Point,
    /// Bilinear filtering - smooth
    Bilinear,
    /// Trilinear filtering - smooth with mipmaps
    Trilinear,
}

impl UnityFilterMode {
    /// Convert from config FilterMode.
    pub fn from_config(mode: &crate::config::FilterMode) -> Self {
        match mode {
            crate::config::FilterMode::Point => Self::Point,
            crate::config::FilterMode::Bilinear => Self::Bilinear,
        }
    }
}

/// Unity export options.
#[derive(Debug, Clone)]
pub struct UnityExportOptions {
    /// Base export options
    pub base: ExportOptions,
    /// Pixels per unit for Unity sprite import
    pub pixels_per_unit: u32,
    /// Texture filter mode
    pub filter_mode: UnityFilterMode,
    /// Generate animation clips
    pub include_animations: bool,
}

impl Default for UnityExportOptions {
    fn default() -> Self {
        Self {
            base: ExportOptions::default(),
            pixels_per_unit: 16,
            filter_mode: UnityFilterMode::Point,
            include_animations: true,
        }
    }
}

/// Unity sprite definition.
#[derive(Debug, Clone, Serialize)]
pub struct UnitySprite {
    /// Sprite name
    pub name: String,
    /// Rectangle in texture (x, y from bottom-left in Unity)
    pub rect: UnityRect,
    /// Pivot point (0-1 normalized)
    pub pivot: UnityVector2,
    /// Border for 9-slice (left, bottom, right, top)
    pub border: UnityVector4,
}

/// Unity rectangle.
#[derive(Debug, Clone, Serialize)]
pub struct UnityRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Unity 2D vector.
#[derive(Debug, Clone, Serialize)]
pub struct UnityVector2 {
    pub x: f32,
    pub y: f32,
}

/// Unity 4D vector (used for borders).
#[derive(Debug, Clone, Serialize)]
pub struct UnityVector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Unity animation clip definition.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityAnimation {
    /// Animation name
    pub name: String,
    /// Frame rate (fps)
    pub frame_rate: u32,
    /// Sprite names in order
    pub sprites: Vec<String>,
    /// Loop animation
    pub loop_animation: bool,
}

/// Complete Unity export data.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityAtlasData {
    /// Texture filename
    pub texture: String,
    /// Texture dimensions
    pub texture_size: UnityVector2,
    /// Pixels per unit setting
    pub pixels_per_unit: u32,
    /// Filter mode
    pub filter_mode: UnityFilterMode,
    /// Sprite definitions
    pub sprites: Vec<UnitySprite>,
    /// Animation definitions
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<UnityAnimation>,
}

/// Unity format exporter.
#[derive(Debug, Default)]
pub struct UnityExporter {
    /// Pixels per unit
    pixels_per_unit: u32,
    /// Filter mode
    filter_mode: UnityFilterMode,
    /// Include animations
    include_animations: bool,
}

impl UnityExporter {
    /// Create a new Unity exporter with default settings.
    pub fn new() -> Self {
        Self { pixels_per_unit: 16, filter_mode: UnityFilterMode::Point, include_animations: true }
    }

    /// Set pixels per unit.
    pub fn with_pixels_per_unit(mut self, ppu: u32) -> Self {
        self.pixels_per_unit = ppu;
        self
    }

    /// Set filter mode.
    pub fn with_filter_mode(mut self, mode: UnityFilterMode) -> Self {
        self.filter_mode = mode;
        self
    }

    /// Enable or disable animation export.
    pub fn with_animations(mut self, enabled: bool) -> Self {
        self.include_animations = enabled;
        self
    }

    /// Export atlas metadata to Unity format.
    pub fn export_unity(
        &self,
        metadata: &AtlasMetadata,
        output_path: &Path,
        options: &UnityExportOptions,
    ) -> Result<(), ExportError> {
        let data = self.build_atlas_data(metadata, options);
        let json = if options.base.pretty {
            serde_json::to_string_pretty(&data)?
        } else {
            serde_json::to_string(&data)?
        };

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Build Unity atlas data from metadata.
    fn build_atlas_data(
        &self,
        metadata: &AtlasMetadata,
        options: &UnityExportOptions,
    ) -> UnityAtlasData {
        let texture_height = metadata.size[1] as f32;

        // Convert frames to Unity sprites
        // Unity uses bottom-left origin, so we need to flip Y
        let mut sprites: Vec<UnitySprite> = metadata
            .frames
            .iter()
            .map(|(name, frame)| {
                // Calculate pivot from origin if available
                let pivot = if let Some(origin) = frame.origin {
                    UnityVector2 {
                        x: origin[0] as f32 / frame.w as f32,
                        // Flip Y for Unity coordinate system
                        y: 1.0 - (origin[1] as f32 / frame.h as f32),
                    }
                } else {
                    // Default pivot at bottom-center (common for sprites)
                    UnityVector2 { x: 0.5, y: 0.0 }
                };

                UnitySprite {
                    name: name.clone(),
                    rect: UnityRect {
                        x: frame.x as f32,
                        // Flip Y: Unity Y starts from bottom
                        y: texture_height - frame.y as f32 - frame.h as f32,
                        w: frame.w as f32,
                        h: frame.h as f32,
                    },
                    pivot,
                    border: UnityVector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                }
            })
            .collect();

        // Sort sprites by name for consistent output
        sprites.sort_by(|a, b| a.name.cmp(&b.name));

        // Convert animations
        let animations: Vec<UnityAnimation> = if options.include_animations {
            metadata
                .animations
                .iter()
                .map(|(name, anim)| UnityAnimation {
                    name: name.clone(),
                    frame_rate: anim.fps,
                    sprites: anim.frames.clone(),
                    loop_animation: true,
                })
                .collect()
        } else {
            vec![]
        };

        UnityAtlasData {
            texture: metadata.image.clone(),
            texture_size: UnityVector2 { x: metadata.size[0] as f32, y: metadata.size[1] as f32 },
            pixels_per_unit: options.pixels_per_unit,
            filter_mode: options.filter_mode,
            sprites,
            animations,
        }
    }

    /// Export to string (for testing).
    pub fn export_to_string(
        &self,
        metadata: &AtlasMetadata,
        options: &UnityExportOptions,
    ) -> Result<String, ExportError> {
        let data = self.build_atlas_data(metadata, options);
        let json = if options.base.pretty {
            serde_json::to_string_pretty(&data)?
        } else {
            serde_json::to_string(&data)?
        };
        Ok(json)
    }
}

impl Exporter for UnityExporter {
    fn export(
        &self,
        metadata: &AtlasMetadata,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<(), ExportError> {
        let unity_options = UnityExportOptions {
            base: options.clone(),
            pixels_per_unit: self.pixels_per_unit,
            filter_mode: self.filter_mode,
            include_animations: self.include_animations,
        };
        self.export_unity(metadata, output_path, &unity_options)
    }

    fn format_name(&self) -> &'static str {
        "unity"
    }

    fn extension(&self) -> &'static str {
        "json"
    }
}

/// Export atlas metadata to Unity format.
///
/// Convenience function for simple export use cases.
pub fn export_unity(
    metadata: &AtlasMetadata,
    output_path: &Path,
    pixels_per_unit: u32,
) -> Result<(), ExportError> {
    let exporter = UnityExporter::new().with_pixels_per_unit(pixels_per_unit);
    let options = UnityExportOptions { pixels_per_unit, ..Default::default() };
    exporter.export_unity(metadata, output_path, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::{AtlasAnimation, AtlasFrame};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_metadata() -> AtlasMetadata {
        AtlasMetadata {
            image: "sprites.png".to_string(),
            size: [128, 128],
            frames: HashMap::from([
                (
                    "player_idle".to_string(),
                    AtlasFrame {
                        x: 0,
                        y: 0,
                        w: 32,
                        h: 32,
                        origin: Some([16, 32]), // Bottom center
                        boxes: None,
                    },
                ),
                (
                    "player_walk_1".to_string(),
                    AtlasFrame { x: 32, y: 0, w: 32, h: 32, origin: None, boxes: None },
                ),
                (
                    "player_walk_2".to_string(),
                    AtlasFrame { x: 64, y: 0, w: 32, h: 32, origin: None, boxes: None },
                ),
            ]),
            animations: HashMap::from([(
                "walk".to_string(),
                AtlasAnimation {
                    frames: vec!["player_walk_1".to_string(), "player_walk_2".to_string()],
                    fps: 10,
                    tags: None,
                },
            )]),
        }
    }

    #[test]
    fn test_unity_exporter_new() {
        let exporter = UnityExporter::new();
        assert_eq!(exporter.format_name(), "unity");
        assert_eq!(exporter.extension(), "json");
        assert_eq!(exporter.pixels_per_unit, 16);
    }

    #[test]
    fn test_unity_exporter_with_options() {
        let exporter = UnityExporter::new()
            .with_pixels_per_unit(32)
            .with_filter_mode(UnityFilterMode::Bilinear)
            .with_animations(false);

        assert_eq!(exporter.pixels_per_unit, 32);
        assert_eq!(exporter.filter_mode, UnityFilterMode::Bilinear);
        assert!(!exporter.include_animations);
    }

    #[test]
    fn test_unity_filter_mode_default() {
        let mode = UnityFilterMode::default();
        assert_eq!(mode, UnityFilterMode::Point);
    }

    #[test]
    fn test_unity_filter_mode_from_config() {
        assert_eq!(
            UnityFilterMode::from_config(&crate::config::FilterMode::Point),
            UnityFilterMode::Point
        );
        assert_eq!(
            UnityFilterMode::from_config(&crate::config::FilterMode::Bilinear),
            UnityFilterMode::Bilinear
        );
    }

    #[test]
    fn test_export_to_string() {
        let exporter = UnityExporter::new();
        let metadata = create_test_metadata();
        let options = UnityExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();

        assert!(json.contains("\"texture\": \"sprites.png\""));
        assert!(json.contains("\"pixelsPerUnit\": 16"));
        assert!(json.contains("\"filterMode\": \"Point\""));
        assert!(json.contains("\"sprites\""));
        assert!(json.contains("\"animations\""));
    }

    #[test]
    fn test_export_unity_creates_file() {
        let temp = TempDir::new().unwrap();
        let output_path = temp.path().join("atlas.json");
        let metadata = create_test_metadata();

        export_unity(&metadata, &output_path, 16).unwrap();

        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("sprites.png"));
    }

    #[test]
    fn test_export_sprite_rect() {
        let exporter = UnityExporter::new();
        let metadata = create_test_metadata();
        let options = UnityExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Find player_idle sprite
        let sprites = data["sprites"].as_array().unwrap();
        let player_idle = sprites.iter().find(|s| s["name"] == "player_idle").unwrap();

        // Check rect - Y should be flipped (128 - 0 - 32 = 96)
        assert_eq!(player_idle["rect"]["x"], 0.0);
        assert_eq!(player_idle["rect"]["y"], 96.0); // Flipped Y
        assert_eq!(player_idle["rect"]["w"], 32.0);
        assert_eq!(player_idle["rect"]["h"], 32.0);
    }

    #[test]
    fn test_export_sprite_pivot_from_origin() {
        let exporter = UnityExporter::new();
        let metadata = create_test_metadata();
        let options = UnityExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let sprites = data["sprites"].as_array().unwrap();
        let player_idle = sprites.iter().find(|s| s["name"] == "player_idle").unwrap();

        // Origin [16, 32] on 32x32 sprite = pivot (0.5, 0.0) after Y flip
        assert_eq!(player_idle["pivot"]["x"], 0.5);
        assert_eq!(player_idle["pivot"]["y"], 0.0);
    }

    #[test]
    fn test_export_sprite_default_pivot() {
        let exporter = UnityExporter::new();
        let metadata = create_test_metadata();
        let options = UnityExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let sprites = data["sprites"].as_array().unwrap();
        let walk_1 = sprites.iter().find(|s| s["name"] == "player_walk_1").unwrap();

        // No origin = default pivot at bottom center (0.5, 0.0)
        assert_eq!(walk_1["pivot"]["x"], 0.5);
        assert_eq!(walk_1["pivot"]["y"], 0.0);
    }

    #[test]
    fn test_export_animations() {
        let exporter = UnityExporter::new();
        let metadata = create_test_metadata();
        let options = UnityExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let animations = data["animations"].as_array().unwrap();
        assert_eq!(animations.len(), 1);

        let walk = &animations[0];
        assert_eq!(walk["name"], "walk");
        assert_eq!(walk["frameRate"], 10);
        assert_eq!(walk["loopAnimation"], true);

        let sprites = walk["sprites"].as_array().unwrap();
        assert_eq!(sprites.len(), 2);
    }

    #[test]
    fn test_export_without_animations() {
        let exporter = UnityExporter::new().with_animations(false);
        let metadata = create_test_metadata();
        let options = UnityExportOptions { include_animations: false, ..Default::default() };

        let json = exporter.export_to_string(&metadata, &options).unwrap();

        // animations should not be in output when empty
        assert!(!json.contains("\"animations\""));
    }

    #[test]
    fn test_export_custom_pixels_per_unit() {
        let exporter = UnityExporter::new().with_pixels_per_unit(100);
        let metadata = create_test_metadata();
        let options = UnityExportOptions { pixels_per_unit: 100, ..Default::default() };

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        assert!(json.contains("\"pixelsPerUnit\": 100"));
    }

    #[test]
    fn test_export_bilinear_filter() {
        let exporter = UnityExporter::new().with_filter_mode(UnityFilterMode::Bilinear);
        let metadata = create_test_metadata();
        let options =
            UnityExportOptions { filter_mode: UnityFilterMode::Bilinear, ..Default::default() };

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        assert!(json.contains("\"filterMode\": \"Bilinear\""));
    }

    #[test]
    fn test_export_via_trait() {
        let temp = TempDir::new().unwrap();
        let output_path = temp.path().join("atlas.json");
        let metadata = create_test_metadata();
        let exporter = UnityExporter::new();
        let options = ExportOptions::default();

        exporter.export(&metadata, &output_path, &options).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_sprites_sorted_by_name() {
        let exporter = UnityExporter::new();
        let metadata = create_test_metadata();
        let options = UnityExportOptions::default();

        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let sprites = data["sprites"].as_array().unwrap();
        let names: Vec<&str> = sprites.iter().map(|s| s["name"].as_str().unwrap()).collect();

        // Should be sorted alphabetically
        assert_eq!(names, vec!["player_idle", "player_walk_1", "player_walk_2"]);
    }

    #[test]
    fn test_unity_export_options_default() {
        let options = UnityExportOptions::default();
        assert_eq!(options.pixels_per_unit, 16);
        assert_eq!(options.filter_mode, UnityFilterMode::Point);
        assert!(options.include_animations);
    }
}
