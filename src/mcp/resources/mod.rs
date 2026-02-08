//! MCP resource handlers for pixelsrc.
//!
//! Exposes static resources (format spec, brief guide, palette catalog)
//! and dynamic resource templates (individual palettes, examples, prompt templates)
//! via the MCP resources/list, resources/read, and resources/templates/list protocol.

mod static_resources;
mod templates;

pub use static_resources::{list_static_resources, read_static_resource};
pub use templates::{list_resource_templates, read_template_resource};
