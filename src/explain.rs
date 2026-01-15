//! Human-readable sprite explanations
//!
//! Provides explanations of sprite structure, tokens, colors, and patterns
//! for AI assistants and human users to understand sprite definitions.

use std::collections::HashMap;

use crate::color::parse_color;
use crate::models::{Animation, Composition, PaletteRef, Sprite, TtpObject, Variant};
use crate::palettes;
use crate::tokenizer::tokenize;

/// Token usage statistics within a sprite
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Token name (e.g., "{skin}")
    pub token: String,
    /// Number of times used in the grid
    pub count: usize,
    /// Percentage of total grid cells
    pub percentage: f64,
    /// Resolved color value (if available)
    pub color: Option<String>,
    /// Human-readable color name (if determinable)
    pub color_name: Option<String>,
}

/// Explanation of a sprite's structure and content
#[derive(Debug)]
pub struct SpriteExplanation {
    /// Sprite name
    pub name: String,
    /// Width in pixels/tokens
    pub width: usize,
    /// Height in pixels/rows
    pub height: usize,
    /// Total number of cells in the grid
    pub total_cells: usize,
    /// Palette reference (name or "inline")
    pub palette_ref: String,
    /// Token usage statistics, sorted by frequency
    pub tokens: Vec<TokenUsage>,
    /// Number of transparent cells
    pub transparent_count: usize,
    /// Percentage of transparent cells
    pub transparency_ratio: f64,
    /// Whether the sprite has consistent row widths
    pub consistent_rows: bool,
    /// Issues or warnings about the sprite
    pub issues: Vec<String>,
}

/// Explanation of a palette's structure
#[derive(Debug)]
pub struct PaletteExplanation {
    /// Palette name
    pub name: String,
    /// Number of colors defined
    pub color_count: usize,
    /// Color mappings (token -> color)
    pub colors: Vec<(String, String, Option<String>)>,
    /// Whether this is a built-in palette
    pub is_builtin: bool,
}

/// Explanation of an animation's structure
#[derive(Debug)]
pub struct AnimationExplanation {
    /// Animation name
    pub name: String,
    /// Frame sprite names
    pub frames: Vec<String>,
    /// Frame count
    pub frame_count: usize,
    /// Duration per frame in ms
    pub duration_ms: u32,
    /// Whether it loops
    pub loops: bool,
}

/// Explanation of a composition's structure
#[derive(Debug)]
pub struct CompositionExplanation {
    /// Composition name
    pub name: String,
    /// Base sprite (if any)
    pub base: Option<String>,
    /// Canvas size
    pub size: Option<[u32; 2]>,
    /// Cell size for tiling
    pub cell_size: [u32; 2],
    /// Sprite mappings
    pub sprite_count: usize,
    /// Layer count
    pub layer_count: usize,
}

/// Explanation of a variant's structure
#[derive(Debug)]
pub struct VariantExplanation {
    /// Variant name
    pub name: String,
    /// Base sprite name
    pub base: String,
    /// Number of color overrides
    pub override_count: usize,
    /// Color overrides (token -> new color)
    pub overrides: Vec<(String, String)>,
}

/// Unified explanation for any pixelsrc object
#[derive(Debug)]
pub enum Explanation {
    Sprite(SpriteExplanation),
    Palette(PaletteExplanation),
    Animation(AnimationExplanation),
    Composition(CompositionExplanation),
    Variant(VariantExplanation),
}

