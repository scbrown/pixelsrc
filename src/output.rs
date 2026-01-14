//! PNG output and file path generation

use image::imageops::FilterType;
use image::RgbaImage;
use std::io;
use std::path::{Path, PathBuf};

/// Error type for output operations
#[derive(Debug)]
pub enum OutputError {
    /// IO error during file operations
    Io(io::Error),
    /// Image encoding error
    Image(image::ImageError),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::Io(e) => write!(f, "IO error: {}", e),
            OutputError::Image(e) => write!(f, "Image error: {}", e),
        }
    }
}

impl std::error::Error for OutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OutputError::Io(e) => Some(e),
            OutputError::Image(e) => Some(e),
        }
    }
}

impl From<io::Error> for OutputError {
    fn from(e: io::Error) -> Self {
        OutputError::Io(e)
    }
}

impl From<image::ImageError> for OutputError {
    fn from(e: image::ImageError) -> Self {
        OutputError::Image(e)
    }
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
            let is_dir = output.as_os_str().to_string_lossy().ends_with('/')
                || output.is_dir();

            if is_dir {
                // -o dir/ → dir/{name}.png
                output.join(format!("{}.png", sprite_name))
            } else if is_single_sprite {
                // -o output.png (single sprite) → output.png
                output.to_path_buf()
            } else {
                // -o output.png (multiple) → output_{name}.png
                let stem = output
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
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
            let input_stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
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
        let path = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            None,
            true,
        );
        assert_eq!(path, PathBuf::from("input_hero.png"));
    }

    #[test]
    fn test_generate_output_path_default_multiple() {
        // Multiple sprites, no -o argument
        let path1 = generate_output_path(
            Path::new("input.jsonl"),
            "hero",
            None,
            false,
        );
        let path2 = generate_output_path(
            Path::new("input.jsonl"),
            "enemy",
            None,
            false,
        );
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
        let path = generate_output_path(
            Path::new("assets/sprites/input.jsonl"),
            "hero",
            None,
            true,
        );
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
        image.put_pixel(1, 1, Rgba([0, 0, 0, 0]));     // Transparent

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
}
