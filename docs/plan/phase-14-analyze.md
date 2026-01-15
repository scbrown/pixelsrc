# Phase 14: Corpus Analysis (`pxl analyze`)

**Goal:** Add a command to analyze pixelsrc files and extract metrics that inform future primitive development and format optimization

**Status:** Complete

**Depends on:** Phase 0 (Core CLI exists)

---

## Scope

Phase 14 adds:
- `pxl analyze` command for corpus analysis
- Token frequency and co-occurrence tracking
- Structural pattern detection (outline, symmetry, gradient)
- Dimensional analysis (sprite sizes, aspect ratios)
- Compression opportunity estimation (RLE potential)
- JSON and text output formats

**Not in scope:** Visualization, watch mode, primitive suggestion, comparison diffs

---

## Motivation

Before designing primitives (common patterns, shapes, reusable components), we need data about actual usage patterns. This phase adds tooling to analyze a corpus of pixelsrc files and surface insights about:

- Which tokens are most common
- What structural patterns emerge (outlines, symmetry, gradients)
- Common dimensions and aspect ratios
- Palette reuse patterns
- Compression opportunities (run-length encoding, row repetition)

This data-driven approach ensures primitives emerge from real usage rather than speculation.

---

## Command Interface

```bash
pxl analyze <files...>              # Analyze specific files
pxl analyze --dir <path>            # Analyze all .jsonl in directory
pxl analyze --recursive <path>      # Include subdirectories
pxl analyze --format json|text      # Output format (default: text)
pxl analyze --output <file>         # Write to file instead of stdout
```

---

## Task Dependency Diagram

```
                              PHASE 14 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                      Phase 0 Complete                           │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Foundation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   14.1 Core Analysis Infrastructure                      │   │
    │  │   - AnalysisReport struct                                │   │
    │  │   - File collection (single, dir, recursive)             │   │
    │  │   - Basic CLI integration                                │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - Analysis Modules)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
    │  │   14.2       │  │   14.3       │  │   14.4       │          │
    │  │  Token       │  │  Structural  │  │  Dimensional │          │
    │  │  Analysis    │  │  Analysis    │  │  Analysis    │          │
    │  └──────────────┘  └──────────────┘  └──────────────┘          │
    └─────────────────────────────────────────────────────────────────┘
              │                 │                 │
              └────────────────┬┴─────────────────┘
                               │
                               ▼
    WAVE 3 (After Wave 2)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   14.5 Compression Estimation                            │   │
    │  │   - Run-length encoding opportunity calculation          │   │
    │  │   - Row repetition detection                             │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 4 (Output & Polish)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   14.6 Output Formatting & CLI Polish                    │   │
    │  │   - JSON output mode                                     │   │
    │  │   - Text report formatting                               │   │
    │  │   - File output option                                   │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 14.1                        (1 task)                   │
    │  Wave 2: 14.2 + 14.3 + 14.4          (3 tasks in parallel)      │
    │  Wave 3: 14.5                        (1 task, needs token data) │
    │  Wave 4: 14.6                        (1 task, needs all above)  │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 14.1: Core Analysis Infrastructure

**Wave:** 1

Create the foundation for analysis functionality.

**Deliverables:**
- New file `src/analyze.rs`:
  ```rust
  use std::path::PathBuf;
  use std::collections::HashMap;

  #[derive(Debug, Default)]
  pub struct AnalysisReport {
      pub files_analyzed: usize,
      pub total_sprites: usize,
      pub total_palettes: usize,
      pub total_compositions: usize,
      pub token_frequency: HashMap<String, usize>,
      pub token_cooccurrence: HashMap<(String, String), usize>,
      pub dimensions: HashMap<(u32, u32), usize>,
      pub structural_patterns: StructuralPatterns,
      pub compression_stats: CompressionStats,
  }

  #[derive(Debug, Default)]
  pub struct StructuralPatterns {
      pub has_outline: usize,
      pub horizontal_symmetry: usize,
      pub vertical_symmetry: usize,
      pub uses_gradients: usize,
      pub transparency_border: usize,
  }

  #[derive(Debug, Default)]
  pub struct CompressionStats {
      pub avg_tokens_per_row: f64,
      pub avg_unique_per_row: f64,
      pub avg_runs_per_row: f64,
      pub estimated_compression_ratio: f64,
  }

  pub fn collect_files(paths: &[PathBuf], recursive: bool) -> Vec<PathBuf>
  pub fn analyze_files(files: &[PathBuf]) -> AnalysisReport
  ```

- Update `src/cli.rs`:
  ```rust
  #[derive(Subcommand)]
  enum Commands {
      // ... existing commands
      /// Analyze pixelsrc files and extract metrics
      Analyze(AnalyzeArgs),
  }

  #[derive(Args)]
  struct AnalyzeArgs {
      /// Files to analyze
      #[arg(required_unless_present = "dir")]
      files: Vec<PathBuf>,

      /// Directory to analyze
      #[arg(long)]
      dir: Option<PathBuf>,

      /// Include subdirectories
      #[arg(long, short)]
      recursive: bool,

      /// Output format
      #[arg(long, default_value = "text")]
      format: OutputFormat,

      /// Write output to file
      #[arg(long, short)]
      output: Option<PathBuf>,
  }

  #[derive(Clone, ValueEnum)]
  enum OutputFormat {
      Text,
      Json,
  }
  ```

- Update `src/lib.rs` to export `analyze` module

**Verification:**
```bash
cargo build
./target/release/pxl analyze --help
# Should show: pxl analyze [OPTIONS] [FILES]... with --dir, --recursive, --format, --output

