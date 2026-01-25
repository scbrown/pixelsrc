//! State rules system with CSS-like selectors
//!
//! Provides a way to define conditional styling based on sprite state.
//!
//! # Selectors
//!
//! - `[token=name]` - Select regions with a specific token name
//! - `[role=type]` - Select regions with a specific role (boundary, fill, etc.)
//! - `.state` - Select when sprite is in a specific state (hover, pressed, etc.)
//!
//! # Example
//!
//! ```ignore
//! {
//!   "type": "state-rules",
//!   "name": "button-states",
//!   "rules": [
//!     {
//!       "selector": ".hover [role=fill]",
//!       "apply": { "color": "#AAFFAA" }
//!     },
//!     {
//!       "selector": ".pressed [token=background]",
//!       "apply": { "color": "#888888" }
//!     }
//!   ]
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{RegionDef, Role};

/// A state rule that applies changes when conditions are met
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateRule {
    /// CSS-like selector to match regions
    pub selector: String,
    /// Changes to apply when selector matches
    pub apply: StateApplication,
}

/// Changes to apply when a state rule matches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StateApplication {
    /// Override the color for matched regions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Override visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// Override z-index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<i32>,
    /// Apply a transform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
}

/// A collection of state rules for a sprite
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateRules {
    /// Name of this state rules definition
    pub name: String,
    /// List of rules in priority order (later rules override earlier)
    pub rules: Vec<StateRule>,
}

/// Parsed selector components
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorPart {
    /// `.state` - matches when sprite is in this state
    State(String),
    /// `[token=name]` - matches region with this token name
    Token(String),
    /// `[role=type]` - matches region with this role
    Role(Role),
}

/// A parsed selector chain
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedSelector {
    /// Required state (if any)
    pub state: Option<String>,
    /// Token name filter (if any)
    pub token: Option<String>,
    /// Role filter (if any)
    pub role: Option<Role>,
}

impl ParsedSelector {
    /// Check if this selector matches a region in the given state
    pub fn matches(&self, current_state: Option<&str>, token_name: &str, region: &RegionDef) -> bool {
        // Check state condition
        if let Some(required_state) = &self.state {
            match current_state {
                Some(state) if state == required_state => {}
                _ => return false,
            }
        }

        // Check token name condition
        if let Some(required_token) = &self.token {
            if token_name != required_token {
                return false;
            }
        }

        // Check role condition
        if let Some(required_role) = &self.role {
            match &region.role {
                Some(role) if role == required_role => {}
                _ => return false,
            }
        }

        true
    }
}

/// Parse a CSS-like selector string into components
///
/// Supports:
/// - `.state` - state condition
/// - `[token=name]` - token name filter
/// - `[role=type]` - role filter
///
/// Multiple conditions can be combined with spaces.
pub fn parse_selector(selector: &str) -> Result<ParsedSelector, SelectorParseError> {
    let mut result = ParsedSelector::default();
    let selector = selector.trim();

    if selector.is_empty() {
        return Err(SelectorParseError::Empty);
    }

    // Split by whitespace and process each part
    for part in selector.split_whitespace() {
        if part.starts_with('.') {
            // State selector
            let state = &part[1..];
            if state.is_empty() {
                return Err(SelectorParseError::InvalidState("empty state name".into()));
            }
            result.state = Some(state.to_string());
        } else if part.starts_with('[') && part.ends_with(']') {
            // Attribute selector
            let inner = &part[1..part.len() - 1];
            parse_attribute_selector(inner, &mut result)?;
        } else {
            return Err(SelectorParseError::UnknownSyntax(part.to_string()));
        }
    }

    Ok(result)
}

fn parse_attribute_selector(attr: &str, result: &mut ParsedSelector) -> Result<(), SelectorParseError> {
    let parts: Vec<&str> = attr.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(SelectorParseError::InvalidAttribute(attr.to_string()));
    }

    let key = parts[0].trim();
    let value = parts[1].trim();

    match key {
        "token" => {
            result.token = Some(value.to_string());
        }
        "role" => {
            let role = parse_role(value)?;
            result.role = Some(role);
        }
        _ => {
            return Err(SelectorParseError::UnknownAttribute(key.to_string()));
        }
    }

    Ok(())
}

