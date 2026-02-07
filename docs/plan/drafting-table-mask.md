# Drafting Table: `pxl mask`

**Goal:** Read-only sprite state queries that let agents inspect what exists before modifying. The "eyes" of the Drafting Table.

**Status:** Planning

**Epic:** TTP-9veg

**Depends on:** Phase 16 (.pxl format - complete)

**Related:**
- [Drafting Table Vision](./drafting-table.md) - Original umbrella design doc
- [Phase 15 AI Tools](./phase-15-ai-tools.md) - `pxl explain`, `pxl validate` (related inspection tools)

---

## Overview

GenAI agents are stateless — they lose track of what they've drawn between context turns. `pxl mask` provides structured queries that let agents recover sprite state without parsing raw grid strings visually.

| Operation | Example | Output |
|-----------|---------|--------|
| Query token | `--query "{skin}"` | List of (x,y) coordinates |
| Sample pixel | `--sample 5,5` | Token name at position |
| Dump region | `--region 0,0,8,8` | All tokens in rectangle |
| Bounding box | `--bounds "{skin}"` | `[x, y, w, h]` of token extent |
| Token count | `--count` | Count of each token |
| Neighbors | `--neighbors 5,5` | Tokens at 4-connected neighbors |
| List sprites | `--list` | All sprite names in file |

All operations are **read-only** — `pxl mask` never modifies files.

---

## Task Dependency Diagram

```
                         PXL MASK TASK FLOW
  ═══════════════════════════════════════════════════════════

  WAVE 1 (Foundation)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-M1: CLI subcommand + token grid extraction  │  │
  │  │  - Mask subcommand registration                  │  │
  │  │  - Parse file, locate sprite                     │  │
  │  │  - Extract 2D token grid from grid/region sprite │  │
  │  │  - --json output mode                            │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 2 (Parallel - Core Queries)
  ┌───────────────────────────────────────────────────────┐
  │  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
  │  │  DT-M2       │  │  DT-M3       │  │  DT-M4     │  │
  │  │  --query     │  │  --sample    │  │  --region   │  │
  │  │  --bounds    │  │  --neighbors │  │  --count    │  │
  │  └──────────────┘  └──────────────┘  └────────────┘  │
  └───────────────────────────────────────────────────────┘
            │                 │                │
            └────────┬────────┘────────────────┘
                     ▼
  WAVE 3 (Utility)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-M5: --list and sprite metadata queries      │  │
  │  │  - List all sprites in file                      │  │
  │  │  - Show sprite size, palette, token inventory    │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 4 (Polish)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-M6: Tests, examples, docs                   │  │
  │  │  - Unit tests for each query operation           │  │
  │  │  - Integration tests (CLI round-trip)            │  │
  │  │  - Agent workflow examples                       │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
```

---

## Tasks

### DT-M1: CLI Subcommand + Token Grid Extraction

**Goal:** Register the `pxl mask` command and build the token grid extraction engine.

**Implementation:**
- Add `Mask` variant to `Commands` enum in `src/cli/mod.rs`
- Create `src/cli/mask.rs` for CLI dispatch
- Create `src/mask.rs` for core query logic

**Token Grid Extraction:**
The core challenge: both grid-based and region-based sprites need to produce a 2D `Vec<Vec<String>>` token grid.

- **Grid sprites:** Parse `{token}` patterns from grid strings (same logic as DT-D2 in draw)
- **Region sprites:** Render regions to get token-at-pixel map (reuse structured renderer logic)
- Share the grid extraction code with `pxl draw` (extract to common module)

**Common flags:**
- `--sprite <name>` — Target sprite (required for most operations)
- `--json` — Output as structured JSON
- File argument is positional

**Acceptance:**
- `pxl mask --help` shows all available operations
- Can extract token grid from both grid and region sprites
- `--json` produces parseable JSON output

---

### DT-M2: Query and Bounds Operations

**Goal:** Find where tokens are located in a sprite.

**`--query "{token}"`:**
```
$ pxl mask hero.pxl --sprite hero --query "{eye}"
{eye}: 4 pixels
  (5, 3)
  (6, 3)
  (9, 3)
  (10, 3)
```

JSON mode:
```json
{"token": "{eye}", "count": 4, "coords": [[5,3], [6,3], [9,3], [10,3]]}
```

**`--bounds "{token}"`:**
```
$ pxl mask hero.pxl --sprite hero --bounds "{skin}"
{skin}: bounding box [3, 2, 10, 12]  (x=3, y=2, w=10, h=12)
```

JSON mode:
```json
{"token": "{skin}", "bounds": [3, 2, 10, 12], "pixel_count": 48}
```

**Implementation:**
- Iterate token grid, collect coordinates matching token
- Bounds: min/max x/y of matching coordinates → [x, y, w, h]

**Acceptance:**
- Query returns all coordinates of a token
- Bounds returns correct bounding box
- Token not found: empty result (not an error)
- `{_}` can be queried like any other token

---

### DT-M3: Sample and Neighbors

**Goal:** Point queries for inspecting specific coordinates.

