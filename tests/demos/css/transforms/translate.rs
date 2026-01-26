//! Translate Transform Demo Tests
//!
//! Tests for position offset using translate(x, y), translateX(x), translateY(y).

use crate::demos::{assert_validates, parse_content};

/// @demo format/css/transforms#translate
/// @title Translate Transform
/// @description Position offset using translate(x, y), translateX(x), translateY(y).
#[test]
fn test_css_transforms_translate() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/translate.jsonl");
    assert_validates(jsonl, true);

    let (palette_registry, sprite_registry, animations) = parse_content(jsonl);

    // Verify sprites can be resolved
    sprite_registry
        .resolve("arrow_right", &palette_registry, false)
        .expect("Sprite 'arrow_right' should resolve");
    sprite_registry
        .resolve("arrow_base", &palette_registry, false)
        .expect("Sprite 'arrow_base' should resolve");

    // Test slide_right animation (translate in X)
    let slide_right = animations.get("slide_right").expect("Animation 'slide_right' not found");
    assert!(slide_right.is_css_keyframes(), "slide_right should use CSS keyframes");
    let kf = slide_right.keyframes.as_ref().unwrap();
    assert_eq!(kf["0%"].transform.as_deref(), Some("translate(0, 0)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("translate(8px, 0)"));

    // Test slide_down animation (translateY)
    let slide_down = animations.get("slide_down").expect("Animation 'slide_down' not found");
    let kf = slide_down.keyframes.as_ref().unwrap();
    assert_eq!(kf["0%"].transform.as_deref(), Some("translateY(0)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("translateY(4px)"));

    // Test slide_diagonal animation (translate both axes)
    let slide_diagonal =
        animations.get("slide_diagonal").expect("Animation 'slide_diagonal' not found");
    let kf = slide_diagonal.keyframes.as_ref().unwrap();
    assert_eq!(kf.len(), 3, "slide_diagonal should have 3 keyframes");
    assert_eq!(kf["50%"].transform.as_deref(), Some("translate(4px, 4px)"));
}

/// @demo format/css/transforms#translate_x
/// @title TranslateX Only
/// @description Horizontal offset using translateX(x) shorthand.
#[test]
fn test_translate_x_shorthand() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/translate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    // slide_right uses translate(x, 0) which is equivalent to translateX
    let slide_right = animations.get("slide_right").expect("Animation 'slide_right' not found");
    let kf = slide_right.keyframes.as_ref().unwrap();

    // Verify horizontal translation
    assert_eq!(kf["0%"].transform.as_deref(), Some("translate(0, 0)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("translate(8px, 0)"));
}

/// @demo format/css/transforms#translate_y
/// @title TranslateY Only
/// @description Vertical offset using translateY(y) shorthand.
#[test]
fn test_translate_y_shorthand() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/translate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let slide_down = animations.get("slide_down").expect("Animation 'slide_down' not found");
    let kf = slide_down.keyframes.as_ref().unwrap();

    // Verify vertical-only translation
    assert_eq!(kf["0%"].transform.as_deref(), Some("translateY(0)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("translateY(4px)"));
}

/// @demo format/css/transforms#translate_diagonal
/// @title Diagonal Translation
/// @description Combined X and Y translation for diagonal movement.
#[test]
fn test_translate_diagonal() {
    let jsonl = include_str!("../../../../examples/demos/css/transforms/translate.jsonl");
    assert_validates(jsonl, true);

    let (_, _, animations) = parse_content(jsonl);

    let slide_diagonal =
        animations.get("slide_diagonal").expect("Animation 'slide_diagonal' not found");
    let kf = slide_diagonal.keyframes.as_ref().unwrap();

    // Verify diagonal path: 0,0 -> 4,4 -> 8,8
    assert_eq!(kf["0%"].transform.as_deref(), Some("translate(0, 0)"));
    assert_eq!(kf["50%"].transform.as_deref(), Some("translate(4px, 4px)"));
    assert_eq!(kf["100%"].transform.as_deref(), Some("translate(8px, 8px)"));
}
