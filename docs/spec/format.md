# Pixelsrc Format Specification

**Version:** 0.3.0 (Draft)

---

## Overview

Pixelsrc (Text To Pixel) is a text-based format for defining pixel art sprites using JSON objects with a `type` field.

**File Extensions:**
- `.pxl` - Preferred extension (supports multi-line JSON)
- `.jsonl` - Legacy extension (supported for backward compatibility)

**Format Support:**
- **Single-line JSONL**: Traditional one-object-per-line format
- **Multi-line JSON**: Objects can span multiple lines for readability

**Design Philosophy:** Lenient by default, strict when requested. When GenAI makes small mistakes, fill the gaps and keep going.

---

## Object Types

### Palette

Defines named color tokens for use in sprites.

```json
{
  "type": "palette",
  "name": "string (required)",
  "colors": {
    "{token}": "#RRGGBB | #RRGGBBAA | #RGB | #RGBA (required, at least one)"
  }
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"palette"` |
| name | Yes | Unique identifier, referenced by sprites |
| colors | Yes | Map of token → color. Tokens must be `{name}` format |

**Color Formats:**
- `#RGB` → expands to `#RRGGBB` (e.g., `#F00` → `#FF0000`)
- `#RGBA` → expands to `#RRGGBBAA` (e.g., `#F00F` → `#FF0000FF`)
- `#RRGGBB` → fully opaque
- `#RRGGBBAA` → with alpha channel

**Reserved Tokens:**
- `{_}` → Recommended for transparency, but not enforced

#### Color Ramps

Auto-generate palette colors along a ramp with hue shifting. Shadows shift toward cool/warm rather than just being darker.

```json
{
  "type": "palette",
  "name": "character",
  "ramps": {
    "skin": {
      "base": "#E8B89D",
      "steps": 5,
      "shadow_shift": {"lightness": -15, "hue": 10, "saturation": 5},
      "highlight_shift": {"lightness": 12, "hue": -5, "saturation": -10}
    }
  },
  "colors": {
    "{_}": "#00000000"
  }
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| ramps | No | - | Map of ramp name to ramp definition |
| ramps.{name}.base | Yes | - | Base color in `#RRGGBB` format |
| ramps.{name}.steps | No | 3 | Total steps (odd numbers center on base) |
| ramps.{name}.shadow_shift | No | auto | Per-step shift toward shadows |
| ramps.{name}.highlight_shift | No | auto | Per-step shift toward highlights |

**Shift Parameters:**

| Parameter | Range | Description |
|-----------|-------|-------------|
| lightness | -100 to 100 | Lightness delta per step |
| hue | -180 to 180 | Hue rotation degrees per step |
| saturation | -100 to 100 | Saturation delta per step |

**Generated Tokens:** For a ramp named `skin` with `steps: 5`:
- `{skin_2}` - Darkest shadow (2 steps dark)
- `{skin_1}` - Shadow (1 step dark)
- `{skin}` - Base color
- `{skin+1}` - Highlight (1 step light)
- `{skin+2}` - Brightest (2 steps light)

**Inline Color Derivation:** Single-color variants without full ramps:

```json
{
  "colors": {
    "{skin}": "#E8B89D",
    "{skin_shadow}": {"from": "{skin}", "shift": {"lightness": -20, "hue": 15}},
    "{skin_highlight}": {"from": "{skin}", "shift": {"lightness": 15, "hue": -10}}
  }
}
```

---

### Sprite

Defines a pixel art image.

```json
{
  "type": "sprite",
  "name": "string (required)",
  "size": [width, height] (optional),
  "palette": "string | object (required)",
  "grid": ["row1", "row2", ...] (required)
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"sprite"` |
| name | Yes | Unique identifier |
| size | No | `[width, height]` - inferred from grid if omitted |
| palette | Yes | Palette name (string) or inline colors (object) |
| grid | Yes | Array of strings, each string is one row of tokens |

**Palette Reference Options:**
- Named: `"palette": "hero_colors"` → references palette defined earlier in stream
- Inline: `"palette": {"{_}": "#00000000", "{skin}": "#FFCC99"}`
- Built-in: `"palette": "@gameboy"` → references built-in palette (Phase 1)

