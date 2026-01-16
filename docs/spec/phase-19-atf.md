# Phase 19: Advanced Texture Features (ATF) Format Specification

**Version:** 0.1.0 (Draft)
**Depends on:** Phase 0 (Core), Phase 16 (.pxl format)

---

## Overview

Phase 19 adds Advanced Texture Features (ATF) to the Pixelsrc format, enabling:
- Enhanced palette management (color ramps, hue-shifted shadows)
- Animation features (palette cycling, frame tags)
- Game engine integration (atlas export, nine-slice, hitboxes)
- Transform operations (dithering, selective outline, squash/stretch)
- Preview tools (onion skinning, blend modes)

All features are backward compatible. Existing `.pxl` files continue to work unchanged.

---

## ATF-2: Color Ramps

**Type:** Palette Attribute

Auto-generate palette colors along a ramp with hue shifting. Shadows shift toward cool/warm rather than just being darker.

### Syntax

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

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| ramps | No | - | Map of ramp name to ramp definition |
| ramps.{name}.base | Yes | - | Base color in `#RRGGBB` format |
| ramps.{name}.steps | No | 3 | Total steps (odd numbers center on base) |
| ramps.{name}.shadow_shift | No | auto | Per-step shift toward shadows |
| ramps.{name}.highlight_shift | No | auto | Per-step shift toward highlights |

### Shift Parameters

| Parameter | Range | Description |
|-----------|-------|-------------|
| lightness | -100 to 100 | Lightness delta per step |
| hue | -180 to 180 | Hue rotation degrees per step |
| saturation | -100 to 100 | Saturation delta per step |

### Generated Tokens

For a ramp named `skin` with `steps: 5`:

| Token | Description |
|-------|-------------|
| `{skin_2}` | Darkest shadow (2 steps dark) |
| `{skin_1}` | Shadow (1 step dark) |
| `{skin}` | Base color |
| `{skin+1}` | Highlight (1 step light) |
| `{skin+2}` | Brightest (2 steps light) |

### Simplified Syntax

```json
{
  "ramps": {
    "skin": {"base": "#E8B89D", "steps": 3}
  }
}
```

Uses sensible defaults:
- `shadow_shift`: `{"lightness": -12, "hue": 8, "saturation": 5}`
- `highlight_shift`: `{"lightness": 10, "hue": -5, "saturation": -8}`

### Inline Color Derivation

Single-color variants without full ramps:

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

## ATF-3: Palette Cycling

**Type:** Animation Attribute

Animate by rotating palette colors instead of changing pixels. Classic technique for water, fire, energy effects.

### Syntax

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

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| sprite | Yes* | - | Single sprite to cycle (*required if no `frames`) |
| palette_cycle | Yes | - | Cycle definition object or array |
| palette_cycle.tokens | Yes | - | Ordered list of tokens to rotate |
| palette_cycle.fps | No | 10 | Frames per second for cycling |
| palette_cycle.direction | No | "forward" | `"forward"` or `"reverse"` |

### How It Works

Single sprite, colors rotate through the token list:
- Frame 0: water1→#1, water2→#2, water3→#3, water4→#4
- Frame 1: water1→#2, water2→#3, water3→#4, water4→#1
- Frame 2: water1→#3, water2→#4, water3→#1, water4→#2
- etc.

### Multiple Cycles

```json
{
  "palette_cycle": [
    {"tokens": ["{water1}", "{water2}", "{water3}"], "fps": 8},
    {"tokens": ["{glow1}", "{glow2}"], "fps": 4}
  ]
}
```

Each cycle runs independently at its own FPS.

### Combined with Frames

Palette cycling can combine with traditional frame animation:

```json
{
  "type": "animation",
  "name": "magic_attack",
  "frames": ["cast1", "cast2", "cast3"],
  "duration": 100,
  "palette_cycle": {
    "tokens": ["{magic1}", "{magic2}", "{magic3}"],
    "fps": 12
  }
}
```

---

## ATF-4: Frame Tags

**Type:** Animation Attribute

Mark frame ranges with semantic names for game engine integration.

### Syntax

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

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| tags | No | - | Map of tag name to tag definition |
| tags.{name}.start | Yes | - | Starting frame index (0-based) |
| tags.{name}.end | Yes | - | Ending frame index (inclusive) |
| tags.{name}.loop | No | true | Whether this segment loops |
| tags.{name}.fps | No | inherit | Override FPS for this tag |

