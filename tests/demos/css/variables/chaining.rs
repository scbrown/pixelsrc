//! CSS Variable Chaining Demo Tests
//!
//! Tests for chaining CSS variables through multiple levels of indirection.

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/variables#chaining_basic
/// @title Basic Variable Chaining
/// @description CSS variables can reference other variables, forming a chain.
#[test]
fn test_basic_chaining() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/chaining.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry
        .get("basic_chain")
        .expect("basic_chain palette should exist");
    assert!(
        palette.colors.len() >= 2,
        "Should have at least 2 colors (transparent + bc)"
    );
}

/// @demo format/css/variables#chaining_deep
/// @title Deep Variable Chaining
/// @description Multiple levels of variable references resolve correctly.
#[test]
fn test_deep_chaining() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/chaining.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry
        .get("deep_chain")
        .expect("deep_chain palette should exist");
    assert!(
        palette.colors.len() >= 2,
        "Should have at least 2 colors (transparent + dc)"
    );
}

/// @demo format/css/variables#chaining_color_mix
/// @title Color-Mix Variable Chaining
/// @description Variables chained through color-mix() function calls.
#[test]
fn test_color_mix_chaining() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/chaining.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, _, _) = parse_content(jsonl);

    let palette = palette_registry
        .get("color_mix_chain")
        .expect("color_mix_chain palette should exist");
    assert!(
        palette.colors.len() >= 2,
        "Should have at least 2 colors (transparent + mc)"
    );
}

/// @demo format/css/variables#chaining_basic_sprite
/// @title Basic Chain Sprite
/// @description Sprite using basic chained variable palette renders correctly.
#[test]
fn test_basic_chain_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/chaining.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("chain_result")
        .expect("chain_result sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}

/// @demo format/css/variables#chaining_deep_sprite
/// @title Deep Chain Sprite
/// @description Sprite using deeply chained variable palette renders correctly.
#[test]
fn test_deep_chain_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/chaining.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("deep_chain_result")
        .expect("deep_chain_result sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}

/// @demo format/css/variables#chaining_mix_sprite
/// @title Color-Mix Chain Sprite
/// @description Sprite using color-mix chained variable palette renders correctly.
#[test]
fn test_mix_chain_sprite() {
    let jsonl = include_str!("../../../../examples/demos/css/variables/chaining.jsonl");
    assert_validates(jsonl, true);

    let (_, sprite_registry, _) = parse_content(jsonl);

    let sprite = sprite_registry
        .get_sprite("shaded_box")
        .expect("shaded_box sprite should exist");
    let size = sprite.size.expect("sprite should have size");
    assert_eq!(size[0], 2, "Sprite width should be 2");
    assert_eq!(size[1], 2, "Sprite height should be 2");
}
