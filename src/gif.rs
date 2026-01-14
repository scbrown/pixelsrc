//! GIF animation rendering

use crate::output::OutputError;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Frame, RgbaImage};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Render a sequence of frames as an animated GIF.
///
/// # Arguments
///
/// * `frames` - The image frames to include in the animation
/// * `duration_ms` - Duration per frame in milliseconds
/// * `loop_anim` - Whether the animation should loop infinitely
/// * `path` - Output file path
///
/// # Returns
///
/// * `Ok(())` on success
/// * `Err(OutputError)` on failure
pub fn render_gif(
    frames: &[RgbaImage],
    duration_ms: u32,
    loop_anim: bool,
    path: &Path,
) -> Result<(), OutputError> {
    if frames.is_empty() {
        return Ok(());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = GifEncoder::new(writer);

    // Set repeat behavior
    let repeat = if loop_anim {
        Repeat::Infinite
    } else {
        Repeat::Finite(0)
    };
    encoder.set_repeat(repeat)?;

    // GIF uses centiseconds (1/100th of a second) for delays
    // Convert milliseconds to centiseconds (divide by 10)
    let delay_cs = (duration_ms / 10).max(1) as u16;

    // Encode each frame
    for rgba_image in frames {
        let delay = image::Delay::from_numer_denom_ms(delay_cs as u32 * 10, 1);
        let frame = Frame::from_parts(rgba_image.clone(), 0, 0, delay);
        encoder.encode_frame(frame)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use tempfile::tempdir;

    /// Create a simple test frame with a solid color
    fn create_test_frame(width: u32, height: u32, color: Rgba<u8>) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = color;
        }
        img
    }

    #[test]
    fn test_render_gif_creates_valid_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.gif");

        let frames = vec![
            create_test_frame(2, 2, Rgba([255, 0, 0, 255])), // Red
            create_test_frame(2, 2, Rgba([0, 255, 0, 255])), // Green
        ];

        let result = render_gif(&frames, 100, true, &path);
        assert!(result.is_ok());
        assert!(path.exists());

        // Verify it's a valid GIF by reading it back
        let img = image::open(&path);
        assert!(img.is_ok());
    }

    #[test]
    fn test_render_gif_frame_duration() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("duration_test.gif");

        let frames = vec![
            create_test_frame(4, 4, Rgba([255, 255, 0, 255])), // Yellow
            create_test_frame(4, 4, Rgba([0, 255, 255, 255])), // Cyan
        ];

        // Use 500ms duration
        let result = render_gif(&frames, 500, true, &path);
        assert!(result.is_ok());
        assert!(path.exists());

        // GIF is created - duration is encoded in the file
        // We can verify the file exists and is valid
        let img = image::open(&path);
        assert!(img.is_ok());
    }

    #[test]
    fn test_render_gif_loop_setting() {
        let dir = tempdir().unwrap();

        // Test with loop=true
        let loop_path = dir.path().join("loop.gif");
        let frames = vec![
            create_test_frame(2, 2, Rgba([255, 0, 0, 255])),
            create_test_frame(2, 2, Rgba([0, 0, 255, 255])),
        ];
        let result = render_gif(&frames, 100, true, &loop_path);
        assert!(result.is_ok());
        assert!(loop_path.exists());

        // Test with loop=false
        let no_loop_path = dir.path().join("no_loop.gif");
        let result = render_gif(&frames, 100, false, &no_loop_path);
        assert!(result.is_ok());
        assert!(no_loop_path.exists());

        // Both files should be valid GIFs
        assert!(image::open(&loop_path).is_ok());
        assert!(image::open(&no_loop_path).is_ok());
    }

    #[test]
    fn test_render_gif_empty_frames() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.gif");

        let frames: Vec<RgbaImage> = vec![];
        let result = render_gif(&frames, 100, true, &path);

        // Should succeed but not create a file (nothing to write)
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_render_gif_single_frame() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("single.gif");

        let frames = vec![create_test_frame(8, 8, Rgba([128, 128, 128, 255]))];

        let result = render_gif(&frames, 100, true, &path);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_render_gif_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/dirs/test.gif");

        let frames = vec![create_test_frame(2, 2, Rgba([255, 0, 0, 255]))];

        let result = render_gif(&frames, 100, true, &path);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_render_gif_minimum_delay() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("min_delay.gif");

        let frames = vec![
            create_test_frame(2, 2, Rgba([255, 0, 0, 255])),
            create_test_frame(2, 2, Rgba([0, 255, 0, 255])),
        ];

        // Very small duration (should be clamped to minimum 10ms = 1 centisecond)
        let result = render_gif(&frames, 5, true, &path);
        assert!(result.is_ok());
        assert!(path.exists());
    }
}
