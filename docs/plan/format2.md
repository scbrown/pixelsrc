# Phase 25: Format Optimization for GenAI Scale

**Goal:** Reduce token overhead and improve AI readability for large sprites (Fallout 2-style scenes)

**Status:** Planning

**Depends on:** Phase 16 (Multi-line JSON)

---

## Motivation

A 320x320 imported sprite produces 462KB of .pxl data. The `{token}` syntax is verbose:
- Current: `{c10}{c10}{c11}` = 15 chars for 3 pixels
- With 2-letter aliases: `cacaca` = 6 chars
- With RLE: `ca*3` = 4 chars

For GenAI to produce Fallout 2-style scenes, we need compact formats.

---

## Features

### Feature A: JSON5 Parser

Replace strict JSON with JSON5 for more forgiving syntax.

**Benefits:**
- Comments (`// AI can think out loud`)
- Trailing commas (less fragile generation)
- Unquoted keys (optional)

**Implementation:**
- Add `json5` crate dependency
- Update parser to use `json5::from_str()`
- All existing .pxl files continue to work (JSON is valid JSON5)

---

### Feature B: Compact Grid Format

Replace brace-delimited concatenated tokens with aliased format.

**New format:**
```json
{
  "type": "sprite",
  "name": "coin",
  "size": [8, 2],
  "palette": "coin_palette",
  "tokens": {
    "__": "{_}",
    "go": "{gold}",
    "sh": "{shine}",
    "da": "{shadow}"
  },
  "grid": [
    "____gogogogo____",
    "__goshshshgogo__"
  ]
}
```

**Behavior:**
- `tokens` field maps short aliases to full `{token}` names
- Parser uses greedy longest-match to split tokens (no spaces required)
- `size` provides expected width for validation
- Spaces are optional - used for readability/alignment only

**Delimiter-free parsing:**
```json
"grid": [
  "__gogogogogo____",
  "__goshshshgogo__"
]
```
Parser knows width=8 from `size`, matches tokens greedily: `__` `go` `go` `go` `go` `go` `__` `__`

**With optional spaces for readability:**
```json
"grid": [
  "__ go go go go go __ __",
  "__ go sh sh sh go go __"
]
```
Spaces ignored during parsing, just for human readability.

---

### Feature C: RLE (Run-Length Encoding)

Compress repeated tokens.

**Syntax:**
```json
"grid": [
  "__go*6__",         // 6 consecutive gold tokens
  "__gosh*2go*3__",   // 2 shine, 3 gold
  "go*8"              // entire row of gold
]
```

**Parsing rules:**
- `token*N` expands to N copies of token
- `N` must be positive integer
- Parsed during greedy matching

---

### Feature D: `pxl compact` Command

Convert existing files to use short aliases and compact format.

**Usage:**
```bash
pxl compact input.pxl -o output.pxl
pxl compact input.pxl --in-place
pxl import image.png -o output.pxl --compact
```

**Algorithm:**
1. Parse file, collect all unique tokens
2. Generate 2-letter aliases (aa, ab, ac... ba, bb...)
   - Reserve `__` for transparency
   - Assign by frequency (most common = shortest/first)
3. Add `tokens` mapping to each sprite
4. Rewrite grid in compact format
5. Optionally apply RLE to repeated tokens

**Example transformation:**
```
Before (462KB):
{"type":"sprite","grid":["{c10}{c10}{c11}{c12}..."]}

After (~100KB):
{
  "type": "sprite",
  "tokens": {"__": "{_}", "ca": "{c10}", "cb": "{c11}", "cc": "{c12}"},
  "grid": ["cacacacbcc..."]
}
```

---

### Feature E: Formatting with Spacing

Spacing is purely for human readability - parser ignores spaces.

**`pxl fmt --spacing 0`** (default, most compact):
```json
"grid": [
  "____gogogogogo____",
  "__goshshshgogo__"
]
```

**`pxl fmt --spacing 1`** - Single space between tokens:
```json
"grid": [
  "__ __ go go go go go go __ __",
  "__ go sh sh sh sh go go __ __"
]
```

**`pxl fmt --align`** - Pad to align columns (for variable-length tokens):
```json
"grid": [
  "__   __   go   go   go   go   __   __",
  "__   go   sh   sh   go   go   go   __"
]
```

**`pxl show`** - Display with spacing for readability (default).

---

## Refactoring: alias.rs

Current `alias.rs` mixes concerns. Split into:

**`src/alias.rs`** (token aliasing only):
- `extract_aliases(grid) -> (HashMap, Vec<String>)`
- `expand_aliases(grid, aliases) -> Vec<String>`
- `generate_two_letter_aliases(tokens) -> HashMap<String, String>`

**`src/grid.rs`** (grid parsing & formatting):
- `parse_grid_row(row, tokens, width) -> Vec<String>` - greedy longest-match parser
- `parse_rle(token) -> Vec<String>` - expand `go*5` to 5 copies
- `format_grid_aligned(rows, spacing) -> Vec<String>` - column alignment
- Move existing `parse_simple_grid`, `format_columns` here

