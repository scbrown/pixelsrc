//! Antialiasing algorithm implementations.
//!
//! This module contains implementations of various antialiasing algorithms
//! that can be applied to rendered pixel art sprites. Each algorithm respects
//! semantic context for intelligent smoothing decisions.
//!
//! # Available Algorithms
//!
//! - [`blur`] - Gaussian blur with semantic masking (`aa-blur`)
//! - [`scale2x`] - Scale2x (EPX) edge-aware 2x upscaling
//! - [`hqx`] - HQ2x/HQ4x lookup table based upscaling with semantic awareness
//! - [`xbr`] - xBR2x/xBR4x edge direction and curvature detection (best quality)

pub mod blur;
pub mod hqx;
pub mod scale2x;
pub mod xbr;

pub use blur::apply_semantic_blur;
pub use hqx::{hq2x, hq4x};
pub use scale2x::{scale2x, Scale2xOptions};
pub use xbr::{xbr2x, xbr4x};
