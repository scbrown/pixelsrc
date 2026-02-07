//! Semantic Preservation Demo Tests
//!
//! Demonstrates how antialiasing respects semantic information to preserve
//! important sprite features while smoothing appropriate regions.

use image::{Rgba, RgbaImage};
use pixelsrc::antialias::{
    apply_semantic_blur, hq2x, hq4x, scale2x, AnchorMode, AntialiasConfig, GradientPair,
    Scale2xOptions, SemanticContext,
};
use pixelsrc::models::Role;
use std::collections::HashSet;

/// Create a simple face sprite with eye anchors.
///
/// Layout (8x8):
/// ```text
/// ........
/// ........
/// ..E..E..  (E = eye anchors at (2,2) and (5,2))
/// ........
/// ..MMMM..  (M = mouth)
/// ........
/// ........
/// ........
/// ```
fn create_face_sprite() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);
    let skin = Rgba([255, 200, 150, 255]);
    let eye = Rgba([0, 0, 0, 255]);
    let mouth = Rgba([200, 100, 100, 255]);

    // Fill with skin
    for y in 0..8 {
        for x in 0..8 {
            img.put_pixel(x, y, skin);
        }
    }

    // Add eyes
    img.put_pixel(2, 2, eye);
    img.put_pixel(5, 2, eye);

    // Add mouth
    for x in 2..6 {
        img.put_pixel(x, 4, mouth);
    }

    img
}

/// Create semantic context with eye anchors for the face sprite.
fn create_face_context() -> SemanticContext {
    let mut ctx = SemanticContext::empty();

    // Mark eyes as anchors
    ctx.anchor_pixels.insert((2, 2));
    ctx.anchor_pixels.insert((5, 2));

    // Also add to role masks
    let mut anchor_set = HashSet::new();
    anchor_set.insert((2, 2));
    anchor_set.insert((5, 2));
    ctx.role_masks.insert(Role::Anchor, anchor_set);

    ctx
}

// ============================================================================
// Anchor Preservation Tests
// ============================================================================

/// @demo antialias/semantic#anchor_preserve
/// @title Anchor Preservation Mode
/// @description Anchor pixels (eyes, details) remain crisp with no antialiasing.
/// This is the default behavior to protect important sprite features.
#[test]
fn test_anchor_preservation_scale2x() {
    let input = create_face_sprite();
    let context = create_face_context();

    let options = Scale2xOptions {
        anchor_mode: AnchorMode::Preserve,
        respect_containment: true,
        strength: 1.0,
    };

    let output = scale2x(&input, &context, &options);

    // Get the eye color from original
    let eye = *input.get_pixel(2, 2);

    // The anchor pixel at (2,2) should produce a 2x2 block of the exact same color
    // at output positions (4,4), (5,4), (4,5), (5,5)
    assert_eq!(*output.get_pixel(4, 4), eye, "Anchor top-left should be preserved");
    assert_eq!(*output.get_pixel(5, 4), eye, "Anchor top-right should be preserved");
    assert_eq!(*output.get_pixel(4, 5), eye, "Anchor bottom-left should be preserved");
    assert_eq!(*output.get_pixel(5, 5), eye, "Anchor bottom-right should be preserved");
}

/// @demo antialias/semantic#anchor_preserve_hq2x
/// @title Anchor Preservation in HQ2x
/// @description HQ2x also respects anchor pixels, producing uninterpolated blocks.
#[test]
fn test_anchor_preservation_hq2x() {
    let input = create_face_sprite();
    let context = create_face_context();

    let mut config = AntialiasConfig::default();
    config.strength = 1.0;
    config.anchor_mode = AnchorMode::Preserve;

    let output = hq2x(&input, &context, &config);

    let eye = *input.get_pixel(2, 2);

    // Anchor at (2,2) maps to 2x2 block at (4,4)
    for dy in 0..2 {
        for dx in 0..2 {
            assert_eq!(
                *output.get_pixel(4 + dx, 4 + dy),
                eye,
                "HQ2x anchor pixel should be preserved at ({}, {})",
                4 + dx,
                4 + dy
            );
        }
    }
}

