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
