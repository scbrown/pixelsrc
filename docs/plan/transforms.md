---
phase: 18
title: Sprite Transforms
---

# Phase 18: Sprite Transforms

**Status:** In Progress (CLI command implemented, format support partial)

**Depends on:** Phase 17 (Colored Grid Display - for `pxl show` visualization)

---

Add transform operations for pixelsrc sprites at two levels:
1. **CLI command** (`pxl transform`) - Source-to-source transformation, outputs new `.pxl`
2. **Format attribute** - Declarative transforms applied at render time (source unchanged)

**Related:** [Colored Grid Display](./colored-grid-display.md) - Use `pxl show` to visualize transforms in terminal

---

## CLI Command: `pxl transform`

### Usage

```bash
pxl transform <input> [options] -o <output>
```

### Flags

| Flag | Description | Example |
|------|-------------|---------|
| `--mirror <axis>` | Mirror horizontally or vertically | `--mirror horizontal`, `--mirror vertical`, `--mirror both` |
| `--rotate <degrees>` | Rotate by 90, 180, or 270 degrees | `--rotate 90` |
| `--flip <axis>` | Alias for mirror (common terminology) | `--flip h`, `--flip v` |
| `--tile <WxH>` | Tile sprite into grid | `--tile 3x2` |
| `--pad <N>` | Add N pixels of transparent padding | `--pad 2` |
| `--outline [color]` | Add 1px outline around opaque pixels | `--outline`, `--outline "#000"` |
| `--crop <X,Y,W,H>` | Extract sub-region | `--crop 0,0,8,8` |
| `--shift <X,Y>` | Circular shift (wrap around) | `--shift 4,0` |
| `-o, --output` | Output file (required) | `-o flipped.pxl` |
| `--sprite <name>` | Transform specific sprite (if multiple) | `--sprite player` |

### Transform Order

Transforms are applied in the order specified:

```bash
# Mirror first, then rotate
pxl transform input.pxl --mirror horizontal --rotate 90 -o output.pxl

# Rotate first, then mirror (different result!)
pxl transform input.pxl --rotate 90 --mirror horizontal -o output.pxl
```

### Output

- Always outputs valid `.pxl` source (not PNG)
- Preserves palette references
- Transforms the grid tokens directly
- Can be piped: `pxl transform --stdin ... | pxl render --stdin`

---

## Format Attribute: `transform`

Declarative transforms applied at render time. Source grid remains unchanged.

### On Sprite (via `source` reference)

Create a transformed sprite that references another:

```json
{"type": "sprite", "name": "arrow_right", "palette": "icons", "grid": ["..."]}
{"type": "sprite", "name": "arrow_left", "source": "arrow_right", "transform": ["mirror-h"]}
{"type": "sprite", "name": "arrow_down", "source": "arrow_right", "transform": ["rotate-90"]}
```

**Rules:**
- `source` references another sprite by name
- `transform` is an array of operations (applied in order)
- Cannot have both `grid` and `source` (mutually exclusive)
- `palette` is optional (inherits from source if omitted)

### On Variant

Variants already reference a sprite; add transforms:

```json
{"type": "sprite", "name": "enemy", "palette": "...", "grid": ["..."]}
{"type": "variant", "name": "enemy_flipped", "sprite": "enemy", "transform": ["mirror-h"]}
{"type": "variant", "name": "enemy_alt", "sprite": "enemy", "palette": "alt_colors", "transform": ["mirror-h"]}
```

### On Composition Layer

Transform individual layers within a composition:

```json
{
  "type": "composition",
  "name": "symmetric_scene",
  "size": [64, 32],
  "layers": [
    {"sprite": "tree", "position": [0, 0]},
    {"sprite": "tree", "position": [48, 0], "transform": ["mirror-h"]},
    {"sprite": "cloud", "position": [16, 0], "transform": ["mirror-v", "rotate:180"]}
  ]
}
```

### On Animation

Transform animation frame sequences:

```json
{"type": "animation", "name": "walk_right", "frames": ["walk_1", "walk_2", "walk_3"], "fps": 8}
{"type": "animation", "name": "walk_right_loop", "source": "walk_right", "transform": ["pingpong"]}
{"type": "animation", "name": "walk_left", "source": "walk_right", "transform": ["mirror-h"]}
```

**Animation transform behavior:**
- **Sprite transforms** (`mirror-h`, `rotate`, `outline`, etc.): Applied to each frame's sprite
- **Sequence transforms** (`pingpong`, `reverse`, `frame-offset`, `hold`): Applied to frame sequence

### Mixed Sprite + Sequence Transforms

Animations can combine both transform types. They're applied in two phases:

**Phase 1: Sprite transforms** (applied to each frame)
**Phase 2: Sequence transforms** (applied to frame order)

```json
{
  "type": "animation",
  "name": "walk_left_loop",
  "source": "walk_right",
  "transform": ["mirror-h", "outline", "pingpong", "hold:0,2"]
}
```