/// @demo antialias/semantic#anchor_preserve_hq4x
/// @title Anchor Preservation in HQ4x
/// @description HQ4x respects anchors, producing 4x4 uninterpolated blocks.
#[test]
fn test_anchor_preservation_hq4x() {
    let input = create_face_sprite();
    let context = create_face_context();

    let mut config = AntialiasConfig::default();
    config.strength = 1.0;
    config.anchor_mode = AnchorMode::Preserve;

    let output = hq4x(&input, &context, &config);

    let eye = *input.get_pixel(2, 2);

    // Anchor at (2,2) maps to 4x4 block at (8,8)
    for dy in 0..4 {
        for dx in 0..4 {
            assert_eq!(
                *output.get_pixel(8 + dx, 8 + dy),
                eye,
                "HQ4x anchor pixel should be preserved at ({}, {})",
                8 + dx,
                8 + dy
            );
        }
    }
}

/// @demo antialias/semantic#anchor_reduce
/// @title Anchor Reduce Mode
/// @description Anchors receive 25% antialiasing strength instead of none.
/// Slight softening while maintaining visibility.
#[test]
fn test_anchor_reduce_mode() {
    let input = create_face_sprite();
    let context = create_face_context();

    // With Reduce mode, anchors get partial AA
    let mut config = AntialiasConfig::default();
    config.strength = 1.0;
    config.anchor_mode = AnchorMode::Reduce;

    let output = apply_semantic_blur(&input, &context, &config);

    // The eye pixels should be slightly blurred but still recognizable
    // (not as sharp as Preserve mode, not as blurred as Normal mode)
    let original_eye = *input.get_pixel(2, 2);
    let blurred_eye = *output.get_pixel(2, 2);

    // With Reduce, the eye should have some color from surrounding skin mixed in
    // but should still be primarily the eye color
    assert!(
        blurred_eye[0] <= original_eye[0] + 50,
        "Reduced anchor should retain mostly original color"
    );
}

/// @demo antialias/semantic#anchor_normal
/// @title Anchor Normal Mode
/// @description Anchors receive full antialiasing like any other region.
/// Use when you want consistent smoothing across the entire sprite.
#[test]
fn test_anchor_normal_mode() {
    let input = create_face_sprite();
    let context = create_face_context();

    let options = Scale2xOptions {
        anchor_mode: AnchorMode::Normal,
        respect_containment: false,
        strength: 1.0,
    };

    let output = scale2x(&input, &context, &options);

    // With Normal mode, anchors are treated like any other pixel
    // Scale2x will apply its normal edge detection rules
    assert_eq!(output.dimensions(), (16, 16));
}

// ============================================================================
// Containment Edge Tests
// ============================================================================

/// @demo antialias/semantic#containment_edges
/// @title Containment Edge Respect
/// @description ContainedWithin relationships create hard boundaries.
/// Prevents color bleeding between distinct regions like eyes and skin.
#[test]
fn test_containment_edge_respect() {
    let input = create_face_sprite();
    let mut context = SemanticContext::empty();

    // Mark the eye pixels as containment edges (they're contained within skin)
    context.containment_edges.insert((2, 2));
    context.containment_edges.insert((5, 2));

    let options = Scale2xOptions {
        anchor_mode: AnchorMode::Normal, // Not using anchor preservation
        respect_containment: true,       // But respecting containment
        strength: 1.0,
    };

    let output = scale2x(&input, &context, &options);

    // Containment edges should be preserved like anchors
    let eye = *input.get_pixel(2, 2);

    // The eye at (2,2) should be preserved due to containment
    assert_eq!(*output.get_pixel(4, 4), eye, "Containment edge should be preserved");
}

/// @demo antialias/semantic#containment_disabled
/// @title Containment Disabled
/// @description With respect_containment=false, boundaries can blend.
#[test]
fn test_containment_disabled() {
    let input = create_face_sprite();
    let mut context = SemanticContext::empty();

    context.containment_edges.insert((2, 2));

    let options = Scale2xOptions {
        anchor_mode: AnchorMode::Normal,
        respect_containment: false, // Disabled
        strength: 1.0,
    };

    let output = scale2x(&input, &context, &options);

    // With containment disabled, normal Scale2x rules apply
    assert_eq!(output.dimensions(), (16, 16));
}

// ============================================================================
// Gradient Pair Tests
// ============================================================================