### Tag-Specific FPS

```json
{
  "tags": {
    "idle": {"start": 0, "end": 1, "fps": 4},
    "run": {"start": 2, "end": 5, "fps": 12},
    "attack": {"start": 6, "end": 9, "fps": 15}
  }
}
```

### Export Behavior

Tags are included in atlas exports and GIF metadata:

```bash
pxl render player.pxl --format atlas -o player
```

Produces `player.json` with:
```json
{
  "animations": {
    "player": {
      "frames": [...],
      "tags": {
        "idle": {"from": 0, "to": 1, "loop": true},
        "run": {"from": 2, "to": 5, "loop": true, "fps": 12}
      }
    }
  }
}
```

---

## ATF-5: Atlas Export

**Type:** CLI Export Format

Pack multiple sprites into a single texture with coordinate metadata.

### Basic Usage

```bash
pxl render game.pxl --format atlas -o game_atlas
```

Outputs:
- `game_atlas.png` - Packed sprite sheet
- `game_atlas.json` - Coordinate metadata

### Output Format

```json
{
  "image": "game_atlas.png",
  "size": [128, 64],
  "frames": {
    "coin": {"x": 0, "y": 0, "w": 16, "h": 16},
    "player": {"x": 16, "y": 0, "w": 32, "h": 32},
    "enemy": {"x": 48, "y": 0, "w": 24, "h": 24}
  },
  "animations": {
    "player_walk": {
      "frames": ["player_walk_1", "player_walk_2", "player_walk_3"],
      "fps": 8,
      "tags": {...}
    }
  }
}
```

### CLI Options

| Option | Description |
|--------|-------------|
| `--format atlas` | Enable atlas packing |
| `--max-size WxH` | Maximum atlas dimensions |
| `--padding N` | Pixels between sprites (prevents bleed) |
| `--power-of-two` | Force power-of-two dimensions |
| `--sprites "pattern"` | Only include matching sprites |
| `--animations-only` | Only include animation frames |

### Format Variants

```bash
pxl render game.pxl --format atlas-aseprite -o out   # Aseprite-compatible
pxl render game.pxl --format atlas-godot -o out      # Godot-compatible
pxl render game.pxl --format atlas-unity -o out      # Unity-compatible
pxl render game.pxl --format atlas-libgdx -o out     # libGDX-compatible
```

### Multiple Atlases

If sprites exceed `--max-size`, multiple atlases are created:

```bash
pxl render game.pxl --format atlas --max-size 256x256 -o atlas
# Outputs: atlas_0.png, atlas_0.json, atlas_1.png, atlas_1.json, ...
```

---

## ATF-6: Nine-Slice

**Type:** Sprite Attribute

Scalable sprites where corners stay fixed and edges/center stretch.

### Syntax

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
  "grid": [
    "{c}{t}{t}{t}{t}{t}{t}{c}",
    "{l}{m}{m}{m}{m}{m}{m}{r}",
    "{l}{m}{m}{m}{m}{m}{m}{r}",
    "{c}{b}{b}{b}{b}{b}{b}{c}"
  ]
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| nine_slice | No | Nine-slice region definition |
| nine_slice.left | Yes | Left border width in pixels |
| nine_slice.right | Yes | Right border width in pixels |
| nine_slice.top | Yes | Top border height in pixels |
| nine_slice.bottom | Yes | Bottom border height in pixels |

### Regions

```
┌─────┬───────────────┬─────┐
│  TL │      TOP      │  TR │  <- top border (fixed height)
├─────┼───────────────┼─────┤
│     │               │     │
│  L  │    CENTER     │  R  │  <- stretches vertically
│     │               │     │
├─────┼───────────────┼─────┤
│  BL │    BOTTOM     │  BR │  <- bottom border (fixed height)
└─────┴───────────────┴─────┘
   ^          ^          ^
  left     stretches   right
(fixed)   horizontally (fixed)
```

### CLI Rendering

```bash
pxl render button.pxl --nine-slice 64x32 -o button_wide.png
```

### In Compositions

```json
{
  "type": "composition",
  "layers": [
    {"sprite": "button", "position": [0, 0], "nine_slice_size": [100, 40]}
  ]
}
```

---

## ATF-7: Hit/Hurt Boxes

