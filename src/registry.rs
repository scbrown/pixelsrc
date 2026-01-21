//! Registry traits and implementations for named items.
//!
//! This module provides:
//! - A unified `Registry` trait for consistent registry interfaces
//! - `PaletteRegistry` for storing named palettes and resolving palette references
//! - `SpriteRegistry` for storing sprites and variants with transform support
//! - `TransformRegistry` for storing user-defined transforms
//! - `CompositionRegistry` for storing layered sprite compositions
//! - `Renderable` enum for unified sprite/composition lookup
//!
//! Most registries support lenient mode (warnings + fallback) and strict mode (errors).

use std::collections::HashMap;
use thiserror::Error;

use crate::color::generate_ramp;
use crate::models::{Composition, Palette, PaletteRef, Sprite, TransformSpec, Variant};
use crate::palette_parser::{PaletteParser, ParseMode};
use crate::palettes;
use crate::transforms::{self, Transform, TransformError};

// ============================================================================
// Registry Trait
// ============================================================================

/// Common trait for registries that store named items.
///
/// This trait provides a unified interface for registries that map string names to values.
/// It defines common operations like checking existence, retrieving items, and counting entries.
///
/// # Type Parameters
///
/// * `V` - The type of value stored in the registry
///
/// # Example
///
/// ```
/// use pixelsrc::registry::{Registry, PaletteRegistry};
/// use pixelsrc::models::Palette;
/// use std::collections::HashMap;
///
/// let mut registry = PaletteRegistry::new();
/// let palette = Palette {
///     name: "mono".to_string(),
///     colors: HashMap::from([("{on}".to_string(), "#FFFFFF".to_string())]),
///     ..Default::default()
/// };
/// registry.register(palette);
///
/// assert!(registry.contains("mono"));
/// assert_eq!(registry.len(), 1);
/// ```
pub trait Registry<V> {
    /// Check if an item with the given name exists in the registry.
    fn contains(&self, name: &str) -> bool;

    /// Get an item by name.
    ///
    /// Returns `None` if no item with the given name exists.
    fn get(&self, name: &str) -> Option<&V>;

    /// Get the number of items in the registry.
    fn len(&self) -> usize;

    /// Check if the registry is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all items from the registry.
    fn clear(&mut self);

    /// Get an iterator over all names in the registry.
    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_>;
}

// ============================================================================
// Palette Registry
// ============================================================================

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

// ============================================================================
// Sprite and Variant Registry
// ============================================================================

/// Error when resolving a sprite or variant.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SpriteError {
    /// Referenced sprite/variant was not found
    #[error("Sprite or variant '{0}' not found")]
    NotFound(String),
    /// Variant references a base sprite that doesn't exist
    #[error("Variant '{variant}' references unknown base sprite '{base}'")]
    BaseNotFound { variant: String, base: String },
    /// Sprite references a source sprite that doesn't exist
    #[error("Sprite '{sprite}' references unknown source sprite '{source_name}'")]
    SourceNotFound { sprite: String, source_name: String },
    /// Circular reference detected during source resolution
    #[error("Circular reference detected for sprite '{sprite}': {}", chain.join(" -> "))]
    CircularReference { sprite: String, chain: Vec<String> },
    /// Error applying transform
    #[error("Transform error for sprite '{sprite}': {message}")]
    TransformError { sprite: String, message: String },
}

/// Warning when resolving a sprite or variant in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteWarning {
    pub message: String,
}

impl SpriteWarning {
    pub fn not_found(name: &str) -> Self {
        Self { message: format!("Sprite or variant '{}' not found", name) }
    }

    pub fn base_not_found(variant: &str, base: &str) -> Self {
        Self { message: format!("Variant '{}' references unknown base sprite '{}'", variant, base) }
    }

    pub fn source_not_found(sprite: &str, source: &str) -> Self {
        Self {
            message: format!("Sprite '{}' references unknown source sprite '{}'", sprite, source),
        }
    }

    pub fn transform_error(sprite: &str, message: &str) -> Self {
        Self { message: format!("Transform error for sprite '{}': {}", sprite, message) }
    }
}

/// A resolved sprite ready for rendering.
///
/// This can be either a direct sprite or a variant expanded to sprite form.
#[derive(Debug, Clone)]
pub struct ResolvedSprite {
    /// The effective name (sprite name or variant name)
    pub name: String,
    /// The size from the base sprite (if any)
    pub size: Option<[u32; 2]>,
    /// The grid data (from base sprite for variants)
    pub grid: Vec<String>,
    /// The merged palette for rendering (base palette + variant overrides)
    pub palette: HashMap<String, String>,
    /// Any warnings generated during resolution
    pub warnings: Vec<SpriteWarning>,
    /// Nine-slice region definition (from base sprite)
    pub nine_slice: Option<crate::models::NineSlice>,
}

// ============================================================================
// Transform Helpers
// ============================================================================

/// Parse a TransformSpec into a Transform.
fn parse_transform_spec(spec: &TransformSpec) -> Result<Transform, TransformError> {
    match spec {
        TransformSpec::String(s) => transforms::parse_transform_str(s),
        TransformSpec::Object { op, params } => {
            // Convert params to serde_json::Value object for parsing
            let mut obj = serde_json::Map::new();
            obj.insert("op".to_string(), serde_json::Value::String(op.clone()));
            for (k, v) in params {
                obj.insert(k.clone(), v.clone());
            }
            transforms::parse_transform_value(&serde_json::Value::Object(obj))
        }
    }
}

/// Apply a single transform to a grid.
///
/// Returns the transformed grid, or an error if the transform cannot be applied.
fn apply_grid_transform(
    grid: &[String],
    transform: &Transform,
) -> Result<Vec<String>, TransformError> {
    match transform {
        // Geometric transforms
        Transform::MirrorH => Ok(mirror_horizontal(grid)),
        Transform::MirrorV => Ok(mirror_vertical(grid)),
        Transform::Rotate { degrees } => rotate_grid(grid, *degrees),

        // Expansion transforms
        Transform::Tile { w, h } => Ok(tile_grid(grid, *w, *h)),
        Transform::Pad { size } => Ok(pad_grid(grid, *size)),
        Transform::Crop { x, y, w, h } => crop_grid(grid, *x, *y, *w, *h),

        // Effects
        Transform::Outline { token, width } => Ok(outline_grid(grid, token.as_deref(), *width)),
        Transform::Shift { x, y } => Ok(shift_grid(grid, *x, *y)),
        Transform::Shadow { x, y, token } => Ok(shadow_grid(grid, *x, *y, token.as_deref())),
        Transform::SelOut { fallback, mapping } => {
            Ok(transforms::apply_selout(grid, fallback.as_deref(), mapping.as_ref()))
        }
        Transform::Scale { x, y } => Ok(transforms::apply_scale(grid, *x, *y)),

        // Dithering - not yet implemented for grid transforms
        Transform::Dither { .. } | Transform::DitherGradient { .. } => {
            Err(TransformError::InvalidParameter {
                op: "dither".to_string(),
                message: "dither transforms are not yet implemented for sprite grids".to_string(),
            })
        }

        // Subpixel - not yet implemented for grid transforms
        Transform::Subpixel { .. } => Err(TransformError::InvalidParameter {
            op: "subpixel".to_string(),
            message: "subpixel transforms are not yet implemented for sprite grids".to_string(),
        }),

        // Animation transforms should not be applied to sprite grids
        Transform::Pingpong { .. }
        | Transform::Reverse
        | Transform::FrameOffset { .. }
        | Transform::Hold { .. } => Err(TransformError::InvalidParameter {
            op: format!("{:?}", transform),
            message: "animation transforms cannot be applied to sprite grids".to_string(),
        }),
    }
}

// ============================================================================
// Grid Transform Implementations
// ============================================================================

/// Mirror grid horizontally (reverse token order in each row).
fn mirror_horizontal(grid: &[String]) -> Vec<String> {
    use crate::tokenizer::tokenize;

    grid.iter()
        .map(|row| {
            let (tokens, _) = tokenize(row);
            tokens.into_iter().rev().collect::<Vec<_>>().join("")
        })
        .collect()
}

/// Mirror grid vertically (reverse row order).
fn mirror_vertical(grid: &[String]) -> Vec<String> {
    grid.iter().rev().cloned().collect()
}

