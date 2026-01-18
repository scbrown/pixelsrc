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
| `blend` | No | Blend mode (default: `"normal"`). Supports `var()` syntax. |
| `opacity` | No | Layer opacity 0.0-1.0 (default: 1.0). Supports `var()` syntax. |

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

Layer blending controls how colors combine when layers overlap. Each blend mode applies a mathematical formula to determine the final pixel color.

```json
{
  "type": "composition",
  "name": "scene",
  "layers": [
    {"fill": "background"},
    {"map": ["..S.."], "blend": "multiply", "opacity": 0.5},
    {"map": ["..G.."], "blend": "add"},
    {"map": ["..P.."]}
  ]
}
```

### Blend Mode Reference

| Mode | Formula | Description |
|------|---------|-------------|
| `normal` | `src over dst` | Standard alpha compositing. Source replaces destination based on alpha. |
| `multiply` | `src × dst` | Multiplies color values (0-1 range). Always darkens. Black stays black, white is transparent. |
| `screen` | `1 - (1-src)(1-dst)` | Inverse of multiply. Always lightens. White stays white, black is transparent. |
| `overlay` | Multiply if dst < 0.5, else screen | Enhances contrast. Dark areas darken, light areas lighten. |
| `add` | `src + dst` | Adds color values (clamped). Creates glowing, luminous effects. |
| `subtract` | `dst - src` | Subtracts source from destination. Can create inverted effects. |
| `difference` | `|src - dst|` | Absolute difference. Useful for detecting changes between layers. |
| `darken` | `min(src, dst)` | Keeps the darker color per channel. |
| `lighten` | `max(src, dst)` | Keeps the lighter color per channel. |

### When to Use Each Mode

**`multiply`** - Shadows, tinting, darkening
```json
{"name": "shadow", "map": ["..S.."], "blend": "multiply", "opacity": 0.3}
```
Use for drop shadows, color overlays that darken, or simulating light absorption.

**`screen`** - Glows, highlights, lightening
```json
{"name": "glow", "map": ["..G.."], "blend": "screen"}
```
Use for light sources, specular highlights, or fog/mist effects.

**`add`** - Particles, fire, magical effects
```json
{"name": "particles", "map": ["..P.."], "blend": "add"}
```
Use for fire, explosions, magic sparkles, or any additive light effect.

**`overlay`** - Contrast, texture application
```json
{"name": "texture", "fill": "noise_texture", "blend": "overlay", "opacity": 0.5}
```
Use for applying textures while preserving underlying tones.

## CSS Variables in Compositions

Composition layers support CSS variable references for `opacity` and `blend` properties. This enables dynamic theming where layer effects can be controlled via palette variables.

### Variable Syntax

Both `opacity` and `blend` accept `var()` syntax:

```json
{
  "type": "composition",
  "name": "themed_scene",
  "layers": [
    {"fill": "background"},
    {"map": ["..S.."], "blend": "var(--shadow-blend)", "opacity": "var(--shadow-opacity)"},
    {"map": ["..G.."], "blend": "var(--glow-mode, add)"}
  ]
}
```

Variables are resolved from the palette's variable registry at render time.

### Opacity with Variables

Opacity can be a literal number or a variable reference:

| Syntax | Description |
|--------|-------------|
| `0.5` | Literal opacity value |
| `"var(--opacity)"` | Variable reference |
| `"var(--opacity, 0.5)"` | Variable with fallback |

```json
{"name": "shadow", "map": ["..S.."], "opacity": "var(--shadow-alpha, 0.3)"}
```

Values are clamped to 0.0-1.0 after resolution.

### Blend Mode with Variables

Blend mode can be a literal string or a variable reference:

| Syntax | Description |
|--------|-------------|
| `"multiply"` | Literal blend mode |
| `"var(--blend)"` | Variable reference |
| `"var(--blend, normal)"` | Variable with fallback |

```json
{"name": "effect", "fill": "overlay_sprite", "blend": "var(--effect-blend, screen)"}
```

If the resolved value is not a valid blend mode, `normal` is used with a warning.

### Complete Variable Example

```jsonl
{"type": "palette", "name": "effects", "colors": {
  "--shadow-opacity": "0.4",
  "--shadow-blend": "multiply",
  "--glow-opacity": "0.8",
  "--glow-blend": "add",
  "{_}": "transparent",
  "{shadow}": "#000000",
  "{glow}": "#FFFF00"
}}
{"type": "sprite", "name": "shadow_sprite", "palette": "effects", "grid": [
  "{shadow}{shadow}",
  "{shadow}{shadow}"
]}
{"type": "sprite", "name": "glow_sprite", "palette": "effects", "grid": [
  "{glow}{glow}",
  "{glow}{glow}"
]}
{"type": "composition", "name": "layered", "size": [16, 16], "cell_size": [8, 8],
  "sprites": {"S": "shadow_sprite", "G": "glow_sprite", ".": null},
  "layers": [
    {"name": "shadows", "map": ["S.",".."], "blend": "var(--shadow-blend)", "opacity": "var(--shadow-opacity)"},
    {"name": "glows", "map": ["..","G."], "blend": "var(--glow-blend)", "opacity": "var(--glow-opacity)"}
  ]}
```

This creates a composition where shadow and glow effects are controlled by palette variables, making it easy to adjust effects across multiple compositions by changing a single variable definition.

### Error Handling

When variable resolution fails:

| Error | Behavior |
|-------|----------|
| Undefined variable (no fallback) | `opacity`: uses 1.0; `blend`: uses `normal` |
| Invalid opacity value | Uses 1.0, emits warning |
| Unknown blend mode | Uses `normal`, emits warning |
| No variable registry | Uses defaults, emits warning |

In strict mode (`--strict`), variable errors cause the render to fail.

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
