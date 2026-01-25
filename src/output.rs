//! PNG output and file path generation

use image::imageops::FilterType;
use image::RgbaImage;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error type for output operations
#[derive(Debug, Error)]
pub enum OutputError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// Image encoding error
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}

/// Save an RGBA image to a PNG file.
///
/// # Arguments
///
/// * `image` - The image to save
/// * `path` - The output file path
///
/// # Returns
///
/// * `Ok(())` on success
/// * `Err(OutputError)` on failure
pub fn save_png(image: &RgbaImage, path: &Path) -> Result<(), OutputError> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    image.save(path)?;
    Ok(())
}

/// Scale image by integer factor using nearest-neighbor interpolation.
///
/// This preserves crisp pixel edges for pixel art.
///
/// # Arguments
///
/// * `image` - The image to scale
/// * `factor` - Scale factor (1-16, where 1 means no scaling)
///
/// # Returns
///
/// The scaled image (or original if factor is 1)
pub fn scale_image(image: RgbaImage, factor: u8) -> RgbaImage {
    if factor <= 1 {
        return image;
    }
    let (w, h) = image.dimensions();
    let new_w = w * factor as u32;
    let new_h = h * factor as u32;
    image::imageops::resize(&image, new_w, new_h, FilterType::Nearest)
}

/// Apply a skew transform along the X axis (horizontal shear).
///
/// Positive angles skew the top edge to the right. Uses nearest-neighbor
/// sampling to preserve crisp pixel edges for pixel art.
///
/// # Arguments
///
/// * `image` - The image to transform
/// * `degrees` - Skew angle in degrees (must be between -89 and 89)
///
/// # Returns
///
/// A new image with the skew applied. The canvas expands to fit the skewed content.
pub fn skew_x(image: &RgbaImage, degrees: f32) -> RgbaImage {
    if degrees.abs() < 0.001 {
        return image.clone();
    }

    let (w, h) = image.dimensions();
    let h_f = h as f32;

    // Calculate the horizontal shift at the top/bottom of the image
    let tan_angle = (degrees * std::f32::consts::PI / 180.0).tan();
    let max_shift = (h_f * tan_angle.abs()).ceil() as u32;

    // New canvas width includes the shift
    let new_w = w + max_shift;
    let new_h = h;

    let mut output = RgbaImage::from_pixel(new_w, new_h, image::Rgba([0, 0, 0, 0]));

    for y in 0..h {
        // Calculate horizontal offset for this row
        // For positive angles: top rows shift right, bottom rows stay
        // For negative angles: top rows stay, bottom rows shift right
        let y_ratio = y as f32 / h_f;
        let x_offset = if degrees > 0.0 {
            ((1.0 - y_ratio) * h_f * tan_angle).round() as i32
        } else {
            (y_ratio * h_f * tan_angle.abs()).round() as i32
        };

        for x in 0..w {
            let src_pixel = image.get_pixel(x, y);
            let new_x = (x as i32 + x_offset) as u32;
            if new_x < new_w {
                output.put_pixel(new_x, y, *src_pixel);
            }
        }
    }

    output
}

/// Apply a skew transform along the Y axis (vertical shear).
///
/// Positive angles skew the left edge downward. Uses nearest-neighbor
/// sampling to preserve crisp pixel edges for pixel art.
///
/// # Arguments
///
/// * `image` - The image to transform
/// * `degrees` - Skew angle in degrees (must be between -89 and 89)
///
/// # Returns
///
/// A new image with the skew applied. The canvas expands to fit the skewed content.
pub fn skew_y(image: &RgbaImage, degrees: f32) -> RgbaImage {
    if degrees.abs() < 0.001 {
        return image.clone();
    }

    let (w, h) = image.dimensions();
    let w_f = w as f32;

    // Calculate the vertical shift at the left/right of the image
    let tan_angle = (degrees * std::f32::consts::PI / 180.0).tan();
    let max_shift = (w_f * tan_angle.abs()).ceil() as u32;

    // New canvas height includes the shift
    let new_w = w;
    let new_h = h + max_shift;

    let mut output = RgbaImage::from_pixel(new_w, new_h, image::Rgba([0, 0, 0, 0]));

    for x in 0..w {
        // Calculate vertical offset for this column
        // For positive angles: left columns shift down, right columns stay
        // For negative angles: left columns stay, right columns shift down
        let x_ratio = x as f32 / w_f;
        let y_offset = if degrees > 0.0 {
            ((1.0 - x_ratio) * w_f * tan_angle).round() as i32
        } else {
            (x_ratio * w_f * tan_angle.abs()).round() as i32
        };

        for y in 0..h {
            let src_pixel = image.get_pixel(x, y);
            let new_y = (y as i32 + y_offset) as u32;
            if new_y < new_h {
                output.put_pixel(x, new_y, *src_pixel);
            }
        }
    }

    output
}

