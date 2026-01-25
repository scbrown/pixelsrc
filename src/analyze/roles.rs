//! Role inference for sprite regions

use std::collections::{HashMap, HashSet};

use crate::models::Role;

use super::shapes::bounding_box;

/// Result of role inference with confidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleInference {
    /// The inferred role
    pub role: Role,
    /// Confidence score from 0.0 to 1.0
    pub confidence: f64,
}

impl RoleInference {
    /// Create a new role inference result.
    pub fn new(role: Role, confidence: f64) -> Self {
        Self { role, confidence: confidence.clamp(0.0, 1.0) }
    }

    /// Check if confidence is low (below threshold).
    pub fn is_low_confidence(&self) -> bool {
        self.confidence < 0.7
    }
}

/// Warning generated when role inference has low confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleInferenceWarning {
    /// The token/region name this warning applies to
    pub token: String,
    /// The inferred role
    pub role: Role,
    /// The confidence score
    pub confidence: f64,
    /// Human-readable warning message
    pub message: String,
}

/// Context for role inference - provides sprite dimensions and region relationships.
#[derive(Debug, Clone)]
pub struct RoleInferenceContext {
    /// Sprite width in pixels
    pub sprite_width: u32,
    /// Sprite height in pixels
    pub sprite_height: u32,
}

impl RoleInferenceContext {
    /// Create a new inference context.
    pub fn new(width: u32, height: u32) -> Self {
        Self { sprite_width: width, sprite_height: height }
    }
}

/// Infers the semantic role of a region based on its properties.
pub struct RoleInferrer;

impl RoleInferrer {
    /// Infer the role of a region given its pixels and context.
    pub fn infer_role(
        pixels: &HashSet<(i32, i32)>,
        ctx: &RoleInferenceContext,
        color: Option<[u8; 4]>,
        adjacent_colors: &[[u8; 4]],
    ) -> Option<RoleInference> {
        if pixels.is_empty() {
            return None;
        }

        if let Some(inference) = Self::infer_boundary(pixels, ctx) {
            return Some(inference);
        }

        if let Some(inference) = Self::infer_anchor(pixels) {
            return Some(inference);
        }

        if let Some(col) = color {
            if !adjacent_colors.is_empty() {
                if let Some(inference) = Self::infer_shadow(col, adjacent_colors) {
                    return Some(inference);
                }
                if let Some(inference) = Self::infer_highlight(col, adjacent_colors) {
                    return Some(inference);
                }
            }
        }

        if let Some(inference) = Self::infer_fill(pixels, ctx) {
            return Some(inference);
        }

        None
    }

    /// Infer 'boundary' role: 1px wide regions on sprite edges.
    pub fn infer_boundary(
        pixels: &HashSet<(i32, i32)>,
        ctx: &RoleInferenceContext,
    ) -> Option<RoleInference> {
        if pixels.is_empty() {
            return None;
        }

        let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let is_thin = width == 1 || height == 1;

        let edge_pixels = pixels
            .iter()
            .filter(|(x, y)| {
                *x == 0
                    || *y == 0
                    || *x == (ctx.sprite_width as i32 - 1)
                    || *y == (ctx.sprite_height as i32 - 1)
            })
            .count();

        let edge_ratio = edge_pixels as f64 / pixels.len() as f64;

        if edge_pixels > 0 && is_thin {
            let confidence = (edge_ratio * 0.7 + 0.3).min(1.0);
            return Some(RoleInference::new(Role::Boundary, confidence));
        }

        if edge_ratio > 0.7 {
            return Some(RoleInference::new(Role::Boundary, edge_ratio * 0.8));
        }

        None
    }

    /// Infer 'anchor' role: small isolated regions (< 4 pixels).
    pub fn infer_anchor(pixels: &HashSet<(i32, i32)>) -> Option<RoleInference> {
        let size = pixels.len();

        if size >= 4 {
            return None;
        }

        let confidence = match size {
            1 => 1.0,
            2 => 0.9,
            3 => 0.8,
            _ => return None,
        };

        Some(RoleInference::new(Role::Anchor, confidence))
    }