**Existing code reuse:**
- `format_columns` already does column alignment
- Extend with greedy parser and RLE support

---

## Task Breakdown

### Wave 1: JSON5 Foundation
- [ ] **25.1** Add json5 crate, update parser
- [ ] **25.2** Update tests to verify JSON5 features work
- [ ] **25.3** Document JSON5 support in primer.md

### Wave 2: New Grid Format
- [ ] **25.4** Add `tokens` field to Sprite model
- [ ] **25.5** Implement greedy longest-match parser (uses `size` width + `tokens` map)
- [ ] **25.6** Make spaces optional (strip before parsing)
- [ ] **25.7** Implement RLE parsing (`go*10` → 10 copies)
- [ ] **25.8** Update format.md spec
- [ ] **25.9** Update all test fixtures to new format

### Wave 3: Compact Command
- [ ] **25.10** Implement 2-letter alias generation (frequency-based, case-sensitive)
- [ ] **25.11** Add `pxl compact` CLI command
- [ ] **25.12** Add `--compact` flag to `pxl import`
- [ ] **25.13** Apply RLE to output when beneficial
- [ ] **25.14** Add round-trip tests

### Wave 4: Integration & Refactor
- [ ] **25.15** Refactor alias.rs → alias.rs + grid.rs
- [ ] **25.16** Add `pxl fmt --spacing N` and `--align` options
- [ ] **25.17** Update `pxl show` to display with alignment by default
- [ ] **25.18** Update primer.md with new format examples
- [ ] **25.19** Test with Fallout import: measure size reduction

---

## Design Decisions

1. **New format only** - No backward compat with `{a}{b}{c}` concatenated format
2. **Greedy longest-match parser** - Uses `size` width + `tokens` map to parse without delimiters
3. **Spaces are optional** - For readability only, ignored by parser
4. **Token aliases are case-sensitive:** `aa` ≠ `AA` ≠ `Aa` (2704 unique 2-letter combos)
5. **Reserved alias:** `__` for transparency
6. **RLE syntax:** `token*N` for repeated tokens
7. **Spacing for display:** `pxl fmt --spacing N` adds spaces for readability
8. **Update all examples** - Migrate docs, tests, fixtures to new format

---

## Success Criteria

1. JSON5 syntax works (comments, trailing commas)
2. Greedy parser correctly splits tokens using `size` and `tokens` map
3. `tokens` field expands aliases properly
4. RLE syntax (`go*10`) expands correctly
5. `pxl compact` produces valid, smaller files
6. Fallout 320x320 import: 462KB → <100KB
7. All examples/tests updated to new format
8. Round-trip: original → compact → render = identical output

---

## Files to Modify

- `Cargo.toml` - add json5 dependency
- `src/models.rs` - add `tokens` field to Sprite
- `src/parser.rs` - greedy parser, token expansion
- `src/alias.rs` - refactor, add 2-letter generation
- `src/grid.rs` (new) - grid parsing/formatting utilities
- `src/cli.rs` - add `compact` command, fmt options
- `src/compact.rs` (new) - compaction logic
- `src/import.rs` - add `--compact` flag
- `src/fmt.rs` - add `--spacing` and `--align` options
- `docs/spec/format.md` - document new format
- `docs/primer.md` - update examples

---

## Verification

```bash
# JSON5 works
echo '{type: "palette", name: "test", colors: {"{_}": "#0000",}}' | pxl validate -

# Delimiter-free grid works
cat > /tmp/test.pxl << 'EOF'
{"type": "palette", "name": "p", "colors": {"{_}": "#0000", "{red}": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [4, 2], "palette": "p", "tokens": {"__": "{_}", "rr": "{red}"}, "grid": ["__rrrr__", "rrrrrrrr"]}
EOF
pxl render /tmp/test.pxl -o /tmp/test.png

# With optional spaces (same result)
cat > /tmp/test2.pxl << 'EOF'
{"type": "palette", "name": "p", "colors": {"{_}": "#0000", "{red}": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [4, 2], "palette": "p", "tokens": {"__": "{_}", "rr": "{red}"}, "grid": ["__ rr rr __", "rr rr rr rr"]}
EOF
pxl render /tmp/test2.pxl -o /tmp/test2.png
diff /tmp/test.png /tmp/test2.png  # Identical

# RLE works
cat > /tmp/test3.pxl << 'EOF'
{"type": "palette", "name": "p", "colors": {"{_}": "#0000", "{red}": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [8, 1], "palette": "p", "tokens": {"__": "{_}", "rr": "{red}"}, "grid": ["__rr*6__"]}
EOF
pxl render /tmp/test3.pxl -o /tmp/test3.png

# Compact import
pxl import /tmp/fallout_example.png -o /tmp/fallout.pxl --compact
ls -la /tmp/fallout.pxl  # Should be much smaller than 462KB

# Round-trip verification
pxl render /tmp/fallout_example.png -o /tmp/before.png
pxl render /tmp/fallout.pxl -o /tmp/after.png
diff /tmp/before.png /tmp/after.png  # Should be identical
```
