---
phase: 17
title: Colored Terminal Output & Alias Management
---

# Phase 17: Colored Terminal Output & Alias Management

**Related:** [Transforms](./transforms.md) - Use `pxl show` to visualize transform results

**Status:** In Progress (pxl show implemented, grid/inline/alias/sketch pending)

**Depends on:** Phase 0 (Core CLI exists)

---

## Summary

Add five new commands for viewing and transforming sprite grids:
1. `pxl show` - Colored terminal display with true-color backgrounds
2. `pxl grid` - Display grid with row/column coordinates for reference
3. `pxl inline` - Expand aliases with column-aligned spacing
4. `pxl alias` - Extract repeated patterns into aliases
5. `pxl sketch` - Create sprite from simple space-separated text grid

## New Commands

### `pxl show <file> [--sprite <name>]`
Display sprite grid with ANSI true-color backgrounds:
```
 a  a  b  b  b  b  a  a
 a  b  c  c  c  c  b  a
 a  b  c  d  d  c  b  a

Legend:
  a = transparent  (#00000000)
  b = body_blue    (#4A90D9)
  c = body_light   (#6BB5FF)
  d = eye_white    (#FFFFFF)
```
- Uses 24-bit ANSI colors (`\x1b[48;2;R;G;Bm` for background)
- Shows alias letter centered in 3-char cell
- Legend at bottom mapping aliases to semantic names and hex colors

### `pxl grid <file> [--sprite <name>]`
Display grid with row/column coordinates for easy reference:
```
     0  1  2  3  4  5  6  7
   ┌────────────────────────
 0 │ _  _  b  b  b  b  _  _
 1 │ _  b  c  c  c  c  b  _
 2 │ _  b  c  d  d  c  b  _
 3 │ _  b  c  d  e  c  b  _
```
- Column numbers across top
- Row numbers down left side
- Simplified token names (first char or alias)
- Optional `--full` flag to show full token names

### `pxl inline <file> [--sprite <name>]`
Expand aliases with padding BETWEEN cells for column alignment:
```
{_}  {_}               {body_blue}  {body_blue}
{_}  {skin_highlight}  {body_light} {body_light}
{_}  {skin}            {hair}       {hair_dark}
```
- Tokens unchanged - padding added AFTER each cell
- Each column starts at consistent position
- Clean separation makes pattern visible
- Output to stdout

### `pxl alias <file> [--sprite <name>]`
Extract common tokens into single-letter aliases:
```json
{
  "aliases": {"a": "transparent", "b": "body_blue", "c": "body_light"},
  "grid": [
    "{a}{a}{b}{b}",
    "{a}{b}{c}{c}"
  ]
}
```
- Outputs JSON with `aliases` map and transformed grid
- Frequency-based assignment (most common = `a`, etc.)
- `{_}` convention preserved (always maps to `_`)

### `pxl sketch [--name <name>] [--palette <palette>]`
Create sprite from simple space-separated text (reads stdin):
```
Input (easy to generate/align):
_ _ b b b b _ _
_ b c c c c b _
_ b c d d c b _

Output:
{
  "type": "sprite",
  "name": "sprite_1",
  "size": [8, 3],
  "palette": {
    "{_}": "#00000000",
    "{b}": "#000000",
    "{c}": "#000000",
    "{d}": "#000000"
  },
  "grid": [
    "{_}{_}{b}{b}{b}{b}{_}{_}",
    "{_}{b}{c}{c}{c}{c}{b}{_}",
    "{_}{b}{c}{d}{d}{c}{b}{_}"
  ]
}
```
- Reads newline-separated rows from stdin
- Each row is space-separated single letters/tokens
- Auto-generates placeholder palette (user fills in colors)
- `_` automatically mapped to transparent
- Optional `--palette` to reference existing palette

## Files to Modify

### `src/cli.rs`
- Add `Show`, `Grid`, `Inline`, `Alias`, `Sketch` variants to `Commands` enum
- Add match arms in `run()` function
- Add handler functions for each command

