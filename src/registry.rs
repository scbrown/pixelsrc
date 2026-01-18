//! Palette registry for resolving palette references
//!
//! The registry stores named palettes and resolves palette references for sprites.
//! Supports both lenient mode (warnings + fallback) and strict mode (errors).
//!
//! Also handles sprite resolution including source references and transform application.

use std::collections::HashMap;
use std::fmt;

use crate::models::{Palette, PaletteRef, Sprite, TransformSpec, Variant};
use crate::palette_parser::{PaletteParser, ParseMode};
use crate::palettes;
use crate::transforms::{self, Transform, TransformError};

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
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteError {
    /// Referenced palette name was not found in registry
    NotFound(String),
    /// Referenced built-in palette (@name) was not found
    BuiltinNotFound(String),
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaletteError::NotFound(name) => write!(f, "Palette '{}' not found", name),
            PaletteError::BuiltinNotFound(name) => {
                write!(f, "Built-in palette '{}' not found", name)
            }
        }
    }
}

impl std::error::Error for PaletteError {}

/// Warning when resolving a palette in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteWarning {
    pub message: String,
}

impl PaletteWarning {
    pub fn not_found(name: &str) -> Self {
        Self {
            message: format!("Palette '{}' not found", name),
        }
    }

    pub fn builtin_not_found(name: &str) -> Self {
        Self {
            message: format!("Built-in palette '{}' not found", name),
        }
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
fn resolve_palette_variables(colors: &HashMap<String, String>, strict: bool) -> (HashMap<String, String>, Vec<PaletteWarning>) {
    let parser = PaletteParser::new();
    let mode = if strict { ParseMode::Strict } else { ParseMode::Lenient };

    match parser.resolve_to_strings(colors, mode) {
        Ok(result) => {
            let warnings: Vec<PaletteWarning> = result.warnings
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
        Self {
            palettes: HashMap::new(),
        }
    }

    /// Register a palette in the registry.
    ///
    /// If a palette with the same name already exists, it is replaced.
    pub fn register(&mut self, palette: Palette) {
        self.palettes.insert(palette.name.clone(), palette);
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
                    let (resolved_colors, _warnings) = resolve_palette_variables(&palette.colors, true);
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
                Ok(ResolvedPalette {
                    colors: resolved_colors,
                    source: PaletteSource::Inline,
                })
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
                    let (resolved_colors, var_warnings) = resolve_palette_variables(&palette.colors, false);
                    let warning = if var_warnings.is_empty() {
                        None
                    } else {
                        // Combine multiple variable warnings into one
                        let messages: Vec<String> = var_warnings.into_iter().map(|w| w.message).collect();
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
                    let messages: Vec<String> = var_warnings.into_iter().map(|w| w.message).collect();
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
            self.resolve_strict(sprite).map(|palette| LenientResult {
                palette,
                warning: None,
            })
        } else {
            Ok(self.resolve_lenient(sprite))
        }
    }
}

// ============================================================================
// Sprite and Variant Registry
// ============================================================================

/// Error when resolving a sprite or variant.
#[derive(Debug, Clone, PartialEq)]
pub enum SpriteError {
    /// Referenced sprite/variant was not found
    NotFound(String),
    /// Variant references a base sprite that doesn't exist
    BaseNotFound { variant: String, base: String },
    /// Sprite references a source sprite that doesn't exist
    SourceNotFound { sprite: String, source: String },
    /// Circular reference detected during source resolution
    CircularReference { sprite: String, chain: Vec<String> },
    /// Error applying transform
    TransformError { sprite: String, message: String },
}

impl fmt::Display for SpriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpriteError::NotFound(name) => write!(f, "Sprite or variant '{}' not found", name),
            SpriteError::BaseNotFound { variant, base } => {
                write!(
                    f,
                    "Variant '{}' references unknown base sprite '{}'",
                    variant, base
                )
            }
            SpriteError::SourceNotFound { sprite, source } => {
                write!(
                    f,
                    "Sprite '{}' references unknown source sprite '{}'",
                    sprite, source
                )
            }
            SpriteError::CircularReference { sprite, chain } => {
                write!(
                    f,
                    "Circular reference detected for sprite '{}': {}",
                    sprite,
                    chain.join(" -> ")
                )
            }
            SpriteError::TransformError { sprite, message } => {
                write!(f, "Transform error for sprite '{}': {}", sprite, message)
            }
        }
    }
}

impl std::error::Error for SpriteError {}

/// Warning when resolving a sprite or variant in lenient mode.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteWarning {
    pub message: String,
}

impl SpriteWarning {
    pub fn not_found(name: &str) -> Self {
        Self {
            message: format!("Sprite or variant '{}' not found", name),
        }
    }

    pub fn base_not_found(variant: &str, base: &str) -> Self {
        Self {
            message: format!(
                "Variant '{}' references unknown base sprite '{}'",
                variant, base
            ),
        }
    }

    pub fn source_not_found(sprite: &str, source: &str) -> Self {
        Self {
            message: format!(
                "Sprite '{}' references unknown source sprite '{}'",
                sprite, source
            ),
        }
    }