/// Rotate grid by 90, 180, or 270 degrees clockwise.
fn rotate_grid(grid: &[String], degrees: u16) -> Result<Vec<String>, TransformError> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Ok(Vec::new());
    }

    // Parse into 2D token array
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
    let height = parsed.len();
    let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width == 0 {
        return Ok(grid.to_vec());
    }

    // Pad rows to uniform width
    let padded: Vec<Vec<String>> = parsed
        .into_iter()
        .map(|mut row| {
            while row.len() < width {
                row.push("{_}".to_string());
            }
            row
        })
        .collect();

    match degrees {
        90 => {
            // Rotate 90 clockwise: new[col][height-1-row] = old[row][col]
            let mut result = vec![vec![String::new(); height]; width];
            for (row, tokens) in padded.iter().enumerate() {
                for (col, token) in tokens.iter().enumerate() {
                    result[col][height - 1 - row] = token.clone();
                }
            }
            Ok(result.into_iter().map(|row| row.join("")).collect())
        }
        180 => {
            // Rotate 180: reverse both row and column order
            Ok(mirror_vertical(&mirror_horizontal(grid)))
        }
        270 => {
            // Rotate 270 clockwise (= 90 counter-clockwise): new[width-1-col][row] = old[row][col]
            let mut result = vec![vec![String::new(); height]; width];
            for (row, tokens) in padded.iter().enumerate() {
                for (col, token) in tokens.iter().enumerate() {
                    result[width - 1 - col][row] = token.clone();
                }
            }
            Ok(result.into_iter().map(|row| row.join("")).collect())
        }
        _ => Err(TransformError::InvalidRotation(degrees)),
    }
}

/// Tile grid into WxH repetitions.
fn tile_grid(grid: &[String], w: u32, h: u32) -> Vec<String> {
    if grid.is_empty() || w == 0 || h == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();

    // Repeat vertically h times
    for _ in 0..h {
        for row in grid {
            // Repeat each row horizontally w times
            result.push(row.repeat(w as usize));
        }
    }

    result
}

/// Add transparent padding around grid.
fn pad_grid(grid: &[String], size: u32) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() || size == 0 {
        return grid.to_vec();
    }

    // Find the width of the grid (in tokens)
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
    let max_width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    let pad_token = "{_}";
    let horizontal_padding: String =
        std::iter::repeat_n(pad_token, size as usize).collect::<Vec<_>>().join("");
    let full_width_row: String =
        std::iter::repeat_n(pad_token, max_width + 2 * size as usize).collect::<Vec<_>>().join("");

    let mut result = Vec::new();

    // Add top padding rows
    for _ in 0..size {
        result.push(full_width_row.clone());
    }

    // Add padded content rows
    for row in &parsed {
        // Pad row to max_width
        let mut padded_row = row.clone();
        while padded_row.len() < max_width {
            padded_row.push(pad_token.to_string());
        }
        let content = padded_row.join("");
        result.push(format!("{}{}{}", horizontal_padding, content, horizontal_padding));
    }

    // Add bottom padding rows
    for _ in 0..size {
        result.push(full_width_row.clone());
    }

    result
}

/// Extract sub-region from grid.
fn crop_grid(
    grid: &[String],
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Result<Vec<String>, TransformError> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Ok(Vec::new());
    }

    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
    let grid_height = parsed.len();
    let grid_width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    // Validate crop region
    if y as usize >= grid_height || x as usize >= grid_width {
        return Err(TransformError::InvalidCropRegion(format!(
            "crop origin ({}, {}) is outside grid bounds ({}x{})",
            x, y, grid_width, grid_height
        )));
    }

    let mut result = Vec::new();

    for row_idx in y..(y + h) {
        if row_idx as usize >= parsed.len() {
            // Pad with transparent tokens if crop extends beyond grid
            let transparent_row: String =
                std::iter::repeat_n("{_}", w as usize).collect::<Vec<_>>().join("");
            result.push(transparent_row);
        } else {
            let row = &parsed[row_idx as usize];
            let mut cropped_row = Vec::new();
            for col_idx in x..(x + w) {
                if (col_idx as usize) < row.len() {
                    cropped_row.push(row[col_idx as usize].clone());
                } else {
                    cropped_row.push("{_}".to_string());
                }
            }
            result.push(cropped_row.join(""));
        }
    }

    Ok(result)
}

/// Add outline around opaque pixels.
fn outline_grid(grid: &[String], token: Option<&str>, width: u32) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() || width == 0 {
        return grid.to_vec();
    }

    let outline_token = token.unwrap_or("{outline}");
    let transparent = "{_}";

    // Parse grid
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
    let height = parsed.len();
    let width_pixels = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width_pixels == 0 {
        return grid.to_vec();
    }

    // Pad to uniform width
    let padded: Vec<Vec<String>> = parsed
        .into_iter()
        .map(|mut row| {
            while row.len() < width_pixels {
                row.push(transparent.to_string());
            }
            row
        })
        .collect();

    // Create output with extra padding for outline
    let new_width = width_pixels + 2 * width as usize;
    let new_height = height + 2 * width as usize;
    let mut result: Vec<Vec<String>> = vec![vec![transparent.to_string(); new_width]; new_height];

    // Copy original content to center
    for (y, row) in padded.iter().enumerate() {
        for (x, token_val) in row.iter().enumerate() {
            result[y + width as usize][x + width as usize] = token_val.clone();
        }
    }

    // Add outline around opaque pixels
    for y in 0..height {
        for x in 0..width_pixels {
            let token_val = &padded[y][x];
            if token_val != transparent {
                // This is an opaque pixel, add outline around it
                let out_y = y + width as usize;
                let out_x = x + width as usize;

                // Mark outline pixels in a square around the opaque pixel
                for dy in -(width as i32)..=(width as i32) {
                    for dx in -(width as i32)..=(width as i32) {
                        if dy == 0 && dx == 0 {
                            continue; // Skip the center (opaque) pixel
                        }
                        let ny = (out_y as i32 + dy) as usize;
                        let nx = (out_x as i32 + dx) as usize;
                        if result[ny][nx] == transparent {
                            result[ny][nx] = outline_token.to_string();
                        }
                    }
                }
            }
        }
    }

    result.into_iter().map(|row| row.join("")).collect()
}

/// Circular shift (wrap around).
fn shift_grid(grid: &[String], x: i32, y: i32) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    // Parse grid
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
    let height = parsed.len();
    let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width == 0 {
        return grid.to_vec();
    }

    // Pad to uniform width
    let padded: Vec<Vec<String>> = parsed
        .into_iter()
        .map(|mut row| {
            while row.len() < width {
                row.push("{_}".to_string());
            }
            row
        })
        .collect();

    // Calculate effective shift (handle negative and wraparound)
    let shift_y = ((y % height as i32) + height as i32) as usize % height;
    let shift_x = ((x % width as i32) + width as i32) as usize % width;

    // Shift vertically
    let mut shifted_rows: Vec<Vec<String>> = Vec::with_capacity(height);
    for i in 0..height {
        let src_y = (i + height - shift_y) % height;
        shifted_rows.push(padded[src_y].clone());
    }

    // Shift horizontally
    let result: Vec<String> = shifted_rows
        .into_iter()
        .map(|row| {
            let mut shifted_row: Vec<String> = Vec::with_capacity(width);
            for i in 0..width {
                let src_x = (i + width - shift_x) % width;
                shifted_row.push(row[src_x].clone());
            }
            shifted_row.join("")
        })
        .collect();

    result
}

/// Add drop shadow.
fn shadow_grid(grid: &[String], x: i32, y: i32, token: Option<&str>) -> Vec<String> {
    use crate::tokenizer::tokenize;

    if grid.is_empty() {
        return Vec::new();
    }

    let shadow_token = token.unwrap_or("{shadow}");
    let transparent = "{_}";

    // Parse grid
    let parsed: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();
    let height = parsed.len();
    let width = parsed.iter().map(|r| r.len()).max().unwrap_or(0);

    if width == 0 {
        return grid.to_vec();
    }

    // Pad to uniform width
    let padded: Vec<Vec<String>> = parsed
        .into_iter()
        .map(|mut row| {
            while row.len() < width {
                row.push(transparent.to_string());
            }
            row
        })
        .collect();

    // Calculate expanded size to accommodate shadow
    let extra_left = if x < 0 { (-x) as usize } else { 0 };
    let extra_right = if x > 0 { x as usize } else { 0 };
    let extra_top = if y < 0 { (-y) as usize } else { 0 };
    let extra_bottom = if y > 0 { y as usize } else { 0 };

    let new_width = width + extra_left + extra_right;
    let new_height = height + extra_top + extra_bottom;

    // Create output grid filled with transparent
    let mut result: Vec<Vec<String>> = vec![vec![transparent.to_string(); new_width]; new_height];

    // Draw shadow first (behind the original)
    for (row_y, row) in padded.iter().enumerate() {
        for (col_x, token_val) in row.iter().enumerate() {
            if token_val != transparent {
                let shadow_y = (row_y as i32 + extra_top as i32 + y) as usize;
                let shadow_x = (col_x as i32 + extra_left as i32 + x) as usize;
                if shadow_y < new_height && shadow_x < new_width {
                    result[shadow_y][shadow_x] = shadow_token.to_string();
                }
            }
        }
    }

    // Draw original on top
    for (row_y, row) in padded.iter().enumerate() {
        for (col_x, token_val) in row.iter().enumerate() {
            let out_y = row_y + extra_top;
            let out_x = col_x + extra_left;
            if token_val != transparent {
                result[out_y][out_x] = token_val.clone();
            }
        }
    }

    result.into_iter().map(|row| row.join("")).collect()
}