### `src/terminal.rs` (new)
Core terminal rendering logic:
- `render_ansi_grid(sprite, palette, aliases) -> String`
- `render_coordinate_grid(sprite, full_names: bool) -> String`
- `color_to_ansi_bg(rgba) -> String`
- `ANSI_RESET: &str = "\x1b[0m"`

### `src/alias.rs` (new)
Alias extraction and expansion:
- `extract_aliases(grid) -> (HashMap<char, String>, Vec<String>)`
- `expand_aliases(grid, aliases) -> Vec<String>`
- `format_columns(grid) -> Vec<String>` - Column-aligned spacing
- `parse_simple_grid(input: &str) -> Vec<Vec<String>>` - Parse space-separated grid
- `simple_grid_to_sprite(grid, name, palette_ref) -> Sprite` - Convert to sprite

### `src/lib.rs`
- Add `pub mod terminal;` and `pub mod alias;`

## Implementation Details

### ANSI True Color Format
```rust
fn color_to_ansi_bg(rgba: Rgba<u8>) -> String {
    if rgba[3] == 0 {
        "\x1b[48;5;236m".to_string()  // Dark gray for transparent
    } else {
        format!("\x1b[48;2;{};{};{}m", rgba[0], rgba[1], rgba[2])
    }
}
```

### Column-Aligned Spacing (padding BETWEEN cells)
```rust
fn format_columns(rows: Vec<Vec<String>>) -> Vec<String> {
    // Find max width per column
    let mut col_widths: Vec<usize> = vec![];
    for row in &rows {
        for (i, token) in row.iter().enumerate() {
            if i >= col_widths.len() {
                col_widths.push(token.len());
            } else {
                col_widths[i] = col_widths[i].max(token.len());
            }
        }
    }

    // Join tokens with spacing to align columns
    rows.iter().map(|row| {
        row.iter().enumerate()
            .map(|(i, token)| {
                let padding = col_widths[i] - token.len();
                format!("{}{}", token, " ".repeat(padding + 2)) // +2 for gap
            })
            .collect::<String>()
            .trim_end()
            .to_string()
    }).collect()
}
```

Example transformation:
```
Input tokens per row:
  Row 0: ["{_}", "{_}", "{body_blue}", "{body_blue}"]
  Row 1: ["{_}", "{skin_highlight}", "{body_light}", "{body_light}"]

Column widths: [3, 17, 12, 12]

Output:
  "{_}  {_}                 {body_blue}   {body_blue}"
  "{_}  {skin_highlight}    {body_light}  {body_light}"
```

### Coordinate Grid Format
```
     0  1  2  3  4  5      <- column headers (2-char width each)
   ┌──────────────────
 0 │ _  _  b  b  b  b      <- row 0
 1 │ _  b  c  c  c  c      <- row 1
```

## Verification

1. `pxl show examples/walk_cycle.pxl` - colored grid with legend
2. `pxl grid examples/heart.pxl` - coordinate reference display
3. `pxl inline examples/hero.pxl` - column-aligned output
4. `pxl alias examples/hero.pxl` - extract to single-letter aliases
5. `echo "_ _ b b\n_ b c b" | pxl sketch --name test` - simple grid to sprite
6. Round-trip: `pxl alias | pxl inline` preserves meaning
7. Workflow: `pxl sketch` -> edit colors -> `pxl show` to verify

## Task Dependency Diagram