/// Analyze a sprite and produce an explanation
pub fn explain_sprite(sprite: &Sprite, palette_colors: Option<&HashMap<String, String>>) -> SpriteExplanation {
    let mut token_counts: HashMap<String, usize> = HashMap::new();
    let mut total_cells = 0;
    let mut first_row_width: Option<usize> = None;
    let mut consistent_rows = true;
    let mut issues = Vec::new();

    // Analyze each row
    for (row_idx, row) in sprite.grid.iter().enumerate() {
        let (tokens, warnings) = tokenize(row);
        let row_width = tokens.len();

        // Check row consistency
        match first_row_width {
            None => first_row_width = Some(row_width),
            Some(expected) if row_width != expected => {
                consistent_rows = false;
                issues.push(format!(
                    "Row {} has {} tokens (expected {})",
                    row_idx + 1,
                    row_width,
                    expected
                ));
            }
            _ => {}
        }

        // Count tokens
        for token in tokens {
            *token_counts.entry(token).or_insert(0) += 1;
            total_cells += 1;
        }

        // Collect tokenization warnings
        for warning in warnings {
            issues.push(warning.message);
        }
    }

    let width = first_row_width.unwrap_or(0);
    let height = sprite.grid.len();

    // Calculate transparency
    let transparent_count = token_counts.get("{_}").copied().unwrap_or(0);
    let transparency_ratio = if total_cells > 0 {
        (transparent_count as f64 / total_cells as f64) * 100.0
    } else {
        0.0
    };

    // Build token usage list
    let mut tokens: Vec<TokenUsage> = token_counts
        .iter()
        .map(|(token, &count)| {
            let percentage = if total_cells > 0 {
                (count as f64 / total_cells as f64) * 100.0
            } else {
                0.0
            };

            let color = palette_colors.and_then(|c| c.get(token).cloned());
            let color_name = color.as_ref().and_then(|c| describe_color(c));

            TokenUsage {
                token: token.clone(),
                count,
                percentage,
                color,
                color_name,
            }
        })
        .collect();

    // Sort by frequency (descending)
    tokens.sort_by(|a, b| b.count.cmp(&a.count));

    // Determine palette reference
    let palette_ref = match &sprite.palette {
        PaletteRef::Named(name) => name.clone(),
        PaletteRef::Inline(_) => "inline".to_string(),
    };

    SpriteExplanation {
        name: sprite.name.clone(),
        width,
        height,
        total_cells,
        palette_ref,
        tokens,
        transparent_count,
        transparency_ratio,
        consistent_rows,
        issues,
    }
}

/// Analyze a palette and produce an explanation
pub fn explain_palette(name: &str, colors: &HashMap<String, String>) -> PaletteExplanation {
    let mut color_list: Vec<(String, String, Option<String>)> = colors
        .iter()
        .map(|(token, color)| {
            let name = describe_color(color);
            (token.clone(), color.clone(), name)
        })
        .collect();

    // Sort alphabetically by token
    color_list.sort_by(|a, b| a.0.cmp(&b.0));

    PaletteExplanation {
        name: name.to_string(),
        color_count: colors.len(),
        colors: color_list,
        is_builtin: false,
    }
}

/// Explain an animation
pub fn explain_animation(animation: &Animation) -> AnimationExplanation {
    AnimationExplanation {
        name: animation.name.clone(),
        frames: animation.frames.clone(),
        frame_count: animation.frames.len(),
        duration_ms: animation.duration_ms(),
        loops: animation.loops(),
    }
}

/// Explain a composition
pub fn explain_composition(composition: &Composition) -> CompositionExplanation {
    CompositionExplanation {
        name: composition.name.clone(),
        base: composition.base.clone(),
        size: composition.size,
        cell_size: composition.cell_size(),
        sprite_count: composition.sprites.len(),
        layer_count: composition.layers.len(),
    }
}

/// Explain a variant
pub fn explain_variant(variant: &Variant) -> VariantExplanation {
    let overrides: Vec<(String, String)> = variant
        .palette
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    VariantExplanation {
        name: variant.name.clone(),
        base: variant.base.clone(),
        override_count: variant.palette.len(),
        overrides,
    }
}

