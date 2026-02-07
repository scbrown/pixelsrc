//! Antialiasing Algorithm Demo Tests
//!
//! Demonstrates each AA algorithm and its scaling/smoothing characteristics.

use image::{Rgba, RgbaImage};
use pixelsrc::antialias::{
    apply_semantic_blur, hq2x, hq4x, scale2x, AAAlgorithm, AntialiasConfig, Scale2xOptions,
    SemanticContext,
};

/// Create a simple test pattern image with distinct regions.
///
/// Creates a checkerboard-like pattern useful for testing edge detection.
fn create_test_pattern(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            // Create a diagonal pattern that shows edge handling
            let color = if (x + y) % 2 == 0 {
                Rgba([0, 0, 0, 255]) // Black
            } else {
                Rgba([255, 255, 255, 255]) // White
            };
            img.put_pixel(x, y, color);
        }
    }
    img
}

/// Create a simple edge pattern for testing edge-aware algorithms.
///
/// Left half is black, right half is white.
fn create_edge_pattern(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    let mid = width / 2;
    for y in 0..height {
        for x in 0..width {
            let color = if x < mid { Rgba([0, 0, 0, 255]) } else { Rgba([255, 255, 255, 255]) };
            img.put_pixel(x, y, color);
        }
    }
    img
}

/// Create a gradient pattern for testing smooth blending.
fn create_gradient_pattern(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let intensity = ((x as f32 / width as f32) * 255.0) as u8;
            img.put_pixel(x, y, Rgba([intensity, intensity, intensity, 255]));
        }
    }
    img
}

// ============================================================================
// Algorithm Selection Tests
// ============================================================================

/// @demo antialias/algorithm#nearest
/// @title Nearest Neighbor (No AA)
/// @description Default passthrough mode - no antialiasing applied.
/// Preserves crisp pixel art edges exactly as defined.
#[test]
fn test_algorithm_nearest() {
    let algo = AAAlgorithm::Nearest;

    // Nearest neighbor should have scale factor of 1
    assert_eq!(algo.scale_factor(), 1, "Nearest should not scale");
    assert!(!algo.is_enabled(), "Nearest should indicate AA is disabled");
    assert_eq!(format!("{}", algo), "nearest");
}

/// @demo antialias/algorithm#scale2x
/// @title Scale2x (EPX) Algorithm
/// @description 2x upscaling with edge-aware interpolation.
/// Smooths diagonal edges while preserving horizontal/vertical lines.
#[test]
fn test_algorithm_scale2x() {
    let input = create_edge_pattern(8, 8);
    let context = SemanticContext::empty();
    let options = Scale2xOptions::default();

    let output = scale2x(&input, &context, &options);

    // Scale2x produces 2x output
    assert_eq!(output.width(), 16, "Scale2x should double width");
    assert_eq!(output.height(), 16, "Scale2x should double height");

    // Verify algorithm metadata
    let algo = AAAlgorithm::Scale2x;
    assert_eq!(algo.scale_factor(), 2);
    assert!(algo.is_enabled());
    assert_eq!(format!("{}", algo), "scale2x");
}

/// @demo antialias/algorithm#scale2x_edge_detection
/// @title Scale2x Edge Detection
/// @description Demonstrates how Scale2x detects and smooths diagonal edges.
#[test]
fn test_scale2x_edge_detection() {
    // Create a simple diagonal pattern that Scale2x should smooth
    let mut input = RgbaImage::new(3, 3);
    let black = Rgba([0, 0, 0, 255]);
    let white = Rgba([255, 255, 255, 255]);

    // Diagonal: B W W
    //           B B W
    //           B B B
    input.put_pixel(0, 0, black);
    input.put_pixel(1, 0, white);
    input.put_pixel(2, 0, white);
    input.put_pixel(0, 1, black);
    input.put_pixel(1, 1, black);
    input.put_pixel(2, 1, white);
    input.put_pixel(0, 2, black);
    input.put_pixel(1, 2, black);
    input.put_pixel(2, 2, black);

    let context = SemanticContext::empty();
    let options = Scale2xOptions::default();

    let output = scale2x(&input, &context, &options);

    // Output should be 6x6
    assert_eq!(output.dimensions(), (6, 6));

    // The algorithm should have processed the edge
    // Corner pixels in the black region should remain black
    assert_eq!(*output.get_pixel(0, 0), black);
}

