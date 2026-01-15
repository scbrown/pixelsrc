//! Corpus analysis for Pixelsrc files
//!
//! Provides tools to analyze pixelsrc files and extract metrics about:
//! - Token frequency and co-occurrence
//! - Dimensional distribution
//! - Structural patterns
//! - Compression opportunities

use std::collections::HashMap;
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

/// Results of structural pattern analysis for a sprite.
#[derive(Debug, Default, Clone)]
pub struct StructuralPatterns {
    /// Whether the sprite has outline tokens on its perimeter
    pub has_outline: bool,
    /// Whether the sprite is horizontally symmetric
    pub horizontal_symmetry: bool,
    /// Whether the sprite is vertically symmetric
    pub vertical_symmetry: bool,
    /// Whether the sprite contains gradient patterns
    pub has_gradients: bool,
    /// Transparency pattern: "border", "interior", "mixed", or "none"
    pub transparency_pattern: String,
}

/// Aggregated structural statistics for a corpus.
#[derive(Debug, Default)]
pub struct StructuralStats {
    /// Number of sprites with outline tokens
    pub with_outline: usize,
    /// Number of sprites with horizontal symmetry
    pub with_horizontal_symmetry: usize,
    /// Number of sprites with vertical symmetry
    pub with_vertical_symmetry: usize,
    /// Number of sprites with gradient patterns
    pub with_gradients: usize,
    /// Number of sprites with transparency on border
    pub transparency_border: usize,
    /// Number of sprites with transparency in interior only
    pub transparency_interior: usize,
    /// Number of sprites with mixed transparency
    pub transparency_mixed: usize,
    /// Number of sprites with no transparency
    pub transparency_none: usize,
}

impl StructuralStats {
    /// Create new structural stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record structural patterns from a sprite analysis.
    pub fn add(&mut self, patterns: &StructuralPatterns) {
        if patterns.has_outline {
            self.with_outline += 1;
        }
        if patterns.horizontal_symmetry {
            self.with_horizontal_symmetry += 1;
        }
        if patterns.vertical_symmetry {
            self.with_vertical_symmetry += 1;
        }
        if patterns.has_gradients {
            self.with_gradients += 1;
        }

        match patterns.transparency_pattern.as_str() {
            "border" => self.transparency_border += 1,
            "interior" => self.transparency_interior += 1,
            "mixed" => self.transparency_mixed += 1,
            _ => self.transparency_none += 1,
        }
    }
}

/// Analyzes structural patterns in sprite grids.
pub struct StructuralAnalyzer;

impl StructuralAnalyzer {
    /// Analyze a sprite's grid and return structural patterns.
    pub fn analyze(grid: &[String]) -> StructuralPatterns {
        let mut patterns = StructuralPatterns::default();

        if grid.is_empty() {
            patterns.transparency_pattern = "none".to_string();
            return patterns;
        }

        // Tokenize all rows
        let tokenized: Vec<Vec<String>> = grid.iter().map(|row| tokenize(row).0).collect();

        if tokenized.is_empty() || tokenized.iter().all(|row| row.is_empty()) {
            patterns.transparency_pattern = "none".to_string();
            return patterns;
        }

        patterns.has_outline = Self::detect_outline(&tokenized);
        patterns.horizontal_symmetry = Self::detect_horizontal_symmetry(&tokenized);
        patterns.vertical_symmetry = Self::detect_vertical_symmetry(&tokenized);
        patterns.has_gradients = Self::detect_gradients(&tokenized);
        patterns.transparency_pattern = Self::analyze_transparency(&tokenized);

        patterns
    }

