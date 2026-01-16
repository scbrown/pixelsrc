//! Onion skinning for animation preview.
//!
//! Renders animation frames with previous/next frames as transparent overlays,
//! a technique used by animators to see motion continuity.

use image::{Rgba, RgbaImage};

/// Configuration for onion skin rendering.
#[derive(Debug, Clone)]
pub struct OnionConfig {
    /// Number of frames before/after to show
    pub count: u32,
    /// Base opacity for ghost frames (0.0-1.0)
    pub opacity: f32,
    /// Tint color for previous frames (RGBA)
    pub prev_color: Rgba<u8>,
    /// Tint color for next frames (RGBA)
    pub next_color: Rgba<u8>,
    /// Whether to fade opacity for distant frames
    pub fade: bool,
}

impl Default for OnionConfig {
    fn default() -> Self {
        Self {
            count: 2,
            opacity: 0.3,
            prev_color: Rgba([0, 0, 255, 255]), // Blue
            next_color: Rgba([0, 255, 0, 255]), // Green
            fade: false,
        }
    }
}

/// Calculate opacity for a ghost frame based on distance from current frame.
///
/// With fade enabled:
/// - Distance 1: 100% of base opacity
/// - Distance 2: 67% of base opacity
/// - Distance 3: 33% of base opacity
fn calculate_opacity(base_opacity: f32, distance: u32, fade: bool) -> f32 {
    if !fade || distance == 0 {
        return base_opacity;
    }

    // Fade formula: opacity decreases as distance increases
    // At distance 1: 100%, distance 2: 67%, distance 3: 33%
    let fade_factor = 1.0 - ((distance - 1) as f32 / 3.0);
    base_opacity * fade_factor.max(0.0)
}

/// Apply a tint color to an image with specified opacity.
///
/// The tint is blended multiplicatively with the original colors.
fn apply_tint(image: &RgbaImage, tint: Rgba<u8>, opacity: f32) -> RgbaImage {
    let mut result = image.clone();
    let opacity_u8 = (opacity * 255.0).clamp(0.0, 255.0) as u8;

    for pixel in result.pixels_mut() {
        if pixel[3] > 0 {
            // Blend tint with original color (multiplicative blend)
            let r = ((pixel[0] as f32 * tint[0] as f32) / 255.0) as u8;
            let g = ((pixel[1] as f32 * tint[1] as f32) / 255.0) as u8;
            let b = ((pixel[2] as f32 * tint[2] as f32) / 255.0) as u8;
            // Scale alpha by opacity
            let a = ((pixel[3] as f32 * opacity_u8 as f32) / 255.0) as u8;
            *pixel = Rgba([r, g, b, a]);
        }
    }

    result
}

