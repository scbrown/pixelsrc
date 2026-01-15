# Phase 15: Corpus Analysis (`pxl analyze`)

**Goal:** Add a command to analyze pixelsrc files and extract metrics that inform future primitive development and format optimization.

**Status:** Planning

**Depends on:** None (can be implemented independently)

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

## Metrics to Collect

### 1. Token Frequency

- Count occurrences of each token across all sprites
- Identify most common tokens (likely: `{_}`, `{outline}`, `{skin}`, `{shadow}`)
- Track tokens unique to single files vs. widely reused
- Calculate percentage distribution

### 2. Token Co-occurrence

- Which tokens appear together in the same palette?
- Build co-occurrence matrix: token pairs that frequently share sprites
- Identify "token families" (e.g., `{skin}`, `{skin_light}`, `{skin_shadow}`)

### 3. Palette Patterns

- Common color relationships (highlight/main/shadow triads)
- Frequency of built-in palette usage (`@gameboy`, `@pico8`, etc.)
- Average palette size (number of unique tokens per sprite)
- Inline vs. named palette ratio

### 4. Dimensional Analysis

- Distribution of sprite sizes (8x8, 16x16, 32x32, etc.)
- Aspect ratios (square vs. tall vs. wide)
- Most common dimensions
- Size correlation with token count

### 5. Structural Patterns

- **Outline detection**: Sprites with `{outline}` or similar tokens on perimeter
- **Symmetry detection**: Horizontal/vertical mirror patterns
- **Gradient detection**: Sequential color tokens in rows/columns
- **Transparency patterns**: How `{_}` is distributed (border vs. interior)

### 6. Grid Patterns (Compression Opportunities)

- **Row similarity**: Identical or near-identical rows within sprites
- **Run-length opportunities**: Consecutive identical tokens per row
- **Common row "shapes"**: Full, sparse, centered, bookend patterns
- **Potential compression ratio**: Estimate token savings from RLE

### 7. Composition Usage

If compositions present in corpus:
- Average number of sprites per composition
- Layer count distribution
- Sprite reuse frequency across compositions
- Canvas size distribution

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

## Implementation Notes

### New module: `src/analyze.rs`

Core structures:
- `AnalysisReport` - aggregates all metrics
- `TokenCounter` - frequency tracking with HashMap
- `CoOccurrenceMatrix` - token pair tracking
- `StructuralAnalyzer` - pattern detection (outline, symmetry, gradient)
- `CompressionEstimator` - RLE opportunity calculation

### Integration with existing code

- Use `parser.rs` to load and parse files
- Use `tokenizer.rs` to extract tokens from grids
- Use `registry.rs` to resolve palette references
- Use `models.rs` types for type-safe analysis

### CLI integration

Add to `cli.rs`:
```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    Analyze(AnalyzeArgs),
}

#[derive(Args)]
struct AnalyzeArgs {
    #[arg(required_unless_present = "dir")]
    files: Vec<PathBuf>,

    #[arg(long)]
    dir: Option<PathBuf>,

    #[arg(long, short)]
    recursive: bool,

    #[arg(long, default_value = "text")]
    format: OutputFormat,

    #[arg(long, short)]
    output: Option<PathBuf>,
}
```

---

## Tasks

### Task 15.1: Core Analysis Infrastructure

Create `src/analyze.rs` with:
- `AnalysisReport` struct
- `TokenCounter` for frequency tracking
- File collection (single files, directory, recursive)
- Basic text output formatting

### Task 15.2: Token Analysis

Implement:
- Token frequency counting across all sprites
- Token co-occurrence matrix
- Token family detection (similar names like `{skin}*`)

### Task 15.3: Structural Analysis

Implement:
- Outline detection (perimeter token analysis)
- Symmetry detection (row/column comparison)
- Gradient detection (sequential color patterns)
- Transparency distribution analysis

### Task 15.4: Compression Estimation

Implement:
- Run-length encoding opportunity calculation
- Row repetition detection
- Compression ratio estimation

### Task 15.5: JSON Output & CLI Polish

- Add `--format json` output
- Add `--output <file>` support
- Error handling for invalid files
- Progress indication for large corpora

---

## Future Extensions

Not in scope for Phase 15, but potential additions:

| Feature | Notes |
|---------|-------|
| `--compare <baseline.json>` | Diff against previous analysis |
| `--suggest-primitives` | Recommend common patterns as primitives |
| `--visualize` | Output charts (SVG or ASCII) |
| `--watch` | Re-analyze on file changes |
| Token clustering | Group semantically similar tokens |

---

## Success Criteria

1. `pxl analyze examples/` produces meaningful report
2. JSON output is machine-parseable for scripting
3. Compression estimates are accurate (validated against manual calculation)
4. Analysis completes in reasonable time (<5s for 100 files)
5. Report provides actionable insights for primitive design