fn parse_role(s: &str) -> Result<Role, SelectorParseError> {
    match s.to_lowercase().as_str() {
        "boundary" => Ok(Role::Boundary),
        "anchor" => Ok(Role::Anchor),
        "fill" => Ok(Role::Fill),
        "shadow" => Ok(Role::Shadow),
        "highlight" => Ok(Role::Highlight),
        _ => Err(SelectorParseError::InvalidRole(s.to_string())),
    }
}

/// Errors that can occur when parsing selectors
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorParseError {
    /// Selector string is empty
    Empty,
    /// Invalid state syntax
    InvalidState(String),
    /// Invalid attribute selector syntax
    InvalidAttribute(String),
    /// Unknown attribute name
    UnknownAttribute(String),
    /// Invalid role value
    InvalidRole(String),
    /// Unknown selector syntax
    UnknownSyntax(String),
}

impl std::fmt::Display for SelectorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorParseError::Empty => write!(f, "selector cannot be empty"),
            SelectorParseError::InvalidState(s) => write!(f, "invalid state selector: {}", s),
            SelectorParseError::InvalidAttribute(s) => write!(f, "invalid attribute selector: {}", s),
            SelectorParseError::UnknownAttribute(s) => write!(f, "unknown attribute: {}", s),
            SelectorParseError::InvalidRole(s) => write!(f, "invalid role: {}", s),
            SelectorParseError::UnknownSyntax(s) => write!(f, "unknown selector syntax: {}", s),
        }
    }
}

impl std::error::Error for SelectorParseError {}

