//! Pixelsrc - Library for parsing and rendering pixel art
//!
//! This library provides functionality to:
//! - Parse JSONL files containing palette and sprite definitions
//! - Render sprites to PNG images
//! - Support both lenient and strict error modes

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
pub mod export;
pub mod explain;
pub mod fmt;
pub mod gif;
pub mod import;
pub mod include;
pub mod init;
pub mod models;
pub mod onion;
pub mod output;
pub mod palette_cycle;
pub mod palettes;
pub mod parser;
pub mod prime;
pub mod registry;
pub mod renderer;
pub mod spritesheet;
pub mod suggest;
pub mod templates;
pub mod terminal;
pub mod tokenizer;
pub mod transforms;
pub mod validate;

#[cfg(feature = "wasm")]
pub mod wasm;
