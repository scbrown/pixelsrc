//! CSS Variable-aware palette parsing
//!
//! This module provides two-pass palette parsing with CSS variable support:
//! - Pass 1: Collect CSS variable definitions (`--name: value`)
//! - Pass 2: Resolve `var()` references and parse colors
//!
//! # Example
//!
//! ```
//! use pixelsrc::palette_parser::{PaletteParser, ParseMode};
//! use std::collections::HashMap;
//!
//! let mut raw_palette = HashMap::new();
//! raw_palette.insert("--primary".to_string(), "#FF0000".to_string());
//! raw_palette.insert("{red}".to_string(), "var(--primary)".to_string());
//! raw_palette.insert("{light}".to_string(), "var(--missing, #FF6666)".to_string());
//!
//! let parser = PaletteParser::new();
//! let result = parser.parse(&raw_palette, ParseMode::Lenient).unwrap();
//!
//! assert_eq!(result.colors.get("{red}"), Some(&image::Rgba([255, 0, 0, 255])));
//! assert_eq!(result.colors.get("{light}"), Some(&image::Rgba([255, 102, 102, 255])));
//! ```

use crate::color::{parse_color, ColorError};
use crate::variables::{VariableError, VariableRegistry};
use image::Rgba;
use std::collections::HashMap;
use thiserror::Error;

/// Error during palette parsing
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PaletteParseError {
    /// Variable resolution failed
    #[error("variable error for '{token}': {error}")]
    VariableError { token: String, error: VariableError },
    /// Color parsing failed after variable resolution
    #[error("color error for '{token}' (value '{value}'): {error}")]
    ColorError { token: String, value: String, error: ColorError },
}

/// Warning during lenient palette parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteParseWarning {
    pub token: String,
    pub message: String,
}

impl PaletteParseWarning {
    pub fn new(token: impl Into<String>, message: impl Into<String>) -> Self {
        Self { token: token.into(), message: message.into() }
    }
}

/// Parsing mode for palette resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// Stop on first error
    Strict,
    /// Continue with warnings, use magenta for failures
    #[default]
    Lenient,
}

/// Magenta color for fallback (invalid/unresolved colors)
pub const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);

/// Result of parsing a palette
#[derive(Debug, Clone)]
pub struct ParsedPalette {
    /// Token -> resolved RGBA color
    pub colors: HashMap<String, Rgba<u8>>,
    /// Variable registry with all CSS variables
    pub variables: VariableRegistry,
    /// Warnings generated during lenient parsing
    pub warnings: Vec<PaletteParseWarning>,
}

impl ParsedPalette {
    /// Create a new empty parsed palette
    pub fn new() -> Self {
        Self { colors: HashMap::new(), variables: VariableRegistry::new(), warnings: Vec::new() }
    }
}

impl Default for ParsedPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// Two-pass palette parser with CSS variable support
///
/// The parser operates in two passes:
/// 1. **Collection pass**: Extract CSS variable definitions (`--name`) into a registry
/// 2. **Resolution pass**: Resolve `var()` references in color values, then parse colors
///
/// This allows forward references - a color can use `var(--name)` even if `--name`
/// is defined later in the palette.
#[derive(Debug, Clone, Default)]
pub struct PaletteParser {
    /// External variable registry (for parent scope variables)
    external_vars: Option<VariableRegistry>,
}

impl PaletteParser {
    /// Create a new palette parser
    pub fn new() -> Self {
        Self { external_vars: None }
    }

    /// Create a parser with external variables (e.g., from a parent palette)
    ///
    /// External variables are used as fallbacks when resolving var() references.
    /// Local palette variables take precedence over external ones.
    pub fn with_external_vars(external: VariableRegistry) -> Self {
        Self { external_vars: Some(external) }
    }

