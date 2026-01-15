//! Corpus analysis for pixelsrc files
//!
//! This module provides tools to analyze pixelsrc files and extract metrics
//! about token usage, structural patterns, and compression opportunities.

use crate::models::Sprite;
use crate::tokenizer::tokenize;

/// Statistics about RLE (Run-Length Encoding) compression opportunities
#[derive(Debug, Clone, Default)]
pub struct RleStats {
    /// Total number of tokens across all rows
    pub total_tokens: usize,
    /// Total number of runs (consecutive identical tokens)
    pub total_runs: usize,
    /// Total number of rows analyzed
    pub total_rows: usize,
    /// Sum of unique tokens per row (for averaging)
    pub total_unique_per_row: usize,
}

impl RleStats {
    /// Average tokens per row
    pub fn avg_tokens_per_row(&self) -> f64 {
        if self.total_rows == 0 {
            0.0
        } else {
            self.total_tokens as f64 / self.total_rows as f64
        }
    }

    /// Average unique tokens per row
    pub fn avg_unique_per_row(&self) -> f64 {
        if self.total_rows == 0 {
            0.0
        } else {
            self.total_unique_per_row as f64 / self.total_rows as f64
        }
    }

    /// Average runs per row
    pub fn avg_runs_per_row(&self) -> f64 {
        if self.total_rows == 0 {
            0.0
        } else {
            self.total_runs as f64 / self.total_rows as f64
        }
    }

    /// Estimated compression ratio from RLE encoding
    ///
    /// Returns the ratio of original tokens to RLE-encoded tokens.
    /// A ratio of 2.0 means the data could be stored in half the space.
    pub fn compression_ratio(&self) -> f64 {
        if self.total_runs == 0 {
            1.0
        } else {
            self.total_tokens as f64 / self.total_runs as f64
        }
    }

    /// Merge another RleStats into this one
    pub fn merge(&mut self, other: &RleStats) {
        self.total_tokens += other.total_tokens;
        self.total_runs += other.total_runs;
        self.total_rows += other.total_rows;
        self.total_unique_per_row += other.total_unique_per_row;
    }
}

/// Statistics about row repetition within sprites
#[derive(Debug, Clone, Default)]
pub struct RowRepetitionStats {
    /// Total number of rows across all sprites
    pub total_rows: usize,
    /// Number of rows that are identical to the previous row
    pub repeated_rows: usize,
    /// Number of sprites analyzed
    pub sprites_analyzed: usize,
}

impl RowRepetitionStats {
    /// Percentage of rows that are repetitions of the previous row
    pub fn repetition_percentage(&self) -> f64 {
        if self.total_rows == 0 {
            0.0
        } else {
            (self.repeated_rows as f64 / self.total_rows as f64) * 100.0
        }
    }

    /// Potential compression ratio from row repetition
    ///
    /// Returns the ratio of original rows to unique rows.
    pub fn compression_ratio(&self) -> f64 {
        let unique_rows = self.total_rows.saturating_sub(self.repeated_rows);
        if unique_rows == 0 {
            1.0
        } else {
            self.total_rows as f64 / unique_rows as f64
        }
    }

    /// Merge another RowRepetitionStats into this one
    pub fn merge(&mut self, other: &RowRepetitionStats) {
        self.total_rows += other.total_rows;
        self.repeated_rows += other.repeated_rows;
        self.sprites_analyzed += other.sprites_analyzed;
    }
}

/// Combined compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// RLE compression opportunities
    pub rle: RleStats,
    /// Row repetition statistics
    pub row_repetition: RowRepetitionStats,
}

impl CompressionStats {
    /// Merge another CompressionStats into this one
    pub fn merge(&mut self, other: &CompressionStats) {
        self.rle.merge(&other.rle);
        self.row_repetition.merge(&other.row_repetition);
    }

    /// Combined compression estimate (RLE + row repetition)
    ///
    /// This is a rough estimate assuming both optimizations are applied.
    pub fn combined_compression_ratio(&self) -> f64 {
        self.rle.compression_ratio() * self.row_repetition.compression_ratio()
    }
}

