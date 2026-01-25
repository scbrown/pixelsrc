//! LSP types and data structures.

use crate::motion::Interpolation;
use crate::transforms::Transform;

/// Completion context for structured format elements
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
    /// Inside a sprite's regions object - suggest shape types and modifiers
    Regions,
    /// Inside a region definition - suggest modifiers
    RegionDef,
    /// Inside a palette's roles object - suggest role values
    Roles,
    /// Inside a palette's relationships object - suggest relationship types
    Relationships,
    /// Inside a state_rules rules array - suggest selector patterns
    StateRules,
    /// Inside a state rule's apply object - suggest applicable properties
    StateRuleApply,
    /// Unknown/other context
    Other,
}

/// Information about a timing function at cursor position
#[derive(Debug, Clone)]
pub struct TimingFunctionInfo {
    /// Raw timing function string from JSON
    pub function_str: String,
    /// Parsed interpolation
    pub interpolation: Interpolation,
}

/// Information about a color found in the document
#[derive(Debug, Clone)]
pub struct ColorMatch {
    /// The original color string as it appears in the document
    #[allow(dead_code)]
    pub original: String,
    /// The resolved RGBA values (0.0-1.0 range)
    pub rgba: (f32, f32, f32, f32),
    /// Start position in the line
    pub start: u32,
    /// End position in the line
    pub end: u32,
}

/// Information about a transform at a cursor position
#[derive(Debug, Clone)]
pub struct TransformInfo {
    /// The parsed transform
    pub transform: Transform,
    /// The raw transform string
    pub raw: String,
    /// Object type (sprite, animation, composition, etc.)
    pub object_type: String,
    /// Object name
    pub object_name: String,
    /// Index in the transform array (0-indexed)
    pub index: usize,
    /// Total number of transforms in the array
    pub total: usize,
}