    /// Parse a raw palette into resolved colors
    ///
    /// # Arguments
    ///
    /// * `raw` - Map of token/variable names to color strings
    /// * `mode` - Strict (fail on first error) or Lenient (continue with warnings)
    ///
    /// # Returns
    ///
    /// In Lenient mode, always returns `Ok` with warnings for any issues.
    /// In Strict mode, returns `Err` on the first error encountered.
    pub fn parse(
        &self,
        raw: &HashMap<String, String>,
        mode: ParseMode,
    ) -> Result<ParsedPalette, PaletteParseError> {
        let mut result = ParsedPalette::new();

        // Pass 1: Collect CSS variable definitions
        self.collect_variables(raw, &mut result.variables);

        // Merge external variables (local takes precedence)
        if let Some(external) = &self.external_vars {
            for (name, value) in external.iter() {
                if !result.variables.contains(name) {
                    result.variables.define(name, value);
                }
            }
        }

        // Pass 2: Resolve colors
        for (token, value) in raw {
            // Skip CSS variable definitions - they're not color tokens
            if token.starts_with("--") {
                continue;
            }

            match self.resolve_color(token, value, &result.variables) {
                Ok(color) => {
                    result.colors.insert(token.clone(), color);
                }
                Err(e) => {
                    if mode == ParseMode::Strict {
                        return Err(e);
                    }
                    // Lenient mode: use magenta and record warning
                    result.warnings.push(PaletteParseWarning::new(token.clone(), e.to_string()));
                    result.colors.insert(token.clone(), MAGENTA);
                }
            }
        }

        Ok(result)
    }

    /// Collect CSS variable definitions from the palette
    ///
    /// CSS variables are entries where the key starts with `--`.
    fn collect_variables(&self, raw: &HashMap<String, String>, registry: &mut VariableRegistry) {
        for (key, value) in raw {
            if key.starts_with("--") {
                registry.define(key, value);
            }
        }
    }

    /// Resolve a single color value
    ///
    /// 1. Resolve any var() references using the registry
    /// 2. Parse the resolved string as a color
    fn resolve_color(
        &self,
        token: &str,
        value: &str,
        registry: &VariableRegistry,
    ) -> Result<Rgba<u8>, PaletteParseError> {
        // Step 1: Resolve var() references
        let resolved = if value.contains("var(") {
            registry.resolve(value).map_err(|e| PaletteParseError::VariableError {
                token: token.to_string(),
                error: e,
            })?
        } else {
            value.to_string()
        };

        // Step 2: Parse as color
        parse_color(&resolved).map_err(|e| PaletteParseError::ColorError {
            token: token.to_string(),
            value: resolved,
            error: e,
        })
    }

    /// Resolve a raw palette to color strings (without parsing to RGBA)
    ///
    /// This is useful when you need the resolved color strings rather than
    /// the parsed RGBA values (e.g., for serialization or display).
    pub fn resolve_to_strings(
        &self,
        raw: &HashMap<String, String>,
        mode: ParseMode,
    ) -> Result<ResolvedPaletteStrings, PaletteParseError> {
        let mut result = ResolvedPaletteStrings {
            colors: HashMap::new(),
            variables: VariableRegistry::new(),
            warnings: Vec::new(),
        };

        // Pass 1: Collect CSS variable definitions
        self.collect_variables(raw, &mut result.variables);

        // Merge external variables
        if let Some(external) = &self.external_vars {
            for (name, value) in external.iter() {
                if !result.variables.contains(name) {
                    result.variables.define(name, value);
                }
            }
        }

        // Pass 2: Resolve var() references
        for (token, value) in raw {
            if token.starts_with("--") {
                continue;
            }

            let resolved = if value.contains("var(") {
                match result.variables.resolve(value) {
                    Ok(r) => r,
                    Err(e) => {
                        if mode == ParseMode::Strict {
                            return Err(PaletteParseError::VariableError {
                                token: token.clone(),
                                error: e,
                            });
                        }
                        result
                            .warnings
                            .push(PaletteParseWarning::new(token.clone(), e.to_string()));
                        // Keep original unresolved value in lenient mode
                        value.clone()
                    }
                }
            } else {
                value.clone()
            };

            result.colors.insert(token.clone(), resolved);
        }

        Ok(result)
    }
}

