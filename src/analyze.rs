//! Corpus analysis for Pixelsrc files
//!
//! Provides tools to analyze pixelsrc files and extract metrics about:
//! - Token frequency and co-occurrence
//! - Dimensional distribution
//! - Structural patterns
//! - Compression opportunities

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::models::{Sprite, TtpObject};
use crate::parser::parse_stream;
use crate::tokenizer::tokenize;

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
        let mut items: Vec<_> = self
            .pairs
            .iter()
            .map(|((a, b), count)| ((a, b), *count))
            .collect();
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
        Self {
            min_family_size: min_size,
        }
    }

    /// Detect token families from a token counter.
    pub fn detect(&self, counter: &TokenCounter) -> Vec<TokenFamily> {
        // Group tokens by their base name (prefix before _ or variant suffix)
        let mut prefix_groups: HashMap<String, Vec<(String, usize)>> = HashMap::new();

        for (token, count) in counter.sorted_by_frequency() {
            // Extract base prefix from token like {skin_light} -> "skin"
            if let Some(base) = self.extract_prefix(token) {
                prefix_groups
                    .entry(base)
                    .or_default()
                    .push((token.clone(), *count));
            }
        }

        // Build families from groups that meet minimum size
        let mut families: Vec<TokenFamily> = prefix_groups
            .into_iter()
            .filter(|(_, tokens)| tokens.len() >= self.min_family_size)
            .map(|(prefix, tokens)| {
                let total_count = tokens.iter().map(|(_, c)| c).sum();
                let token_names = tokens.into_iter().map(|(t, _)| t).collect();
                TokenFamily {
                    prefix,
                    tokens: token_names,
                    total_count,
                }
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
        let base = inner
            .split('_')
            .next()
            .unwrap_or(inner)
            .trim_end_matches(|c: char| c.is_ascii_digit());

        if base.is_empty() {
            None
        } else {
            Some(base.to_string())
        }
    }
}

/// Dimensional statistics for sprites.
#[derive(Debug, Default)]
pub struct DimensionStats {
    /// Map from (width, height) to count
    dimensions: HashMap<(u32, u32), usize>,
}

impl DimensionStats {
    /// Create new dimension stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a sprite's dimensions.
    pub fn add(&mut self, width: u32, height: u32) {
        *self.dimensions.entry((width, height)).or_insert(0) += 1;
    }

    /// Get dimension counts sorted by frequency.
    pub fn sorted_by_frequency(&self) -> Vec<((u32, u32), usize)> {
        let mut items: Vec<_> = self.dimensions.iter().map(|(k, v)| (*k, *v)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items
    }

    /// Get total sprite count.
    pub fn total(&self) -> usize {
        self.dimensions.values().sum()
    }
}

/// Aggregated analysis report for a corpus.
#[derive(Debug, Default)]
pub struct AnalysisReport {
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Number of files that failed to parse
    pub files_failed: usize,
    /// Total sprites found
    pub total_sprites: usize,
    /// Total palettes found
    pub total_palettes: usize,
    /// Total compositions found
    pub total_compositions: usize,
    /// Total animations found
    pub total_animations: usize,
    /// Total variants found
    pub total_variants: usize,
    /// Token frequency counter
    pub token_counter: TokenCounter,
    /// Token co-occurrence matrix
    pub co_occurrence: CoOccurrenceMatrix,
    /// Dimension statistics
    pub dimension_stats: DimensionStats,
    /// Palette sizes (tokens per palette)
    pub palette_sizes: Vec<usize>,
    /// Files that had parse errors
    pub failed_files: Vec<(PathBuf, String)>,
}

impl AnalysisReport {
    /// Create a new empty analysis report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a sprite and add its data to the report.
    pub fn analyze_sprite(&mut self, sprite: &Sprite) {
        self.total_sprites += 1;

        // Extract dimensions from grid
        let height = sprite.grid.len() as u32;
        let width = if height > 0 {
            // Tokenize first row to get width
            let (tokens, _) = tokenize(&sprite.grid[0]);
            tokens.len() as u32
        } else {
            0
        };

        // Use explicit size if provided, otherwise use grid dimensions
        let (w, h) = match sprite.size {
            Some([sw, sh]) => (sw, sh),
            None => (width, height),
        };
        self.dimension_stats.add(w, h);

        // Collect all unique tokens in this sprite for co-occurrence tracking
        let mut sprite_tokens: HashSet<String> = HashSet::new();

        // Count all tokens in the grid
        for row in &sprite.grid {
            let (tokens, _) = tokenize(row);
            for token in tokens {
                self.token_counter.add(&token);
                sprite_tokens.insert(token);
            }
        }

        // Record co-occurrences
        self.co_occurrence.record_sprite(&sprite_tokens);
    }

    /// Analyze a single file and add results to the report.
    pub fn analyze_file(&mut self, path: &Path) -> Result<(), String> {
        let file = fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let result = parse_stream(reader);

        for obj in result.objects {
            match obj {
                TtpObject::Sprite(sprite) => {
                    self.analyze_sprite(&sprite);
                }
                TtpObject::Palette(palette) => {
                    self.total_palettes += 1;
                    self.palette_sizes.push(palette.colors.len());
                }
                TtpObject::Composition(_) => {
                    self.total_compositions += 1;
                }
                TtpObject::Animation(_) => {
                    self.total_animations += 1;
                }
                TtpObject::Variant(_) => {
                    self.total_variants += 1;
                }
            }
        }

        self.files_analyzed += 1;
        Ok(())
    }

    /// Get average palette size.
    pub fn avg_palette_size(&self) -> f64 {
        if self.palette_sizes.is_empty() {
            return 0.0;
        }
        let sum: usize = self.palette_sizes.iter().sum();
        sum as f64 / self.palette_sizes.len() as f64
    }

    /// Detect token families from the collected tokens.
    pub fn token_families(&self) -> Vec<TokenFamily> {
        let detector = TokenFamilyDetector::new();
        detector.detect(&self.token_counter)
    }
}

/// Collect files to analyze based on input specification.
pub fn collect_files(
    files: &[PathBuf],
    dir: Option<&Path>,
    recursive: bool,
) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();

    // If specific files provided, use them
    if !files.is_empty() {
        for path in files {
            if path.exists() {
                result.push(path.clone());
            } else {
                return Err(format!("File not found: {}", path.display()));
            }
        }
        return Ok(result);
    }

    // If directory provided, scan it
    if let Some(dir_path) = dir {
        if !dir_path.exists() {
            return Err(format!("Directory not found: {}", dir_path.display()));
        }
        if !dir_path.is_dir() {
            return Err(format!("Not a directory: {}", dir_path.display()));
        }

        collect_from_directory(dir_path, recursive, &mut result)?;
    }

    Ok(result)
}

/// Recursively collect .jsonl and .pxl files from a directory.
fn collect_from_directory(
    dir: &Path,
    recursive: bool,
    result: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if recursive {
                collect_from_directory(&path, recursive, result)?;
            }
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext == "jsonl" || ext == "pxl" {
                result.push(path);
            }
        }
    }

    Ok(())
}

