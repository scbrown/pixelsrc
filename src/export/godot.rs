//! Godot engine export format.
//!
//! Exports atlas metadata to Godot-compatible `.tres` resource files.
//! Supports AtlasTexture for individual frames, SpriteFrames for AnimatedSprite2D,
//! and AnimationLibrary for AnimationPlayer.
//!
//! # Output Formats
//!
//! ## AtlasTexture (.tres)
//!
//! For each frame in the atlas, generates a resource like:
//!
//! ```text
//! [gd_resource type="AtlasTexture" load_steps=2 format=3]
//!
//! [ext_resource type="Texture2D" path="res://assets/atlas.png" id="1"]
//!
//! [resource]
//! atlas = ExtResource("1")
//! region = Rect2(0, 0, 32, 32)
//! ```
//!
//! ## SpriteFrames (.tres)
//!
//! For animations with AnimatedSprite2D, generates SpriteFrames resources:
//!
//! ```text
//! [gd_resource type="SpriteFrames" load_steps=4 format=3]
//!
//! [ext_resource type="AtlasTexture" path="res://assets/frame_1.tres" id="1"]
//! [ext_resource type="AtlasTexture" path="res://assets/frame_2.tres" id="2"]
//!
//! [resource]
//! animations = [{
//!   "frames": [{...}, {...}],
//!   "loop": true,
//!   "name": &"walk",
//!   "speed": 10.0
//! }]
//! ```
//!
//! ## AnimationLibrary (.tres)
//!
//! For animations with AnimationPlayer, generates AnimationLibrary resources:
//!
//! ```text
//! [gd_resource type="AnimationLibrary" load_steps=3 format=3]
//!
//! [ext_resource type="AtlasTexture" path="res://assets/frame_1.tres" id="1"]
//! [ext_resource type="AtlasTexture" path="res://assets/frame_2.tres" id="2"]
//!
//! [sub_resource type="Animation" id="Animation_walk"]
//! resource_name = "walk"
//! length = 0.2
//! loop_mode = 1
//! tracks/0/type = "value"
//! tracks/0/path = NodePath(".:texture")
//! tracks/0/keys = { "times": ..., "values": [...] }
//!
//! [resource]
//! _data = { "walk": SubResource("Animation_walk") }
//! ```

use crate::atlas::AtlasMetadata;
use crate::export::{ExportError, ExportOptions, Exporter};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Godot export configuration options.
#[derive(Debug, Clone)]
pub struct GodotExportOptions {
    /// Base export options
    pub base: ExportOptions,
    /// Godot resource path prefix (e.g., "res://assets/sprites")
    pub resource_path: String,
    /// Generate SpriteFrames for animations (used with AnimatedSprite2D)
    pub sprite_frames: bool,
    /// Generate AnimationLibrary for animations (used with AnimationPlayer)
    pub animation_player: bool,
    /// Generate individual AtlasTexture files
    pub atlas_textures: bool,
}

impl Default for GodotExportOptions {
    fn default() -> Self {
        Self {
            base: ExportOptions::default(),
            resource_path: "res://assets/sprites".to_string(),
            sprite_frames: true,
            animation_player: true,
            atlas_textures: true,
        }
    }
}

/// Godot format exporter.
///
/// Exports atlas metadata to Godot `.tres` resource files.
#[derive(Debug, Default)]
pub struct GodotExporter {
    /// Resource path prefix
    resource_path: String,
    /// Generate SpriteFrames (for AnimatedSprite2D)
    sprite_frames: bool,
    /// Generate AnimationLibrary (for AnimationPlayer)
    animation_player: bool,
}

impl GodotExporter {
    /// Create a new Godot exporter with default settings.
    pub fn new() -> Self {
        Self {
            resource_path: "res://assets/sprites".to_string(),
            sprite_frames: true,
            animation_player: true,
        }
    }

    /// Create a Godot exporter with custom resource path.
    pub fn with_resource_path(mut self, path: &str) -> Self {
        self.resource_path = path.to_string();
        self
    }

    /// Enable or disable SpriteFrames generation (for AnimatedSprite2D).
    pub fn with_sprite_frames(mut self, enabled: bool) -> Self {
        self.sprite_frames = enabled;
        self
    }

