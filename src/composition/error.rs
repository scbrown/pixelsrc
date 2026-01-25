//! Error types for composition rendering

use thiserror::Error;

/// A warning generated during composition rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

/// Error when rendering a composition in strict mode.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompositionError {
    /// Sprite dimensions exceed cell size
    #[error("Sprite '{sprite_name}' ({sprite_w}x{sprite_h}) exceeds cell size ({cell_w}x{cell_h}) in composition '{composition_name}'", sprite_w = sprite_size.0, sprite_h = sprite_size.1, cell_w = cell_size.0, cell_h = cell_size.1)]
    SizeMismatch {
        sprite_name: String,
        sprite_size: (u32, u32),
        cell_size: (u32, u32),
        composition_name: String,
    },
    /// Canvas size is not divisible by cell_size
    #[error("Size ({size_w}x{size_h}) is not divisible by cell_size ({cell_w}x{cell_h}) in composition '{composition_name}'", size_w = size.0, size_h = size.1, cell_w = cell_size.0, cell_h = cell_size.1)]
    SizeNotDivisible { size: (u32, u32), cell_size: (u32, u32), composition_name: String },
    /// Map dimensions don't match expected grid size
    #[error("Map dimensions ({actual_w}x{actual_h}) don't match expected grid size ({expected_w}x{expected_h}) for {layer_desc} in composition '{composition_name}'", actual_w = actual_dimensions.0, actual_h = actual_dimensions.1, expected_w = expected_dimensions.0, expected_h = expected_dimensions.1, layer_desc = layer_name.as_ref().map(|n| format!("layer '{}'", n)).unwrap_or_else(|| "unnamed layer".to_string()))]
    MapDimensionMismatch {
        layer_name: Option<String>,
        actual_dimensions: (usize, usize),
        expected_dimensions: (u32, u32),
        composition_name: String,
    },
    /// Cycle detected in nested composition references
    #[error("Cycle detected in composition rendering: {}", cycle_path.join(" -> "))]
    CycleDetected {
        /// The composition names forming the cycle (e.g., ["A", "B", "A"])
        cycle_path: Vec<String>,
    },
}
