# Drafting Table: `pxl draw`

**Goal:** Coordinate-based sprite editing that moves spatial logic from the LLM to the Rust binary. Modifies files in-place by default.

**Status:** Planning

**Epic:** TTP-xxby

**Depends on:** Phase 16 (.pxl format - complete)

**Related:**
- [Drafting Table Vision](./drafting-table.md) - Original umbrella design doc
- [Structured Format Spec](./structured-format-spec.md) - RegionDef shape primitives

---

## Overview

`pxl draw` is the first command that **modifies** existing `.pxl` files. It provides precise, coordinate-based sprite manipulation so agents don't need to count characters in grid strings or reason about spatial positions.

| Operation | Example |
|-----------|---------|
| Set pixel | `pxl draw f.pxl --sprite hero --set 5,10="{eye}"` |
| Fill rect | `pxl draw f.pxl --sprite hero --rect 0,0,16,4="{sky}"` |
| Draw line | `pxl draw f.pxl --sprite hero --line 0,0,15,15="{rope}"` |
| Fill circle | `pxl draw f.pxl --sprite hero --circle 8,8,4="{gem}"` |
| Flood fill | `pxl draw f.pxl --sprite hero --flood 5,5="{water}"` |
| Erase | `pxl draw f.pxl --sprite hero --erase 5,10` |

### Key Design Decisions

