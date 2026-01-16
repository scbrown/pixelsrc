# Composition

A composition layers multiple sprites onto a canvas using a character-based map. This is useful for building scenes, tile maps, and complex images from smaller sprite components.

## Basic Syntax

```json
{
  "type": "composition",
  "name": "string (required)",
  "base": "string (optional)",
  "size": [width, height],
  "cell_size": [width, height],
  "sprites": { "char": "sprite_name" | null, ... },
  "layers": [ { "name": "...", "map": [...] }, ... ]
}
```

## Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `type` | Yes | - | Must be `"composition"` |
| `name` | Yes | - | Unique identifier |
| `base` | No | - | Base sprite to render first (background) |
| `size` | No | Inferred | Canvas size `[width, height]` in pixels |
| `cell_size` | No | `[1, 1]` | Size of each map character in pixels |
| `sprites` | Yes | - | Map of single characters to sprite names |
| `layers` | Yes | - | Array of layers, rendered bottom-to-top |

## Layer Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | No | Layer identifier (for debugging) |
| `fill` | No | Sprite name to fill entire layer |
| `map` | No | Array of strings - character map for sprite placement |
| `blend` | No | Blend mode (default: `"normal"`) |
| `opacity` | No | Layer opacity 0.0-1.0 (default: 1.0) |

## Simple Example

```json
{"type": "composition", "name": "scene", "size": [32, 32],
  "sprites": {".": null, "H": "hero", "T": "tree"},
  "layers": [
    {"map": [
      "........",
      ".T....T.",
      "........",
      "...H....",
      "........"
    ]}
  ]}
```

The `.` character maps to `null` (transparent/no sprite), while `H` and `T` place the hero and tree sprites.

## Cell Size (Tiling)

The `cell_size` field enables tiling - composing large images from smaller sprites:

| Cell Size | Meaning |
|-----------|---------|
| `[1, 1]` (default) | Each map character = 1 pixel position |
| `[8, 8]` | Each map character = 8×8 pixel region |
| `[16, 16]` | Each map character = 16×16 tile |
| `[4, 8]` | Custom cells for text/UI composition |

Sprites are placed at `(col * cell_size[0], row * cell_size[1])`.

### Pixel Overlay Example

```json
{"type": "composition", "name": "icon", "size": [8, 8], "cell_size": [1, 1],
  "sprites": {".": null, "x": "pixel"},
  "layers": [{"map": [
    "..xx..",
    ".xxxx.",
    "xxxxxx",
    "xxxxxx",
    ".xxxx.",
    "..xx.."
  ]}]}
```

### Tiled Map Example

```json
{"type": "composition", "name": "level", "size": [128, 96], "cell_size": [16, 16],
  "sprites": {
    "G": "grass_tile",
    "W": "water_tile",
    "T": "tree_tile",
    ".": null
  },
  "layers": [{"map": [
    "GGGGGGGG",
    "GGTGGGTG",
    "GGGGGGGG",
    "WWWWWWWW",
    "GGGGGGGG",
    "GGGGGGGG"
  ]}]}
```

## Multiple Layers

Layers are rendered bottom-to-top. The first layer is the background:

```json
{"type": "composition", "name": "scene", "size": [64, 64], "cell_size": [16, 16],
  "sprites": {"G": "grass", "H": "hero", "T": "tree", ".": null},
  "layers": [
    {"name": "ground", "map": [
      "GGGG",
      "GGGG",
      "GGGG",
      "GGGG"
    ]},
    {"name": "objects", "map": [
      "T...",
      "....",
      ".H..",
      "...T"
    ]}
  ]}
```

## Base Sprite

Use `base` to set a full-canvas background:

```json
{"type": "composition", "name": "hero_scene", "base": "sky_background",
  "sprites": {".": null, "H": "hero"},
  "layers": [
    {"name": "characters", "map": ["..H.."]}
  ]}
```

The base sprite renders first, covering the entire canvas.

## Fill Layers

Fill an entire layer with a single sprite:

```json
{"type": "composition", "name": "ocean", "size": [128, 128], "cell_size": [16, 16],
  "sprites": {"B": "boat"},
  "layers": [
    {"name": "water", "fill": "water_tile"},
    {"name": "objects", "map": [
      "........",
      "...B....",
      "........"
    ]}
  ]}
```

## Blend Modes

Layer blending for visual effects:

```json
{
  "type": "composition",
  "name": "scene",
  "layers": [
    {"sprite": "background", "position": [0, 0]},
    {"sprite": "shadow", "position": [10, 20], "blend": "multiply", "opacity": 0.5},
    {"sprite": "glow", "position": [5, 5], "blend": "add"},
    {"sprite": "player", "position": [16, 16]}
  ]
}
```

### Available Blend Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `normal` | Standard alpha compositing | Default |
| `multiply` | Darkens underlying colors | Shadows, color tinting |
| `screen` | Lightens underlying colors | Glows, highlights |
| `overlay` | Combines multiply/screen | Contrast enhancement |
| `add` | Additive blending | Glow effects, particles |
| `subtract` | Subtractive blending | Special effects |
| `difference` | Color difference | Masks, effects |
| `darken` | Keeps darker color | Shadows |
| `lighten` | Keeps lighter color | Highlights |

## Size Inference

Canvas size is determined (in priority order):

1. Explicit `size` field
2. `base` sprite dimensions
3. Inferred from `layers` and `cell_size`

## Size Mismatch Handling

When a sprite is larger than the cell size:

- **Lenient mode:** Emits warning, anchors top-left, may overwrite adjacent cells
- **Strict mode:** Returns error

## Text Banner Example

```json
{"type": "composition", "name": "banner", "size": [20, 8], "cell_size": [4, 8],
  "sprites": {
    "{": "bracket_l",
    "p": "letter_p",
    "x": "letter_x",
    "l": "letter_l",
    "}": "bracket_r"
  },
  "layers": [{"map": ["{pxl}"]}]}
```

## Complete Example

```json
{"type": "palette", "name": "tiles", "colors": {
  "{_}": "#0000",
  "{grass}": "#228B22",
  "{water}": "#4169E1",
  "{sand}": "#F4A460"
}}

{"type": "sprite", "name": "grass_tile", "palette": "tiles", "grid": [
  "{grass}{grass}",
  "{grass}{grass}"
]}

{"type": "sprite", "name": "water_tile", "palette": "tiles", "grid": [
  "{water}{water}",
  "{water}{water}"
]}

{"type": "sprite", "name": "beach_tile", "palette": "tiles", "grid": [
  "{sand}{sand}",
  "{sand}{sand}"
]}

{"type": "composition", "name": "island", "size": [16, 16], "cell_size": [2, 2],
  "sprites": {"G": "grass_tile", "W": "water_tile", "S": "beach_tile"},
  "layers": [{"map": [
    "WWWWWWWW",
    "WWSSSSWW",
    "WSGGGSW",
    "WSGGGGSW",
    "WSGGGGSW",
    "WSGGGSW",
    "WWSSSSWW",
    "WWWWWWWW"
  ]}]}
```

This creates a 16x16 pixel island from 2x2 pixel tiles.