    pub fn transform_error(sprite: &str, message: &str) -> Self {
        Self {
            message: format!("Transform error for sprite '{}': {}", sprite, message),
        }
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
fn apply_grid_transform(grid: &[String], transform: &Transform) -> Result<Vec<String>, TransformError> {
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
        Transform::Subpixel { .. } => {
            Err(TransformError::InvalidParameter {
                op: "subpixel".to_string(),
                message: "subpixel transforms are not yet implemented for sprite grids".to_string(),
            })
        }

        // Animation transforms should not be applied to sprite grids
        Transform::Pingpong { .. }
        | Transform::Reverse
        | Transform::FrameOffset { .. }
        | Transform::Hold { .. } => {
            Err(TransformError::InvalidParameter {
                op: format!("{:?}", transform),
                message: "animation transforms cannot be applied to sprite grids".to_string(),
            })
        }
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
    let horizontal_padding: String = std::iter::repeat(pad_token)
        .take(size as usize)
        .collect::<Vec<_>>()
        .join("");
    let full_width_row: String = std::iter::repeat(pad_token)
        .take(max_width + 2 * size as usize)
        .collect::<Vec<_>>()
        .join("");

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
fn crop_grid(grid: &[String], x: u32, y: u32, w: u32, h: u32) -> Result<Vec<String>, TransformError> {
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
            let transparent_row: String = std::iter::repeat("{_}")
                .take(w as usize)
                .collect::<Vec<_>>()
                .join("");
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
        Self {
            sprites: HashMap::new(),
            variants: HashMap::new(),
        }
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
                        message: format!(
                            "Circular reference detected: {}",
                            visited.join(" -> ")
                        ),
                    }],
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
                    let source_resolved =
                        self.resolve_sprite_internal(source_sprite, palette_registry, strict, visited)?;
                    warnings.extend(source_resolved.warnings);
                    source_resolved.grid
                }
                None => {
                    if strict {
                        return Err(SpriteError::SourceNotFound {
                            sprite: sprite.name.clone(),
                            source: source_name.clone(),
                        });
                    } else {
                        warnings.push(SpriteWarning::source_not_found(
                            &sprite.name,
                            source_name,
                        ));
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
                        warnings.push(SpriteWarning::transform_error(
                            &sprite.name,
                            &e.to_string(),
                        ));
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
                    warnings.push(SpriteWarning {
                        message: warning.message,
                    });
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
                    });
                }
            }
        };

        let mut warnings = Vec::new();

        // Resolve the base sprite's palette
        let base_palette = match palette_registry.resolve(base_sprite, strict) {
            Ok(result) => {
                if let Some(warning) = result.warning {
                    warnings.push(SpriteWarning {
                        message: warning.message,
                    });
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
            match self.apply_transforms_to_grid(&base_grid, transform_specs, &variant.name, strict) {
                Ok((transformed, transform_warnings)) => {
                    warnings.extend(transform_warnings);
                    transformed
                }
                Err(e) => {
                    if strict {
                        return Err(e);
                    } else {
                        warnings.push(SpriteWarning::transform_error(
                            &variant.name,
                            &e.to_string(),
                        ));
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
        })
    }

    /// Get all sprite and variant names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.sprites.keys().chain(self.variants.keys())
    }
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
        }
    }

    fn checker_sprite_named() -> Sprite {
        Sprite {
            name: "checker".to_string(),
            size: None,
            palette: PaletteRef::Named("mono".to_string()),
            grid: vec![
                "{on}{off}{on}{off}".to_string(),
                "{off}{on}{off}{on}".to_string(),
            ],
            metadata: None, ..Default::default()
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
            metadata: None, ..Default::default()
        }
    }

    fn bad_ref_sprite() -> Sprite {
        Sprite {
            name: "bad_ref".to_string(),
            size: None,
            palette: PaletteRef::Named("nonexistent".to_string()),
            grid: vec!["{x}{x}".to_string()],
            metadata: None, ..Default::default()
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
        };
        let palette2 = Palette {
            name: "test".to_string(),
            colors: HashMap::from([("{b}".to_string(), "#00FF00".to_string())]),
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
        assert_eq!(
            result,
            Err(PaletteError::NotFound("nonexistent".to_string()))
        );
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
        assert_eq!(
            result.palette.source,
            PaletteSource::Named("mono".to_string())
        );
    }

    #[test]
    fn test_resolve_lenient_named_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = bad_ref_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert!(result
            .warning
            .as_ref()
            .unwrap()
            .message
            .contains("nonexistent"));
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
        assert_eq!(
            result.unwrap_err(),
            PaletteError::NotFound("nonexistent".to_string())
        );
    }

    #[test]
    fn test_fixture_unknown_palette_ref_lenient() {
        let registry = PaletteRegistry::new();

        // {"type": "sprite", "name": "bad_ref", "palette": "nonexistent", "grid": ["{x}{x}"]}
        let sprite = bad_ref_sprite();
        let result = registry.resolve_lenient(&sprite);

        assert!(result.warning.is_some());
        assert_eq!(
            result.warning.unwrap().message,
            "Palette 'nonexistent' not found"
        );
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
            metadata: None, ..Default::default()
        }
    }

    fn builtin_nonexistent_sprite() -> Sprite {
        Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Named("@nonexistent".to_string()),
            grid: vec!["{x}{x}".to_string()],
            metadata: None, ..Default::default()
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
            metadata: None, ..Default::default()
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
        assert_eq!(
            result.colors.get("{lightest}"),
            Some(&"#9BBC0F".to_string())
        );
        assert_eq!(result.colors.get("{dark}"), Some(&"#306230".to_string()));
    }

    #[test]
    fn test_resolve_strict_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_strict(&sprite);
        assert_eq!(
            result,
            Err(PaletteError::BuiltinNotFound("nonexistent".to_string()))
        );
    }

    #[test]
    fn test_resolve_lenient_builtin_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_gameboy_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_none());
        assert_eq!(
            result.palette.source,
            PaletteSource::Builtin("gameboy".to_string())
        );
        assert_eq!(
            result.palette.colors.get("{lightest}"),
            Some(&"#9BBC0F".to_string())
        );
    }

    #[test]
    fn test_resolve_lenient_builtin_not_found() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve_lenient(&sprite);
        assert!(result.warning.is_some());
        assert_eq!(
            result.warning.unwrap().message,
            "Built-in palette 'nonexistent' not found"
        );
        assert_eq!(result.palette.source, PaletteSource::Fallback);
        assert!(result.palette.colors.is_empty());
    }

    #[test]
    fn test_resolve_combined_builtin_strict() {
        let registry = PaletteRegistry::new();
        let sprite = builtin_nonexistent_sprite();

        let result = registry.resolve(&sprite, true);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            PaletteError::BuiltinNotFound("nonexistent".to_string())
        );
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
        assert_eq!(
            result.colors.get("{lightest}"),
            Some(&"#9BBC0F".to_string())
        );
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
                metadata: None, ..Default::default()
            };
            let result = registry.resolve_strict(&sprite);
            assert!(
                result.is_ok(),
                "Built-in palette @{} should be resolvable",
                name
            );
            assert_eq!(
                result.unwrap().source,
                PaletteSource::Builtin(name.to_string())
            );
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

        let result = sprite_registry
            .resolve("hero", &palette_registry, false)
            .unwrap();
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

        let result = sprite_registry
            .resolve("hero_red", &palette_registry, false)
            .unwrap();
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

        let result = sprite_registry
            .resolve("hero_alt", &palette_registry, false)
            .unwrap();
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

        let result = sprite_registry
            .resolve("ghost", &palette_registry, false)
            .unwrap();
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

        let result = sprite_registry
            .resolve("missing", &palette_registry, false)
            .unwrap();
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

        let sprite_result = sprite_registry
            .resolve("hero", &palette_registry, false)
            .unwrap();
        let variant_result = sprite_registry
            .resolve("hero_red", &palette_registry, false)
            .unwrap();

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

        let result = sprite_registry
            .resolve("checker_red", &palette_registry, false)
            .unwrap();
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
        };
        sprite_registry.register_sprite(derived);

        // Resolve derived should get base's grid
        let result = sprite_registry
            .resolve("derived", &palette_registry, false)
            .unwrap();

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
        };
        sprite_registry.register_sprite(mirrored);

        let result = sprite_registry
            .resolve("mirrored", &palette_registry, false)
            .unwrap();

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
            grid: vec![
                "{1}{2}".to_string(),
                "{3}{4}".to_string(),
            ],
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
        };
        sprite_registry.register_sprite(rotated);

        let result = sprite_registry
            .resolve("rotated", &palette_registry, false)
            .unwrap();

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
        };
        sprite_registry.register_sprite(transformed);

        let result = sprite_registry
            .resolve("transformed", &palette_registry, false)
            .unwrap();

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
        };
        sprite_registry.register_sprite(derived);

        // Strict mode should error
        let result = sprite_registry.resolve("derived", &palette_registry, true);
        assert!(result.is_err());
        match result.unwrap_err() {
            SpriteError::SourceNotFound { sprite, source } => {
                assert_eq!(sprite, "derived");
                assert_eq!(source, "nonexistent");
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
        };
        sprite_registry.register_sprite(derived);

        // Lenient mode should return empty grid with warning
        let result = sprite_registry
            .resolve("derived", &palette_registry, false)
            .unwrap();
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
        };
        let b = Sprite {
            name: "b".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec![],
            source: Some("a".to_string()),
            transform: None,
            metadata: None,
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

        let result = sprite_registry
            .resolve("variant", &palette_registry, false)
            .unwrap();

        // Grid should be mirrored
        assert_eq!(result.grid[0], "{b}{a}");
        // Palette should have overridden color
        assert_eq!(result.palette.get("{a}").unwrap(), "#0000FF");
        // Original color for {b} should be from base
        assert_eq!(result.palette.get("{b}").unwrap(), "#00FF00");
    }
}
