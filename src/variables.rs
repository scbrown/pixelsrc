//! CSS Variable Registry for palette variable resolution
//!
//! This module provides a registry for CSS custom properties (variables) that supports:
//! - Variable definition with `--name: value` syntax
//! - Variable resolution with `var(--name)` or `var(--name, fallback)` syntax
//! - Circular dependency detection
//! - Nested variable references
//!
//! # Example
//!
//! ```
//! use pixelsrc::variables::VariableRegistry;
//!
//! let mut registry = VariableRegistry::new();
//! registry.define("--primary", "#FF0000");
//! registry.define("--accent", "var(--primary)");
//!
//! assert_eq!(registry.resolve("var(--primary)").unwrap(), "#FF0000");
//! assert_eq!(registry.resolve("var(--accent)").unwrap(), "#FF0000");
//! assert_eq!(registry.resolve("var(--missing, blue)").unwrap(), "blue");
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Error type for variable resolution failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableError {
    /// Variable is not defined and no fallback was provided
    Undefined(String),
    /// Circular dependency detected in variable resolution
    Circular(Vec<String>),
    /// Invalid variable syntax
    InvalidSyntax(String),
    /// Maximum recursion depth exceeded
    MaxDepthExceeded,
}

impl fmt::Display for VariableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableError::Undefined(name) => {
                write!(f, "undefined variable '{}' with no fallback", name)
            }
            VariableError::Circular(chain) => {
                write!(f, "circular dependency: {}", chain.join(" -> "))
            }
            VariableError::InvalidSyntax(msg) => {
                write!(f, "invalid variable syntax: {}", msg)
            }
            VariableError::MaxDepthExceeded => {
                write!(f, "maximum variable resolution depth exceeded")
            }
        }
    }
}

impl std::error::Error for VariableError {}

/// Maximum depth for variable resolution to prevent stack overflow
const MAX_RESOLUTION_DEPTH: usize = 100;

/// Registry for CSS custom properties (variables)
///
/// Stores variable definitions and resolves `var()` references with support for
/// fallback values and circular dependency detection.
#[derive(Debug, Clone, Default)]
pub struct VariableRegistry {
    /// Variable name -> raw value (may contain var() references)
    variables: HashMap<String, String>,
}

impl VariableRegistry {
    /// Create a new empty variable registry
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    /// Define a CSS variable
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name (with or without `--` prefix)
    /// * `value` - Variable value (may contain `var()` references)
    ///
    /// # Example
    ///
    /// ```
    /// use pixelsrc::variables::VariableRegistry;
    ///
    /// let mut reg = VariableRegistry::new();
    /// reg.define("--primary", "#FF0000");
    /// reg.define("primary", "#FF0000"); // Also works, -- is added
    /// ```
    pub fn define(&mut self, name: &str, value: &str) {
        let normalized_name = Self::normalize_name(name);
        self.variables.insert(normalized_name, value.to_string());
    }

    /// Check if a variable is defined
    pub fn contains(&self, name: &str) -> bool {
        let normalized_name = Self::normalize_name(name);
        self.variables.contains_key(&normalized_name)
    }

    /// Get the raw (unresolved) value of a variable
    pub fn get_raw(&self, name: &str) -> Option<&str> {
        let normalized_name = Self::normalize_name(name);
        self.variables.get(&normalized_name).map(|s| s.as_str())
    }

    /// Get the number of defined variables
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Resolve a value, expanding any `var()` references
    ///
    /// Handles:
    /// - `var(--name)` - Simple reference
    /// - `var(--name, fallback)` - Reference with fallback
    /// - Nested `var()` in both variable values and fallbacks
    ///
    /// # Arguments
    ///
    /// * `value` - Value to resolve (may be a plain value or contain var() references)
    ///
    /// # Returns
    ///
    /// The fully resolved value, or an error if resolution fails.
    ///
    /// # Example
    ///
    /// ```
    /// use pixelsrc::variables::VariableRegistry;
    ///
    /// let mut reg = VariableRegistry::new();
    /// reg.define("--primary", "#FF0000");
    /// reg.define("--accent", "var(--primary)");
    ///
    /// assert_eq!(reg.resolve("#00FF00").unwrap(), "#00FF00"); // No var()
    /// assert_eq!(reg.resolve("var(--primary)").unwrap(), "#FF0000");
    /// assert_eq!(reg.resolve("var(--accent)").unwrap(), "#FF0000"); // Nested
    /// assert_eq!(reg.resolve("var(--missing, blue)").unwrap(), "blue"); // Fallback
    /// ```
    pub fn resolve(&self, value: &str) -> Result<String, VariableError> {
        let mut visited = HashSet::new();
        self.resolve_internal(value, &mut visited, 0)
    }

