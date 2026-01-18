//! Blend modes for composition layers (ATF-10)
//!
//! This module provides blend mode types and CSS variable resolution
//! for layer blending and opacity.

use crate::error::Warning;
use crate::models::VarOr;
use crate::variables::VariableRegistry;
use serde::{Deserialize, Serialize};

// ============================================================================
// Blend Modes (ATF-10)
// ============================================================================

/// Blend modes for composition layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    /// Standard alpha compositing (source over destination)
    #[default]
    Normal,
    /// Darkens underlying colors: result = base * blend
    Multiply,
    /// Lightens underlying colors: result = 1 - (1 - base) * (1 - blend)
    Screen,
    /// Combines multiply/screen based on base brightness
    Overlay,
    /// Additive blending: result = min(1, base + blend)
    Add,
    /// Subtractive blending: result = max(0, base - blend)
    Subtract,
    /// Color difference: result = abs(base - blend)
    Difference,
    /// Keeps darker color: result = min(base, blend)
    Darken,
    /// Keeps lighter color: result = max(base, blend)
    Lighten,
}

impl BlendMode {
    /// Parse a blend mode from string
    pub fn from_str(s: &str) -> Option<BlendMode> {
        match s.to_lowercase().as_str() {
            "normal" => Some(BlendMode::Normal),
            "multiply" => Some(BlendMode::Multiply),
            "screen" => Some(BlendMode::Screen),
            "overlay" => Some(BlendMode::Overlay),
            "add" | "additive" => Some(BlendMode::Add),
            "subtract" | "subtractive" => Some(BlendMode::Subtract),
            "difference" => Some(BlendMode::Difference),
            "darken" => Some(BlendMode::Darken),
            "lighten" => Some(BlendMode::Lighten),
            _ => None,
        }
    }

    /// Apply blend mode to a single color channel (values are 0.0-1.0)
    pub(crate) fn blend_channel(&self, base: f32, blend: f32) -> f32 {
        match self {
            BlendMode::Normal => blend,
            BlendMode::Multiply => base * blend,
            BlendMode::Screen => 1.0 - (1.0 - base) * (1.0 - blend),
            BlendMode::Overlay => {
                if base < 0.5 {
                    2.0 * base * blend
                } else {
                    1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
                }
            }
            BlendMode::Add => (base + blend).min(1.0),
            BlendMode::Subtract => (base - blend).max(0.0),
            BlendMode::Difference => (base - blend).abs(),
            BlendMode::Darken => base.min(blend),
            BlendMode::Lighten => base.max(blend),
        }
    }
}

// ============================================================================
// CSS Variable Resolution (CSS-9)
// ============================================================================

/// Resolve a blend mode string, potentially containing a var() reference.
///
/// Returns the resolved blend mode and any warning if resolution failed.
pub fn resolve_blend_mode(
    blend: Option<&str>,
    registry: Option<&VariableRegistry>,
) -> (BlendMode, Option<Warning>) {
    let Some(blend_str) = blend else {
        return (BlendMode::Normal, None);
    };

    // Check if it contains var() and we have a registry
    let resolved = if blend_str.contains("var(") {
        if let Some(reg) = registry {
            match reg.resolve(blend_str) {
                Ok(resolved) => resolved,
                Err(e) => {
                    return (
                        BlendMode::Normal,
                        Some(Warning::new(format!(
                            "Failed to resolve blend mode variable '{}': {}, using normal",
                            blend_str, e
                        ))),
                    );
                }
            }
        } else {
            // No registry provided but var() used - warn and use default
            return (
                BlendMode::Normal,
                Some(Warning::new(format!(
                    "Blend mode '{}' contains var() but no variable registry provided, using normal",
                    blend_str
                ))),
            );
        }
    } else {
        blend_str.to_string()
    };

    // Parse the resolved string
    match BlendMode::from_str(&resolved) {
        Some(mode) => (mode, None),
        None => (
            BlendMode::Normal,
            Some(Warning::new(format!("Unknown blend mode '{}', using normal", resolved))),
        ),
    }
}