/// Registry for sprites and variants.
///
/// Handles resolution of sprite names to renderable sprites, including
/// expanding variants to their effective sprite representation.
#[derive(Debug, Clone, Default)]
pub struct SpriteRegistry {
    sprites: HashMap<String, Sprite>,
    variants: HashMap<String, Variant>,
}

impl SpriteRegistry {
    /// Create a new empty sprite registry.
    pub fn new() -> Self {
        Self { sprites: HashMap::new(), variants: HashMap::new() }
    }

    /// Register a sprite.
    pub fn register_sprite(&mut self, sprite: Sprite) {
        self.sprites.insert(sprite.name.clone(), sprite);
    }

    /// Register a variant.
    pub fn register_variant(&mut self, variant: Variant) {
        self.variants.insert(variant.name.clone(), variant);
    }

    /// Get a sprite by name (does not resolve variants).
    pub fn get_sprite(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }

    /// Get a variant by name.
    pub fn get_variant(&self, name: &str) -> Option<&Variant> {
        self.variants.get(name)
    }

    /// Check if a name refers to a sprite or variant.
    pub fn contains(&self, name: &str) -> bool {
        self.sprites.contains_key(name) || self.variants.contains_key(name)
    }

    /// Resolve a name to a sprite-like structure, expanding variants.
    ///
    /// In strict mode, returns an error if the name or base is not found.
    /// In lenient mode, returns a fallback result with warnings.
    ///
    /// The returned ResolvedSprite contains the effective grid and merged palette.
    pub fn resolve(
        &self,
        name: &str,
        palette_registry: &PaletteRegistry,
        strict: bool,
    ) -> Result<ResolvedSprite, SpriteError> {
        // First, check if it's a direct sprite
        if let Some(sprite) = self.sprites.get(name) {
            return self.resolve_sprite(sprite, palette_registry, strict);
        }

        // Check if it's a variant
        if let Some(variant) = self.variants.get(name) {
            return self.resolve_variant(variant, palette_registry, strict);
        }

        // Not found
        if strict {
            Err(SpriteError::NotFound(name.to_string()))
        } else {
            Ok(ResolvedSprite {
                name: name.to_string(),
                size: None,
                grid: vec![],
                palette: HashMap::new(),
                warnings: vec![SpriteWarning::not_found(name)],
                nine_slice: None,
            })
        }
    }

    /// Resolve a direct sprite to a ResolvedSprite.
    fn resolve_sprite(
        &self,
        sprite: &Sprite,
        palette_registry: &PaletteRegistry,
        strict: bool,
    ) -> Result<ResolvedSprite, SpriteError> {
        // Use the internal resolver with cycle detection
        self.resolve_sprite_internal(sprite, palette_registry, strict, &mut Vec::new())
    }

    /// Internal sprite resolution with cycle detection.
    ///
    /// The `visited` parameter tracks sprites in the current resolution chain
    /// to detect circular references.
    fn resolve_sprite_internal(
        &self,
        sprite: &Sprite,
        palette_registry: &PaletteRegistry,
        strict: bool,
        visited: &mut Vec<String>,
    ) -> Result<ResolvedSprite, SpriteError> {
        let mut warnings = Vec::new();

        // Check for circular reference
        if visited.contains(&sprite.name) {
            visited.push(sprite.name.clone());
            if strict {
                return Err(SpriteError::CircularReference {
                    sprite: sprite.name.clone(),
                    chain: visited.clone(),
                });
            } else {
                return Ok(ResolvedSprite {
                    name: sprite.name.clone(),
                    size: None,
                    grid: vec![],
                    palette: HashMap::new(),
                    warnings: vec![SpriteWarning {
                        message: format!("Circular reference detected: {}", visited.join(" -> ")),
                    }],
                    nine_slice: None,
                });
            }
        }

        // Mark as visited
        visited.push(sprite.name.clone());

        // Determine the grid: either from source reference or direct grid
        let base_grid = if let Some(source_name) = &sprite.source {
            // Resolve the source sprite
            match self.sprites.get(source_name) {
                Some(source_sprite) => {
                    // Recursively resolve the source sprite
                    let source_resolved = self.resolve_sprite_internal(
                        source_sprite,
                        palette_registry,
                        strict,
                        visited,
                    )?;
                    warnings.extend(source_resolved.warnings);
                    source_resolved.grid
                }
                None => {
                    if strict {
                        return Err(SpriteError::SourceNotFound {
                            sprite: sprite.name.clone(),
                            source_name: source_name.clone(),
                        });
                    } else {
                        warnings.push(SpriteWarning::source_not_found(&sprite.name, source_name));
                        // Return empty grid on source not found in lenient mode
                        Vec::new()
                    }
                }
            }
        } else {
            // Use the sprite's own grid
            sprite.grid.clone()
        };

        // Apply transforms if any
        let grid = if let Some(transform_specs) = &sprite.transform {
            match self.apply_transforms_to_grid(&base_grid, transform_specs, &sprite.name, strict) {
                Ok((transformed, transform_warnings)) => {
                    warnings.extend(transform_warnings);
                    transformed
                }
                Err(e) => {
                    if strict {
                        return Err(e);
                    } else {
                        warnings.push(SpriteWarning::transform_error(&sprite.name, &e.to_string()));
                        base_grid
                    }
                }
            }
        } else {
            base_grid
        };

        // Resolve the sprite's palette
        let palette = match palette_registry.resolve(sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning { message: warning.message });
                }
                result.palette.colors
            }
            Err(e) => {
                // In strict mode, this would have returned an error from resolve()
                // In lenient mode, we got a fallback. Map the error for strict.
                if strict {
                    // The resolve() function already handles strict vs lenient
                    return Err(SpriteError::NotFound(format!("palette error: {}", e)));
                }
                HashMap::new()
            }
        };

        Ok(ResolvedSprite {
            name: sprite.name.clone(),
            size: sprite.size,
            grid,
            palette,
            warnings,
            nine_slice: sprite.nine_slice.clone(),
        })
    }

    /// Apply a list of transforms to a grid.
    ///
    /// Returns the transformed grid and any warnings generated.
    fn apply_transforms_to_grid(
        &self,
        grid: &[String],
        transform_specs: &[TransformSpec],
        sprite_name: &str,
        strict: bool,
    ) -> Result<(Vec<String>, Vec<SpriteWarning>), SpriteError> {
        let mut warnings = Vec::new();
        let mut current_grid = grid.to_vec();

        for spec in transform_specs {
            // Parse the TransformSpec into a Transform
            let transform = match parse_transform_spec(spec) {
                Ok(t) => t,
                Err(e) => {
                    if strict {
                        return Err(SpriteError::TransformError {
                            sprite: sprite_name.to_string(),
                            message: e.to_string(),
                        });
                    } else {
                        warnings.push(SpriteWarning::transform_error(sprite_name, &e.to_string()));
                        continue;
                    }
                }
            };

            // Skip animation-only transforms for sprites
            if transforms::is_animation_transform(&transform) {
                warnings.push(SpriteWarning::transform_error(
                    sprite_name,
                    &format!("{:?} is an animation-only transform", transform),
                ));
                continue;
            }

            // Apply the transform
            match apply_grid_transform(&current_grid, &transform) {
                Ok(new_grid) => {
                    current_grid = new_grid;
                }
                Err(e) => {
                    if strict {
                        return Err(SpriteError::TransformError {
                            sprite: sprite_name.to_string(),
                            message: e.to_string(),
                        });
                    } else {
                        warnings.push(SpriteWarning::transform_error(sprite_name, &e.to_string()));
                    }
                }
            }
        }

        Ok((current_grid, warnings))
    }

    /// Resolve a variant to a ResolvedSprite by expanding from its base.
    fn resolve_variant(
        &self,
        variant: &Variant,
        palette_registry: &PaletteRegistry,
        strict: bool,
    ) -> Result<ResolvedSprite, SpriteError> {
        // Look up the base sprite
        let base_sprite = match self.sprites.get(&variant.base) {
            Some(sprite) => sprite,
            None => {
                if strict {
                    return Err(SpriteError::BaseNotFound {
                        variant: variant.name.clone(),
                        base: variant.base.clone(),
                    });
                } else {
                    return Ok(ResolvedSprite {
                        name: variant.name.clone(),
                        size: None,
                        grid: vec![],
                        palette: HashMap::new(),
                        warnings: vec![SpriteWarning::base_not_found(&variant.name, &variant.base)],
                        nine_slice: None,
                    });
                }
            }
        };

        let mut warnings = Vec::new();

        // Resolve the base sprite's palette
        let base_palette = match palette_registry.resolve(base_sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning { message: warning.message });
                }
                result.palette.colors
            }
            Err(e) => {
                if strict {
                    return Err(SpriteError::NotFound(format!("base palette error: {}", e)));
                }
                HashMap::new()
            }
        };

        // Merge palettes: start with base, override with variant's palette
        let mut merged_palette = base_palette;
        for (token, color) in &variant.palette {
            merged_palette.insert(token.clone(), color.clone());
        }

        // Start with base grid
        let base_grid = base_sprite.grid.clone();

        // Apply transforms if any
        let grid = if let Some(transform_specs) = &variant.transform {
            match self.apply_transforms_to_grid(&base_grid, transform_specs, &variant.name, strict)
            {
                Ok((transformed, transform_warnings)) => {
                    warnings.extend(transform_warnings);
                    transformed
                }
                Err(e) => {
                    if strict {
                        return Err(e);
                    } else {
                        warnings
                            .push(SpriteWarning::transform_error(&variant.name, &e.to_string()));
                        base_grid
                    }
                }
            }
        } else {
            base_grid
        };

        Ok(ResolvedSprite {
            name: variant.name.clone(),
            size: base_sprite.size,
            grid,
            palette: merged_palette,
            warnings,
            nine_slice: base_sprite.nine_slice.clone(),
        })
    }

    /// Get all sprite and variant names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.sprites.keys().chain(self.variants.keys())
    }

    /// Get the total number of sprites and variants in the registry.
    pub fn len(&self) -> usize {
        self.sprites.len() + self.variants.len()
    }

    /// Check if the registry is empty (no sprites or variants).
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty() && self.variants.is_empty()
    }

    /// Clear all sprites and variants from the registry.
    pub fn clear(&mut self) {
        self.sprites.clear();
        self.variants.clear();
    }

    /// Get the number of sprites (excluding variants).
    pub fn sprite_count(&self) -> usize {
        self.sprites.len()
    }

    /// Get the number of variants (excluding sprites).
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    /// Iterate over all sprites in the registry.
    pub fn sprites(&self) -> impl Iterator<Item = (&String, &Sprite)> {
        self.sprites.iter()
    }

    /// Iterate over all variants in the registry.
    pub fn variants(&self) -> impl Iterator<Item = (&String, &Variant)> {
        self.variants.iter()
    }
}

