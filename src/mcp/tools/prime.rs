//! Input schema for pixelsrc_prime tool.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrimeInput {
    /// If true, return the brief (compact) primer instead of the full guide.
    #[serde(default)]
    pub brief: bool,

    /// Optional section to return. One of: format, examples, tips, full.
    /// Ignored when brief is true.
    pub section: Option<String>,
}