/// Resolve an opacity value, potentially containing a var() reference.
///
/// Returns the resolved opacity (clamped to 0.0-1.0) and any warning if resolution failed.
pub fn resolve_opacity(
    opacity: Option<&VarOr<f64>>,
    registry: Option<&VariableRegistry>,
) -> (f64, Option<Warning>) {
    let Some(opacity_val) = opacity else {
        return (1.0, None);
    };

    match opacity_val {
        VarOr::Value(v) => (*v, None),
        VarOr::Var(var_str) => {
            if let Some(reg) = registry {
                match reg.resolve(var_str) {
                    Ok(resolved) => {
                        // Try to parse the resolved string as f64
                        match resolved.trim().parse::<f64>() {
                            Ok(v) => (v.clamp(0.0, 1.0), None),
                            Err(_) => (
                                1.0,
                                Some(Warning::new(format!(
                                    "Opacity variable '{}' resolved to '{}' which is not a valid number, using 1.0",
                                    var_str, resolved
                                ))),
                            ),
                        }
                    }
                    Err(e) => (
                        1.0,
                        Some(Warning::new(format!(
                            "Failed to resolve opacity variable '{}': {}, using 1.0",
                            var_str, e
                        ))),
                    ),
                }
            } else {
                (
                    1.0,
                    Some(Warning::new(format!(
                        "Opacity '{}' contains var() but no variable registry provided, using 1.0",
                        var_str
                    ))),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Blend Mode Tests (ATF-10)
    // ========================================================================

    #[test]
    fn test_blend_mode_from_str() {
        assert_eq!(BlendMode::from_str("normal"), Some(BlendMode::Normal));
        assert_eq!(BlendMode::from_str("multiply"), Some(BlendMode::Multiply));
        assert_eq!(BlendMode::from_str("screen"), Some(BlendMode::Screen));
        assert_eq!(BlendMode::from_str("overlay"), Some(BlendMode::Overlay));
        assert_eq!(BlendMode::from_str("add"), Some(BlendMode::Add));
        assert_eq!(BlendMode::from_str("additive"), Some(BlendMode::Add));
        assert_eq!(BlendMode::from_str("subtract"), Some(BlendMode::Subtract));
        assert_eq!(BlendMode::from_str("difference"), Some(BlendMode::Difference));
        assert_eq!(BlendMode::from_str("darken"), Some(BlendMode::Darken));
        assert_eq!(BlendMode::from_str("lighten"), Some(BlendMode::Lighten));
        assert_eq!(BlendMode::from_str("NORMAL"), Some(BlendMode::Normal)); // case insensitive
        assert_eq!(BlendMode::from_str("unknown"), None);
    }

    #[test]
    fn test_blend_mode_multiply() {
        // Multiply: result = base * blend
        let mode = BlendMode::Multiply;
        // 0.5 * 0.5 = 0.25
        assert!((mode.blend_channel(0.5, 0.5) - 0.25).abs() < 0.01);
        // 1.0 * 0.5 = 0.5
        assert!((mode.blend_channel(1.0, 0.5) - 0.5).abs() < 0.01);
        // 0.0 * anything = 0.0
        assert!((mode.blend_channel(0.0, 1.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_screen() {
        // Screen: result = 1 - (1 - base) * (1 - blend)
        let mode = BlendMode::Screen;
        // 1 - (1 - 0.5) * (1 - 0.5) = 1 - 0.25 = 0.75
        assert!((mode.blend_channel(0.5, 0.5) - 0.75).abs() < 0.01);
        // Screen with white = white
        assert!((mode.blend_channel(0.5, 1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_add() {
        // Add: result = min(1, base + blend)
        let mode = BlendMode::Add;
        assert!((mode.blend_channel(0.3, 0.4) - 0.7).abs() < 0.01);
        // Clamped at 1.0
        assert!((mode.blend_channel(0.8, 0.5) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_subtract() {
        // Subtract: result = max(0, base - blend)
        let mode = BlendMode::Subtract;
        assert!((mode.blend_channel(0.7, 0.3) - 0.4).abs() < 0.01);
        // Clamped at 0.0
        assert!((mode.blend_channel(0.3, 0.7) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_difference() {
        // Difference: result = abs(base - blend)
        let mode = BlendMode::Difference;
        assert!((mode.blend_channel(0.7, 0.3) - 0.4).abs() < 0.01);
        assert!((mode.blend_channel(0.3, 0.7) - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_darken() {
        // Darken: result = min(base, blend)
        let mode = BlendMode::Darken;
        assert!((mode.blend_channel(0.7, 0.3) - 0.3).abs() < 0.01);
        assert!((mode.blend_channel(0.3, 0.7) - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_blend_mode_lighten() {
        // Lighten: result = max(base, blend)
        let mode = BlendMode::Lighten;
        assert!((mode.blend_channel(0.7, 0.3) - 0.7).abs() < 0.01);
        assert!((mode.blend_channel(0.3, 0.7) - 0.7).abs() < 0.01);
    }

    // ========================================================================
    // CSS Variable Tests (CSS-9)
    // ========================================================================

    #[test]
    fn test_resolve_blend_mode_with_var() {
        let mut registry = VariableRegistry::new();
        registry.define("--blend", "multiply");

        let (mode, warning) = resolve_blend_mode(Some("var(--blend)"), Some(&registry));
        assert_eq!(mode, BlendMode::Multiply);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_blend_mode_with_var_fallback() {
        let registry = VariableRegistry::new();

        // Missing variable with fallback
        let (mode, warning) = resolve_blend_mode(Some("var(--missing, screen)"), Some(&registry));
        assert_eq!(mode, BlendMode::Screen);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_blend_mode_undefined_var() {
        let registry = VariableRegistry::new();

        // Missing variable without fallback
        let (mode, warning) = resolve_blend_mode(Some("var(--undefined)"), Some(&registry));
        assert_eq!(mode, BlendMode::Normal);
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("Failed to resolve"));
    }

    #[test]
    fn test_resolve_blend_mode_no_registry() {
        // var() used but no registry provided
        let (mode, warning) = resolve_blend_mode(Some("var(--blend)"), None);
        assert_eq!(mode, BlendMode::Normal);
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("no variable registry"));
    }

    #[test]
    fn test_resolve_opacity_with_var() {
        let mut registry = VariableRegistry::new();
        registry.define("--opacity", "0.5");

        let var_opacity = VarOr::Var("var(--opacity)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), Some(&registry));
        assert!((opacity - 0.5).abs() < 0.001);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_opacity_with_var_fallback() {
        let registry = VariableRegistry::new();

        // Missing variable with numeric fallback
        let var_opacity = VarOr::Var("var(--missing, 0.75)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), Some(&registry));
        assert!((opacity - 0.75).abs() < 0.001);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_opacity_literal() {
        let registry = VariableRegistry::new();

        // Literal value (not var)
        let literal_opacity = VarOr::Value(0.3);
        let (opacity, warning) = resolve_opacity(Some(&literal_opacity), Some(&registry));
        assert!((opacity - 0.3).abs() < 0.001);
        assert!(warning.is_none());
    }

    #[test]
    fn test_resolve_opacity_clamps_values() {
        let mut registry = VariableRegistry::new();
        registry.define("--over", "2.0");
        registry.define("--under", "-0.5");

        // Value over 1.0 should clamp to 1.0
        let over = VarOr::Var("var(--over)".to_string());
        let (opacity, _) = resolve_opacity(Some(&over), Some(&registry));
        assert!((opacity - 1.0).abs() < 0.001);

        // Value under 0.0 should clamp to 0.0
        let under = VarOr::Var("var(--under)".to_string());
        let (opacity, _) = resolve_opacity(Some(&under), Some(&registry));
        assert!(opacity.abs() < 0.001);
    }

    #[test]
    fn test_resolve_opacity_invalid_number() {
        let mut registry = VariableRegistry::new();
        registry.define("--invalid", "not-a-number");

        let var_opacity = VarOr::Var("var(--invalid)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), Some(&registry));
        assert!((opacity - 1.0).abs() < 0.001); // Falls back to 1.0
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("not a valid number"));
    }

    #[test]
    fn test_resolve_opacity_no_registry() {
        let var_opacity = VarOr::Var("var(--opacity)".to_string());
        let (opacity, warning) = resolve_opacity(Some(&var_opacity), None);
        assert!((opacity - 1.0).abs() < 0.001); // Falls back to 1.0
        assert!(warning.is_some());
        assert!(warning.unwrap().message.contains("no variable registry"));
    }
}