**Execution order:**
1. `mirror-h` → flip each frame horizontally
2. `outline` → add outline to each frame
3. `pingpong` → duplicate frames in reverse (1,2,3 → 1,2,3,2,1)
4. `hold:0,2` → hold first frame for 2 extra ticks

The transform array is processed in order, but sprite transforms are batched and applied first, then sequence transforms are batched and applied second. This ensures predictable results regardless of how you order them in the array.

**Explicit phase control** (optional, for edge cases):
```json
{
  "type": "animation",
  "name": "complex",
  "source": "base",
  "sprite_transform": ["mirror-h", "outline"],
  "sequence_transform": ["pingpong", "hold:0,2"]
}
```

Using separate `sprite_transform` and `sequence_transform` arrays gives explicit control when needed.

---

## Transform Operations

### Parameterized Syntax

Transforms can be specified as strings or objects. Both are valid in the format:

**String syntax** (simple or with colon params):
```json
"transform": ["mirror-h", "rotate:90", "tile:3x2", "pad:4"]
```

**Object syntax** (for complex params):
```json
"transform": [
  "mirror-h",
  {"op": "tile", "w": 3, "h": 2},
  {"op": "outline", "token": "{border}", "width": 2}
]
```

**CLI syntax**:
```bash
pxl transform input.pxl --mirror horizontal --rotate 90 --tile 3x2 --pad 4
```

### Geometric (Grid-Level)

| Operation | Aliases | Description | Params |
|-----------|---------|-------------|--------|
| `mirror-h` | `symmetry-h`, `flip-h` | Mirror horizontally (left↔right) | — |
| `mirror-v` | `symmetry-v`, `flip-v` | Mirror vertically (top↔bottom) | — |
| `rotate` | `rot` | Rotate clockwise | `degrees`: 90, 180, 270 |

**Examples:**
- `"mirror-h"` or `"symmetry-h"` — same operation
- `"rotate:90"` or `{"op": "rotate", "degrees": 90}`

### Expansion

| Operation | Description | Params |
|-----------|-------------|--------|
| `tile` | Tile sprite into grid | `w`, `h` (or `WxH` string) |
| `pad` | Add transparent padding | `size` (pixels) |
| `crop` | Extract sub-region | `x`, `y`, `w`, `h` |

**Examples:**
- `"tile:3x2"` — tile 3 wide, 2 tall
- `{"op": "pad", "size": 4}` — 4px padding all sides
- `{"op": "crop", "x": 0, "y": 0, "w": 8, "h": 8}` — extract 8x8 region

### Effects

| Operation | Description | Params |
|-----------|-------------|--------|
| `outline` | Add outline around opaque pixels | `token` (optional), `width` (default 1) |
| `shift` | Circular shift (wrap around) | `x`, `y` (pixels) |
| `shadow` | Add drop shadow | `x`, `y`, `token` |

**Examples:**
- `"outline"` — 1px black outline
- `{"op": "outline", "token": "{border}", "width": 2}` — 2px outline using palette token
- `"shift:4,0"` — shift 4px right with wrap

### Animation Transforms

| Operation | Description | Params |
|-----------|-------------|--------|
| `pingpong` | Duplicate frames in reverse (1,2,3 → 1,2,3,2,1) | `exclude_ends` (bool, default false) |
| `reverse` | Reverse frame order | — |
| `frame-offset` | Rotate frame order | `offset` (int) |
| `hold` | Duplicate specific frames | `frame`, `count` |

**Examples:**
- `"pingpong"` — creates smooth loop
- `{"op": "pingpong", "exclude_ends": true}` — 1,2,3 → 1,2,3,2 (no duplicate endpoints)
- `"reverse"` — play backwards
- `{"op": "frame-offset", "offset": 2}` — start from frame 2
- `{"op": "hold", "frame": 0, "count": 3}` — hold first frame for 3 ticks

---

## Implementation Plan

### Phase 17.1: Core Transform Module

Create `src/transforms.rs`:

```rust
/// A single transform operation with optional parameters
#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    // Geometric
    MirrorH,
    MirrorV,
    Rotate { degrees: u16 },  // 90, 180, 270

    // Expansion
    Tile { w: u32, h: u32 },
    Pad { size: u32 },
    Crop { x: u32, y: u32, w: u32, h: u32 },

    // Effects
    Outline { token: Option<String>, width: u32 },
    Shift { x: i32, y: i32 },
    Shadow { x: i32, y: i32, token: Option<String> },

    // Animation (only valid for Animation type)
    Pingpong { exclude_ends: bool },
    Reverse,
    FrameOffset { offset: i32 },
    Hold { frame: usize, count: usize },
}

/// Parse transform from string syntax: "mirror-h", "rotate:90", "tile:3x2"
pub fn parse_transform_str(s: &str) -> Result<Transform, TransformError>;

/// Parse transform from JSON value (string or object)
pub fn parse_transform_value(value: &serde_json::Value) -> Result<Transform, TransformError>;

/// Transform a grid of token rows (for sprites)
pub fn transform_grid(grid: &[String], transforms: &[Transform]) -> Result<Vec<String>, TransformError>;

/// Transform animation frames
pub fn transform_frames(frames: &[String], transforms: &[Transform]) -> Result<Vec<String>, TransformError>;
```