    /// Detect if the sprite has outline tokens on its perimeter.
    ///
    /// An outline is detected if:
    /// - Tokens named "outline", "{outline}", or containing "outline" are on the perimeter
    /// - OR the same non-transparent token appears consistently on all edges
    fn detect_outline(grid: &[Vec<String>]) -> bool {
        if grid.is_empty() {
            return false;
        }

        let height = grid.len();
        let width = grid.iter().map(|r| r.len()).max().unwrap_or(0);

        if width == 0 {
            return false;
        }

        // Collect perimeter tokens
        let mut perimeter_tokens: Vec<&String> = Vec::new();

        // Top row
        if !grid.is_empty() {
            perimeter_tokens.extend(grid[0].iter());
        }

        // Bottom row (if different from top)
        if height > 1 {
            perimeter_tokens.extend(grid[height - 1].iter());
        }

        // Left and right columns (excluding corners already counted)
        for row in grid.iter().take(height.saturating_sub(1)).skip(1) {
            if !row.is_empty() {
                perimeter_tokens.push(&row[0]);
            }
            if row.len() > 1 {
                perimeter_tokens.push(&row[row.len() - 1]);
            }
        }

        // Check for explicit "outline" tokens
        let has_outline_token = perimeter_tokens.iter().any(|t| {
            let lower = t.to_lowercase();
            lower.contains("outline") || lower.contains("border") || lower.contains("edge")
        });

        if has_outline_token {
            return true;
        }

        // Check for consistent non-transparent token on perimeter
        // Filter out transparency tokens
        let non_transparent: Vec<&&String> = perimeter_tokens
            .iter()
            .filter(|t| **t != "{_}" && **t != "{ }" && **t != "{transparent}")
            .collect();

        if non_transparent.is_empty() {
            return false;
        }

        // Check if a single token dominates the perimeter (>60%)
        let mut token_counts: HashMap<&String, usize> = HashMap::new();
        for t in &non_transparent {
            *token_counts.entry(*t).or_insert(0) += 1;
        }

        if let Some((_, &max_count)) = token_counts.iter().max_by_key(|(_, c)| *c) {
            let ratio = max_count as f64 / non_transparent.len() as f64;
            return ratio > 0.6;
        }

        false
    }

    /// Detect horizontal symmetry (left-right mirror).
    fn detect_horizontal_symmetry(grid: &[Vec<String>]) -> bool {
        if grid.is_empty() {
            return false;
        }

        for row in grid {
            if row.is_empty() {
                continue;
            }
            let len = row.len();
            for i in 0..len / 2 {
                if row[i] != row[len - 1 - i] {
                    return false;
                }
            }
        }

        true
    }

    /// Detect vertical symmetry (top-bottom mirror).
    fn detect_vertical_symmetry(grid: &[Vec<String>]) -> bool {
        let height = grid.len();
        if height == 0 {
            return false;
        }

        for i in 0..height / 2 {
            let top = &grid[i];
            let bottom = &grid[height - 1 - i];
            if top != bottom {
                return false;
            }
        }

        true
    }

    /// Detect gradient patterns in the sprite.
    ///
    /// A gradient is detected when tokens appear in a sequential pattern,
    /// typically with numbered suffixes (e.g., skin1, skin2, skin3) or
    /// semantic progression (light, mid, dark).
    fn detect_gradients(grid: &[Vec<String>]) -> bool {
        // Common gradient suffix patterns
        let numbered_pattern = regex::Regex::new(r"\{(\w+)(\d+)\}").ok();
        let shade_words = [
            "light",
            "mid",
            "dark",
            "shadow",
            "highlight",
            "bright",
            "dim",
        ];

        // Check for numbered sequence patterns
        if let Some(ref re) = numbered_pattern {
            let mut bases_with_numbers: HashMap<String, Vec<u32>> = HashMap::new();

            for row in grid {
                for token in row {
                    if let Some(caps) = re.captures(token) {
                        let base = caps
                            .get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        let num: u32 = caps
                            .get(2)
                            .and_then(|m| m.as_str().parse().ok())
                            .unwrap_or(0);
                        bases_with_numbers.entry(base).or_default().push(num);
                    }
                }
            }

            // Check if any base has sequential numbers
            for (_, nums) in bases_with_numbers {
                if nums.len() >= 2 {
                    let mut sorted = nums.clone();
                    sorted.sort();
                    sorted.dedup();
                    // Check for at least 2 consecutive numbers
                    for window in sorted.windows(2) {
                        if window[1] == window[0] + 1 {
                            return true;
                        }
                    }
                }
            }
        }

        // Check for shade word patterns
        let all_tokens: Vec<&String> = grid.iter().flat_map(|row| row.iter()).collect();
        let lowercase_tokens: Vec<String> = all_tokens.iter().map(|t| t.to_lowercase()).collect();

        // Check if multiple shade words appear
        let shade_count = shade_words
            .iter()
            .filter(|&word| lowercase_tokens.iter().any(|t| t.contains(word)))
            .count();

        shade_count >= 2
    }