impl Registry<Sprite> for SpriteRegistry {
    fn contains(&self, name: &str) -> bool {
        self.sprites.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }

    fn len(&self) -> usize {
        self.sprites.len()
    }

    fn clear(&mut self) {
        self.sprites.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.sprites.keys())
    }
}

impl Registry<Variant> for SpriteRegistry {
    fn contains(&self, name: &str) -> bool {
        self.variants.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Variant> {
        self.variants.get(name)
    }

    fn len(&self) -> usize {
        self.variants.len()
    }

    fn clear(&mut self) {
        self.variants.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.variants.keys())
    }
}

// ============================================================================
// Transform Registry (TRF-10)
// ============================================================================

use crate::models::TransformDef;

/// Registry for user-defined transforms.
///
/// Stores TransformDef objects that can be referenced by name in transform arrays.
/// Supports parameterized transforms, keyframe animations, and transform cycling.
#[derive(Debug, Clone, Default)]
pub struct TransformRegistry {
    transforms: HashMap<String, TransformDef>,
}

impl TransformRegistry {
    /// Create a new empty transform registry.
    pub fn new() -> Self {
        Self { transforms: HashMap::new() }
    }

    /// Register a user-defined transform.
    pub fn register(&mut self, transform: TransformDef) {
        self.transforms.insert(transform.name.clone(), transform);
    }

    /// Get a transform definition by name.
    pub fn get(&self, name: &str) -> Option<&TransformDef> {
        self.transforms.get(name)
    }

    /// Check if a transform with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.transforms.contains_key(name)
    }

    /// Get the number of registered transforms.
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Clear all transforms from the registry.
    pub fn clear(&mut self) {
        self.transforms.clear();
    }

    /// Iterate over all transforms in the registry.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &TransformDef)> {
        self.transforms.iter()
    }

    /// Expand a user-defined transform for a specific frame.
    ///
    /// If the transform is a simple ops-only transform, returns the ops directly.
    /// For keyframe animations, generates the appropriate transforms for the given frame.
    /// For cycling transforms, returns the transforms for the current cycle position.
    ///
    /// # Arguments
    /// * `name` - The name of the user-defined transform
    /// * `params` - Parameter values for parameterized transforms
    /// * `frame` - Current frame number (for keyframe animations)
    /// * `total_frames` - Total frames in the animation
    pub fn expand(
        &self,
        name: &str,
        params: &HashMap<String, f64>,
        frame: u32,
        total_frames: u32,
    ) -> Result<Vec<Transform>, TransformError> {
        let transform_def = self
            .transforms
            .get(name)
            .ok_or_else(|| TransformError::UnknownOperation(name.to_string()))?;

        transforms::generate_frame_transforms(transform_def, frame, total_frames, params)
    }

    /// Expand a simple (non-animated) user-defined transform.
    ///
    /// For simple ops-only transforms, returns the ops.
    /// For keyframe animations, returns transforms for frame 0.
    pub fn expand_simple(
        &self,
        name: &str,
        params: &HashMap<String, f64>,
    ) -> Result<Vec<Transform>, TransformError> {
        self.expand(name, params, 0, 1)
    }
}

impl Registry<TransformDef> for TransformRegistry {
    fn contains(&self, name: &str) -> bool {
        self.transforms.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&TransformDef> {
        self.transforms.get(name)
    }

    fn len(&self) -> usize {
        self.transforms.len()
    }

    fn clear(&mut self) {
        self.transforms.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.transforms.keys())
    }
}

// ============================================================================
// Composition Registry (NC-1)
// ============================================================================

/// Registry for named compositions.
///
/// Stores Composition objects that can be looked up by name.
/// Compositions define layered sprite arrangements for complex visuals.
#[derive(Debug, Clone, Default)]
pub struct CompositionRegistry {
    compositions: HashMap<String, Composition>,
}

impl CompositionRegistry {
    /// Create a new empty composition registry.
    pub fn new() -> Self {
        Self { compositions: HashMap::new() }
    }

    /// Register a composition in the registry.
    ///
    /// If a composition with the same name already exists, it is replaced.
    pub fn register(&mut self, composition: Composition) {
        self.compositions.insert(composition.name.clone(), composition);
    }

    /// Get a composition by name.
    pub fn get(&self, name: &str) -> Option<&Composition> {
        self.compositions.get(name)
    }

    /// Check if a composition with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.compositions.contains_key(name)
    }

    /// Get the number of compositions in the registry.
    pub fn len(&self) -> usize {
        self.compositions.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.compositions.is_empty()
    }

    /// Clear all compositions from the registry.
    pub fn clear(&mut self) {
        self.compositions.clear();
    }

    /// Get an iterator over all composition names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.compositions.keys()
    }

    /// Iterate over all compositions in the registry.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Composition)> {
        self.compositions.iter()
    }
}

impl Registry<Composition> for CompositionRegistry {
    fn contains(&self, name: &str) -> bool {
        self.compositions.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Composition> {
        self.compositions.get(name)
    }

    fn len(&self) -> usize {
        self.compositions.len()
    }

    fn clear(&mut self) {
        self.compositions.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.compositions.keys())
    }
}

// ============================================================================
// Unified Renderable Lookup (NC-1)
// ============================================================================

