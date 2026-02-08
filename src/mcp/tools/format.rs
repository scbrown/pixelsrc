//! Input schema for pixelsrc_format tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FormatInput {
    /// The .pxl source content to format.
    pub source: String,
}
