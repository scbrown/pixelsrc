//! Transform Command Demo Tests
//!
//! Demonstrates the `pxl transform` command functionality for applying
//! geometric and spatial transformations to sprites.

use image::RgbaImage;
use pixelsrc::transforms::{
    apply_image_transform, apply_image_transforms, explain_transform, parse_transform_str,
    Transform,
};

// ============================================================================
// Transform Parsing Tests
// ============================================================================
#[test]
fn test_parse_mirror_horizontal() {
    let transform = parse_transform_str("mirror-h").unwrap();
    assert_eq!(transform, Transform::MirrorH);
}

#[test]
fn test_parse_mirror_vertical() {
    let transform = parse_transform_str("mirror-v").unwrap();
    assert_eq!(transform, Transform::MirrorV);
}

#[test]
fn test_parse_rotate() {
    let t90 = parse_transform_str("rotate:90").unwrap();
    assert_eq!(t90, Transform::Rotate { degrees: 90 });

    let t180 = parse_transform_str("rotate:180").unwrap();
    assert_eq!(t180, Transform::Rotate { degrees: 180 });

    let t270 = parse_transform_str("rotate:270").unwrap();
    assert_eq!(t270, Transform::Rotate { degrees: 270 });
}

#[test]
fn test_parse_tile() {
    let transform = parse_transform_str("tile:2x3").unwrap();
    assert_eq!(transform, Transform::Tile { w: 2, h: 3 });
}

#[test]
fn test_parse_pad() {
    let transform = parse_transform_str("pad:4").unwrap();
    assert_eq!(transform, Transform::Pad { size: 4 });
}

#[test]
fn test_parse_crop() {
    let transform = parse_transform_str("crop:0,0,10,10").unwrap();
    assert_eq!(transform, Transform::Crop { x: 0, y: 0, w: 10, h: 10 });
}

#[test]
fn test_parse_shift() {
    let transform = parse_transform_str("shift:5,-3").unwrap();
    assert_eq!(transform, Transform::Shift { x: 5, y: -3 });
}

#[test]
fn test_parse_scale() {
    let transform = parse_transform_str("scale:2.0,1.5").unwrap();
    assert_eq!(transform, Transform::Scale { x: 2.0, y: 1.5 });
}

#[test]
fn test_parse_flip_aliases() {
    // flip-h is alias for mirror-h
    let transform = parse_transform_str("flip-h").unwrap();
    assert_eq!(transform, Transform::MirrorH);

    // flip-v is alias for mirror-v
    let transform = parse_transform_str("flip-v").unwrap();
    assert_eq!(transform, Transform::MirrorV);
}

// ============================================================================
// Transform Application Tests
// ============================================================================
/// Helper to create a simple test image (2x2 with distinct pixel colors)
fn create_test_image() -> RgbaImage {
    let mut img = RgbaImage::new(2, 2);
    img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255])); // Red top-left
    img.put_pixel(1, 0, image::Rgba([0, 255, 0, 255])); // Green top-right
    img.put_pixel(0, 1, image::Rgba([0, 0, 255, 255])); // Blue bottom-left
    img.put_pixel(1, 1, image::Rgba([255, 255, 0, 255])); // Yellow bottom-right
    img
}

#[test]
fn test_apply_mirror_horizontal() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::MirrorH, None).unwrap();

    // After horizontal mirror, left and right swap
    assert_eq!(result.get_pixel(0, 0), &image::Rgba([0, 255, 0, 255])); // Was top-right (green)
    assert_eq!(result.get_pixel(1, 0), &image::Rgba([255, 0, 0, 255])); // Was top-left (red)
}

#[test]
fn test_apply_mirror_vertical() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::MirrorV, None).unwrap();

    // After vertical mirror, top and bottom swap
    assert_eq!(result.get_pixel(0, 0), &image::Rgba([0, 0, 255, 255])); // Was bottom-left (blue)
    assert_eq!(result.get_pixel(0, 1), &image::Rgba([255, 0, 0, 255])); // Was top-left (red)
}

#[test]
fn test_apply_rotate_90() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::Rotate { degrees: 90 }, None).unwrap();

    // After 90° clockwise rotation:
    // - Top-left comes from bottom-left
    // - Top-right comes from top-left
    assert_eq!(result.get_pixel(0, 0), &image::Rgba([0, 0, 255, 255])); // Was bottom-left (blue)
    assert_eq!(result.get_pixel(1, 0), &image::Rgba([255, 0, 0, 255])); // Was top-left (red)
}

#[test]
fn test_apply_rotate_180() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::Rotate { degrees: 180 }, None).unwrap();

    // After 180° rotation, everything is flipped
    assert_eq!(result.get_pixel(0, 0), &image::Rgba([255, 255, 0, 255])); // Was bottom-right (yellow)
    assert_eq!(result.get_pixel(1, 1), &image::Rgba([255, 0, 0, 255])); // Was top-left (red)
}

