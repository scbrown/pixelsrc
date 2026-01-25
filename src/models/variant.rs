//! Variant type for palette-only sprite modifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::transform::TransformSpec;

/// A variant is a palette-only modification of a base sprite.
///
/// Variants allow creating color variations of sprites without duplicating
/// the region data. The variant copies the base sprite's regions and applies
/// palette overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variant {
    pub name: String,
    pub base: String,
    pub palette: HashMap<String, String>,
    /// Transforms to apply when resolving this variant
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<Vec<TransformSpec>>,
}