./target/release/pxl analyze examples/
# Should run without error (may show empty report)
```

**Test Fixture:** `tests/fixtures/valid/analyze_corpus/` directory with 3+ sample sprites

**Dependencies:** Phase 0 complete

---

### Task 14.2: Token Analysis

**Wave:** 2 (parallel with 14.3, 14.4)

Implement token frequency counting and co-occurrence tracking.

**Deliverables:**
- Add to `src/analyze.rs`:
  ```rust
  pub struct TokenAnalyzer {
      frequency: HashMap<String, usize>,
      cooccurrence: HashMap<(String, String), usize>,
      sprites_with_token: HashMap<String, usize>,
  }

  impl TokenAnalyzer {
      pub fn new() -> Self
      pub fn analyze_sprite(&mut self, sprite: &Sprite)
      pub fn top_tokens(&self, n: usize) -> Vec<(&str, usize)>
      pub fn top_cooccurrences(&self, n: usize) -> Vec<((&str, &str), usize)>
      pub fn token_families(&self) -> Vec<Vec<&str>>  // e.g., {skin}, {skin_light}, {skin_shadow}
  }
  ```

- Track:
  - Token frequency across all sprites
  - Token co-occurrence (pairs appearing in same palette)
  - Token families (similar names detected via prefix matching)

**Verification:**
```bash
cargo test analyze::token
# Test: Empty sprite → empty results
# Test: Single sprite → correct token counts
# Test: Multiple sprites → aggregated counts
# Test: Co-occurrence matrix builds correctly

