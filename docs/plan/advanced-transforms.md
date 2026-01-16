---
phase: 19
title: Advanced Transforms & Animation Features
---

# Phase 19: Advanced Transforms & Animation Features

**Status:** Not Started

**Depends on:** Phase 18 (Sprite Transforms - core transform system required)

---

Advanced features for power users. See [personas](../personas.md) for user context.

**Related:**
- [Transforms](./transforms.md) - Core transform system (implement first)
- [Colored Grid Display](./colored-grid-display.md) - Terminal preview tools

---

## Feature Overview

| Feature | Persona | Implementation | Priority |
|---------|---------|----------------|----------|
| **Palette cycling** | Animator+ | Animation attribute | ★★★ High |
| **Color ramps** | Pixel Artist+ | Palette attribute | ★★★ High |
| **Frame tags** | Game Dev | Animation attribute | ★★★ High |
| **Atlas export** | Game Dev | CLI export format | ★★★ High |
| **Nine-slice** | Game Dev | Sprite attribute | ★★☆ Medium |
| **Dithering patterns** | Pixel Artist | Transform operation | ★★☆ Medium |
| **Hue-shifted shadows** | Pixel Artist | Palette attribute | ★★☆ Medium |
| **Sel-out (selective outline)** | Pixel Artist | Transform operation | ★★☆ Medium |
| **Blend modes** | Pixel Artist+ | Composition layer attr | ★★☆ Medium |
| **Onion skinning** | Animator | CLI preview feature | ★★☆ Medium |
| **Sub-pixel animation** | Motion Designer | Transform operation | ★☆☆ Lower |
| **Squash & stretch** | Motion Designer | Transform operation | ★☆☆ Lower |
| **Secondary motion** | Motion Designer | Animation attribute | ★☆☆ Lower |
| **Particle systems** | Motion Designer | New type | ★☆☆ Lower |
| **Parallax hints** | Game Dev | Composition layer attr | ★☆☆ Lower |
| **Hit/hurt boxes** | Game Dev | Metadata | ★☆☆ Lower |
| **Arc motion paths** | Motion Designer | Keyframe feature | ★☆☆ Lower |

---

## High Priority Features

### Palette Cycling

**Persona:** Animator, Motion Designer, Game Dev

Animate by rotating palette colors instead of changing pixels. Classic technique for water, fire, energy effects.

**Implementation:** Animation attribute (not a transform)

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

**How it works:**
- Single sprite, no frame changes
- Colors rotate through the token list
- Frame 0: water1→#1, water2→#2, water3→#3, water4→#4
- Frame 1: water1→#2, water2→#3, water3→#4, water4→#1
- etc.

**Multiple cycles:**
```json
{
  "palette_cycle": [
    {"tokens": ["{water1}", "{water2}", "{water3}"], "fps": 8},
    {"tokens": ["{glow1}", "{glow2}"], "fps": 4}
  ]
}
```

**Why not a transform?** Palette cycling doesn't modify the grid or generate frames in the traditional sense - it modifies color mapping over time. It's fundamentally different from spatial transforms.

---

### Color Ramps

**Persona:** Pixel Artist, Motion Designer

