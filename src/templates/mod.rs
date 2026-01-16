//! Project templates for pixelsrc.
//!
//! Provides template generation for project files like justfiles,
//! configuration, and scaffolding.
//!
//! # Justfile Templates
//!
//! Generate justfiles tailored to different workflows:
//!
//! ```ignore
//! use pixelsrc::templates::{JustfileTemplate, generate_justfile};
//!
//! let content = generate_justfile(JustfileTemplate::Game, "my-game");
//! std::fs::write("justfile", content)?;
//! ```
//!
//! Available templates:
//! - `Minimal`: Basic render commands
//! - `Artist`: Static art workflow
//! - `Animator`: Animation and GIF workflow
//! - `Game`: Full game asset pipeline with atlases

pub mod justfile;

pub use justfile::*;