```
                          COLORED GRID DISPLAY TASK FLOW
═══════════════════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Phase 0 Complete                                  │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 1 (Foundation - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │         CGD-1                  │  │         CGD-2                  │    │
│  │    Terminal Module             │  │     Alias Module               │    │
│  │    (src/terminal.rs)           │  │    (src/alias.rs)              │    │
│  │    - ANSI color utilities      │  │    - Alias extraction          │    │
│  │    - Coordinate rendering      │  │    - Column formatting         │    │
│  │                                │  │    - Simple grid parsing       │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                    │
            ▼                                    ▼
WAVE 2 (Commands - Parallel within groups)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────┐                               │
│  │  After CGD-1:                            │                               │
│  │  ┌─────────────┐  ┌─────────────┐        │                               │
│  │  │   CGD-3     │  │   CGD-4     │        │                               │
│  │  │  Show Cmd   │  │  Grid Cmd   │        │                               │
│  │  └─────────────┘  └─────────────┘        │                               │
│  └──────────────────────────────────────────┘                               │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  After CGD-2:                                                        │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                   │   │
│  │  │   CGD-5     │  │   CGD-6     │  │   CGD-7     │                   │   │
│  │  │ Inline Cmd  │  │  Alias Cmd  │  │ Sketch Cmd  │                   │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 3 (Testing)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CGD-8                                    │    │
│  │                    Test Suite for All Commands                      │    │
│  │                    (unit + integration tests)                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 4 (Documentation - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐     │
│  │  CGD-9    │ │  CGD-10   │ │  CGD-11   │ │  CGD-12   │ │  CGD-13   │     │
│  │  prime    │ │  format   │ │  prompts  │ │  demo.sh  │ │  lib.rs   │     │
│  │  output   │ │  spec     │ │  guides   │ │  examples │ │  exports  │     │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Wave 1: CGD-1 + CGD-2                    (2 tasks in parallel)            │
│  Wave 2: CGD-3 + CGD-4 (after CGD-1)      (2 tasks in parallel)            │
│          CGD-5 + CGD-6 + CGD-7 (after CGD-2) (3 tasks in parallel)         │
│  Wave 3: CGD-8                            (1 task, needs all commands)     │
│  Wave 4: CGD-9 through CGD-13             (5 tasks in parallel)            │
└─────────────────────────────────────────────────────────────────────────────┘

CRITICAL PATH: CGD-1 → CGD-3 → CGD-8 → CGD-9 (or any Wave 4 task)
              OR: CGD-2 → CGD-5/6/7 → CGD-8 → CGD-9

BEADS CREATION ORDER:
  1. CGD-1, CGD-2 (no deps)
  2. CGD-3, CGD-4 (dep: CGD-1), CGD-5, CGD-6, CGD-7 (dep: CGD-2)
  3. CGD-8 (dep: CGD-3, CGD-4, CGD-5, CGD-6, CGD-7)
  4. CGD-9, CGD-10, CGD-11, CGD-12, CGD-13 (dep: CGD-8)
```

---

## Tasks

### Task CGD-1: Terminal Module

**Wave:** 1 (parallel with CGD-2)

Create core terminal rendering utilities for ANSI color output.

**Deliverables:**
- New file `src/terminal.rs`:
  ```rust
  //! Terminal rendering utilities for colored sprite display

  use image::Rgba;

  pub const ANSI_RESET: &str = "\x1b[0m";

  /// Convert RGBA color to ANSI 24-bit background escape sequence
  pub fn color_to_ansi_bg(rgba: Rgba<u8>) -> String {
      if rgba[3] == 0 {
          "\x1b[48;5;236m".to_string()  // Dark gray for transparent
      } else {
          format!("\x1b[48;2;{};{};{}m", rgba[0], rgba[1], rgba[2])
      }
  }

  /// Render sprite grid with ANSI color backgrounds
  /// Returns colored grid string and legend
  pub fn render_ansi_grid(
      grid: &[String],
      palette: &HashMap<String, Rgba<u8>>,
      aliases: &HashMap<char, String>,
  ) -> (String, String)

  /// Render grid with row/column coordinate headers
  pub fn render_coordinate_grid(
      grid: &[String],
      full_names: bool,
  ) -> String
  ```

- Update `src/lib.rs` to add `pub mod terminal;`

**Verification:**
```bash
cargo build
cargo test terminal
# Test: color_to_ansi_bg produces valid escape sequences
# Test: transparent color uses fallback gray
# Test: render_ansi_grid produces colored output
```

**Dependencies:** Phase 0 complete

---

### Task CGD-2: Alias Module

**Wave:** 1 (parallel with CGD-1)

Create alias extraction, expansion, and simple grid parsing utilities.