#[test]
fn test_apply_tile() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::Tile { w: 2, h: 2 }, None).unwrap();

    // Tiling 2x2 creates 4x4 image
    assert_eq!(result.dimensions(), (4, 4));

    // Original pattern should repeat
    assert_eq!(result.get_pixel(0, 0), result.get_pixel(2, 0)); // Top-left repeats
    assert_eq!(result.get_pixel(0, 0), result.get_pixel(0, 2)); // And vertically
}

#[test]
fn test_apply_pad() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::Pad { size: 1 }, None).unwrap();

    // Padding by 1 on each side increases size by 2
    assert_eq!(result.dimensions(), (4, 4));

    // Original content should be in the center
    assert_eq!(result.get_pixel(1, 1), &image::Rgba([255, 0, 0, 255])); // Red at offset (1,1)

    // Edges should be transparent
    assert_eq!(result.get_pixel(0, 0), &image::Rgba([0, 0, 0, 0]));
}

#[test]
fn test_apply_crop() {
    // Create a 4x4 image
    let mut img = RgbaImage::new(4, 4);
    for y in 0..4 {
        for x in 0..4 {
            img.put_pixel(x, y, image::Rgba([((x + y) * 50) as u8, 0, 0, 255]));
        }
    }

    let result =
        apply_image_transform(&img, &Transform::Crop { x: 1, y: 1, w: 2, h: 2 }, None).unwrap();

    // Crop to 2x2 starting at (1,1)
    assert_eq!(result.dimensions(), (2, 2));
}

#[test]
fn test_apply_scale() {
    let img = create_test_image();
    let result = apply_image_transform(&img, &Transform::Scale { x: 2.0, y: 2.0 }, None).unwrap();

    // 2x scale doubles dimensions
    assert_eq!(result.dimensions(), (4, 4));
}

// ============================================================================
// Transform Chain Tests
// ============================================================================
#[test]
fn test_apply_multiple_transforms() {
    let img = create_test_image();
    let transforms = vec![Transform::MirrorH, Transform::Rotate { degrees: 90 }];

    let result = apply_image_transforms(&img, &transforms, None).unwrap();

    // Transforms are applied in order: first mirror, then rotate
    assert_eq!(result.dimensions(), (2, 2));
}

#[test]
fn test_transforms_chain_order_matters() {
    let img = create_test_image();

    // Mirror then rotate
    let result1 = apply_image_transforms(
        &img,
        &[Transform::MirrorH, Transform::Rotate { degrees: 90 }],
        None,
    )
    .unwrap();

    // Rotate then mirror
    let result2 = apply_image_transforms(
        &img,
        &[Transform::Rotate { degrees: 90 }, Transform::MirrorH],
        None,
    )
    .unwrap();

    // Results should be different
    let p1 = result1.get_pixel(0, 0);
    let p2 = result2.get_pixel(0, 0);
    assert_ne!(p1, p2, "Transform order should affect result");
}

// ============================================================================
// Transform Explanation Tests
// ============================================================================
#[test]
fn test_explain_transform_mirror() {
    let explanation = explain_transform(&Transform::MirrorH);
    assert!(explanation.contains("horizontally") || explanation.contains("Flip"));
}

#[test]
fn test_explain_transform_rotate() {
    let explanation = explain_transform(&Transform::Rotate { degrees: 90 });
    assert!(explanation.contains("90"));
    assert!(explanation.contains("Rotate") || explanation.contains("clockwise"));
}

#[test]
fn test_explain_transform_tile() {
    let explanation = explain_transform(&Transform::Tile { w: 3, h: 2 });
    assert!(explanation.contains("3") && explanation.contains("2"));
}

// ============================================================================
// Edge Cases
// ============================================================================
#[test]
fn test_transform_empty_chain() {
    let img = create_test_image();
    let result = apply_image_transforms(&img, &[], None).unwrap();

    // Empty transform list should return unchanged image
    assert_eq!(result.dimensions(), img.dimensions());
    assert_eq!(result.get_pixel(0, 0), img.get_pixel(0, 0));
}

#[test]
fn test_invalid_rotation_degrees() {
    let result = parse_transform_str("rotate:45");
    assert!(result.is_err(), "45° rotation should be invalid");
}

#[test]
fn test_crop_larger_than_image() {
    let img = create_test_image(); // 2x2
    let result =
        apply_image_transform(&img, &Transform::Crop { x: 0, y: 0, w: 10, h: 10 }, None).unwrap();

    // Crop should clamp to image bounds
    assert!(result.width() <= 2 && result.height() <= 2);
}
