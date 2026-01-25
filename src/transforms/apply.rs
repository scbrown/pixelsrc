//! Transform application functions for images and animations
//!
//! Provides functions to apply transforms to images and animation frame sequences.

use image::RgbaImage;

use super::anchor::scale_image;
use super::types::{Transform, TransformError};

/// Apply a single transform to an image.
///
/// Handles geometric and spatial transforms that operate on pixel data:
/// - MirrorH, MirrorV, Rotate
/// - Scale, SkewX, SkewY
/// - Tile, Pad, Crop, Shift
///
/// Animation transforms (Pingpong, Reverse, etc.) should use `apply_animation_transform` instead.
///
/// # Arguments
/// * `image` - The image to transform
/// * `transform` - The transform operation to apply
/// * `palette` - Optional palette for color-based transforms (outline, shadow, etc.)
///
/// # Returns
/// The transformed image, or an error if the transform cannot be applied
pub fn apply_image_transform(
    image: &RgbaImage,
    transform: &Transform,
    _palette: Option<&std::collections::HashMap<String, String>>,
) -> Result<RgbaImage, TransformError> {
    match transform {
        Transform::MirrorH => Ok(image::imageops::flip_horizontal(image)),
        Transform::MirrorV => Ok(image::imageops::flip_vertical(image)),
        Transform::Rotate { degrees } => match degrees {
            90 => Ok(image::imageops::rotate90(image)),
            180 => Ok(image::imageops::rotate180(image)),
            270 => Ok(image::imageops::rotate270(image)),
            _ => Err(TransformError::InvalidParameter {
                op: "rotate".to_string(),
                message: format!("invalid rotation: {}° (must be 90, 180, or 270)", degrees),
            }),
        },
        Transform::Scale { x, y } => Ok(scale_image(image, *x, *y)),
        Transform::SkewX { degrees } => Ok(crate::output::skew_x(image, *degrees)),
        Transform::SkewY { degrees } => Ok(crate::output::skew_y(image, *degrees)),
        Transform::Tile { w, h } => {
            let (img_w, img_h) = image.dimensions();
            let new_w = img_w * w;
            let new_h = img_h * h;
            let mut result = RgbaImage::new(new_w, new_h);
            for ty in 0..*h {
                for tx in 0..*w {
                    for y in 0..img_h {
                        for x in 0..img_w {
                            let pixel = *image.get_pixel(x, y);
                            result.put_pixel(tx * img_w + x, ty * img_h + y, pixel);
                        }
                    }
                }
            }
            Ok(result)
        }
        Transform::Pad { size } => {
            let (w, h) = image.dimensions();
            let new_w = w + size * 2;
            let new_h = h + size * 2;
            let mut result = RgbaImage::from_pixel(new_w, new_h, image::Rgba([0, 0, 0, 0]));
            for y in 0..h {
                for x in 0..w {
                    let pixel = *image.get_pixel(x, y);
                    result.put_pixel(x + size, y + size, pixel);
                }
            }
            Ok(result)
        }
        Transform::Crop { x, y, w, h } => {
            let (img_w, img_h) = image.dimensions();
            // Clamp crop region to image bounds
            let end_x = (*x + *w).min(img_w);
            let end_y = (*y + *h).min(img_h);
            let crop_w = end_x.saturating_sub(*x);
            let crop_h = end_y.saturating_sub(*y);
            if crop_w == 0 || crop_h == 0 {
                return Ok(RgbaImage::new(1, 1));
            }
            let mut result = RgbaImage::new(crop_w, crop_h);
            for dy in 0..crop_h {
                for dx in 0..crop_w {
                    let pixel = *image.get_pixel(*x + dx, *y + dy);
                    result.put_pixel(dx, dy, pixel);
                }
            }
            Ok(result)
        }
        Transform::Shift { x: dx, y: dy } => {
            let (w, h) = image.dimensions();
            let mut result = RgbaImage::from_pixel(w, h, image::Rgba([0, 0, 0, 0]));
            for y in 0..h {
                for x in 0..w {
                    let new_x = x as i32 + dx;
                    let new_y = y as i32 + dy;
                    if new_x >= 0 && new_x < w as i32 && new_y >= 0 && new_y < h as i32 {
                        let pixel = *image.get_pixel(x, y);
                        result.put_pixel(new_x as u32, new_y as u32, pixel);
                    }
                }
            }
            Ok(result)
        }
        // Animation transforms should use apply_animation_transform
        Transform::Pingpong { .. }
        | Transform::Reverse
        | Transform::FrameOffset { .. }
        | Transform::Hold { .. } => Err(TransformError::InvalidParameter {
            op: "image_transform".to_string(),
            message: "animation transforms cannot be applied to images".to_string(),
        }),
        // Color-based transforms are not yet implemented for images
        Transform::Outline { .. }
        | Transform::Shadow { .. }
        | Transform::SelOut { .. }
        | Transform::Dither { .. }
        | Transform::DitherGradient { .. }
        | Transform::Subpixel { .. } => Err(TransformError::InvalidParameter {
            op: "image_transform".to_string(),
            message: "color-based transforms require palette context".to_string(),
        }),
    }
}