**Deliverables:**
- New file `src/alias.rs`:
  ```rust
  //! Alias extraction, expansion, and simple grid utilities

  use std::collections::HashMap;

  /// Extract common tokens into single-letter aliases
  /// Returns (alias_map, transformed_grid)
  pub fn extract_aliases(grid: &[String]) -> (HashMap<char, String>, Vec<String>) {
      // Frequency-based assignment: most common = 'a', etc.
      // {_} always maps to '_'
  }

  /// Expand aliases back to full token names
  pub fn expand_aliases(
      grid: &[String],
      aliases: &HashMap<char, String>,
  ) -> Vec<String>

  /// Format grid with column-aligned spacing between cells
  pub fn format_columns(rows: Vec<Vec<String>>) -> Vec<String> {
      // Find max width per column
      // Add padding after each token for alignment
  }

  /// Parse space-separated simple grid input
  pub fn parse_simple_grid(input: &str) -> Vec<Vec<String>>

  /// Convert simple grid to sprite definition
  pub fn simple_grid_to_sprite(
      grid: Vec<Vec<String>>,
      name: &str,
      palette_ref: Option<&str>,
  ) -> serde_json::Value
  ```

- Update `src/lib.rs` to add `pub mod alias;`

**Verification:**
```bash
cargo build
cargo test alias
# Test: extract_aliases assigns letters by frequency
# Test: {_} always maps to '_'
# Test: format_columns aligns columns correctly
# Test: parse_simple_grid handles multi-row input
# Test: simple_grid_to_sprite produces valid JSON
```

**Test Fixture:** `tests/fixtures/valid/alias_test.jsonl`
```jsonl
{"type": "palette", "name": "test", "colors": {"{_}": "#00000000", "{a}": "#FF0000", "{b}": "#00FF00"}}
{"type": "sprite", "name": "alias_test", "size": [4, 2], "palette": "test", "grid": ["{_}{a}{a}{_}", "{_}{b}{b}{_}"]}
```

**Dependencies:** Phase 0 complete

---

### Task CGD-3: Show Command

**Wave:** 2 (after CGD-1)

Add `pxl show` command for colored terminal display.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  #[derive(Subcommand)]
  enum Commands {
      // ... existing commands
      /// Display sprite with colored terminal output
      Show(ShowArgs),
  }

  #[derive(Args)]
  struct ShowArgs {
      /// Input file
      file: PathBuf,
      /// Sprite name (if file contains multiple)
      #[arg(long)]
      sprite: Option<String>,
  }
  ```

- Implement handler using `terminal::render_ansi_grid`
- Output format: colored grid + legend

**Verification:**
```bash
./target/release/pxl show examples/walk_cycle.pxl
# Should display colored grid with legend
./target/release/pxl show examples/hero.jsonl --sprite hero_idle
# Should display specific sprite
```

**Dependencies:** Task CGD-1

---

### Task CGD-4: Grid Command

**Wave:** 2 (after CGD-1, parallel with CGD-3)

Add `pxl grid` command for coordinate reference display.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Display grid with row/column coordinates
  Grid(GridArgs),

  #[derive(Args)]
  struct GridArgs {
      /// Input file
      file: PathBuf,
      /// Sprite name (if file contains multiple)
      #[arg(long)]
      sprite: Option<String>,
      /// Show full token names instead of abbreviations
      #[arg(long)]
      full: bool,
  }
  ```

- Implement handler using `terminal::render_coordinate_grid`

**Verification:**
```bash
./target/release/pxl grid examples/heart.pxl
# Should show grid with row/column numbers
./target/release/pxl grid examples/heart.pxl --full
# Should show full token names
```

**Dependencies:** Task CGD-1

---

### Task CGD-5: Inline Command

**Wave:** 2 (after CGD-2, parallel with CGD-6, CGD-7)

