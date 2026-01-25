# Pixelsrc Format Specification

**Version:** 2.0.0

---

## Overview

Pixelsrc is a structured text format for defining pixel art using geometric regions rather than pixel grids. Sprites are described semantically, enabling AI-optimized generation that scales with complexity rather than pixel count.

**File Extension:** `.pxl`

**Format:** JSON5 (comments, trailing commas, unquoted keys)

**Design Philosophy:** Lenient by default, strict when requested. When GenAI makes small mistakes, fill the gaps and keep going.

---

## Key Concepts

### Structured Regions vs Grid

The v2 format replaces pixel grids with geometric regions:

```json5
// v1 (removed): pixel-by-pixel grid
// "grid": ["{o}{o}{o}", "{o}{s}{o}", "{o}{o}{o}"]

// v2: semantic regions
regions: {
  outline: { stroke: [0, 0, 3, 3] },
  skin: { fill: "inside(outline)" }
}
```

**Advantages:**
- Scales with semantic complexity, not pixel count
- 64x64 sprite takes same space as 8x8 with similar structure
- Easy to edit (change a number, not rewrite rows)
- AI-optimized (describe intent, compiler resolves pixels)

### Token System

Tokens are named color identifiers defined in palettes. In v2, tokens are referenced directly without braces:

```json5
{
  type: "palette",
  colors: {
    _: "transparent",      // underscore for transparency
    skin: "#FFD5B4",       // semantic names
    dark_hair: "#4A3728"   // underscores for multi-word
  }
}
```

Regions use token names directly:
```json5
regions: {
  skin: { fill: "inside(outline)" },  // token = region name
  dark_hair: { rect: [2, 0, 12, 4] }
}
```

---

## Object Types

### Palette

Defines named color tokens, semantic roles, and relationships.

```json5
{
  type: "palette",
  name: "string (required)",
  colors: {
    token: "#RRGGBB | #RRGGBBAA | css-color"
  },
  roles: {
    token: "boundary | anchor | fill | shadow | highlight"
  },
  relationships: {
    token: { type: "relationship-type", target: "token" }
  }
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"palette"` |
| name | Yes | Unique identifier, referenced by sprites |
| colors | Yes | Map of token → color |
| roles | No | Map of token → semantic role |
| relationships | No | Map of token → relationship definition |

**Color Formats:**
- `#RGB` → expands to `#RRGGBB`
- `#RGBA` → expands to `#RRGGBBAA`
- `#RRGGBB` → fully opaque
- `#RRGGBBAA` → with alpha channel
- CSS colors: `red`, `transparent`, `rgb()`, `hsl()`, `oklch()`
- CSS variables: `var(--name)`, `var(--name, fallback)`
- CSS functions: `color-mix(in oklch, color 70%, black)`

**Roles:**
| Role | Meaning | Transform Behavior |
|------|---------|-------------------|
| `boundary` | Edge-defining (outlines) | High priority, preserve connectivity |
| `anchor` | Critical details (eyes) | Must survive transforms (min 1px) |
| `fill` | Interior mass (skin, clothes) | Can shrink, low priority |
| `shadow` | Depth indicators | Derives from parent |
| `highlight` | Light indicators | Derives from parent |

**Relationship Types:**
| Type | Meaning |
|------|---------|
| `derives-from` | Color derived from another token |
| `contained-within` | Spatially inside another region |
| `adjacent-to` | Must touch specified region |
| `paired-with` | Symmetric relationship |

**Example:**
```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    eye: "#4169E1",
  },
  roles: {
    outline: "boundary",
    skin: "fill",
    eye: "anchor",
    "skin-shadow": "shadow",
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" },
  },
}
```

---

### Sprite

Defines a pixel art image using named regions.

```json5
{
  type: "sprite",
  name: "string (required)",
  size: [width, height],
  palette: "string (required)",
  regions: {
    token: { shape_definition }
  }
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"sprite"` |
| name | Yes | Unique identifier |
| size | Yes | `[width, height]` in pixels |
| palette | Yes | Palette name to use for colors |
| regions | Yes | Map of token names to region definitions |
| background | No | Token to fill empty pixels (default: `_`) |
| origin | No | Anchor point `[x, y]` for transforms |
| metadata | No | Custom data passthrough |
| state-rules | No | Name of state rules to apply |

