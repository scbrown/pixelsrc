//! Palettes CLI Demos
//!
//! Demo tests for the `pxl palettes` command that lists and shows built-in palettes.

use pixelsrc::palettes::{get_builtin, list_builtins};

/// @demo cli/palettes#list
/// @title List Built-in Palettes
/// @description Show all available built-in palette names.
#[test]
fn test_palettes_list() {
    let builtins = list_builtins();

    // Should have several built-in palettes
    assert!(!builtins.is_empty(), "Should have built-in palettes");
    assert!(builtins.len() >= 5, "Should have at least 5 built-in palettes");

    // Should include classic palettes
    assert!(builtins.contains(&"gameboy"), "Should have gameboy palette");
    assert!(builtins.contains(&"nes"), "Should have nes palette");
    assert!(builtins.contains(&"pico8"), "Should have pico8 palette");
}

/// @demo cli/palettes#show_gameboy
/// @title Show Game Boy Palette
/// @description Display the classic Game Boy 4-color green palette.
#[test]
fn test_palettes_show_gameboy() {
    let palette = get_builtin("gameboy").expect("gameboy palette should exist");

    // Game Boy has 4-5 colors (4 classic + possible transparent)
    assert!(palette.colors.len() >= 4, "Game Boy palette should have at least 4 colors");

    // Should have standard token names
    assert!(palette.colors.contains_key("{lightest}"), "Should have {{lightest}} token");
    assert!(palette.colors.contains_key("{darkest}"), "Should have {{darkest}} token");
}

/// @demo cli/palettes#show_pico8
/// @title Show PICO-8 Palette
/// @description Display the fantasy console PICO-8 16-color palette.
#[test]
fn test_palettes_show_pico8() {
    let palette = get_builtin("pico8").expect("pico8 palette should exist");

    // PICO-8 has 16-17 colors (16 classic + possible transparent)
    assert!(palette.colors.len() >= 16, "PICO-8 palette should have at least 16 colors");
}

/// @demo cli/palettes#show_nes
/// @title Show NES Palette
/// @description Display a subset of the NES/Famicom color palette.
#[test]
fn test_palettes_show_nes() {
    let palette = get_builtin("nes").expect("nes palette should exist");

    // NES palette should have multiple colors
    assert!(!palette.colors.is_empty(), "NES palette should have colors");
}

/// @demo cli/palettes#show_grayscale
/// @title Show Grayscale Palette
/// @description Display grayscale colors for simple black and white art.
#[test]
fn test_palettes_show_grayscale() {
    let palette = get_builtin("grayscale").expect("grayscale palette should exist");

    // Should have black, white, and grays
    assert!(palette.colors.len() >= 4, "Grayscale should have at least 4 shades");
}

/// @demo cli/palettes#show_1bit
/// @title Show 1-Bit Palette
/// @description Display simple 2-color black and white palette.
#[test]
fn test_palettes_show_1bit() {
    let palette = get_builtin("1bit").expect("1bit palette should exist");

    // 1-bit is 2-3 colors (black, white, + possible transparent)
    assert!(palette.colors.len() >= 2, "1-bit palette should have at least 2 colors");
    assert!(palette.colors.len() <= 3, "1-bit palette should have at most 3 colors");
}

/// @demo cli/palettes#show_dracula
/// @title Show Dracula Palette
/// @description Display the Dracula theme color palette.
#[test]
fn test_palettes_show_dracula() {
    let palette = get_builtin("dracula").expect("dracula palette should exist");

    // Dracula should have the theme colors
    assert!(!palette.colors.is_empty(), "Dracula palette should have colors");
}

/// @demo cli/palettes#not_found
/// @title Palette Not Found
/// @description Handle unknown palette names gracefully.
#[test]
fn test_palettes_not_found() {
    let result = get_builtin("nonexistent");
    assert!(result.is_none(), "Unknown palette should return None");
}
