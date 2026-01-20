//! Composition Demo Tests (DT-5)
//!
//! Demonstrates composition features:
//! - Basic sprite stacking/layering
//! - Blend modes (normal, multiply, screen, overlay)
//! - Grid positioning via cell_size and maps
//! - Background fills (solid and pattern)
//!
//! Each demo parses composition JSONL fixtures and verifies layer structure,
//! blend modes, and sprite resolution.

mod basic_layers;
mod blend_modes;
mod fills;
mod positioning;