/// Analyzes compression opportunities in a sprite
pub struct CompressionEstimator;

impl CompressionEstimator {
    /// Analyze RLE opportunities in a single row
    ///
    /// Returns (token_count, run_count, unique_count) for the row.
    pub fn analyze_row_rle(row: &str) -> (usize, usize, usize) {
        let (tokens, _warnings) = tokenize(row);

        if tokens.is_empty() {
            return (0, 0, 0);
        }

        let token_count = tokens.len();

        // Count runs - consecutive identical tokens
        let mut run_count = 1;
        for i in 1..tokens.len() {
            if tokens[i] != tokens[i - 1] {
                run_count += 1;
            }
        }

        // Count unique tokens in this row
        let mut unique: Vec<&String> = tokens.iter().collect();
        unique.sort();
        unique.dedup();
        let unique_count = unique.len();

        (token_count, run_count, unique_count)
    }

    /// Analyze RLE opportunities across all rows of a sprite
    pub fn analyze_sprite_rle(sprite: &Sprite) -> RleStats {
        let mut stats = RleStats::default();

        for row in &sprite.grid {
            let (tokens, runs, unique) = Self::analyze_row_rle(row);
            stats.total_tokens += tokens;
            stats.total_runs += runs;
            stats.total_rows += 1;
            stats.total_unique_per_row += unique;
        }

        stats
    }

    /// Detect repeated rows in a sprite
    ///
    /// A row is considered "repeated" if it's identical to the previous row.
    pub fn analyze_row_repetition(sprite: &Sprite) -> RowRepetitionStats {
        let mut stats = RowRepetitionStats {
            total_rows: sprite.grid.len(),
            repeated_rows: 0,
            sprites_analyzed: 1,
        };

        if sprite.grid.len() < 2 {
            return stats;
        }

        for i in 1..sprite.grid.len() {
            if sprite.grid[i] == sprite.grid[i - 1] {
                stats.repeated_rows += 1;
            }
        }

        stats
    }

