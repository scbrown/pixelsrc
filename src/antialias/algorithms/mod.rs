//! Antialiasing algorithm implementations.
//!
//! This module contains implementations of various antialiasing algorithms
//! that can be applied to rendered pixel art sprites.
//!
//! # Available Algorithms
//!
//! - [`blur`] - Gaussian blur with semantic masking (`aa-blur`)
//!
//! # Planned Algorithms
//!
//! - `scale2x` - Scale2x (EPX) algorithm for 2x upscaling
//! - `hq2x`/`hq4x` - High-quality 2x/4x upscaling
//! - `xbr2x`/`xbr4x` - xBR edge-aware 2x/4x upscaling

pub mod blur;

pub use blur::apply_semantic_blur;
