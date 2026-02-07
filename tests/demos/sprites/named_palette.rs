//! Named palette demos
//!
//! Sprites using palette references instead of inline colors.

use crate::demos::{assert_uses_palette, assert_validates, capture_render_info, parse_content};

/// @demo format/sprite#named_palette
/// @title Named Palette Reference
/// @description Sprites can reference a named palette defined separately.
#[test]
fn test_named_palette() {
    let jsonl = r##"{"type": "palette", "name": "gameboy", "colors": {"{_}": "#9BBC0F", "{1}": "#8BAC0F", "{2}": "#306230", "{3}": "#0F380F"}}
{"type": "sprite", "name": "tile", "palette": "gameboy", "size": [2, 2], "regions": {"_": {"points": [[0,0]]}, "1": {"points": [[1,0]]}, "2": {"points": [[0,1]]}, "3": {"points": [[1,1]]}}}"##;

    assert_validates(jsonl, true);
    assert_uses_palette(jsonl, "tile", "gameboy");

    let (palette_registry, _, _) = parse_content(jsonl);
    assert!(palette_registry.contains("gameboy"), "Palette 'gameboy' should be registered");

    let info = capture_render_info(jsonl, "tile");
    assert_eq!(info.width, 2);
    assert_eq!(info.height, 2);
    assert_eq!(info.palette_name, Some("gameboy".to_string()));
}
