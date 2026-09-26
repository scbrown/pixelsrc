//! Token family detection for semantic grouping

use std::collections::HashMap;

use super::tokens::TokenCounter;

/// Token family - a group of semantically related tokens with common prefix.
#[derive(Debug, Clone)]
pub struct TokenFamily {
    /// The common prefix (e.g., "skin" for {skin}, {skin_light}, {skin_shadow})
    pub prefix: String,
    /// All tokens in this family
    pub tokens: Vec<String>,
    /// Total occurrences across all tokens in family
    pub total_count: usize,
}

/// Detects and groups tokens into semantic families based on naming patterns.
#[derive(Debug, Default)]
pub struct TokenFamilyDetector {
    /// Minimum family size to report
    min_family_size: usize,
}

impl TokenFamilyDetector {
    /// Create a new detector with default settings.
    pub fn new() -> Self {
        Self { min_family_size: 2 }
    }

    /// Create a detector with custom minimum family size.
    pub fn with_min_size(min_size: usize) -> Self {
        Self { min_family_size: min_size }
    }

    /// Detect token families from a token counter.
    pub fn detect(&self, counter: &TokenCounter) -> Vec<TokenFamily> {
        // Group tokens by their base name (prefix before _ or variant suffix)
        let mut prefix_groups: HashMap<String, Vec<(String, usize)>> = HashMap::new();

        for (token, count) in counter.sorted_by_frequency() {
            // Extract base prefix from token like {skin_light} -> "skin"
            if let Some(base) = self.extract_prefix(token) {
                prefix_groups.entry(base).or_default().push((token.clone(), *count));
            }
        }

        // Build families from groups that meet minimum size
        let mut families: Vec<TokenFamily> = prefix_groups
            .into_iter()
            .filter(|(_, tokens)| tokens.len() >= self.min_family_size)
            .map(|(prefix, tokens)| {
                let total_count = tokens.iter().map(|(_, c)| c).sum();
                let token_names = tokens.into_iter().map(|(t, _)| t).collect();
                TokenFamily { prefix, tokens: token_names, total_count }
            })
            .collect();

        // Sort by total count descending
        families.sort_by(|a, b| b.total_count.cmp(&a.total_count));
        families
    }

    /// Extract the base prefix from a token.
    /// {skin} -> "skin"
    /// {skin_light} -> "skin"
    /// {hair_dark} -> "hair"
    /// {_} -> None (transparency token)
    fn extract_prefix(&self, token: &str) -> Option<String> {
        // Strip braces
        let inner = token.trim_start_matches('{').trim_end_matches('}');

        // Skip transparency token
        if inner == "_" || inner.is_empty() {
            return None;
        }

        // Find the base prefix (before first underscore or digit suffix)
        let base =
            inner.split('_').next().unwrap_or(inner).trim_end_matches(|c: char| c.is_ascii_digit());

        if base.is_empty() {
            None
        } else {
            Some(base.to_string())
        }
    }
}
