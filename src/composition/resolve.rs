//! CSS variable resolution for composition properties

use crate::models::VarOr;
use crate::variables::VariableRegistry;

use super::blend::BlendMode;
use super::error::Warning;

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
