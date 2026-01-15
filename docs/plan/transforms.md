# Sprite Transforms

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