/// Apply a sequence of transforms to an image.
///
/// Transforms are applied in order from left to right.
///
/// # Arguments
/// * `image` - The image to transform
/// * `transforms` - The sequence of transforms to apply
/// * `palette` - Optional palette for color-based transforms
///
/// # Returns
/// The transformed image, or an error if any transform cannot be applied
pub fn apply_image_transforms(
    image: &RgbaImage,
    transforms: &[Transform],
    palette: Option<&std::collections::HashMap<String, String>>,
) -> Result<RgbaImage, TransformError> {
    let mut result = image.clone();
    for transform in transforms {
        result = apply_image_transform(&result, transform, palette)?;
    }
    Ok(result)
}

/// Apply pingpong transform: duplicate frames in reverse order for forward-backward play.
///
/// Given frames [A, B, C], produces:
/// - `exclude_ends=false`: [A, B, C, C, B, A]
/// - `exclude_ends=true`:  [A, B, C, B]
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
/// * `exclude_ends` - If true, don't duplicate first/last frame in reverse
///
/// # Returns
/// A new Vec with the pingpong sequence
pub fn apply_pingpong<T: Clone>(frames: &[T], exclude_ends: bool) -> Vec<T> {
    if frames.is_empty() {
        return Vec::new();
    }
    if frames.len() == 1 {
        return frames.to_vec();
    }

    let mut result = frames.to_vec();

    if exclude_ends {
        // Reverse without first and last: [A, B, C] -> [A, B, C, B]
        // Skip first (0) and last (len-1) when reversing
        for i in (1..frames.len() - 1).rev() {
            result.push(frames[i].clone());
        }
    } else {
        // Full reverse including ends: [A, B, C] -> [A, B, C, C, B, A]
        for i in (0..frames.len()).rev() {
            result.push(frames[i].clone());
        }
    }

    result
}

/// Apply reverse transform: reverse the order of frames.
///
/// Given frames [A, B, C], produces [C, B, A].
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
///
/// # Returns
/// A new Vec with reversed frame order
pub fn apply_reverse<T: Clone>(frames: &[T]) -> Vec<T> {
    frames.iter().rev().cloned().collect()
}

/// Apply frame-offset transform: rotate frames by offset positions.
///
/// Given frames [A, B, C, D] with offset=1, produces [B, C, D, A].
/// Negative offsets rotate in the opposite direction.
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
/// * `offset` - Number of positions to rotate (positive = forward, negative = backward)
///
/// # Returns
/// A new Vec with rotated frame order
pub fn apply_frame_offset<T: Clone>(frames: &[T], offset: i32) -> Vec<T> {
    if frames.is_empty() {
        return Vec::new();
    }

    let len = frames.len() as i32;
    // Normalize offset to positive value within range
    let normalized = ((offset % len) + len) % len;

    let mut result = Vec::with_capacity(frames.len());
    for i in 0..frames.len() {
        let idx = (i as i32 + normalized) % len;
        result.push(frames[idx as usize].clone());
    }
    result
}