/// Apply state rules to regions and return modified regions
pub fn apply_state_rules(
    regions: &HashMap<String, RegionDef>,
    rules: &StateRules,
    current_state: Option<&str>,
    palette: &mut HashMap<String, String>,
) -> HashMap<String, RegionDef> {
    let mut result = regions.clone();

    for rule in &rules.rules {
        let selector = match parse_selector(&rule.selector) {
            Ok(s) => s,
            Err(_) => continue, // Skip invalid selectors
        };

        for (token_name, region) in result.iter_mut() {
            if selector.matches(current_state, token_name, region) {
                // Apply color change to palette
                if let Some(color) = &rule.apply.color {
                    palette.insert(format!("{{{}}}", token_name), color.clone());
                }

                // Apply z-index change
                if let Some(z) = rule.apply.z {
                    region.z = Some(z);
                }

                // Note: visibility and transform would need additional handling
                // in the rendering pipeline, which is beyond this MVP
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_selector() {
        let selector = parse_selector(".hover").unwrap();
        assert_eq!(selector.state, Some("hover".to_string()));
        assert!(selector.token.is_none());
        assert!(selector.role.is_none());
    }

    #[test]
    fn test_parse_token_selector() {
        let selector = parse_selector("[token=background]").unwrap();
        assert!(selector.state.is_none());
        assert_eq!(selector.token, Some("background".to_string()));
        assert!(selector.role.is_none());
    }

    #[test]
    fn test_parse_role_selector() {
        let selector = parse_selector("[role=fill]").unwrap();
        assert!(selector.state.is_none());
        assert!(selector.token.is_none());
        assert_eq!(selector.role, Some(Role::Fill));
    }

    #[test]
    fn test_parse_combined_selector() {
        let selector = parse_selector(".pressed [role=boundary]").unwrap();
        assert_eq!(selector.state, Some("pressed".to_string()));
        assert!(selector.token.is_none());
        assert_eq!(selector.role, Some(Role::Boundary));
    }

    #[test]
    fn test_parse_all_combined() {
        let selector = parse_selector(".active [token=btn] [role=fill]").unwrap();
        assert_eq!(selector.state, Some("active".to_string()));
        assert_eq!(selector.token, Some("btn".to_string()));
        assert_eq!(selector.role, Some(Role::Fill));
    }

    #[test]
    fn test_parse_empty_selector() {
        assert!(parse_selector("").is_err());
        assert!(parse_selector("   ").is_err());
    }

    #[test]
    fn test_parse_invalid_state() {
        assert!(parse_selector(".").is_err());
    }

    #[test]
    fn test_parse_invalid_attribute() {
        assert!(parse_selector("[invalid]").is_err());
        assert!(parse_selector("[no-equals-sign]").is_err());
    }

    #[test]
    fn test_parse_unknown_attribute() {
        assert!(parse_selector("[unknown=value]").is_err());
    }

    #[test]
    fn test_parse_invalid_role() {
        assert!(parse_selector("[role=invalid]").is_err());
    }

    #[test]
    fn test_parse_all_roles() {
        assert_eq!(parse_selector("[role=boundary]").unwrap().role, Some(Role::Boundary));
        assert_eq!(parse_selector("[role=anchor]").unwrap().role, Some(Role::Anchor));
        assert_eq!(parse_selector("[role=fill]").unwrap().role, Some(Role::Fill));
        assert_eq!(parse_selector("[role=shadow]").unwrap().role, Some(Role::Shadow));
        assert_eq!(parse_selector("[role=highlight]").unwrap().role, Some(Role::Highlight));
    }

    #[test]
    fn test_selector_matches_state() {
        let selector = ParsedSelector {
            state: Some("hover".to_string()),
            token: None,
            role: None,
        };
        let region = RegionDef::default();

        assert!(selector.matches(Some("hover"), "any", &region));
        assert!(!selector.matches(Some("pressed"), "any", &region));
        assert!(!selector.matches(None, "any", &region));
    }

    #[test]
    fn test_selector_matches_token() {
        let selector = ParsedSelector {
            state: None,
            token: Some("bg".to_string()),
            role: None,
        };
        let region = RegionDef::default();

        assert!(selector.matches(None, "bg", &region));
        assert!(!selector.matches(None, "fg", &region));
    }

    #[test]
    fn test_selector_matches_role() {
        let selector = ParsedSelector {
            state: None,
            token: None,
            role: Some(Role::Fill),
        };

        let mut region_with_role = RegionDef::default();
        region_with_role.role = Some(Role::Fill);

        let mut region_wrong_role = RegionDef::default();
        region_wrong_role.role = Some(Role::Boundary);

        let region_no_role = RegionDef::default();

        assert!(selector.matches(None, "any", &region_with_role));
        assert!(!selector.matches(None, "any", &region_wrong_role));
        assert!(!selector.matches(None, "any", &region_no_role));
    }

    #[test]
    fn test_selector_matches_combined() {
        let selector = ParsedSelector {
            state: Some("hover".to_string()),
            token: Some("btn".to_string()),
            role: Some(Role::Fill),
        };

        let mut matching_region = RegionDef::default();
        matching_region.role = Some(Role::Fill);

        // All conditions must match
        assert!(selector.matches(Some("hover"), "btn", &matching_region));

        // Fail on wrong state
        assert!(!selector.matches(Some("pressed"), "btn", &matching_region));

        // Fail on wrong token
        assert!(!selector.matches(Some("hover"), "other", &matching_region));

        // Fail on wrong role
        let mut wrong_role = RegionDef::default();
        wrong_role.role = Some(Role::Boundary);
        assert!(!selector.matches(Some("hover"), "btn", &wrong_role));
    }

    #[test]
    fn test_state_rules_serde() {
        let rules = StateRules {
            name: "button".to_string(),
            rules: vec![
                StateRule {
                    selector: ".hover [role=fill]".to_string(),
                    apply: StateApplication {
                        color: Some("#AAFFAA".to_string()),
                        visible: None,
                        z: None,
                        transform: None,
                    },
                },
            ],
        };

        let json = serde_json::to_string(&rules).unwrap();
        let parsed: StateRules = serde_json::from_str(&json).unwrap();
        assert_eq!(rules, parsed);
    }

    #[test]
    fn test_apply_state_rules() {
        let mut regions = HashMap::new();
        let mut fill_region = RegionDef::default();
        fill_region.role = Some(Role::Fill);
        regions.insert("bg".to_string(), fill_region);

        let rules = StateRules {
            name: "test".to_string(),
            rules: vec![StateRule {
                selector: ".hover [role=fill]".to_string(),
                apply: StateApplication {
                    color: Some("#FF0000".to_string()),
                    ..Default::default()
                },
            }],
        };

        let mut palette = HashMap::new();
        palette.insert("{bg}".to_string(), "#000000".to_string());

        // Without hover state - no change
        apply_state_rules(&regions, &rules, None, &mut palette);
        assert_eq!(palette.get("{bg}"), Some(&"#000000".to_string()));

        // With hover state - color changes
        apply_state_rules(&regions, &rules, Some("hover"), &mut palette);
        assert_eq!(palette.get("{bg}"), Some(&"#FF0000".to_string()));
    }
}