/// Format the analysis report as text.
pub fn format_report_text(report: &AnalysisReport) -> String {
    let mut output = String::new();

    // Header
    output.push_str("Pixelsrc Analysis Report\n");
    output.push_str("========================\n");
    output.push_str(&format!("Files analyzed: {}\n", report.files_analyzed));
    if report.files_failed > 0 {
        output.push_str(&format!("Files failed: {}\n", report.files_failed));
    }
    output.push_str(&format!("Total sprites: {}\n", report.total_sprites));
    output.push_str(&format!("Total palettes: {}\n", report.total_palettes));
    if report.total_compositions > 0 {
        output.push_str(&format!(
            "Total compositions: {}\n",
            report.total_compositions
        ));
    }
    if report.total_animations > 0 {
        output.push_str(&format!("Total animations: {}\n", report.total_animations));
    }
    if report.total_variants > 0 {
        output.push_str(&format!("Total variants: {}\n", report.total_variants));
    }
    output.push('\n');

    // Token frequency (top 10)
    if report.token_counter.unique_count() > 0 {
        output.push_str("TOKEN FREQUENCY (top 10)\n");
        output.push_str("────────────────────────\n");
        for (token, count) in report.token_counter.top_n(10) {
            let percentage = report.token_counter.percentage(token);
            output.push_str(&format!(
                "  {:12} {:>8}  ({:.1}%)\n",
                token, count, percentage
            ));
        }
        output.push('\n');
    }

    // Token co-occurrence (top 5 pairs)
    if report.co_occurrence.pair_count() > 0 {
        output.push_str("TOKEN CO-OCCURRENCE (top 5 pairs)\n");
        output.push_str("─────────────────────────────────\n");
        for ((token1, token2), count) in report.co_occurrence.top_n(5) {
            output.push_str(&format!(
                "  {} + {:12} {:>4} sprites\n",
                token1, token2, count
            ));
        }
        output.push('\n');
    }

    // Token families
    let families = report.token_families();
    if !families.is_empty() {
        output.push_str("TOKEN FAMILIES\n");
        output.push_str("──────────────\n");
        for family in families.iter().take(5) {
            let tokens_str = family.tokens.join(", ");
            output.push_str(&format!(
                "  {:<12} {} tokens, {} occurrences\n",
                format!("{{{}*}}", family.prefix),
                family.tokens.len(),
                family.total_count
            ));
            // Show first few members if space allows
            if family.tokens.len() <= 4 {
                output.push_str(&format!("               {}\n", tokens_str));
            } else {
                let preview: Vec<_> = family.tokens.iter().take(3).cloned().collect();
                output.push_str(&format!(
                    "               {}, ... +{} more\n",
                    preview.join(", "),
                    family.tokens.len() - 3
                ));
            }
        }
        output.push('\n');
    }

    // Dimensions
    if report.dimension_stats.total() > 0 {
        output.push_str("DIMENSIONS\n");
        output.push_str("──────────\n");
        let sorted_dims = report.dimension_stats.sorted_by_frequency();
        let total = report.dimension_stats.total();
        for ((w, h), count) in sorted_dims.iter().take(5) {
            let percentage = (*count as f64 / total as f64) * 100.0;
            output.push_str(&format!(
                "  {:>3}x{:<3}    {:>4} sprites ({:.0}%)\n",
                w, h, count, percentage
            ));
        }
        output.push('\n');
    }

    // Palette patterns
    if !report.palette_sizes.is_empty() {
        output.push_str("PALETTE PATTERNS\n");
        output.push_str("────────────────\n");
        output.push_str(&format!(
            "  Avg tokens/palette:    {:.1}\n",
            report.avg_palette_size()
        ));
        output.push('\n');
    }

    // Failed files (if any)
    if !report.failed_files.is_empty() {
        output.push_str("FAILED FILES\n");
        output.push_str("────────────\n");
        for (path, error) in &report.failed_files {
            output.push_str(&format!("  {}: {}\n", path.display(), error));
        }
        output.push('\n');
    }

    output
}

