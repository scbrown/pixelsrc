//! Antialiasing algorithm implementations.
//!
//! This module contains implementations of various upscaling algorithms
//! used for antialiasing pixel art. Each algorithm respects semantic
//! context for intelligent smoothing decisions.
//!
//! # Available Algorithms
//!
//! - [`scale2x`] - Scale2x (EPX) edge-aware 2x upscaling

pub mod scale2x;

pub use scale2x::{scale2x, Scale2xOptions};
