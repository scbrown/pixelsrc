//! Semantic analysis for PNG import.
//!
//! Provides naming hints based on region position, color, and detected roles.

use std::collections::{HashMap, HashSet};

use super::color_quantization::LabColor;
use super::NamingHint;
use crate::models::Role;

/// Semantic region position within the sprite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SemanticPosition {
    TopCenter,   // Hair region
    Center,      // Face/body center
    Bottom,      // Feet/base
    Edge,        // Border/background
    TopCorner,   // Hair edge or accessory
    Surrounding, // Background
}

/// Analyze the semantic position of a region within the sprite.
pub(crate) fn analyze_semantic_position(
    pixels: &HashSet<(i32, i32)>,
    width: u32,
    height: u32,
) -> (SemanticPosition, f64) {
    if pixels.is_empty() {
        return (SemanticPosition::Edge, 0.0);
    }

    let total_pixels = (width * height) as f64;
    let coverage = pixels.len() as f64 / total_pixels;

    // Large coverage suggests background
    if coverage > 0.5 {
        return (SemanticPosition::Surrounding, 0.9);
    }

    // Calculate centroid
    let sum_x: i32 = pixels.iter().map(|(x, _)| *x).sum();
    let sum_y: i32 = pixels.iter().map(|(_, y)| *y).sum();
    let centroid_x = sum_x as f64 / pixels.len() as f64;
    let centroid_y = sum_y as f64 / pixels.len() as f64;

    // Normalize to 0-1 range
    let norm_x = centroid_x / width as f64;
    let norm_y = centroid_y / height as f64;

    // Check if region touches edges (potential background)
    let touches_left = pixels.iter().any(|(x, _)| *x == 0);
    let touches_right = pixels.iter().any(|(x, _)| *x as u32 == width - 1);
    let touches_top = pixels.iter().any(|(_, y)| *y == 0);
    let touches_bottom = pixels.iter().any(|(_, y)| *y as u32 == height - 1);
    let edge_count =
        [touches_left, touches_right, touches_top, touches_bottom].iter().filter(|&&b| b).count();

    // Touches multiple edges and moderate coverage = background
    if edge_count >= 2 && coverage > 0.15 {
        return (SemanticPosition::Surrounding, 0.8);
    }

    // Determine position based on centroid
    let is_center_x = (0.25..0.75).contains(&norm_x);
    let is_top = norm_y < 0.33;
    let is_middle = (0.33..0.66).contains(&norm_y);
    let is_bottom = norm_y >= 0.66;

    if is_center_x && is_top {
        (SemanticPosition::TopCenter, 0.7)
    } else if is_center_x && is_middle {
        (SemanticPosition::Center, 0.7)
    } else if is_center_x && is_bottom {
        (SemanticPosition::Bottom, 0.7)
    } else if is_top && !is_center_x {
        (SemanticPosition::TopCorner, 0.6)
    } else {
        (SemanticPosition::Edge, 0.5)
    }
}

/// Check if a color is a skin tone (various skin tones).
pub(crate) fn is_skin_tone(color: &[u8; 4]) -> bool {
    let lab = LabColor::from_rgb(color[0], color[1], color[2]);

    // Skin tones have:
    // - Moderate to high lightness (L: 40-90)
    // - Positive a (reddish)
    // - Moderate positive b (yellowish)
    lab.l > 40.0 && lab.l < 90.0 && lab.a > 5.0 && lab.a < 40.0 && lab.b > 5.0 && lab.b < 50.0
}

/// Check if a color is dark (potential outline/shadow/hair).
pub(crate) fn is_dark_color(color: &[u8; 4]) -> bool {
    let lab = LabColor::from_rgb(color[0], color[1], color[2]);
    lab.l < 35.0
}

/// Check if a color is light (potential highlight/white).
pub(crate) fn is_light_color(color: &[u8; 4]) -> bool {
    let lab = LabColor::from_rgb(color[0], color[1], color[2]);
    lab.l > 85.0
}

