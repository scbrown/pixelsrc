//! Emoji art output for terminal preview
//!
//! Maps colors to emoji based on hue/saturation/lightness for quick
//! terminal-based visualization of sprites.

use image::Rgba;

/// Available emoji for color mapping
const BLACK: &str = "⬛";
const WHITE: &str = "⬜";
const RED: &str = "🟥";
const ORANGE: &str = "🟧";
const YELLOW: &str = "🟨";
const GREEN: &str = "🟩";
const BLUE: &str = "🟦";
const PURPLE: &str = "🟪";
const BROWN: &str = "🟫";

/// Convert an RGBA color to the closest emoji representation.
///
/// # Color Mapping
///
/// - Transparent pixels (alpha < 128) -> ⬜ (white square, visually "empty")
/// - Black/very dark (lightness < 15%) -> ⬛
/// - White/very light (lightness > 85%) -> ⬜
/// - Low saturation (< 15%) -> grayscale (⬛ or ⬜ based on lightness)
/// - Hue-based mapping for saturated colors:
///   - Red: 0-15° or 345-360°
///   - Orange: 15-45°
///   - Yellow: 45-75°
///   - Green: 75-165°
///   - Blue: 165-255°
///   - Purple: 255-345°
///
/// # Examples
///
/// ```
/// use pixelsrc::emoji::color_to_emoji;
/// use image::Rgba;
///
/// // Red
/// assert_eq!(color_to_emoji(Rgba([255, 0, 0, 255])), "🟥");
///
/// // Transparent
/// assert_eq!(color_to_emoji(Rgba([255, 0, 0, 0])), "⬜");
///
/// // Black
/// assert_eq!(color_to_emoji(Rgba([0, 0, 0, 255])), "⬛");
/// ```
pub fn color_to_emoji(color: Rgba<u8>) -> &'static str {
    let [r, g, b, a] = color.0;

    // Transparent pixels render as white/empty
    if a < 128 {
        return WHITE;
    }

    // Convert to HSL for better color categorization
    let (h, s, l) = rgb_to_hsl(r, g, b);

    // Very dark colors -> black
    if l < 0.15 {
        return BLACK;
    }

    // Very light colors -> white
    if l > 0.85 {
        return WHITE;
    }

    // Low saturation -> grayscale
    if s < 0.15 {
        return if l < 0.5 { BLACK } else { WHITE };
    }

    // Map hue to emoji color
    // Brown is a special case: medium-low saturation + low-medium lightness + orange-ish hue
    // Brown is essentially "dark orange" - saturated orange/yellow hues with low lightness
    if (0.2..0.5).contains(&l) && (15.0..50.0).contains(&h) && s < 0.7 {
        return BROWN;
    }

    hue_to_emoji(h)
}

/// Map a hue value (0-360) to the closest emoji color.
fn hue_to_emoji(hue: f32) -> &'static str {
    // Normalize hue to 0-360 range
    let h = hue % 360.0;

    if !(15.0..345.0).contains(&h) {
        RED
    } else if h < 45.0 {
        ORANGE
    } else if h < 75.0 {
        YELLOW
    } else if h < 165.0 {
        GREEN
    } else if h < 255.0 {
        BLUE
    } else {
        PURPLE
    }
}

/// Convert RGB to HSL color space.
///
/// Returns (hue, saturation, lightness) where:
/// - hue is in degrees (0-360)
/// - saturation is 0.0-1.0
/// - lightness is 0.0-1.0
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Lightness
    let l = (max + min) / 2.0;

    // Saturation
    let s = if delta < f32::EPSILON { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs()) };

    // Hue
    let h = if delta < f32::EPSILON {
        0.0
    } else if (max - r).abs() < f32::EPSILON {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < f32::EPSILON {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    // Normalize hue to 0-360
    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, l)
}

/// Render an RGBA image to emoji art string.
///
/// Each pixel becomes one emoji character. Rows are separated by newlines.
pub fn render_emoji_art(image: &image::RgbaImage) -> String {
    let mut output = String::new();

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            output.push_str(color_to_emoji(*pixel));
        }
        output.push('\n');
    }

    output
}