Add `pxl inline` command for column-aligned output.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Expand grid with column-aligned spacing
  Inline(InlineArgs),

  #[derive(Args)]
  struct InlineArgs {
      /// Input file
      file: PathBuf,
      /// Sprite name (if file contains multiple)
      #[arg(long)]
      sprite: Option<String>,
  }
  ```

- Implement handler using `alias::format_columns`

**Verification:**
```bash
./target/release/pxl inline examples/hero.pxl
# Should output column-aligned tokens
# Columns should align vertically across rows
```

**Dependencies:** Task CGD-2

---

### Task CGD-6: Alias Command

**Wave:** 2 (after CGD-2, parallel with CGD-5, CGD-7)

Add `pxl alias` command to extract repeated patterns.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Extract repeated patterns into single-letter aliases
  Alias(AliasArgs),

  #[derive(Args)]
  struct AliasArgs {
      /// Input file
      file: PathBuf,
      /// Sprite name (if file contains multiple)
      #[arg(long)]
      sprite: Option<String>,
  }
  ```

- Implement handler using `alias::extract_aliases`
- Output JSON with `aliases` map and transformed `grid`

**Verification:**
```bash
./target/release/pxl alias examples/hero.pxl
# Should output JSON with aliases and transformed grid
./target/release/pxl alias examples/hero.pxl | jq '.aliases'
# Should show alias mapping
```

**Dependencies:** Task CGD-2

---

### Task CGD-7: Sketch Command

**Wave:** 2 (after CGD-2, parallel with CGD-5, CGD-6)

Add `pxl sketch` command to create sprites from simple text grids.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Create sprite from space-separated text grid (stdin)
  Sketch(SketchArgs),

  #[derive(Args)]
  struct SketchArgs {
      /// Sprite name
      #[arg(long, default_value = "sprite_1")]
      name: String,
      /// Reference existing palette by name
      #[arg(long)]
      palette: Option<String>,
  }
  ```

- Implement handler:
  - Read from stdin
  - Parse using `alias::parse_simple_grid`
  - Convert using `alias::simple_grid_to_sprite`
  - Output JSON sprite definition

**Verification:**
```bash
echo "_ _ b b\n_ b c b" | ./target/release/pxl sketch --name test
# Should output valid sprite JSON

