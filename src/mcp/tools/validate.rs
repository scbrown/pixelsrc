//! Input schema for pixelsrc_validate tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateInput {
    /// Inline .pxl source content to validate.
    pub source: Option<String>,

    /// Path to a .pxl file to validate.
    pub path: Option<String>,

    /// If true, treat warnings as errors (default: false).
    #[serde(default)]
    pub strict: bool,
}