- **In-place by default**: Like `pxl fmt`, reads and writes back to same file
- **`--output <file>`**: Write to different file instead
- **`--dry-run`**: Show what would change without writing
- **Grid sprites only (initially)**: Region-based sprites are a future extension
- **Multiple ops per invocation**: Process operations left-to-right
- **Lenient token handling**: Warn (don't error) if token not in palette

---

## Task Dependency Diagram

```
                         PXL DRAW TASK FLOW
  ═══════════════════════════════════════════════════════════

  WAVE 1 (Foundation)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-D1: File read-modify-write pipeline          │  │
  │  │  - Parse file, locate sprite, modify, reserialize│  │
  │  │  - --output and --dry-run support                │  │
  │  │  - CLI subcommand registration                   │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 2 (Foundation - Grid Editing Core)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-D2: Grid token editing engine                │  │
  │  │  - Parse grid rows into 2D token array           │  │
  │  │  - Set token at (x,y)                            │  │
  │  │  - Reserialize 2D array back to grid strings     │  │
  │  │  - Coordinate bounds checking                    │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 3 (Parallel - Shape Operations)
  ┌───────────────────────────────────────────────────────┐
  │  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
  │  │  DT-D3       │  │  DT-D4       │  │  DT-D5     │  │
  │  │  --rect fill │  │  --line      │  │  --circle   │  │
  │  │              │  │  (Bresenham) │  │  --ellipse  │  │
  │  └──────────────┘  └──────────────┘  └────────────┘  │
  └───────────────────────────────────────────────────────┘
            │                 │                │
            └────────┬────────┘────────────────┘
                     ▼
  WAVE 4 (Depends on set pixel)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-D6: Flood fill                              │  │
  │  │  - BFS/DFS from seed point                       │  │
  │  │  - Fills connected region of same token          │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 5 (Polish)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-D7: Tests, examples, docs                   │  │
  │  │  - Unit tests for each operation                 │  │
  │  │  - Integration tests (CLI round-trip)            │  │
  │  │  - Edge cases and error handling                 │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
```

---

## Tasks

### DT-D1: File Read-Modify-Write Pipeline

**Goal:** Establish the core pattern for commands that modify `.pxl` files.

**This is the hardest task** — it defines the editing architecture that `draw` (and future editing commands) will use.

**Implementation:**
- Add `Draw` variant to `Commands` enum in `src/cli/mod.rs`
- Create `src/cli/draw.rs` for CLI dispatch
- Create `src/draw.rs` for core editing logic

**Pipeline:**
1. Read file content (`fs::read_to_string`)
2. Parse into `Vec<TtpObject>` (preserving order and non-sprite objects)
3. Find target sprite by `--sprite <name>`
4. Apply modifications to sprite's grid
5. Reserialize all objects back to `.pxl` format
6. Write back to file (or `--output` path)

**Key challenge:** Preserving file structure during round-trip. The parser currently discards formatting. Options:
- **Option A:** Parse → modify → format with `pxl fmt` logic (changes formatting but preserves semantics)
- **Option B:** Line-level surgery (find the sprite's JSON object, modify only that section)
- **Recommendation:** Option A — simpler, and `pxl fmt` output is the canonical format anyway

**CLI Interface:**
```
pxl draw <file> --sprite <name> [operations...] [--output <file>] [--dry-run]
```

**Acceptance:**
- Can read a `.pxl` file, find a sprite, and write it back unchanged
- `--output` writes to different file
- `--dry-run` prints diff without writing
- Non-sprite objects (palettes, compositions, animations) are preserved
- Error if sprite not found

---

### DT-D2: Grid Token Editing Engine

**Goal:** Parse grid strings into a 2D token array, modify individual cells, and reserialize.

**Grid Parsing:**
```
Grid string: "{skin}{hair}{eye}"
→ Token array: ["skin", "hair", "eye"]
```

The grid uses `{token}` syntax where each token is delimited by braces. Parse by scanning for `{...}` patterns.

**Operations:**
- `set(x, y, token)` — Set single cell
- `erase(x, y)` — Set cell to `{_}` (transparent)
- `get(x, y)` → token name (for validation/dry-run)

**Reserialization:**
- Convert 2D token array back to grid strings
- Maintain consistent formatting (tokens left-justified, matching `pxl fmt` style)

**Coordinate system:**
- (0,0) is top-left
- x = column (horizontal), y = row (vertical)
- Out-of-bounds is an error

**Acceptance:**
- `pxl draw f.pxl --sprite hero --set 5,10="{eye}"` modifies correct pixel
- `pxl draw f.pxl --sprite hero --erase 5,10` sets pixel to `{_}`
- Round-trip: original file → draw → render matches expected output
- Out-of-bounds coordinates produce clear error message

---

### DT-D3: Rectangle Fill

**Goal:** Fill a rectangular region with a token.

**Syntax:** `--rect x,y,w,h="{token}"`

**Implementation:**
- Iterate x..x+w, y..y+h and set each cell
- Clamp to sprite bounds (warn if rect extends beyond)

**Acceptance:**
- `pxl draw f.pxl --sprite hero --rect 0,0,16,4="{sky}"` fills top 4 rows
- Partial overlap with sprite bounds: fills what fits, warns about clipped region
- Zero-width or zero-height rect is a no-op with warning

---

### DT-D4: Line Drawing (Bresenham)

**Goal:** Draw a line between two points using Bresenham's algorithm.

**Syntax:** `--line x1,y1,x2,y2="{token}"`

**Implementation:**
- Standard Bresenham's line algorithm
- Existing `shapes.rs` may have line logic to reuse from region rendering

**Acceptance:**
- Horizontal, vertical, and diagonal lines work
- Single-pixel line (same start/end) sets one pixel
- Out-of-bounds endpoints: draw the visible portion, warn

---

### DT-D5: Circle and Ellipse Fill

**Goal:** Fill circular/elliptical regions.

**Syntax:**
- `--circle cx,cy,r="{token}"` — Filled circle
- `--ellipse cx,cy,rx,ry="{token}"` — Filled ellipse

**Implementation:**
- Midpoint circle algorithm (or iterate bounding box, test distance)
- Existing `shapes.rs` has circle/ellipse rendering for regions — reuse

**Acceptance:**
- Circle centered at (8,8) radius 4 fills expected pixels
- Radius 0 fills single pixel
- Ellipse with different rx/ry creates oval shape

---

### DT-D6: Flood Fill

**Goal:** Fill connected region of same token from a seed point.

**Syntax:** `--flood x,y="{token}"`

**Implementation:**
- BFS from (x,y)
- Fill all 4-connected pixels matching the original token at (x,y)
- Stop at sprite bounds and different tokens
- Guard against filling with same token (no-op)

**Acceptance:**
- Flood fill from center of `{_}` region fills all connected `{_}` pixels
- Does not cross token boundaries
- Filling with same token is a no-op (not an error)
- Large sprites don't stack overflow (use iterative BFS, not recursive DFS)

---

### DT-D7: Tests, Examples, Documentation

**Goal:** Comprehensive testing and documentation.

**Tests:**
- Unit tests for grid parsing/reserialization
- Unit tests for each shape operation
- Integration tests: `pxl draw` CLI round-trip
- Round-trip: scaffold → draw → render → verify pixels
- Edge cases: 1x1 sprite, draw at boundaries, empty sprite

**Examples:**
- Example workflow: scaffold → draw basic shapes → render
- Before/after showing draw operations

**Documentation:**
- Update `docs/plan/README.md`
- Add draw section to `pxl prime` output
- Help text with usage examples for each operation

**Acceptance:**
- All tests pass
- Each operation has at least 3 test cases (normal, boundary, error)

---

## Multiple Operations Per Invocation

Operations are processed left-to-right:
```
pxl draw hero.pxl --sprite hero \
  --rect 0,0,16,16="{sky}" \
  --rect 0,12,16,4="{grass}" \
  --circle 8,4,2="{sun}" \
  --set 8,8="{flower}"
```

This fills sky, then grass over bottom, then sun, then a flower — all in one invocation, one file write.

---

## Success Criteria

1. Agent can modify individual pixels without editing raw grid strings
2. Agent can fill regions, draw lines, and circles with single commands
3. File round-trip preserves all non-modified content
4. `--dry-run` shows changes without modifying files
5. All modified files pass `pxl validate --strict`
