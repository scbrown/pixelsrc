# Phase 2: Composition

**Goal:** Unified composition system for layered sprites and scenes

**Status:** Complete

**Depends on:** Phase 1 complete

---

## Scope

Phase 2 adds a unified `composition` type that enables:
- Layering sprites on sprites (character + hat + sword)
- Building tile-based scenes (terrain + objects + actors)
- Palette variants for color swaps
- Configurable grid resolution via `cell_size`

The same concept and terminology works at any scale - from pixel-perfect overlays to large tile-based scenes.

**Not in scope:** Animation, run-length encoding, scene inheritance

---

## Core Concepts

### Composition Type

A composition layers sprites onto a base or canvas:

```json
{
  "type": "composition",
  "name": "hero_equipped",
  "base": "hero_base",
  "cell_size": [4, 4],
  "sprites": {
    ".": null,
    "H": "crown",
    "S": "sword"
  },
  "layers": [
    {"name": "gear", "map": [".H..", "....", "....", "...S"]}
  ]
}
```

```json
{
  "type": "composition",
  "name": "forest_scene",
  "size": [512, 512],
  "cell_size": [16, 16],
  "sprites": {
    ".": null,
    "g": "grass_tile",
    "T": "tree",
    "@": "hero"
  },
  "layers": [
    {"name": "terrain", "fill": "g"},
    {"name": "objects", "map": ["T....T", "......", "......", "T....T"]},
    {"name": "actors", "map": ["......", "..@...", "......", "......"]}
  ]
}
```

### Unified Terminology

| Term | Meaning |
|------|---------|
| `composition` | Type name for any layered sprite assembly |
| `base` | Foundation sprite to build on (size inferred) |
| `fill` | Default sprite to tile the canvas (in layer) |
| `size` | Canvas dimensions in pixels (optional, inferred from base) |
| `cell_size` | Grid resolution - each char in map = this many pixels |
| `sprites` | Map of single-char keys to sprite names |
| `layers` | Array of named grids, rendered bottom to top |
| `map` | The positioning grid within a layer |

### Variant Type

Palette-only variations of existing sprites:

```json
{
  "type": "variant",
  "name": "blue_hat",
  "base": "hat",
  "palette": {
    "{primary}": "#0000FF",
    "{accent}": "#0044AA"
  }
}
```

---

## Task Dependency Diagram

