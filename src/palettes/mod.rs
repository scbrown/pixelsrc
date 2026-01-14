//! Built-in palette definitions.
//!
//! Provides a set of commonly used pixel art palettes that can be
//! referenced by name using the `@name` syntax.

use crate::models::Palette;
use std::collections::HashMap;

/// List of all available built-in palette names.
const BUILTIN_NAMES: &[&str] = &["gameboy", "nes", "pico8", "grayscale", "1bit"];

/// Returns a list of all available built-in palette names.
pub fn list_builtins() -> Vec<&'static str> {
    BUILTIN_NAMES.to_vec()
}

/// Returns a built-in palette by name, or None if not found.
pub fn get_builtin(name: &str) -> Option<Palette> {
    match name {
        "gameboy" => Some(gameboy()),
        "nes" => Some(nes()),
        "pico8" => Some(pico8()),
        "grayscale" => Some(grayscale()),
        "1bit" => Some(one_bit()),
        _ => None,
    }
}

/// Game Boy 4-color green palette.
/// Reference: https://lospec.com/palette-list/nintendo-gameboy-bgb
fn gameboy() -> Palette {
    Palette {
        name: "gameboy".to_string(),
        colors: HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{lightest}".to_string(), "#9BBC0F".to_string()),
            ("{light}".to_string(), "#8BAC0F".to_string()),
            ("{dark}".to_string(), "#306230".to_string()),
            ("{darkest}".to_string(), "#0F380F".to_string()),
        ]),
    }
}

/// NES-inspired palette with key representative colors.
/// Reference: https://lospec.com/palette-list/nintendo-entertainment-system
fn nes() -> Palette {
    Palette {
        name: "nes".to_string(),
        colors: HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{black}".to_string(), "#000000".to_string()),
            ("{white}".to_string(), "#FCFCFC".to_string()),
            ("{red}".to_string(), "#A80020".to_string()),
            ("{green}".to_string(), "#00A800".to_string()),
            ("{blue}".to_string(), "#0058F8".to_string()),
            ("{cyan}".to_string(), "#00B8D8".to_string()),
            ("{yellow}".to_string(), "#F8D800".to_string()),
            ("{orange}".to_string(), "#F83800".to_string()),
            ("{pink}".to_string(), "#F878F8".to_string()),
            ("{brown}".to_string(), "#503000".to_string()),
            ("{gray}".to_string(), "#7C7C7C".to_string()),
            ("{skin}".to_string(), "#FCB8B8".to_string()),
        ]),
    }
}

/// PICO-8 16-color palette.
/// Reference: https://lospec.com/palette-list/pico-8
fn pico8() -> Palette {
    Palette {
        name: "pico8".to_string(),
        colors: HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{black}".to_string(), "#000000".to_string()),
            ("{dark_blue}".to_string(), "#1D2B53".to_string()),
            ("{dark_purple}".to_string(), "#7E2553".to_string()),
            ("{dark_green}".to_string(), "#008751".to_string()),
            ("{brown}".to_string(), "#AB5236".to_string()),
            ("{dark_gray}".to_string(), "#5F574F".to_string()),
            ("{light_gray}".to_string(), "#C2C3C7".to_string()),
            ("{white}".to_string(), "#FFF1E8".to_string()),
            ("{red}".to_string(), "#FF004D".to_string()),
            ("{orange}".to_string(), "#FFA300".to_string()),
            ("{yellow}".to_string(), "#FFEC27".to_string()),
            ("{green}".to_string(), "#00E436".to_string()),
            ("{blue}".to_string(), "#29ADFF".to_string()),
            ("{indigo}".to_string(), "#83769C".to_string()),
            ("{pink}".to_string(), "#FF77A8".to_string()),
            ("{peach}".to_string(), "#FFCCAA".to_string()),
        ]),
    }
}