**Alias resolution** (in `parse_transform_str`):
- `symmetry-h` → `MirrorH`
- `symmetry-v` → `MirrorV`
- `flip-h` → `MirrorH`
- `flip-v` → `MirrorV`
- `rot` → `Rotate`

**Key functions:**
- `mirror_horizontal(grid)` - Reverse token order in each row
- `mirror_vertical(grid)` - Reverse row order
- `rotate_grid(grid, degrees)` - Transpose + mirror combinations
- `tile_grid(grid, w, h)` - Repeat grid
- `pad_grid(grid, size, token)` - Add border
- `outline_grid(grid, token, width)` - Add outline around opaque
- `pingpong_frames(frames, exclude_ends)` - Duplicate frames in reverse
- `reverse_frames(frames)` - Reverse frame order

### Phase 17.2: CLI Command

Add to `cli.rs`:

```rust
/// Transform sprites (mirror, rotate, tile, etc.)
Transform {
    /// Input file
    input: PathBuf,

    /// Mirror axis (horizontal, vertical, both)
    #[arg(long)]
    mirror: Option<String>,

    /// Rotate degrees (90, 180, 270)
    #[arg(long)]
    rotate: Option<u16>,

    /// Tile pattern (e.g., "2x2", "3x1")
    #[arg(long)]
    tile: Option<String>,

    /// Padding pixels
    #[arg(long)]
    pad: Option<u32>,

    /// Add outline
    #[arg(long)]
    outline: bool,

    /// Crop region (X,Y,W,H)
    #[arg(long)]
    crop: Option<String>,

    /// Shift pixels (X,Y)
    #[arg(long)]
    shift: Option<String>,

    /// Target sprite name
    #[arg(long)]
    sprite: Option<String>,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,

    /// Read from stdin
    #[arg(long)]
    stdin: bool,
}
```

### Phase 17.3: Format Support

Update `models.rs`:

```rust
/// Transform specification - can be string or object in JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransformSpec {
    String(String),                        // "mirror-h", "rotate:90"
    Object {
        op: String,
        #[serde(flatten)]
        params: HashMap<String, serde_json::Value>,
    },
}

pub struct Sprite {
    pub name: String,
    pub size: Option<[u32; 2]>,
    pub palette: Option<PaletteRef>,
    pub grid: Option<Vec<String>>,         // None if using source
    pub source: Option<String>,            // Reference to another sprite
    pub transform: Option<Vec<TransformSpec>>,
}

pub struct Variant {
    pub name: String,
    #[serde(alias = "base")]             // Backwards compat
    pub source: String,                   // Renamed from 'base', alias kept
    pub palette: Option<PaletteRef>,
    pub transform: Option<Vec<TransformSpec>>,  // NEW
}

pub struct CompositionLayer {
    pub sprite: String,
    pub position: [i32; 2],
    pub transform: Option<Vec<TransformSpec>>,  // NEW
}

pub struct Animation {
    pub name: String,
    pub frames: Option<Vec<String>>,       // None if using source
    pub source: Option<String>,            // Reference to another animation
    pub fps: Option<u32>,
    pub transform: Option<Vec<TransformSpec>>,  // NEW
}

/// User-defined transform
pub struct TransformDef {
    pub name: String,
    pub params: Option<Vec<String>>,              // Parameter names
    pub ops: Option<Vec<TransformSpec>>,          // Simple sequence
    pub compose: Option<Vec<TransformSpec>>,      // Parallel composition
    pub cycle: Option<Vec<Vec<String>>>,          // Per-frame cycling
    pub frames: Option<u32>,                      // For keyframe generation
    pub keyframes: Option<KeyframeSpec>,          // Keyframe data
    pub easing: Option<String>,                   // Default easing
}

/// Keyframe specification - array or object form
#[serde(untagged)]
pub enum KeyframeSpec {
    Array(Vec<Keyframe>),                         // [{frame: 0, shift-y: 0}, ...]
    Properties(HashMap<String, PropertyKeyframes>), // {shift-y: {expr: "..."}}
}

pub struct Keyframe {
    pub frame: u32,
    #[serde(flatten)]
    pub values: HashMap<String, f64>,             // property -> value
}

pub struct PropertyKeyframes {
    pub expr: Option<String>,                     // Math expression
    pub keyframes: Option<Vec<(u32, f64)>>,       // (frame, value) pairs
    pub easing: Option<String>,                   // Per-property easing
}
```

Update `registry.rs` to resolve transforms during sprite/animation resolution.

### Phase 17.4: Render Integration