**`--sample x,y`:**
```
$ pxl mask hero.pxl --sprite hero --sample 5,3
(5, 3): {eye}
```

JSON mode:
```json
{"x": 5, "y": 3, "token": "{eye}"}
```

**`--neighbors x,y`:**
```
$ pxl mask hero.pxl --sprite hero --neighbors 5,3
(5, 3): {eye}
  up    (5, 2): {skin}
  down  (5, 4): {skin}
  left  (4, 3): {skin}
  right (6, 3): {eye}
```

JSON mode:
```json
{"x": 5, "y": 3, "token": "{eye}", "neighbors": {"up": "{skin}", "down": "{skin}", "left": "{skin}", "right": "{eye}"}}
```

**Implementation:**
- Direct index into token grid
- Neighbors: check bounds, return null/omit for edge pixels

**Acceptance:**
- Sample returns correct token
- Out-of-bounds coordinates produce clear error
- Neighbors at edges omit out-of-bounds directions
- Corner pixel shows only 2 neighbors

---

### DT-M4: Region Dump and Count

**Goal:** Bulk inspection operations.

**`--region x,y,w,h`:**
```
$ pxl mask hero.pxl --sprite hero --region 4,2,4,4
Region (4,2)-(8,6):
  {skin}{skin}{eye}{skin}
  {skin}{skin}{skin}{skin}
  {skin}{nose}{skin}{skin}
  {skin}{mouth}{mouth}{skin}
```

JSON mode: 2D array of tokens.

**`--count`:**
```
$ pxl mask hero.pxl --sprite hero --count
Token counts for "hero" (16x16 = 256 pixels):
  {_}:       180  (70.3%)
  {skin}:     42  (16.4%)
  {outline}:  20  (7.8%)
  {hair}:      8  (3.1%)
  {eye}:       4  (1.6%)
  {mouth}:     2  (0.8%)
```

JSON mode: `{"tokens": {"{_}": 180, "{skin}": 42, ...}, "total": 256}`

**Implementation:**
- Region: slice 2D token grid
- Count: iterate grid, build frequency map, sort by count descending

**Acceptance:**
- Region dump matches visual grid content
- Count percentages sum to 100%
- Region that extends beyond sprite bounds: show what fits, warn
- Empty sprite returns all `{_}`

---

### DT-M5: List and Sprite Info

**Goal:** File-level queries for discovering what's in a `.pxl` file.

**`--list`:**
```
$ pxl mask hero.pxl --list
Sprites in hero.pxl:
  hero          16x16  palette: hero_palette  (grid)
  hero_shadow   16x16  palette: hero_palette  (grid)
  hero_outline  16x16  palette: hero_palette  (regions)

Compositions:
  hero_sheet    64x16

Animations:
  hero_walk     4 frames, 400ms
```

JSON mode: structured inventory of all objects.

**Implementation:**
- Parse file, enumerate all TtpObject entries
- For sprites: show name, size, palette, format (grid vs regions)
- For compositions/animations: show key metadata

**Acceptance:**
- Lists all sprites, compositions, animations
- Shows useful metadata for each
- Works with both `.pxl` and `.jsonl` files

---

### DT-M6: Tests, Examples, Documentation

**Goal:** Comprehensive testing and agent workflow documentation.

**Tests:**
- Unit tests for token grid extraction (grid and region sprites)
- Unit tests for each query operation
- Integration tests: `pxl mask` CLI with various flags
- Test with real example files from `examples/`

**Examples:**
- Agent workflow: mask → inspect → draw → mask (verify)
- Show how mask enables stateless editing

**Documentation:**
- Update `docs/plan/README.md`
- Add mask section to `pxl prime` output
- Help text with usage examples

**Acceptance:**
- All tests pass
- Each operation has at least 3 test cases
- Agent workflow example demonstrates practical value

---

## Agent Workflow Example

How a stateless agent uses mask + draw together:

```bash
# 1. Scaffold a new sprite
pxl scaffold sprite --name hero --size 16x16 --tokens "skin,hair,eye,mouth" > hero.pxl

# 2. Draw the head outline
pxl draw hero.pxl --sprite hero --circle 8,8,6="{skin}"

# 3. Check what we have
pxl mask hero.pxl --sprite hero --count --json
# → {"tokens": {"{_}": 143, "{skin}": 113}}

# 4. Add eyes
pxl draw hero.pxl --sprite hero --set 6,6="{eye}" --set 10,6="{eye}"

# 5. Verify eye placement
pxl mask hero.pxl --sprite hero --query "{eye}" --json
# → {"token": "{eye}", "count": 2, "coords": [[6,6], [10,6]]}

# 6. Check symmetry
pxl mask hero.pxl --sprite hero --neighbors 6,6 --json
# → {"neighbors": {"up": "{skin}", "down": "{skin}", "left": "{skin}", "right": "{skin}"}}
```

---

## Success Criteria

1. Agent can find all locations of a token without visual parsing
2. Agent can inspect any pixel by coordinate
3. Agent can get a spatial overview (region dump, counts, bounds)
4. All queries work on both grid and region sprites
5. JSON output is machine-parseable and consistent
6. Zero writes to any file — strictly read-only