/// 8-shade grayscale palette from white to black.
fn grayscale() -> Palette {
    Palette {
        name: "grayscale".to_string(),
        colors: HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{white}".to_string(), "#FFFFFF".to_string()),
            ("{gray1}".to_string(), "#DFDFDF".to_string()),
            ("{gray2}".to_string(), "#BFBFBF".to_string()),
            ("{gray3}".to_string(), "#9F9F9F".to_string()),
            ("{gray4}".to_string(), "#7F7F7F".to_string()),
            ("{gray5}".to_string(), "#5F5F5F".to_string()),
            ("{gray6}".to_string(), "#3F3F3F".to_string()),
            ("{black}".to_string(), "#000000".to_string()),
        ]),
    }
}

/// 1-bit black and white palette.
fn one_bit() -> Palette {
    Palette {
        name: "1bit".to_string(),
        colors: HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{black}".to_string(), "#000000".to_string()),
            ("{white}".to_string(), "#FFFFFF".to_string()),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtins() {
        let builtins = list_builtins();
        assert!(builtins.contains(&"gameboy"));
        assert!(builtins.contains(&"nes"));
        assert!(builtins.contains(&"pico8"));
        assert!(builtins.contains(&"grayscale"));
        assert!(builtins.contains(&"1bit"));
        assert_eq!(builtins.len(), 5);
    }

    #[test]
    fn test_get_builtin_gameboy() {
        let palette = get_builtin("gameboy").expect("gameboy palette should exist");
        assert_eq!(palette.name, "gameboy");
        assert_eq!(palette.colors.get("{lightest}"), Some(&"#9BBC0F".to_string()));
        assert_eq!(palette.colors.get("{light}"), Some(&"#8BAC0F".to_string()));
        assert_eq!(palette.colors.get("{dark}"), Some(&"#306230".to_string()));
        assert_eq!(palette.colors.get("{darkest}"), Some(&"#0F380F".to_string()));
    }

    #[test]
    fn test_get_builtin_nes() {
        let palette = get_builtin("nes").expect("nes palette should exist");
        assert_eq!(palette.name, "nes");
        assert!(palette.colors.contains_key("{red}"));
        assert!(palette.colors.contains_key("{green}"));
        assert!(palette.colors.contains_key("{blue}"));
    }

    #[test]
    fn test_get_builtin_pico8() {
        let palette = get_builtin("pico8").expect("pico8 palette should exist");
        assert_eq!(palette.name, "pico8");
        // PICO-8 has 16 colors + transparent
        assert_eq!(palette.colors.len(), 17);
        assert_eq!(palette.colors.get("{black}"), Some(&"#000000".to_string()));
        assert_eq!(palette.colors.get("{white}"), Some(&"#FFF1E8".to_string()));
    }

    #[test]
    fn test_get_builtin_grayscale() {
        let palette = get_builtin("grayscale").expect("grayscale palette should exist");
        assert_eq!(palette.name, "grayscale");
        // 8 shades + transparent
        assert_eq!(palette.colors.len(), 9);
        assert_eq!(palette.colors.get("{white}"), Some(&"#FFFFFF".to_string()));
        assert_eq!(palette.colors.get("{black}"), Some(&"#000000".to_string()));
    }

    #[test]
    fn test_get_builtin_1bit() {
        let palette = get_builtin("1bit").expect("1bit palette should exist");
        assert_eq!(palette.name, "1bit");
        // Black, white, and transparent
        assert_eq!(palette.colors.len(), 3);
        assert_eq!(palette.colors.get("{black}"), Some(&"#000000".to_string()));
        assert_eq!(palette.colors.get("{white}"), Some(&"#FFFFFF".to_string()));
    }

    #[test]
    fn test_get_builtin_nonexistent() {
        assert!(get_builtin("nonexistent").is_none());
        assert!(get_builtin("").is_none());
        assert!(get_builtin("Gameboy").is_none()); // case-sensitive
    }

    #[test]
    fn test_all_builtins_have_transparent() {
        for name in list_builtins() {
            let palette = get_builtin(name).expect("all listed builtins should exist");
            assert!(
                palette.colors.contains_key("{_}"),
                "Palette {} should have transparent color {{_}}",
                name
            );
        }
    }
}