Update `renderer.rs`:
- Apply transforms after resolving sprite but before rasterizing
- Transform the token grid, not the pixels (preserves quality)

---

## Examples

### CLI: Create mirrored arrow set

```bash
# Start with right-facing arrow
cat arrow_right.pxl
# {"type": "sprite", "name": "arrow", "palette": {...}, "grid": [...]}

# Generate all directions
pxl transform arrow_right.pxl --mirror horizontal -o arrow_left.pxl
pxl transform arrow_right.pxl --rotate 90 -o arrow_down.pxl
pxl transform arrow_right.pxl --rotate 270 -o arrow_up.pxl
```

### Format: Symmetric character

```json
{"type": "palette", "name": "char", "colors": {"{_}": "#00000000", "{x}": "#FFD700"}}
{"type": "sprite", "name": "letter_A_left", "palette": "char", "grid": [
  "{_}{_}{x}",
  "{_}{x}{x}",
  "{x}{_}{x}",
  "{x}{x}{x}",
  "{x}{_}{x}"
]}
{"type": "sprite", "name": "letter_A", "source": "letter_A_left", "transform": ["symmetry-h"]}
```

*Note: `symmetry-h` would append mirrored version to create full symmetric sprite*

### Format: Tiled background

```json
{"type": "sprite", "name": "grass_tile", "palette": "nature", "grid": ["..."]}
{"type": "sprite", "name": "grass_bg", "source": "grass_tile", "transform": ["tile-4x4"]}
```

### Format: Composition with transforms

```json
{
  "type": "composition",
  "name": "forest",
  "size": [128, 64],
  "layers": [
    {"sprite": "tree", "position": [0, 0]},
    {"sprite": "tree", "position": [32, 0], "transform": ["mirror-h"]},
    {"sprite": "tree", "position": [64, 0]},
    {"sprite": "tree", "position": [96, 0], "transform": ["mirror-h"]}
  ]
}
```

---

## Validation & Warnings

### Result Validation