```
                              PHASE 2 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                      Phase 1 Complete                           │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Foundation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   2.1 Composition Model                                  │   │
    │  │   - Add Composition struct to models                     │   │
    │  │   - Parse "type": "composition"                          │   │
    │  │   - sprites map, layers array, basic single-layer        │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - Core Features)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
    │  │   2.2            │  │   2.3            │  │   2.4        │  │
    │  │  Multi-Layer     │  │  Cell Size       │  │  Base & Fill │  │
    │  │  Rendering       │  │  Scaling         │  │  Support     │  │
    │  └──────────────────┘  └──────────────────┘  └──────────────┘  │
    └─────────────────────────────────────────────────────────────────┘
              │                       │                     │
              └───────────────────────┼─────────────────────┘
                                      │
                                      ▼
    WAVE 3 (Parallel - Polish)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────┐  ┌────────────────────────┐  │
    │  │   2.5                        │  │   2.6                  │  │
    │  │  Size Mismatch Handling      │  │  Variant Type          │  │
    │  │  - Top-left anchor           │  │  - Palette overrides   │  │
    │  │  - Warn lenient / err strict │  │  - Base sprite ref     │  │
    │  └──────────────────────────────┘  └────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘
              │                                   │
              └───────────────────┬───────────────┘
                                  │
                                  ▼
    WAVE 4 (Integration)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   2.7 Examples, Tests & CLI                              │   │
    │  │   - Character with gear example                          │   │
    │  │   - Scene with layers example                            │   │
    │  │   - Edge case tests                                      │   │
    │  │   - CLI support for composition rendering                │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 2.1                        (1 task)                    │
    │  Wave 2: 2.2 + 2.3 + 2.4            (3 tasks in parallel)       │
    │  Wave 3: 2.5 + 2.6                  (2 tasks in parallel)       │
    │  Wave 4: 2.7                        (1 task, needs all above)   │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 2.1: Composition Model

**Wave:** 1

Add composition type to data models.

**Deliverables:**
- Update `src/models.rs`:
  ```rust
  pub struct CompositionLayer {
      pub name: Option<String>,
      pub fill: Option<String>,      // Sprite key to fill layer
      pub map: Option<Vec<String>>,  // Grid of sprite keys
  }

  pub struct Composition {
      pub name: String,
      pub base: Option<String>,      // Base sprite name
      pub size: Option<[u32; 2]>,    // Canvas size in pixels
      pub cell_size: Option<[u32; 2]>, // Grid resolution, default [1, 1]
      pub sprites: HashMap<String, Option<String>>, // char -> sprite name
      pub layers: Vec<CompositionLayer>,
  }

  pub enum TtpObject {
      Palette(Palette),
      Sprite(Sprite),
      Composition(Composition),
      // Animation added in Phase 3
  }
  ```
- Basic rendering: single layer, cell_size [1, 1]

**Verification:**
```bash
cargo test models
# Test: Composition JSON deserializes correctly
# Test: sprites map parses single-char keys
# Test: layers array with name and map
# Test: Default cell_size is [1, 1]
```

**Test Fixture:** `tests/fixtures/valid/composition_basic.jsonl`

**Dependencies:** Phase 1 complete

---

### Task 2.2: Multi-Layer Rendering

**Wave:** 2 (parallel with 2.3, 2.4)

Render multiple layers with z-order stacking.

**Deliverables:**
- `src/composition.rs`:
  ```rust
  pub fn render_composition(
      comp: &Composition,
      sprites: &HashMap<String, RgbaImage>,
  ) -> RgbaImage
  // Renders layers bottom to top
  // Later layers overwrite earlier (streaming-friendly)
  ```

**Verification:**
```bash
cargo test composition
# Test: Single layer renders correctly
# Test: Two layers stack (later on top)
# Test: Three+ layers stack correctly
# Test: Layer with all "." renders nothing
```

**Test Fixture:** `tests/fixtures/valid/composition_multilayer.jsonl`

**Dependencies:** Task 2.1

---

### Task 2.3: Cell Size Scaling

**Wave:** 2 (parallel with 2.2, 2.4)

Support configurable grid resolution.

**Deliverables:**
- Update composition renderer:
  ```rust
  // cell_size determines pixels per grid char
  // cell_size: [4, 4] means each char = 4x4 pixel area
  // Sprite top-left aligns to cell top-left
  ```
- Size inference:
  - If `base` provided: canvas = base sprite size
  - If `size` provided: use explicitly
  - Otherwise: infer from layers and cell_size

**Verification:**
```bash
cargo test cell_size
# Test: cell_size [1, 1] - pixel-perfect overlay
# Test: cell_size [4, 4] - 4x4 grid cells
# Test: cell_size [16, 16] - tile-based scene
# Test: Size inference from base sprite
# Test: Size inference from explicit size
```

**Test Fixtures:**
- `tests/fixtures/valid/composition_cellsize_1.jsonl`
- `tests/fixtures/valid/composition_cellsize_16.jsonl`

**Dependencies:** Task 2.1

---

### Task 2.4: Base & Fill Support

**Wave:** 2 (parallel with 2.2, 2.3)

Support base sprites and layer fills.

**Deliverables:**
- `base` field: render base sprite first, layers on top
- `fill` field on layers: tile the specified sprite across entire layer before applying map
- Size inference from base sprite dimensions

**Verification:**
```bash
cargo test base_fill
# Test: base sprite renders as foundation
# Test: layers render on top of base
# Test: fill tiles sprite across layer
# Test: fill + map combines correctly
# Test: Size inferred from base when size omitted
```

**Test Fixtures:**
- `tests/fixtures/valid/composition_with_base.jsonl`
- `tests/fixtures/valid/composition_with_fill.jsonl`

**Dependencies:** Task 2.1

---

### Task 2.5: Size Mismatch Handling

**Wave:** 3 (parallel with 2.6)

Handle sprites larger than cells gracefully.

**Deliverables:**
- When sprite size > cell size:
  - Anchor from top-left of cell
  - Sprite overwrites adjacent cells (streaming-friendly)
  - Lenient mode: emit warning, continue
  - Strict mode: emit error, fail
- Update `src/composition.rs` with bounds checking

**Verification:**
```bash
cargo test size_mismatch
# Test: Sprite exactly fits cell - no warning
# Test: Sprite larger than cell - warning in lenient
# Test: Sprite larger than cell - error in strict
# Test: Large sprite overwrites from top-left correctly
```

**Test Fixtures:**
- `tests/fixtures/lenient/composition_size_mismatch.jsonl`
- `tests/fixtures/invalid/composition_size_mismatch_strict.jsonl`

**Dependencies:** Tasks 2.2, 2.3, 2.4

---

### Task 2.6: Variant Type

**Wave:** 3 (parallel with 2.5)

Add palette-only sprite variants.

**Deliverables:**
- Update `src/models.rs`:
  ```rust
  pub struct Variant {
      pub name: String,
      pub base: String,  // Base sprite name
      pub palette: HashMap<String, String>, // Token -> color overrides
  }

  pub enum TtpObject {
      Palette(Palette),
      Sprite(Sprite),
      Composition(Composition),
      Variant(Variant),
  }
  ```
- Render variant by:
  1. Copy base sprite's grid
  2. Apply palette overrides
  3. Render with modified palette

**Verification:**
```bash
cargo test variant
# Test: Variant with single color override
# Test: Variant with multiple overrides
# Test: Variant referencing unknown base - error
# Test: Variant usable in composition sprites map
```

**Test Fixture:** `tests/fixtures/valid/variant.jsonl`

**Dependencies:** Tasks 2.2, 2.3, 2.4

---

### Task 2.7: Examples, Tests & CLI

**Wave:** 4 (after all above)

Complete integration with examples and CLI.

**Deliverables:**
- Example files:
  - `examples/hero_equipped.jsonl` - character with hat, sword
  - `examples/forest_scene.jsonl` - tile-based scene with layers
  - `examples/color_variants.jsonl` - hat with red/blue/green variants
- CLI support:
  - `pxl render input.jsonl` renders compositions like sprites
  - Composition outputs to `{name}.png`
- Update `demo.sh` with composition examples

**Verification:**
```bash
# Render character composition
./target/release/pxl render examples/hero_equipped.jsonl -o /tmp/
ls /tmp/hero_equipped.png

