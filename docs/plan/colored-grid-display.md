# Plan: Colored Terminal Output & Alias Management

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

1. `pxl show examples/walk_cycle.jsonl` - colored grid with legend
2. `pxl grid examples/heart.jsonl` - coordinate reference display
3. `pxl inline examples/hero.jsonl` - column-aligned output
4. `pxl alias examples/hero.jsonl` - extract to single-letter aliases
5. `echo "_ _ b b\n_ b c b" | pxl sketch --name test` - simple grid to sprite
6. Round-trip: `pxl alias | pxl inline` preserves meaning
7. Workflow: `pxl sketch` -> edit colors -> `pxl show` to verify

## Tasks

### Implementation
1. Create `src/terminal.rs` with ANSI color utilities
2. Create `src/alias.rs` with alias extraction/expansion/simple-grid parsing
3. Add `Show` command to CLI
4. Add `Grid` command to CLI
5. Add `Inline` command to CLI
6. Add `Alias` command to CLI
7. Add `Sketch` command to CLI
8. Add tests for new functionality

### Documentation & Polish
9. Update `pxl prime` output with new commands and workflow
10. Update `docs/spec/format.md` if format changes needed
11. Update `docs/prompts/` guides with new generation workflow
12. Update `src/lib.rs` exports for any new public APIs
13. Update CLI help text and command descriptions
14. Update `demo.sh` with examples of all new commands
15. Review and update any other user-facing docs in `docs/`

## Dependencies

No new crate dependencies needed - ANSI codes are simple string formatting.