/// @demo antialias/algorithm#hq2x
/// @title HQ2x Algorithm
/// @description 2x upscaling with pattern-based interpolation.
/// Uses YUV color comparison for perceptually accurate edge detection.
#[test]
fn test_algorithm_hq2x() {
    let input = create_edge_pattern(8, 8);
    let context = SemanticContext::empty();
    let mut config = AntialiasConfig::default();
    config.strength = 1.0;

    let output = hq2x(&input, &context, &config);

    // HQ2x produces 2x output
    assert_eq!(output.width(), 16, "HQ2x should double width");
    assert_eq!(output.height(), 16, "HQ2x should double height");

    // Verify algorithm metadata
    let algo = AAAlgorithm::Hq2x;
    assert_eq!(algo.scale_factor(), 2);
    assert!(algo.is_enabled());
    assert_eq!(format!("{}", algo), "hq2x");
}

/// @demo antialias/algorithm#hq4x
/// @title HQ4x Algorithm
/// @description 4x upscaling with fine-grained pattern-based interpolation.
/// Produces smoother results than HQ2x with more gradual transitions.
#[test]
fn test_algorithm_hq4x() {
    let input = create_edge_pattern(4, 4);
    let context = SemanticContext::empty();
    let mut config = AntialiasConfig::default();
    config.strength = 1.0;

    let output = hq4x(&input, &context, &config);

    // HQ4x produces 4x output
    assert_eq!(output.width(), 16, "HQ4x should quadruple width");
    assert_eq!(output.height(), 16, "HQ4x should quadruple height");

    // Verify algorithm metadata
    let algo = AAAlgorithm::Hq4x;
    assert_eq!(algo.scale_factor(), 4);
    assert!(algo.is_enabled());
    assert_eq!(format!("{}", algo), "hq4x");
}

/// @demo antialias/algorithm#aa_blur
/// @title AA-Blur Algorithm
/// @description Gaussian blur with semantic masking.
/// Applies blur selectively based on pixel roles - anchors stay crisp.
#[test]
fn test_algorithm_aa_blur() {
    let input = create_gradient_pattern(8, 8);
    let context = SemanticContext::empty();
    let mut config = AntialiasConfig::default();
    config.strength = 1.0;

    let output = apply_semantic_blur(&input, &context, &config);

    // AA-Blur maintains original dimensions (no scaling)
    assert_eq!(output.width(), input.width(), "AA-blur should not change width");
    assert_eq!(output.height(), input.height(), "AA-blur should not change height");

    // Verify algorithm metadata
    let algo = AAAlgorithm::AaBlur;
    assert_eq!(algo.scale_factor(), 1, "AA-blur should not scale");
    assert!(algo.is_enabled());
    assert_eq!(format!("{}", algo), "aa-blur");
}

// ============================================================================
// Algorithm Comparison Tests
// ============================================================================

/// @demo antialias/algorithm#scale_factors
/// @title Algorithm Scale Factors
/// @description Each algorithm produces a specific output scale.
/// Nearest and AA-Blur preserve size; others upscale 2x or 4x.
#[test]
fn test_all_algorithm_scale_factors() {
    // No scaling
    assert_eq!(AAAlgorithm::Nearest.scale_factor(), 1);
    assert_eq!(AAAlgorithm::AaBlur.scale_factor(), 1);

    // 2x scaling
    assert_eq!(AAAlgorithm::Scale2x.scale_factor(), 2);
    assert_eq!(AAAlgorithm::Hq2x.scale_factor(), 2);
    assert_eq!(AAAlgorithm::Xbr2x.scale_factor(), 2);

    // 4x scaling
    assert_eq!(AAAlgorithm::Hq4x.scale_factor(), 4);
    assert_eq!(AAAlgorithm::Xbr4x.scale_factor(), 4);
}