/// Composite a source image over a destination image with alpha blending.
fn composite_over(dest: &mut RgbaImage, src: &RgbaImage) {
    if dest.dimensions() != src.dimensions() {
        return;
    }

    for (dest_pixel, src_pixel) in dest.pixels_mut().zip(src.pixels()) {
        if src_pixel[3] == 0 {
            continue;
        }

        let src_a = src_pixel[3] as f32 / 255.0;
        let dest_a = dest_pixel[3] as f32 / 255.0;

        // Standard alpha compositing (over operation)
        let out_a = src_a + dest_a * (1.0 - src_a);

        if out_a > 0.0 {
            let blend = |s: u8, d: u8| -> u8 {
                let sf = s as f32 / 255.0;
                let df = d as f32 / 255.0;
                let out = (sf * src_a + df * dest_a * (1.0 - src_a)) / out_a;
                (out * 255.0).clamp(0.0, 255.0) as u8
            };

            dest_pixel[0] = blend(src_pixel[0], dest_pixel[0]);
            dest_pixel[1] = blend(src_pixel[1], dest_pixel[1]);
            dest_pixel[2] = blend(src_pixel[2], dest_pixel[2]);
            dest_pixel[3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Render an animation frame with onion skinning.
///
/// # Arguments
///
/// * `frames` - All animation frame images
/// * `current_frame` - Index of the current frame to display
/// * `config` - Onion skin configuration
///
/// # Returns
///
/// A new image with the current frame and ghost overlays.
pub fn render_onion_skin(
    frames: &[RgbaImage],
    current_frame: usize,
    config: &OnionConfig,
) -> RgbaImage {
    if frames.is_empty() {
        return RgbaImage::new(1, 1);
    }

    let current_frame = current_frame.min(frames.len() - 1);
    let current = &frames[current_frame];

    // Start with transparent background
    let mut result = RgbaImage::from_pixel(current.width(), current.height(), Rgba([0, 0, 0, 0]));

    // Render previous frames (farthest first, so closer ones are on top)
    for i in (1..=config.count).rev() {
        let frame_idx = current_frame as i32 - i as i32;
        if frame_idx >= 0 {
            let opacity = calculate_opacity(config.opacity, i, config.fade);
            let tinted = apply_tint(&frames[frame_idx as usize], config.prev_color, opacity);
            composite_over(&mut result, &tinted);
        }
    }

    // Render next frames (farthest first)
    for i in (1..=config.count).rev() {
        let frame_idx = current_frame + i as usize;
        if frame_idx < frames.len() {
            let opacity = calculate_opacity(config.opacity, i, config.fade);
            let tinted = apply_tint(&frames[frame_idx], config.next_color, opacity);
            composite_over(&mut result, &tinted);
        }
    }

    // Render current frame on top (full opacity, no tint)
    composite_over(&mut result, current);

    result
}

/// Parse a hex color string to RGBA.
///
/// Accepts formats: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
pub fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    match hex.len() {
        3 => {
            // #RGB -> #RRGGBB
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(Rgba([r, g, b, 255]))
        }
        4 => {
            // #RGBA -> #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
            Some(Rgba([r, g, b, a]))
        }
        6 => {
            // #RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Rgba([r, g, b, 255]))
        }
        8 => {
            // #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Rgba([r, g, b, a]))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_6digit() {
        assert_eq!(parse_hex_color("#FF0000"), Some(Rgba([255, 0, 0, 255])));
        assert_eq!(parse_hex_color("#00FF00"), Some(Rgba([0, 255, 0, 255])));
        assert_eq!(parse_hex_color("#0000FF"), Some(Rgba([0, 0, 255, 255])));
        assert_eq!(parse_hex_color("FFFFFF"), Some(Rgba([255, 255, 255, 255])));
    }

    #[test]
    fn test_parse_hex_color_3digit() {
        assert_eq!(parse_hex_color("#F00"), Some(Rgba([255, 0, 0, 255])));
        assert_eq!(parse_hex_color("#0F0"), Some(Rgba([0, 255, 0, 255])));
        assert_eq!(parse_hex_color("#00F"), Some(Rgba([0, 0, 255, 255])));
    }

    #[test]
    fn test_parse_hex_color_8digit() {
        assert_eq!(parse_hex_color("#FF000080"), Some(Rgba([255, 0, 0, 128])));
        assert_eq!(parse_hex_color("#00FF00FF"), Some(Rgba([0, 255, 0, 255])));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#GG0000"), None);
        assert_eq!(parse_hex_color("#12345"), None);
    }

    #[test]
    fn test_calculate_opacity_no_fade() {
        let base = 0.3;
        assert_eq!(calculate_opacity(base, 1, false), 0.3);
        assert_eq!(calculate_opacity(base, 2, false), 0.3);
        assert_eq!(calculate_opacity(base, 3, false), 0.3);
    }

    #[test]
    fn test_calculate_opacity_with_fade() {
        let base = 0.3;
        // Distance 1: 100%
        assert!((calculate_opacity(base, 1, true) - 0.3).abs() < 0.001);
        // Distance 2: ~67%
        assert!((calculate_opacity(base, 2, true) - 0.2).abs() < 0.001);
        // Distance 3: ~33%
        assert!((calculate_opacity(base, 3, true) - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_onion_config_default() {
        let config = OnionConfig::default();
        assert_eq!(config.count, 2);
        assert!((config.opacity - 0.3).abs() < 0.001);
        assert_eq!(config.prev_color, Rgba([0, 0, 255, 255]));
        assert_eq!(config.next_color, Rgba([0, 255, 0, 255]));
        assert!(!config.fade);
    }

    #[test]
    fn test_render_onion_skin_single_frame() {
        let frame = RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255]));
        let frames = vec![frame];
        let config = OnionConfig::default();

        let result = render_onion_skin(&frames, 0, &config);

        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
        // Current frame should be visible
        assert_eq!(*result.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_render_onion_skin_empty_frames() {
        let frames: Vec<RgbaImage> = vec![];
        let config = OnionConfig::default();

        let result = render_onion_skin(&frames, 0, &config);

        // Should return a 1x1 empty image
        assert_eq!(result.width(), 1);
        assert_eq!(result.height(), 1);
    }

    #[test]
    fn test_render_onion_skin_with_ghosts() {
        // Create 3 frames: red, green, blue
        let frames = vec![
            RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255])),
            RgbaImage::from_pixel(4, 4, Rgba([0, 255, 0, 255])),
            RgbaImage::from_pixel(4, 4, Rgba([0, 0, 255, 255])),
        ];
        let config = OnionConfig {
            count: 1,
            opacity: 0.5,
            prev_color: Rgba([255, 255, 255, 255]), // White tint (no color change)
            next_color: Rgba([255, 255, 255, 255]), // White tint
            fade: false,
        };

        let result = render_onion_skin(&frames, 1, &config);

        // Result should be a blend of red (prev), green (current), blue (next)
        // Current frame (green) should be most prominent
        let pixel = result.get_pixel(0, 0);
        assert!(pixel[1] > pixel[0]); // More green than red (green is current frame at full opacity)
        assert!(pixel[1] > pixel[2]); // More green than blue
    }

    #[test]
    fn test_apply_tint() {
        let image = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let tint = Rgba([255, 0, 0, 255]); // Red tint
        let opacity = 0.5;

        let result = apply_tint(&image, tint, opacity);

        // White * red tint = red, at 50% opacity
        let pixel = result.get_pixel(0, 0);
        assert_eq!(pixel[0], 255); // Full red
        assert_eq!(pixel[1], 0);   // No green
        assert_eq!(pixel[2], 0);   // No blue
        assert_eq!(pixel[3], 127); // ~50% alpha
    }

    #[test]
    fn test_composite_over_transparent_src() {
        let mut dest = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        let src = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 0])); // Fully transparent

        composite_over(&mut dest, &src);

        // Dest should be unchanged
        assert_eq!(*dest.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_composite_over_opaque_src() {
        let mut dest = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        let src = RgbaImage::from_pixel(2, 2, Rgba([0, 255, 0, 255])); // Fully opaque green

        composite_over(&mut dest, &src);

        // Src should completely cover dest
        assert_eq!(*dest.get_pixel(0, 0), Rgba([0, 255, 0, 255]));
    }
}
