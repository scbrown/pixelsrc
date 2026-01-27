//! Antialiasing Demo Tests (Phase AA)
//!
//! Demo tests for semantic-aware antialiasing algorithms and preservation behavior.
//!
//! # Algorithms Demonstrated
//!
//! - `nearest` - No antialiasing (passthrough), preserves crisp pixel art
//! - `scale2x` - 2x edge-aware upscaling using EPX algorithm
//! - `hq2x` - 2x pattern-based upscaling with YUV color comparison
//! - `hq4x` - 4x pattern-based upscaling with fine interpolation
//! - `aa-blur` - Gaussian blur with semantic masking
//!
//! # Semantic Preservation
//!
//! - **Anchors** - Critical details (eyes, etc.) remain crisp
//! - **Containment** - Region boundaries don't blend across
//! - **Gradients** - Shadow/highlight transitions smooth naturally

mod algorithms;
mod semantic;