    /// Enable or disable AnimationLibrary generation (for AnimationPlayer).
    pub fn with_animation_player(mut self, enabled: bool) -> Self {
        self.animation_player = enabled;
        self
    }

    /// Export atlas metadata to Godot format.
    ///
    /// Creates:
    /// - One `.tres` file with AtlasTexture for each frame
    /// - One `_frames.tres` file with SpriteFrames if animations exist (for AnimatedSprite2D)
    /// - One `_anims.tres` file with AnimationLibrary if animations exist (for AnimationPlayer)
    pub fn export_godot(
        &self,
        metadata: &AtlasMetadata,
        output_dir: &Path,
        options: &GodotExportOptions,
    ) -> Result<Vec<std::path::PathBuf>, ExportError> {
        let mut outputs = Vec::new();

        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        // Get atlas base name (without extension)
        let atlas_name = metadata.image.trim_end_matches(".png").trim_end_matches(".PNG");

        // Generate AtlasTexture for each frame
        if options.atlas_textures {
            for (frame_name, frame) in &metadata.frames {
                let content = self.generate_atlas_texture(
                    &metadata.image,
                    frame.x,
                    frame.y,
                    frame.w,
                    frame.h,
                    &options.resource_path,
                );

                let output_path = output_dir.join(format!("{}.tres", frame_name));
                let mut file = File::create(&output_path)?;
                file.write_all(content.as_bytes())?;
                outputs.push(output_path);
            }
        }

        // Generate SpriteFrames for animations (used with AnimatedSprite2D)
        if options.sprite_frames && !metadata.animations.is_empty() {
            let content = self.generate_sprite_frames(metadata, &options.resource_path);
            let output_path = output_dir.join(format!("{}_frames.tres", atlas_name));
            let mut file = File::create(&output_path)?;
            file.write_all(content.as_bytes())?;
            outputs.push(output_path);
        }

        // Generate AnimationLibrary for animations (used with AnimationPlayer)
        if options.animation_player && !metadata.animations.is_empty() {
            let content = self.generate_animation_library(metadata, &options.resource_path);
            let output_path = output_dir.join(format!("{}_anims.tres", atlas_name));
            let mut file = File::create(&output_path)?;
            file.write_all(content.as_bytes())?;
            outputs.push(output_path);
        }

        Ok(outputs)
    }

    /// Generate AtlasTexture resource content.
    fn generate_atlas_texture(
        &self,
        image_path: &str,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        resource_path: &str,
    ) -> String {
        let texture_path = format!("{}/{}", resource_path, image_path);

        format!(
            r#"[gd_resource type="AtlasTexture" load_steps=2 format=3]

[ext_resource type="Texture2D" path="{}" id="1"]

[resource]
atlas = ExtResource("1")
region = Rect2({}, {}, {}, {})
"#,
            texture_path, x, y, w, h
        )
    }

    /// Generate SpriteFrames resource content.
    fn generate_sprite_frames(&self, metadata: &AtlasMetadata, resource_path: &str) -> String {
        let mut lines = Vec::new();

        // Count resources needed
        let frame_count: usize = metadata.animations.values().map(|a| a.frames.len()).sum();
        let load_steps = frame_count + 1;

        // Header
        lines.push(format!(
            "[gd_resource type=\"SpriteFrames\" load_steps={} format=3]",
            load_steps
        ));
        lines.push(String::new());

        // External resources (AtlasTextures for each frame)
        let mut ext_id = 1;
        let mut frame_to_id: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        for anim in metadata.animations.values() {
            for frame_name in &anim.frames {
                if !frame_to_id.contains_key(frame_name) {
                    let texture_path = format!("{}/{}.tres", resource_path, frame_name);
                    lines.push(format!(
                        "[ext_resource type=\"AtlasTexture\" path=\"{}\" id=\"{}\"]",
                        texture_path, ext_id
                    ));
                    frame_to_id.insert(frame_name.clone(), ext_id);
                    ext_id += 1;
                }
            }
        }

        lines.push(String::new());
        lines.push("[resource]".to_string());

        // Build animations array
        let mut anim_entries = Vec::new();
        for (anim_name, anim) in &metadata.animations {
            let fps = anim.fps as f64;

            // Build frames array
            let mut frame_entries = Vec::new();
            for frame_name in &anim.frames {
                if let Some(&id) = frame_to_id.get(frame_name) {
                    frame_entries.push(format!(
                        "{{\"duration\": 1.0, \"texture\": ExtResource(\"{}\")}}",
                        id
                    ));
                }
            }

            let frames_str = frame_entries.join(", ");
            anim_entries.push(format!(
                "{{\"frames\": [{}], \"loop\": true, \"name\": &\"{}\", \"speed\": {:.1}}}",
                frames_str, anim_name, fps
            ));
        }

        lines.push(format!("animations = [{}]", anim_entries.join(", ")));

        lines.join("\n")
    }