/// Apply hold transform: duplicate a specific frame multiple times.
///
/// Given frames [A, B, C] with frame=1 and count=3, produces [A, B, B, B, C].
///
/// # Arguments
/// * `frames` - The animation frames (sprite names)
/// * `frame` - Index of the frame to hold (0-based)
/// * `count` - Number of times to repeat the frame (total occurrences)
///
/// # Returns
/// A new Vec with the held frame duplicated, or original if frame index is invalid
pub fn apply_hold<T: Clone>(frames: &[T], frame: usize, count: usize) -> Vec<T> {
    if frames.is_empty() || frame >= frames.len() {
        return frames.to_vec();
    }
    if count == 0 {
        // count=0 means remove the frame
        let mut result = Vec::with_capacity(frames.len().saturating_sub(1));
        for (i, f) in frames.iter().enumerate() {
            if i != frame {
                result.push(f.clone());
            }
        }
        return result;
    }

    let mut result = Vec::with_capacity(frames.len() + count - 1);
    for (i, f) in frames.iter().enumerate() {
        if i == frame {
            // Insert count copies
            for _ in 0..count {
                result.push(f.clone());
            }
        } else {
            result.push(f.clone());
        }
    }
    result
}

/// Apply an animation transform to a list of frames.
///
/// Only animation-specific transforms are applied here. Non-animation transforms
/// (geometric, expansion, effects) return an error.
///
/// # Arguments
/// * `transform` - The transform to apply
/// * `frames` - The animation frames (sprite names)
///
/// # Returns
/// A new Vec with the transform applied, or an error if not an animation transform
pub fn apply_animation_transform<T: Clone>(
    transform: &Transform,
    frames: &[T],
) -> Result<Vec<T>, TransformError> {
    match transform {
        Transform::Pingpong { exclude_ends } => Ok(apply_pingpong(frames, *exclude_ends)),
        Transform::Reverse => Ok(apply_reverse(frames)),
        Transform::FrameOffset { offset } => Ok(apply_frame_offset(frames, *offset)),
        Transform::Hold { frame, count } => Ok(apply_hold(frames, *frame, *count)),
        _ => Err(TransformError::InvalidParameter {
            op: "animation_transform".to_string(),
            message: "transform is not an animation transform".to_string(),
        }),
    }
}

