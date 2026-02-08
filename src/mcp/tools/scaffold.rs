//! Input schema for pixelsrc_scaffold tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScaffoldInput {
    /// Asset type to scaffold: "sprite", "animation" (or "anim"), "palette", or "composition".
    pub asset_type: String,

    /// Name for the generated asset. Must be lowercase letters, numbers, and underscores.
    pub name: String,

    /// Optional palette name or preset. For palette assets, this selects a preset
    /// (forest, medieval, synthwave, ocean). For sprite/composition, this names the palette.
    pub palette: Option<String>,

    /// Width in pixels (for sprite/composition). Default: 16.
    pub width: Option<u32>,

    /// Height in pixels (for sprite/composition). Default: 16.
    pub height: Option<u32>,

    /// Cell width for compositions. Default: 8.
    pub cell_width: Option<u32>,

    /// Cell height for compositions. Default: 8.
    pub cell_height: Option<u32>,

    /// Comma-separated list of token names for sprite palette generation.
    pub tokens: Option<String>,

    /// Comma-separated hex colors for palette generation (e.g. "#FF0000,#00FF00").
    pub colors: Option<String>,
}