/// Explain any TtpObject
pub fn explain_object(obj: &TtpObject, palette_colors: Option<&HashMap<String, String>>) -> Explanation {
    match obj {
        TtpObject::Sprite(sprite) => Explanation::Sprite(explain_sprite(sprite, palette_colors)),
        TtpObject::Palette(palette) => {
            Explanation::Palette(explain_palette(&palette.name, &palette.colors))
        }
        TtpObject::Animation(anim) => Explanation::Animation(explain_animation(anim)),
        TtpObject::Composition(comp) => Explanation::Composition(explain_composition(comp)),
        TtpObject::Variant(variant) => Explanation::Variant(explain_variant(variant)),
    }
}

/// Describe a hex color in human-readable terms
pub fn describe_color(hex: &str) -> Option<String> {
    // Parse the color
    let rgba = parse_color(hex).ok()?;

    // Check for transparency
    if rgba[3] == 0 {
        return Some("transparent".to_string());
    }

    let r = rgba[0];
    let g = rgba[1];
    let b = rgba[2];

    // Check for grayscale
    if r == g && g == b {
        return Some(match r {
            0 => "black",
            255 => "white",
            0..=63 => "dark gray",
            64..=127 => "gray",
            128..=191 => "light gray",
            192..=254 => "very light gray",
        }
        .to_string());
    }

    // Determine dominant color(s)
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    // Calculate hue
    let hue = if max == min {
        0.0 // gray
    } else {
        let delta = (max - min) as f64;
        let h = if max == r {
            ((g as f64 - b as f64) / delta) % 6.0
        } else if max == g {
            ((b as f64 - r as f64) / delta) + 2.0
        } else {
            ((r as f64 - g as f64) / delta) + 4.0
        };
        h * 60.0
    };

    // Normalize hue to 0-360
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    // Calculate saturation and lightness
    let lightness = (max as f64 + min as f64) / 510.0; // 0-1
    let saturation = if max == min {
        0.0
    } else if lightness <= 0.5 {
        (max - min) as f64 / (max + min) as f64
    } else {
        (max - min) as f64 / (510.0 - max as f64 - min as f64)
    };

    // Map hue to color name
    let base_color = match hue as u32 {
        0..=14 | 346..=360 => "red",
        15..=44 => "orange",
        45..=74 => "yellow",
        75..=154 => "green",
        155..=184 => "cyan",
        185..=254 => "blue",
        255..=284 => "purple",
        285..=345 => "magenta",
        _ => "red",
    };

    // Add modifiers based on saturation and lightness
    let modifier = if saturation < 0.2 {
        "grayish "
    } else if lightness < 0.2 {
        "dark "
    } else if lightness > 0.8 {
        "light "
    } else if saturation > 0.8 && lightness > 0.4 && lightness < 0.6 {
        "bright "
    } else {
        ""
    };

    Some(format!("{}{}", modifier, base_color))
}

/// Format a sprite explanation as human-readable text
pub fn format_sprite_explanation(exp: &SpriteExplanation) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("Sprite: {}\n", exp.name));
    output.push_str(&format!("Size: {}x{} pixels ({} cells)\n", exp.width, exp.height, exp.total_cells));
    output.push_str(&format!("Palette: {}\n", exp.palette_ref));
    output.push('\n');

    // Token summary
    output.push_str("TOKENS USED\n");
    output.push_str("-----------\n");

    for usage in &exp.tokens {
        let color_desc = match (&usage.color, &usage.color_name) {
            (Some(hex), Some(name)) => format!(" {} ({})", hex, name),
            (Some(hex), None) => format!(" {}", hex),
            _ => String::new(),
        };

        output.push_str(&format!(
            "  {:12} {:>4}x ({:>5.1}%){}",
            usage.token, usage.count, usage.percentage, color_desc
        ));
        output.push('\n');
    }
    output.push('\n');

    // Structure info
    output.push_str("STRUCTURE\n");
    output.push_str("---------\n");
    output.push_str(&format!("  Transparency: {:.1}% ({} cells)\n", exp.transparency_ratio, exp.transparent_count));
    output.push_str(&format!("  Row consistency: {}\n", if exp.consistent_rows { "yes" } else { "no" }));
    output.push_str(&format!("  Unique tokens: {}\n", exp.tokens.len()));

    // Issues
    if !exp.issues.is_empty() {
        output.push('\n');
        output.push_str("ISSUES\n");
        output.push_str("------\n");
        for issue in &exp.issues {
            output.push_str(&format!("  - {}\n", issue));
        }
    }

    output
}

