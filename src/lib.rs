//! Pixelsrc - Library for parsing and rendering pixel art
//!
//! This library provides functionality to:
//! - Parse JSONL files containing palette and sprite definitions
//! - Render sprites to PNG images
//! - Support both lenient and strict error modes
//!
//! # Quick Start
//!
//! ```no_run
//! use pixelsrc::{parse_stream, TtpObject, PaletteRegistry, SpriteRegistry, render_resolved};
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! // Parse a .pxl file
//! let file = File::open("sprites.pxl").unwrap();
//! let result = parse_stream(BufReader::new(file));
//!
//! // Build registries
//! let mut palettes = PaletteRegistry::new();
//! let mut sprites = SpriteRegistry::new();
//!
//! for obj in result.objects {
//!     match obj {
//!         TtpObject::Palette(p) => palettes.register(p),
//!         TtpObject::Sprite(s) => sprites.register_sprite(s),
//!         TtpObject::Variant(v) => sprites.register_variant(v),
//!         _ => {}
//!     }
//! }
//!
//! // Resolve and render a sprite
//! let resolved = sprites.resolve("my_sprite", &palettes, false).unwrap();
//! let (image, warnings) = render_resolved(&resolved);
//! ```

pub mod alias;
pub mod analyze;
pub mod animation;
pub mod atlas;
pub mod build;
pub mod cli;
pub mod color;
pub mod composition;
pub mod config;
pub mod diff;
pub mod emoji;
pub mod explain;
pub mod export;
pub mod fmt;
pub mod gif;
pub mod import;
pub mod include;
pub mod init;
pub mod lsp;
pub mod lsp_agent_client;
pub mod models;
pub mod motion;
pub mod onion;
pub mod output;
pub mod palette_cycle;
pub mod palette_parser;
pub mod palettes;
pub mod parser;
pub mod prime;
pub mod registry;
pub mod renderer;
pub mod scaffold;
pub mod spritesheet;
pub mod suggest;
pub mod templates;
pub mod terminal;
pub mod tokenizer;
pub mod transforms;
pub mod validate;
pub mod variables;
pub mod watch;

#[cfg(feature = "wasm")]
pub mod wasm;

// ============================================================================
// Re-exports for convenience
// ============================================================================

// Core data types
pub use models::{
    Animation, Composition, CompositionLayer, Palette, PaletteRef, Sprite, TransformSpec,
    TtpObject, Variant, Warning,
};

// Parsing
pub use parser::{parse_line, parse_stream, ParseError, ParseResult};

// Color
pub use color::{parse_color, ColorError};

// Registry types
pub use registry::{
    PaletteError, PaletteRegistry, PaletteSource, Registry, ResolvedPalette, ResolvedSprite,
    SpriteRegistry,
};

// Rendering
pub use renderer::{render_resolved, render_sprite};

// Tokenizer
pub use tokenizer::tokenize;