./target/release/pxl analyze examples/ | grep "TOKEN FREQUENCY"
# Should show top tokens with counts
```

**Test Fixture:** `tests/fixtures/valid/analyze_tokens.jsonl`
```jsonl
{"type": "palette", "name": "test", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{skin_shadow}": "#CC9966", "{hair}": "#8B4513"}}
{"type": "sprite", "name": "char1", "size": [4, 4], "palette": "test", "grid": ["{_}{skin}{skin}{_}", "{skin}{skin}{skin}{skin}", "{skin_shadow}{skin}{skin}{skin_shadow}", "{_}{skin_shadow}{skin_shadow}{_}"]}
{"type": "sprite", "name": "char2", "size": [4, 4], "palette": "test", "grid": ["{_}{hair}{hair}{_}", "{hair}{skin}{skin}{hair}", "{_}{skin}{skin}{_}", "{_}{_}{_}{_}"]}
```

**Dependencies:** Task 14.1

---

### Task 14.3: Structural Analysis

**Wave:** 2 (parallel with 14.2, 14.4)

Implement pattern detection for common sprite structures.

**Deliverables:**
- Add to `src/analyze.rs`:
  ```rust
  pub struct StructuralAnalyzer;

  impl StructuralAnalyzer {
      /// Detect if sprite has outline tokens on perimeter
      pub fn has_outline(sprite: &Sprite) -> bool

      /// Detect horizontal mirror symmetry (left = right)
      pub fn is_horizontally_symmetric(sprite: &Sprite) -> bool

      /// Detect vertical mirror symmetry (top = bottom)
      pub fn is_vertically_symmetric(sprite: &Sprite) -> bool

      /// Detect gradient patterns (sequential tokens in rows/columns)
      pub fn has_gradient(sprite: &Sprite) -> bool

      /// Detect transparency on border (outer ring is {_})
      pub fn has_transparency_border(sprite: &Sprite) -> bool

      /// Analyze sprite and return all detected patterns
      pub fn analyze(sprite: &Sprite) -> StructuralPatterns
  }
  ```

- Pattern detection rules:
  - **Outline**: First/last row and first/last column contain `{outline}` or similar token
  - **H-Symmetry**: Row[i] == reverse(Row[i]) for all rows
  - **V-Symmetry**: Row[i] == Row[height-1-i] for first half of rows
  - **Gradient**: 3+ consecutive different tokens in a row/column
  - **Transparency border**: Outer ring is `{_}` or transparent token

**Verification:**
```bash
cargo test analyze::structural
# Test: Symmetric sprite detected as symmetric
# Test: Asymmetric sprite detected as not symmetric
# Test: Outline sprite detected
# Test: Gradient pattern detected

./target/release/pxl analyze examples/ | grep "STRUCTURAL"
# Should show pattern counts
```

**Test Fixture:** `tests/fixtures/valid/analyze_patterns.jsonl`
```jsonl
{"type": "palette", "name": "patterns", "colors": {"{_}": "#00000000", "{o}": "#000000", "{a}": "#FF0000", "{b}": "#00FF00", "{c}": "#0000FF"}}
{"type": "sprite", "name": "symmetric", "size": [5, 3], "palette": "patterns", "grid": ["{a}{b}{c}{b}{a}", "{a}{b}{c}{b}{a}", "{a}{b}{c}{b}{a}"]}
{"type": "sprite", "name": "outlined", "size": [4, 4], "palette": "patterns", "grid": ["{o}{o}{o}{o}", "{o}{a}{a}{o}", "{o}{a}{a}{o}", "{o}{o}{o}{o}"]}
{"type": "sprite", "name": "gradient", "size": [4, 1], "palette": "patterns", "grid": ["{a}{b}{c}{_}"]}
```

**Dependencies:** Task 14.1

---

### Task 14.4: Dimensional Analysis

**Wave:** 2 (parallel with 14.2, 14.3)

Track sprite sizes and aspect ratios.

**Deliverables:**
- Add to `src/analyze.rs`:
  ```rust
  pub struct DimensionalAnalyzer {
      size_counts: HashMap<(u32, u32), usize>,
  }

  impl DimensionalAnalyzer {
      pub fn new() -> Self
      pub fn add_sprite(&mut self, sprite: &Sprite)
      pub fn top_sizes(&self, n: usize) -> Vec<((u32, u32), usize)>
      pub fn aspect_ratio_distribution(&self) -> HashMap<AspectRatio, usize>
      pub fn size_categories(&self) -> SizeCategories
  }

  #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  pub enum AspectRatio {
      Square,      // 1:1
      Portrait,    // taller than wide
      Landscape,   // wider than tall
  }

  #[derive(Debug, Default)]
  pub struct SizeCategories {
      pub tiny: usize,     // <= 8x8
      pub small: usize,    // 9-16
      pub medium: usize,   // 17-32
      pub large: usize,    // 33-64
      pub huge: usize,     // > 64
  }
  ```

**Verification:**
```bash
cargo test analyze::dimensional
# Test: Single sprite → correct size recorded
# Test: Multiple sizes → distribution correct
# Test: Aspect ratios categorized correctly