/// Format a palette explanation as human-readable text
pub fn format_palette_explanation(exp: &PaletteExplanation) -> String {
    let mut output = String::new();

    output.push_str(&format!("Palette: {}\n", exp.name));
    output.push_str(&format!("Colors: {}\n", exp.color_count));
    if exp.is_builtin {
        output.push_str("Type: built-in\n");
    }
    output.push('\n');

    output.push_str("COLOR MAPPINGS\n");
    output.push_str("--------------\n");

    for (token, hex, name) in &exp.colors {
        let desc = name.as_ref().map(|n| format!(" ({})", n)).unwrap_or_default();
        output.push_str(&format!("  {:12} => {}{}\n", token, hex, desc));
    }

    output
}

/// Format an animation explanation as human-readable text
pub fn format_animation_explanation(exp: &AnimationExplanation) -> String {
    let mut output = String::new();

    output.push_str(&format!("Animation: {}\n", exp.name));
    output.push_str(&format!("Frames: {}\n", exp.frame_count));
    output.push_str(&format!("Duration: {}ms per frame\n", exp.duration_ms));
    output.push_str(&format!("Loops: {}\n", if exp.loops { "yes" } else { "no" }));
    output.push('\n');

    output.push_str("FRAME SEQUENCE\n");
    output.push_str("--------------\n");
    for (i, frame) in exp.frames.iter().enumerate() {
        output.push_str(&format!("  {}: {}\n", i + 1, frame));
    }

    output
}

/// Format a composition explanation as human-readable text
pub fn format_composition_explanation(exp: &CompositionExplanation) -> String {
    let mut output = String::new();

    output.push_str(&format!("Composition: {}\n", exp.name));
    if let Some(base) = &exp.base {
        output.push_str(&format!("Base sprite: {}\n", base));
    }
    if let Some(size) = exp.size {
        output.push_str(&format!("Canvas size: {}x{}\n", size[0], size[1]));
    }
    output.push_str(&format!("Cell size: {}x{}\n", exp.cell_size[0], exp.cell_size[1]));
    output.push_str(&format!("Sprite mappings: {}\n", exp.sprite_count));
    output.push_str(&format!("Layers: {}\n", exp.layer_count));

    output
}

/// Format a variant explanation as human-readable text
pub fn format_variant_explanation(exp: &VariantExplanation) -> String {
    let mut output = String::new();

    output.push_str(&format!("Variant: {}\n", exp.name));
    output.push_str(&format!("Base sprite: {}\n", exp.base));
    output.push_str(&format!("Color overrides: {}\n", exp.override_count));
    output.push('\n');

    if !exp.overrides.is_empty() {
        output.push_str("PALETTE OVERRIDES\n");
        output.push_str("-----------------\n");
        for (token, color) in &exp.overrides {
            let desc = describe_color(color)
                .map(|n| format!(" ({})", n))
                .unwrap_or_default();
            output.push_str(&format!("  {:12} => {}{}\n", token, color, desc));
        }
    }

    output
}

/// Format any explanation as human-readable text
pub fn format_explanation(exp: &Explanation) -> String {
    match exp {
        Explanation::Sprite(s) => format_sprite_explanation(s),
        Explanation::Palette(p) => format_palette_explanation(p),
        Explanation::Animation(a) => format_animation_explanation(a),
        Explanation::Composition(c) => format_composition_explanation(c),
        Explanation::Variant(v) => format_variant_explanation(v),
    }
}