    /// Infer 'fill' role: large interior regions.
    pub fn infer_fill(
        pixels: &HashSet<(i32, i32)>,
        ctx: &RoleInferenceContext,
    ) -> Option<RoleInference> {
        if pixels.is_empty() {
            return None;
        }

        let size = pixels.len();
        let sprite_area = (ctx.sprite_width * ctx.sprite_height) as usize;

        let size_ratio = size as f64 / sprite_area as f64;
        if size_ratio < 0.05 {
            return None;
        }

        let interior_pixels = pixels
            .iter()
            .filter(|(x, y)| {
                *x > 0
                    && *y > 0
                    && *x < (ctx.sprite_width as i32 - 1)
                    && *y < (ctx.sprite_height as i32 - 1)
            })
            .count();

        let interior_ratio = interior_pixels as f64 / size as f64;

        if interior_ratio < 0.5 {
            return None;
        }

        let confidence = (size_ratio.min(0.5) * 2.0 * 0.4 + interior_ratio * 0.6).min(1.0);

        Some(RoleInference::new(Role::Fill, confidence))
    }

    /// Infer 'shadow' role: darker than adjacent regions.
    pub fn infer_shadow(color: [u8; 4], adjacent_colors: &[[u8; 4]]) -> Option<RoleInference> {
        if adjacent_colors.is_empty() {
            return None;
        }

        let our_brightness = color_brightness(color);
        let avg_adjacent_brightness: f64 =
            adjacent_colors.iter().map(|c| color_brightness(*c)).sum::<f64>()
                / adjacent_colors.len() as f64;

        let brightness_diff = avg_adjacent_brightness - our_brightness;

        if brightness_diff < 0.15 {
            return None;
        }

        let confidence = ((brightness_diff - 0.15) / 0.25 * 0.3 + 0.7).min(1.0);

        Some(RoleInference::new(Role::Shadow, confidence))
    }

    /// Infer 'highlight' role: lighter than adjacent regions.
    pub fn infer_highlight(color: [u8; 4], adjacent_colors: &[[u8; 4]]) -> Option<RoleInference> {
        if adjacent_colors.is_empty() {
            return None;
        }

        let our_brightness = color_brightness(color);
        let avg_adjacent_brightness: f64 =
            adjacent_colors.iter().map(|c| color_brightness(*c)).sum::<f64>()
                / adjacent_colors.len() as f64;

        let brightness_diff = our_brightness - avg_adjacent_brightness;

        if brightness_diff < 0.15 {
            return None;
        }

        let confidence = ((brightness_diff - 0.15) / 0.25 * 0.3 + 0.7).min(1.0);

        Some(RoleInference::new(Role::Highlight, confidence))
    }

    /// Generate warnings for low-confidence inferences.
    pub fn generate_warnings(
        token: &str,
        inference: &RoleInference,
    ) -> Option<RoleInferenceWarning> {
        if inference.is_low_confidence() {
            Some(RoleInferenceWarning {
                token: token.to_string(),
                role: inference.role,
                confidence: inference.confidence,
                message: format!(
                    "Low confidence ({:.0}%) inferring '{}' role for token '{}'. \
                     Consider specifying the role explicitly.",
                    inference.confidence * 100.0,
                    inference.role,
                    token
                ),
            })
        } else {
            None
        }
    }
}

/// Calculate perceived brightness of an RGBA color (0.0 to 1.0).
pub(crate) fn color_brightness(color: [u8; 4]) -> f64 {
    let [r, g, b, _a] = color;
    (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0
}

/// Batch inference of roles for multiple regions.
pub fn infer_roles_batch(
    regions: &HashMap<String, (HashSet<(i32, i32)>, Option<[u8; 4]>)>,
    ctx: &RoleInferenceContext,
) -> (HashMap<String, RoleInference>, Vec<RoleInferenceWarning>) {
    let mut inferences = HashMap::new();
    let mut warnings = Vec::new();

    let all_colors: Vec<[u8; 4]> =
        regions.values().filter_map(|(_, color)| *color).collect();

    for (name, (pixels, color)) in regions {
        let adjacent: Vec<[u8; 4]> = all_colors
            .iter()
            .filter(|c| color.map(|col| **c != col).unwrap_or(true))
            .copied()
            .collect();

        if let Some(inference) = RoleInferrer::infer_role(pixels, ctx, *color, &adjacent) {
            if let Some(warning) = RoleInferrer::generate_warnings(name, &inference) {
                warnings.push(warning);
            }
            inferences.insert(name.clone(), inference);
        }
    }

    (inferences, warnings)
}