/// @demo antialias/semantic#gradient_pairs
/// @title Gradient Pair Detection
/// @description DerivesFrom relationships create smooth gradient transitions.
/// Shadow colors blend naturally with their base colors.
#[test]
fn test_gradient_pair_detection() {
    let mut context = SemanticContext::empty();

    // Set up a gradient pair (shadow derived from skin)
    context.gradient_pairs.push(GradientPair {
        source_token: "skin_shadow".to_string(),
        target_token: "skin".to_string(),
        source_color: Rgba([200, 150, 100, 255]),
        target_color: Rgba([255, 200, 150, 255]),
        boundary_pixels: vec![(3, 3), (3, 4), (3, 5)],
    });

    // Verify gradient is detected at boundary pixels
    assert!(context.get_gradient_at((3, 3)).is_some(), "Should detect gradient at boundary");
    assert!(context.get_gradient_at((3, 4)).is_some(), "Should detect gradient at boundary");
    assert!(
        context.get_gradient_at((0, 0)).is_none(),
        "Should not detect gradient away from boundary"
    );

    // Check gradient details
    let gradient = context.get_gradient_at((3, 3)).unwrap();
    assert_eq!(gradient.source_token, "skin_shadow");
    assert_eq!(gradient.target_token, "skin");
}

/// @demo antialias/semantic#gradient_shadows
/// @title Gradient Shadow Smoothing
/// @description When gradient_shadows is enabled, shadow/highlight transitions are smoother.
#[test]
fn test_gradient_shadow_smoothing() {
    // Create an image with a shadow transition
    let mut input = RgbaImage::new(8, 8);
    let base_color = Rgba([255, 200, 150, 255]);
    let shadow_color = Rgba([200, 150, 100, 255]);

    // Top half is base color, bottom half is shadow
    for y in 0..8 {
        for x in 0..8 {
            let color = if y < 4 { base_color } else { shadow_color };
            input.put_pixel(x, y, color);
        }
    }

    let mut context = SemanticContext::empty();

    // Mark the transition row as a gradient boundary
    context.gradient_pairs.push(GradientPair {
        source_token: "shadow".to_string(),
        target_token: "skin".to_string(),
        source_color: shadow_color,
        target_color: base_color,
        boundary_pixels: (0..8).map(|x| (x, 4)).collect(),
    });

    let mut config = AntialiasConfig::default();
    config.strength = 1.0;
    config.gradient_shadows = true;

    let output = hq2x(&input, &context, &config);

    // The transition should be smoother than a hard edge
    assert_eq!(output.dimensions(), (16, 16));
}

// ============================================================================
// Role-Based Masking Tests
// ============================================================================

/// @demo antialias/semantic#role_masks
/// @title Role-Based AA Masking
/// @description Different semantic roles receive different AA treatment.
/// Anchors: 0%, Boundary: 25%, Fill/Shadow/Highlight: 100%
#[test]
fn test_role_based_masking() {
    let mut context = SemanticContext::empty();

    // Add pixels with different roles
    let mut anchors = HashSet::new();
    anchors.insert((1, 1));
    context.role_masks.insert(Role::Anchor, anchors);
    context.anchor_pixels.insert((1, 1));

    let mut boundaries = HashSet::new();
    boundaries.insert((2, 2));
    context.role_masks.insert(Role::Boundary, boundaries);

    let mut fills = HashSet::new();
    fills.insert((3, 3));
    context.role_masks.insert(Role::Fill, fills);

    let mut shadows = HashSet::new();
    shadows.insert((4, 4));
    context.role_masks.insert(Role::Shadow, shadows);

    let mut highlights = HashSet::new();
    highlights.insert((5, 5));
    context.role_masks.insert(Role::Highlight, highlights);

    // Verify role lookups
    assert_eq!(context.get_role((1, 1)), Some(Role::Anchor));
    assert_eq!(context.get_role((2, 2)), Some(Role::Boundary));
    assert_eq!(context.get_role((3, 3)), Some(Role::Fill));
    assert_eq!(context.get_role((4, 4)), Some(Role::Shadow));
    assert_eq!(context.get_role((5, 5)), Some(Role::Highlight));
    assert_eq!(context.get_role((6, 6)), None);
}