/// Generate token naming suggestions based on detected features.
pub(crate) fn generate_naming_hints(
    roles: &HashMap<String, Role>,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    token_colors: &HashMap<String, [u8; 4]>,
    width: u32,
    height: u32,
) -> Vec<NamingHint> {
    let mut hints = Vec::new();
    let total_pixels = (width * height) as usize;

    for (token, pixels) in token_pixels {
        // Skip transparent token
        if token == "{_}" {
            continue;
        }

        let color = token_colors.get(token);
        let role = roles.get(token);
        let (position, position_confidence) = analyze_semantic_position(pixels, width, height);
        let size = pixels.len();
        let coverage = size as f64 / total_pixels as f64;

        // Build suggestion based on multiple factors
        let (suggested, reason) = suggest_semantic_name(
            position,
            position_confidence,
            color,
            role,
            size,
            coverage,
            width,
            height,
        );

        if let Some(suggested_name) = suggested {
            if token != &suggested_name {
                hints.push(NamingHint { token: token.clone(), suggested_name, reason });
            }
        }
    }

    hints
}

/// Suggest a semantic name based on position, color, role, and size.
fn suggest_semantic_name(
    position: SemanticPosition,
    position_confidence: f64,
    color: Option<&[u8; 4]>,
    role: Option<&Role>,
    size: usize,
    coverage: f64,
    _width: u32,
    _height: u32,
) -> (Option<String>, String) {
    // Background detection: large coverage or surrounding position
    if coverage > 0.4 || position == SemanticPosition::Surrounding {
        return (Some("{bg}".to_string()), "Large coverage, likely background".to_string());
    }

    // Check for skin tones in center = face/skin
    if let Some(c) = color {
        if is_skin_tone(c) {
            if position == SemanticPosition::Center {
                return (Some("{face}".to_string()), "Skin tone in center region".to_string());
            } else if position == SemanticPosition::TopCenter {
                return (Some("{skin}".to_string()), "Skin tone in upper region".to_string());
            } else if coverage > 0.05 {
                return (Some("{skin}".to_string()), "Detected skin tone color".to_string());
            }
        }

        // Dark colors
        if is_dark_color(c) {
            // Top center dark = hair
            if position == SemanticPosition::TopCenter {
                return (Some("{hair}".to_string()), "Dark color in top center".to_string());
            }
            // Small dark spots near center = eyes
            if size <= 6 && position == SemanticPosition::Center {
                return (Some("{eye}".to_string()), "Small dark region in center".to_string());
            }
            // Role-based dark suggestions
            if matches!(role, Some(Role::Boundary)) {
                return (Some("{outline}".to_string()), "Dark boundary region".to_string());
            }
            if matches!(role, Some(Role::Shadow)) {
                return (Some("{shadow}".to_string()), "Dark shadow region".to_string());
            }
            // Generic dark
            if position == SemanticPosition::Edge {
                return (Some("{outline}".to_string()), "Dark edge region".to_string());
            }
        }

        // Light colors
        if is_light_color(c) {
            if matches!(role, Some(Role::Highlight)) {
                return (Some("{highlight}".to_string()), "Light highlight region".to_string());
            }
            // Small light spots = eyes/reflections
            if size <= 4 {
                return (
                    Some("{gleam}".to_string()),
                    "Small light region (reflection)".to_string(),
                );
            }
        }
    }

    // Size-based suggestions for small features
    if size == 1 {
        return (Some("{dot}".to_string()), "Single pixel feature".to_string());
    }
    if size <= 4 {
        if position == SemanticPosition::Center {
            return (Some("{eye}".to_string()), "Small centered feature".to_string());
        }
        return (Some("{detail}".to_string()), "Small detail region".to_string());
    }

    // Position-based fallbacks
    match position {
        SemanticPosition::TopCenter if position_confidence > 0.6 => {
            (Some("{top}".to_string()), "Top center region".to_string())
        }
        SemanticPosition::Center if position_confidence > 0.6 => {
            (Some("{body}".to_string()), "Central body region".to_string())
        }
        SemanticPosition::Bottom if position_confidence > 0.6 => {
            (Some("{base}".to_string()), "Bottom base region".to_string())
        }
        _ => {
            // Fall back to role-based hints
            if let Some(r) = role {
                let name = match r {
                    Role::Boundary => "{outline}",
                    Role::Anchor => "{marker}",
                    Role::Fill => "{fill}",
                    Role::Shadow => "{shadow}",
                    Role::Highlight => "{highlight}",
                };
                (Some(name.to_string()), format!("Detected as {} role", r))
            } else {
                (None, String::new())
            }
        }
    }
}
