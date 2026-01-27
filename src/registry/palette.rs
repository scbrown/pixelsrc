//! Palette registry for named palettes.

use std::collections::HashMap;
use thiserror::Error;

use crate::color::generate_ramp;
use crate::models::{Palette, PaletteRef, Sprite};
use crate::palette_parser::{PaletteParser, ParseMode};
use crate::palettes;

use super::traits::Registry;

/// Magenta fallback color for missing palettes/tokens
pub const MAGENTA_FALLBACK: &str = "#FF00FF";

/// A resolved palette ready for rendering - maps tokens to color strings.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPalette {
    pub colors: HashMap<String, String>,
    pub source: PaletteSource,
}

/// Indicates where the resolved palette came from.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PaletteSource {
    /// Resolved from a named palette in the registry
    Named(String),
    /// Resolved from a built-in palette (@name syntax)
    Builtin(String),
    /// Inline palette defined in the sprite
    Inline,
    /// Fallback used when palette was not found (lenient mode)
    Fallback,
}

/// Error when resolving a palette in strict mode.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum PaletteError {
    /// Referenced palette name was not found in registry
    #[error("Palette '{0}' not found")]
    NotFound(String),
    /// Referenced built-in palette (@name) was not found
    #[error("Built-in palette '{0}' not found")]
    BuiltinNotFound(String),
}

/// Warning when resolving a palette in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteWarning {
    pub message: String,
}

impl PaletteWarning {
    pub fn not_found(name: &str) -> Self {
        Self { message: format!("Palette '{}' not found", name) }
    }

    pub fn builtin_not_found(name: &str) -> Self {
        Self { message: format!("Built-in palette '{}' not found", name) }
    }
}

/// Resolution result for lenient mode - always succeeds but may have warnings.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientResult {
    pub palette: ResolvedPalette,
    pub warning: Option<PaletteWarning>,
}

/// Resolve CSS variables in palette colors.
///
/// Takes a raw palette colors map and resolves any `var(--name)` references.
/// Returns a new map with resolved color strings.
fn resolve_palette_variables(
    colors: &HashMap<String, String>,
    strict: bool,
) -> (HashMap<String, String>, Vec<PaletteWarning>) {
    let parser = PaletteParser::new();
    let mode = if strict { ParseMode::Strict } else { ParseMode::Lenient };

    match parser.resolve_to_strings(colors, mode) {
        Ok(result) => {
            let warnings: Vec<PaletteWarning> = result
                .warnings
                .into_iter()
                .map(|w| PaletteWarning { message: w.message })
                .collect();
            (result.colors, warnings)
        }
        Err(e) => {
            // In strict mode this shouldn't happen as we'd return early,
            // but in case it does, return the original with a warning
            let mut warnings = Vec::new();
            warnings.push(PaletteWarning { message: e.to_string() });
            (colors.clone(), warnings)
        }
    }
}

/// Registry for named palettes.
#[derive(Debug, Clone, Default)]
pub struct PaletteRegistry {
    palettes: HashMap<String, Palette>,
}

