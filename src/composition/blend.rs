//! Blend modes for composition layers

use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

/// Blend modes for composition layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    /// Standard alpha compositing (source over destination)
    #[default]
    Normal,
    /// Darkens underlying colors: result = base * blend
    Multiply,
    /// Lightens underlying colors: result = 1 - (1 - base) * (1 - blend)
    Screen,
    /// Combines multiply/screen based on base brightness
    Overlay,
    /// Additive blending: result = min(1, base + blend)
    Add,
    /// Subtractive blending: result = max(0, base - blend)
    Subtract,
    /// Color difference: result = abs(base - blend)
    Difference,
    /// Keeps darker color: result = min(base, blend)
    Darken,
    /// Keeps lighter color: result = max(base, blend)
    Lighten,
}

impl BlendMode {
    /// Parse a blend mode from string
    pub fn from_str(s: &str) -> Option<BlendMode> {
        match s.to_lowercase().as_str() {
            "normal" => Some(BlendMode::Normal),
            "multiply" => Some(BlendMode::Multiply),
            "screen" => Some(BlendMode::Screen),
            "overlay" => Some(BlendMode::Overlay),
            "add" | "additive" => Some(BlendMode::Add),
            "subtract" | "subtractive" => Some(BlendMode::Subtract),
            "difference" => Some(BlendMode::Difference),
            "darken" => Some(BlendMode::Darken),
            "lighten" => Some(BlendMode::Lighten),
            _ => None,
        }
    }

    /// Apply blend mode to a single color channel (values are 0.0-1.0)
    pub(crate) fn blend_channel(&self, base: f32, blend: f32) -> f32 {
        match self {
            BlendMode::Normal => blend,
            BlendMode::Multiply => base * blend,
            BlendMode::Screen => 1.0 - (1.0 - base) * (1.0 - blend),
            BlendMode::Overlay => {
                if base < 0.5 {
                    2.0 * base * blend
                } else {
                    1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
                }
            }
            BlendMode::Add => (base + blend).min(1.0),
            BlendMode::Subtract => (base - blend).max(0.0),
            BlendMode::Difference => (base - blend).abs(),
            BlendMode::Darken => base.min(blend),
            BlendMode::Lighten => base.max(blend),
        }
    }
}

/// Blit a sprite onto the canvas at the given position.
/// Uses alpha blending for transparent pixels.
pub(crate) fn blit_sprite(canvas: &mut RgbaImage, sprite: &RgbaImage, x: u32, y: u32) {
    blit_sprite_blended(canvas, sprite, x, y, BlendMode::Normal, 1.0);
}

/// Blit a sprite onto the canvas with blend mode and opacity.
///
/// # Arguments
/// * `canvas` - The destination image
/// * `sprite` - The sprite to blit
/// * `x`, `y` - Position on canvas
/// * `blend_mode` - How to blend colors (ATF-10)
/// * `opacity` - Layer opacity (0.0-1.0)
pub(crate) fn blit_sprite_blended(
    canvas: &mut RgbaImage,
    sprite: &RgbaImage,
    x: u32,
    y: u32,
    blend_mode: BlendMode,
    opacity: f64,
) {
    let canvas_width = canvas.width();
    let canvas_height = canvas.height();
    let opacity = opacity.clamp(0.0, 1.0) as f32;

    for (sy, row) in sprite.rows().enumerate() {
        let dest_y = y + sy as u32;
        if dest_y >= canvas_height {
            break;
        }

        for (sx, pixel) in row.enumerate() {
            let dest_x = x + sx as u32;
            if dest_x >= canvas_width {
                break;
            }

            let src = pixel;
            // Fully transparent source, skip
            if src[3] == 0 {
                continue;
            }

            // Apply layer opacity to source alpha
            let src_alpha = (src[3] as f32 / 255.0) * opacity;
            if src_alpha == 0.0 {
                continue;
            }

            let dst = canvas.get_pixel(dest_x, dest_y);
            let blended = blend_pixels(src, dst, blend_mode, src_alpha);
            canvas.put_pixel(dest_x, dest_y, blended);
        }
    }
}

/// Blend source pixel over destination using the specified blend mode and opacity.
pub(crate) fn blend_pixels(src: &Rgba<u8>, dst: &Rgba<u8>, mode: BlendMode, src_alpha: f32) -> Rgba<u8> {
    let dst_alpha = dst[3] as f32 / 255.0;

    // Normalize colors to 0.0-1.0
    let src_r = src[0] as f32 / 255.0;
    let src_g = src[1] as f32 / 255.0;
    let src_b = src[2] as f32 / 255.0;

    let dst_r = dst[0] as f32 / 255.0;
    let dst_g = dst[1] as f32 / 255.0;
    let dst_b = dst[2] as f32 / 255.0;

    // Apply blend mode to get the blended color (before alpha compositing)
    let blended_r = mode.blend_channel(dst_r, src_r);
    let blended_g = mode.blend_channel(dst_g, src_g);
    let blended_b = mode.blend_channel(dst_b, src_b);

    // Now composite the blended color over destination using porter-duff "source over"
    // out_alpha = src_alpha + dst_alpha * (1 - src_alpha)
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

    if out_alpha == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }

    // Composite each channel:
    // out_color = (blended_color * src_alpha + dst_color * dst_alpha * (1 - src_alpha)) / out_alpha
    let composite = |blended: f32, dst: f32| -> u8 {
        let result = (blended * src_alpha + dst * dst_alpha * (1.0 - src_alpha)) / out_alpha;
        (result.clamp(0.0, 1.0) * 255.0).round() as u8
    };

    Rgba([
        composite(blended_r, dst_r),
        composite(blended_g, dst_g),
        composite(blended_b, dst_b),
        (out_alpha * 255.0).round() as u8,
    ])
}
