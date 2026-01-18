//! Export formats for atlas metadata.
//!
//! This module provides exporters for various game engine and tool formats.
//! The generic JSON export provides a universal format that can be used as a
//! base for custom integrations.
//!
//! # Supported Formats
//!
//! - **JSON** (BST-6): Generic JSON format with frame positions, animations, and metadata
//! - **Godot** (BST-12): Godot engine .tres resource files
//! - **Unity** (BST-13): Unity sprite metadata JSON
//! - **libGDX** (BST-14): libGDX TextureAtlas format (.atlas files)
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::export::{JsonExporter, ExportOptions};
//! use pixelsrc::atlas::AtlasMetadata;
//!
//! let metadata = AtlasMetadata { /* ... */ };
//! let exporter = JsonExporter::new();
//! exporter.export(&metadata, "output.json", &ExportOptions::default())?;
//! ```

pub mod godot;
pub mod json;
pub mod libgdx;
pub mod unity;

pub use godot::*;
pub use json::*;
pub use libgdx::*;
pub use unity::*;

use std::path::Path;

/// Common error type for export operations.
#[derive(Debug)]
pub enum ExportError {
    /// IO error during file writing
    Io(std::io::Error),
    /// Serialization error
    Serialization(String),
    /// Invalid configuration
    Config(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(e) => write!(f, "IO error: {}", e),
            ExportError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ExportError::Config(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        ExportError::Io(e)
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(e: serde_json::Error) -> Self {
        ExportError::Serialization(e.to_string())
    }
}

/// Options for export operations.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Pretty print output (with indentation)
    pub pretty: bool,
    /// Include source file paths in output
    pub include_sources: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self { pretty: true, include_sources: false }
    }
}

/// Trait for export format implementations.
pub trait Exporter {
    /// Export atlas metadata to the specified path.
    fn export(
        &self,
        metadata: &crate::atlas::AtlasMetadata,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<(), ExportError>;

    /// Get the format name for this exporter.
    fn format_name(&self) -> &'static str;

    /// Get the default file extension for this format.
    fn extension(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();
        assert!(options.pretty);
        assert!(!options.include_sources);
    }

    #[test]
    fn test_export_error_display() {
        let io_err =
            ExportError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(io_err.to_string().contains("IO error"));

        let ser_err = ExportError::Serialization("invalid json".to_string());
        assert!(ser_err.to_string().contains("Serialization error"));

        let cfg_err = ExportError::Config("missing field".to_string());
        assert!(cfg_err.to_string().contains("Configuration error"));
    }
}