    /// Full compression analysis for a sprite
    pub fn analyze_sprite(sprite: &Sprite) -> CompressionStats {
        CompressionStats {
            rle: Self::analyze_sprite_rle(sprite),
            row_repetition: Self::analyze_row_repetition(sprite),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaletteRef;
    use std::collections::HashMap;

    fn make_sprite(name: &str, grid: Vec<&str>) -> Sprite {
        Sprite {
            name: name.to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),
            grid: grid.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_analyze_row_rle_simple() {
        // 6 tokens, 3 runs, 2 unique
        let (tokens, runs, unique) = CompressionEstimator::analyze_row_rle("{a}{a}{a}{b}{b}{b}");
        assert_eq!(tokens, 6);
        assert_eq!(runs, 2);
        assert_eq!(unique, 2);
    }

    #[test]
    fn test_analyze_row_rle_no_runs() {
        // Alternating tokens - no compression opportunity
        let (tokens, runs, unique) = CompressionEstimator::analyze_row_rle("{a}{b}{a}{b}");
        assert_eq!(tokens, 4);
        assert_eq!(runs, 4);
        assert_eq!(unique, 2);
    }

    #[test]
    fn test_analyze_row_rle_all_same() {
        // All same token - maximum compression
        let (tokens, runs, unique) = CompressionEstimator::analyze_row_rle("{a}{a}{a}{a}{a}");
        assert_eq!(tokens, 5);
        assert_eq!(runs, 1);
        assert_eq!(unique, 1);
    }

    #[test]
    fn test_analyze_row_rle_empty() {
        let (tokens, runs, unique) = CompressionEstimator::analyze_row_rle("");
        assert_eq!(tokens, 0);
        assert_eq!(runs, 0);
        assert_eq!(unique, 0);
    }

    #[test]
    fn test_analyze_sprite_rle() {
        let sprite = make_sprite(
            "test",
            vec![
                "{a}{a}{a}{b}{b}{b}", // 6 tokens, 2 runs
                "{a}{b}{a}{b}",       // 4 tokens, 4 runs
                "{a}{a}{a}{a}{a}",    // 5 tokens, 1 run
            ],
        );
        let stats = CompressionEstimator::analyze_sprite_rle(&sprite);
        assert_eq!(stats.total_tokens, 15);
        assert_eq!(stats.total_runs, 7);
        assert_eq!(stats.total_rows, 3);
        assert!((stats.compression_ratio() - 15.0 / 7.0).abs() < 0.001);
    }

    #[test]
    fn test_analyze_row_repetition_none() {
        let sprite = make_sprite("test", vec!["{a}{b}", "{b}{a}", "{a}{a}"]);
        let stats = CompressionEstimator::analyze_row_repetition(&sprite);
        assert_eq!(stats.total_rows, 3);
        assert_eq!(stats.repeated_rows, 0);
    }

    #[test]
    fn test_analyze_row_repetition_some() {
        let sprite = make_sprite(
            "test",
            vec![
                "{a}{a}", "{a}{a}", // repeated
                "{b}{b}", "{b}{b}", // repeated
                "{b}{b}", // repeated
            ],
        );
        let stats = CompressionEstimator::analyze_row_repetition(&sprite);
        assert_eq!(stats.total_rows, 5);
        assert_eq!(stats.repeated_rows, 3);
        assert!((stats.repetition_percentage() - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_analyze_row_repetition_all() {
        let sprite = make_sprite("test", vec!["{a}{a}", "{a}{a}", "{a}{a}", "{a}{a}"]);
        let stats = CompressionEstimator::analyze_row_repetition(&sprite);
        assert_eq!(stats.total_rows, 4);
        assert_eq!(stats.repeated_rows, 3);
        assert!((stats.compression_ratio() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_analyze_sprite_full() {
        let sprite = make_sprite(
            "test",
            vec![
                "{_}{_}{_}{_}{a}{a}{a}{_}{_}{_}{_}", // 11 tokens, lots of runs
                "{_}{_}{_}{_}{a}{a}{a}{_}{_}{_}{_}", // repeated row
                "{_}{_}{_}{a}{a}{a}{a}{a}{_}{_}{_}",
            ],
        );
        let stats = CompressionEstimator::analyze_sprite(&sprite);

        assert_eq!(stats.rle.total_tokens, 33);
        assert_eq!(stats.row_repetition.total_rows, 3);
        assert_eq!(stats.row_repetition.repeated_rows, 1);
    }

    #[test]
    fn test_rle_stats_merge() {
        let mut stats1 = RleStats {
            total_tokens: 10,
            total_runs: 5,
            total_rows: 2,
            total_unique_per_row: 4,
        };
        let stats2 = RleStats {
            total_tokens: 20,
            total_runs: 8,
            total_rows: 3,
            total_unique_per_row: 6,
        };
        stats1.merge(&stats2);
        assert_eq!(stats1.total_tokens, 30);
        assert_eq!(stats1.total_runs, 13);
        assert_eq!(stats1.total_rows, 5);
        assert_eq!(stats1.total_unique_per_row, 10);
    }

    #[test]
    fn test_realistic_hero_sprite() {
        // Simulating the hero_idle sprite pattern
        let sprite = make_sprite(
            "hero",
            vec![
                "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
                "{_}{_}{_}{_}{_}{_}{a}{a}{a}{a}{_}{_}{_}{_}{_}{_}",
                "{_}{_}{_}{_}{_}{a}{b}{b}{b}{b}{a}{_}{_}{_}{_}{_}",
                "{_}{_}{_}{_}{a}{b}{b}{b}{b}{b}{b}{a}{_}{_}{_}{_}",
                "{_}{_}{_}{_}{a}{b}{b}{b}{b}{b}{b}{a}{_}{_}{_}{_}", // repeated
            ],
        );
        let stats = CompressionEstimator::analyze_sprite(&sprite);

        // Should have good RLE compression (lots of {_} runs)
        assert!(stats.rle.compression_ratio() > 1.5);

        // Should detect 1 repeated row
        assert_eq!(stats.row_repetition.repeated_rows, 1);
    }
}