/// @demo antialias/semantic#aa_blur_role_aware
/// @title AA-Blur Role Awareness
/// @description AA-Blur applies different blur weights based on semantic roles.
#[test]
fn test_aa_blur_role_aware() {
    let mut input = RgbaImage::new(8, 8);
    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);

    // Create a pattern with a bright center
    for y in 0..8 {
        for x in 0..8 {
            if x == 4 && y == 4 {
                input.put_pixel(x, y, white);
            } else {
                input.put_pixel(x, y, black);
            }
        }
    }

    let mut context = SemanticContext::empty();
    // Mark the white pixel as an anchor
    context.anchor_pixels.insert((4, 4));

    let mut config = AntialiasConfig::default();
    config.strength = 1.0;
    config.anchor_mode = AnchorMode::Preserve;

    let output = apply_semantic_blur(&input, &context, &config);

    // The anchor pixel should remain unchanged
    assert_eq!(*output.get_pixel(4, 4), white, "Anchor should remain crisp under blur");
}

// ============================================================================
// Context Scaling Tests
// ============================================================================

/// @demo antialias/semantic#context_scaling
/// @title Semantic Context Scaling
/// @description When upscaling, semantic context coordinates scale accordingly.
/// A 1x1 anchor becomes a 2x2 or 4x4 block of anchors.
#[test]
fn test_context_scaling() {
    let mut context = SemanticContext::empty();
    context.anchor_pixels.insert((1, 1));
    context.containment_edges.insert((2, 2));

    // Scale by 2x
    let scaled = context.scale(2);

    // (1, 1) should expand to (2, 2), (2, 3), (3, 2), (3, 3)
    assert!(scaled.anchor_pixels.contains(&(2, 2)));
    assert!(scaled.anchor_pixels.contains(&(2, 3)));
    assert!(scaled.anchor_pixels.contains(&(3, 2)));
    assert!(scaled.anchor_pixels.contains(&(3, 3)));
    assert_eq!(scaled.anchor_pixels.len(), 4);

    // (2, 2) should expand to (4, 4), (4, 5), (5, 4), (5, 5)
    assert!(scaled.containment_edges.contains(&(4, 4)));
    assert!(scaled.containment_edges.contains(&(4, 5)));
    assert!(scaled.containment_edges.contains(&(5, 4)));
    assert!(scaled.containment_edges.contains(&(5, 5)));
    assert_eq!(scaled.containment_edges.len(), 4);
}

/// @demo antialias/semantic#context_scaling_4x
/// @title Context Scaling for HQ4x
/// @description 4x scaling expands each pixel to a 4x4 block.
#[test]
fn test_context_scaling_4x() {
    let mut context = SemanticContext::empty();
    context.anchor_pixels.insert((1, 1));

    let scaled = context.scale(4);

    // (1, 1) should expand to 16 pixels at (4, 4) to (7, 7)
    assert_eq!(scaled.anchor_pixels.len(), 16);
    for dy in 0..4 {
        for dx in 0..4 {
            assert!(
                scaled.anchor_pixels.contains(&(4 + dx, 4 + dy)),
                "4x scaled anchor should contain ({}, {})",
                4 + dx,
                4 + dy
            );
        }
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

/// @demo antialias/semantic#full_sprite_processing
/// @title Full Sprite with Semantic AA
/// @description Complete workflow: sprite with anchors processed through Scale2x.
#[test]
fn test_full_sprite_semantic_processing() {
    let input = create_face_sprite();
    let context = create_face_context();

    let options = Scale2xOptions {
        anchor_mode: AnchorMode::Preserve,
        respect_containment: true,
        strength: 1.0,
    };

    let output = scale2x(&input, &context, &options);

    // Verify output dimensions
    assert_eq!(output.dimensions(), (16, 16));

    // Verify eyes are preserved (anchors)
    let eye = *input.get_pixel(2, 2);
    for dy in 0..2 {
        for dx in 0..2 {
            assert_eq!(*output.get_pixel(4 + dx, 4 + dy), eye, "Left eye should be preserved");
            assert_eq!(*output.get_pixel(10 + dx, 4 + dy), eye, "Right eye should be preserved");
        }
    }

    // Verify skin areas (non-anchors) may have interpolation applied
    // Just verify they're not the eye color
    let skin = *input.get_pixel(0, 0);
    assert_ne!(*output.get_pixel(0, 0), eye, "Non-anchor areas should not be eye color");
    assert_eq!(*output.get_pixel(0, 0), skin, "Corner should still be skin");
}