./target/release/pxl analyze examples/ | grep "DIMENSIONS"
# Should show size distribution
```

**Dependencies:** Task 14.1

---

### Task 14.5: Compression Estimation

**Wave:** 3 (after 14.2)

Calculate potential compression savings from run-length encoding.

**Deliverables:**
- Add to `src/analyze.rs`:
  ```rust
  pub struct CompressionEstimator;

  impl CompressionEstimator {
      /// Count consecutive identical tokens in a row
      pub fn count_runs(row: &[String]) -> usize

      /// Calculate RLE savings for a sprite
      pub fn estimate_rle_savings(sprite: &Sprite) -> RleSavings

      /// Detect identical or near-identical rows
      pub fn find_repeated_rows(sprite: &Sprite) -> Vec<(usize, usize)>

      /// Aggregate compression stats across corpus
      pub fn analyze_corpus(sprites: &[Sprite]) -> CompressionStats
  }

  #[derive(Debug)]
  pub struct RleSavings {
      pub original_tokens: usize,
      pub compressed_tokens: usize,
      pub ratio: f64,
  }
  ```

- Metrics to calculate:
  - Average tokens per row
  - Average unique tokens per row
  - Average runs per row (consecutive identical tokens)
  - Potential compression ratio (original / RLE encoded)

**Verification:**
```bash
cargo test analyze::compression
# Test: Row of identical tokens → high compression
# Test: Row of unique tokens → no compression
# Test: Mixed row → accurate run count

./target/release/pxl analyze examples/ | grep "COMPRESSION"
# Should show compression statistics
```

**Test Fixture:** `tests/fixtures/valid/analyze_compression.jsonl`
```jsonl
{"type": "palette", "name": "compress", "colors": {"{_}": "#00000000", "{a}": "#FF0000"}}
{"type": "sprite", "name": "compressible", "size": [16, 4], "palette": "compress", "grid": [
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{a}{a}{a}{a}{a}{a}{a}{a}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{a}{a}{a}{a}{a}{a}{a}{a}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}"
]}
```

**Dependencies:** Task 14.2

---

### Task 14.6: Output Formatting & CLI Polish

**Wave:** 4 (after all above)

Complete output formatting and CLI integration.

**Deliverables:**
- Text output format (default):
  ```
  Pixelsrc Analysis Report
  ========================
  Files analyzed: 47
  Total sprites: 156
  Total palettes: 89
  Total compositions: 12

  TOKEN FREQUENCY (top 10)
  ────────────────────────
    {_}        45,231  (28.4%)
    {outline}  12,847  (8.1%)
    {skin}      8,392  (5.3%)
    ...

  TOKEN CO-OCCURRENCE (top 5 pairs)
  ─────────────────────────────────
    {skin} + {skin_shadow}     67 sprites
    ...

  DIMENSIONS
  ──────────
    16x16      67 sprites (43%)
    8x8        42 sprites (27%)
    ...

  STRUCTURAL PATTERNS
  ───────────────────
    Has outline:        89 sprites (57%)
    Horizontal sym:     34 sprites (22%)
    ...

  RUN-LENGTH OPPORTUNITIES
  ────────────────────────
    Avg tokens/row:           14.3
    Potential compression:    ~2.5x
  ```

- JSON output format (`--format json`):
  ```rust
  impl AnalysisReport {
      pub fn to_json(&self) -> serde_json::Value
  }
  ```
  ```json
  {
    "files_analyzed": 47,
    "total_sprites": 156,
    "token_frequency": [{"token": "{_}", "count": 45231, "percentage": 28.4}, ...],
    "dimensions": [{"size": [16, 16], "count": 67, "percentage": 43}, ...],
    ...
  }
  ```

- File output (`--output <file>`):
  ```rust
  fn write_report(report: &AnalysisReport, path: &Path, format: OutputFormat) -> Result<()>
  ```

- Progress indication for large corpora (stderr):
  ```
  Analyzing 100 files... [=====>    ] 50%
  ```

- Error handling:
  - Skip invalid files with warning
  - Continue on parse errors
  - Report count of skipped files

**Verification:**
```bash
# Text output
./target/release/pxl analyze examples/
# Should show formatted report

