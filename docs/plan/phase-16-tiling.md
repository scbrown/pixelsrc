# Phase 16: Composition Tiling (`cell_size` Extension)

**Goal:** Extend the composition system to support tiling - arranging sprites at larger cell sizes to enable chunked generation of larger images.

**Status:** Planning

**Depends on:** Phase 2 (Composition) complete

---

## Motivation

### The Context Problem

GenAI models have limited context windows. A 64x64 sprite with semantic tokens like `{skin}{hair}{outline}` requires significant token budget. Larger images (128x128, 256x256) become impractical to generate in a single pass.

### The Solution: Tiling

Instead of generating one massive sprite, generate multiple smaller sprites (tiles) and compose them into a larger image. This:

1. Keeps each generation within comfortable context limits
2. Allows parallel generation of independent regions
3. Enables progressive refinement (overview → details)
4. Reuses the existing composition mental model

### Why Extend Composition (Not New Concept)

The current composition system already places sprites on a canvas using a character map:

```jsonl
{"type": "composition", "size": [64, 64], "sprites": {"H": "hero"}, "layers": [{"map": ["..H.."]}]}
```

Each character represents a sprite placed at a position. **Tiling is the same concept at a different scale** - instead of placing sprites at pixel positions, we place them at region positions.

Adding `cell_size` to composition unifies both use cases under one mental model.

---

## Design

### Current Composition (Implicit `cell_size: [1, 1]`)

```jsonl
{"type": "sprite", "name": "hero", "size": [16, 16], "grid": [...]}
{"type": "composition", "name": "scene", "size": [64, 64],
  "sprites": {"H": "hero", "T": "tree"},
  "layers": [{"map": [
    "........",
    "..H.....",
    "........",
    "T......T"
  ]}]
}
```

Each character in the map = 1 pixel position. Sprites are placed at that coordinate.

### Extended Composition (Explicit `cell_size`)

```jsonl
{"type": "sprite", "name": "sky_left", "size": [32, 32], "grid": [...]}
{"type": "sprite", "name": "sky_right", "size": [32, 32], "grid": [...]}
{"type": "sprite", "name": "ground_left", "size": [32, 32], "grid": [...]}
{"type": "sprite", "name": "ground_right", "size": [32, 32], "grid": [...]}

{"type": "composition", "name": "scene", "size": [64, 64],
  "cell_size": [32, 32],
  "sprites": {"A": "sky_left", "B": "sky_right", "C": "ground_left", "D": "ground_right"},
  "layers": [{"map": [
    "AB",
    "CD"
  ]}]
}
```

Each character in the map = one 32x32 cell. The composition renders a 2x2 grid of 32x32 tiles = 64x64 total.

### Key Properties

| Property | Default | Description |
|----------|---------|-------------|
| `cell_size` | `[1, 1]` | Size of each map character in pixels |
| `size` | Required | Total canvas size (must be divisible by cell_size) |

### Validation Rules

1. `size[0]` must be divisible by `cell_size[0]`
2. `size[1]` must be divisible by `cell_size[1]`
3. Sprites placed in cells should ideally match `cell_size` (warning if not, still renders)
4. Map dimensions must match `size / cell_size`

---

## Examples

### Example 1: 2x2 Tiled Scene (64x64 from 32x32 tiles)

```jsonl
{"type": "palette", "name": "nature", "colors": {"{_}": "#00000000", "{sky}": "#87CEEB", "{grass}": "#228B22"}}

{"type": "sprite", "name": "tile_tl", "size": [32, 32], "palette": "nature", "grid": [...]}
{"type": "sprite", "name": "tile_tr", "size": [32, 32], "palette": "nature", "grid": [...]}
{"type": "sprite", "name": "tile_bl", "size": [32, 32], "palette": "nature", "grid": [...]}
{"type": "sprite", "name": "tile_br", "size": [32, 32], "palette": "nature", "grid": [...]}

{"type": "composition", "name": "landscape", "size": [64, 64], "cell_size": [32, 32],
  "sprites": {"1": "tile_tl", "2": "tile_tr", "3": "tile_bl", "4": "tile_br"},
  "layers": [{"map": ["12", "34"]}]
}
```

### Example 2: 3x3 Tiled Scene (96x96 from 32x32 tiles)

```jsonl
{"type": "composition", "name": "large_scene", "size": [96, 96], "cell_size": [32, 32],
  "sprites": {
    "A": "sky_left", "B": "sky_mid", "C": "sky_right",
    "D": "mid_left", "E": "mid_mid", "F": "mid_right",
    "G": "ground_left", "H": "ground_mid", "I": "ground_right"
  },
  "layers": [{"map": [
    "ABC",
    "DEF",
    "GHI"
  ]}]
}
```

### Example 3: Mixed - Tiled Background + Placed Sprites

