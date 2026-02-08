//! Input schema for pixelsrc_suggest tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestInput {
    /// Inline .pxl source content to analyze for suggestions.
    pub source: Option<String>,

    /// Path to a .pxl file to analyze for suggestions.
    pub path: Option<String>,
}
