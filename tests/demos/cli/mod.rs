//! CLI Command Demo Tests
//!
//! Demonstrates the command-line interface functionality for Pixelsrc.
//! These tests verify the behavior of CLI commands through their underlying
//! library functions, serving as both verification and documentation.
//!
//! ## Commands Covered
//!
//! - `pxl render` - Render sprites to PNG format
//! - `pxl validate` - Validate Pixelsrc files for correctness
//! - `pxl import` - Import PNG images to Pixelsrc format

pub mod import_cmd;
pub mod render;
pub mod validate;
