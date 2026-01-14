//! TTP (Text To Pixel) - Library for parsing and rendering pixel art
//!
//! This library provides functionality to:
//! - Parse JSONL files containing palette and sprite definitions
//! - Render sprites to PNG images
//! - Support both lenient and strict error modes

pub mod cli;
pub mod color;
pub mod models;
pub mod output;
pub mod parser;
pub mod registry;
pub mod renderer;
pub mod tokenizer;