**Type:** Sprite/Animation Metadata

Collision regions for game integration.

### Sprite Metadata

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

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| metadata | No | Sprite metadata object |
| metadata.origin | No | Sprite origin point `[x, y]` |
| metadata.boxes | No | Map of box name to rectangle |
| metadata.boxes.{name}.x | Yes | Box X position |
| metadata.boxes.{name}.y | Yes | Box Y position |
| metadata.boxes.{name}.w | Yes | Box width |
| metadata.boxes.{name}.h | Yes | Box height |

### Per-Frame Boxes in Animations

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

### Box Types (Convention)

| Name | Purpose |
|------|---------|
| `hurt` | Damage-receiving region |
| `hit` | Damage-dealing region |
| `collide` | Physics collision boundary |
| `trigger` | Interaction trigger zone |

### Export Behavior

Metadata is included in atlas exports:

```json
{
  "frames": {
    "player_attack": {
      "x": 0, "y": 0, "w": 32, "h": 32,
      "origin": [16, 32],
      "boxes": {
        "hurt": {"x": 4, "y": 0, "w": 24, "h": 32},
        "hit": {"x": 20, "y": 8, "w": 20, "h": 16}
      }
    }
  }
}
```

---

## ATF-8: Dithering Patterns

**Type:** Transform Operation

Apply dithering patterns for gradients, transparency effects, and texture.

### Syntax

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

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | Must be `"dither"` |
| pattern | Yes | - | Dither pattern name |
| tokens | Yes | - | Two-element array `[dark, light]` |
| threshold | No | 0.5 | Blend threshold (0.0-1.0) |
| seed | No | auto | Random seed for noise pattern |

### Built-in Patterns

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

### Gradient Dither

```json
{
  "op": "dither-gradient",
  "direction": "vertical",
  "from": "{sky_light}",
  "to": "{sky_dark}",
  "pattern": "ordered-4x4"
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | Must be `"dither-gradient"` |
| direction | No | "vertical" | `"vertical"`, `"horizontal"`, `"radial"` |
| from | Yes | - | Starting token |
| to | Yes | - | Ending token |
| pattern | No | "ordered-4x4" | Dither pattern to use |

---

## ATF-9: Selective Outline (Sel-out)

**Type:** Transform Operation

Outline color varies based on adjacent fill color, creating softer edges.

### Syntax

```json
{
  "transform": [
    {"op": "sel-out", "fallback": "{outline}"}
  ]
}
```

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | Must be `"sel-out"` |
| fallback | No | "{_}" | Default outline color |
| auto_darken | No | 0.3 | Auto-darken factor (0.0-1.0) |
| mapping | No | auto | Explicit fill→outline mapping |

### How It Works

1. Examines each outline pixel's neighboring fill pixels
2. Picks a darker/shifted version of the most common neighbor
3. Falls back to specified color if undetermined

### Explicit Mapping

```json
{
  "op": "sel-out",
  "mapping": {
    "{skin}": "{skin_dark}",
    "{hair}": "{hair_dark}",
    "{shirt}": "{shirt_outline}",
    "*": "{outline}"
  }
}
```

The `"*"` key provides the fallback for unmapped tokens.

### Auto-Darken Mode

```json
{
  "op": "sel-out",
  "auto_darken": 0.4
}
```

Automatically generates outline colors by darkening fills by the specified factor.

---

## ATF-10: Blend Modes

**Type:** Composition Layer Attribute

Layer blending for compositions.

### Syntax

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

### Layer Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| blend | No | "normal" | Blend mode name |
| opacity | No | 1.0 | Layer opacity (0.0-1.0) |

### Blend Modes

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

### Blend Math

| Mode | Formula |
|------|---------|
| multiply | `result = base * blend` |
| screen | `result = 1 - (1 - base) * (1 - blend)` |
| overlay | `result = base < 0.5 ? 2*base*blend : 1 - 2*(1-base)*(1-blend)` |
| add | `result = min(1, base + blend)` |

---

## ATF-11: Onion Skinning

**Type:** CLI Preview Feature

Preview previous/next frames as transparent overlays.

### Usage

```bash
pxl show walk_cycle.pxl --onion 2
```

Shows current frame with 2 previous and 2 next frames as ghosts.

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--onion N` | - | Number of frames before/after to show |
| `--onion-opacity F` | 0.3 | Ghost frame opacity (0.0-1.0) |
| `--onion-prev-color HEX` | #0000FF | Tint for previous frames |
| `--onion-next-color HEX` | #00FF00 | Tint for next frames |
| `--onion-fade` | false | Decrease opacity for distant frames |

