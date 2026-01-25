//! Token frequency tracking for corpus analysis

use std::collections::{HashMap, HashSet};

/// Tracks token frequency across a corpus.
#[derive(Debug, Default)]
pub struct TokenCounter {
    /// Map from token to occurrence count
    counts: HashMap<String, usize>,
    /// Total token occurrences
    total: usize,
}

impl TokenCounter {
    /// Create a new empty token counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a token occurrence.
    pub fn add(&mut self, token: &str) {
        *self.counts.entry(token.to_string()).or_insert(0) += 1;
        self.total += 1;
    }

    /// Add multiple occurrences of a token.
    pub fn add_count(&mut self, token: &str, count: usize) {
        *self.counts.entry(token.to_string()).or_insert(0) += count;
        self.total += count;
    }

    /// Get the count for a specific token.
    pub fn get(&self, token: &str) -> usize {
        self.counts.get(token).copied().unwrap_or(0)
    }

    /// Get total token occurrences.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Get the number of unique tokens.
    pub fn unique_count(&self) -> usize {
        self.counts.len()
    }

    /// Get tokens sorted by frequency (descending).
    pub fn sorted_by_frequency(&self) -> Vec<(&String, &usize)> {
        let mut items: Vec<_> = self.counts.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));
        items
    }

    /// Get the top N tokens by frequency.
    pub fn top_n(&self, n: usize) -> Vec<(&String, &usize)> {
        self.sorted_by_frequency().into_iter().take(n).collect()
    }

    /// Calculate percentage for a token.
    pub fn percentage(&self, token: &str) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        let count = self.get(token);
        (count as f64 / self.total as f64) * 100.0
    }
}

/// Tracks token co-occurrence across sprites.
///
/// Records which tokens appear together in the same sprite, enabling
/// analysis of token relationships and discovery of semantic groups.
#[derive(Debug, Default)]
pub struct CoOccurrenceMatrix {
    /// Map from (token1, token2) pair to sprite count where they co-occur
    /// Pairs are stored in sorted order to avoid duplicates
    pairs: HashMap<(String, String), usize>,
}

impl CoOccurrenceMatrix {
    /// Create a new empty co-occurrence matrix.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that a set of tokens appeared together in one sprite.
    pub fn record_sprite(&mut self, tokens: &HashSet<String>) {
        let mut token_list: Vec<_> = tokens.iter().collect();
        token_list.sort();

        // Record all unique pairs
        for i in 0..token_list.len() {
            for j in (i + 1)..token_list.len() {
                let pair = (token_list[i].clone(), token_list[j].clone());
                *self.pairs.entry(pair).or_insert(0) += 1;
            }
        }
    }

    /// Get the co-occurrence count for a specific pair.
    pub fn get(&self, token1: &str, token2: &str) -> usize {
        // Ensure sorted order for lookup
        let pair = if token1 < token2 {
            (token1.to_string(), token2.to_string())
        } else {
            (token2.to_string(), token1.to_string())
        };
        self.pairs.get(&pair).copied().unwrap_or(0)
    }

    /// Get top N token pairs by co-occurrence count.
    pub fn top_n(&self, n: usize) -> Vec<((&String, &String), usize)> {
        let mut items: Vec<_> = self.pairs.iter().map(|((a, b), count)| ((a, b), *count)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(n);
        items
    }

    /// Get all pairs involving a specific token, sorted by count.
    pub fn pairs_for_token(&self, token: &str) -> Vec<(&String, usize)> {
        let mut results: Vec<_> = self
            .pairs
            .iter()
            .filter_map(|((a, b), count)| {
                if a == token {
                    Some((b, *count))
                } else if b == token {
                    Some((a, *count))
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Get total number of unique pairs recorded.
    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }
}