```jsonl
{"type": "composition", "name": "game_scene", "size": [128, 64], "cell_size": [32, 32],
  "sprites": {
    "S": "sky_tile",
    "G": "ground_tile"
  },
  "layers": [
    {"map": ["SSSS", "GGGG"]}
  ]
}

{"type": "composition", "name": "game_scene_with_characters", "size": [128, 64],
  "sprites": {"H": "hero", "E": "enemy"},
  "layers": [
    {"base": "game_scene"},
    {"map": ["....H...", "......E."]}
  ]
}
```

This shows how tiling and traditional composition can be combined - a tiled background with characters placed on top.

---

## Implementation Notes

### Changes to `composition.rs`

1. Add `cell_size` field to `Composition` struct (default `[1, 1]`)
2. Modify map parsing to scale positions by `cell_size`
3. Add validation for size/cell_size divisibility
4. Update rendering loop to place sprites at scaled positions

### Changes to `models.rs`

```rust
#[derive(Debug, Deserialize)]
pub struct Composition {
    pub name: String,
    pub size: [u32; 2],
    #[serde(default = "default_cell_size")]
    pub cell_size: [u32; 2],
    pub sprites: HashMap<char, String>,
    pub layers: Vec<Layer>,
}

fn default_cell_size() -> [u32; 2] {
    [1, 1]
}
```

### Rendering Logic Change

Current:
```rust
// Position = map character position
let x = col as u32;
let y = row as u32;
```

New:
```rust
// Position = map character position * cell_size
let x = col as u32 * composition.cell_size[0];
let y = row as u32 * composition.cell_size[1];
```

---

## Tasks

### Task 16.1: Add `cell_size` to Composition Model

- Add `cell_size` field to `Composition` struct in `models.rs`
- Default to `[1, 1]` for backwards compatibility
- Add serde deserialization support

### Task 16.2: Update Composition Rendering

- Modify `composition.rs` to use `cell_size` when calculating sprite positions
- Ensure existing compositions (no `cell_size`) work identically

### Task 16.3: Add Validation

- Validate `size` is divisible by `cell_size`
- Validate map dimensions match expected grid (`size / cell_size`)
- Warn (don't error in lenient mode) if sprite size doesn't match `cell_size`

### Task 16.4: Add Examples and Tests

- Create `examples/tiled_scene.jsonl` demonstrating the feature
- Add unit tests for `cell_size` rendering
- Add integration tests for validation rules

### Task 16.5: Update Documentation

- Update `docs/spec/format.md` with `cell_size` specification
- Add examples to website gallery
- Update system prompts with tiling guidance

---

## AI Generation Workflow

This is how an AI would use tiling to generate a large image:

### Step 1: Plan the Layout

```
User: "Create a 128x128 forest scene"
AI thinks: "128x128 is large. I'll tile it as 4x4 grid of 32x32 tiles."
```

### Step 2: Generate Overview (Optional)

AI might first generate a low-res "guide" to plan the scene:
```jsonl
{"type": "sprite", "name": "guide", "size": [4, 4], "grid": [
  "{sky}{sky}{sky}{sky}",
  "{tree}{sky}{sky}{tree}",
  "{tree}{grass}{grass}{tree}",
  "{grass}{grass}{grass}{grass}"
]}
```

### Step 3: Generate Each Tile

AI generates each 32x32 tile, using the guide for context:
```
"Generate tile (0,0): This is top-left, should be mostly sky based on guide"
"Generate tile (1,0): This is top-middle, pure sky"
...
```

### Step 4: Compose Final Image

```jsonl
{"type": "composition", "name": "forest", "size": [128, 128], "cell_size": [32, 32],
  "sprites": {...},
  "layers": [{"map": ["ABCD", "EFGH", "IJKL", "MNOP"]}]
}
```

---

## Relationship to Guide Concept

The "guide" mentioned above is **not a format feature** - it's a workflow pattern:

1. AI generates a small sprite as a planning sketch
2. AI uses that sprite as context when generating detailed tiles
3. The renderer only sees the final composition

This keeps the format simple while enabling sophisticated generation workflows. The guide is just another sprite that the AI references mentally.

---

## Future Considerations

Not in scope for Phase 16:

| Feature | Notes |
|---------|-------|
| Edge constraints | Formal `edges` field to specify tile connectivity (see BACKLOG) |
| Overlap/blending | Tiles that overlap for seamless transitions |
| Auto-tiling | Renderer automatically splits large sprites into tiles |
| Tile libraries | Reusable tile collections for terrain, etc. |

---

## Success Criteria

1. Existing compositions (no `cell_size`) render identically (backwards compatible)
2. `cell_size` compositions render tiles at correct positions
3. Validation catches size/cell_size mismatch
4. Examples demonstrate practical tiled scene creation
5. Documentation explains tiling workflow for AI generation
