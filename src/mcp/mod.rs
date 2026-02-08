//! MCP (Model Context Protocol) server implementation for Pixelsrc
//!
//! Exposes pixelsrc capabilities as MCP tools and resources so AI models
//! can render, validate, explain, and manipulate pixel art directly.
//!
//! Start the server with `pxl mcp` (hidden command, feature-gated).

pub mod prompts;
pub mod resources;
mod server;
pub mod tools;

pub use server::{run_server, PixelsrcMcpServer};
