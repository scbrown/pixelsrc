//! MCP resource handlers for pixelsrc.
//!
//! Exposes static resources (format spec, brief guide, palette catalog)
//! via the MCP resources/list and resources/read protocol.

mod static_resources;

pub use static_resources::{list_static_resources, read_static_resource};