**Example:**
```json5
{
  type: "sprite",
  name: "coin",
  size: [8, 8],
  palette: "gold",
  regions: {
    _: "background",
    outline: { stroke: [1, 1, 6, 6], round: 2 },
    gold: { fill: "inside(outline)" },
    shine: { points: [[3, 3], [4, 2]] },
  },
}
```

---

### Region Definitions

Regions define shapes using geometric primitives.

#### Shape Primitives

| Shape | Syntax | Description |
|-------|--------|-------------|
| `points` | `[[x, y], ...]` | Individual pixels |
| `line` | `[[x1, y1], [x2, y2], ...]` | Bresenham line segments |
| `rect` | `[x, y, w, h]` | Filled rectangle |
| `stroke` | `[x, y, w, h]` | Rectangle outline |
| `ellipse` | `[cx, cy, rx, ry]` | Filled ellipse |
| `circle` | `[cx, cy, r]` | Filled circle |
| `polygon` | `[[x, y], ...]` | Filled polygon |
| `path` | `"M x,y L x,y ..."` | SVG-lite path (M, L, H, V, Z) |
| `fill` | `"inside(token)"` | Flood fill inside boundary |

#### Modifiers

| Modifier | Value | Description |
|----------|-------|-------------|
| `symmetric` | `"x"`, `"y"`, `"xy"`, or number | Mirror across axis |
| `x` | `[min, max]` | Limit to column range |
| `y` | `[min, max]` | Limit to row range |
| `z` | number | Render order (higher = on top) |
| `round` | number | Corner radius for rect/stroke |
| `thickness` | number | Line/stroke thickness |
| `within` | `"token"` | Validation: must be inside token |
| `adjacent-to` | `"token"` | Validation: must touch token |
| `except` | `["token", ...]` | Subtract these tokens' pixels |

#### Compound Operations

```json5
// Union: combine shapes
hair: {
  union: [
    { rect: [2, 0, 12, 2] },
    { rect: [0, 2, 16, 2] }
  ]
}

// Subtract: remove from base
face: {
  base: { rect: [2, 4, 12, 8] },
  subtract: [{ points: [[5, 6]] }]
}

// Intersect: keep overlap only
visor: {
  intersect: [
    { rect: [2, 4, 12, 4] },
    { fill: "inside(helmet)" }
  ]
}
```

#### Transform Modifiers

```json5
// Repeat/tile
bricks: {
  rect: [0, 0, 4, 2],
  repeat: [8, 16],
  spacing: [1, 1],
  "offset-alternate": true
}

// Geometric transform
sword: {
  line: [[0, 0], [0, 8]],
  transform: "rotate(45deg) translate(12, 4)"
}

// Jitter (controlled randomness)
grass: {
  points: [[0, 15], [4, 15], [8, 15]],
  jitter: { y: [-2, 0] },
  seed: 42
}
```

#### Auto-Generation

```json5
// Auto-outline around a region
outline: { "auto-outline": "body", thickness: 1 }

// Auto-shadow
shadow: { "auto-shadow": "body", offset: [1, 1] }

// Background (fill unoccupied pixels)
_: "background"
```

---

### State Rules

Define visual states without separate sprite definitions.

```json5
{
  type: "state_rules",
  name: "string (required)",
  rules: {
    "selector": { effect }
  }
}
```

**Selectors:**
| Selector | Example | Meaning |
|----------|---------|---------|
| `[token=name]` | `[token=eye]` | Exact token match |
| `[token*=str]` | `[token*=skin]` | Token contains substring |
| `[role=type]` | `[role=boundary]` | Match by role |
| `.state` | `.damaged` | Match when state class active |

**Effects:**
| Effect | Example | Description |
|--------|---------|-------------|
| `filter` | `"brightness(2)"` | CSS filter |
| `animation` | `"flash 0.1s 3"` | CSS animation |
| `opacity` | `0.5` | Transparency |

**Example:**
```json5
{
  type: "state_rules",
  name: "combat",
  rules: {
    ".damaged [token]": {
      filter: "brightness(2)",
      animation: "flash 0.1s 3"
    },
    ".poisoned [token=skin]": {
      filter: "hue-rotate(80deg)"
    },
    "[role=boundary]": {
      filter: "drop-shadow(0 0 1px black)"
    }
  }
}
```

---

### Animation

Defines a sequence of sprites as an animation.

```json5
{
  type: "animation",
  name: "string (required)",
  frames: ["sprite_name", ...],
  duration: number,
  loop: boolean
}
```