# Render scene
./target/release/pxl render examples/forest_scene.jsonl -o /tmp/
ls /tmp/forest_scene.png

# Render variants
./target/release/pxl render examples/color_variants.jsonl -o /tmp/
ls /tmp/red_hat.png /tmp/blue_hat.png /tmp/green_hat.png

# Demo shows compositions
./demo.sh
```

**Dependencies:** Tasks 2.5, 2.6

---

## Example Files

### examples/hero_equipped.jsonl

```jsonl
{"type": "palette", "name": "hero_colors", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{hair}": "#8B4513", "{shirt}": "#4169E1", "{pants}": "#2F4F4F"}}
{"type": "palette", "name": "gear_colors", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{steel}": "#708090", "{gem}": "#FF0000"}}
{"type": "sprite", "name": "hero_base", "size": [16, 24], "palette": "hero_colors", "grid": ["..."]}
{"type": "sprite", "name": "crown", "size": [8, 4], "palette": "gear_colors", "grid": ["..."]}
{"type": "sprite", "name": "sword", "size": [4, 12], "palette": "gear_colors", "grid": ["..."]}
{"type": "composition", "name": "hero_equipped", "base": "hero_base", "cell_size": [4, 4], "sprites": {".": null, "C": "crown", "S": "sword"}, "layers": [{"name": "gear", "map": [".C..", "....", "....", "....", "....", "...S"]}]}
```

### examples/forest_scene.jsonl

```jsonl
{"type": "palette", "name": "terrain", "colors": {"{_}": "#00000000", "{g}": "#228B22", "{d}": "#8B4513", "{w}": "#4169E1"}}
{"type": "sprite", "name": "grass_tile", "size": [16, 16], "palette": "terrain", "grid": ["..."]}
{"type": "sprite", "name": "tree", "size": [32, 48], "palette": "terrain", "grid": ["..."]}
{"type": "sprite", "name": "hero", "size": [16, 24], "palette": "hero_colors", "grid": ["..."]}
{"type": "composition", "name": "forest_scene", "size": [256, 256], "cell_size": [16, 16], "sprites": {".": null, "g": "grass_tile", "T": "tree", "@": "hero"}, "layers": [{"name": "terrain", "fill": "g"}, {"name": "objects", "map": ["T.....T.........T...", "...................."]}, {"name": "actors", "map": [".....................", "........@..........."]}]}
```

---

## demo.sh Updates

After Phase 2, demo.sh shows:

```bash
echo "── Phase 2: Composition ────────────────────────────────────────"

echo "Character Composition:"
echo "Input: examples/hero_equipped.jsonl"
$PXL render examples/hero_equipped.jsonl -o /tmp/hero_equipped.png
echo "Output: /tmp/hero_equipped.png"
echo ""

echo "Scene Composition:"
echo "Input: examples/forest_scene.jsonl"
$PXL render examples/forest_scene.jsonl -o /tmp/forest_scene.png
echo "Output: /tmp/forest_scene.png"
echo ""

echo "Color Variants:"
$PXL render examples/color_variants.jsonl -o /tmp/
echo "Outputs: /tmp/red_hat.png, /tmp/blue_hat.png, /tmp/green_hat.png"
```

---

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. Composition fixtures parse and render
./target/release/pxl render tests/fixtures/valid/composition_basic.jsonl

# 3. Multi-layer stacking
./target/release/pxl render tests/fixtures/valid/composition_multilayer.jsonl -o /tmp/
open /tmp/multilayer.png

# 4. Cell size variants
./target/release/pxl render tests/fixtures/valid/composition_cellsize_16.jsonl -o /tmp/
open /tmp/scene.png

# 5. Variants render with overridden colors
./target/release/pxl render tests/fixtures/valid/variant.jsonl -o /tmp/
open /tmp/blue_hat.png

# 6. Demo updated
./demo.sh
```

---

## Future Phases (Related)

These features were discussed but deferred:

**Phase 6: Token Efficiency**
- Run-length encoding: `g*32` for 32 grass tiles
- Row repetition: `{"row": "gggg", "repeat": 10}`
- Default fill with exceptions

**Phase 7: Inheritance**
- `"extends": "village_day"` for scene variants
- Palette swaps across entire compositions
- Day/night, seasonal, weather variants
