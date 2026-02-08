//! MCP tool definitions for Pixelsrc
//!
//! Each tool wraps an existing library function, exposing it as a structured
//! MCP tool that AI models can call.

pub mod analyze;
pub mod format;
pub mod import;
pub mod palettes;
pub mod prime;
pub mod render;
pub mod scaffold;
pub mod suggest;
pub mod validate;
