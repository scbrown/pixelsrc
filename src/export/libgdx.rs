//! libGDX engine export format.
//!
//! Exports atlas metadata to libGDX TextureAtlas format (.atlas files).
//! This is the standard format used by libGDX's TexturePacker and TextureAtlas classes.
//!
//! # Output Format
//!
//! The libGDX exporter generates a text-based .atlas file:
//!
//! ```text
//! atlas.png
//! size: 256, 256
//! format: RGBA8888
//! filter: Nearest, Nearest
//! repeat: none
//! player_idle
//!   rotate: false
//!   xy: 0, 0
//!   size: 32, 32
//!   orig: 32, 32
//!   offset: 0, 0
//!   index: -1
//! player_walk
//!   rotate: false
//!   xy: 32, 0
//!   size: 32, 32
//!   orig: 32, 32
//!   offset: 0, 0
//!   index: 0
//! ```
//!
//! This format is directly loadable by libGDX's `TextureAtlas` class.

use crate::atlas::AtlasMetadata;
use crate::export::{ExportError, ExportOptions, Exporter, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// libGDX texture filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibGdxFilterMode {
    /// Nearest neighbor filtering - pixel perfect
    #[default]
    Nearest,
    /// Linear filtering - smooth
    Linear,
    /// Mipmap with nearest filtering
    MipMapNearestNearest,
    /// Mipmap with linear filtering
    MipMapLinearLinear,
}

impl LibGdxFilterMode {
    /// Get the libGDX format string for this filter mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            LibGdxFilterMode::Nearest => "Nearest",
            LibGdxFilterMode::Linear => "Linear",
            LibGdxFilterMode::MipMapNearestNearest => "MipMapNearestNearest",
            LibGdxFilterMode::MipMapLinearLinear => "MipMapLinearLinear",
        }
    }
}

/// libGDX texture repeat mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibGdxRepeatMode {
    /// No repeat (clamp to edge)
    #[default]
    None,
    /// Repeat in X direction
    X,
    /// Repeat in Y direction
    Y,
    /// Repeat in both directions
    XY,
}

impl LibGdxRepeatMode {
    /// Get the libGDX format string for this repeat mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            LibGdxRepeatMode::None => "none",
            LibGdxRepeatMode::X => "x",
            LibGdxRepeatMode::Y => "y",
            LibGdxRepeatMode::XY => "xy",
        }
    }
}

/// libGDX export configuration options.
#[derive(Debug, Clone)]
pub struct LibGdxExportOptions {
    /// Base export options
    pub base: ExportOptions,
    /// Texture filter mode (minification, magnification)
    pub filter: (LibGdxFilterMode, LibGdxFilterMode),
    /// Texture repeat mode
    pub repeat: LibGdxRepeatMode,
    /// Pixel format (e.g., "RGBA8888", "RGB888", "RGBA4444")
    pub format: String,
}

impl Default for LibGdxExportOptions {
    fn default() -> Self {
        Self {
            base: ExportOptions::default(),
            filter: (LibGdxFilterMode::Nearest, LibGdxFilterMode::Nearest),
            repeat: LibGdxRepeatMode::None,
            format: "RGBA8888".to_string(),
        }
    }
}

/// libGDX format exporter.
///
/// Exports atlas metadata to libGDX TextureAtlas (.atlas) format.
#[derive(Debug)]
pub struct LibGdxExporter {
    /// Minification filter
    min_filter: LibGdxFilterMode,
    /// Magnification filter
    mag_filter: LibGdxFilterMode,
    /// Repeat mode
    repeat: LibGdxRepeatMode,
    /// Pixel format
    format: String,
}

impl Default for LibGdxExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl LibGdxExporter {
    /// Create a new libGDX exporter with default settings.
    pub fn new() -> Self {
        Self {
            min_filter: LibGdxFilterMode::Nearest,
            mag_filter: LibGdxFilterMode::Nearest,
            repeat: LibGdxRepeatMode::None,
            format: "RGBA8888".to_string(),
        }
    }