**Grid Format:**
- Each string is one row of the sprite
- Tokens are `{name}` format, concatenated: `"{a}{b}{c}"`
- Rows are ordered top-to-bottom
- Tokens within row are left-to-right

#### Nine-Slice

Scalable sprites where corners stay fixed and edges/center stretch.

```json
{
  "type": "sprite",
  "name": "button",
  "palette": "ui",
  "nine_slice": {
    "left": 4,
    "right": 4,
    "top": 4,
    "bottom": 4
  },
  "grid": [...]
}
```

| Field | Required | Description |
|-------|----------|-------------|
| nine_slice | No | Nine-slice region definition |
| nine_slice.left | Yes | Left border width in pixels |
| nine_slice.right | Yes | Right border width in pixels |
| nine_slice.top | Yes | Top border height in pixels |
| nine_slice.bottom | Yes | Bottom border height in pixels |

**CLI rendering:** `pxl render button.pxl --nine-slice 64x32 -o button_wide.png`

#### Sprite Metadata

Additional data for game engine integration.

```json
{
  "type": "sprite",
  "name": "player_attack",
  "grid": [...],
  "metadata": {
    "origin": [16, 32],
    "boxes": {
      "hurt": {"x": 4, "y": 0, "w": 24, "h": 32},
      "hit": {"x": 20, "y": 8, "w": 20, "h": 16}
    }
  }
}
```

| Field | Required | Description |
|-------|----------|-------------|
| metadata | No | Sprite metadata object |
| metadata.origin | No | Sprite origin point `[x, y]` |
| metadata.boxes | No | Map of box name to rectangle |

**Box Types (Convention):**

| Name | Purpose |
|------|---------|
| `hurt` | Damage-receiving region |
| `hit` | Damage-dealing region |
| `collide` | Physics collision boundary |
| `trigger` | Interaction trigger zone |

---

### Animation

Defines a sequence of sprites as an animation.

```json
{
  "type": "animation",
  "name": "string (required)",
  "frames": ["sprite_name", ...] (required),
  "duration": number (optional, default 100),
  "loop": boolean (optional, default true)
}
```

**Fields:**
| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| type | Yes | - | Must be `"animation"` |
| name | Yes | - | Unique identifier |
| frames | Yes | - | Array of sprite names in order |
| duration | No | 100 | Milliseconds per frame |
| loop | No | true | Whether animation loops |

#### Palette Cycling

Animate by rotating palette colors instead of changing pixels. Classic technique for water, fire, energy effects.