/// A renderable entity that can be either a sprite or a composition.
///
/// This enum provides unified lookup across sprite and composition registries,
/// allowing rendering code to handle both types through a single interface.
#[derive(Debug, Clone)]
pub enum Renderable<'a> {
    /// A sprite (direct or resolved from variant)
    Sprite(&'a Sprite),
    /// A composition of layered sprites
    Composition(&'a Composition),
}

impl<'a> Renderable<'a> {
    /// Get the name of the renderable entity.
    pub fn name(&self) -> &str {
        match self {
            Renderable::Sprite(sprite) => &sprite.name,
            Renderable::Composition(composition) => &composition.name,
        }
    }

    /// Check if this is a sprite.
    pub fn is_sprite(&self) -> bool {
        matches!(self, Renderable::Sprite(_))
    }

    /// Check if this is a composition.
    pub fn is_composition(&self) -> bool {
        matches!(self, Renderable::Composition(_))
    }

    /// Get the sprite if this is a Sprite variant.
    pub fn as_sprite(&self) -> Option<&'a Sprite> {
        match self {
            Renderable::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    /// Get the composition if this is a Composition variant.
    pub fn as_composition(&self) -> Option<&'a Composition> {
        match self {
            Renderable::Composition(composition) => Some(composition),
            _ => None,
        }
    }
}

/// Look up a renderable by name across sprite and composition registries.
///
/// Searches sprites first, then compositions. Returns the first match found.
/// This enables unified rendering where a name can refer to either a sprite
/// or a composition without the caller needing to know which.
pub fn lookup_renderable<'a>(
    name: &str,
    sprite_registry: &'a SpriteRegistry,
    composition_registry: &'a CompositionRegistry,
) -> Option<Renderable<'a>> {
    // Check sprites first (including variants via the direct sprite lookup)
    if let Some(sprite) = sprite_registry.get_sprite(name) {
        return Some(Renderable::Sprite(sprite));
    }

    // Then check compositions
    if let Some(composition) = composition_registry.get(name) {
        return Some(Renderable::Composition(composition));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mono_palette() -> Palette {
        Palette {
            name: "mono".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{on}".to_string(), "#FFFFFF".to_string()),
                ("{off}".to_string(), "#000000".to_string()),
            ]),
            ..Default::default()
        }
    }

    fn checker_sprite_named() -> Sprite {
        Sprite {
            name: "checker".to_string(),
            size: None,
            palette: PaletteRef::Named("mono".to_string()),
            grid: vec!["{on}{off}{on}{off}".to_string(), "{off}{on}{off}{on}".to_string()],
            metadata: None,
            ..Default::default()
        }
    }

    fn dot_sprite_inline() -> Sprite {
        Sprite {
            name: "dot".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#FF0000".to_string()),
            ])),
            grid: vec!["{x}".to_string()],
            metadata: None,
            ..Default::default()
        }
    }

    fn bad_ref_sprite() -> Sprite {
        Sprite {
            name: "bad_ref".to_string(),
            size: None,
            palette: PaletteRef::Named("nonexistent".to_string()),
            grid: vec!["{x}{x}".to_string()],
            metadata: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_registry_new_is_empty() {
        let registry = PaletteRegistry::new();
        assert!(!registry.contains("anything"));
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = PaletteRegistry::new();
        let palette = mono_palette();
        registry.register(palette.clone());

        assert!(registry.contains("mono"));
        let retrieved = registry.get("mono").unwrap();
        assert_eq!(retrieved.name, "mono");
        assert_eq!(retrieved.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
    }

    #[test]
    fn test_register_overwrites() {
        let mut registry = PaletteRegistry::new();
        let palette1 = Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{a}".to_string(), "#FF0000".to_string())]),
            ..Default::default()
        };
        let palette2 = Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{b}".to_string(), "#00FF00".to_string())]),
            ..Default::default()
        };

        registry.register(palette1);
        registry.register(palette2);

        let retrieved = registry.get("test").unwrap();
        assert!(retrieved.colors.contains_key("{b}"));
        assert!(!retrieved.colors.contains_key("{a}"));
    }

    #[test]
    fn test_resolve_strict_named_found() {
        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        let sprite = checker_sprite_named();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Named("mono".to_string()));
        assert_eq!(result.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
    }

    #[test]
    fn test_resolve_strict_named_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve_strict(&sprite);
        assert_eq!(result, Err(PaletteError::NotFound("nonexistent".to_string())));
    }

    #[test]
    fn test_resolve_strict_inline() {
        let registry = PaletteRegistry::new();
        let sprite = dot_sprite_inline();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Inline);
        assert_eq!(result.colors.get("{x}"), Some(&"#FF0000".to_string()));
    }

    #[test]
    fn test_resolve_lenient_named_found() {
        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        let sprite = checker_sprite_named();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(result.palette.source, PaletteSource::Named("mono".to_string()));
    }

    #[test]
    fn test_resolve_lenient_named_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert!(result.warning.as_ref().unwrap().message.contains("nonexistent"));
        assert_eq!(result.palette.source, PaletteSource::Fallback);
        assert!(result.palette.colors.is_empty());
    }

    #[test]
    fn test_resolve_lenient_inline() {
        let registry = PaletteRegistry::new();
        let sprite = dot_sprite_inline();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(result.palette.source, PaletteSource::Inline);
    }

    #[test]
    fn test_resolve_combined_strict() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve(&sprite, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_combined_lenient() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve(&sprite, false).unwrap();
        assert!(result.warning.is_some());
        assert_eq!(result.palette.source, PaletteSource::Fallback);
    }

    // Tests matching fixture: tests/fixtures/valid/named_palette.jsonl
    #[test]
    fn test_fixture_named_palette() {
        let mut registry = PaletteRegistry::new();

        // {"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF", "{off}": "#000000"}}
        registry.register(mono_palette());

        // {"type": "sprite", "name": "checker", "palette": "mono", "grid": [...]}
        let sprite = checker_sprite_named();
        let result = registry.resolve_strict(&sprite).unwrap();

        assert_eq!(result.source, PaletteSource::Named("mono".to_string()));
        assert_eq!(result.colors.len(), 3);
        assert_eq!(result.colors.get("{_}"), Some(&"#00000000".to_string()));
        assert_eq!(result.colors.get("{on}"), Some(&"#FFFFFF".to_string()));
        assert_eq!(result.colors.get("{off}"), Some(&"#000000".to_string()));
    }

    // Tests matching fixture: tests/fixtures/invalid/unknown_palette_ref.jsonl
    #[test]
    fn test_fixture_unknown_palette_ref_strict() {
        let registry = PaletteRegistry::new();

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "grid": ["{x}{x}"]}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_strict(&sprite);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PaletteError::NotFound("nonexistent".to_string()));
    }

    #[test]
    fn test_fixture_unknown_palette_ref_lenient() {
        let registry = PaletteRegistry::new();

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "grid": ["{x}{x}"]}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_lenient(&sprite);

        assert!(result.warning.is_some());
        assert_eq!(result.warning.unwrap().message, "Palette 'nonexistent' not found");
        assert_eq!(result.palette.source, PaletteSource::Fallback);
    }

    // ============================================================
    // Built-in palette resolution tests (@name syntax)
    // ============================================================

    fn builtin_gameboy_sprite() -> Sprite {
        Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("@gameboy".to_string()),
            grid: vec!["{lightest}{dark}".to_string()],
            metadata: None,
            ..Default::default()
        }
    }

    fn builtin_nonexistent_sprite() -> Sprite {
        Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("@nonexistent".to_string()),
            grid: vec!["{x}{x}".to_string()],
            metadata: None,
            ..Default::default()
        }
    }

    // ========== SpriteRegistry Tests ==========

    fn hero_sprite() -> Sprite {
        Sprite {
            name: "hero".to_string(),
            size: Some([4, 4]),
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{skin}".to_string(), "#FFCC99".to_string()),
                ("{hair}".to_string(), "#333333".to_string()),
            ])),
            grid: vec![
                "{_}{hair}{hair}{_}".to_string(),
                "{hair}{skin}{skin}{hair}".to_string(),
                "{_}{skin}{skin}{_}".to_string(),
                "{_}{skin}{skin}{_}".to_string(),
            ],
            metadata: None,
            ..Default::default()
        }
    }

    fn hero_red_variant() -> Variant {
        Variant {
            name: "hero_red".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([("{skin}".to_string(), "#FF6666".to_string())]),
            ..Default::default()
        }
    }

    fn hero_alt_variant() -> Variant {
        Variant {
            name: "hero_alt".to_string(),
            base: "hero".to_string(),
            palette: HashMap::from([
                ("{skin}".to_string(), "#66FF66".to_string()),
                ("{hair}".to_string(), "#FFFF00".to_string()),
            ]),
            ..Default::default()
        }
    }

    fn bad_base_variant() -> Variant {
        Variant {
            name: "ghost".to_string(),
            base: "nonexistent".to_string(),
            palette: HashMap::new(),
            ..Default::default()
        }
    }

    #[test]
    fn test_resolve_strict_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Builtin("gameboy".to_string()));
        assert_eq!(result.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
        assert_eq!(result.colors.get("{dark}"), Some(&"#306230".to_string()));
    }

    #[test]
    fn test_resolve_strict_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_strict(&sprite);
        assert_eq!(result, Err(PaletteError::BuiltinNotFound("nonexistent".to_string())));
    }

    #[test]
    fn test_resolve_lenient_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(result.palette.source, PaletteSource::Builtin("gameboy".to_string()));
        assert_eq!(result.palette.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
    }

    #[test]
    fn test_resolve_lenient_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert_eq!(result.warning.unwrap().message, "Built-in palette 'nonexistent' not found");
        assert_eq!(result.palette.source, PaletteSource::Fallback);
        assert!(result.palette.colors.is_empty());
    }

    #[test]
    fn test_resolve_combined_builtin_strict() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve(&sprite, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PaletteError::BuiltinNotFound("nonexistent".to_string()));
    }

    #[test]
    fn test_resolve_combined_builtin_lenient() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve(&sprite, false).unwrap();
        assert!(result.warning.is_some());
        assert_eq!(result.palette.source, PaletteSource::Fallback);
    }

    // Test fixture matching plan doc:
    // {"type": "sprite", "name": "test", "palette": "@gameboy", "grid": ["{lightest}{dark}"]}
    #[test]
    fn test_fixture_builtin_palette() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_strict(&sprite).unwrap();
        assert_eq!(result.source, PaletteSource::Builtin("gameboy".to_string()));
        // Verify correct gameboy colors
        assert_eq!(result.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
        assert_eq!(result.colors.get("{light}"), Some(&"#8BAC0F".to_string()));
        assert_eq!(result.colors.get("{dark}"), Some(&"#306230".to_string()));
        assert_eq!(result.colors.get("{darkest}"), Some(&"#0F380F".to_string()));
    }

    #[test]
    fn test_all_builtins_resolvable() {
        let registry = PaletteRegistry::new();
        let builtin_names = ["gameboy", "nes", "pico8", "grayscale", "1bit"];

        for name in builtin_names {
            let sprite = Sprite {
                name: "test".to_string(),
                size: None,
                palette: PaletteRef::Named(format!("@{}", name)),
                grid: vec!["{_}".to_string()],
                metadata: None,
                ..Default::default()
            };
            let result = registry.resolve_strict(&sprite);
            assert!(result.is_ok(), "Built-in palette @{} should be resolvable", name);
            assert_eq!(result.unwrap().source, PaletteSource::Builtin(name.to_string()));
        }
    }

    #[test]
    fn test_sprite_registry_new() {
        let registry = SpriteRegistry::new();
        assert!(!registry.contains("anything"));
    }

    #[test]
    fn test_sprite_registry_register_sprite() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());

        assert!(registry.contains("hero"));
        assert!(registry.get_sprite("hero").is_some());
        assert!(registry.get_variant("hero").is_none());
    }

    #[test]
    fn test_sprite_registry_register_variant() {
        let mut registry = SpriteRegistry::new();
        registry.register_variant(hero_red_variant());

        assert!(registry.contains("hero_red"));
        assert!(registry.get_sprite("hero_red").is_none());
        assert!(registry.get_variant("hero_red").is_some());
    }

    #[test]
    fn test_sprite_registry_resolve_sprite() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        assert_eq!(result.name, "hero");
        assert_eq!(result.size, Some([4, 4]));
        assert_eq!(result.grid.len(), 4);
        assert_eq!(result.palette.get("{skin}"), Some(&"#FFCC99".to_string()));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_sprite_registry_resolve_variant_single_override() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_red_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();
        assert_eq!(result.name, "hero_red");
        assert_eq!(result.size, Some([4, 4])); // Inherited from base
        assert_eq!(result.grid.len(), 4); // Copied from base

        // skin should be overridden
        assert_eq!(result.palette.get("{skin}"), Some(&"#FF6666".to_string()));
        // hair and _ should be inherited from base
        assert_eq!(result.palette.get("{hair}"), Some(&"#333333".to_string()));
        assert_eq!(result.palette.get("{_}"), Some(&"#00000000".to_string()));

        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_sprite_registry_resolve_variant_multiple_overrides() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_alt_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("hero_alt", &palette_registry, false).unwrap();
        assert_eq!(result.name, "hero_alt");

        // Both skin and hair should be overridden
        assert_eq!(result.palette.get("{skin}"), Some(&"#66FF66".to_string()));
        assert_eq!(result.palette.get("{hair}"), Some(&"#FFFF00".to_string()));
        // _ should be inherited from base
        assert_eq!(result.palette.get("{_}"), Some(&"#00000000".to_string()));
    }

    #[test]
    fn test_sprite_registry_variant_unknown_base_strict() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_variant(bad_base_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("ghost", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::BaseNotFound { variant, base } => {
                assert_eq!(variant, "ghost");
                assert_eq!(base, "nonexistent");
            }
            _ => panic!("Expected BaseNotFound error"),
        }
    }

    #[test]
    fn test_sprite_registry_variant_unknown_base_lenient() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_variant(bad_base_variant());

        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("ghost", &palette_registry, false).unwrap();
        assert_eq!(result.name, "ghost");
        assert!(result.grid.is_empty());
        assert!(result.palette.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].message.contains("nonexistent"));
    }

    #[test]
    fn test_sprite_registry_not_found_strict() {
        let sprite_registry = SpriteRegistry::new();
        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("missing", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::NotFound(name) => assert_eq!(name, "missing"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_sprite_registry_not_found_lenient() {
        let sprite_registry = SpriteRegistry::new();
        let palette_registry = PaletteRegistry::new();

        let result = sprite_registry.resolve("missing", &palette_registry, false).unwrap();
        assert_eq!(result.name, "missing");
        assert!(result.grid.is_empty());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_sprite_registry_variant_preserves_grid() {
        // Ensure variant copies base grid exactly
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        sprite_registry.register_variant(hero_red_variant());

        let palette_registry = PaletteRegistry::new();

        let sprite_result = sprite_registry.resolve("hero", &palette_registry, false).unwrap();
        let variant_result = sprite_registry.resolve("hero_red", &palette_registry, false).unwrap();

        // Grid should be identical
        assert_eq!(sprite_result.grid, variant_result.grid);
        // Size should be identical
        assert_eq!(sprite_result.size, variant_result.size);
    }

    #[test]
    fn test_sprite_registry_variant_with_named_palette() {
        // Test variant of a sprite that uses a named palette
        let mut sprite_registry = SpriteRegistry::new();
        let mut palette_registry = PaletteRegistry::new();

        palette_registry.register(mono_palette());

        let sprite = checker_sprite_named();
        sprite_registry.register_sprite(sprite);

        // Create a variant that overrides {on}
        let variant = Variant {
            name: "checker_red".to_string(),
            base: "checker".to_string(),
            palette: HashMap::from([("{on}".to_string(), "#FF0000".to_string())]),
            ..Default::default()
        };
        sprite_registry.register_variant(variant);

        let result = sprite_registry.resolve("checker_red", &palette_registry, false).unwrap();
        assert_eq!(result.name, "checker_red");
        // {on} should be overridden
        assert_eq!(result.palette.get("{on}"), Some(&"#FF0000".to_string()));
        // {off} and {_} should be inherited from the mono palette
        assert_eq!(result.palette.get("{off}"), Some(&"#000000".to_string()));
        assert_eq!(result.palette.get("{_}"), Some(&"#00000000".to_string()));
    }

    #[test]
    fn test_sprite_registry_names() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        let names: Vec<_> = registry.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"hero".to_string()));
        assert!(names.contains(&&"hero_red".to_string()));
    }

    // ========== Transform Resolution Tests ==========

    #[test]
    fn test_resolve_sprite_with_source() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#FF0000".to_string()),
            ])),
            grid: vec!["{x}{x}".to_string(), "{_}{x}".to_string()],
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite that sources from base
        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#00FF00".to_string()), // Different color
            ])),
            grid: vec![], // Empty grid - should get from source
            source: Some("base".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Resolve derived should get base's grid
        let result = sprite_registry.resolve("derived", &palette_registry, false).unwrap();

        assert_eq!(result.name, "derived");
        assert_eq!(result.grid.len(), 2);
        assert_eq!(result.grid[0], "{x}{x}");
        assert_eq!(result.grid[1], "{_}{x}");
    }

    #[test]
    fn test_resolve_sprite_with_mirror_h_transform() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),
            grid: vec!["{a}{b}".to_string()],
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite with horizontal mirror transform
        let mirrored = Sprite {
            name: "mirrored".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),
            grid: vec![],
            source: Some("base".to_string()),
            transform: Some(vec![TransformSpec::String("mirror-h".to_string())]),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(mirrored);

        let result = sprite_registry.resolve("mirrored", &palette_registry, false).unwrap();

        // Grid should be horizontally mirrored: "{a}{b}" -> "{b}{a}"
        assert_eq!(result.grid.len(), 1);
        assert_eq!(result.grid[0], "{b}{a}");
    }

    #[test]
    fn test_resolve_sprite_with_rotate_transform() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a 2x2 base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{1}".to_string(), "#FF0000".to_string()),
                ("{2}".to_string(), "#00FF00".to_string()),
                ("{3}".to_string(), "#0000FF".to_string()),
                ("{4}".to_string(), "#FFFF00".to_string()),
            ])),
            grid: vec!["{1}{2}".to_string(), "{3}{4}".to_string()],
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite with 90 degree rotation
        let rotated = Sprite {
            name: "rotated".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("base".to_string()),
            transform: Some(vec![TransformSpec::String("rotate:90".to_string())]),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(rotated);

        let result = sprite_registry.resolve("rotated", &palette_registry, false).unwrap();

        // 90 degree clockwise rotation:
        // Original:    Rotated:
        // {1}{2}       {3}{1}
        // {3}{4}       {4}{2}
        assert_eq!(result.grid.len(), 2);
        assert_eq!(result.grid[0], "{3}{1}");
        assert_eq!(result.grid[1], "{4}{2}");
    }

    #[test]
    fn test_resolve_sprite_with_chained_transforms() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register a base sprite
        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),
            grid: vec!["{a}{b}".to_string()],
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Register a sprite with chained transforms: mirror-h then tile 2x1
        let transformed = Sprite {
            name: "transformed".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("base".to_string()),
            transform: Some(vec![
                TransformSpec::String("mirror-h".to_string()),
                TransformSpec::String("tile:2x1".to_string()),
            ]),
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(transformed);

        let result = sprite_registry.resolve("transformed", &palette_registry, false).unwrap();

        // First mirror-h: "{a}{b}" -> "{b}{a}"
        // Then tile 2x1: "{b}{a}" -> "{b}{a}{b}{a}"
        assert_eq!(result.grid.len(), 1);
        assert_eq!(result.grid[0], "{b}{a}{b}{a}");
    }

    #[test]
    fn test_resolve_sprite_source_not_found_strict() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("nonexistent".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Strict mode should error
        let result = sprite_registry.resolve("derived", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::SourceNotFound { sprite, source_name } => {
                assert_eq!(sprite, "derived");
                assert_eq!(source_name, "nonexistent");
            }
            e => panic!("Expected SourceNotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_sprite_source_not_found_lenient() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        let derived = Sprite {
            name: "derived".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("nonexistent".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(derived);

        // Lenient mode should return empty grid with warning
        let result = sprite_registry.resolve("derived", &palette_registry, false).unwrap();
        assert!(result.grid.is_empty());
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_resolve_sprite_circular_reference_strict() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Create sprites that reference each other
        let a = Sprite {
            name: "a".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("b".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        let b = Sprite {
            name: "b".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("a".to_string()),
            transform: None,
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(a);
        sprite_registry.register_sprite(b);

        // Strict mode should detect circular reference
        let result = sprite_registry.resolve("a", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::CircularReference { sprite, chain } => {
                assert_eq!(sprite, "a");
                assert!(chain.len() >= 2);
            }
            e => panic!("Expected CircularReference, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_variant_with_transform() {
        let palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        let base = Sprite {
            name: "base".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),
            grid: vec!["{a}{b}".to_string()],
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(base);

        // Variant with transform
        let variant = Variant {
            name: "variant".to_string(),
            base: "base".to_string(),
            palette: HashMap::from([("{a}".to_string(), "#0000FF".to_string())]),
            transform: Some(vec![TransformSpec::String("mirror-h".to_string())]),
        };
        sprite_registry.register_variant(variant);

        let result = sprite_registry.resolve("variant", &palette_registry, false).unwrap();

        // Grid should be mirrored
        assert_eq!(result.grid[0], "{b}{a}");
        // Palette should have overridden color
        assert_eq!(result.palette.get("{a}").unwrap(), "#0000FF");
        // Original color for {b} should be from base
        assert_eq!(result.palette.get("{b}").unwrap(), "#00FF00");
    }

    // ========== Registry Trait Tests ==========

    #[test]
    fn test_palette_registry_trait_len_and_empty() {
        let mut registry = PaletteRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(mono_palette());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.register(Palette {
            name: "other".to_string(),
            colors: HashMap::new(),
            ..Default::default()
        });
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_palette_registry_trait_names() {
        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        registry.register(Palette {
            name: "other".to_string(),
            colors: HashMap::new(),
            ..Default::default()
        });

        let names: Vec<_> = registry.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"mono".to_string()));
        assert!(names.contains(&&"other".to_string()));
    }

    #[test]
    fn test_palette_registry_trait_via_generic() {
        fn check_registry<V>(reg: &impl Registry<V>) -> usize {
            reg.len()
        }

        let mut registry = PaletteRegistry::new();
        registry.register(mono_palette());
        assert_eq!(check_registry::<Palette>(&registry), 1);
    }

    #[test]
    fn test_sprite_registry_len_and_empty() {
        let mut registry = SpriteRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register_sprite(hero_sprite());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.sprite_count(), 1);
        assert_eq!(registry.variant_count(), 0);

        registry.register_variant(hero_red_variant());
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.sprite_count(), 1);
        assert_eq!(registry.variant_count(), 1);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_sprite_registry_sprite_trait_via_generic() {
        fn check_registry<V>(reg: &impl Registry<V>) -> usize {
            reg.len()
        }

        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        // Via the Sprite trait, only sprites count
        assert_eq!(check_registry::<Sprite>(&registry), 1);
        // Via the Variant trait, only variants count
        assert_eq!(check_registry::<Variant>(&registry), 1);
    }

    #[test]
    fn test_sprite_registry_trait_contains() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        // Registry<Sprite> contains checks sprites only
        assert!(Registry::<Sprite>::contains(&registry, "hero"));
        assert!(!Registry::<Sprite>::contains(&registry, "hero_red"));

        // Registry<Variant> contains checks variants only
        assert!(!Registry::<Variant>::contains(&registry, "hero"));
        assert!(Registry::<Variant>::contains(&registry, "hero_red"));

        // The regular contains method checks both
        assert!(registry.contains("hero"));
        assert!(registry.contains("hero_red"));
    }

    #[test]
    fn test_sprite_registry_trait_get() {
        let mut registry = SpriteRegistry::new();
        registry.register_sprite(hero_sprite());
        registry.register_variant(hero_red_variant());

        // Registry<Sprite>::get returns sprites
        let sprite = Registry::<Sprite>::get(&registry, "hero");
        assert!(sprite.is_some());
        assert_eq!(sprite.unwrap().name, "hero");

        // Registry<Variant>::get returns variants
        let variant = Registry::<Variant>::get(&registry, "hero_red");
        assert!(variant.is_some());
        assert_eq!(variant.unwrap().name, "hero_red");
    }

    // ========== CompositionRegistry Tests (NC-1) ==========

    fn test_composition() -> Composition {
        Composition {
            name: "hero_scene".to_string(),
            base: None,
            size: Some([16, 16]),
            cell_size: Some([8, 8]),
            sprites: HashMap::from([
                ("hero".to_string(), Some("hero".to_string())),
                ("bg".to_string(), Some("background".to_string())),
            ]),
            layers: vec![],
        }
    }

    fn alt_composition() -> Composition {
        Composition {
            name: "alt_scene".to_string(),
            base: None,
            size: Some([32, 32]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        }
    }

    #[test]
    fn test_composition_registry_new() {
        let registry = CompositionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(!registry.contains("anything"));
    }

    #[test]
    fn test_composition_registry_register_and_get() {
        let mut registry = CompositionRegistry::new();
        let composition = test_composition();
        registry.register(composition);

        assert!(registry.contains("hero_scene"));
        let retrieved = registry.get("hero_scene").unwrap();
        assert_eq!(retrieved.name, "hero_scene");
        assert_eq!(retrieved.size, Some([16, 16]));
    }

    #[test]
    fn test_composition_registry_register_overwrites() {
        let mut registry = CompositionRegistry::new();
        let comp1 = Composition {
            name: "scene".to_string(),
            base: None,
            size: Some([8, 8]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };
        let comp2 = Composition {
            name: "scene".to_string(),
            base: None,
            size: Some([16, 16]),
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };

        registry.register(comp1);
        registry.register(comp2);

        assert_eq!(registry.len(), 1);
        let retrieved = registry.get("scene").unwrap();
        assert_eq!(retrieved.size, Some([16, 16]));
    }

    #[test]
    fn test_composition_registry_len_and_empty() {
        let mut registry = CompositionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(test_composition());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.register(alt_composition());
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_composition_registry_names() {
        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());
        registry.register(alt_composition());

        let names: Vec<_> = registry.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"hero_scene".to_string()));
        assert!(names.contains(&&"alt_scene".to_string()));
    }

    #[test]
    fn test_composition_registry_iter() {
        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());
        registry.register(alt_composition());

        let items: Vec<_> = registry.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_composition_registry_trait_via_generic() {
        fn check_registry<V>(reg: &impl Registry<V>) -> usize {
            reg.len()
        }

        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());
        assert_eq!(check_registry::<Composition>(&registry), 1);
    }

    #[test]
    fn test_composition_registry_trait_contains_and_get() {
        let mut registry = CompositionRegistry::new();
        registry.register(test_composition());

        assert!(Registry::<Composition>::contains(&registry, "hero_scene"));
        assert!(!Registry::<Composition>::contains(&registry, "nonexistent"));

        let composition = Registry::<Composition>::get(&registry, "hero_scene");
        assert!(composition.is_some());
        assert_eq!(composition.unwrap().name, "hero_scene");
    }

    // ========== Renderable and Unified Lookup Tests (NC-1) ==========

    #[test]
    fn test_renderable_sprite() {
        let sprite = hero_sprite();
        let renderable = Renderable::Sprite(&sprite);

        assert_eq!(renderable.name(), "hero");
        assert!(renderable.is_sprite());
        assert!(!renderable.is_composition());
        assert!(renderable.as_sprite().is_some());
        assert!(renderable.as_composition().is_none());
    }

    #[test]
    fn test_renderable_composition() {
        let composition = test_composition();
        let renderable = Renderable::Composition(&composition);

        assert_eq!(renderable.name(), "hero_scene");
        assert!(!renderable.is_sprite());
        assert!(renderable.is_composition());
        assert!(renderable.as_sprite().is_none());
        assert!(renderable.as_composition().is_some());
    }

    #[test]
    fn test_lookup_renderable_finds_sprite() {
        let mut sprite_registry = SpriteRegistry::new();
        sprite_registry.register_sprite(hero_sprite());
        let composition_registry = CompositionRegistry::new();

        let result = lookup_renderable("hero", &sprite_registry, &composition_registry);
        assert!(result.is_some());
        let renderable = result.unwrap();
        assert!(renderable.is_sprite());
        assert_eq!(renderable.name(), "hero");
    }

    #[test]
    fn test_lookup_renderable_finds_composition() {
        let sprite_registry = SpriteRegistry::new();
        let mut composition_registry = CompositionRegistry::new();
        composition_registry.register(test_composition());

        let result = lookup_renderable("hero_scene", &sprite_registry, &composition_registry);
        assert!(result.is_some());
        let renderable = result.unwrap();
        assert!(renderable.is_composition());
        assert_eq!(renderable.name(), "hero_scene");
    }

    #[test]
    fn test_lookup_renderable_sprite_takes_precedence() {
        // If both sprite and composition have the same name, sprite wins
        let mut sprite_registry = SpriteRegistry::new();
        let sprite = Sprite {
            name: "shared_name".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            metadata: None,
            ..Default::default()
        };
        sprite_registry.register_sprite(sprite);

        let mut composition_registry = CompositionRegistry::new();
        let composition = Composition {
            name: "shared_name".to_string(),
            base: None,
            size: None,
            cell_size: None,
            sprites: HashMap::new(),
            layers: vec![],
        };
        composition_registry.register(composition);

        let result = lookup_renderable("shared_name", &sprite_registry, &composition_registry);
        assert!(result.is_some());
        // Sprite takes precedence
        assert!(result.unwrap().is_sprite());
    }

    #[test]
    fn test_lookup_renderable_not_found() {
        let sprite_registry = SpriteRegistry::new();
        let composition_registry = CompositionRegistry::new();

        let result = lookup_renderable("nonexistent", &sprite_registry, &composition_registry);
        assert!(result.is_none());
    }

    // ========================================================================
    // Palette Ramp Expansion Tests
    // ========================================================================

    #[test]
    fn test_palette_ramp_expansion() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "skin_tones".to_string(),
            colors: HashMap::from([("{_}".to_string(), "#00000000".to_string())]),
            ramps: Some(HashMap::from([(
                "skin".to_string(),
                ColorRamp {
                    base: "#E8B89D".to_string(),
                    steps: Some(3),
                    shadow_shift: None,
                    highlight_shift: None,
                },
            )])),
        };

        registry.register(palette);

        let stored = registry.get("skin_tones").unwrap();

        // The ramp should have been expanded into colors
        assert!(stored.colors.contains_key("{skin}"), "Base color token should exist");
        assert!(stored.colors.contains_key("{skin_1}"), "Shadow token should exist");
        assert!(stored.colors.contains_key("{skin+1}"), "Highlight token should exist");

        // Verify base color is correct
        assert_eq!(stored.colors.get("{skin}").unwrap(), "#E8B89D");

        // Ramps should be cleared after expansion
        assert!(stored.ramps.is_none(), "Ramps should be None after expansion");
    }

    #[test]
    fn test_palette_ramp_expansion_5_steps() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "metals".to_string(),
            colors: HashMap::new(),
            ramps: Some(HashMap::from([(
                "gold".to_string(),
                ColorRamp {
                    base: "#FFD700".to_string(),
                    steps: Some(5),
                    shadow_shift: None,
                    highlight_shift: None,
                },
            )])),
        };

        registry.register(palette);
        let stored = registry.get("metals").unwrap();

        // 5 steps: _2, _1, base, +1, +2
        assert!(stored.colors.contains_key("{gold_2}"), "Darkest shadow should exist");
        assert!(stored.colors.contains_key("{gold_1}"), "Shadow should exist");
        assert!(stored.colors.contains_key("{gold}"), "Base should exist");
        assert!(stored.colors.contains_key("{gold+1}"), "Highlight should exist");
        assert!(stored.colors.contains_key("{gold+2}"), "Brightest should exist");
        assert_eq!(stored.colors.len(), 5);
    }

    #[test]
    fn test_palette_ramp_preserves_existing_colors() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "character".to_string(),
            colors: HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{hair}".to_string(), "#4A3728".to_string()),
            ]),
            ramps: Some(HashMap::from([(
                "skin".to_string(),
                ColorRamp {
                    base: "#E8B89D".to_string(),
                    steps: Some(3),
                    shadow_shift: None,
                    highlight_shift: None,
                },
            )])),
        };

        registry.register(palette);
        let stored = registry.get("character").unwrap();

        // Original colors should still be there
        assert_eq!(stored.colors.get("{_}").unwrap(), "#00000000");
        assert_eq!(stored.colors.get("{hair}").unwrap(), "#4A3728");

        // Plus the ramp colors
        assert!(stored.colors.contains_key("{skin}"));
        assert!(stored.colors.contains_key("{skin_1}"));
        assert!(stored.colors.contains_key("{skin+1}"));
    }

    #[test]
    fn test_palette_multiple_ramps() {
        use crate::models::ColorRamp;

        let mut registry = PaletteRegistry::new();

        let palette = Palette {
            name: "sprite".to_string(),
            colors: HashMap::new(),
            ramps: Some(HashMap::from([
                (
                    "skin".to_string(),
                    ColorRamp {
                        base: "#E8B89D".to_string(),
                        steps: Some(3),
                        shadow_shift: None,
                        highlight_shift: None,
                    },
                ),
                (
                    "hair".to_string(),
                    ColorRamp {
                        base: "#4A3728".to_string(),
                        steps: Some(3),
                        shadow_shift: None,
                        highlight_shift: None,
                    },
                ),
            ])),
        };

        registry.register(palette);
        let stored = registry.get("sprite").unwrap();

        // Both ramps should be expanded
        assert!(stored.colors.contains_key("{skin}"));
        assert!(stored.colors.contains_key("{skin_1}"));
        assert!(stored.colors.contains_key("{skin+1}"));
        assert!(stored.colors.contains_key("{hair}"));
        assert!(stored.colors.contains_key("{hair_1}"));
        assert!(stored.colors.contains_key("{hair+1}"));
    }
}