    /// Set the minification filter.
    pub fn with_min_filter(mut self, filter: LibGdxFilterMode) -> Self {
        self.min_filter = filter;
        self
    }

    /// Set the magnification filter.
    pub fn with_mag_filter(mut self, filter: LibGdxFilterMode) -> Self {
        self.mag_filter = filter;
        self
    }

    /// Set both filters to the same mode.
    pub fn with_filter(mut self, filter: LibGdxFilterMode) -> Self {
        self.min_filter = filter;
        self.mag_filter = filter;
        self
    }

    /// Set the repeat mode.
    pub fn with_repeat(mut self, repeat: LibGdxRepeatMode) -> Self {
        self.repeat = repeat;
        self
    }

    /// Set the pixel format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Export atlas metadata to libGDX format.
    pub fn export_libgdx(
        &self,
        metadata: &AtlasMetadata,
        output_path: &Path,
        _options: &LibGdxExportOptions,
    ) -> Result<()> {
        let content = self.generate_atlas_content(metadata);

        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Generate the atlas file content as a string.
    pub fn export_to_string(&self, metadata: &AtlasMetadata) -> String {
        self.generate_atlas_content(metadata)
    }

    /// Generate the libGDX TextureAtlas content.
    fn generate_atlas_content(&self, metadata: &AtlasMetadata) -> String {
        let mut content = String::new();

        // Header: image filename
        content.push_str(&metadata.image);
        content.push('\n');

        // Atlas properties
        content.push_str(&format!("size: {}, {}\n", metadata.size[0], metadata.size[1]));
        content.push_str(&format!("format: {}\n", self.format));
        content.push_str(&format!(
            "filter: {}, {}\n",
            self.min_filter.as_str(),
            self.mag_filter.as_str()
        ));
        content.push_str(&format!("repeat: {}\n", self.repeat.as_str()));

        // Collect and sort frame names for deterministic output
        let mut frame_names: Vec<&String> = metadata.frames.keys().collect();
        frame_names.sort();

        // Build animation frame indices map
        let animation_indices = self.build_animation_indices(metadata);

        // Write each frame/region
        for name in frame_names {
            if let Some(frame) = metadata.frames.get(name) {
                content.push_str(name);
                content.push('\n');

                // Region properties (indented with 2 spaces)
                content.push_str("  rotate: false\n");
                content.push_str(&format!("  xy: {}, {}\n", frame.x, frame.y));
                content.push_str(&format!("  size: {}, {}\n", frame.w, frame.h));
                content.push_str(&format!("  orig: {}, {}\n", frame.w, frame.h));

                // Calculate offset from origin if present
                let (offset_x, offset_y) =
                    if let Some(origin) = &frame.origin { (origin[0], origin[1]) } else { (0, 0) };
                content.push_str(&format!("  offset: {}, {}\n", offset_x, offset_y));

                // Index for animation sequences (-1 means not part of sequence)
                let index = animation_indices.get(name.as_str()).copied().unwrap_or(-1);
                content.push_str(&format!("  index: {}\n", index));
            }
        }

        content
    }

    /// Build a map of frame names to their animation sequence indices.
    fn build_animation_indices<'a>(
        &self,
        metadata: &'a AtlasMetadata,
    ) -> std::collections::HashMap<&'a str, i32> {
        let mut indices = std::collections::HashMap::new();

        for animation in metadata.animations.values() {
            for (i, frame_name) in animation.frames.iter().enumerate() {
                indices.insert(frame_name.as_str(), i as i32);
            }
        }

        indices
    }
}

impl Exporter for LibGdxExporter {
    fn export(
        &self,
        metadata: &AtlasMetadata,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<()> {
        let libgdx_options = LibGdxExportOptions { base: options.clone(), ..Default::default() };
        self.export_libgdx(metadata, output_path, &libgdx_options)
    }

    fn format_name(&self) -> &'static str {
        "libGDX TextureAtlas"
    }