```json
{
  "type": "animation",
  "name": "waterfall",
  "sprite": "water_static",
  "palette_cycle": {
    "tokens": ["{water1}", "{water2}", "{water3}", "{water4}"],
    "fps": 8,
    "direction": "forward"
  }
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| sprite | Yes* | - | Single sprite to cycle (*required if no `frames`) |
| palette_cycle | Yes | - | Cycle definition object or array |
| palette_cycle.tokens | Yes | - | Ordered list of tokens to rotate |
| palette_cycle.fps | No | 10 | Frames per second for cycling |
| palette_cycle.direction | No | "forward" | `"forward"` or `"reverse"` |

**Multiple Cycles:**
```json
{
  "palette_cycle": [
    {"tokens": ["{water1}", "{water2}", "{water3}"], "fps": 8},
    {"tokens": ["{glow1}", "{glow2}"], "fps": 4}
  ]
}
```

#### Frame Tags

Mark frame ranges with semantic names for game engine integration.

```json
{
  "type": "animation",
  "name": "player",
  "frames": ["idle1", "idle2", "run1", "run2", "run3", "run4", "jump", "fall"],
  "fps": 10,
  "tags": {
    "idle": {"start": 0, "end": 1, "loop": true},
    "run": {"start": 2, "end": 5, "loop": true},
    "jump": {"start": 6, "end": 6, "loop": false},
    "fall": {"start": 7, "end": 7, "loop": false}
  }
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| tags | No | - | Map of tag name to tag definition |
| tags.{name}.start | Yes | - | Starting frame index (0-based) |
| tags.{name}.end | Yes | - | Ending frame index (inclusive) |
| tags.{name}.loop | No | true | Whether this segment loops |
| tags.{name}.fps | No | inherit | Override FPS for this tag |

#### Per-Frame Metadata

Hitboxes and metadata that vary per frame:

```json
{
  "type": "animation",
  "name": "attack",
  "frames": ["f1", "f2", "f3"],
  "frame_metadata": [
    {"boxes": {"hit": null}},
    {"boxes": {"hit": {"x": 20, "y": 8, "w": 20, "h": 16}}},
    {"boxes": {"hit": {"x": 24, "y": 4, "w": 24, "h": 20}}}
  ]
}
```

#### Secondary Motion (Attachments)

Animate attached elements (hair, capes, tails) that follow the parent animation with configurable delay.

```json
{
  "type": "animation",
  "name": "hero_walk",
  "frames": ["walk_1", "walk_2", "walk_3", "walk_4"],
  "duration": 100,
  "attachments": [
    {
      "name": "hair",
      "anchor": [12, 4],
      "chain": ["hair_1", "hair_2", "hair_3"],
      "delay": 1,
      "follow": "position"
    },
    {
      "name": "cape",
      "anchor": [8, 8],
      "chain": ["cape_top", "cape_mid", "cape_bottom"],
      "delay": 2,
      "follow": "velocity",
      "z_index": -1
    }
  ]
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| attachments | No | - | Array of attachment definitions |
| attachments[].name | Yes | - | Identifier for this attachment |
| attachments[].anchor | Yes | - | Attachment point `[x, y]` on parent sprite |
| attachments[].chain | Yes | - | Array of sprite names forming the chain |
| attachments[].delay | No | 1 | Frame delay between chain segments |
| attachments[].follow | No | "position" | `"position"`, `"velocity"`, or `"rotation"` |
| attachments[].damping | No | 0.8 | Oscillation damping (0.0-1.0) |
| attachments[].stiffness | No | 0.5 | Spring stiffness (0.0-1.0) |
| attachments[].z_index | No | 0 | Render order (negative = behind parent) |

---

### Variant

Defines a color variation of an existing sprite.

```json
{
  "type": "variant",
  "name": "string (required)",
  "base": "string (required)",
  "palette": { "{token}": "#color", ... } (required)
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"variant"` |
| name | Yes | Unique identifier for this variant |
| base | Yes | Name of the sprite to derive from |
| palette | Yes | Color overrides - replaces matching tokens from base |

**Behavior:**
- Inherits grid and size from base sprite
- Only specified tokens are overridden; others remain from base
- Base sprite must be defined before the variant

**Example:**
```jsonl
{"type": "sprite", "name": "hero", "palette": {"{skin}": "#FFCC99", "{hair}": "#8B4513"}, "grid": [...]}
{"type": "variant", "name": "hero_red", "base": "hero", "palette": {"{hair}": "#FF0000"}}
```

---

### Composition

Composes multiple sprites onto a canvas using a character-based map.

```json
{
  "type": "composition",
  "name": "string (required)",
  "base": "string (optional)",
  "size": [width, height] (optional),
  "cell_size": [width, height] (optional, default [1, 1]),
  "sprites": { "char": "sprite_name" | null, ... } (required),
  "layers": [ { "name": "...", "map": [...] }, ... ] (required)
}
```

**Fields:**
| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| type | Yes | - | Must be `"composition"` |
| name | Yes | - | Unique identifier |
| base | No | - | Base sprite to render first (background) |
| size | No | Inferred | Canvas size `[width, height]` in pixels |
| cell_size | No | `[1, 1]` | Size of each map character in pixels |
| sprites | Yes | - | Map of single characters to sprite names (`null` = transparent) |
| layers | Yes | - | Array of layers, rendered bottom-to-top |

**Layer Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| name | No | Layer identifier (for debugging) |
| fill | No | Sprite name to fill entire layer |
| map | No | Array of strings - character map for sprite placement |
| blend | No | Blend mode (default: "normal") |
| opacity | No | Layer opacity 0.0-1.0 (default: 1.0) |

#### Blend Modes

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

**Size Inference (priority order):**
1. Explicit `size` field
2. `base` sprite dimensions
3. Inferred from `layers` and `cell_size`

**Cell Size (Tiling):**

The `cell_size` field enables tiling - composing large images from smaller sprites:
- `cell_size: [1, 1]` (default) - each map character = 1 pixel position
- `cell_size: [8, 8]` - each map character = 8×8 pixel region
- `cell_size: [32, 32]` - each map character = 32×32 pixel tile
- `cell_size: [4, 8]` - custom cells for text/UI composition

Sprites are placed at `(col * cell_size[0], row * cell_size[1])`.

**Size Mismatch Handling:**
- **Lenient mode:** Sprite larger than cell emits warning, anchors top-left, overwrites adjacent cells
- **Strict mode:** Sprite larger than cell returns error

**Example (Pixel Overlay):**
```jsonl
{"type": "composition", "name": "scene", "size": [32, 32],
  "sprites": {".": null, "H": "hero"},
  "layers": [{"map": ["........", "..H.....", "........"]}]}
```

**Example (Tiled Scene):**
```jsonl
{"type": "composition", "name": "landscape", "size": [64, 64], "cell_size": [32, 32],
  "sprites": {"A": "sky_left", "B": "sky_right", "C": "ground_left", "D": "ground_right"},
  "layers": [{"map": ["AB", "CD"]}]}
```

**Example (Layered with Base):**
```jsonl
{"type": "composition", "name": "hero_scene", "base": "background",
  "sprites": {".": null, "H": "hero", "E": "enemy"},
  "layers": [
    {"name": "characters", "map": ["..H..", "...E."]}
  ]}
```

**Example (Text Banner):**
```jsonl
{"type": "composition", "name": "banner", "size": [20, 8], "cell_size": [4, 8],
  "sprites": {"{": "bracket_l", "p": "letter_p", "x": "letter_x", "l": "letter_l", "}": "bracket_r"},
  "layers": [{"map": ["{pxl}"]}]}
```

#### Nested Compositions

Compositions can reference other compositions in their `sprites` map, enabling hierarchical scene construction. When rendering, if a sprite name is not found in the sprite pool, the renderer checks for a composition with that name and renders it recursively.

**Key Features:**
- **Recursive rendering:** Compositions can contain other compositions at any nesting depth
- **Cycle detection:** Self-referential cycles (A → B → A) are detected and reported as errors
- **Caching:** Rendered compositions are cached, so repeated references reuse the cached result

**Example (Nested Scene):**
```jsonl
{"type": "sprite", "name": "tree", "data": ["G", "T"], "palette": {"G": "#228B22", "T": "#8B4513"}}
{"type": "sprite", "name": "rock", "data": ["RR", "RR"], "palette": {"R": "#696969"}}
{"type": "composition", "name": "forest_tile", "size": [16, 16], "cell_size": [8, 8],
  "sprites": {"T": "tree", ".": null},
  "layers": [{"map": ["TT", ".T"]}]}
{"type": "composition", "name": "scene", "size": [32, 32], "cell_size": [16, 16],
  "sprites": {"F": "forest_tile", "R": "rock"},
  "layers": [{"map": ["FF", "FR"]}]}
```

In this example, `scene` references `forest_tile` as if it were a sprite. When rendering `scene`, the renderer:
1. Encounters `F` mapped to `forest_tile`
2. Finds no sprite named `forest_tile`, but finds a composition
3. Recursively renders `forest_tile` to produce an image
4. Uses that rendered image in place of a sprite

**Base as Composition:**

The `base` field can also reference a composition:
```jsonl
{"type": "composition", "name": "background", "size": [64, 64], ...}
{"type": "composition", "name": "scene", "base": "background",
  "sprites": {"H": "hero"},
  "layers": [{"map": ["..H.."]}]}
```

**Cycle Detection:**

Circular references are detected and produce an error:
```jsonl
{"type": "composition", "name": "A", "sprites": {"B": "comp_b"}, ...}
{"type": "composition", "name": "comp_b", "sprites": {"A": "A"}, ...}
```
This produces: `Error: Cycle detected in composition references: A -> comp_b -> A`

---

## Transform Operations

Transforms modify sprites at render time without changing the source.

### Dithering Patterns

Apply dithering patterns for gradients, transparency effects, and texture.

```json
{
  "type": "sprite",
  "name": "gradient",
  "source": "solid",
  "transform": [
    {"op": "dither", "pattern": "checker", "tokens": ["{dark}", "{light}"], "threshold": 0.5}
  ]
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | Must be `"dither"` |
| pattern | Yes | - | Dither pattern name |
| tokens | Yes | - | Two-element array `[dark, light]` |
| threshold | No | 0.5 | Blend threshold (0.0-1.0) |
| seed | No | auto | Random seed for noise pattern |

**Built-in Patterns:**

| Pattern | Description |
|---------|-------------|
| `checker` | 2x2 checkerboard |
| `ordered-2x2` | 2x2 Bayer matrix (4 levels) |
| `ordered-4x4` | 4x4 Bayer matrix (16 levels) |
| `ordered-8x8` | 8x8 Bayer matrix (64 levels) |
| `diagonal` | Diagonal line pattern |
| `horizontal` | Horizontal line pattern |
| `vertical` | Vertical line pattern |
| `noise` | Random dither (seeded) |

**Gradient Dither:**
```json
{
  "op": "dither-gradient",
  "direction": "vertical",
  "from": "{sky_light}",
  "to": "{sky_dark}",
  "pattern": "ordered-4x4"
}
```

### Selective Outline (Sel-out)

Outline color varies based on adjacent fill color, creating softer edges.

```json
{
  "transform": [
    {"op": "sel-out", "fallback": "{outline}"}
  ]
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | Must be `"sel-out"` |
| fallback | No | "{_}" | Default outline color |
| auto_darken | No | 0.3 | Auto-darken factor (0.0-1.0) |
| mapping | No | auto | Explicit fill→outline mapping |

**Explicit Mapping:**
```json
{
  "op": "sel-out",
  "mapping": {
    "{skin}": "{skin_dark}",
    "{hair}": "{hair_dark}",
    "*": "{outline}"
  }
}
```

### Squash & Stretch

Deform sprites for impact and bounce effects.

```json
{
  "transform": [
    {"op": "squash", "amount": 0.3}
  ]
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | `"squash"` or `"stretch"` |
| amount | Yes | - | Deformation amount (0.0-1.0) |
| anchor | No | "center" | Transform anchor point |
| preserve_area | No | true | Maintain sprite area |

**Anchor Points:** `"center"`, `"bottom"`, `"top"`, or `[x, y]` coordinates.

---

## Token Parsing

Tokens in grid strings follow this pattern:

```
\{[^}]+\}
```

**Parsing Algorithm:**
1. Scan string left-to-right
2. On `{`, begin token capture
3. On `}`, end token capture, emit token
4. Characters outside `{...}` are errors (see Error Handling)

**Examples:**
| Grid String | Parsed Tokens |
|-------------|---------------|
| `"{a}{b}{c}"` | `["{a}", "{b}", "{c}"]` |
| `"{_}{skin}{_}"` | `["{_}", "{skin}", "{_}"]` |
| `"{long_name}{x}"` | `["{long_name}", "{x}"]` |

**Token Names:**
- Case sensitive: `{Skin}` ≠ `{skin}`
- Whitespace preserved: `{ skin }` is a valid (but discouraged) token
- Recommended: lowercase, underscores: `{dark_skin}`, `{hair_highlight}`

---

## Error Handling

Pixelsrc has two modes: **lenient** (default) and **strict**.

### Lenient Mode (Default)

Fill gaps, warn, continue. Designed for GenAI iteration.

| Error | Behavior | Warning |
|-------|----------|---------|
| Row too short | Pad with `{_}` (transparent) | "Row N has M tokens, expected W" |
| Row too long | Truncate | "Row N has M tokens, expected W, truncating" |
| Unknown token in grid | Render as magenta `#FF00FF` | "Unknown token {foo} in sprite X" |
| Undefined palette reference | Error if no inline fallback | "Palette 'X' not found" |
| Duplicate name | Last definition wins | "Duplicate sprite name 'X', using latest" |
| Invalid color format | Use magenta `#FF00FF` | "Invalid color 'X', using magenta" |
| Characters outside tokens | Ignore | "Unexpected character 'X' in grid row" |
| Empty grid | Create 1x1 transparent | "Empty grid in sprite X" |
| Missing required field | Error (cannot fill) | "Missing required field 'X'" |

### Strict Mode (`--strict`)

Fail on first error. Designed for CI/validation.

All warnings in lenient mode become errors in strict mode. Processing stops at first error with non-zero exit code.

---

## Size Inference

If `size` is omitted:
- Width = max tokens in any row
- Height = number of rows

If `size` is provided:
- Rows are padded/truncated to match width
- Grid is padded/truncated to match height

---

## Stream Processing

Pixelsrc files use streaming JSON parsing:

1. Objects are parsed as complete JSON values (may span multiple lines)
2. Objects are processed in order of appearance
3. Palettes must be defined before sprites that reference them (by name)
4. Forward references are errors (lenient: use magenta, strict: fail)

**Whitespace:** Ignored between objects
**Comments:** Not supported in JSON (use separate documentation)

### Single-Line Format (JSONL)

Traditional format with one object per line:

```jsonl
{"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF"}}
{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}
```

### Multi-Line Format

Objects can span multiple lines for improved readability, especially for sprite grids:

```json
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "colors", "grid": [
  "{_}{_}{hair}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}"
]}
```

Both formats parse identically—the renderer handles concatenated JSON objects regardless of whitespace.

---

## Output Behavior

### Default Output Naming

```bash
pxl render input.pxl      # or input.jsonl
```

| Scenario | Output |
|----------|--------|
| Single sprite "hero" | `input_hero.png` |
| Multiple sprites | `input_{name}.png` for each |
| With `-o output.png` (single sprite) | `output.png` |
| With `-o output.png` (multiple) | `output_{name}.png` |
| With `-o dir/` | `dir/{name}.png` |
| With `--sprite hero` | Only render "hero" |

Both `.pxl` and `.jsonl` extensions produce identical output—the extension only affects the source file naming convention.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (lenient: may have warnings) |
| 1 | Error (strict: any warning; lenient: fatal error) |
| 2 | Invalid arguments |

### Atlas Export

Pack multiple sprites into a single texture with coordinate metadata for game engines.

```bash
pxl render game.pxl --format atlas -o game_atlas
# Outputs: game_atlas.png + game_atlas.json
```

**Output JSON Structure:**
```json
{
  "image": "game_atlas.png",
  "size": [128, 64],
  "frames": {
    "coin": {"x": 0, "y": 0, "w": 16, "h": 16},
    "player": {"x": 16, "y": 0, "w": 32, "h": 32}
  },
  "animations": {
    "player_walk": {
      "frames": ["player_walk_1", "player_walk_2"],
      "fps": 8,
      "tags": {"idle": {"from": 0, "to": 1}}
    }
  }
}
```

**Packing Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--max-size WxH` | Maximum atlas dimensions | `--max-size 512x512` |
| `--padding N` | Pixels between sprites | `--padding 1` |
| `--power-of-two` | Force power-of-2 dimensions | `--power-of-two` |
| `--sprites "pattern"` | Filter sprites by glob | `--sprites "player_*"` |

**Format Variants:**

| Format | Description |
|--------|-------------|
| `atlas` | Generic JSON (default) |
| `atlas-aseprite` | Aseprite-compatible JSON |
| `atlas-godot` | Godot resource format |
| `atlas-unity` | Unity sprite atlas |
| `atlas-libgdx` | libGDX texture packer |

### Onion Skinning

Preview animation frames with ghosted previous/next frames.

```bash
pxl show walk_cycle.pxl --onion 2
```

| Option | Default | Description |
|--------|---------|-------------|
| `--onion N` | - | Show N frames before/after |
| `--onion-opacity` | 0.3 | Ghost frame opacity |
| `--onion-prev-color` | "#FF0000" | Tint for previous frames |
| `--onion-next-color` | "#0000FF" | Tint for next frames |

---

## Formatting

The `pxl fmt` command formats pixelsrc files for improved readability.

### Command Options

```bash
pxl fmt <files...>         # Format files in-place
pxl fmt <files> --check    # Check formatting without writing (exit 1 if changes needed)
pxl fmt <files> --stdout   # Write formatted output to stdout
```

### Formatting Rules

The formatter applies these rules:

**Sprites** - Grid arrays expanded for visual alignment:
```json
{"type": "sprite", "name": "hero", "size": [16, 16], "palette": "colors", "grid": [
  "{_}{_}{_}{_}{o}{o}{o}{o}{o}{o}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{o}{skin}{skin}{skin}{skin}{skin}{skin}{o}{_}{_}{_}{_}{_}",
  ...
]}
```

**Compositions** - Layer maps expanded for visual clarity:
```json
{"type": "composition", "name": "scene", "size": [64, 64], "sprites": {"H": "hero", "T": "tree"}, "layers": [
  {"name": "background", "fill": "grass"},
  {"name": "objects", "map": [
    "T......T",
    "........",
    "...H....",
    "........"
  ]}
]}
```

**Palettes** - Single line for compactness:
```json
{"type": "palette", "name": "colors", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{o}": "#000000"}}
```

**Animations** - Single line:
```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

### Round-Trip Safety

Formatting is lossless—rendered output is identical before and after formatting:

```bash
pxl render input.jsonl -o before.png
pxl fmt input.jsonl --stdout > formatted.pxl
pxl render formatted.pxl -o after.png
diff before.png after.png  # Identical
```

---

## Examples

### Minimal Sprite (Inline Palette)

```jsonl
{"type": "sprite", "name": "dot", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}
```

### Sprite with Named Palette (Multi-Line)

```json
{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFD5B4", "{hair}": "#8B4513", "{shirt}": "#4169E1"}}

{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "hero", "grid": [
  "{_}{_}{hair}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}"
]}
```

### Checker Pattern (Single-Line)

```jsonl
{"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF", "{off}": "#000000"}}
{"type": "sprite", "name": "checker", "palette": "mono", "grid": ["{on}{off}{on}{off}", "{off}{on}{off}{on}", "{on}{off}{on}{off}", "{off}{on}{off}{on}"]}
```

### Animation

```jsonl
{"type": "palette", "name": "blink", "colors": {"{_}": "#00000000", "{o}": "#FFFF00"}}
{"type": "sprite", "name": "on", "palette": "blink", "grid": ["{o}{o}", "{o}{o}"]}
{"type": "sprite", "name": "off", "palette": "blink", "grid": ["{_}{_}", "{_}{_}"]}
{"type": "animation", "name": "blink_anim", "frames": ["on", "off"], "duration": 500, "loop": true}
```

---

## Implementation Notes

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `serde`, `serde_json` | JSON parsing |
| `image` | PNG/GIF/WebP generation (native, no ImageMagick) |
| `clap` | CLI argument parsing |
| `regex` | Token extraction (or manual parser) |

### Rendering Pipeline

1. Parse JSONL line-by-line
2. Build palette registry (name → colors map)
3. For each sprite:
   a. Resolve palette (named or inline)
   b. Parse grid into 2D token array
   c. Map tokens to RGBA colors
   d. Create `RgbaImage` and set pixels
   e. Save to output format

---

## Version History

| Version | Changes |
|---------|---------|
| 0.3.0 | ATF: color ramps, palette cycling, frame tags, nine-slice, blend modes, transforms, atlas export |
| 0.2.1 | Added composition with cell_size for tiling |
| 0.2.0 | Add `.pxl` extension, multi-line JSON support, `pxl fmt` command |
| 0.1.0 | Initial draft |
