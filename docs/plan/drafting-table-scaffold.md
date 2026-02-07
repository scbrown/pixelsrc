# Drafting Table: `pxl scaffold`

**Goal:** Generate valid skeleton `.pxl` structures from minimal input, eliminating mechanical drudgery for GenAI agents.

**Status:** Planning

**Epic:** TTP-smmx

**Depends on:** Phase 12 (Tiling - complete), Phase 16 (.pxl format - complete), Phase 20 (Build System - complete)

**Related:**
- [Drafting Table Vision](./drafting-table.md) - Original umbrella design doc
- [Phase 12 Tiling](./phase-12-tiling.md) - `cell_size` composition system
- [Build System](./build-system.md) - `pxl init`, `pxl new` (existing scaffolding)

---

## Overview

`pxl scaffold` generates valid, ready-to-edit `.pxl` structures. An agent says "I need a 16x16 hero sprite" and gets a complete file with palette, empty grid, and correct metadata — no manual boilerplate.

| Subcommand | Purpose |
|------------|---------|
| `pxl scaffold sprite` | Empty sprite with palette and grid |
| `pxl scaffold composition` | Tiled composition with placeholder tile sprites |
| `pxl scaffold palette` | Palette from preset or color list |

### Relationship to `pxl new`

`pxl new` (Phase 20) creates minimal assets inside an existing project. `pxl scaffold` is more powerful:
- Generates complete, self-contained files (palette + sprite in one file)
- Supports tiled compositions with auto-generated tile sprites
- Designed for agent workflows (stdout output, JSON-friendly)
- `pxl new` could be updated to delegate to scaffold internally

---

## Task Dependency Diagram

```
                        PXL SCAFFOLD TASK FLOW
  ═══════════════════════════════════════════════════════════

  WAVE 1 (Foundation)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-S1: CLI subcommand + dispatch               │  │
  │  │  - Scaffold subcommand with sprite/comp/palette  │  │
  │  │  - --output, --format flags                      │  │
  │  │  - Stdout default, file output optional          │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 2 (Parallel - Core Generators)
  ┌───────────────────────────────────────────────────────┐
  │  ┌──────────────────┐  ┌──────────────────────────┐  │
  │  │  DT-S2            │  │  DT-S3                   │  │
  │  │  Scaffold sprite  │  │  Scaffold palette         │  │
  │  │  (grid + palette) │  │  (presets + color list)   │  │
  │  └──────────────────┘  └──────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │                         │
            └────────┬────────────────┘
                     ▼
  WAVE 3 (Depends on sprite + palette)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-S4: Scaffold composition                    │  │
  │  │  - Tiled composition with cell_size              │  │
  │  │  - Auto-generate tile sprites                    │  │
  │  │  - Character map with placeholders               │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
            │
            ▼
  WAVE 4 (Polish)
  ┌───────────────────────────────────────────────────────┐
  │  ┌─────────────────────────────────────────────────┐  │
  │  │  DT-S5: Tests, examples, docs                   │  │
  │  │  - Unit tests for each generator                 │  │
  │  │  - Integration tests (CLI round-trip)            │  │
  │  │  - Example outputs in examples/                  │  │
  │  └─────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────┘
```

---

## Tasks

### DT-S1: CLI Subcommand + Dispatch

**Goal:** Add `pxl scaffold` with sub-subcommands.

**Implementation:**
- Add `Scaffold` variant to `Commands` enum in `src/cli/mod.rs`
- Create `ScaffoldAction` enum: `Sprite`, `Composition`, `Palette`
- Create `src/cli/scaffold.rs` for CLI dispatch
- Create `src/scaffold.rs` for core generation logic (testable without CLI)

**CLI Interface:**
```
pxl scaffold sprite --name "hero" --size 16x16 [--palette medieval] [--tokens "skin,hair,eye"]
pxl scaffold composition --name "level" --size 128x128 --cell-size 32x32 [--palette nature]
pxl scaffold palette --name "warm" [--preset forest|medieval|synthwave] [--colors "#FF0000,#00FF00"]
```

**Common flags:**
- `--output <file>` — Write to file instead of stdout
- `--format pxl|jsonl` — Output format (default: pxl)

**Acceptance:**
- `pxl scaffold --help` shows subcommands
- Each subcommand has `--help` with examples
- Dispatches to correct generator function

---

### DT-S2: Scaffold Sprite

**Goal:** Generate a complete, valid sprite file from minimal parameters.

**Input:** `pxl scaffold sprite --name "hero" --size 16x16 --palette medieval --tokens "skin,hair,eye,outline"`

