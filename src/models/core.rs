//! Core types used across models.

use serde::{Deserialize, Serialize};

/// A value that can be either a literal value or a CSS variable reference.
///
/// Used for composition layer properties like `opacity` and `blend` that can
/// use `var()` syntax to reference CSS custom properties.
///
/// # Examples
///
/// ```
/// use pixelsrc::models::VarOr;
///
/// // Can be deserialized from either a literal or a var() string
/// let literal: VarOr<f64> = serde_json::from_str("0.5").unwrap();
/// let var_ref: VarOr<f64> = serde_json::from_str("\"var(--opacity)\"").unwrap();
///
/// assert!(matches!(literal, VarOr::Value(0.5)));
/// assert!(matches!(var_ref, VarOr::Var(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VarOr<T> {
    /// A literal value
    Value(T),
    /// A CSS variable reference (e.g., "var(--name)" or "var(--name, fallback)")
    Var(String),
}

impl<T: Default> Default for VarOr<T> {
    fn default() -> Self {
        VarOr::Value(T::default())
    }
}

impl<T> VarOr<T> {
    /// Returns true if this is a var() reference
    pub fn is_var(&self) -> bool {
        matches!(self, VarOr::Var(_))
    }

    /// Returns true if this is a literal value
    pub fn is_value(&self) -> bool {
        matches!(self, VarOr::Value(_))
    }

    /// Returns the literal value if present
    pub fn as_value(&self) -> Option<&T> {
        match self {
            VarOr::Value(v) => Some(v),
            VarOr::Var(_) => None,
        }
    }

    /// Returns the var() string if present
    pub fn as_var(&self) -> Option<&str> {
        match self {
            VarOr::Value(_) => None,
            VarOr::Var(s) => Some(s),
        }
    }
}

impl<T: Copy> VarOr<T> {
    /// Get the value, returning None if it's a var() reference
    pub fn value(&self) -> Option<T> {
        match self {
            VarOr::Value(v) => Some(*v),
            VarOr::Var(_) => None,
        }
    }
}

impl From<f64> for VarOr<f64> {
    fn from(v: f64) -> Self {
        VarOr::Value(v)
    }
}

impl From<String> for VarOr<f64> {
    fn from(s: String) -> Self {
        VarOr::Var(s)
    }
}

/// A duration value that can be either a raw millisecond number or a CSS time string.
///
/// # Examples
///
/// ```
/// use pixelsrc::models::Duration;
///
/// // Can be deserialized from either format
/// let ms: Duration = serde_json::from_str("100").unwrap();
/// let css: Duration = serde_json::from_str("\"500ms\"").unwrap();
///
/// assert_eq!(ms.as_milliseconds(), Some(100));
/// assert_eq!(css.as_milliseconds(), Some(500));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Duration {
    /// Raw milliseconds (backwards compatible)
    Milliseconds(u32),
    /// CSS time string (e.g., "500ms", "1s", "0.5s")
    CssString(String),
}

impl Duration {
    /// Parse the duration and return milliseconds.
    ///
    /// Returns `None` if the CSS string cannot be parsed.
    pub fn as_milliseconds(&self) -> Option<u32> {
        match self {
            Duration::Milliseconds(ms) => Some(*ms),
            Duration::CssString(s) => parse_css_duration(s),
        }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration::Milliseconds(100)
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Duration::Milliseconds(ms) => write!(f, "{}", ms),
            Duration::CssString(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl From<u32> for Duration {
    fn from(ms: u32) -> Self {
        Duration::Milliseconds(ms)
    }
}

impl From<&str> for Duration {
    fn from(s: &str) -> Self {
        Duration::CssString(s.to_string())
    }
}

/// Parse a CSS duration string into milliseconds.
///
/// Supports:
/// - `<number>ms` - milliseconds (e.g., "500ms")
/// - `<number>s` - seconds (e.g., "1.5s")
pub fn parse_css_duration(s: &str) -> Option<u32> {
    let s = s.trim().to_lowercase();

    if let Some(ms_str) = s.strip_suffix("ms") {
        ms_str.trim().parse::<f64>().ok().map(|v| v as u32)
    } else if let Some(s_str) = s.strip_suffix('s') {
        s_str.trim().parse::<f64>().ok().map(|v| (v * 1000.0) as u32)
    } else {
        // Try parsing as raw number (assume milliseconds)
        s.parse::<f64>().ok().map(|v| v as u32)
    }
}
