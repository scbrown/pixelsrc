//! Configuration module for pixelsrc build system
//!
//! Provides types and parsing for `pxl.toml` project configuration.
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::config::{load_config, find_config, merge_cli_overrides, CliOverrides};
//!
//! // Find and load config
//! let mut config = load_config(None)?;
//!
//! // Apply CLI overrides
//! let overrides = CliOverrides {
//!     out: Some("dist".into()),
//!     strict: Some(true),
//!     ..Default::default()
//! };
//! merge_cli_overrides(&mut config, &overrides);
//! ```

pub mod loader;
pub mod schema;

pub use loader::*;
pub use schema::*;
