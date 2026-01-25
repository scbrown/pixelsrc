//! Compression estimation for sprite analysis

use crate::models::Sprite;

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
    /// Note: Grid format is deprecated - always returns (0, 0, 0).
    #[allow(unused_variables)]
    pub fn analyze_row_rle(_row: &str) -> (usize, usize, usize) {
        // Grid format is deprecated - RLE analysis not available
        (0, 0, 0)
    }

    /// Analyze RLE opportunities across all rows of a sprite.
    ///
    /// NOTE: This function is deprecated for v2 region-based sprites.
    /// Grid-based analysis is no longer supported. See TTP-7i4v.
    #[allow(unused_variables)]
    pub fn analyze_sprite_rle(_sprite: &Sprite) -> RleStats {
        // TODO: Update for v2 region-based format
        RleStats::default()
    }

    /// Detect repeated rows in a sprite.
    ///
    /// NOTE: This function is deprecated for v2 region-based sprites.
    /// Grid-based analysis is no longer supported. See TTP-7i4v.
    #[allow(unused_variables)]
    pub fn analyze_row_repetition(_sprite: &Sprite) -> RowRepetitionStats {
        // TODO: Update for v2 region-based format
        RowRepetitionStats::default()
    }

    /// Full compression analysis for a sprite.
    ///
    /// NOTE: This function is deprecated for v2 region-based sprites.
    /// Grid-based analysis is no longer supported. See TTP-7i4v.
    pub fn analyze_sprite(sprite: &Sprite) -> CompressionStats {
        CompressionStats {
            rle: Self::analyze_sprite_rle(sprite),
            row_repetition: Self::analyze_row_repetition(sprite),
        }
    }
}
