//! Error and warning types for composition rendering

use std::fmt;

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
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionError {
    /// Sprite dimensions exceed cell size
    SizeMismatch {
        sprite_name: String,
        sprite_size: (u32, u32),
        cell_size: (u32, u32),
        composition_name: String,
    },
    /// Canvas size is not divisible by cell_size
    SizeNotDivisible { size: (u32, u32), cell_size: (u32, u32), composition_name: String },
    /// Map dimensions don't match expected grid size
    MapDimensionMismatch {
        layer_name: Option<String>,
        actual_dimensions: (usize, usize),
        expected_dimensions: (u32, u32),
        composition_name: String,
    },
}

impl fmt::Display for CompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositionError::SizeMismatch {
                sprite_name,
                sprite_size,
                cell_size,
                composition_name,
            } => write!(
                f,
                "Sprite '{}' ({}x{}) exceeds cell size ({}x{}) in composition '{}'",
                sprite_name,
                sprite_size.0,
                sprite_size.1,
                cell_size.0,
                cell_size.1,
                composition_name
            ),
            CompositionError::SizeNotDivisible { size, cell_size, composition_name } => write!(
                f,
                "Size ({}x{}) is not divisible by cell_size ({}x{}) in composition '{}'",
                size.0, size.1, cell_size.0, cell_size.1, composition_name
            ),
            CompositionError::MapDimensionMismatch {
                layer_name,
                actual_dimensions,
                expected_dimensions,
                composition_name,
            } => {
                let layer_desc = layer_name
                    .as_ref()
                    .map(|n| format!("layer '{}'", n))
                    .unwrap_or_else(|| "unnamed layer".to_string());
                write!(
                    f,
                    "Map dimensions ({}x{}) don't match expected grid size ({}x{}) for {} in composition '{}'",
                    actual_dimensions.0,
                    actual_dimensions.1,
                    expected_dimensions.0,
                    expected_dimensions.1,
                    layer_desc,
                    composition_name
                )
            }
        }
    }
}

impl std::error::Error for CompositionError {}