**Fields:**
| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| type | Yes | - | Must be `"animation"` |
| name | Yes | - | Unique identifier |
| frames | Yes | - | Array of sprite names |
| duration | No | 100 | Milliseconds per frame |
| loop | No | true | Whether animation loops |

---

### Variant

Defines a color variation of an existing sprite.

```json5
{
  type: "variant",
  name: "string (required)",
  base: "string (required)",
  palette: { token: "#color" }
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"variant"` |
| name | Yes | Unique identifier for this variant |
| base | Yes | Name of the sprite to derive from |
| palette | Yes | Color overrides |

---

### Composition

Layers multiple sprites onto a canvas.

```json5
{
  type: "composition",
  name: "string (required)",
  size: [width, height],
  layers: [
    { sprite: "name", position: [x, y] }
  ]
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"composition"` |
| name | Yes | Unique identifier |
| size | Yes | Canvas size `[width, height]` |
| layers | Yes | Array of layer definitions |

**Layer Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| sprite | Yes | Sprite name to render |
| position | No | Position `[x, y]` (default: `[0, 0]`) |
| blend | No | Blend mode (default: `"normal"`) |
| opacity | No | Layer opacity 0.0-1.0 |

---

## Region Resolution Order

Regions are processed in two passes:

1. **Shape resolution** (definition order): Each region's pixels are computed. Pixel-affecting operations require forward definitions.

2. **Validation** (after all resolved): Constraints (`within`, `adjacent-to`) are checked.

**Forward reference rules:**
- `fill: "inside(X)"` - X must be defined earlier
- `except: [X]` - X must be defined earlier
- `auto-outline: X` - X must be defined earlier
- `within: X` - X can be defined anywhere (validation-only)
- `adjacent-to: X` - X can be defined anywhere (validation-only)

---

## Error Handling

### Lenient Mode (Default)

| Error | Behavior |
|-------|----------|
| Unknown token | Render as magenta `#FF00FF` |
| Region outside canvas | Clip with warning |
| Forward reference in fill | Error |
| Duplicate name | Last definition wins |
| Invalid color | Use magenta placeholder |
| Missing palette | All regions render white |

### Strict Mode (`--strict`)

All warnings become errors. Processing stops at first error with non-zero exit code.

---

## Stream Processing

1. Objects are parsed as complete JSON5 values
2. Objects are processed in order of appearance
3. Palettes must be defined before sprites that reference them
4. Regions within a sprite must define dependencies before dependents

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (lenient: may have warnings) |
| 1 | Error (strict: any warning; lenient: fatal error) |
| 2 | Invalid arguments |

---

## Complete Example

```json5
// hero.pxl - Complete character definition
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    hair: "#8B4513",
    eye: "#4169E1",
    pupil: "#000000",
    shirt: "#E74C3C",
  },
  roles: {
    outline: "boundary",
    eye: "anchor",
    pupil: "anchor",
    skin: "fill",
    hair: "fill",
    shirt: "fill",
    "skin-shadow": "shadow",
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" },
    pupil: { type: "contained-within", target: "eye" },
  },
}

{
  type: "sprite",
  name: "hero",
  size: [16, 24],
  palette: "hero",
  regions: {
    // Background
    _: "background",

    // Head outline
    "head-outline": { stroke: [4, 0, 8, 10], round: 2 },

    // Hair (top of head)
    hair: { fill: "inside(head-outline)", y: [0, 4] },

    // Face
    skin: {
      fill: "inside(head-outline)",
      y: [4, 10],
      except: ["eye", "pupil"],
    },

    // Eyes (symmetric)
    eye: { rect: [5, 5, 2, 2], symmetric: "x" },

    // Pupils
    pupil: {
      points: [[6, 6]],
      symmetric: "x",
      within: "eye",
    },

    // Body outline
    "body-outline": { stroke: [3, 10, 10, 14] },

    // Shirt
    shirt: { fill: "inside(body-outline)" },

    // Shadow on skin
    "skin-shadow": {
      fill: "inside(head-outline)",
      y: [8, 10],
      "adjacent-to": "skin",
    },
  },
}
```

---

## Version History

| Version | Changes |
|---------|---------|
| 2.0.0 | Structured regions, JSON5, semantic metadata, state rules |
| 0.3.0 | ATF features (removed in v2) |
| 0.2.0 | Multi-line JSON, `.pxl` extension |
| 0.1.0 | Initial draft |