/// Result of resolving palette to strings (without RGBA parsing)
#[derive(Debug, Clone)]
pub struct ResolvedPaletteStrings {
    /// Token -> resolved color string
    pub colors: HashMap<String, String>,
    /// Variable registry with all CSS variables
    pub variables: VariableRegistry,
    /// Warnings generated during resolution
    pub warnings: Vec<PaletteParseWarning>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_palette(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    // ========== Basic parsing tests ==========

    #[test]
    fn test_parse_simple_colors() {
        let raw = make_palette(&[("{r}", "#FF0000"), ("{g}", "#00FF00"), ("{b}", "#0000FF")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{r}"), Some(&Rgba([255, 0, 0, 255])));
        assert_eq!(result.colors.get("{g}"), Some(&Rgba([0, 255, 0, 255])));
        assert_eq!(result.colors.get("{b}"), Some(&Rgba([0, 0, 255, 255])));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_parse_css_color_formats() {
        let raw = make_palette(&[
            ("{hex}", "#FF0000"),
            ("{rgb}", "rgb(0, 255, 0)"),
            ("{hsl}", "hsl(240, 100%, 50%)"),
            ("{named}", "coral"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{hex}"), Some(&Rgba([255, 0, 0, 255])));
        assert_eq!(result.colors.get("{rgb}"), Some(&Rgba([0, 255, 0, 255])));
        assert_eq!(result.colors.get("{hsl}"), Some(&Rgba([0, 0, 255, 255])));
        assert_eq!(result.colors.get("{named}"), Some(&Rgba([255, 127, 80, 255])));
    }

    // ========== Variable resolution tests ==========

    #[test]
    fn test_simple_var_reference() {
        let raw = make_palette(&[("--primary", "#FF0000"), ("{red}", "var(--primary)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{red}"), Some(&Rgba([255, 0, 0, 255])));
        assert!(result.warnings.is_empty());
        // Variable should not appear in colors
        assert!(!result.colors.contains_key("--primary"));
    }

    #[test]
    fn test_var_with_fallback() {
        let raw = make_palette(&[("{color}", "var(--missing, #00FF00)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&Rgba([0, 255, 0, 255])));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_nested_var_reference() {
        let raw = make_palette(&[
            ("--base", "#FF0000"),
            ("--primary", "var(--base)"),
            ("{color}", "var(--primary)"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn test_forward_reference() {
        // {color} references --primary which is defined after it
        let raw = make_palette(&[("{color}", "var(--primary)"), ("--primary", "#0000FF")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        // Two-pass parsing should handle forward references
        assert_eq!(result.colors.get("{color}"), Some(&Rgba([0, 0, 255, 255])));
    }

    #[test]
    fn test_multiple_var_in_value() {
        let raw = make_palette(&[
            ("--r", "255"),
            ("--g", "128"),
            ("--b", "0"),
            ("{color}", "rgb(var(--r), var(--g), var(--b))"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&Rgba([255, 128, 0, 255])));
    }

    // ========== Lenient mode tests ==========

    #[test]
    fn test_lenient_undefined_var() {
        let raw = make_palette(&[("{color}", "var(--undefined)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        // Should use magenta fallback
        assert_eq!(result.colors.get("{color}"), Some(&MAGENTA));
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("undefined"));
    }

    #[test]
    fn test_lenient_invalid_color() {
        let raw = make_palette(&[("{valid}", "#FF0000"), ("{invalid}", "not-a-color")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{valid}"), Some(&Rgba([255, 0, 0, 255])));
        assert_eq!(result.colors.get("{invalid}"), Some(&MAGENTA));
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_lenient_multiple_errors() {
        let raw = make_palette(&[
            ("{good}", "#FF0000"),
            ("{bad1}", "var(--undefined)"),
            ("{bad2}", "invalid-color"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{good}"), Some(&Rgba([255, 0, 0, 255])));
        assert_eq!(result.colors.get("{bad1}"), Some(&MAGENTA));
        assert_eq!(result.colors.get("{bad2}"), Some(&MAGENTA));
        assert_eq!(result.warnings.len(), 2);
    }

    // ========== Strict mode tests ==========

    #[test]
    fn test_strict_undefined_var() {
        let raw = make_palette(&[("{color}", "var(--undefined)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Strict);

        assert!(result.is_err());
        match result.unwrap_err() {
            PaletteParseError::VariableError { token, .. } => {
                assert_eq!(token, "{color}");
            }
            _ => panic!("Expected VariableError"),
        }
    }

    #[test]
    fn test_strict_invalid_color() {
        let raw = make_palette(&[("{color}", "not-a-color")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Strict);

        assert!(result.is_err());
        match result.unwrap_err() {
            PaletteParseError::ColorError { token, .. } => {
                assert_eq!(token, "{color}");
            }
            _ => panic!("Expected ColorError"),
        }
    }

    #[test]
    fn test_strict_stops_on_first_error() {
        let raw = make_palette(&[("{bad1}", "var(--undefined)"), ("{bad2}", "also-undefined")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Strict);

        assert!(result.is_err());
        // Should have stopped at first error (exact order may vary due to HashMap)
    }

    // ========== External variables tests ==========

    #[test]
    fn test_external_variables() {
        let mut external = VariableRegistry::new();
        external.define("--global", "#00FF00");

        let raw = make_palette(&[("{color}", "var(--global)")]);

        let parser = PaletteParser::with_external_vars(external);
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&Rgba([0, 255, 0, 255])));
    }

    #[test]
    fn test_local_overrides_external() {
        let mut external = VariableRegistry::new();
        external.define("--color", "#FF0000");

        let raw = make_palette(&[
            ("--color", "#00FF00"), // Local override
            ("{token}", "var(--color)"),
        ]);

        let parser = PaletteParser::with_external_vars(external);
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        // Local definition should take precedence
        assert_eq!(result.colors.get("{token}"), Some(&Rgba([0, 255, 0, 255])));
    }

    // ========== resolve_to_strings tests ==========

    #[test]
    fn test_resolve_to_strings() {
        let raw = make_palette(&[
            ("--primary", "#FF0000"),
            ("{color}", "var(--primary)"),
            ("{static}", "#00FF00"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.resolve_to_strings(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&"#FF0000".to_string()));
        assert_eq!(result.colors.get("{static}"), Some(&"#00FF00".to_string()));
    }

    // ========== Error display tests ==========

    #[test]
    fn test_error_display() {
        let var_err = PaletteParseError::VariableError {
            token: "{test}".to_string(),
            error: VariableError::Undefined("--missing".to_string()),
        };
        let display = format!("{}", var_err);
        assert!(display.contains("{test}"));
        assert!(display.contains("--missing"));

        let color_err = PaletteParseError::ColorError {
            token: "{test}".to_string(),
            value: "bad".to_string(),
            error: ColorError::CssParse("invalid".to_string()),
        };
        let display = format!("{}", color_err);
        assert!(display.contains("{test}"));
        assert!(display.contains("bad"));
    }

    // ========== Circular reference tests ==========

    #[test]
    fn test_circular_var_reference_lenient() {
        let raw =
            make_palette(&[("--a", "var(--b)"), ("--b", "var(--a)"), ("{color}", "var(--a)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        // Should detect circular reference and use magenta
        assert_eq!(result.colors.get("{color}"), Some(&MAGENTA));
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_circular_var_reference_strict() {
        let raw =
            make_palette(&[("--a", "var(--b)"), ("--b", "var(--a)"), ("{color}", "var(--a)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Strict);

        assert!(result.is_err());
    }

    // ========== Edge cases ==========

    #[test]
    fn test_empty_palette() {
        let raw = HashMap::new();

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert!(result.colors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_only_variables() {
        let raw = make_palette(&[("--a", "#FF0000"), ("--b", "#00FF00")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        // No color tokens, just variables
        assert!(result.colors.is_empty());
        assert_eq!(result.variables.len(), 2);
    }

    #[test]
    fn test_var_without_dashes_in_name() {
        // var(primary) should be normalized to var(--primary)
        let raw = make_palette(&[("--primary", "#FF0000"), ("{color}", "var(primary)")]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn test_complex_fallback_chain() {
        let raw = make_palette(&[
            ("--deep", "#FF0000"),
            ("{color}", "var(--missing, var(--also-missing, var(--deep)))"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        assert_eq!(result.colors.get("{color}"), Some(&Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn test_preserve_variable_registry() {
        let raw = make_palette(&[
            ("--primary", "#FF0000"),
            ("--secondary", "var(--primary)"),
            ("{color}", "var(--secondary)"),
        ]);

        let parser = PaletteParser::new();
        let result = parser.parse(&raw, ParseMode::Lenient).unwrap();

        // Variables should be available in the result
        assert!(result.variables.contains("--primary"));
        assert!(result.variables.contains("--secondary"));

        // Can use the registry for further resolution
        assert_eq!(result.variables.resolve_var("--primary").unwrap(), "#FF0000");
    }
}