    fn extension(&self) -> &'static str {
        "atlas"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::{AtlasAnimation, AtlasFrame};
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_test_metadata() -> AtlasMetadata {
        let mut frames = HashMap::new();
        frames.insert(
            "player_idle".to_string(),
            AtlasFrame { x: 0, y: 0, w: 32, h: 32, origin: None, boxes: None },
        );
        frames.insert(
            "player_walk_1".to_string(),
            AtlasFrame { x: 32, y: 0, w: 32, h: 32, origin: Some([16, 32]), boxes: None },
        );
        frames.insert(
            "player_walk_2".to_string(),
            AtlasFrame { x: 64, y: 0, w: 32, h: 32, origin: Some([16, 32]), boxes: None },
        );

        let mut animations = HashMap::new();
        animations.insert(
            "walk".to_string(),
            AtlasAnimation {
                frames: vec!["player_walk_1".to_string(), "player_walk_2".to_string()],
                fps: 10,
                tags: None,
            },
        );

        AtlasMetadata { image: "atlas.png".to_string(), size: [128, 64], frames, animations }
    }

    #[test]
    fn test_libgdx_exporter_new() {
        let exporter = LibGdxExporter::new();
        assert_eq!(exporter.min_filter, LibGdxFilterMode::Nearest);
        assert_eq!(exporter.mag_filter, LibGdxFilterMode::Nearest);
        assert_eq!(exporter.repeat, LibGdxRepeatMode::None);
        assert_eq!(exporter.format, "RGBA8888");
    }

    #[test]
    fn test_libgdx_exporter_with_options() {
        let exporter = LibGdxExporter::new()
            .with_min_filter(LibGdxFilterMode::Linear)
            .with_mag_filter(LibGdxFilterMode::MipMapLinearLinear)
            .with_repeat(LibGdxRepeatMode::XY)
            .with_format("RGB888");

        assert_eq!(exporter.min_filter, LibGdxFilterMode::Linear);
        assert_eq!(exporter.mag_filter, LibGdxFilterMode::MipMapLinearLinear);
        assert_eq!(exporter.repeat, LibGdxRepeatMode::XY);
        assert_eq!(exporter.format, "RGB888");
    }

    #[test]
    fn test_libgdx_export_options_default() {
        let options = LibGdxExportOptions::default();
        assert_eq!(options.filter.0, LibGdxFilterMode::Nearest);
        assert_eq!(options.filter.1, LibGdxFilterMode::Nearest);
        assert_eq!(options.repeat, LibGdxRepeatMode::None);
        assert_eq!(options.format, "RGBA8888");
    }

    #[test]
    fn test_filter_mode_as_str() {
        assert_eq!(LibGdxFilterMode::Nearest.as_str(), "Nearest");
        assert_eq!(LibGdxFilterMode::Linear.as_str(), "Linear");
        assert_eq!(LibGdxFilterMode::MipMapNearestNearest.as_str(), "MipMapNearestNearest");
        assert_eq!(LibGdxFilterMode::MipMapLinearLinear.as_str(), "MipMapLinearLinear");
    }

    #[test]
    fn test_repeat_mode_as_str() {
        assert_eq!(LibGdxRepeatMode::None.as_str(), "none");
        assert_eq!(LibGdxRepeatMode::X.as_str(), "x");
        assert_eq!(LibGdxRepeatMode::Y.as_str(), "y");
        assert_eq!(LibGdxRepeatMode::XY.as_str(), "xy");
    }