# JSON output
./target/release/pxl analyze examples/ --format json | jq '.total_sprites'
# Should return number

# File output
./target/release/pxl analyze examples/ --output /tmp/report.txt
cat /tmp/report.txt

# JSON file output
./target/release/pxl analyze examples/ --format json --output /tmp/report.json
jq '.' /tmp/report.json
```

**Dependencies:** Tasks 14.2, 14.3, 14.4, 14.5

---

## Output Example (text format)

```
Pixelsrc Analysis Report
========================
Files analyzed: 47
Total sprites: 156
Total palettes: 89
Total compositions: 12

TOKEN FREQUENCY (top 10)
────────────────────────
  {_}        45,231  (28.4%)
  {outline}  12,847  (8.1%)
  {skin}      8,392  (5.3%)
  {shadow}    7,104  (4.5%)
  {hair}      5,231  (3.3%)
  {white}     4,892  (3.1%)
  {black}     4,103  (2.6%)
  {eye}       3,847  (2.4%)
  {gold}      3,102  (2.0%)
  {shine}     2,891  (1.8%)

TOKEN CO-OCCURRENCE (top 5 pairs)
─────────────────────────────────
  {skin} + {skin_shadow}     67 sprites
  {outline} + {_}            64 sprites
  {gold} + {shine}           23 sprites
  {hair} + {hair_dark}       21 sprites
  {eye} + {pupil}            19 sprites

DIMENSIONS
──────────
  16x16      67 sprites (43%)
  8x8        42 sprites (27%)
  32x32      28 sprites (18%)
  24x24      11 sprites (7%)
  Other       8 sprites (5%)

STRUCTURAL PATTERNS
───────────────────
  Has outline:        89 sprites (57%)
  Horizontal sym:     34 sprites (22%)
  Vertical sym:       12 sprites (8%)
  Uses gradients:     45 sprites (29%)
  Transparency border: 78 sprites (50%)

RUN-LENGTH OPPORTUNITIES
────────────────────────
  Avg tokens/row:           14.3
  Avg unique tokens/row:     4.2
  Avg runs/row:              5.8
  Potential compression:    ~2.5x

PALETTE PATTERNS
────────────────
  Avg tokens/palette:        8.3
  Uses built-in palette:    23 sprites (15%)
  Has highlight/shadow:     67 sprites (43%)
  Inline palette:           34 sprites (22%)
```

---

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. CLI help works
./target/release/pxl analyze --help

# 3. Analyze examples directory
./target/release/pxl analyze examples/
# Should produce meaningful report

# 4. JSON output is valid
./target/release/pxl analyze examples/ --format json | jq '.'

# 5. Recursive works
./target/release/pxl analyze --recursive tests/fixtures/
# Should find all .jsonl files

# 6. File output works
./target/release/pxl analyze examples/ --output /tmp/report.txt
cat /tmp/report.txt

# 7. Performance test
time ./target/release/pxl analyze --recursive examples/
# Should complete in < 5s for typical corpus
```

---

## Success Criteria

1. `pxl analyze examples/` produces meaningful report
2. JSON output is machine-parseable for scripting
3. Compression estimates are accurate (validated against manual calculation)
4. Analysis completes in reasonable time (<5s for 100 files)
5. Report provides actionable insights for primitive design

---

## Future Extensions

Not in scope for Phase 14, but potential additions:

| Feature | Notes |
|---------|-------|
| `--compare <baseline.json>` | Diff against previous analysis |
| `--suggest-primitives` | Recommend common patterns as primitives |
| `--visualize` | Output charts (SVG or ASCII) |
| `--watch` | Re-analyze on file changes |
| Token clustering | Group semantically similar tokens |