    /// Generate AnimationLibrary resource content for AnimationPlayer.
    ///
    /// Creates an AnimationLibrary with Animation sub-resources that animate
    /// the `texture` property of a Sprite2D node.
    fn generate_animation_library(&self, metadata: &AtlasMetadata, resource_path: &str) -> String {
        let mut lines = Vec::new();

        // Collect all unique frames across all animations
        let mut all_frames: std::collections::HashSet<String> = std::collections::HashSet::new();
        for anim in metadata.animations.values() {
            for frame_name in &anim.frames {
                all_frames.insert(frame_name.clone());
            }
        }

        // Sort animation names for deterministic output
        let mut anim_names: Vec<&String> = metadata.animations.keys().collect();
        anim_names.sort();

        // Calculate load_steps: 1 (base) + ext_resources + sub_resources
        let ext_count = all_frames.len();
        let sub_count = metadata.animations.len();
        let load_steps = 1 + ext_count + sub_count;

        // Header
        lines.push(format!(
            "[gd_resource type=\"AnimationLibrary\" load_steps={} format=3]",
            load_steps
        ));
        lines.push(String::new());

        // External resources (AtlasTextures for each frame)
        let mut sorted_frames: Vec<&String> = all_frames.iter().collect();
        sorted_frames.sort();

        let mut frame_to_id: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        for (idx, frame_name) in sorted_frames.iter().enumerate() {
            let ext_id = idx as u32 + 1;
            let texture_path = format!("{}/{}.tres", resource_path, frame_name);
            lines.push(format!(
                "[ext_resource type=\"AtlasTexture\" path=\"{}\" id=\"{}\"]",
                texture_path, ext_id
            ));
            frame_to_id.insert((*frame_name).clone(), ext_id);
        }

        lines.push(String::new());

        // Sub-resources (Animation for each animation)
        for anim_name in &anim_names {
            let anim = &metadata.animations[*anim_name];
            let fps = anim.fps as f64;
            let frame_count = anim.frames.len();

            // Calculate animation length based on fps
            let length = if frame_count > 0 && fps > 0.0 {
                frame_count as f64 / fps
            } else {
                1.0
            };

            // Build times array (evenly spaced)
            let times: Vec<String> = (0..frame_count)
                .map(|i| {
                    let t = if frame_count > 1 {
                        (i as f64) * length / (frame_count as f64)
                    } else {
                        0.0
                    };
                    format!("{:.6}", t)
                })
                .collect();
            let times_str = times.join(", ");

            // Build transitions array (all 1.0 for instant transitions)
            let transitions_str = vec!["1".to_string(); frame_count].join(", ");

            // Build values array (ExtResource references)
            let values: Vec<String> = anim
                .frames
                .iter()
                .filter_map(|frame_name| {
                    frame_to_id
                        .get(frame_name)
                        .map(|id| format!("ExtResource(\"{}\")", id))
                })
                .collect();
            let values_str = values.join(", ");

            lines.push(format!(
                "[sub_resource type=\"Animation\" id=\"Animation_{}\"]",
                anim_name
            ));
            lines.push(format!("resource_name = \"{}\"", anim_name));
            lines.push(format!("length = {:.6}", length));
            lines.push("loop_mode = 1".to_string()); // 1 = loop
            lines.push("tracks/0/type = \"value\"".to_string());
            lines.push("tracks/0/imported = false".to_string());
            lines.push("tracks/0/enabled = true".to_string());
            lines.push("tracks/0/path = NodePath(\".:texture\")".to_string());
            lines.push("tracks/0/interp = 1".to_string()); // 1 = discrete
            lines.push("tracks/0/loop_wrap = true".to_string());
            lines.push(format!(
                "tracks/0/keys = {{\"times\": PackedFloat32Array({}), \"transitions\": PackedFloat32Array({}), \"update\": 1, \"values\": [{}]}}",
                times_str, transitions_str, values_str
            ));
            lines.push(String::new());
        }

        // Main resource section with animation data
        lines.push("[resource]".to_string());
        let anim_data: Vec<String> = anim_names
            .iter()
            .map(|name| format!("\"{}\" = SubResource(\"Animation_{}\")", name, name))
            .collect();
        lines.push(format!("_data = {{{}}}", anim_data.join(", ")));

        lines.join("\n")
    }