echo "a b c\nb c a" | ./target/release/pxl sketch --palette @synthwave
# Should reference existing palette
```

**Dependencies:** Task CGD-2

---

### Task CGD-8: Test Suite

**Wave:** 3 (after CGD-3, CGD-4, CGD-5, CGD-6, CGD-7)

Add comprehensive tests for all new functionality.

**Deliverables:**
- `tests/terminal_tests.rs`:
  - ANSI escape sequence generation
  - Transparent color handling
  - Grid rendering output

- `tests/alias_tests.rs`:
  - Alias extraction frequency ordering
  - `{_}` special handling
  - Column alignment calculation
  - Simple grid parsing

- `tests/cli_integration.rs` additions:
  - `pxl show` produces output
  - `pxl grid` shows coordinates
  - `pxl inline` aligns columns
  - `pxl alias` produces valid JSON
  - `pxl sketch` creates sprite from stdin
  - Round-trip: `pxl alias | pxl inline` preserves meaning

**Test Fixtures:**
- `tests/fixtures/valid/show_test.jsonl`
- `tests/fixtures/valid/grid_test.jsonl`
- `tests/fixtures/valid/sketch_input.txt`

**Verification:**
```bash
cargo test terminal
cargo test alias
cargo test show
cargo test grid
cargo test inline
cargo test sketch
cargo test --test cli_integration
```

**Dependencies:** Tasks CGD-3, CGD-4, CGD-5, CGD-6, CGD-7

---

### Task CGD-9: Update Prime Output

**Wave:** 4 (after CGD-8, parallel with CGD-10 through CGD-13)

Update `pxl prime` to document new commands and workflow.

**Deliverables:**
- Update `src/prime.rs`:
  - Add `pxl show`, `pxl grid`, `pxl inline`, `pxl alias`, `pxl sketch` to command list
  - Add workflow example: sketch → edit colors → show to verify

**Verification:**
```bash
./target/release/pxl prime | grep -A2 "show"
./target/release/pxl prime | grep -A2 "sketch"
```

**Dependencies:** Task CGD-8

---

### Task CGD-10: Update Format Spec

**Wave:** 4 (parallel with CGD-9, CGD-11, CGD-12, CGD-13)

Update format specification if needed.

**Deliverables:**
- Review `docs/spec/format.md` for any format additions
- Document alias JSON output format if not already specified
- Add examples of sketch input format

**Verification:**
```bash
grep "alias" docs/spec/format.md
# Should document alias output format
```

**Dependencies:** Task CGD-8

---

### Task CGD-11: Update Prompt Guides

**Wave:** 4 (parallel with CGD-9, CGD-10, CGD-12, CGD-13)

Update AI prompt guides with new generation workflow.

**Deliverables:**
- Update `docs/prompts/system-prompt.md`:
  - Add sketch workflow for rapid prototyping
  - Show how to use `pxl show` for verification

- Update `docs/prompts/sprite-examples.md`:
  - Add examples using sketch input format
  - Show alias extraction workflow

**Verification:**
```bash
grep "sketch" docs/prompts/system-prompt.md
grep "show" docs/prompts/sprite-examples.md
```

**Dependencies:** Task CGD-8

---

### Task CGD-12: Update Demo Script

**Wave:** 4 (parallel with CGD-9, CGD-10, CGD-11, CGD-13)

Add examples of all new commands to demo.sh.

**Deliverables:**
- Update `demo.sh`:
  ```bash
  # Colored display
  pxl show examples/heart.pxl

  # Coordinate grid
  pxl grid examples/heart.pxl

  # Column-aligned output
  pxl inline examples/hero.pxl

  # Extract aliases
  pxl alias examples/hero.pxl

  # Create from sketch
  echo "_ _ b b\n_ b c c" | pxl sketch --name quick_test
  ```

**Verification:**
```bash
./demo.sh  # Should run without errors
grep "pxl show" demo.sh
grep "pxl sketch" demo.sh
```

**Dependencies:** Task CGD-8

---

### Task CGD-13: Update Lib Exports

**Wave:** 4 (parallel with CGD-9, CGD-10, CGD-11, CGD-12)

Ensure public APIs are properly exported.

**Deliverables:**
- Update `src/lib.rs`:
  ```rust
  pub mod terminal;
  pub mod alias;

  // Re-export commonly used items
  pub use terminal::{render_ansi_grid, render_coordinate_grid, ANSI_RESET};
  pub use alias::{extract_aliases, format_columns, parse_simple_grid};
  ```

**Verification:**
```bash
cargo doc --open
# Verify terminal and alias modules appear in docs
# Verify key functions are documented
```

**Dependencies:** Task CGD-8

---

## Crate Dependencies

No new crate dependencies needed - ANSI codes are simple string formatting.

---

## Verification Summary

```bash
# 1. All existing tests pass
cargo test

# 2. New module tests pass
cargo test terminal
cargo test alias

# 3. CLI commands work
./target/release/pxl show examples/walk_cycle.pxl
./target/release/pxl grid examples/heart.pxl
./target/release/pxl inline examples/hero.pxl
./target/release/pxl alias examples/hero.pxl
echo "_ b b _\nb c c b" | ./target/release/pxl sketch --name test

# 4. Round-trip verification
./target/release/pxl alias examples/hero.pxl | ./target/release/pxl inline
# Output should be semantically equivalent

# 5. Workflow verification
echo "_ _ b b\n_ b c b" | ./target/release/pxl sketch --name demo > /tmp/demo.jsonl
./target/release/pxl show /tmp/demo.jsonl
# Should display colored output

# 6. Documentation updated
./target/release/pxl prime | grep show
./target/release/pxl prime | grep sketch
```

---

## Success Criteria

1. All five commands work as documented
2. `pxl show` displays true-color output in compatible terminals
3. `pxl grid` provides accurate coordinate reference
4. `pxl inline` maintains semantic equivalence while improving readability
5. `pxl alias` extracts patterns with consistent frequency-based ordering
6. `pxl sketch` enables rapid prototyping from simple text input
7. Round-trip operations preserve meaning
8. All tests pass
9. Documentation reflects new capabilities