    /// Analyze how transparency is distributed in the sprite.
    ///
    /// Returns: "border" (only on edges), "interior" (only inside),
    /// "mixed" (both), or "none" (no transparency).
    fn analyze_transparency(grid: &[Vec<String>]) -> String {
        if grid.is_empty() {
            return "none".to_string();
        }

        let height = grid.len();
        let width = grid.iter().map(|r| r.len()).max().unwrap_or(0);

        if width == 0 || height == 0 {
            return "none".to_string();
        }

        let is_transparent = |token: &str| -> bool {
            token == "{_}" || token == "{ }" || token.to_lowercase().contains("transparent")
        };

        let mut border_transparent = false;
        let mut interior_transparent = false;

        for (row_idx, row) in grid.iter().enumerate() {
            for (col_idx, token) in row.iter().enumerate() {
                if is_transparent(token) {
                    let is_border = row_idx == 0
                        || row_idx == height - 1
                        || col_idx == 0
                        || col_idx == row.len() - 1;

                    if is_border {
                        border_transparent = true;
                    } else {
                        interior_transparent = true;
                    }
                }
            }
        }

        match (border_transparent, interior_transparent) {
            (true, true) => "mixed".to_string(),
            (true, false) => "border".to_string(),
            (false, true) => "interior".to_string(),
            (false, false) => "none".to_string(),
        }
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
    /// Dimension statistics
    pub dimension_stats: DimensionStats,
    /// Structural pattern statistics
    pub structural_stats: StructuralStats,
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

        // Count all tokens in the grid
        for row in &sprite.grid {
            let (tokens, _) = tokenize(row);
            for token in tokens {
                self.token_counter.add(&token);
            }
        }

        // Analyze structural patterns
        let patterns = StructuralAnalyzer::analyze(&sprite.grid);
        self.structural_stats.add(&patterns);
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

    // Structural patterns
    if report.total_sprites > 0 {
        let stats = &report.structural_stats;
        let total = report.total_sprites;
        output.push_str("STRUCTURAL PATTERNS\n");
        output.push_str("───────────────────\n");
        output.push_str(&format!(
            "  Has outline:        {:>4} sprites ({:.0}%)\n",
            stats.with_outline,
            (stats.with_outline as f64 / total as f64) * 100.0
        ));
        output.push_str(&format!(
            "  Horizontal sym:     {:>4} sprites ({:.0}%)\n",
            stats.with_horizontal_symmetry,
            (stats.with_horizontal_symmetry as f64 / total as f64) * 100.0
        ));
        output.push_str(&format!(
            "  Vertical sym:       {:>4} sprites ({:.0}%)\n",
            stats.with_vertical_symmetry,
            (stats.with_vertical_symmetry as f64 / total as f64) * 100.0
        ));
        output.push_str(&format!(
            "  Uses gradients:     {:>4} sprites ({:.0}%)\n",
            stats.with_gradients,
            (stats.with_gradients as f64 / total as f64) * 100.0
        ));
        output.push_str(&format!(
            "  Transparency border:{:>4} sprites ({:.0}%)\n",
            stats.transparency_border,
            (stats.transparency_border as f64 / total as f64) * 100.0
        ));
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

    // Structural Analysis Tests

    #[test]
    fn test_horizontal_symmetry_detected() {
        // A horizontally symmetric 3x3 sprite
        let grid = vec![
            "{a}{b}{a}".to_string(),
            "{c}{d}{c}".to_string(),
            "{e}{f}{e}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.horizontal_symmetry);
    }

    #[test]
    fn test_horizontal_symmetry_not_detected() {
        // An asymmetric sprite
        let grid = vec!["{a}{b}{c}".to_string(), "{d}{e}{f}".to_string()];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(!patterns.horizontal_symmetry);
    }

    #[test]
    fn test_vertical_symmetry_detected() {
        // A vertically symmetric sprite
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
            "{a}{b}{c}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.vertical_symmetry);
    }

    #[test]
    fn test_vertical_symmetry_not_detected() {
        // Not vertically symmetric
        let grid = vec![
            "{a}{b}{c}".to_string(),
            "{d}{e}{f}".to_string(),
            "{g}{h}{i}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(!patterns.vertical_symmetry);
    }

    #[test]
    fn test_outline_with_explicit_token() {
        // Sprite with {outline} tokens on perimeter
        let grid = vec![
            "{outline}{outline}{outline}".to_string(),
            "{outline}{skin}{outline}".to_string(),
            "{outline}{outline}{outline}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.has_outline);
    }

    #[test]
    fn test_outline_with_border_token() {
        // Sprite with {border} tokens (should also count as outline)
        let grid = vec![
            "{border}{border}{border}".to_string(),
            "{border}{fill}{border}".to_string(),
            "{border}{border}{border}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.has_outline);
    }

    #[test]
    fn test_outline_with_consistent_perimeter() {
        // Sprite where same token dominates the perimeter
        let grid = vec![
            "{black}{black}{black}{black}".to_string(),
            "{black}{red}{red}{black}".to_string(),
            "{black}{red}{red}{black}".to_string(),
            "{black}{black}{black}{black}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.has_outline);
    }

    #[test]
    fn test_gradient_with_numbered_tokens() {
        // Sprite with numbered gradient tokens
        let grid = vec![
            "{skin1}{skin2}{skin3}".to_string(),
            "{skin1}{skin2}{skin3}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.has_gradients);
    }

    #[test]
    fn test_gradient_with_shade_words() {
        // Sprite with light/dark gradient
        let grid = vec![
            "{highlight}{skin}{shadow}".to_string(),
            "{light}{mid}{dark}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(patterns.has_gradients);
    }

    #[test]
    fn test_no_gradient() {
        // Sprite without gradient patterns
        let grid = vec![
            "{red}{blue}{green}".to_string(),
            "{yellow}{purple}{orange}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert!(!patterns.has_gradients);
    }

    #[test]
    fn test_transparency_border_only() {
        // Transparency only on the border
        let grid = vec![
            "{_}{_}{_}".to_string(),
            "{_}{x}{_}".to_string(),
            "{_}{_}{_}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert_eq!(patterns.transparency_pattern, "border");
    }

    #[test]
    fn test_transparency_interior_only() {
        // Transparency only in interior (unusual but possible)
        let grid = vec![
            "{a}{a}{a}".to_string(),
            "{a}{_}{a}".to_string(),
            "{a}{a}{a}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert_eq!(patterns.transparency_pattern, "interior");
    }

    #[test]
    fn test_transparency_mixed() {
        // Transparency on both border and interior
        let grid = vec![
            "{_}{a}{_}".to_string(),
            "{a}{_}{a}".to_string(),
            "{_}{a}{_}".to_string(),
        ];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert_eq!(patterns.transparency_pattern, "mixed");
    }

    #[test]
    fn test_transparency_none() {
        // No transparency tokens
        let grid = vec!["{a}{b}{c}".to_string(), "{d}{e}{f}".to_string()];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert_eq!(patterns.transparency_pattern, "none");
    }

    #[test]
    fn test_structural_stats_aggregation() {
        let mut stats = StructuralStats::new();

        let patterns1 = StructuralPatterns {
            has_outline: true,
            horizontal_symmetry: true,
            vertical_symmetry: false,
            has_gradients: true,
            transparency_pattern: "border".to_string(),
        };

        let patterns2 = StructuralPatterns {
            has_outline: false,
            horizontal_symmetry: true,
            vertical_symmetry: true,
            has_gradients: false,
            transparency_pattern: "none".to_string(),
        };

        stats.add(&patterns1);
        stats.add(&patterns2);

        assert_eq!(stats.with_outline, 1);
        assert_eq!(stats.with_horizontal_symmetry, 2);
        assert_eq!(stats.with_vertical_symmetry, 1);
        assert_eq!(stats.with_gradients, 1);
        assert_eq!(stats.transparency_border, 1);
        assert_eq!(stats.transparency_none, 1);
    }

    #[test]
    fn test_empty_grid() {
        let grid: Vec<String> = vec![];
        let patterns = StructuralAnalyzer::analyze(&grid);
        assert_eq!(patterns.transparency_pattern, "none");
        assert!(!patterns.has_outline);
        assert!(!patterns.has_gradients);
    }
}