    /// Export a single AtlasTexture to a string.
    pub fn export_atlas_texture_to_string(
        &self,
        image_path: &str,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> String {
        self.generate_atlas_texture(image_path, x, y, w, h, &self.resource_path)
    }
}

impl Exporter for GodotExporter {
    fn export(
        &self,
        metadata: &AtlasMetadata,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<(), ExportError> {
        let godot_options = GodotExportOptions {
            base: options.clone(),
            resource_path: self.resource_path.clone(),
            sprite_frames: self.sprite_frames,
            animation_player: self.animation_player,
            atlas_textures: true,
        };

        // If output_path is a file, use its parent as directory
        let output_dir = if output_path.extension().is_some() {
            output_path.parent().unwrap_or(output_path)
        } else {
            output_path
        };

        self.export_godot(metadata, output_dir, &godot_options)?;
        Ok(())
    }

    fn format_name(&self) -> &'static str {
        "godot"
    }

    fn extension(&self) -> &'static str {
        "tres"
    }
}

/// Export atlas metadata to Godot format.
///
/// Convenience function for simple export use cases.
pub fn export_godot(
    metadata: &AtlasMetadata,
    output_dir: &Path,
    resource_path: &str,
) -> Result<Vec<std::path::PathBuf>, ExportError> {
    let exporter = GodotExporter::new().with_resource_path(resource_path);
    let options =
        GodotExportOptions { resource_path: resource_path.to_string(), ..Default::default() };
    exporter.export_godot(metadata, output_dir, &options)
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
                    AtlasFrame { x: 0, y: 0, w: 32, h: 32, origin: None, boxes: None },
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
    fn test_godot_exporter_new() {
        let exporter = GodotExporter::new();
        assert_eq!(exporter.format_name(), "godot");
        assert_eq!(exporter.extension(), "tres");
    }

    #[test]
    fn test_godot_exporter_with_options() {
        let exporter = GodotExporter::new()
            .with_resource_path("res://game/assets")
            .with_sprite_frames(false)
            .with_animation_player(false);

        assert_eq!(exporter.resource_path, "res://game/assets");
        assert!(!exporter.sprite_frames);
        assert!(!exporter.animation_player);
    }

    #[test]
    fn test_generate_atlas_texture() {
        let exporter = GodotExporter::new();
        let content = exporter.generate_atlas_texture("test.png", 10, 20, 32, 48, "res://sprites");

        assert!(content.contains("[gd_resource type=\"AtlasTexture\""));
        assert!(content.contains("load_steps=2"));
        assert!(content.contains("format=3"));
        assert!(content.contains("res://sprites/test.png"));
        assert!(content.contains("Rect2(10, 20, 32, 48)"));
    }

    #[test]
    fn test_generate_sprite_frames() {
        let exporter = GodotExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.generate_sprite_frames(&metadata, "res://sprites");

        assert!(content.contains("[gd_resource type=\"SpriteFrames\""));
        assert!(content.contains("animations = ["));
        assert!(content.contains("\"name\": &\"walk\""));
        assert!(content.contains("\"speed\": 10.0"));
        assert!(content.contains("\"loop\": true"));
    }

    #[test]
    fn test_export_godot_creates_files() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();

        let outputs = export_godot(&metadata, temp.path(), "res://assets").unwrap();

        // Should create AtlasTexture for each frame + SpriteFrames + AnimationLibrary
        assert!(outputs.len() >= 5); // 3 frames + 1 sprite_frames + 1 anim_library

        // Check AtlasTexture files exist
        assert!(temp.path().join("player_idle.tres").exists());
        assert!(temp.path().join("player_walk_1.tres").exists());
        assert!(temp.path().join("player_walk_2.tres").exists());

        // Check SpriteFrames file exists
        assert!(temp.path().join("sprites_frames.tres").exists());

        // Check AnimationLibrary file exists
        assert!(temp.path().join("sprites_anims.tres").exists());
    }

    #[test]
    fn test_export_godot_atlas_texture_content() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();

        export_godot(&metadata, temp.path(), "res://game").unwrap();

        let content = fs::read_to_string(temp.path().join("player_idle.tres")).unwrap();
        assert!(content.contains("AtlasTexture"));
        assert!(content.contains("res://game/sprites.png"));
        assert!(content.contains("Rect2(0, 0, 32, 32)"));
    }

    #[test]
    fn test_export_godot_sprite_frames_content() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();

        export_godot(&metadata, temp.path(), "res://game").unwrap();

        let content = fs::read_to_string(temp.path().join("sprites_frames.tres")).unwrap();
        assert!(content.contains("SpriteFrames"));
        assert!(content.contains("walk"));
        assert!(content.contains("player_walk_1.tres"));
        assert!(content.contains("player_walk_2.tres"));
    }

    #[test]
    fn test_export_without_animations() {
        let temp = TempDir::new().unwrap();
        let metadata = AtlasMetadata {
            image: "static.png".to_string(),
            size: [64, 64],
            frames: HashMap::from([(
                "icon".to_string(),
                AtlasFrame { x: 0, y: 0, w: 64, h: 64, origin: None, boxes: None },
            )]),
            animations: HashMap::new(),
        };

        let outputs = export_godot(&metadata, temp.path(), "res://ui").unwrap();

        // Should only create AtlasTexture, no SpriteFrames
        assert_eq!(outputs.len(), 1);
        assert!(temp.path().join("icon.tres").exists());
        assert!(!temp.path().join("static_frames.tres").exists());
    }

    #[test]
    fn test_export_godot_options_no_sprite_frames() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();

        let exporter = GodotExporter::new()
            .with_resource_path("res://test")
            .with_sprite_frames(false)
            .with_animation_player(false);

        let options = GodotExportOptions {
            resource_path: "res://test".to_string(),
            sprite_frames: false,
            animation_player: false,
            atlas_textures: true,
            base: ExportOptions::default(),
        };

        let outputs = exporter.export_godot(&metadata, temp.path(), &options).unwrap();

        // Should only create AtlasTextures, no SpriteFrames or AnimationLibrary
        assert_eq!(outputs.len(), 3); // Only the 3 frames
        assert!(!temp.path().join("sprites_frames.tres").exists());
        assert!(!temp.path().join("sprites_anims.tres").exists());
    }

    #[test]
    fn test_export_via_trait() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();
        let exporter = GodotExporter::new();
        let options = ExportOptions::default();

        exporter.export(&metadata, temp.path(), &options).unwrap();

        // Should have created files
        assert!(temp.path().join("player_idle.tres").exists());
    }

    #[test]
    fn test_godot_export_options_default() {
        let options = GodotExportOptions::default();
        assert_eq!(options.resource_path, "res://assets/sprites");
        assert!(options.sprite_frames);
        assert!(options.animation_player);
        assert!(options.atlas_textures);
    }

    #[test]
    fn test_atlas_texture_to_string() {
        let exporter = GodotExporter::new().with_resource_path("res://game");
        let content = exporter.export_atlas_texture_to_string("atlas.png", 0, 0, 16, 16);

        assert!(content.contains("res://game/atlas.png"));
        assert!(content.contains("Rect2(0, 0, 16, 16)"));
    }

    #[test]
    fn test_generate_animation_library() {
        let exporter = GodotExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.generate_animation_library(&metadata, "res://sprites");

        // Check resource header
        assert!(content.contains("[gd_resource type=\"AnimationLibrary\""));
        assert!(content.contains("format=3"));

        // Check external resources for frames
        assert!(content.contains("[ext_resource type=\"AtlasTexture\""));
        assert!(content.contains("player_walk_1.tres"));
        assert!(content.contains("player_walk_2.tres"));

        // Check animation sub-resource
        assert!(content.contains("[sub_resource type=\"Animation\""));
        assert!(content.contains("resource_name = \"walk\""));
        assert!(content.contains("loop_mode = 1"));
        assert!(content.contains("tracks/0/type = \"value\""));
        assert!(content.contains("tracks/0/path = NodePath(\".:texture\")"));

        // Check main resource section
        assert!(content.contains("[resource]"));
        assert!(content.contains("_data = {"));
        assert!(content.contains("\"walk\" = SubResource"));
    }

    #[test]
    fn test_animation_library_timing() {
        let exporter = GodotExporter::new();
        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [128, 64],
            frames: HashMap::from([
                (
                    "frame1".to_string(),
                    AtlasFrame { x: 0, y: 0, w: 32, h: 32, origin: None, boxes: None },
                ),
                (
                    "frame2".to_string(),
                    AtlasFrame { x: 32, y: 0, w: 32, h: 32, origin: None, boxes: None },
                ),
            ]),
            animations: HashMap::from([(
                "test_anim".to_string(),
                AtlasAnimation {
                    frames: vec!["frame1".to_string(), "frame2".to_string()],
                    fps: 10, // 10 fps = 0.2 seconds for 2 frames
                    tags: None,
                },
            )]),
        };

        let content = exporter.generate_animation_library(&metadata, "res://test");

        // 2 frames at 10 fps = 0.2 second animation
        assert!(content.contains("length = 0.2"));
        // Times should be evenly spaced: 0, 0.1
        assert!(content.contains("PackedFloat32Array(0.0"));
    }

    #[test]
    fn test_export_godot_animation_library_content() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();

        export_godot(&metadata, temp.path(), "res://game").unwrap();

        let content = fs::read_to_string(temp.path().join("sprites_anims.tres")).unwrap();

        // Verify AnimationLibrary structure
        assert!(content.contains("AnimationLibrary"));
        assert!(content.contains("Animation"));
        assert!(content.contains("walk"));
        assert!(content.contains("player_walk_1.tres"));
        assert!(content.contains("player_walk_2.tres"));
    }

    #[test]
    fn test_export_godot_options_no_animation_player() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_metadata();

        let exporter = GodotExporter::new()
            .with_resource_path("res://test")
            .with_sprite_frames(true)
            .with_animation_player(false);

        let options = GodotExportOptions {
            resource_path: "res://test".to_string(),
            sprite_frames: true,
            animation_player: false,
            atlas_textures: true,
            base: ExportOptions::default(),
        };

        let outputs = exporter.export_godot(&metadata, temp.path(), &options).unwrap();

        // Should create AtlasTextures + SpriteFrames, but not AnimationLibrary
        assert_eq!(outputs.len(), 4); // 3 frames + 1 sprite_frames
        assert!(temp.path().join("sprites_frames.tres").exists());
        assert!(!temp.path().join("sprites_anims.tres").exists());
    }

    #[test]
    fn test_animation_library_multiple_animations() {
        let exporter = GodotExporter::new();
        let metadata = AtlasMetadata {
            image: "multi.png".to_string(),
            size: [128, 64],
            frames: HashMap::from([
                (
                    "idle_1".to_string(),
                    AtlasFrame { x: 0, y: 0, w: 16, h: 16, origin: None, boxes: None },
                ),
                (
                    "idle_2".to_string(),
                    AtlasFrame { x: 16, y: 0, w: 16, h: 16, origin: None, boxes: None },
                ),
                (
                    "walk_1".to_string(),
                    AtlasFrame { x: 32, y: 0, w: 16, h: 16, origin: None, boxes: None },
                ),
                (
                    "walk_2".to_string(),
                    AtlasFrame { x: 48, y: 0, w: 16, h: 16, origin: None, boxes: None },
                ),
            ]),
            animations: HashMap::from([
                (
                    "idle".to_string(),
                    AtlasAnimation {
                        frames: vec!["idle_1".to_string(), "idle_2".to_string()],
                        fps: 5,
                        tags: None,
                    },
                ),
                (
                    "walk".to_string(),
                    AtlasAnimation {
                        frames: vec!["walk_1".to_string(), "walk_2".to_string()],
                        fps: 10,
                        tags: None,
                    },
                ),
            ]),
        };

        let content = exporter.generate_animation_library(&metadata, "res://sprites");

        // Both animations should be present
        assert!(content.contains("Animation_idle"));
        assert!(content.contains("Animation_walk"));
        assert!(content.contains("\"idle\" = SubResource"));
        assert!(content.contains("\"walk\" = SubResource"));
    }
}
