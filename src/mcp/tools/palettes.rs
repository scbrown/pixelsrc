//! Input schema for pixelsrc_palettes tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PalettesInput {
    /// Action to perform: "list" to list all palette names, "show" to show a specific palette.
    pub action: String,

    /// Palette name (required when action is "show"). May include or omit the leading "@".
    pub name: Option<String>,
}