    /// Resolve a variable by name (without var() wrapper)
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name (with or without `--` prefix)
    ///
    /// # Returns
    ///
    /// The resolved value, or an error if the variable is undefined or circular.
    pub fn resolve_var(&self, name: &str) -> Result<String, VariableError> {
        let normalized = Self::normalize_name(name);
        match self.variables.get(&normalized) {
            Some(value) => self.resolve(value),
            None => Err(VariableError::Undefined(normalized)),
        }
    }

    /// Internal resolution with cycle detection
    fn resolve_internal(
        &self,
        value: &str,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> Result<String, VariableError> {
        if depth > MAX_RESOLUTION_DEPTH {
            return Err(VariableError::MaxDepthExceeded);
        }

        // If value doesn't contain var(), return as-is
        if !value.contains("var(") {
            return Ok(value.to_string());
        }

        // Process all var() references in the value
        let mut result = value.to_string();

        // Keep resolving until no more var() references
        loop {
            match self.find_var_reference(&result) {
                None => break,
                Some((start, end, var_name, fallback)) => {
                    let normalized_name = Self::normalize_name(&var_name);

                    // Check for circular reference
                    if visited.contains(&normalized_name) {
                        let mut chain: Vec<String> = visited.iter().cloned().collect();
                        chain.push(normalized_name);
                        return Err(VariableError::Circular(chain));
                    }

                    // Resolve the variable
                    let resolved_value = match self.variables.get(&normalized_name) {
                        Some(var_value) => {
                            visited.insert(normalized_name.clone());
                            let resolved = self.resolve_internal(var_value, visited, depth + 1)?;
                            visited.remove(&normalized_name);
                            resolved
                        }
                        None => {
                            // Variable not defined, use fallback if provided
                            match fallback {
                                Some(fb) => {
                                    // Fallback may also contain var() references
                                    self.resolve_internal(&fb, visited, depth + 1)?
                                }
                                None => {
                                    return Err(VariableError::Undefined(normalized_name));
                                }
                            }
                        }
                    };

                    // Replace the var() reference with resolved value
                    result = format!("{}{}{}", &result[..start], resolved_value, &result[end..]);
                }
            }
        }

        Ok(result)
    }

    /// Find the first var() reference in a string
    ///
    /// Returns (start, end, var_name, optional_fallback)
    fn find_var_reference(&self, s: &str) -> Option<(usize, usize, String, Option<String>)> {
        let start = s.find("var(")?;

        // Find matching closing paren, handling nested parens
        let rest = &s[start + 4..];
        let mut paren_depth = 1;
        let mut end_offset = 0;
        let mut comma_pos: Option<usize> = None;

        for (i, c) in rest.char_indices() {
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        end_offset = i;
                        break;
                    }
                }
                ',' if paren_depth == 1 && comma_pos.is_none() => {
                    comma_pos = Some(i);
                }
                _ => {}
            }
        }

        if paren_depth != 0 {
            return None; // Unmatched parentheses
        }

        let content = &rest[..end_offset];
        let end = start + 4 + end_offset + 1; // +1 for closing paren

        let (var_name, fallback) = match comma_pos {
            Some(comma) => {
                let name = content[..comma].trim().to_string();
                let fb = content[comma + 1..].trim().to_string();
                (name, Some(fb))
            }
            None => (content.trim().to_string(), None),
        };

        Some((start, end, var_name, fallback))
    }

    /// Normalize variable name to always have -- prefix
    fn normalize_name(name: &str) -> String {
        let trimmed = name.trim();
        if trimmed.starts_with("--") {
            trimmed.to_string()
        } else {
            format!("--{}", trimmed)
        }
    }

    /// Iterate over all defined variables
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.variables.iter()
    }

    /// Get all variable names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get_raw() {
        let mut reg = VariableRegistry::new();
        reg.define("--primary", "#FF0000");

        assert!(reg.contains("--primary"));
        assert!(reg.contains("primary")); // Also works without --
        assert_eq!(reg.get_raw("--primary"), Some("#FF0000"));
        assert_eq!(reg.get_raw("primary"), Some("#FF0000"));
    }

    #[test]
    fn test_define_without_dashes() {
        let mut reg = VariableRegistry::new();
        reg.define("accent", "#00FF00");

        assert!(reg.contains("--accent"));
        assert_eq!(reg.get_raw("--accent"), Some("#00FF00"));
    }

    #[test]
    fn test_resolve_plain_value() {
        let reg = VariableRegistry::new();
        assert_eq!(reg.resolve("#FF0000").unwrap(), "#FF0000");
        assert_eq!(reg.resolve("blue").unwrap(), "blue");
        assert_eq!(reg.resolve("rgb(255, 0, 0)").unwrap(), "rgb(255, 0, 0)");
    }

    #[test]
    fn test_resolve_simple_var() {
        let mut reg = VariableRegistry::new();
        reg.define("--primary", "#FF0000");

        assert_eq!(reg.resolve("var(--primary)").unwrap(), "#FF0000");
    }

    #[test]
    fn test_resolve_var_without_dashes() {
        let mut reg = VariableRegistry::new();
        reg.define("--primary", "#FF0000");

        // var() should work with or without -- (normalizes internally)
        assert_eq!(reg.resolve("var(primary)").unwrap(), "#FF0000");
    }

    #[test]
    fn test_resolve_var_with_fallback() {
        let reg = VariableRegistry::new();

        // Undefined variable with fallback
        assert_eq!(reg.resolve("var(--missing, blue)").unwrap(), "blue");
        assert_eq!(reg.resolve("var(--missing, #FF0000)").unwrap(), "#FF0000");
    }

    #[test]
    fn test_resolve_var_fallback_with_spaces() {
        let reg = VariableRegistry::new();

        // Fallback with spaces should be trimmed
        assert_eq!(reg.resolve("var(--missing,   blue  )").unwrap(), "blue");
    }

    #[test]
    fn test_resolve_nested_var() {
        let mut reg = VariableRegistry::new();
        reg.define("--primary", "#FF0000");
        reg.define("--accent", "var(--primary)");

        assert_eq!(reg.resolve("var(--accent)").unwrap(), "#FF0000");
    }

    #[test]
    fn test_resolve_deeply_nested() {
        let mut reg = VariableRegistry::new();
        reg.define("--base", "#FF0000");
        reg.define("--level1", "var(--base)");
        reg.define("--level2", "var(--level1)");
        reg.define("--level3", "var(--level2)");

        assert_eq!(reg.resolve("var(--level3)").unwrap(), "#FF0000");
    }

    #[test]
    fn test_resolve_var_in_fallback() {
        let mut reg = VariableRegistry::new();
        reg.define("--backup", "#00FF00");

        // Fallback contains a var() reference
        assert_eq!(reg.resolve("var(--missing, var(--backup))").unwrap(), "#00FF00");
    }

    #[test]
    fn test_resolve_multiple_vars_in_value() {
        let mut reg = VariableRegistry::new();
        reg.define("--r", "255");
        reg.define("--g", "128");
        reg.define("--b", "0");

        // Multiple var() in one value
        assert_eq!(reg.resolve("rgb(var(--r), var(--g), var(--b))").unwrap(), "rgb(255, 128, 0)");
    }

    #[test]
    fn test_error_undefined_no_fallback() {
        let reg = VariableRegistry::new();

        let err = reg.resolve("var(--undefined)").unwrap_err();
        assert!(matches!(err, VariableError::Undefined(name) if name == "--undefined"));
    }

    #[test]
    fn test_error_circular_simple() {
        let mut reg = VariableRegistry::new();
        reg.define("--a", "var(--b)");
        reg.define("--b", "var(--a)");

        let err = reg.resolve("var(--a)").unwrap_err();
        assert!(matches!(err, VariableError::Circular(_)));
    }

    #[test]
    fn test_error_circular_self_reference() {
        let mut reg = VariableRegistry::new();
        reg.define("--self", "var(--self)");

        let err = reg.resolve("var(--self)").unwrap_err();
        assert!(matches!(err, VariableError::Circular(_)));
    }

    #[test]
    fn test_error_circular_chain() {
        let mut reg = VariableRegistry::new();
        reg.define("--a", "var(--b)");
        reg.define("--b", "var(--c)");
        reg.define("--c", "var(--a)");

        let err = reg.resolve("var(--a)").unwrap_err();
        assert!(matches!(err, VariableError::Circular(_)));
    }

    #[test]
    fn test_resolve_var_by_name() {
        let mut reg = VariableRegistry::new();
        reg.define("--primary", "#FF0000");

        assert_eq!(reg.resolve_var("--primary").unwrap(), "#FF0000");
        assert_eq!(reg.resolve_var("primary").unwrap(), "#FF0000");
    }

    #[test]
    fn test_registry_len_and_empty() {
        let mut reg = VariableRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);

        reg.define("--a", "1");
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), 1);

        reg.define("--b", "2");
        assert_eq!(reg.len(), 2);

        reg.clear();
        assert!(reg.is_empty());
    }

    #[test]
    fn test_registry_iter() {
        let mut reg = VariableRegistry::new();
        reg.define("--a", "1");
        reg.define("--b", "2");

        let names: Vec<_> = reg.names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"--a".to_string()));
        assert!(names.contains(&&"--b".to_string()));
    }

    #[test]
    fn test_overwrite_variable() {
        let mut reg = VariableRegistry::new();
        reg.define("--color", "red");
        assert_eq!(reg.resolve("var(--color)").unwrap(), "red");

        reg.define("--color", "blue");
        assert_eq!(reg.resolve("var(--color)").unwrap(), "blue");
    }

    #[test]
    fn test_whitespace_handling() {
        let mut reg = VariableRegistry::new();
        reg.define("  --spaced  ", "#FF0000");

        assert!(reg.contains("--spaced"));
        assert_eq!(reg.resolve("var(  --spaced  )").unwrap(), "#FF0000");
    }

    #[test]
    fn test_error_display() {
        let err = VariableError::Undefined("--test".to_string());
        assert_eq!(err.to_string(), "undefined variable '--test' with no fallback");

        let err =
            VariableError::Circular(vec!["--a".to_string(), "--b".to_string(), "--a".to_string()]);
        assert_eq!(err.to_string(), "circular dependency: --a -> --b -> --a");

        let err = VariableError::InvalidSyntax("test error".to_string());
        assert_eq!(err.to_string(), "invalid variable syntax: test error");

        let err = VariableError::MaxDepthExceeded;
        assert_eq!(err.to_string(), "maximum variable resolution depth exceeded");
    }

    #[test]
    fn test_clone_and_debug() {
        let mut reg = VariableRegistry::new();
        reg.define("--test", "value");

        let cloned = reg.clone();
        assert_eq!(cloned.resolve("var(--test)").unwrap(), "value");

        // Debug should work without panicking
        let _ = format!("{:?}", reg);
    }

    #[test]
    fn test_complex_fallback_chain() {
        let mut reg = VariableRegistry::new();
        reg.define("--fallback", "final");

        // var(--missing, var(--also-missing, var(--fallback)))
        let result = reg.resolve("var(--missing, var(--also-missing, var(--fallback)))").unwrap();
        assert_eq!(result, "final");
    }

    #[test]
    fn test_mixed_content_with_var() {
        let mut reg = VariableRegistry::new();
        reg.define("--size", "10px");

        // var() embedded in other content
        assert_eq!(reg.resolve("border: 1px solid var(--size)").unwrap(), "border: 1px solid 10px");
    }

    #[test]
    fn test_nested_parens_in_fallback() {
        let reg = VariableRegistry::new();

        // Fallback contains function with parens
        let result = reg.resolve("var(--missing, rgb(255, 0, 0))").unwrap();
        assert_eq!(result, "rgb(255, 0, 0)");
    }

    #[test]
    fn test_default_trait() {
        let reg = VariableRegistry::default();
        assert!(reg.is_empty());
    }
}