**Output:**
```
{
  "type": "palette",
  "name": "hero_palette",
  "colors": {
    "{_}": "#00000000",
    "{skin}": "#FFCC99",
    "{hair}": "#663300",
    "{eye}": "#000000",
    "{outline}": "#333333"
  }
}

{
  "type": "sprite",
  "name": "hero",
  "size": [16, 16],
  "palette": "hero_palette",
  "grid": [
    "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
    // ... 16 rows of {_} tokens
  ]
}
```

**Logic:**
- If `--palette <name>` matches a built-in palette, use it
- If `--tokens` provided, generate palette with auto-assigned colors (spread across hue wheel)
- If neither, generate minimal palette with just `{_}`
- Grid is always all-transparent (`{_}`)
- Size parsing: `WxH` format (e.g., "16x16", "32x64")

**Acceptance:**
- Output parses successfully with `pxl validate`
- Output renders to a transparent PNG with `pxl render`
- Built-in palette names resolve correctly
- Custom tokens get reasonable auto-colors

---

### DT-S3: Scaffold Palette

**Goal:** Generate palette definitions from presets or color lists.

**Modes:**
1. **Preset:** `pxl scaffold palette --name "warm" --preset forest`
   - Uses built-in palette presets (medieval, forest, synthwave, ocean, etc.)
   - Output is a standalone palette object

2. **Color list:** `pxl scaffold palette --name "custom" --colors "#FF0000,#00FF00,#0000FF"`
   - Auto-generates token names: `{c1}`, `{c2}`, `{c3}` (or `--token-prefix` to customize)
   - Always includes `{_}` for transparency

3. **From image:** `pxl scaffold palette --name "sampled" --from-image ref.png --max-colors 8`
   - Extract palette from reference image (reuse import logic)

**Acceptance:**
- `--preset` lists available presets when given unknown name
- Color list generates valid hex colors
- Output validates with `pxl validate`

---

### DT-S4: Scaffold Composition

**Goal:** Generate tiled compositions with auto-generated placeholder tile sprites.

**Input:** `pxl scaffold composition --name "level" --size 128x128 --cell-size 32x32 --palette nature`

**Output:**
```
{ "type": "palette", "name": "nature", ... }

{ "type": "sprite", "name": "tile_0_0", "size": [32, 32], "palette": "nature", "grid": [...] }
{ "type": "sprite", "name": "tile_1_0", "size": [32, 32], "palette": "nature", "grid": [...] }
{ "type": "sprite", "name": "tile_0_1", "size": [32, 32], "palette": "nature", "grid": [...] }
{ "type": "sprite", "name": "tile_1_1", "size": [32, 32], "palette": "nature", "grid": [...] }

{
  "type": "composition",
  "name": "level",
  "size": [128, 128],
  "cell_size": [32, 32],
  "sprites": { "A": "tile_0_0", "B": "tile_1_0", "C": "tile_0_1", "D": "tile_1_1" },
  "layers": [{ "map": ["AB", "CD"] }]
}
```

**Logic:**
- Validate size is divisible by cell_size
- Generate `(size[0]/cell_size[0]) * (size[1]/cell_size[1])` tile sprites
- Auto-assign single-char symbols (A-Z, then a-z, then 0-9)
- Tile sprites are transparent grids
- Composition map matches grid dimensions

**Acceptance:**
- Output validates and renders (transparent image of correct size)
- Cell size validation catches non-divisible sizes
- Generates correct number of tile sprites
- Works up to 62 tiles (A-Z + a-z + 0-9); errors above that with clear message

---

### DT-S5: Tests, Examples, Documentation

**Goal:** Comprehensive test coverage and documentation.

**Tests:**
- Unit tests for each generator in `src/scaffold.rs`
- CLI integration tests (run `pxl scaffold` and verify output)
- Round-trip: scaffold → validate → render
- Edge cases: 1x1 sprite, maximum tile count, invalid sizes

**Examples:**
- `examples/demos/scaffold/` directory with sample outputs
- Show sprite, composition, and palette scaffolding

**Documentation:**
- Update `docs/plan/README.md` with scaffold phase
- Add scaffold section to `pxl prime` output
- Help text with usage examples

**Acceptance:**
- All tests pass
- `cargo test` includes scaffold tests
- Examples render correctly

---

## Success Criteria

1. Agent can generate a valid 16x16 sprite file with one command
2. Agent can generate a tiled 128x128 composition with one command
3. All scaffold output passes `pxl validate --strict`
4. All scaffold output renders correctly with `pxl render`
5. Output is in idiomatic `.pxl` format (multi-line, formatted)