### Fade Mode

With `--onion-fade`, opacity decreases with distance:
- Frame ±1: 100% of specified opacity
- Frame ±2: 67% of specified opacity
- Frame ±3: 33% of specified opacity

### Output

Renders to terminal using ANSI colors (requires compatible terminal) or exports to image:

```bash
pxl show walk.pxl --onion 2 -o onion_preview.png
```

---

## ATF-12: Squash & Stretch

**Type:** Transform Operation

Deform sprites for impact and bounce effects.

### Basic Syntax

```json
{
  "transform": [
    {"op": "squash", "amount": 0.3}
  ]
}
```

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| op | Yes | - | `"squash"` or `"stretch"` |
| amount | Yes | - | Deformation amount (0.0-1.0) |
| anchor | No | "center" | Transform anchor point |
| preserve_area | No | true | Maintain sprite area |

### Behavior

- `squash`: Compress vertically, expand horizontally
- `stretch`: Expand vertically, compress horizontally

With `preserve_area: true`:
- `scale_x = 1 + amount`, `scale_y = 1 / (1 + amount)`

### Anchor Points

| Anchor | Description |
|--------|-------------|
| `"center"` | Scale from center |
| `"bottom"` | Scale from bottom edge |
| `"top"` | Scale from top edge |
| `[x, y]` | Custom anchor coordinates |

### Keyframed Animation

```json
{
  "type": "transform",
  "name": "bounce-impact",
  "frames": 8,
  "keyframes": [
    {"frame": 0, "squash": 0},
    {"frame": 2, "squash": 0.4},
    {"frame": 4, "squash": -0.2},
    {"frame": 8, "squash": 0}
  ],
  "easing": "ease-out",
  "anchor": "bottom"
}
```

Negative values for `squash` produce stretch effect.

### Scaling Mode

| Mode | Description |
|------|-------------|
| `nearest` | Nearest neighbor (blocky, crisp) |
| `smooth` | Bilinear interpolation (blurry) |
| `row-duplicate` | Duplicate/remove rows (pixel-art friendly) |

```json
{
  "op": "squash",
  "amount": 0.3,
  "mode": "row-duplicate"
}
```

---

## Compatibility Matrix

| Feature | Sprite | Animation | Composition | Palette | CLI |
|---------|--------|-----------|-------------|---------|-----|
| Color Ramps | - | - | - | **Yes** | - |
| Palette Cycling | - | **Yes** | - | - | - |
| Frame Tags | - | **Yes** | - | - | - |
| Atlas Export | - | - | - | - | **Yes** |
| Nine-Slice | **Yes** | - | layer | - | **Yes** |
| Hit/Hurt Boxes | **Yes** | **Yes** | - | - | - |
| Dithering | transform | transform | - | - | - |
| Selective Outline | transform | transform | - | - | - |
| Blend Modes | - | - | **Yes** | - | - |
| Onion Skinning | - | - | - | - | **Yes** |
| Squash & Stretch | transform | transform | - | - | - |

---

## Error Handling

### Lenient Mode (Default)

| Error | Behavior |
|-------|----------|
| Invalid ramp steps | Use default (3) |
| Unknown blend mode | Use "normal" |
| Invalid box coordinates | Warn, skip box |
| Missing metadata field | Skip optional field |
| Invalid dither pattern | Use "checker" |

### Strict Mode (`--strict`)

All warnings become errors. Processing stops on first error.

---

## Version History

| Version | Changes |
|---------|---------|
| 0.1.0 | Initial draft - ATF-2 through ATF-12 |

---

## Implementation Priority

### Wave 1 (High Priority)
- ATF-2: Color Ramps
- ATF-3: Palette Cycling
- ATF-4: Frame Tags
- ATF-5: Atlas Export

### Wave 2 (Medium Priority)
- ATF-6: Nine-Slice
- ATF-7: Hit/Hurt Boxes
- ATF-10: Blend Modes

### Wave 3 (Standard Priority)
- ATF-8: Dithering Patterns
- ATF-9: Selective Outline
- ATF-11: Onion Skinning
- ATF-12: Squash & Stretch
