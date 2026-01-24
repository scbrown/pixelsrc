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
use crate::shapes;
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
                TtpObject::Particle(_) => {
                    // Particle systems are runtime constructs, not analyzed
                }
                TtpObject::Transform(_) => {
                    // User-defined transforms are handled during rendering
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
        output.push_str(&format!("Total compositions: {}\n", report.total_compositions));
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
            output.push_str(&format!("  {:12} {:>8}  ({:.1}%)\n", token, count, percentage));
        }
        output.push('\n');
    }

    // Token co-occurrence (top 5 pairs)
    if report.co_occurrence.pair_count() > 0 {
        output.push_str("TOKEN CO-OCCURRENCE (top 5 pairs)\n");
        output.push_str("─────────────────────────────────\n");
        for ((token1, token2), count) in report.co_occurrence.top_n(5) {
            output.push_str(&format!("  {} + {:12} {:>4} sprites\n", token1, token2, count));
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
        output.push_str(&format!("  Avg tokens/palette:    {:.1}\n", report.avg_palette_size()));
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

// ============================================================================

// Shape Detection (24.12)
// ============================================================================

/// Result of shape detection with confidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeDetection<T> {
    /// The detected shape parameters
    pub shape: T,
    /// Confidence score from 0.0 to 1.0
    pub confidence: f64,
}

impl<T> ShapeDetection<T> {
    /// Create a new shape detection result.
    pub fn new(shape: T, confidence: f64) -> Self {
        Self { shape, confidence: confidence.clamp(0.0, 1.0) }
    }
}

/// Detected shape type with parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedShape {
    /// Filled rectangle: [x, y, width, height]
    Rect([i32; 4]),
    /// Stroked (hollow) rectangle: [x, y, width, height]
    Stroke([i32; 4]),
    /// Ellipse: [cx, cy, rx, ry]
    Ellipse([i32; 4]),
    /// Line defined by endpoints
    Line(Vec<[i32; 2]>),
    /// Polygon defined by vertices (fallback)
    Polygon(Vec<[i32; 2]>),
}

/// Compute bounding box of a pixel set.
///
/// Returns (min_x, min_y, max_x, max_y) or None if empty.
fn bounding_box(pixels: &HashSet<(i32, i32)>) -> Option<(i32, i32, i32, i32)> {
    if pixels.is_empty() {
        return None;
    }

    let min_x = pixels.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = pixels.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = pixels.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = pixels.iter().map(|(_, y)| *y).max().unwrap();

    Some((min_x, min_y, max_x, max_y))
}

/// Detect if pixels form a filled rectangle.
///
/// Checks if the pixel set exactly matches a filled rectangle by comparing
/// the pixel count to the bounding box area.
///
/// Returns the rectangle parameters [x, y, width, height] if detected.

// Symmetry Detection (24.13)
// ============================================================================

/// Represents the type of symmetry detected in a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symmetric {
    /// Symmetric along the X-axis (left-right mirroring)
    X,
    /// Symmetric along the Y-axis (top-bottom mirroring)
    Y,
    /// Symmetric along both axes
    XY,
}

/// Detects symmetry in a pixel buffer.
///
/// Analyzes the given pixel data to determine if the image is symmetric
/// along the X-axis (left-right), Y-axis (top-bottom), or both.
///
/// # Arguments
///
/// * `pixels` - A slice of RGBA pixel data (4 bytes per pixel)
/// * `width` - The width of the image in pixels
/// * `height` - The height of the image in pixels
///
/// # Returns
///
/// * `Some(Symmetric::XY)` if symmetric along both axes
/// * `Some(Symmetric::X)` if symmetric along X-axis only (left-right)
/// * `Some(Symmetric::Y)` if symmetric along Y-axis only (top-bottom)
/// * `None` if not symmetric
///
/// # Examples
///
/// ```