/// Check if a transform is an animation transform.
///
/// Animation transforms only make sense for animations (not sprites):
/// - Pingpong, Reverse, FrameOffset, Hold
pub fn is_animation_transform(transform: &Transform) -> bool {
    matches!(
        transform,
        Transform::Pingpong { .. }
            | Transform::Reverse
            | Transform::FrameOffset { .. }
            | Transform::Hold { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_pingpong_basic() {
        let frames = vec!["A", "B", "C"];
        let result = apply_pingpong(&frames, false);
        assert_eq!(result, vec!["A", "B", "C", "C", "B", "A"]);
    }

    #[test]
    fn test_apply_pingpong_exclude_ends() {
        let frames = vec!["A", "B", "C"];
        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["A", "B", "C", "B"]);
    }

    #[test]
    fn test_apply_pingpong_single_frame() {
        let frames = vec!["A"];
        let result = apply_pingpong(&frames, false);
        assert_eq!(result, vec!["A"]);

        let result = apply_pingpong(&frames, true);
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_apply_pingpong_two_frames() {
        let frames = vec!["A", "B"];
        let result = apply_pingpong(&frames, false);
        assert_eq!(result, vec!["A", "B", "B", "A"]);

        let result = apply_pingpong(&frames, true);
        // With exclude_ends, nothing to add (indices 1..len-1 is empty for len=2)
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_apply_pingpong_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_pingpong(&frames, false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_reverse_basic() {
        let frames = vec!["A", "B", "C", "D"];
        let result = apply_reverse(&frames);
        assert_eq!(result, vec!["D", "C", "B", "A"]);
    }

    #[test]
    fn test_apply_reverse_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_reverse(&frames);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_reverse_single() {
        let frames = vec!["A"];
        let result = apply_reverse(&frames);
        assert_eq!(result, vec!["A"]);
    }

    #[test]
    fn test_apply_frame_offset_positive() {
        let frames = vec!["A", "B", "C", "D"];
        let result = apply_frame_offset(&frames, 1);
        assert_eq!(result, vec!["B", "C", "D", "A"]);
    }

    #[test]
    fn test_apply_frame_offset_negative() {
        let frames = vec!["A", "B", "C", "D"];
        let result = apply_frame_offset(&frames, -1);
        assert_eq!(result, vec!["D", "A", "B", "C"]);
    }

    #[test]
    fn test_apply_frame_offset_zero() {
        let frames = vec!["A", "B", "C"];
        let result = apply_frame_offset(&frames, 0);
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_apply_frame_offset_wrap() {
        let frames = vec!["A", "B", "C"];
        let result = apply_frame_offset(&frames, 3);
        assert_eq!(result, vec!["A", "B", "C"]); // Full rotation

        let result = apply_frame_offset(&frames, 4);
        assert_eq!(result, vec!["B", "C", "A"]); // Same as offset=1
    }

    #[test]
    fn test_apply_frame_offset_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_frame_offset(&frames, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_hold_basic() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 1, 3);
        assert_eq!(result, vec!["A", "B", "B", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_first_frame() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 0, 2);
        assert_eq!(result, vec!["A", "A", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_last_frame() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 2, 4);
        assert_eq!(result, vec!["A", "B", "C", "C", "C", "C"]);
    }

    #[test]
    fn test_apply_hold_count_one() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 1, 1);
        // count=1 means keep single copy (no change)
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_count_zero() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 1, 0);
        // count=0 removes the frame
        assert_eq!(result, vec!["A", "C"]);
    }

    #[test]
    fn test_apply_hold_invalid_index() {
        let frames = vec!["A", "B", "C"];
        let result = apply_hold(&frames, 10, 5);
        // Invalid index returns original
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_apply_hold_empty() {
        let frames: Vec<&str> = vec![];
        let result = apply_hold(&frames, 0, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_is_animation_transform() {
        assert!(is_animation_transform(&Transform::Pingpong { exclude_ends: false }));
        assert!(is_animation_transform(&Transform::Reverse));
        assert!(is_animation_transform(&Transform::FrameOffset { offset: 1 }));
        assert!(is_animation_transform(&Transform::Hold { frame: 0, count: 2 }));

        assert!(!is_animation_transform(&Transform::MirrorH));
        assert!(!is_animation_transform(&Transform::MirrorV));
        assert!(!is_animation_transform(&Transform::Rotate { degrees: 90 }));
        assert!(!is_animation_transform(&Transform::Scale { x: 1.0, y: 1.0 }));
        assert!(!is_animation_transform(&Transform::Tile { w: 2, h: 2 }));
    }

    #[test]
    fn test_apply_animation_transform_pingpong() {
        let frames = vec!["A", "B", "C"];
        let result =
            apply_animation_transform(&Transform::Pingpong { exclude_ends: false }, &frames)
                .unwrap();
        assert_eq!(result, vec!["A", "B", "C", "C", "B", "A"]);
    }

    #[test]
    fn test_apply_animation_transform_reverse() {
        let frames = vec!["A", "B", "C"];
        let result = apply_animation_transform(&Transform::Reverse, &frames).unwrap();
        assert_eq!(result, vec!["C", "B", "A"]);
    }

    #[test]
    fn test_apply_animation_transform_offset() {
        let frames = vec!["A", "B", "C"];
        let result =
            apply_animation_transform(&Transform::FrameOffset { offset: 1 }, &frames).unwrap();
        assert_eq!(result, vec!["B", "C", "A"]);
    }

    #[test]
    fn test_apply_animation_transform_hold() {
        let frames = vec!["A", "B", "C"];
        let result =
            apply_animation_transform(&Transform::Hold { frame: 1, count: 2 }, &frames).unwrap();
        assert_eq!(result, vec!["A", "B", "B", "C"]);
    }

    #[test]
    fn test_apply_animation_transform_non_animation() {
        let frames = vec!["A", "B", "C"];
        let result = apply_animation_transform(&Transform::MirrorH, &frames);
        assert!(result.is_err());
    }
}
