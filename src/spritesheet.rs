//! Spritesheet rendering - combines multiple frames into a grid layout

use image::{Rgba, RgbaImage};

/// Transparent color used for padding
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Render multiple frames into a spritesheet grid.
///
/// # Arguments
///
/// * `frames` - Slice of RGBA images to combine
/// * `cols` - Optional number of columns. If None, uses horizontal layout (1 row)
///
/// # Returns
///
/// A single RGBA image containing all frames arranged in a grid.
/// All frames are padded to match the largest frame dimensions.
///
/// # Examples
///
/// ```
/// use image::RgbaImage;
/// use pixelsrc::spritesheet::render_spritesheet;
///
/// // Create 4 simple 2x2 frames
/// let frame = RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
/// let frames = vec![frame.clone(), frame.clone(), frame.clone(), frame.clone()];
///
/// // Default: horizontal layout (4x1 grid)
/// let sheet = render_spritesheet(&frames, None);
/// assert_eq!(sheet.width(), 8);  // 4 frames * 2 pixels
/// assert_eq!(sheet.height(), 2);
///
/// // With cols=2: 2x2 grid
/// let sheet = render_spritesheet(&frames, Some(2));
/// assert_eq!(sheet.width(), 4);  // 2 cols * 2 pixels
/// assert_eq!(sheet.height(), 4); // 2 rows * 2 pixels
/// ```
pub fn render_spritesheet(frames: &[RgbaImage], cols: Option<u32>) -> RgbaImage {
    if frames.is_empty() {
        return RgbaImage::from_pixel(1, 1, TRANSPARENT);
    }

    // Find the maximum dimensions across all frames
    let max_width = frames.iter().map(|f| f.width()).max().unwrap_or(1);
    let max_height = frames.iter().map(|f| f.height()).max().unwrap_or(1);

    // Calculate grid dimensions
    let num_frames = frames.len() as u32;
    let columns = cols.unwrap_or(num_frames); // Default: horizontal layout (all in one row)
    let rows = num_frames.div_ceil(columns);

    // Create output image
    let sheet_width = columns * max_width;
    let sheet_height = rows * max_height;
    let mut sheet = RgbaImage::from_pixel(sheet_width, sheet_height, TRANSPARENT);

    // Copy each frame to its position in the grid
    for (i, frame) in frames.iter().enumerate() {
        let col = (i as u32) % columns;
        let row = (i as u32) / columns;

        let dest_x = col * max_width;
        let dest_y = row * max_height;

        // Copy frame pixels (centered if smaller than max dimensions)
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let pixel = *frame.get_pixel(x, y);
                sheet.put_pixel(dest_x + x, dest_y + y, pixel);
            }
        }
        // Remaining pixels stay transparent (default from from_pixel)
    }

    sheet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solid_frame(width: u32, height: u32, color: Rgba<u8>) -> RgbaImage {
        RgbaImage::from_pixel(width, height, color)
    }

    #[test]
    fn test_empty_frames() {
        let sheet = render_spritesheet(&[], None);
        assert_eq!(sheet.width(), 1);
        assert_eq!(sheet.height(), 1);
        assert_eq!(*sheet.get_pixel(0, 0), TRANSPARENT);
    }

    #[test]
    fn test_single_frame() {
        let red = Rgba([255, 0, 0, 255]);
        let frame = make_solid_frame(3, 3, red);
        let sheet = render_spritesheet(&[frame], None);

        assert_eq!(sheet.width(), 3);
        assert_eq!(sheet.height(), 3);
        assert_eq!(*sheet.get_pixel(0, 0), red);
        assert_eq!(*sheet.get_pixel(2, 2), red);
    }

    #[test]
    fn test_four_frames_horizontal() {
        // 4 frames → 4x1 spritesheet (default horizontal layout)
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);
        let yellow = Rgba([255, 255, 0, 255]);

        let frames = vec![
            make_solid_frame(2, 2, red),
            make_solid_frame(2, 2, green),
            make_solid_frame(2, 2, blue),
            make_solid_frame(2, 2, yellow),
        ];

        let sheet = render_spritesheet(&frames, None);

        assert_eq!(sheet.width(), 8); // 4 frames * 2 pixels
        assert_eq!(sheet.height(), 2);

        // Check first pixel of each frame
        assert_eq!(*sheet.get_pixel(0, 0), red); // Frame 0 at (0,0)
        assert_eq!(*sheet.get_pixel(2, 0), green); // Frame 1 at (2,0)
        assert_eq!(*sheet.get_pixel(4, 0), blue); // Frame 2 at (4,0)
        assert_eq!(*sheet.get_pixel(6, 0), yellow); // Frame 3 at (6,0)
    }

    #[test]
    fn test_different_sized_frames_padded() {
        // Different sized frames → padded to largest
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);

        let small_frame = make_solid_frame(2, 2, red);
        let large_frame = make_solid_frame(4, 4, green);

        let frames = vec![small_frame, large_frame];
        let sheet = render_spritesheet(&frames, None);

        // Sheet should be 8x4 (2 cols * 4 max_width, 1 row * 4 max_height)
        assert_eq!(sheet.width(), 8);
        assert_eq!(sheet.height(), 4);

        // Frame 0 (small, 2x2): red in top-left 2x2, transparent padding
        assert_eq!(*sheet.get_pixel(0, 0), red);
        assert_eq!(*sheet.get_pixel(1, 1), red);
        assert_eq!(*sheet.get_pixel(2, 0), TRANSPARENT); // Padding
        assert_eq!(*sheet.get_pixel(0, 2), TRANSPARENT); // Padding

        // Frame 1 (large, 4x4): green fills the cell
        assert_eq!(*sheet.get_pixel(4, 0), green);
        assert_eq!(*sheet.get_pixel(7, 3), green);
    }

    #[test]
    fn test_custom_columns_2x2() {
        // Custom columns (cols=2) → 2x2 grid
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);
        let yellow = Rgba([255, 255, 0, 255]);

        let frames = vec![
            make_solid_frame(2, 2, red),
            make_solid_frame(2, 2, green),
            make_solid_frame(2, 2, blue),
            make_solid_frame(2, 2, yellow),
        ];

        let sheet = render_spritesheet(&frames, Some(2));

        assert_eq!(sheet.width(), 4); // 2 cols * 2 pixels
        assert_eq!(sheet.height(), 4); // 2 rows * 2 pixels

        // Row 0: red (0,0), green (2,0)
        assert_eq!(*sheet.get_pixel(0, 0), red);
        assert_eq!(*sheet.get_pixel(2, 0), green);

        // Row 1: blue (0,2), yellow (2,2)
        assert_eq!(*sheet.get_pixel(0, 2), blue);
        assert_eq!(*sheet.get_pixel(2, 2), yellow);
    }

    #[test]
    fn test_uneven_grid() {
        // 3 frames with cols=2 → 2 cols, 2 rows (with empty cell)
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);

        let frames = vec![
            make_solid_frame(2, 2, red),
            make_solid_frame(2, 2, green),
            make_solid_frame(2, 2, blue),
        ];

        let sheet = render_spritesheet(&frames, Some(2));

        assert_eq!(sheet.width(), 4); // 2 cols * 2 pixels
        assert_eq!(sheet.height(), 4); // 2 rows * 2 pixels

        // Row 0: red, green
        assert_eq!(*sheet.get_pixel(0, 0), red);
        assert_eq!(*sheet.get_pixel(2, 0), green);

        // Row 1: blue, transparent (empty cell)
        assert_eq!(*sheet.get_pixel(0, 2), blue);
        assert_eq!(*sheet.get_pixel(2, 2), TRANSPARENT);
    }

    #[test]
    fn test_cols_greater_than_frames() {
        // cols=10 with 3 frames → still 3x1 (cols capped at frame count effectively)
        let red = Rgba([255, 0, 0, 255]);

        let frames = vec![
            make_solid_frame(2, 2, red),
            make_solid_frame(2, 2, red),
            make_solid_frame(2, 2, red),
        ];

        let sheet = render_spritesheet(&frames, Some(10));

        // Width should still be 10 cols * 2 pixels = 20
        // But only 3 frames, rest is empty
        assert_eq!(sheet.width(), 20);
        assert_eq!(sheet.height(), 2);

        // First 3 cells have content
        assert_eq!(*sheet.get_pixel(0, 0), red);
        assert_eq!(*sheet.get_pixel(2, 0), red);
        assert_eq!(*sheet.get_pixel(4, 0), red);

        // Rest is transparent
        assert_eq!(*sheet.get_pixel(6, 0), TRANSPARENT);
    }

    #[test]
    fn test_single_column() {
        // cols=1 → vertical layout (Nx1 grid)
        let red = Rgba([255, 0, 0, 255]);
        let green = Rgba([0, 255, 0, 255]);
        let blue = Rgba([0, 0, 255, 255]);

        let frames = vec![
            make_solid_frame(3, 2, red),
            make_solid_frame(3, 2, green),
            make_solid_frame(3, 2, blue),
        ];

        let sheet = render_spritesheet(&frames, Some(1));

        assert_eq!(sheet.width(), 3); // 1 col * 3 pixels
        assert_eq!(sheet.height(), 6); // 3 rows * 2 pixels

        // Vertically stacked
        assert_eq!(*sheet.get_pixel(0, 0), red);
        assert_eq!(*sheet.get_pixel(0, 2), green);
        assert_eq!(*sheet.get_pixel(0, 4), blue);
    }
}