/// use pixelsrc::analyze::detect_rect;
/// use std::collections::HashSet;
///
/// let pixels: HashSet<(i32, i32)> = [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().collect();
/// let result = detect_rect(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// assert_eq!(detection.shape, [0, 0, 2, 2]);
/// assert!((detection.confidence - 1.0).abs() < 0.001);
/// ```
pub fn detect_rect(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<[i32; 4]>> {
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let expected_area = (width * height) as usize;
    let actual_area = pixels.len();

    if actual_area == expected_area {
        // Perfect match - all pixels within bounding box are filled
        Some(ShapeDetection::new([min_x, min_y, width, height], 1.0))
    } else {
        // Calculate confidence based on fill ratio
        let fill_ratio = actual_area as f64 / expected_area as f64;
        // Only consider it a rectangle if fill ratio is very high
        if fill_ratio >= 0.95 {
            Some(ShapeDetection::new([min_x, min_y, width, height], fill_ratio))
        } else {
            None
        }
    }
}

/// Detect if pixels form a stroked (hollow) rectangle.
///
/// Checks if the pixel set matches a hollow rectangle by verifying:
/// 1. The outline pixels match a stroked rectangle
/// 2. The interior is empty
///
/// Returns the rectangle parameters [x, y, width, height] if detected.
/// Assumes thickness of 1 pixel.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_stroke;
/// use pixelsrc::shapes::rasterize_stroke;
///
/// let pixels = rasterize_stroke(0, 0, 5, 5, 1);
/// let result = detect_stroke(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// assert_eq!(detection.shape, [0, 0, 5, 5]);
/// ```
pub fn detect_stroke(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<[i32; 4]>> {
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    // A stroke needs to be at least 3x3 to have an interior
    if width < 3 || height < 3 {
        return None;
    }

    // Check that interior is empty (for 1-pixel thickness)
    let mut has_interior_pixel = false;
    for x in (min_x + 1)..max_x {
        for y in (min_y + 1)..max_y {
            if pixels.contains(&(x, y)) {
                has_interior_pixel = true;
                break;
            }
        }
        if has_interior_pixel {
            break;
        }
    }

    if has_interior_pixel {
        return None;
    }

    // Generate expected stroke and compare
    let expected = shapes::rasterize_stroke(min_x, min_y, width, height, 1);
    let matching = pixels.intersection(&expected).count();
    let confidence = matching as f64 / pixels.len().max(expected.len()) as f64;

    if confidence >= 0.95 {
        Some(ShapeDetection::new([min_x, min_y, width, height], confidence))
    } else {
        None
    }
}

/// Detect if pixels form a Bresenham line.
///
/// Checks if the pixel set matches a line by testing all possible endpoint
/// combinations and finding the best match using Bresenham's algorithm.
///
/// Returns the line endpoints as a vector of [x, y] pairs if detected.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_line;
/// use pixelsrc::shapes::rasterize_line;
///
/// let pixels = rasterize_line((0, 0), (5, 3));
/// let result = detect_line(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// assert_eq!(detection.shape.len(), 2);
/// ```
pub fn detect_line(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<Vec<[i32; 2]>>> {
    if pixels.is_empty() {
        return None;
    }

    // For very small pixel sets, they're trivially lines
    if pixels.len() == 1 {
        let (x, y) = *pixels.iter().next().unwrap();
        return Some(ShapeDetection::new(vec![[x, y], [x, y]], 1.0));
    }

    if pixels.len() == 2 {
        let points: Vec<_> = pixels.iter().collect();
        let (x0, y0) = *points[0];
        let (x1, y1) = *points[1];
        // Check if they're adjacent (valid 2-pixel line)
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        if dx <= 1 && dy <= 1 {
            return Some(ShapeDetection::new(vec![[x0, y0], [x1, y1]], 1.0));
        }
    }

    // Find extreme points that could be endpoints
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    // Collect candidate endpoints (pixels at the extremes)
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for &(x, y) in pixels {
        if x == min_x || x == max_x || y == min_y || y == max_y {
            candidates.push((x, y));
        }
    }

    // Try all pairs of candidates to find the best line fit
    let mut best_match: Option<((i32, i32), (i32, i32), f64)> = None;

    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let p0 = candidates[i];
            let p1 = candidates[j];

            let line_pixels = shapes::rasterize_line(p0, p1);

            // Check if the rasterized line matches the input pixels
            if line_pixels.len() == pixels.len() {
                let matching = pixels.intersection(&line_pixels).count();
                let confidence = matching as f64 / pixels.len() as f64;

                if confidence > best_match.map(|(_, _, c)| c).unwrap_or(0.0) {
                    best_match = Some((p0, p1, confidence));
                }

/// use pixelsrc::analyze::{detect_symmetry, Symmetric};
///
/// // A 2x2 symmetric pattern (all same color)
/// let pixels: Vec<u8> = vec![
///     255, 0, 0, 255,  255, 0, 0, 255,  // row 0
///     255, 0, 0, 255,  255, 0, 0, 255,  // row 1
/// ];
/// assert_eq!(detect_symmetry(&pixels, 2, 2), Some(Symmetric::XY));
///
/// // Not symmetric
/// let asymmetric: Vec<u8> = vec![
///     255, 0, 0, 255,  0, 255, 0, 255,  // row 0
///     0, 0, 255, 255,  255, 255, 0, 255,  // row 1
/// ];
/// assert_eq!(detect_symmetry(&asymmetric, 2, 2), None);
/// ```
pub fn detect_symmetry(pixels: &[u8], width: u32, height: u32) -> Option<Symmetric> {
    let width = width as usize;
    let height = height as usize;
    let bytes_per_pixel = 4;

    // Check for empty or invalid input
    if width == 0 || height == 0 || pixels.len() != width * height * bytes_per_pixel {
        return None;
    }

    let x_symmetric = is_x_symmetric(pixels, width, height, bytes_per_pixel);
    let y_symmetric = is_y_symmetric(pixels, width, height, bytes_per_pixel);

    match (x_symmetric, y_symmetric) {
        (true, true) => Some(Symmetric::XY),
        (true, false) => Some(Symmetric::X),
        (false, true) => Some(Symmetric::Y),
        (false, false) => None,
    }
}

/// Checks if the image is symmetric along the X-axis (left-right mirroring).
///
/// Compares columns from the left edge with corresponding columns from the right edge.
fn is_x_symmetric(pixels: &[u8], width: usize, height: usize, bpp: usize) -> bool {
    let half_width = width / 2;

    for y in 0..height {
        for x in 0..half_width {
            let left_idx = (y * width + x) * bpp;
            let right_idx = (y * width + (width - 1 - x)) * bpp;

            // Compare all 4 bytes (RGBA)
            if pixels[left_idx..left_idx + bpp] != pixels[right_idx..right_idx + bpp] {
                return false;
            }
        }
    }


    best_match.and_then(|(p0, p1, confidence)| {
        if confidence >= 0.95 {
            Some(ShapeDetection::new(vec![[p0.0, p0.1], [p1.0, p1.1]], confidence))
        } else {
            None
        }
    })
}

/// Detect if pixels form an ellipse.
///
/// Uses a roundness heuristic to determine if the pixel set matches an ellipse
/// by comparing the actual pixel count to the expected ellipse area.
///
/// Returns the ellipse parameters [cx, cy, rx, ry] if detected.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::detect_ellipse;
/// use pixelsrc::shapes::rasterize_ellipse;
///
/// let pixels = rasterize_ellipse(10, 10, 5, 3);
/// let result = detect_ellipse(&pixels);
/// assert!(result.is_some());
/// let detection = result.unwrap();
/// // Center should be close to (10, 10)
/// assert!((detection.shape[0] - 10).abs() <= 1);
/// assert!((detection.shape[1] - 10).abs() <= 1);
/// ```
pub fn detect_ellipse(pixels: &HashSet<(i32, i32)>) -> Option<ShapeDetection<[i32; 4]>> {
    let (min_x, min_y, max_x, max_y) = bounding_box(pixels)?;

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    // Ellipse needs reasonable size
    if width < 3 || height < 3 {
        return None;
    }

    // Calculate center and radii
    let cx = (min_x + max_x) / 2;
    let cy = (min_y + max_y) / 2;
    let rx = width / 2;
    let ry = height / 2;

    // Skip if radii are too small
    if rx < 1 || ry < 1 {
        return None;
    }

    // Generate expected ellipse and compare
    let expected = shapes::rasterize_ellipse(cx, cy, rx, ry);

    if expected.is_empty() {
        return None;
    }

    // Calculate overlap between actual and expected pixels
    let intersection = pixels.intersection(&expected).count();
    let union_size = pixels.len() + expected.len() - intersection;

    // Jaccard similarity (intersection over union)
    let jaccard = if union_size > 0 { intersection as f64 / union_size as f64 } else { 0.0 };

    // Also check the expected ellipse area vs actual
    // Expected area of ellipse = π * rx * ry
    let expected_area = std::f64::consts::PI * (rx as f64) * (ry as f64);
    let area_ratio = pixels.len() as f64 / expected_area;

    // Combine metrics for confidence
    // Good ellipse: Jaccard > 0.8 and area ratio close to 1.0
    let area_confidence = 1.0 - (area_ratio - 1.0).abs().min(1.0);
    let confidence = (jaccard + area_confidence) / 2.0;

    if confidence >= 0.7 {
        Some(ShapeDetection::new([cx, cy, rx, ry], confidence))
    } else {
        None
    }
}

/// Extract polygon vertices from a set of pixels using convex hull.
///
/// Computes the convex hull of the pixel set to find the vertices
/// that define the polygon boundary.
fn extract_polygon_vertices(pixels: &HashSet<(i32, i32)>) -> Vec<[i32; 2]> {
    if pixels.is_empty() {
        return Vec::new();
    }

    let mut points: Vec<(i32, i32)> = pixels.iter().copied().collect();

    // Find convex hull using Graham scan
    // First, find the lowest point (and leftmost if tied)
    points.sort_by(|a, b| {
        if a.1 != b.1 {
            a.1.cmp(&b.1)
        } else {
            a.0.cmp(&b.0)
        }
    });

    let start = points[0];

    // Sort remaining points by polar angle from start
    points[1..].sort_by(|a, b| {
        let cross = cross_product(start, *a, *b);
        if cross == 0 {
            // Collinear - sort by distance
            let dist_a = (a.0 - start.0).pow(2) + (a.1 - start.1).pow(2);
            let dist_b = (b.0 - start.0).pow(2) + (b.1 - start.1).pow(2);
            dist_a.cmp(&dist_b)
        } else if cross > 0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    // Build convex hull
    let mut hull: Vec<(i32, i32)> = Vec::new();
    for point in points {
        while hull.len() >= 2 {
            let top = hull[hull.len() - 1];
            let second = hull[hull.len() - 2];
            if cross_product(second, top, point) <= 0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(point);
    }

    hull.into_iter().map(|(x, y)| [x, y]).collect()
}

/// Compute cross product for convex hull algorithm.
fn cross_product(o: (i32, i32), a: (i32, i32), b: (i32, i32)) -> i64 {
    let ox = o.0 as i64;
    let oy = o.1 as i64;
    let ax = a.0 as i64;
    let ay = a.1 as i64;
    let bx = b.0 as i64;
    let by = b.1 as i64;
    (ax - ox) * (by - oy) - (ay - oy) * (bx - ox)
}

/// Detect the shape of a pixel set with confidence scoring.
///
/// Tries to detect the shape in order of specificity:
/// 1. Line (simplest)
/// 2. Stroked rectangle (hollow)
/// 3. Filled rectangle
/// 4. Ellipse
/// 5. Falls back to polygon
///
/// Returns the detected shape with confidence score.
///
/// # Examples
///
/// ```
/// use pixelsrc::analyze::{detect_shape, DetectedShape};
/// use pixelsrc::shapes::rasterize_rect;
///
/// let pixels = rasterize_rect(0, 0, 4, 3);
/// let (shape, confidence) = detect_shape(&pixels);
/// assert!(matches!(shape, DetectedShape::Rect(_)));
/// assert!(confidence >= 0.95);
/// ```
pub fn detect_shape(pixels: &HashSet<(i32, i32)>) -> (DetectedShape, f64) {
    if pixels.is_empty() {
        return (DetectedShape::Polygon(Vec::new()), 0.0);
    }

    // Try line detection first (simplest shape)
    if let Some(detection) = detect_line(pixels) {
        if detection.confidence >= 0.95 {
            return (DetectedShape::Line(detection.shape), detection.confidence);
        }
    }

    // Try stroked rectangle (before filled, as strokes are more specific)
    if let Some(detection) = detect_stroke(pixels) {
        if detection.confidence >= 0.95 {
            return (DetectedShape::Stroke(detection.shape), detection.confidence);
        }
    }

    // Try filled rectangle
    if let Some(detection) = detect_rect(pixels) {
        if detection.confidence >= 0.95 {
            return (DetectedShape::Rect(detection.shape), detection.confidence);
        }
    }

    // Try ellipse
    if let Some(detection) = detect_ellipse(pixels) {
        if detection.confidence >= 0.7 {
            return (DetectedShape::Ellipse(detection.shape), detection.confidence);
        }
    }

    // Fall back to polygon
    let vertices = extract_polygon_vertices(pixels);
    // Polygon confidence based on how well the convex hull represents the pixels
    let hull_pixels = shapes::rasterize_polygon(
        &vertices.iter().map(|[x, y]| (*x, *y)).collect::<Vec<_>>(),
    );
    let intersection = pixels.intersection(&hull_pixels).count();
    let confidence = if pixels.is_empty() {
        0.0
    } else {
        intersection as f64 / pixels.len() as f64
    };

    (DetectedShape::Polygon(vertices), confidence)

    true
}

/// Checks if the image is symmetric along the Y-axis (top-bottom mirroring).
///
/// Compares rows from the top edge with corresponding rows from the bottom edge.
fn is_y_symmetric(pixels: &[u8], width: usize, height: usize, bpp: usize) -> bool {
    let half_height = height / 2;

    for y in 0..half_height {
        let top_row_start = y * width * bpp;
        let bottom_row_start = (height - 1 - y) * width * bpp;

        // Compare entire rows
        let top_row = &pixels[top_row_start..top_row_start + width * bpp];
        let bottom_row = &pixels[bottom_row_start..bottom_row_start + width * bpp];

        if top_row != bottom_row {
            return false;
        }
    }

    true
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
            ..Default::default()
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
        let sprite =
            make_compression_test_sprite("test", vec!["{a}{a}", "{a}{a}", "{a}{a}", "{a}{a}"]);
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
        let mut stats1 =
            RleStats { total_tokens: 10, total_runs: 5, total_rows: 2, total_unique_per_row: 4 };
        let stats2 =
            RleStats { total_tokens: 20, total_runs: 8, total_rows: 3, total_unique_per_row: 6 };
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

    // ========================================================================

    // Shape Detection Tests (24.12)
    // ========================================================================

    #[test]
    fn test_detect_rect_perfect() {
        let pixels = shapes::rasterize_rect(0, 0, 4, 3);
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [0, 0, 4, 3]);
        assert!((detection.confidence - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_rect_with_offset() {
        let pixels = shapes::rasterize_rect(5, 10, 3, 2);
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [5, 10, 3, 2]);
    }

    #[test]
    fn test_detect_rect_single_pixel() {
        let pixels: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [0, 0, 1, 1]);
    }

    #[test]
    fn test_detect_rect_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = detect_rect(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_rect_not_a_rect() {
        // L-shape is not a rectangle
        let pixels: HashSet<(i32, i32)> =
            [(0, 0), (1, 0), (2, 0), (0, 1), (0, 2)].into_iter().collect();
        let result = detect_rect(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_stroke_perfect() {
        let pixels = shapes::rasterize_stroke(0, 0, 5, 5, 1);
        let result = detect_stroke(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [0, 0, 5, 5]);
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_stroke_with_offset() {
        let pixels = shapes::rasterize_stroke(3, 7, 6, 4, 1);
        let result = detect_stroke(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [3, 7, 6, 4]);
    }

    #[test]
    fn test_detect_stroke_too_small() {
        // 2x2 stroke doesn't have interior, so we can't detect it as a stroke
        let pixels = shapes::rasterize_stroke(0, 0, 2, 2, 1);
        let result = detect_stroke(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_stroke_filled_rect_is_not_stroke() {
        let pixels = shapes::rasterize_rect(0, 0, 5, 5);
        let result = detect_stroke(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_line_horizontal() {
        let pixels = shapes::rasterize_line((0, 0), (5, 0));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape.len(), 2);
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_vertical() {
        let pixels = shapes::rasterize_line((0, 0), (0, 5));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_diagonal() {
        let pixels = shapes::rasterize_line((0, 0), (5, 5));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_steep() {
        let pixels = shapes::rasterize_line((0, 0), (3, 7));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_single_pixel() {
        let pixels: HashSet<(i32, i32)> = [(5, 5)].into_iter().collect();
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, vec![[5, 5], [5, 5]]);
    }

    #[test]
    fn test_detect_line_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = detect_line(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_ellipse_circle() {
        let pixels = shapes::rasterize_ellipse(10, 10, 5, 5);
        let result = detect_ellipse(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        // Center should be close to (10, 10)
        assert!((detection.shape[0] - 10).abs() <= 1);
        assert!((detection.shape[1] - 10).abs() <= 1);
        assert!(detection.confidence >= 0.7);
    }

    #[test]
    fn test_detect_ellipse_horizontal() {
        let pixels = shapes::rasterize_ellipse(10, 10, 6, 3);
        let result = detect_ellipse(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.7);
    }

    #[test]
    fn test_detect_ellipse_vertical() {
        let pixels = shapes::rasterize_ellipse(10, 10, 3, 6);
        let result = detect_ellipse(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.7);
    }

    #[test]
    fn test_detect_ellipse_too_small() {
        let pixels = shapes::rasterize_ellipse(1, 1, 1, 1);
        let result = detect_ellipse(&pixels);
        // Very small ellipses may not be detected
        // This is acceptable - they look like rectangles anyway
        if let Some(detection) = result {
            assert!(detection.confidence >= 0.7);
        }
    }

    #[test]
    fn test_detect_shape_rect() {
        let pixels = shapes::rasterize_rect(0, 0, 5, 4);
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Rect([0, 0, 5, 4])));
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_detect_shape_stroke() {
        let pixels = shapes::rasterize_stroke(0, 0, 6, 6, 1);
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Stroke([0, 0, 6, 6])));
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_detect_shape_line() {
        let pixels = shapes::rasterize_line((0, 0), (10, 5));
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Line(_)));
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_detect_shape_ellipse() {
        let pixels = shapes::rasterize_ellipse(15, 15, 8, 5);
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Ellipse(_)));
        assert!(confidence >= 0.7);
    }

    #[test]
    fn test_detect_shape_polygon_fallback() {
        // Create an irregular shape that doesn't match any primitive
        let pixels: HashSet<(i32, i32)> = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (1, 1),
            (2, 1),
            (2, 2),
            (3, 2),
            (4, 2),
        ]
        .into_iter()
        .collect();
        let (shape, _confidence) = detect_shape(&pixels);
        // Should fall back to polygon since it's not a recognized primitive
        assert!(matches!(shape, DetectedShape::Polygon(_)));
    }

    #[test]
    fn test_detect_shape_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Polygon(ref v) if v.is_empty()));
        assert!((confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_shape_detection_negative_coords() {
        let pixels = shapes::rasterize_rect(-5, -3, 4, 3);
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [-5, -3, 4, 3]);
    }

    #[test]
    fn test_confidence_scoring() {
        // Perfect rectangle should have confidence 1.0
        let pixels = shapes::rasterize_rect(0, 0, 5, 5);
        let detection = detect_rect(&pixels).unwrap();
        assert!((detection.confidence - 1.0).abs() < 0.001);

        // Line should have high confidence
        let pixels = shapes::rasterize_line((0, 0), (10, 10));
        let detection = detect_line(&pixels).unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_bounding_box() {
        let pixels: HashSet<(i32, i32)> = [(0, 0), (5, 3), (2, 7), (-1, 4)].into_iter().collect();
        let bbox = bounding_box(&pixels);
        assert_eq!(bbox, Some((-1, 0, 5, 7)));
    }

    #[test]
    fn test_bounding_box_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let bbox = bounding_box(&pixels);
        assert_eq!(bbox, None);
    }

    #[test]
    fn test_extract_polygon_vertices_triangle() {
        // Create a triangle-ish shape
        let triangle = shapes::rasterize_polygon(&[(0, 0), (6, 0), (3, 4)]);
        let vertices = extract_polygon_vertices(&triangle);
        // Convex hull should have 3 vertices
        assert!(vertices.len() >= 3);
    }

    #[test]
    fn test_extract_polygon_vertices_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let vertices = extract_polygon_vertices(&pixels);
        assert!(vertices.is_empty());

    // Symmetry detection tests (24.13)
    // ========================================================================

    /// Helper to create RGBA pixel data from a simple color index grid.
    /// Each cell is a u8 index that maps to a color.
    fn make_pixel_grid(grid: &[&[u8]], width: usize, height: usize) -> Vec<u8> {
        let colors: [(u8, u8, u8, u8); 4] = [
            (255, 0, 0, 255),   // 0: red
            (0, 255, 0, 255),   // 1: green
            (0, 0, 255, 255),   // 2: blue
            (255, 255, 0, 255), // 3: yellow
        ];

        let mut pixels = Vec::with_capacity(width * height * 4);
        for row in grid {
            for &idx in *row {
                let (r, g, b, a) = colors[idx as usize];
                pixels.extend_from_slice(&[r, g, b, a]);
            }
        }
        pixels
    }

    #[test]
    fn test_detect_symmetry_x_axis() {
        // X-axis symmetric (left-right mirror):
        // R G G R
        // B Y Y B
        let grid: &[&[u8]] = &[&[0, 1, 1, 0], &[2, 3, 3, 2]];
        let pixels = make_pixel_grid(grid, 4, 2);
        assert_eq!(detect_symmetry(&pixels, 4, 2), Some(Symmetric::X));
    }

    #[test]
    fn test_detect_symmetry_y_axis() {
        // Y-axis symmetric (top-bottom mirror):
        // R G B
        // R G B
        let grid: &[&[u8]] = &[&[0, 1, 2], &[0, 1, 2]];
        let pixels = make_pixel_grid(grid, 3, 2);
        assert_eq!(detect_symmetry(&pixels, 3, 2), Some(Symmetric::Y));
    }

    #[test]
    fn test_detect_symmetry_xy_axes() {
        // Both axes symmetric:
        // R G G R
        // B Y Y B
        // B Y Y B
        // R G G R
        let grid: &[&[u8]] = &[&[0, 1, 1, 0], &[2, 3, 3, 2], &[2, 3, 3, 2], &[0, 1, 1, 0]];
        let pixels = make_pixel_grid(grid, 4, 4);
        assert_eq!(detect_symmetry(&pixels, 4, 4), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_none() {
        // Not symmetric:
        // R G B
        // Y R G
        let grid: &[&[u8]] = &[&[0, 1, 2], &[3, 0, 1]];
        let pixels = make_pixel_grid(grid, 3, 2);
        assert_eq!(detect_symmetry(&pixels, 3, 2), None);
    }

    #[test]
    fn test_detect_symmetry_single_pixel() {
        // Single pixel is always symmetric on both axes
        let pixels: Vec<u8> = vec![255, 0, 0, 255];
        assert_eq!(detect_symmetry(&pixels, 1, 1), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_uniform_color() {
        // All same color - symmetric on both axes
        let grid: &[&[u8]] = &[&[0, 0, 0], &[0, 0, 0], &[0, 0, 0]];
        let pixels = make_pixel_grid(grid, 3, 3);
        assert_eq!(detect_symmetry(&pixels, 3, 3), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_empty() {
        let pixels: Vec<u8> = vec![];
        assert_eq!(detect_symmetry(&pixels, 0, 0), None);
    }

    #[test]
    fn test_detect_symmetry_invalid_buffer_size() {
        // Buffer too small for dimensions
        let pixels: Vec<u8> = vec![255, 0, 0, 255];
        assert_eq!(detect_symmetry(&pixels, 2, 2), None);
    }

    #[test]
    fn test_detect_symmetry_odd_width_x() {
        // Odd width, X-axis symmetric:
        // R G B G R
        let grid: &[&[u8]] = &[&[0, 1, 2, 1, 0]];
        let pixels = make_pixel_grid(grid, 5, 1);
        assert_eq!(detect_symmetry(&pixels, 5, 1), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_odd_height_y() {
        // Odd height, Y-axis symmetric:
        // R
        // G
        // R
        let grid: &[&[u8]] = &[&[0], &[1], &[0]];
        let pixels = make_pixel_grid(grid, 1, 3);
        assert_eq!(detect_symmetry(&pixels, 1, 3), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_x_only_not_y() {
        // X-symmetric but not Y-symmetric:
        // R G G R  (x-symmetric)
        // B Y Y B  (x-symmetric)
        // R B B R  (x-symmetric, but different from row 0)
        let grid: &[&[u8]] = &[&[0, 1, 1, 0], &[2, 3, 3, 2], &[0, 2, 2, 0]];
        let pixels = make_pixel_grid(grid, 4, 3);
        assert_eq!(detect_symmetry(&pixels, 4, 3), Some(Symmetric::X));
    }

    #[test]
    fn test_detect_symmetry_y_only_not_x() {
        // Y-symmetric but not X-symmetric:
        // R G B
        // Y R G
        // Y R G
        // R G B
        let grid: &[&[u8]] = &[&[0, 1, 2], &[3, 0, 1], &[3, 0, 1], &[0, 1, 2]];
        let pixels = make_pixel_grid(grid, 3, 4);
        assert_eq!(detect_symmetry(&pixels, 3, 4), Some(Symmetric::Y));
    }
}
