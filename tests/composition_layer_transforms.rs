//! Layer transforms apply before placement and never alter shared cached inputs.

use image::{Rgba, RgbaImage};
use pixelsrc::composition::{render_composition, render_composition_nested, RenderContext};
use pixelsrc::models::Composition;
use pixelsrc::registry::CompositionRegistry;
use serde_json::json;
use std::collections::HashMap;

fn sprites() -> HashMap<String, RgbaImage> {
    let mut image = RgbaImage::new(4, 1);
    image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    HashMap::from([("dot".into(), image)])
}

#[test]
fn transforms_run_in_order_before_each_cell_placement() {
    let comp: Composition = serde_json::from_value(json!({
        "name": "pair", "size": [8, 1], "cell_size": [4, 1],
        "sprites": {"A": "dot"},
        "layers": [{"map": ["AA"], "transform": ["shift:1,0", "mirror-h"]}]
    }))
    .unwrap();
    let sprites = sprites();
    let (image, warnings) = render_composition(&comp, &sprites, true, None).unwrap();
    assert!(warnings.is_empty());
    for x in 0..8 {
        assert_eq!(image.get_pixel(x, 0)[3] > 0, x == 2 || x == 6);
    }
    assert_eq!(sprites["dot"].get_pixel(0, 0)[3], 255);
}

#[test]
fn nested_layer_transforms_leave_cached_compositions_unchanged() {
    let inner: Composition = serde_json::from_value(json!({
        "name": "inner", "size": [4, 1], "cell_size": [4, 1],
        "sprites": {"A": "dot"}, "layers": [{"map": ["A"]}]
    }))
    .unwrap();
    let outer: Composition = serde_json::from_value(json!({
        "name": "outer", "size": [8, 1], "cell_size": [4, 1],
        "sprites": {"N": "inner"},
        "layers": [
            {"map": ["NN"], "transform": ["shift:1,0", "mirror-h"]},
            {"map": ["N"]}
        ]
    }))
    .unwrap();
    let mut registry = CompositionRegistry::new();
    registry.register(inner);
    let sprites = sprites();
    let mut context = RenderContext::new();
    let (image, warnings) =
        render_composition_nested(&outer, &sprites, Some(&registry), &mut context, true, None)
            .unwrap();
    assert!(warnings.is_empty());
    for x in 0..8 {
        assert_eq!(image.get_pixel(x, 0)[3] > 0, [0, 2, 6].contains(&x));
    }
    assert_eq!(context.get_cached("inner").unwrap(), &sprites["dot"]);
}

#[test]
fn invalid_layer_transforms_warn_or_fail_in_both_renderers() {
    let comp: Composition = serde_json::from_value(json!({
        "name": "bad", "size": [4, 1], "cell_size": [4, 1],
        "sprites": {"A": "dot"},
        "layers": [{"name": "pose", "map": ["A"],
                    "transform": ["unknown-operation", {"op": "shift", "x": 1, "y": 0}]}]
    }))
    .unwrap();
    for nested in [false, true] {
        for strict in [false, true] {
            let sprites = sprites();
            let result = if nested {
                render_composition_nested(
                    &comp,
                    &sprites,
                    None,
                    &mut RenderContext::new(),
                    strict,
                    None,
                )
            } else {
                render_composition(&comp, &sprites, strict, None)
            };
            if strict {
                let error = result.unwrap_err().to_string();
                assert!(error.contains("pose") && error.contains("invalid transform"));
            } else {
                let (image, warnings) = result.unwrap();
                assert_eq!(warnings.len(), 1);
                assert!(warnings[0].message.contains("invalid transform"));
                assert_eq!(image.get_pixel(0, 0)[3], 0);
                assert_eq!(image.get_pixel(1, 0)[3], 255);
            }
        }
    }
}

#[test]
fn transformed_sprite_dimensions_are_validated() {
    let comp: Composition = serde_json::from_value(json!({
        "name": "large", "size": [4, 1], "cell_size": [4, 1],
        "sprites": {"A": "dot"},
        "layers": [{"map": ["A"], "transform": ["pad:1"]}]
    }))
    .unwrap();
    let error = render_composition(&comp, &sprites(), true, None).unwrap_err();
    assert!(error.to_string().contains("exceeds cell size"));
}
