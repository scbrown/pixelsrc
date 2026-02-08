//! Input schema for pixelsrc_analyze tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeInput {
    /// Inline .pxl source to analyze. Mutually exclusive with `path`.
    pub source: Option<String>,

    /// File or directory path to analyze. Mutually exclusive with `source`.
    pub path: Option<String>,

    /// When path is a directory, scan recursively. Default: true.
    #[serde(default = "default_true")]
    pub recursive: bool,
}

fn default_true() -> bool {
    true
}
