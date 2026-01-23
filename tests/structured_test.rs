//! Integration tests for structured sprite rendering

use pixelsrc::models::RegionDef;
use std::collections::HashMap;

#[test]
fn test_structured_renderer_basic() {
    let mut regions = HashMap::new();

    // Create an outline region
    regions.insert(
        "o".to_string(),
        RegionDef {
            stroke: Some([0, 0, 8, 8]),
            ..Default::default()
        },
    );

    // Create a fill region
    regions.insert(
        "f".to_string(),
        RegionDef {
            fill: Some("inside(o)".to_string()),
            ..Default::default()
        },
    );

    let palette = HashMap::from([
        ("_".to_string(), "#00000000".to_string()),
        ("o".to_string(), "#000000".to_string()),
        ("f".to_string(), "#FF0000".to_string()),
    ]);

    let (image, warnings) = pixelsrc::structured::render_structured(&regions, Some([8, 8]), &palette);

    assert!(warnings.is_empty());
    assert_eq!(image.width(), 8);
    assert_eq!(image.height(), 8);

    // Check corners are black (outline)
    assert_eq!(image.get_pixel(0, 0)[0], 0);
    assert_eq!(image.get_pixel(7, 0)[0], 0);
    assert_eq!(image.get_pixel(0, 7)[0], 0);
    assert_eq!(image.get_pixel(7, 7)[0], 0);

    // Check center is red (filled)
    let center_pixel = image.get_pixel(4, 4);
    assert_eq!(center_pixel[0], 255); // R
    assert_eq!(center_pixel[1], 0);   // G
    assert_eq!(center_pixel[2], 0);   // B
}

#[test]
fn test_structured_renderer_rect() {
    let mut regions = HashMap::new();

    regions.insert(
        "r".to_string(),
        RegionDef {
            rect: Some([2, 2, 4, 4]),
            ..Default::default()
        },
    );

    let palette = HashMap::from([("r".to_string(), "#00FF00".to_string())]);

    let (image, warnings) = pixelsrc::structured::render_structured(&regions, Some([8, 8]), &palette);

    assert!(warnings.is_empty());
    assert_eq!(image.width(), 8);
    assert_eq!(image.height(), 8);

    // Check rect is green in the expected position
    let pixel = image.get_pixel(3, 3);
    assert_eq!(pixel[0], 0);   // R
    assert_eq!(pixel[1], 255); // G
    assert_eq!(pixel[2], 0);   // B
}

#[test]
fn test_structured_renderer_circle() {
    let mut regions = HashMap::new();

    regions.insert(
        "c".to_string(),
        RegionDef {
            circle: Some([4, 4, 3]),
            ..Default::default()
        },
    );

    let palette = HashMap::from([("c".to_string(), "#0000FF".to_string())]);

    let (image, warnings) = pixelsrc::structured::render_structured(&regions, Some([8, 8]), &palette);

    assert!(warnings.is_empty());
    assert_eq!(image.width(), 8);
    assert_eq!(image.height(), 8);

    // Check center of circle is blue
    let pixel = image.get_pixel(4, 4);
    assert_eq!(pixel[0], 0);   // R
    assert_eq!(pixel[1], 0);   // G
    assert_eq!(pixel[2], 255); // B
}

#[test]
fn test_structured_renderer_z_order() {
    let mut regions = HashMap::new();

    // Lower z-order (background)
    regions.insert(
        "bg".to_string(),
        RegionDef {
            rect: Some([0, 0, 8, 8]),
            z: Some(0),
            ..Default::default()
        },
    );

    // Higher z-order (foreground)
    regions.insert(
        "fg".to_string(),
        RegionDef {
            rect: Some([3, 3, 2, 2]),
            z: Some(1),
            ..Default::default()
        },
    );

    let palette = HashMap::from([
        ("bg".to_string(), "#FF0000".to_string()),
        ("fg".to_string(), "#00FF00".to_string()),
    ]);

    let (image, warnings) = pixelsrc::structured::render_structured(&regions, Some([8, 8]), &palette);

    assert!(warnings.is_empty());

    // Background should be red
    let bg_pixel = image.get_pixel(1, 1);
    assert_eq!(bg_pixel[0], 255); // R

    // Foreground should be green
    let fg_pixel = image.get_pixel(3, 3);
    assert_eq!(fg_pixel[1], 255); // G
}

#[test]
fn test_structured_renderer_except_modifier() {
    let mut regions = HashMap::new();

    // Base region
    regions.insert(
        "base".to_string(),
        RegionDef {
            rect: Some([0, 0, 4, 4]),
            ..Default::default()
        },
    );

    // Region that excludes pixels from base
    regions.insert(
        "hole".to_string(),
        RegionDef {
            rect: Some([1, 1, 2, 2]),
            except: Some(vec!["base".to_string()]),
            ..Default::default()
        },
    );

    let palette = HashMap::from([
        ("base".to_string(), "#FF0000".to_string()),
        ("hole".to_string(), "#00FF00".to_string()),
    ]);

    let (image, warnings) = pixelsrc::structured::render_structured(&regions, Some([4, 4]), &palette);

    assert!(warnings.is_empty());

    // Center should be green (hole), not red
    let center_pixel = image.get_pixel(2, 2);
    assert_eq!(center_pixel[1], 255); // G

    // Corner should be red (base)
    let corner_pixel = image.get_pixel(0, 0);
    assert_eq!(corner_pixel[0], 255); // R
}