impl PaletteRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self { palettes: HashMap::new() }
    }

    /// Register a palette in the registry.
    ///
    /// If a palette with the same name already exists, it is replaced.
    /// Color ramps are automatically expanded into individual color tokens.
    pub fn register(&mut self, palette: Palette) {
        let expanded = Self::expand_ramps(palette);
        self.palettes.insert(expanded.name.clone(), expanded);
    }

    /// Expand color ramps into individual color tokens.
    ///
    /// For each ramp, generates tokens like:
    /// - `{skin_2}` (darkest shadow)
    /// - `{skin_1}` (shadow)
    /// - `{skin}` (base)
    /// - `{skin+1}` (highlight)
    /// - `{skin+2}` (brightest)
    fn expand_ramps(mut palette: Palette) -> Palette {
        let Some(ramps) = palette.ramps.take() else {
            return palette;
        };

        for (name, ramp) in ramps {
            let shadow = ramp.shadow_shift();
            let highlight = ramp.highlight_shift();

            // Generate the ramp colors
            let ramp_colors = generate_ramp(
                &ramp.base,
                ramp.steps(),
                (
                    shadow.hue.unwrap_or(0.0),
                    shadow.saturation.unwrap_or(0.0),
                    shadow.lightness.unwrap_or(0.0),
                ),
                (
                    highlight.hue.unwrap_or(0.0),
                    highlight.saturation.unwrap_or(0.0),
                    highlight.lightness.unwrap_or(0.0),
                ),
            );

            // Add generated colors to the palette
            if let Ok(colors) = ramp_colors {
                for (suffix, color) in colors {
                    let token = format!("{{{}{}}}", name, suffix);
                    palette.colors.insert(token, color);
                }
            }
            // If generation fails (e.g., invalid base color), silently skip
            // The invalid color will be caught during rendering
        }

        palette
    }

    /// Get a palette by name.
    pub fn get(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(name)
    }

    /// Check if a palette with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.palettes.contains_key(name)
    }

    /// Resolve a sprite's palette reference in strict mode.
    ///
    /// Returns an error if a named palette is not found.
    /// Handles @name syntax for built-in palettes.
    /// Resolves CSS variables (var(--name)) in palette colors.
    pub fn resolve_strict(&self, sprite: &Sprite) -> Result<ResolvedPalette, PaletteError> {
        match &sprite.palette {
            PaletteRef::Named(name) => {
                // Check for built-in palette reference (@name syntax)
                if let Some(builtin_name) = name.strip_prefix('@') {
                    if let Some(palette) = palettes::get_builtin(builtin_name) {
                        // Built-in palettes don't have CSS variables, use as-is
                        Ok(ResolvedPalette {
                            colors: palette.colors.clone(),
                            source: PaletteSource::Builtin(builtin_name.to_string()),
                        })
                    } else {
                        Err(PaletteError::BuiltinNotFound(builtin_name.to_string()))
                    }
                } else if let Some(palette) = self.palettes.get(name) {
                    // Resolve CSS variables in the palette
                    let (resolved_colors, _warnings) =
                        resolve_palette_variables(&palette.colors, true);
                    Ok(ResolvedPalette {
                        colors: resolved_colors,
                        source: PaletteSource::Named(name.clone()),
                    })
                } else {
                    Err(PaletteError::NotFound(name.clone()))
                }
            }
            PaletteRef::Inline(colors) => {
                // Resolve CSS variables in inline palettes too
                let (resolved_colors, _warnings) = resolve_palette_variables(colors, true);
                Ok(ResolvedPalette { colors: resolved_colors, source: PaletteSource::Inline })
            }
        }
    }

    /// Resolve a sprite's palette reference in lenient mode.
    ///
    /// Always returns a palette. If a named palette is not found, returns
    /// an empty fallback palette with a warning.
    /// Handles @name syntax for built-in palettes.
    /// Resolves CSS variables (var(--name)) in palette colors.
    pub fn resolve_lenient(&self, sprite: &Sprite) -> LenientResult {
        match &sprite.palette {
            PaletteRef::Named(name) => {
                // Check for built-in palette reference (@name syntax)
                if let Some(builtin_name) = name.strip_prefix('@') {
                    if let Some(palette) = palettes::get_builtin(builtin_name) {
                        // Built-in palettes don't have CSS variables, use as-is
                        LenientResult {
                            palette: ResolvedPalette {
                                colors: palette.colors.clone(),
                                source: PaletteSource::Builtin(builtin_name.to_string()),
                            },
                            warning: None,
                        }
                    } else {
                        // Fallback: empty palette (tokens will get magenta during rendering)
                        LenientResult {
                            palette: ResolvedPalette {
                                colors: HashMap::new(),
                                source: PaletteSource::Fallback,
                            },
                            warning: Some(PaletteWarning::builtin_not_found(builtin_name)),
                        }
                    }
                } else if let Some(palette) = self.palettes.get(name) {
                    // Resolve CSS variables in the palette
                    let (resolved_colors, var_warnings) =
                        resolve_palette_variables(&palette.colors, false);
                    let warning = if var_warnings.is_empty() {
                        None
                    } else {
                        // Combine multiple variable warnings into one
                        let messages: Vec<String> =
                            var_warnings.into_iter().map(|w| w.message).collect();
                        Some(PaletteWarning { message: messages.join("; ") })
                    };
                    LenientResult {
                        palette: ResolvedPalette {
                            colors: resolved_colors,
                            source: PaletteSource::Named(name.clone()),
                        },
                        warning,
                    }
                } else {
                    // Fallback: empty palette (tokens will get magenta during rendering)
                    LenientResult {
                        palette: ResolvedPalette {
                            colors: HashMap::new(),
                            source: PaletteSource::Fallback,
                        },
                        warning: Some(PaletteWarning::not_found(name)),
                    }
                }
            }
            PaletteRef::Inline(colors) => {
                // Resolve CSS variables in inline palettes too
                let (resolved_colors, var_warnings) = resolve_palette_variables(colors, false);
                let warning = if var_warnings.is_empty() {
                    None
                } else {
                    let messages: Vec<String> =
                        var_warnings.into_iter().map(|w| w.message).collect();
                    Some(PaletteWarning { message: messages.join("; ") })
                };
                LenientResult {
                    palette: ResolvedPalette {
                        colors: resolved_colors,
                        source: PaletteSource::Inline,
                    },
                    warning,
                }
            }
        }
    }

    /// Resolve a sprite's palette reference.
    ///
    /// In strict mode, returns an error for missing palettes.
    /// In lenient mode, returns a fallback with a warning.
    pub fn resolve(&self, sprite: &Sprite, strict: bool) -> Result<LenientResult, PaletteError> {
        if strict {
            self.resolve_strict(sprite).map(|palette| LenientResult { palette, warning: None })
        } else {
            Ok(self.resolve_lenient(sprite))
        }
    }

    /// Get the number of palettes in the registry.
    pub fn len(&self) -> usize {
        self.palettes.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.palettes.is_empty()
    }

    /// Clear all palettes from the registry.
    pub fn clear(&mut self) {
        self.palettes.clear();
    }

    /// Get an iterator over all palette names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.palettes.keys()
    }

    /// Iterate over all palettes in the registry.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Palette)> {
        self.palettes.iter()
    }
}

impl Registry<Palette> for PaletteRegistry {
    fn contains(&self, name: &str) -> bool {
        self.palettes.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(name)
    }

    fn len(&self) -> usize {
        self.palettes.len()
    }

    fn clear(&mut self) {
        self.palettes.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.palettes.keys())
    }
}