/// @demo antialias/algorithm#solid_color_preservation
/// @title Solid Color Preservation
/// @description All algorithms should preserve solid color regions unchanged.
#[test]
fn test_solid_color_preservation() {
    let mut input = RgbaImage::new(4, 4);
    let red = Rgba([255, 0, 0, 255]);
    for y in 0..4 {
        for x in 0..4 {
            input.put_pixel(x, y, red);
        }
    }

    let context = SemanticContext::empty();
    let mut config = AntialiasConfig::default();
    config.strength = 1.0;

    // Scale2x
    let output = scale2x(&input, &context, &Scale2xOptions::default());
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(*output.get_pixel(x, y), red, "Scale2x should preserve solid color");
        }
    }

    // HQ2x
    let output = hq2x(&input, &context, &config);
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(*output.get_pixel(x, y), red, "HQ2x should preserve solid color");
        }
    }

    // HQ4x
    let output = hq4x(&input, &context, &config);
    for y in 0..16 {
        for x in 0..16 {
            assert_eq!(*output.get_pixel(x, y), red, "HQ4x should preserve solid color");
        }
    }
}

/// @demo antialias/algorithm#strength_control
/// @title Strength Control
/// @description Antialiasing strength from 0.0 (off) to 1.0 (full).
/// Zero strength produces nearest-neighbor equivalent.
#[test]
fn test_strength_control() {
    let input = create_test_pattern(4, 4);
    let context = SemanticContext::empty();

    // Zero strength Scale2x should act like nearest neighbor
    let options = Scale2xOptions { strength: 0.0, ..Default::default() };
    let output_zero = scale2x(&input, &context, &options);

    // Each pixel should be duplicated without interpolation
    for y in 0..4 {
        for x in 0..4 {
            let original = input.get_pixel(x, y);
            // Check the 2x2 block
            assert_eq!(output_zero.get_pixel(x * 2, y * 2), original);
            assert_eq!(output_zero.get_pixel(x * 2 + 1, y * 2), original);
            assert_eq!(output_zero.get_pixel(x * 2, y * 2 + 1), original);
            assert_eq!(output_zero.get_pixel(x * 2 + 1, y * 2 + 1), original);
        }
    }

    // Full strength should produce interpolated results
    let options_full = Scale2xOptions { strength: 1.0, ..Default::default() };
    let output_full = scale2x(&input, &context, &options_full);

    // Output dimensions should match
    assert_eq!(output_full.dimensions(), output_zero.dimensions());
}

// ============================================================================
// Serialization Tests
// ============================================================================

/// @demo antialias/config#serialization
/// @title Algorithm Serialization
/// @description Algorithms serialize to kebab-case names for config files.
#[test]
fn test_algorithm_serialization() {
    // Test JSON serialization
    let algo = AAAlgorithm::Hq4x;
    let json = serde_json::to_string(&algo).unwrap();
    assert_eq!(json, "\"hq4x\"");

    // AA-blur uses hyphenated name
    let algo = AAAlgorithm::AaBlur;
    let json = serde_json::to_string(&algo).unwrap();
    assert_eq!(json, "\"aa-blur\"");

    // Test deserialization
    let algo: AAAlgorithm = serde_json::from_str("\"scale2x\"").unwrap();
    assert_eq!(algo, AAAlgorithm::Scale2x);

    let algo: AAAlgorithm = serde_json::from_str("\"aa-blur\"").unwrap();
    assert_eq!(algo, AAAlgorithm::AaBlur);
}

/// @demo antialias/config#config_structure
/// @title Antialias Config Structure
/// @description Full configuration with algorithm, strength, and modes.
#[test]
fn test_config_structure() {
    let json = r#"{
        "enabled": true,
        "algorithm": "hq2x",
        "strength": 0.8,
        "anchor_mode": "preserve",
        "gradient_shadows": true,
        "respect_containment": true,
        "semantic_aware": true
    }"#;

    let config: AntialiasConfig = serde_json::from_str(json).unwrap();
    assert!(config.enabled);
    assert_eq!(config.algorithm, AAAlgorithm::Hq2x);
    assert!((config.strength - 0.8).abs() < 0.001);
    assert!(config.gradient_shadows);
    assert!(config.respect_containment);
    assert!(config.semantic_aware);
}
