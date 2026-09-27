//! Transform operations for sprites and animations
//!
//! Supports both CLI transforms (`pxl transform`) and format attributes
//! (`"transform": ["mirror-h", "rotate:90"]`).
//!
//! # Module Structure
//!
//! - [`types`] - Core transform types and error definitions
//! - [`dither`] - Dither patterns for pixel art effects
//! - [`parsing`] - Transform parsing from strings and JSON
//! - [`css`] - CSS transform string parsing
//! - [`apply`] - Transform application to images and animations
//! - [`expression`] - Expression evaluation for keyframe animations
//! - [`anchor`] - Anchor-preserving scaling for pixel art

pub mod anchor;
pub mod apply;
pub mod css;
pub mod dither;
pub mod expression;
pub mod parsing;
pub mod types;

// Re-export main types at the module level for convenience
pub use anchor::{scale_image, scale_image_with_anchor_preservation, AnchorBounds};
pub use apply::{
    apply_animation_transform, apply_frame_offset, apply_hold, apply_image_transform,
    apply_image_transforms, apply_pingpong, apply_reverse, is_animation_transform,
};
pub use css::{parse_css_transform, CssTransform, CssTransformError};
pub use dither::{DitherPattern, GradientDirection};
pub use expression::{
    generate_frame_transforms, interpolate_keyframes, ExpressionError, ExpressionEvaluator,
};
pub use parsing::{parse_token_pair, parse_transform_str, parse_transform_value};
pub use types::{explain_transform, Transform, TransformError};

/// Result type alias for transform operations.
pub type Result<T> = std::result::Result<T, TransformError>;

/// Apply image transform specifications in order, retaining valid operations in
/// lenient rendering. Callers turn the returned diagnostics into strict errors.
pub(crate) fn apply_image_transform_specs(
    mut image: image::RgbaImage,
    specs: &[crate::models::TransformSpec],
    palette: Option<&std::collections::HashMap<String, String>>,
) -> (image::RgbaImage, Vec<String>) {
    use crate::models::TransformSpec;

    let mut warnings = Vec::new();
    for spec in specs {
        let parsed = match spec {
            TransformSpec::String(s) => parse_transform_str(s),
            TransformSpec::Object { op, params } => {
                let mut object: serde_json::Map<String, serde_json::Value> =
                    params.clone().into_iter().collect();
                object.insert("op".to_string(), serde_json::Value::String(op.clone()));
                parse_transform_value(&serde_json::Value::Object(object))
            }
        };
        match parsed {
            Ok(transform) if is_animation_transform(&transform) => continue,
            Ok(transform) => match apply_image_transform(&image, &transform, palette) {
                Ok(transformed) => image = transformed,
                Err(error) => warnings.push(format!("transform error: {}", error)),
            },
            Err(error) => warnings.push(format!("invalid transform: {}", error)),
        }
    }
    (image, warnings)
}
