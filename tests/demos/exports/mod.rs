//! Atlas Export Demo Tests (DT-8)
//!
//! Demonstrates atlas metadata export to various game engine formats:
//! - Godot (.tres resource files)
//! - Unity (JSON metadata + .meta + .anim files)
//! - libGDX (.atlas TextureAtlas format)
//!
//! Each demo parses sprite/animation JSONL fixtures and exports atlas
//! metadata in the target engine's format.

mod atlas_aseprite;
mod atlas_godot;
mod atlas_libgdx;
mod atlas_unity;