/// Generate the output path for a sprite.
///
/// # Output Naming Rules (from spec)
///
/// | Scenario | Output |
/// |----------|--------|
/// | Single sprite "hero" | `input_hero.png` |
/// | Multiple sprites | `input_{name}.png` for each |
/// | With `-o output.png` (single sprite) | `output.png` |
/// | With `-o output.png` (multiple) | `output_{name}.png` |
/// | With `-o dir/` | `dir/{name}.png` |
///
/// # Arguments
///
/// * `input` - The input file path (used for default naming)
/// * `sprite_name` - The name of the sprite being saved
/// * `output_arg` - The `-o` argument value, if provided
/// * `is_single_sprite` - Whether there's only one sprite in the input
///
/// # Returns
///
/// The path where the sprite should be saved
pub fn generate_output_path(
    input: &Path,
    sprite_name: &str,
    output_arg: Option<&Path>,
    is_single_sprite: bool,
) -> PathBuf {
    match output_arg {
        Some(output) => {
            // Check if output is a directory (ends with / or is existing directory)
            let is_dir = output.as_os_str().to_string_lossy().ends_with('/') || output.is_dir();

            if is_dir {
                // -o dir/ → dir/{name}.png
                output.join(format!("{}.png", sprite_name))
            } else if is_single_sprite {
                // -o output.png (single sprite) → output.png
                output.to_path_buf()
            } else {
                // -o output.png (multiple) → output_{name}.png
                let stem = output.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                let parent = output.parent().unwrap_or(Path::new(""));
                if parent.as_os_str().is_empty() {
                    PathBuf::from(format!("{}_{}.png", stem, sprite_name))
                } else {
                    parent.join(format!("{}_{}.png", stem, sprite_name))
                }
            }
        }
        None => {
            // Default: {input_stem}_{sprite_name}.png
            let input_stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let parent = input.parent().unwrap_or(Path::new(""));
            if parent.as_os_str().is_empty() {
                PathBuf::from(format!("{}_{}.png", input_stem, sprite_name))
            } else {
                parent.join(format!("{}_{}.png", input_stem, sprite_name))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_generate_output_path_default_single() {
        // Single sprite, no -o argument
        let path = generate_output_path(Path::new("input.jsonl"), "hero", None, true);
        assert_eq!(path, PathBuf::from("input_hero.png"));
    }

    #[test]
    fn test_generate_output_path_default_multiple() {
        // Multiple sprites, no -o argument
        let path1 = generate_output_path(Path::new("input.jsonl"), "hero", None, false);
        let path2 = generate_output_path(Path::new("input.jsonl"), "enemy", None, false);
        assert_eq!(path1, PathBuf::from("input_hero.png"));
        assert_eq!(path2, PathBuf::from("input_enemy.png"));
    }

    #[test]
    fn test_generate_output_path_explicit_file_single() {
        // Single sprite with -o output.png
        let path = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            Some(Path::new("output.png")),
            true,
        );
        assert_eq!(path, PathBuf::from("output.png"));
    }

    #[test]
    fn test_generate_output_path_explicit_file_multiple() {
        // Multiple sprites with -o output.png
        let path1 = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            Some(Path::new("output.png")),
            false,
        );
        let path2 = generate_output_path(
            Path::new("input.jsonl"),
            "enemy",
            Some(Path::new("output.png")),
            false,
        );
        assert_eq!(path1, PathBuf::from("output_hero.png"));
        assert_eq!(path2, PathBuf::from("output_enemy.png"));
    }

    #[test]
    fn test_generate_output_path_directory() {
        // -o dir/ (trailing slash)
        let path = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            Some(Path::new("outdir/")),
            true,
        );
        assert_eq!(path, PathBuf::from("outdir/hero.png"));
    }

    #[test]
    fn test_generate_output_path_directory_multiple() {
        // -o dir/ with multiple sprites
        let path1 = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            Some(Path::new("sprites/")),
            false,
        );
        let path2 = generate_output_path(
            Path::new("input.jsonl"),
            "enemy",
            Some(Path::new("sprites/")),
            false,
        );
        assert_eq!(path1, PathBuf::from("sprites/hero.png"));
        assert_eq!(path2, PathBuf::from("sprites/enemy.png"));
    }

    #[test]
    fn test_generate_output_path_nested_input() {
        // Input in subdirectory
        let path =
            generate_output_path(Path::new("assets/sprites/input.jsonl"), "hero", None, true);
        assert_eq!(path, PathBuf::from("assets/sprites/input_hero.png"));
    }

    #[test]
    fn test_generate_output_path_nested_output() {
        // Output to nested directory
        let path = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            Some(Path::new("build/sprites/output.png")),
            true,
        );
        assert_eq!(path, PathBuf::from("build/sprites/output.png"));
    }

    #[test]
    fn test_generate_output_path_nested_output_multiple() {
        // Output to nested directory with multiple sprites
        let path = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            Some(Path::new("build/sprites/output.png")),
            false,
        );
        assert_eq!(path, PathBuf::from("build/sprites/output_hero.png"));
    }

    #[test]
    fn test_save_png_basic() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.png");

        // Create a simple 2x2 image
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Red
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // Green
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255])); // Blue
        image.put_pixel(1, 1, Rgba([0, 0, 0, 0])); // Transparent

        let result = save_png(&image, &path);
        assert!(result.is_ok());
        assert!(path.exists());

        // Read it back and verify
        let loaded = image::open(&path).unwrap().to_rgba8();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        assert_eq!(*loaded.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*loaded.get_pixel(1, 0), Rgba([0, 255, 0, 255]));
        assert_eq!(*loaded.get_pixel(0, 1), Rgba([0, 0, 255, 255]));
        assert_eq!(*loaded.get_pixel(1, 1), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_save_png_creates_parent_dirs() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/dirs/test.png");

        let image = RgbaImage::new(1, 1);
        let result = save_png(&image, &path);

        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_scale_image_factor_one_returns_original() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        image.put_pixel(1, 1, Rgba([255, 255, 0, 255]));

        let scaled = scale_image(image, 1);

        assert_eq!(scaled.width(), 2);
        assert_eq!(scaled.height(), 2);
        // Verify pixels unchanged
        assert_eq!(*scaled.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*scaled.get_pixel(1, 0), Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn test_scale_image_factor_zero_returns_original() {
        let image = RgbaImage::new(3, 3);
        let scaled = scale_image(image, 0);

        // factor <= 1 returns original
        assert_eq!(scaled.width(), 3);
        assert_eq!(scaled.height(), 3);
    }

    #[test]
    fn test_scale_image_factor_two() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Red
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // Green
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255])); // Blue
        image.put_pixel(1, 1, Rgba([255, 255, 0, 255])); // Yellow

        let scaled = scale_image(image, 2);

        // 2x2 scaled by 2 = 4x4
        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);

        // Each original pixel becomes a 2x2 block
        // Red at (0,0) should fill (0,0), (1,0), (0,1), (1,1)
        assert_eq!(*scaled.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*scaled.get_pixel(1, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*scaled.get_pixel(0, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*scaled.get_pixel(1, 1), Rgba([255, 0, 0, 255]));

        // Green at (1,0) should fill (2,0), (3,0), (2,1), (3,1)
        assert_eq!(*scaled.get_pixel(2, 0), Rgba([0, 255, 0, 255]));
        assert_eq!(*scaled.get_pixel(3, 0), Rgba([0, 255, 0, 255]));

        // Blue at (0,1) should fill (0,2), (1,2), (0,3), (1,3)
        assert_eq!(*scaled.get_pixel(0, 2), Rgba([0, 0, 255, 255]));
        assert_eq!(*scaled.get_pixel(1, 3), Rgba([0, 0, 255, 255]));

        // Yellow at (1,1) should fill (2,2), (3,2), (2,3), (3,3)
        assert_eq!(*scaled.get_pixel(3, 3), Rgba([255, 255, 0, 255]));
    }

    #[test]
    fn test_scale_image_factor_four() {
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([128, 64, 32, 200]));

        let scaled = scale_image(image, 4);

        // 1x1 scaled by 4 = 4x4
        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 4);

        // All pixels should be the same color
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(
                    *scaled.get_pixel(x, y),
                    Rgba([128, 64, 32, 200]),
                    "Pixel at ({}, {}) should match original",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_scale_image_preserves_transparency() {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Opaque red
        image.put_pixel(1, 0, Rgba([0, 0, 0, 0])); // Transparent

        let scaled = scale_image(image, 2);

        assert_eq!(scaled.width(), 4);
        assert_eq!(scaled.height(), 2);

        // Opaque red should remain opaque
        assert_eq!(*scaled.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*scaled.get_pixel(1, 1), Rgba([255, 0, 0, 255]));

        // Transparent should remain transparent
        assert_eq!(*scaled.get_pixel(2, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*scaled.get_pixel(3, 1), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_scale_image_large_factor() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([100, 100, 100, 255]));
        image.put_pixel(1, 0, Rgba([200, 200, 200, 255]));
        image.put_pixel(0, 1, Rgba([50, 50, 50, 255]));
        image.put_pixel(1, 1, Rgba([150, 150, 150, 255]));

        let scaled = scale_image(image, 8);

        // 2x2 scaled by 8 = 16x16
        assert_eq!(scaled.width(), 16);
        assert_eq!(scaled.height(), 16);

        // Spot check corners of each quadrant
        assert_eq!(*scaled.get_pixel(0, 0), Rgba([100, 100, 100, 255]));
        assert_eq!(*scaled.get_pixel(7, 7), Rgba([100, 100, 100, 255]));
        assert_eq!(*scaled.get_pixel(8, 0), Rgba([200, 200, 200, 255]));
        assert_eq!(*scaled.get_pixel(15, 7), Rgba([200, 200, 200, 255]));
        assert_eq!(*scaled.get_pixel(0, 8), Rgba([50, 50, 50, 255]));
        assert_eq!(*scaled.get_pixel(8, 8), Rgba([150, 150, 150, 255]));
    }

    #[test]
    fn test_skew_x_zero_angle() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        image.put_pixel(1, 1, Rgba([255, 255, 0, 255]));

        let skewed = skew_x(&image, 0.0);

        // Zero angle should return identical image
        assert_eq!(skewed.width(), 2);
        assert_eq!(skewed.height(), 2);
        assert_eq!(*skewed.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*skewed.get_pixel(1, 0), Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn test_skew_x_positive_angle() {
        let mut image = RgbaImage::new(2, 4);
        // Fill with red for visibility
        for y in 0..4 {
            for x in 0..2 {
                image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let skewed = skew_x(&image, 45.0);

        // Canvas should expand horizontally
        assert!(skewed.width() > 2);
        assert_eq!(skewed.height(), 4);

        // Top row should be shifted right
        // Bottom row should stay near original position
    }

    #[test]
    fn test_skew_y_zero_angle() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        image.put_pixel(1, 1, Rgba([255, 255, 0, 255]));

        let skewed = skew_y(&image, 0.0);

        // Zero angle should return identical image
        assert_eq!(skewed.width(), 2);
        assert_eq!(skewed.height(), 2);
        assert_eq!(*skewed.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*skewed.get_pixel(1, 0), Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn test_skew_y_positive_angle() {
        let mut image = RgbaImage::new(4, 2);
        // Fill with red for visibility
        for y in 0..2 {
            for x in 0..4 {
                image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let skewed = skew_y(&image, 45.0);

        // Canvas should expand vertically
        assert_eq!(skewed.width(), 4);
        assert!(skewed.height() > 2);

        // Left column should be shifted down
        // Right column should stay near original position
    }

    #[test]
    fn test_skew_x_preserves_transparency() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Opaque
        image.put_pixel(1, 0, Rgba([0, 0, 0, 0])); // Transparent
        image.put_pixel(0, 1, Rgba([0, 255, 0, 128])); // Semi-transparent
        image.put_pixel(1, 1, Rgba([0, 0, 255, 255])); // Opaque

        let skewed = skew_x(&image, 20.0);

        // Check that we still have transparent and semi-transparent pixels somewhere
        let mut has_transparent = false;
        let mut has_semi = false;
        for y in 0..skewed.height() {
            for x in 0..skewed.width() {
                let p = skewed.get_pixel(x, y);
                if p[3] == 0 {
                    has_transparent = true;
                }
                if p[3] > 0 && p[3] < 255 {
                    has_semi = true;
                }
            }
        }
        assert!(has_transparent, "Should preserve transparent pixels");
        assert!(has_semi, "Should preserve semi-transparent pixels");
    }
}
