//! Build pipeline module for pixelsrc
//!
//! Provides the core build system for processing `.pxl` files into
//! sprite atlases, animations, and game engine exports.
//!
//! # Overview
//!
//! The build pipeline consists of:
//! - **Discovery**: Find source files using glob patterns from config
//! - **Planning**: Create a build plan with steps for each target
//! - **Execution**: Run build steps to produce outputs
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::build::{BuildContext, BuildPipeline};
//! use pixelsrc::config::load_config;
//!
//! let config = load_config(None)?;
//! let context = BuildContext::new(config, project_root);
//! let pipeline = BuildPipeline::new(context);
//!
//! let result = pipeline.build()?;
//! println!("Built {} targets", result.targets_built);
//! ```

pub mod context;
pub mod discovery;
pub mod incremental;
pub mod manifest;
pub mod parallel;
pub mod pipeline;
pub mod result;
pub mod target;

pub use context::*;
pub use discovery::*;
pub use incremental::*;
pub use manifest::*;
pub use parallel::*;
pub use pipeline::*;
pub use result::*;
pub use target::*;
