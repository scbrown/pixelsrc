//! Format CLI Demos
//!
//! Demo tests for the `pxl fmt` command that formats JSONL for readability.

use pixelsrc::fmt::format_pixelsrc;
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::io::Cursor;
#[test]
fn test_fmt_basic() {
    let input = include_str!("../../../examples/demos/cli/format/fmt_input.jsonl");

    // Format the content
    let formatted = format_pixelsrc(input).expect("Formatting should succeed");

    // Verify formatted content is still valid JSONL
    let reader = Cursor::new(&formatted);
    let parse_result = parse_stream(reader);

    // Count objects
    let mut palette_count = 0;
    let mut sprite_count = 0;
    for obj in &parse_result.objects {
        match obj {
            TtpObject::Palette(_) => palette_count += 1,
            TtpObject::Sprite(_) => sprite_count += 1,
            _ => {}
        }
    }

    assert_eq!(palette_count, 1, "Should have 1 palette");
    assert_eq!(sprite_count, 2, "Should have 2 sprites");
}
/// @title Single Row Stays Compact
/// @description Single-row sprites remain on a single line for compactness./// @demo cli/fmt#palette_sorted
/// @title Palette Colors Sorted
/// @description Palette colors are sorted alphabetically for consistent output.
#[test]
fn test_fmt_palette_sorted() {
    // Create palette with unsorted colors
    let input = r##"{"type": "palette", "name": "test", "colors": {"{z}": "#000000", "{a}": "#FFFFFF", "{m}": "#888888"}}"##;

    let formatted = format_pixelsrc(input).expect("Formatting should succeed");

    // Find positions of color keys to verify sorting
    let a_pos = formatted.find("\"{a}\"").expect("{{a}} should be in output");
    let m_pos = formatted.find("\"{m}\"").expect("{{m}} should be in output");
    let z_pos = formatted.find("\"{z}\"").expect("{{z}} should be in output");

    assert!(a_pos < m_pos, "{{a}} should come before {{m}}");
    assert!(m_pos < z_pos, "{{m}} should come before {{z}}");
}
#[test]
fn test_fmt_roundtrip() {
    let input = include_str!("../../../examples/demos/cli/format/fmt_input.jsonl");

    // Parse original
    let original_reader = Cursor::new(input);
    let original_result = parse_stream(original_reader);

    // Format and parse again
    let formatted = format_pixelsrc(input).expect("Formatting should succeed");
    let formatted_reader = Cursor::new(&formatted);
    let formatted_result = parse_stream(formatted_reader);

    // Same number of objects
    assert_eq!(
        original_result.objects.len(),
        formatted_result.objects.len(),
        "Formatted content should have same number of objects"
    );

    // Verify sprites have same grids
    let original_sprites: Vec<_> = original_result
        .objects
        .iter()
        .filter_map(|o| match o {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
        .collect();

    let formatted_sprites: Vec<_> = formatted_result
        .objects
        .iter()
        .filter_map(|o| match o {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
        .collect();

    for (orig, fmt) in original_sprites.iter().zip(formatted_sprites.iter()) {
        assert_eq!(orig.name, fmt.name, "Sprite names should match");
        assert_eq!(orig.size, fmt.size, "Sprite sizes should match");
    }
}
#[test]
fn test_fmt_composition() {
    let input = r##"{"type": "composition", "name": "scene", "size": [16, 16], "sprites": {".": null, "H": "hero"}, "layers": [{"name": "ground", "fill": "."}, {"name": "objects", "map": ["....", "..H.", "...."]}]}"##;

    let formatted = format_pixelsrc(input).expect("Formatting should succeed");

    // Composition should have layers on separate lines
    assert!(formatted.contains('\n'), "Composition should have multi-line formatting");

    // Verify it still parses
    let reader = Cursor::new(&formatted);
    let result = parse_stream(reader);
    let compositions: Vec<_> = result
        .objects
        .iter()
        .filter_map(|o| match o {
            TtpObject::Composition(c) => Some(c),
            _ => None,
        })
        .collect();

    assert_eq!(compositions.len(), 1, "Should have 1 composition");
    assert_eq!(compositions[0].layers.len(), 2, "Should have 2 layers");
}