Warn (don't error) when transforms produce unexpected results:

| Condition | Warning |
|-----------|---------|
| Crop outside bounds | `"crop region extends beyond sprite bounds"` |
| Empty result | `"transform resulted in 0x0 sprite"` |
| Missing source | `"source sprite 'foo' not found"` |

### Expansion Warnings

When transforms would create large outputs, warn but allow override:

```
⚠ Warning: Transform chain will expand sprite from 8x8 to 6400x6400 pixels
  - tile:10x10 (8x8 → 80x80)
  - tile:10x10 (80x80 → 800x800)
  - tile:8x8 (800x800 → 6400x6400)

  Use --allow-large to proceed, or reduce expansion.
```

**Thresholds:**
- Warn if result > 1024x1024 pixels
- Warn if result > 10x source size
- Warn if frame count > 100 (for animations)

**CLI override:**
```bash
pxl transform input.pxl --tile 10x10 --tile 10x10 --allow-large -o huge.pxl
```

**Format override** (per-sprite):
```json
{
  "type": "sprite",
  "name": "massive_bg",
  "source": "tile",
  "transform": ["tile:50x50"],
  "allow_large": true
}
```

---

## Resolved Design Decisions

1. **Symmetry operations**: `symmetry-h` and `symmetry-v` are aliases for `mirror-h`/`mirror-v`

2. **Parameterized transforms**: Support both syntaxes:
   - String with colon: `"tile:3x2"`, `"rotate:90"`, `"pad:4"`
   - Object form: `{"op": "tile", "w": 3, "h": 2}`

3. **Animation transforms**: Yes — `pingpong`, `reverse`, `frame-offset`, `hold`

4. **Mixed transforms on animations**: Yes — sprite transforms apply to each frame, sequence transforms apply to frame order. Can use single `transform` array (auto-sorted) or explicit `sprite_transform`/`sequence_transform` arrays.

5. **Expansion warnings**: Warn on large results but allow user override via `--allow-large` (CLI) or `allow_large: true` (format)

6. **Unified `source` attribute**: Use `source` as the canonical name for referencing base sprites/animations. Alias `base` (Variant) for backwards compatibility.

---

## User-Defined Transforms

Users can define custom, reusable, parameterized transforms.

### Named Transform Sequences

Simple reuse of transform chains:

```json
{
  "type": "transform",
  "name": "flip-glow",
  "ops": ["mirror-h", "outline"]
}
```

Usage:
```json
{"type": "sprite", "name": "enemy_left", "source": "enemy_right", "transform": ["flip-glow"]}
```

### Parameterized Transforms

Transforms with configurable values:

```json
{
  "type": "transform",
  "name": "padded-outline",
  "params": ["padding", "outline_width"],
  "ops": [
    {"op": "pad", "size": "${padding}"},
    {"op": "outline", "width": "${outline_width}"}
  ]
}
```

Usage:
```json
{"transform": [{"op": "padded-outline", "padding": 2, "outline_width": 1}]}
```

### Transform Composition

Combine multiple effects in parallel (computed together per-frame):

```json
{
  "type": "transform",
  "name": "chaos-shake",
  "compose": [
    {"op": "shake", "axis": "x", "amount": 2, "rate": 1.0},
    {"op": "shake", "axis": "y", "amount": 1, "rate": 1.5}
  ]
}
```

Both effects calculated and combined each frame, rather than applied sequentially.

### Keyframe Animation Generation

Create animations from static sprites by defining key moments:

```json
{
  "type": "transform",
  "name": "hop",
  "frames": 8,
  "keyframes": [
    {"frame": 0, "shift-y": 0},
    {"frame": 4, "shift-y": -4},
    {"frame": 8, "shift-y": 0}
  ],
  "easing": "ease-out"
}
```

**How it works:**
- Define values at specific frames (keyframes)
- System interpolates frames in between
- Easing controls the interpolation curve

**Apply to static sprite → generates animation:**
```json
{"type": "animation", "name": "coin_hop", "source": "coin", "transform": ["hop"]}
```

### Easing Functions

Control how values interpolate between keyframes:

| Easing | Description | Use Case |
|--------|-------------|----------|
| `linear` | Constant speed | Mechanical movement |
| `ease-in` | Slow start, fast end | Falling, acceleration |
| `ease-out` | Fast start, slow end | Throwing upward, deceleration |
| `ease-in-out` | Slow start and end | Smooth natural motion |
| `bounce` | Overshoots and settles | Landing, UI pop-in |
| `elastic` | Spring-like oscillation | Wobbly, cartoonish |

```
Linear:       ●───●───●───●───●  (constant)
Ease-out:     ●─────●───●──●─●  (decelerating)
Ease-in:      ●─●──●───●─────●  (accelerating)
Ease-in-out:  ●──●────●────●──●  (smooth S-curve)
Bounce:       ●───●─●─●●●        (overshoot + settle)
```

### Mathematical Expressions

For advanced animation, use expressions with math functions:

```json
{
  "type": "transform",
  "name": "accelerating-fall",
  "params": ["gravity", "max_speed"],
  "frames": 12,
  "keyframes": {
    "shift-y": {
      "expr": "min(frame * frame * ${gravity}, ${max_speed})"
    }
  }
}
```

**Available variables:**
- `frame` - Current frame index (0-based)
- `t` - Normalized time (0.0 to 1.0)
- `total_frames` - Total frame count
- Any user-defined `params`

**Available functions:**
- `sin(x)`, `cos(x)`, `tan(x)` - Trigonometry
- `pow(base, exp)` - Exponentiation
- `sqrt(x)` - Square root
- `min(a, b)`, `max(a, b)` - Clamping
- `abs(x)` - Absolute value
- `floor(x)`, `ceil(x)`, `round(x)` - Rounding

### Complex Example: Spiral

Multi-param transform with exponential decay:

```json
{
  "type": "transform",
  "name": "spiral-in",
  "params": ["start_radius", "decay", "spin_rate"],
  "frames": 16,
  "keyframes": {
    "shift-x": {"expr": "${start_radius} * pow(${decay}, frame) * cos(frame * ${spin_rate})"},
    "shift-y": {"expr": "${start_radius} * pow(${decay}, frame) * sin(frame * ${spin_rate})"}
  }
}
```

Usage:
```json
{
  "type": "animation",
  "name": "coin_collect",
  "source": "coin",
  "transform": [
    {"op": "spiral-in", "start_radius": 8, "decay": 0.85, "spin_rate": 0.5}
  ]
}
```

Creates a 16-frame animation of the coin spiraling inward.

### Composed Keyframe Animations

Combine multiple keyframe effects:

```json
{
  "type": "transform",
  "name": "dramatic-entrance",
  "frames": 12,
  "compose": [
    {
      "keyframes": [
        {"frame": 0, "shift-y": -16},
        {"frame": 8, "shift-y": 0}
      ],
      "easing": "bounce"
    },
    {
      "keyframes": [
        {"frame": 0, "scale": 0.5},
        {"frame": 12, "scale": 1.0}
      ],
      "easing": "ease-out"
    }
  ]
}
```

Object drops in from above (with bounce) while scaling up.

### Per-Frame Transform Cycling

Apply different transforms to each frame in a cycle:

```json
{
  "type": "transform",
  "name": "shiver",
  "cycle": [
    ["shift:1,0"],
    ["shift:-1,0"],
    ["shift:0,1"],
    ["shift:0,-1"]
  ]
}
```

Frame 0 → shift right, frame 1 → shift left, frame 2 → shift down, frame 3 → shift up, frame 4 → shift right (repeats).

---

## Future Extensions

- **Symmetry generation**: `extend-h`, `extend-v` (append mirrored copy to create full sprite from half)
- **Color transforms**: `invert`, `grayscale`, `hue-shift`
- **Blend modes**: For composition layers
- **Conditional transforms**: Apply based on variant state

---

## Task Dependency Diagram

```
                           SPRITE TRANSFORMS TASK FLOW
═══════════════════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Phase 0 Complete                                  │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 1 (Foundation)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            TRF-1                                    │    │
│  │               Transform Module Foundation                           │    │
│  │               (src/transforms.rs)                                   │    │
│  │               - Transform enum                                      │    │
│  │               - parse_transform_str/value                           │    │
│  │               - lib.rs export                                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 2 (Transform Types - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │    TRF-2      │  │    TRF-3      │  │    TRF-4      │  │    TRF-5      │ │
│  │  Geometric    │  │  Expansion    │  │   Effect      │  │  Animation    │ │
│  │  Transforms   │  │  Transforms   │  │  Transforms   │  │  Transforms   │ │
│  │  - mirror     │  │  - tile       │  │  - outline    │  │  - pingpong   │ │
│  │  - rotate     │  │  - pad        │  │  - shift      │  │  - reverse    │ │
│  │               │  │  - crop       │  │  - shadow     │  │  - hold       │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘ │
│                                                                     │       │
│  ┌──────────────────────────────────────────────────────────────────┼─────┐ │
│  │                            TRF-7                                 │     │ │
│  │               Format Support Models                              │     │ │
│  │               - TransformSpec                                    │     │ │
│  │               - Update Sprite/Variant/Animation                  │     │ │
│  └──────────────────────────────────────────────────────────────────┼─────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                                         │
            ▼                                                         ▼
WAVE 3 (Integration)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            TRF-6                                    │    │
│  │               Transform CLI Command                                 │    │
│  │               (pxl transform with all flags)                        │    │
│  │               Needs: TRF-2, TRF-3, TRF-4, TRF-5                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            TRF-8                                    │    │
│  │               Format Support Registry                               │    │
│  │               (resolve transforms in registry)                      │    │
│  │               Needs: TRF-7                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                     │
            │                                     ▼
            │         ┌─────────────────────────────────────────────────────┐
            │         │                    TRF-9                            │
            │         │           Render Integration                        │
            │         │           Needs: TRF-7, TRF-8                        │
            │         └─────────────────────────────────────────────────────┘
            │                                     │
            │         ┌─────────────────────────────────────────────────────┐
            │         │                    TRF-10                           │
            │         │           User-Defined Transforms                   │
            │         │           - TransformDef, keyframes                 │
            │         │           - Easing, expressions                     │
            │         │           Needs: TRF-8                              │
            │         └─────────────────────────────────────────────────────┘
            │                                     │
            ▼                                     ▼
WAVE 4 (Testing)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            TRF-11                                   │    │
│  │                    Transform Test Suite                             │    │
│  │                    (unit + integration tests)                       │    │
│  │                    Needs: TRF-6, TRF-9, TRF-10                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 5 (Documentation)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            TRF-12                                   │    │
│  │                    Transform Documentation                          │    │
│  │                    - prime output                                   │    │
│  │                    - format spec                                    │    │
│  │                    - demo.sh examples                               │    │
│  │                    Needs: TRF-11                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Wave 1: TRF-1                               (1 task)                       │
│  Wave 2: TRF-2 + TRF-3 + TRF-4 + TRF-5 + TRF-7  (5 tasks in parallel)       │
│  Wave 3: TRF-6 (after TRF-2-5) + TRF-8 (after TRF-7)  (2 parallel tracks)   │
│          TRF-9 (after TRF-7,8) + TRF-10 (after TRF-8)                       │
│  Wave 4: TRF-11                              (1 task, needs TRF-6,9,10)     │
│  Wave 5: TRF-12                              (1 task)                       │
└─────────────────────────────────────────────────────────────────────────────┘

CRITICAL PATH: TRF-1 → TRF-2 → TRF-6 → TRF-11 → TRF-12
          OR:  TRF-1 → TRF-7 → TRF-8 → TRF-10 → TRF-11 → TRF-12

BEADS CREATION ORDER:
  1. TRF-1 (no deps)
  2. TRF-2, TRF-3, TRF-4, TRF-5, TRF-7 (dep: TRF-1)
  3. TRF-6 (dep: TRF-2,3,4,5), TRF-8 (dep: TRF-7)
  4. TRF-9 (dep: TRF-7,8), TRF-10 (dep: TRF-8)
  5. TRF-11 (dep: TRF-6,9,10)
  6. TRF-12 (dep: TRF-11)
```

---

## Tasks

### Task TRF-1: Transform Module Foundation

**Wave:** 1

Create the core transform infrastructure.

**Deliverables:**
- New file `src/transforms.rs`:
  ```rust
  /// A single transform operation with optional parameters
  #[derive(Debug, Clone, PartialEq)]
  pub enum Transform {
      // Geometric
      MirrorH,
      MirrorV,
      Rotate { degrees: u16 },

      // Expansion
      Tile { w: u32, h: u32 },
      Pad { size: u32 },
      Crop { x: u32, y: u32, w: u32, h: u32 },

      // Effects
      Outline { token: Option<String>, width: u32 },
      Shift { x: i32, y: i32 },
      Shadow { x: i32, y: i32, token: Option<String> },

      // Animation
      Pingpong { exclude_ends: bool },
      Reverse,
      FrameOffset { offset: i32 },
      Hold { frame: usize, count: usize },
  }

  /// Parse transform from string syntax
  pub fn parse_transform_str(s: &str) -> Result<Transform, TransformError>;

  /// Parse transform from JSON value
  pub fn parse_transform_value(value: &serde_json::Value) -> Result<Transform, TransformError>;
  ```
- Update `src/lib.rs` to add `pub mod transforms;`

**Verification:**
```bash
cargo build
cargo test transforms
```

**Dependencies:** Phase 0 complete

---

### Task TRF-2: Geometric Transforms

**Wave:** 2 (parallel with TRF-3, TRF-4, TRF-5, TRF-7)

Implement mirror and rotate operations.

**Deliverables:**
- In `src/transforms.rs`:
  ```rust
  /// Mirror grid horizontally (reverse token order in each row)
  pub fn mirror_horizontal(grid: &[String]) -> Vec<String>

  /// Mirror grid vertically (reverse row order)
  pub fn mirror_vertical(grid: &[String]) -> Vec<String>

  /// Rotate grid by 90, 180, or 270 degrees clockwise
  pub fn rotate_grid(grid: &[String], degrees: u16) -> Result<Vec<String>, TransformError>
  ```

**Verification:**
```bash
cargo test mirror
cargo test rotate
```

**Dependencies:** Task TRF-1

---

### Task TRF-3: Expansion Transforms

**Wave:** 2 (parallel with TRF-2, TRF-4, TRF-5, TRF-7)

Implement tile, pad, and crop operations.

**Deliverables:**
- In `src/transforms.rs`:
  ```rust
  /// Tile grid into WxH repetitions
  pub fn tile_grid(grid: &[String], w: u32, h: u32) -> Vec<String>

  /// Add transparent padding around grid
  pub fn pad_grid(grid: &[String], size: u32, token: &str) -> Vec<String>

  /// Extract sub-region from grid
  pub fn crop_grid(grid: &[String], x: u32, y: u32, w: u32, h: u32) -> Result<Vec<String>, TransformError>
  ```

**Verification:**
```bash
cargo test tile
cargo test pad
cargo test crop
```

**Dependencies:** Task TRF-1

---

### Task TRF-4: Effect Transforms

**Wave:** 2 (parallel with TRF-2, TRF-3, TRF-5, TRF-7)

Implement outline, shift, and shadow operations.

**Deliverables:**
- In `src/transforms.rs`:
  ```rust
  /// Add outline around opaque pixels
  pub fn outline_grid(grid: &[String], token: Option<&str>, width: u32) -> Vec<String>

  /// Circular shift (wrap around)
  pub fn shift_grid(grid: &[String], x: i32, y: i32) -> Vec<String>

  /// Add drop shadow
  pub fn shadow_grid(grid: &[String], x: i32, y: i32, token: Option<&str>) -> Vec<String>
  ```

**Verification:**
```bash
cargo test outline
cargo test shift
cargo test shadow
```

**Dependencies:** Task TRF-1

---

### Task TRF-5: Animation Transforms

**Wave:** 2 (parallel with TRF-2, TRF-3, TRF-4, TRF-7)

Implement frame sequence operations.

**Deliverables:**
- In `src/transforms.rs`:
  ```rust
  /// Duplicate frames in reverse (1,2,3 → 1,2,3,2,1)
  pub fn pingpong_frames(frames: &[String], exclude_ends: bool) -> Vec<String>

  /// Reverse frame order
  pub fn reverse_frames(frames: &[String]) -> Vec<String>

  /// Rotate frame order by offset
  pub fn frame_offset(frames: &[String], offset: i32) -> Vec<String>

  /// Hold specific frame for extra ticks
  pub fn hold_frame(frames: &[String], frame: usize, count: usize) -> Vec<String>
  ```

**Verification:**
```bash
cargo test pingpong
cargo test reverse_frames
cargo test frame_offset
cargo test hold
```

**Dependencies:** Task TRF-1

---

### Task TRF-6: Transform CLI Command

**Wave:** 3 (after TRF-2, TRF-3, TRF-4, TRF-5)

Add `pxl transform` command with all flags.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Transform sprites (mirror, rotate, tile, etc.)
  Transform {
      input: PathBuf,
      #[arg(long)] mirror: Option<String>,
      #[arg(long)] rotate: Option<u16>,
      #[arg(long)] tile: Option<String>,
      #[arg(long)] pad: Option<u32>,
      #[arg(long)] outline: bool,
      #[arg(long)] crop: Option<String>,
      #[arg(long)] shift: Option<String>,
      #[arg(long)] sprite: Option<String>,
      #[arg(short, long)] output: PathBuf,
      #[arg(long)] stdin: bool,
      #[arg(long)] allow_large: bool,
  }
  ```

**Verification:**
```bash
./target/release/pxl transform examples/arrow.pxl --mirror horizontal -o arrow_left.pxl
./target/release/pxl transform examples/tile.pxl --tile 3x3 -o tiled.pxl
./target/release/pxl transform examples/sprite.pxl --rotate 90 --outline -o output.pxl
```

**Dependencies:** Tasks TRF-2, TRF-3, TRF-4, TRF-5

---

### Task TRF-7: Format Support Models

**Wave:** 2 (parallel with TRF-2, TRF-3, TRF-4, TRF-5)

Add transform support to format types.

**Deliverables:**
- Update `src/models.rs`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(untagged)]
  pub enum TransformSpec {
      String(String),
      Object { op: String, #[serde(flatten)] params: HashMap<String, Value> },
  }

  // Add to Sprite, Variant, CompositionLayer, Animation:
  pub transform: Option<Vec<TransformSpec>>,

  // Add source field to Sprite:
  pub source: Option<String>,
  ```

**Verification:**
```bash
cargo build
cargo test models
```

**Dependencies:** Task TRF-1

---

### Task TRF-8: Format Support Registry

**Wave:** 3 (after TRF-7)

Resolve transforms during sprite/animation resolution.

**Deliverables:**
- Update `src/registry.rs`:
  - Resolve `source` references for sprites
  - Apply transforms when resolving sprites/animations
  - Handle transform chains

**Verification:**
```bash
cargo test registry
./target/release/pxl render examples/transformed.pxl
```

**Dependencies:** Task TRF-7

---

### Task TRF-9: Render Integration

**Wave:** 3 (after TRF-7, TRF-8)

Apply transforms during rendering.

**Deliverables:**
- Update `src/renderer.rs`:
  - Apply transforms after resolving sprite but before rasterizing
  - Transform the token grid, not the pixels

**Verification:**
```bash
./target/release/pxl render examples/with_transforms.pxl -o output.png
```

**Dependencies:** Tasks TRF-7, TRF-8

---

### Task TRF-10: User-Defined Transforms

**Wave:** 3 (after TRF-8)

Support custom reusable transforms with keyframes.

**Deliverables:**
- Add `TransformDef` type to `src/models.rs`
- Implement keyframe interpolation
- Implement easing functions (linear, ease-in, ease-out, etc.)
- Implement expression evaluation for advanced animations

**Verification:**
```bash
cargo test keyframe
cargo test easing
./target/release/pxl render examples/custom_transform.pxl -o output.gif
```

**Dependencies:** Task TRF-8

---

### Task TRF-11: Transform Test Suite

**Wave:** 4 (after TRF-6, TRF-9, TRF-10)

Comprehensive tests for all transform functionality.

**Deliverables:**
- `tests/transform_tests.rs`:
  - Unit tests for each transform operation
  - Edge cases (empty grids, single-pixel, etc.)
  - Transform composition tests
- `tests/cli_integration.rs` additions:
  - CLI transform command tests
  - Round-trip tests
- Test fixtures in `tests/fixtures/valid/`

**Verification:**
```bash
cargo test transforms
cargo test --test cli_integration transform
```

**Dependencies:** Tasks TRF-6, TRF-9, TRF-10

---

### Task TRF-12: Transform Documentation

**Wave:** 5 (after TRF-11)

Update all documentation for transform feature.

**Deliverables:**
- Update `src/prime.rs` with transform commands and examples
- Update `docs/spec/format.md` with transform syntax
- Update `demo.sh` with transform examples

**Verification:**
```bash
./target/release/pxl prime | grep transform
grep "transform" docs/spec/format.md
./demo.sh  # Should run without errors
```

**Dependencies:** Task TRF-11

---

## Verification Summary

```bash
# 1. All existing tests pass
cargo test

# 2. Transform module tests pass
cargo test transforms

# 3. CLI command works
./target/release/pxl transform examples/arrow.pxl --mirror horizontal -o left.pxl
./target/release/pxl transform examples/tile.pxl --tile 3x3 -o tiled.pxl
./target/release/pxl transform examples/sprite.pxl --rotate 90 --pad 2 --outline -o output.pxl

# 4. Format transforms work
./target/release/pxl render examples/with_transforms.pxl -o output.png

# 5. Chain transforms work
./target/release/pxl transform input.pxl --mirror h --rotate 90 --tile 2x2 -o output.pxl

# 6. Documentation updated
./target/release/pxl prime | grep transform
```

---

## Success Criteria

1. All transform operations work as documented
2. CLI `pxl transform` supports all flags and chains transforms correctly
3. Format `transform` attribute works on sprites, variants, compositions, and animations
4. User-defined transforms support keyframes and easing
5. Large expansion warnings work with `--allow-large` override
6. All tests pass
7. Documentation reflects new capabilities