/// Resolve palette colors from a sprite's palette reference
pub fn resolve_palette_colors(
    palette_ref: &PaletteRef,
    known_palettes: &HashMap<String, HashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    match palette_ref {
        PaletteRef::Named(name) => {
            // Check for built-in palette
            if name.starts_with('@') {
                let builtin_name = name.strip_prefix('@').unwrap_or(name);
                return palettes::get_builtin(builtin_name).map(|p| p.colors.clone());
            }
            // Check known palettes
            known_palettes.get(name).cloned()
        }
        PaletteRef::Inline(colors) => Some(colors.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_color_transparent() {
        assert_eq!(describe_color("#00000000"), Some("transparent".to_string()));
    }

    #[test]
    fn test_describe_color_black_white() {
        assert_eq!(describe_color("#000000"), Some("black".to_string()));
        assert_eq!(describe_color("#FFFFFF"), Some("white".to_string()));
    }

    #[test]
    fn test_describe_color_primary() {
        assert!(describe_color("#FF0000").unwrap().contains("red"));
        assert!(describe_color("#00FF00").unwrap().contains("green"));
        assert!(describe_color("#0000FF").unwrap().contains("blue"));
    }

    #[test]
    fn test_explain_sprite_basic() {
        let sprite = Sprite {
            name: "test".to_string(),
            size: Some([2, 2]),
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{x}".to_string(), "#FF0000".to_string()),
            ])),
            grid: vec!["{_}{x}".to_string(), "{x}{_}".to_string()],
        };

        let colors = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{x}".to_string(), "#FF0000".to_string()),
        ]);

        let exp = explain_sprite(&sprite, Some(&colors));

        assert_eq!(exp.name, "test");
        assert_eq!(exp.width, 2);
        assert_eq!(exp.height, 2);
        assert_eq!(exp.total_cells, 4);
        assert_eq!(exp.transparent_count, 2);
        assert!((exp.transparency_ratio - 50.0).abs() < 0.01);
        assert!(exp.consistent_rows);
        assert!(exp.issues.is_empty());
    }

    #[test]
    fn test_explain_sprite_inconsistent_rows() {
        let sprite = Sprite {
            name: "uneven".to_string(),
            size: None,
            palette: PaletteRef::Named("test".to_string()),
            grid: vec!["{a}{b}{c}".to_string(), "{a}{b}".to_string()],
        };

        let exp = explain_sprite(&sprite, None);

        assert!(!exp.consistent_rows);
        assert!(!exp.issues.is_empty());
    }

    #[test]
    fn test_explain_palette_basic() {
        let colors = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{x}".to_string(), "#FF0000".to_string()),
        ]);

        let exp = explain_palette("test", &colors);

        assert_eq!(exp.name, "test");
        assert_eq!(exp.color_count, 2);
        assert!(!exp.is_builtin);
    }

    #[test]
    fn test_format_sprite_explanation() {
        let exp = SpriteExplanation {
            name: "test".to_string(),
            width: 8,
            height: 8,
            total_cells: 64,
            palette_ref: "hero".to_string(),
            tokens: vec![
                TokenUsage {
                    token: "{_}".to_string(),
                    count: 32,
                    percentage: 50.0,
                    color: Some("#00000000".to_string()),
                    color_name: Some("transparent".to_string()),
                },
                TokenUsage {
                    token: "{x}".to_string(),
                    count: 32,
                    percentage: 50.0,
                    color: Some("#FF0000".to_string()),
                    color_name: Some("red".to_string()),
                },
            ],
            transparent_count: 32,
            transparency_ratio: 50.0,
            consistent_rows: true,
            issues: vec![],
        };

        let output = format_sprite_explanation(&exp);

        assert!(output.contains("Sprite: test"));
        assert!(output.contains("Size: 8x8"));
        assert!(output.contains("Palette: hero"));
        assert!(output.contains("{_}"));
        assert!(output.contains("{x}"));
        assert!(output.contains("transparent"));
    }
}
