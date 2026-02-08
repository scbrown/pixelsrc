//! Analysis report generation and file collection

use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::models::{Sprite, TtpObject};
use crate::parser::parse_stream;

use super::dimensions::DimensionStats;
use super::families::{TokenFamily, TokenFamilyDetector};
use super::tokens::{CoOccurrenceMatrix, TokenCounter};

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
    ///
    /// NOTE: This function requires update for v2 region-based format.
    /// Grid-based analysis is deprecated. See TTP-7i4v for grid removal.
    pub fn analyze_sprite(&mut self, sprite: &Sprite) {
        self.total_sprites += 1;

        // Use explicit size if provided
        if let Some([w, h]) = sprite.size {
            self.dimension_stats.add(w, h);
        }

        // TODO: Update for v2 region-based format
        // Grid-based token analysis is deprecated.
        // For now, we only count sprites and record explicit sizes.
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
                TtpObject::StateRules(_) => {
                    // State rules are runtime styling, not analyzed
                }
                TtpObject::Import(_) => {
                    // Import declarations are resolved during loading, not analyzed
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