// ============================================================================
// Compression Estimation (13.4)
// ============================================================================

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

    #[test]
    fn test_token_counter_basic() {
        let mut counter = TokenCounter::new();
        counter.add("{_}");
        counter.add("{_}");
        counter.add("{x}");

        assert_eq!(counter.get("{_}"), 2);
        assert_eq!(counter.get("{x}"), 1);
        assert_eq!(counter.get("{y}"), 0);
        assert_eq!(counter.total(), 3);
        assert_eq!(counter.unique_count(), 2);
    }

    #[test]
    fn test_token_counter_percentage() {
        let mut counter = TokenCounter::new();
        counter.add("{_}");
        counter.add("{_}");
        counter.add("{x}");
        counter.add("{x}");

        assert!((counter.percentage("{_}") - 50.0).abs() < 0.01);
        assert!((counter.percentage("{x}") - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_token_counter_top_n() {
        let mut counter = TokenCounter::new();
        counter.add_count("{_}", 100);
        counter.add_count("{x}", 50);
        counter.add_count("{y}", 25);

        let top = counter.top_n(2);
        assert_eq!(top.len(), 2);
        assert_eq!(*top[0].0, "{_}");
        assert_eq!(*top[1].0, "{x}");
    }

    #[test]
    fn test_dimension_stats() {
        let mut stats = DimensionStats::new();
        stats.add(16, 16);
        stats.add(16, 16);
        stats.add(8, 8);

        assert_eq!(stats.total(), 3);
        let sorted = stats.sorted_by_frequency();
        assert_eq!(sorted[0], ((16, 16), 2));
        assert_eq!(sorted[1], ((8, 8), 1));
    }

    #[test]
    fn test_analysis_report_avg_palette_size() {
        let mut report = AnalysisReport::new();
        report.palette_sizes = vec![4, 6, 8];
        assert!((report.avg_palette_size() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_analysis_report_empty_palette() {
        let report = AnalysisReport::new();
        assert!((report.avg_palette_size() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_co_occurrence_basic() {
        let mut matrix = CoOccurrenceMatrix::new();

        let mut tokens1: HashSet<String> = HashSet::new();
        tokens1.insert("{skin}".to_string());
        tokens1.insert("{outline}".to_string());
        tokens1.insert("{_}".to_string());

        let mut tokens2: HashSet<String> = HashSet::new();
        tokens2.insert("{skin}".to_string());
        tokens2.insert("{hair}".to_string());

        matrix.record_sprite(&tokens1);
        matrix.record_sprite(&tokens2);

        // skin+outline appears in 1 sprite
        assert_eq!(matrix.get("{skin}", "{outline}"), 1);
        // skin appears in both sprites but with different partners
        assert_eq!(matrix.get("{skin}", "{_}"), 1);
        assert_eq!(matrix.get("{skin}", "{hair}"), 1);
        // hair+outline never co-occur
        assert_eq!(matrix.get("{hair}", "{outline}"), 0);
    }

    #[test]
    fn test_co_occurrence_top_n() {
        let mut matrix = CoOccurrenceMatrix::new();

        // Record same sprite tokens twice to get higher counts
        let mut tokens: HashSet<String> = HashSet::new();
        tokens.insert("{skin}".to_string());
        tokens.insert("{outline}".to_string());

        matrix.record_sprite(&tokens);
        matrix.record_sprite(&tokens);

        let top = matrix.top_n(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].1, 2); // 2 occurrences
    }

    #[test]
    fn test_co_occurrence_pairs_for_token() {
        let mut matrix = CoOccurrenceMatrix::new();

        let mut tokens1: HashSet<String> = HashSet::new();
        tokens1.insert("{skin}".to_string());
        tokens1.insert("{a}".to_string());
        tokens1.insert("{b}".to_string());

        matrix.record_sprite(&tokens1);

        let pairs = matrix.pairs_for_token("{skin}");
        assert_eq!(pairs.len(), 2);
        // Both {a} and {b} appear once with {skin}
        assert!(pairs.iter().all(|(_, c)| *c == 1));
    }

    #[test]
    fn test_token_family_detector() {
        let mut counter = TokenCounter::new();
        counter.add_count("{skin}", 100);
        counter.add_count("{skin_light}", 50);
        counter.add_count("{skin_shadow}", 30);
        counter.add_count("{hair}", 80);
        counter.add_count("{hair_dark}", 40);
        counter.add_count("{outline}", 200);

        let detector = TokenFamilyDetector::new();
        let families = detector.detect(&counter);

        // Should find "skin" and "hair" families
        assert!(families.len() >= 2);

        // Find the skin family
        let skin_family = families.iter().find(|f| f.prefix == "skin");
        assert!(skin_family.is_some());
        let skin = skin_family.unwrap();
        assert_eq!(skin.tokens.len(), 3);
        assert_eq!(skin.total_count, 180); // 100 + 50 + 30
    }

    #[test]
    fn test_token_family_prefix_extraction() {
        let detector = TokenFamilyDetector::new();

        // Test extract_prefix directly via a simple test
        let mut counter = TokenCounter::new();
        counter.add("{_}"); // Should be skipped (transparency)
        counter.add("{skin}");
        counter.add("{skin_light}");
        counter.add("{color1}");
        counter.add("{color2}");

        let families = detector.detect(&counter);

        // Should have "skin" and "color" families
        let prefixes: Vec<_> = families.iter().map(|f| f.prefix.as_str()).collect();
        assert!(prefixes.contains(&"skin"));
        assert!(prefixes.contains(&"color"));
        // {_} should not create a family
        assert!(!prefixes.iter().any(|p| p.is_empty() || *p == "_"));
    }

    // ========================================================================
    // Compression estimation tests (13.4)
    // ========================================================================

    fn make_compression_test_sprite(name: &str, grid: Vec<&str>) -> Sprite {
        use crate::models::PaletteRef;
        use std::collections::HashMap;
        Sprite {
            name: name.to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([
                ("{_}".to_string(), "#00000000".to_string()),
                ("{a}".to_string(), "#FF0000".to_string()),
                ("{b}".to_string(), "#00FF00".to_string()),
            ])),
            grid: grid.into_iter().map(|s| s.to_string()).collect(),
            metadata: None,
        }
    }

    #[test]
    fn test_analyze_row_rle_simple() {
        // 6 tokens, 2 runs, 2 unique
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
        let sprite = make_compression_test_sprite(
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
        let sprite = make_compression_test_sprite("test", vec!["{a}{b}", "{b}{a}", "{a}{a}"]);
        let stats = CompressionEstimator::analyze_row_repetition(&sprite);
        assert_eq!(stats.total_rows, 3);
        assert_eq!(stats.repeated_rows, 0);
    }

    #[test]
    fn test_analyze_row_repetition_some() {
        let sprite = make_compression_test_sprite(
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
        let sprite = make_compression_test_sprite("test", vec!["{a}{a}", "{a}{a}", "{a}{a}", "{a}{a}"]);
        let stats = CompressionEstimator::analyze_row_repetition(&sprite);
        assert_eq!(stats.total_rows, 4);
        assert_eq!(stats.repeated_rows, 3);
        assert!((stats.compression_ratio() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_analyze_sprite_full() {
        let sprite = make_compression_test_sprite(
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
        let sprite = make_compression_test_sprite(
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