    #[test]
    fn test_export_to_string_header() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        assert!(content.starts_with("atlas.png\n"));
        assert!(content.contains("size: 128, 64\n"));
        assert!(content.contains("format: RGBA8888\n"));
        assert!(content.contains("filter: Nearest, Nearest\n"));
        assert!(content.contains("repeat: none\n"));
    }

    #[test]
    fn test_export_to_string_frames() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        // Check player_idle frame
        assert!(content.contains("player_idle\n"));
        assert!(content.contains("  xy: 0, 0\n"));
        assert!(content.contains("  size: 32, 32\n"));

        // Check player_walk_1 frame
        assert!(content.contains("player_walk_1\n"));
        assert!(content.contains("  xy: 32, 0\n"));
    }

    #[test]
    fn test_export_frame_with_origin() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        // player_walk_1 has origin [16, 32]
        // The offset should reflect this
        assert!(content.contains("  offset: 16, 32\n"));
    }

    #[test]
    fn test_export_frame_without_origin() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        // player_idle has no origin, should default to 0, 0
        // Need to check the specific frame's offset
        let lines: Vec<&str> = content.lines().collect();
        let idle_idx = lines.iter().position(|l| *l == "player_idle").unwrap();
        let offset_line = lines[idle_idx + 5]; // offset is 5 lines after frame name
        assert_eq!(offset_line, "  offset: 0, 0");
    }

    #[test]
    fn test_export_animation_indices() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        // player_walk_1 should have index 0, player_walk_2 should have index 1
        let lines: Vec<&str> = content.lines().collect();

        // Find player_walk_1 and check its index
        let walk1_idx = lines.iter().position(|l| *l == "player_walk_1").unwrap();
        let index1_line = lines[walk1_idx + 6]; // index is 6 lines after frame name
        assert_eq!(index1_line, "  index: 0");

        // Find player_walk_2 and check its index
        let walk2_idx = lines.iter().position(|l| *l == "player_walk_2").unwrap();
        let index2_line = lines[walk2_idx + 6];
        assert_eq!(index2_line, "  index: 1");

        // player_idle should have index -1 (not in animation)
        let idle_idx = lines.iter().position(|l| *l == "player_idle").unwrap();
        let index_idle_line = lines[idle_idx + 6];
        assert_eq!(index_idle_line, "  index: -1");
    }

    #[test]
    fn test_export_creates_file() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let options = LibGdxExportOptions::default();

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test.atlas");

        exporter.export_libgdx(&metadata, &output_path, &options).unwrap();

        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.starts_with("atlas.png\n"));
    }

    #[test]
    fn test_export_creates_directories() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let options = LibGdxExportOptions::default();

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("nested").join("dir").join("test.atlas");

        exporter.export_libgdx(&metadata, &output_path, &options).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_export_via_trait() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let options = ExportOptions::default();

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test.atlas");

        exporter.export(&metadata, &output_path, &options).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_format_name() {
        let exporter = LibGdxExporter::new();
        assert_eq!(exporter.format_name(), "libGDX TextureAtlas");
    }

    #[test]
    fn test_extension() {
        let exporter = LibGdxExporter::new();
        assert_eq!(exporter.extension(), "atlas");
    }

    #[test]
    fn test_export_with_linear_filter() {
        let exporter = LibGdxExporter::new().with_filter(LibGdxFilterMode::Linear);
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        assert!(content.contains("filter: Linear, Linear\n"));
    }

    #[test]
    fn test_export_with_repeat() {
        let exporter = LibGdxExporter::new().with_repeat(LibGdxRepeatMode::XY);
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        assert!(content.contains("repeat: xy\n"));
    }

    #[test]
    fn test_frames_sorted_alphabetically() {
        let exporter = LibGdxExporter::new();
        let metadata = create_test_metadata();
        let content = exporter.export_to_string(&metadata);

        let lines: Vec<&str> = content.lines().collect();

        // Find frame name lines (lines that don't start with whitespace after header)
        let frame_lines: Vec<&str> = lines
            .iter()
            .skip(5) // Skip header lines
            .filter(|l| !l.starts_with(' ') && !l.is_empty())
            .copied()
            .collect();

        assert_eq!(frame_lines, vec!["player_idle", "player_walk_1", "player_walk_2"]);
    }

    #[test]
    fn test_export_empty_animations() {
        let exporter = LibGdxExporter::new();

        let mut frames = HashMap::new();
        frames.insert(
            "sprite".to_string(),
            AtlasFrame { x: 0, y: 0, w: 16, h: 16, origin: None, boxes: None },
        );

        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [16, 16],
            frames,
            animations: HashMap::new(),
        };

        let content = exporter.export_to_string(&metadata);

        // With no animations, all frames should have index -1
        assert!(content.contains("  index: -1\n"));
    }
}