Auto-generate palette colors along a ramp with hue shifting (shadows aren't just darker - they shift toward cool/warm).

**Implementation:** Palette attribute

```json
{
  "type": "palette",
  "name": "skin",
  "ramps": {
    "skin": {
      "base": "#E8B89D",
      "steps": 5,
      "shadow_shift": {"lightness": -15, "hue": 10, "saturation": 5},
      "highlight_shift": {"lightness": 12, "hue": -5, "saturation": -10}
    }
  }
}
```

**Generates tokens:**
- `{skin_2}` (darkest shadow)
- `{skin_1}` (shadow)
- `{skin}` (base)
- `{skin+1}` (highlight)
- `{skin+2}` (brightest highlight)

**Simpler syntax:**
```json
{
  "ramps": {
    "skin": {"base": "#E8B89D", "steps": 3}
  }
}
```
Uses sensible defaults for shift values.

**Multiple ramps:**
```json
{
  "ramps": {
    "skin": {"base": "#E8B89D", "steps": 3},
    "hair": {"base": "#4A3728", "steps": 4},
    "metal": {"base": "#8899AA", "steps": 5, "shadow_shift": {"hue": 20}}
  }
}
```

---

### Frame Tags

**Persona:** Game Developer

Mark frame ranges with semantic names for game engine integration.

**Implementation:** Animation attribute

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

**Export includes tags:**
```bash
pxl render player.pxl --format atlas --output player.json
```

```json
{
  "frames": [...],
  "tags": {
    "idle": {"from": 0, "to": 1, "loop": true},
    "run": {"from": 2, "to": 5, "loop": true}
  }
}
```

**Tag-specific FPS:**
```json
{
  "tags": {
    "idle": {"start": 0, "end": 1, "fps": 4},
    "run": {"start": 2, "end": 5, "fps": 12}
  }
}
```

---

### Atlas Export

**Persona:** Game Developer

Pack multiple sprites into a single texture with coordinate metadata. Game engines load one image instead of hundreds = faster loading.

**Implementation:** CLI export format

**Basic usage:**
```bash
pxl render game.pxl --format atlas -o game_atlas
# Outputs: game_atlas.png + game_atlas.json
```

**Output structure:**

`game_atlas.png` - Single image with all sprites packed:
```
┌────────────────────────┐
│ coin │ player │ enemy  │
├──────┴────────┼────────┤
│    tree       │ chest  │
└───────────────┴────────┘
```

`game_atlas.json` - Coordinate data:
```json
{
  "image": "game_atlas.png",
  "size": [128, 64],
  "frames": {
    "coin": {"x": 0, "y": 0, "w": 16, "h": 16},
    "player": {"x": 16, "y": 0, "w": 32, "h": 32},
    "enemy": {"x": 48, "y": 0, "w": 24, "h": 24},
    "tree": {"x": 0, "y": 32, "w": 48, "h": 32},
    "chest": {"x": 48, "y": 32, "w": 24, "h": 24}
  },
  "animations": {
    "player_walk": {
      "frames": ["player_walk_1", "player_walk_2", "player_walk_3"],
      "fps": 8,
      "tags": {
        "idle": {"from": 0, "to": 1},
        "run": {"from": 2, "to": 5}
      }
    }
  }
}
```

**Packing options:**
```bash
# Max atlas size
pxl render game.pxl --format atlas --max-size 512x512 -o atlas

# Padding between sprites (prevents bleed)
pxl render game.pxl --format atlas --padding 1 -o atlas

# Power-of-two dimensions (required by some engines)
pxl render game.pxl --format atlas --power-of-two -o atlas

# Multiple atlases if needed
pxl render game.pxl --format atlas --max-size 256x256 -o atlas
# Outputs: atlas_0.png, atlas_0.json, atlas_1.png, atlas_1.json, ...
```

**Format variants:**
```bash
# Generic JSON (default)
pxl render game.pxl --format atlas -o out

# Aseprite-compatible
pxl render game.pxl --format atlas-aseprite -o out

# Godot-compatible
pxl render game.pxl --format atlas-godot -o out

# Unity-compatible
pxl render game.pxl --format atlas-unity -o out

# libGDX-compatible
pxl render game.pxl --format atlas-libgdx -o out
```

**Selective export:**
```bash
# Only specific sprites
pxl render game.pxl --format atlas --sprites "player_*,enemy_*" -o characters

# Only animations
pxl render game.pxl --format atlas --animations-only -o anims
```

---

## Medium Priority Features

### Nine-Slice

**Persona:** Game Developer

Scalable sprites where corners stay fixed and edges/center stretch.

**Implementation:** Sprite attribute

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

**Render at size:**
```bash
pxl render button.pxl --nine-slice 64x32 -o button_wide.png
```

**In compositions:**
```json
{
  "layers": [
    {"sprite": "button", "position": [0, 0], "nine_slice_size": [100, 40]}
  ]
}
```

---

### Dithering Patterns

**Persona:** Pixel Artist

Apply dithering patterns for gradients, transparency effects, texture.

**Implementation:** Transform operation

```json
{"type": "sprite", "name": "gradient", "source": "solid", "transform": [
  {"op": "dither", "pattern": "checker", "tokens": ["{dark}", "{light}"], "threshold": 0.5}
]}
```

**Built-in patterns:**
- `checker` - Checkerboard
- `ordered-2x2` - 2x2 Bayer matrix
- `ordered-4x4` - 4x4 Bayer matrix
- `diagonal` - Diagonal lines
- `horizontal` - Horizontal lines
- `noise` - Random (seeded)

**Gradient dither:**
```json
{
  "op": "dither-gradient",
  "direction": "vertical",
  "from": "{sky_light}",
  "to": "{sky_dark}",
  "pattern": "ordered-4x4"
}
```

---

### Hue-Shifted Shadows

**Persona:** Pixel Artist

Auto-generate shadow/highlight colors with hue shifting built into palette.

**Implementation:** Palette attribute (see Color Ramps above - this is the same feature)

**Quick syntax for individual colors:**
```json
{
  "type": "palette",
  "colors": {
    "{skin}": "#E8B89D",
    "{skin_shadow}": {"from": "{skin}", "shift": {"lightness": -20, "hue": 15}},
    "{skin_highlight}": {"from": "{skin}", "shift": {"lightness": 15, "hue": -10}}
  }
}
```

---

### Selective Outline (Sel-out)

**Persona:** Pixel Artist

Outline color varies based on the adjacent fill color, creating softer edges.

**Implementation:** Transform operation

```json
{"transform": [
  {"op": "sel-out", "fallback": "{outline}"}
]}
```

**How it works:**
- Examines each outline pixel's neighboring fill pixels
- Picks a darker/shifted version of the most common neighbor
- Falls back to specified color if can't determine

**With explicit mapping:**
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

---

### Blend Modes

**Persona:** Pixel Artist, Motion Designer

Layer blending for compositions.

**Implementation:** Composition layer attribute

```json
{
  "type": "composition",
  "layers": [
    {"sprite": "background", "position": [0, 0]},
    {"sprite": "shadow", "position": [10, 20], "blend": "multiply", "opacity": 0.5},
    {"sprite": "glow", "position": [5, 5], "blend": "screen"},
    {"sprite": "player", "position": [16, 16]}
  ]
}
```

**Blend modes:**
- `normal` (default)
- `multiply` - Darken
- `screen` - Lighten
- `overlay` - Contrast
- `add` - Additive (glow)
- `subtract` - Subtractive

---

### Onion Skinning

**Persona:** Animator

Preview previous/next frames as transparent overlays.

**Implementation:** CLI preview feature

```bash
pxl show walk_cycle.pxl --onion 2
```

Shows current frame with 2 previous and 2 next frames as ghosts.

**Options:**
- `--onion <count>` - Number of frames before/after
- `--onion-opacity <0-1>` - Ghost opacity (default 0.3)
- `--onion-prev-color <hex>` - Tint for previous frames
- `--onion-next-color <hex>` - Tint for next frames

---

## Lower Priority Features

### Sub-pixel Animation

**Persona:** Motion Designer

Create apparent motion smaller than 1 pixel by shifting colors.

**Implementation:** Transform operation with keyframes

```json
{
  "type": "transform",
  "name": "subpixel-shift",
  "params": ["amount"],
  "keyframes": {
    "subpixel-x": {"expr": "${amount} * sin(t * 3.14159)"}
  }
}
```

**How it works:**
- `subpixel-x: 0.5` means "50% blend toward the right pixel"
- Renderer interpolates colors at boundaries
- Creates smooth apparent motion < 1px

---

### Squash & Stretch

**Persona:** Motion Designer

Deform sprites for impact and bounce.

**Implementation:** Transform operation

```json
{
  "type": "transform",
  "name": "squash",
  "params": ["amount"],
  "ops": [
    {"op": "scale", "x": "${1 + amount}", "y": "${1 - amount}"}
  ]
}
```

**With keyframes:**
```json
{
  "type": "transform",
  "name": "bounce-squash",
  "frames": 8,
  "keyframes": [
    {"frame": 0, "scale-x": 1.0, "scale-y": 1.0},
    {"frame": 3, "scale-x": 0.8, "scale-y": 1.3},
    {"frame": 5, "scale-x": 1.2, "scale-y": 0.8},
    {"frame": 8, "scale-x": 1.0, "scale-y": 1.0}
  ],
  "easing": "ease-out"
}
```

**Note:** Scaling pixel art non-uniformly requires interpolation decisions. Options:
- Nearest neighbor (blocky but crisp)
- Smooth (blurry but fluid)
- Row/column duplication (pixel-art friendly)

---

### Secondary Motion

**Persona:** Motion Designer

Child elements that follow parent with delay/dampening (hair, capes, tails).

**Implementation:** Animation attribute

```json
{
  "type": "animation",
  "name": "run_with_cape",
  "source": "run",
  "attachments": [
    {
      "sprite": "cape",
      "anchor": [8, 4],
      "follow": "parent",
      "delay": 2,
      "damping": 0.7
    }
  ]
}
```

**Parameters:**
- `anchor` - Attachment point on parent
- `delay` - Frames behind parent motion
- `damping` - How much motion is reduced (0-1)
- `spring` - Springy overshoot factor

---

### Particle Systems

**Persona:** Motion Designer

Define particle emitters for effects (sparks, dust, rain).

**Implementation:** New type

```json
{
  "type": "particle",
  "name": "sparkle",
  "sprite": "spark",
  "emitter": {
    "rate": 5,
    "lifetime": [10, 20],
    "velocity": {"x": [-2, 2], "y": [-4, -1]},
    "gravity": 0.2,
    "fade": true,
    "rotation": [0, 360]
  }
}
```

**Use in composition:**
```json
{
  "type": "composition",
  "layers": [
    {"sprite": "gem"},
    {"particle": "sparkle", "position": [8, 8]}
  ]
}
```

---

### Parallax Hints

**Persona:** Game Developer

Depth values for scroll-speed calculation in game engines.

**Implementation:** Composition layer attribute

```json
{
  "type": "composition",
  "name": "scene",
  "layers": [
    {"sprite": "sky", "position": [0, 0], "parallax": 0.1},
    {"sprite": "mountains", "position": [0, 20], "parallax": 0.3},
    {"sprite": "trees", "position": [0, 40], "parallax": 0.7},
    {"sprite": "ground", "position": [0, 56], "parallax": 1.0}
  ]
}
```

**Exported as metadata** for game engine to interpret.

---

### Hit/Hurt Boxes

**Persona:** Game Developer

Collision regions per sprite or per frame.

**Implementation:** Metadata

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

**Per-frame boxes in animations:**
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

---

### Arc Motion Paths

**Persona:** Motion Designer

Motion follows curved paths instead of linear interpolation.

**Implementation:** Keyframe enhancement

```json
{
  "type": "transform",
  "name": "throw-arc",
  "frames": 12,
  "keyframes": [
    {"frame": 0, "shift-x": 0, "shift-y": 0},
    {"frame": 6, "shift-x": 24, "shift-y": -16},
    {"frame": 12, "shift-x": 48, "shift-y": 0}
  ],
  "interpolation": "bezier",
  "path": "arc"
}
```

**Or explicit control points:**
```json
{
  "keyframes": [
    {"frame": 0, "position": [0, 0], "control": [8, -20]},
    {"frame": 12, "position": [48, 0], "control": [40, -20]}
  ]
}
```

---

## Implementation Notes

### What Needs New Attributes

| Feature | Where | Attribute |
|---------|-------|-----------|
| Palette cycling | Animation | `palette_cycle` |
| Color ramps | Palette | `ramps` |
| Frame tags | Animation | `tags` |
| Nine-slice | Sprite | `nine_slice` |
| Blend modes | Composition layer | `blend`, `opacity` |
| Parallax | Composition layer | `parallax` |
| Secondary motion | Animation | `attachments` |
| Hit boxes | Sprite/Animation | `metadata.boxes` |

### What Can Be Transform Operations

- Dithering patterns
- Selective outline
- Sub-pixel animation
- Squash & stretch
- All geometric transforms

### What Needs New Types

- Particle systems (`type: particle`)

### What's CLI Only

- Onion skinning (`pxl show --onion`)

---

## Progressive Implementation

**Phase A:** High-priority features for Pixel Artist & Animator
- Color ramps
- Palette cycling
- Frame tags

**Phase B:** Game developer features
- Nine-slice
- Metadata / hitboxes
- Export format options

**Phase C:** Pixel art polish
- Dithering patterns
- Selective outline
- Blend modes

**Phase D:** Motion designer power features
- Squash & stretch
- Sub-pixel animation
- Secondary motion
- Arc paths

**Phase E:** Advanced systems
- Particle systems

---

## Open Questions

1. **Palette cycling + transforms:** If an animation has both palette cycling and sprite transforms, what's the interaction?

2. **Nine-slice + transforms:** Can you transform a nine-sliced sprite? (Probably: transform first, then nine-slice)

3. **Frame metadata format:** Should per-frame metadata be inline or in a separate structure?

4. **Particle randomness:** How to handle seeded randomness for reproducible particle effects?

5. **Blend mode support in CLI:** Should `pxl render` support blend modes, or compositions-only?

---

## Task Dependency Diagram

```
                      ADVANCED TRANSFORMS TASK FLOW
═══════════════════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Phase 18 Complete (TRF-12)                              │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 1 (Documentation First)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            ATF-1                                    │    │
│  │               Documentation & Format Spec                           │    │
│  │               - Spec all new attributes                             │    │
│  │               - Update format.md                                    │    │
│  │               - Create examples                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 2 (High Priority - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │    ATF-2      │  │    ATF-3      │  │    ATF-4      │  │    ATF-5      │ │
│  │  Color Ramps  │  │   Palette     │  │  Frame Tags   │  │    Atlas      │ │
│  │  (Palette)    │  │   Cycling     │  │  (Animation)  │  │   Export      │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 3 (Game Dev Features - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────┐  ┌─────────────────────────────────┐   │
│  │            ATF-6                │  │            ATF-7                │   │
│  │         Nine-Slice              │  │       Hit/Hurt Boxes            │   │
│  │         (Sprite)                │  │        (Metadata)               │   │
│  └─────────────────────────────────┘  └─────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 4 (Pixel Art Polish - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │    ATF-8      │  │    ATF-9      │  │   ATF-10      │  │   ATF-11      │ │
│  │  Dithering    │  │  Selective    │  │    Blend      │  │    Onion      │ │
│  │  Patterns     │  │   Outline     │  │    Modes      │  │   Skinning    │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 5 (Motion Designer - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │   ATF-12      │  │   ATF-13      │  │   ATF-14      │  │   ATF-15      │ │
│  │   Squash &    │  │  Sub-pixel    │  │  Secondary    │  │  Arc Motion   │ │
│  │   Stretch     │  │  Animation    │  │   Motion      │  │    Paths      │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 6 (Advanced Systems)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                           ATF-16                                    │    │
│  │                      Particle Systems                               │    │
│  │                      (New type: particle)                           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 7 (Testing)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                           ATF-17                                    │    │
│  │                    Comprehensive Test Suite                         │    │
│  │                    (all features)                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Wave 1: ATF-1 (docs first)                    (1 task)                     │
│  Wave 2: ATF-2 + ATF-3 + ATF-4 + ATF-5         (4 tasks in parallel)        │
│  Wave 3: ATF-6 + ATF-7                         (2 tasks in parallel)        │
│  Wave 4: ATF-8 + ATF-9 + ATF-10 + ATF-11       (4 tasks in parallel)        │
│  Wave 5: ATF-12 + ATF-13 + ATF-14 + ATF-15     (4 tasks in parallel)        │
│  Wave 6: ATF-16                                (1 task)                     │
│  Wave 7: ATF-17                                (1 task)                     │
└─────────────────────────────────────────────────────────────────────────────┘

CRITICAL PATH: ATF-1 → ATF-2 → ... → ATF-16 → ATF-17

BEADS CREATION ORDER:
  1. ATF-1 (dep: TRF-12 - Phase 18 complete)
  2. ATF-2 through ATF-16 (dep: ATF-1)
  3. ATF-17 (dep: ATF-2 through ATF-16)
```

---

## Tasks

### Task ATF-1: Documentation & Format Spec

**Wave:** 1 (FIRST - docs before implementation)

Write comprehensive format specification for all Phase 19 features before implementation begins.

**Deliverables:**
- Update `docs/spec/format.md` with:
  - `palette.ramps` attribute syntax
  - `animation.palette_cycle` attribute syntax
  - `animation.tags` attribute syntax
  - `sprite.nine_slice` attribute syntax
  - `composition.layer.blend` and `opacity` attributes
  - `metadata.boxes` for hit/hurt regions
  - `animation.attachments` for secondary motion
  - `particle` type specification
- Create example files demonstrating each feature
- Update `src/prime.rs` with new feature documentation

**Verification:**
```bash
grep "palette_cycle" docs/spec/format.md
grep "nine_slice" docs/spec/format.md
./target/release/pxl prime | grep -i "advanced"
```

**Dependencies:** Task TRF-12 (Phase 18 complete)

---

### Task ATF-2: Color Ramps

**Wave:** 2 (parallel with ATF-3, ATF-4, ATF-5)

Implement automatic color ramp generation with hue-shifted shadows/highlights.

**Deliverables:**
- Add `ramps` attribute to Palette in `src/models.rs`
- Implement ramp generation with configurable shifts
- Generate tokens: `{name_2}`, `{name_1}`, `{name}`, `{name+1}`, `{name+2}`

**Verification:**
```bash
./target/release/pxl render examples/color_ramps.pxl -o output.png
```

**Dependencies:** Task ATF-1

---

### Task ATF-3: Palette Cycling

**Wave:** 2 (parallel with ATF-2, ATF-4, ATF-5)

Implement palette color rotation for water/fire/energy effects.

**Deliverables:**
- Add `palette_cycle` attribute to Animation in `src/models.rs`
- Support single and multiple cycles
- Implement color rotation during render

**Verification:**
```bash
./target/release/pxl render examples/waterfall.pxl -o waterfall.gif
```

**Dependencies:** Task ATF-1

---

### Task ATF-4: Frame Tags

**Wave:** 2 (parallel with ATF-2, ATF-3, ATF-5)

Implement semantic frame range markers for game engine integration.

**Deliverables:**
- Add `tags` attribute to Animation in `src/models.rs`
- Support per-tag FPS and loop settings
- Include tags in atlas export metadata

**Verification:**
```bash
./target/release/pxl render examples/player.pxl --format atlas -o player
cat player.json | jq '.tags'
```

**Dependencies:** Task ATF-1

---

### Task ATF-5: Atlas Export

**Wave:** 2 (parallel with ATF-2, ATF-3, ATF-4)

Implement sprite atlas packing with coordinate metadata.

**Deliverables:**
- Add `--format atlas` to CLI render command
- Implement rectangle packing algorithm
- Support options: `--max-size`, `--padding`, `--power-of-two`
- Support format variants: `atlas-aseprite`, `atlas-godot`, `atlas-unity`

**Verification:**
```bash
./target/release/pxl render game.pxl --format atlas -o game_atlas
ls game_atlas.png game_atlas.json
```

**Dependencies:** Task ATF-1

---

### Task ATF-6: Nine-Slice

**Wave:** 3 (parallel with ATF-7)

Implement scalable sprite slicing for UI elements.

**Deliverables:**
- Add `nine_slice` attribute to Sprite in `src/models.rs`
- Implement nine-slice rendering at arbitrary sizes
- Support `--nine-slice WxH` CLI option

**Verification:**
```bash
./target/release/pxl render button.pxl --nine-slice 64x32 -o button_wide.png
```

**Dependencies:** Task ATF-1

---

### Task ATF-7: Hit/Hurt Boxes

**Wave:** 3 (parallel with ATF-6)

Implement collision region metadata for sprites and animations.

**Deliverables:**
- Add `metadata.boxes` to Sprite in `src/models.rs`
- Add `frame_metadata` to Animation for per-frame boxes
- Include box data in atlas export

**Verification:**
```bash
./target/release/pxl render attack.pxl --format atlas -o attack
cat attack.json | jq '.frames[0].boxes'
```

**Dependencies:** Task ATF-1

---

### Task ATF-8: Dithering Patterns

**Wave:** 4 (parallel with ATF-9, ATF-10, ATF-11)

Implement dither transform operations with various patterns.

**Deliverables:**
- Add `dither` transform operation
- Implement patterns: checker, ordered-2x2, ordered-4x4, diagonal, noise
- Add `dither-gradient` for directional gradients

**Verification:**
```bash
./target/release/pxl transform sprite.pxl --dither checker -o dithered.pxl
```

**Dependencies:** Task ATF-1

---

### Task ATF-9: Selective Outline

**Wave:** 4 (parallel with ATF-8, ATF-10, ATF-11)

Implement color-aware outline generation (sel-out).

**Deliverables:**
- Add `sel-out` transform operation
- Implement neighbor color detection
- Support explicit color mapping

**Verification:**
```bash
./target/release/pxl transform sprite.pxl --sel-out -o outlined.pxl
```

**Dependencies:** Task ATF-1

---

### Task ATF-10: Blend Modes

**Wave:** 4 (parallel with ATF-8, ATF-9, ATF-11)

Implement layer blending for compositions.

**Deliverables:**
- Add `blend` and `opacity` attributes to CompositionLayer
- Implement modes: normal, multiply, screen, overlay, add, subtract

**Verification:**
```bash
./target/release/pxl render composition.pxl -o output.png
```

**Dependencies:** Task ATF-1

---

### Task ATF-11: Onion Skinning

**Wave:** 4 (parallel with ATF-8, ATF-9, ATF-10)

Implement animation preview with frame ghosts.

**Deliverables:**
- Add `--onion N` flag to `pxl show` command
- Render previous/next frames as transparent overlays
- Support `--onion-opacity` and `--onion-prev-color`/`--onion-next-color`

**Verification:**
```bash
./target/release/pxl show walk_cycle.pxl --onion 2
```

**Dependencies:** Task ATF-1

---

### Task ATF-12: Squash & Stretch

**Wave:** 5 (parallel with ATF-13, ATF-14, ATF-15)

Implement scale-based deformation for impact effects.

**Deliverables:**
- Add `scale` transform operation with x/y parameters
- Support keyframe-based squash/stretch animations
- Handle pixel interpolation options

**Verification:**
```bash
./target/release/pxl render bounce.pxl -o bounce.gif
```

**Dependencies:** Task ATF-1

---

### Task ATF-13: Sub-pixel Animation

**Wave:** 5 (parallel with ATF-12, ATF-14, ATF-15)

Implement apparent motion smaller than 1 pixel via color blending.

**Deliverables:**
- Add `subpixel-x` and `subpixel-y` transform properties
- Implement color interpolation at sub-pixel boundaries

**Verification:**
```bash
./target/release/pxl render smooth_motion.pxl -o smooth.gif
```

**Dependencies:** Task ATF-1

---

### Task ATF-14: Secondary Motion

**Wave:** 5 (parallel with ATF-12, ATF-13, ATF-15)

Implement attached elements with delayed/dampened motion.

**Deliverables:**
- Add `attachments` attribute to Animation
- Implement delay and damping physics
- Support spring-based overshoot

**Verification:**
```bash
./target/release/pxl render run_with_cape.pxl -o cape.gif
```

**Dependencies:** Task ATF-1

---

### Task ATF-15: Arc Motion Paths

**Wave:** 5 (parallel with ATF-12, ATF-13, ATF-14)

Implement curved motion interpolation using bezier paths.

**Deliverables:**
- Add `interpolation: bezier` keyframe option
- Add `path: arc` for automatic curve fitting
- Support explicit control points

**Verification:**
```bash
./target/release/pxl render throw_arc.pxl -o throw.gif
```

**Dependencies:** Task ATF-1

---

### Task ATF-16: Particle Systems

**Wave:** 6

Implement particle emitters for effects like sparks, dust, rain.

**Deliverables:**
- Add new `particle` type to format
- Implement emitter with rate, lifetime, velocity, gravity
- Support fade, rotation, and random seed

**Verification:**
```bash
./target/release/pxl render sparkle.pxl -o sparkle.gif
```

**Dependencies:** Task ATF-1

---

### Task ATF-17: Test Suite

**Wave:** 7

Comprehensive tests for all Phase 19 features.

**Deliverables:**
- Unit tests for each new feature
- Integration tests for CLI commands
- Test fixtures for all new format attributes
- Round-trip tests

**Verification:**
```bash
cargo test advanced
cargo test --test cli_integration atlas
cargo test --test cli_integration particle
```

**Dependencies:** Tasks ATF-2 through ATF-16

---

## Verification Summary

```bash
# 1. All existing tests pass
cargo test

# 2. New feature tests pass
cargo test advanced
cargo test palette_cycle
cargo test nine_slice
cargo test particle

# 3. CLI commands work
./target/release/pxl render examples/waterfall.pxl -o waterfall.gif
./target/release/pxl render game.pxl --format atlas -o atlas
./target/release/pxl show walk.pxl --onion 2

# 4. Documentation updated
./target/release/pxl prime | grep -i palette_cycle
grep "particle" docs/spec/format.md
```

---

## Success Criteria

1. All high-priority features work (color ramps, palette cycling, frame tags, atlas export)
2. Game dev features work (nine-slice, hit boxes)
3. Pixel art polish features work (dithering, sel-out, blend modes, onion skinning)
4. Motion designer features work (squash/stretch, sub-pixel, secondary motion, arc paths)
5. Particle systems work
6. All tests pass
7. Documentation complete and accurate
