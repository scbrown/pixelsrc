# Regions & Shapes

Regions define the visual structure of a sprite using geometric primitives. Each region is a named area that maps to a color token from the palette.

## Overview

```json5
{
  type: "sprite",
  name: "hero",
  size: [16, 16],
  palette: "hero",
  regions: {
    outline: { stroke: [0, 0, 16, 16] },
    skin: { fill: "inside(outline)", y: [4, 12] },
    eye: { points: [[5, 6]], symmetric: "x" }
  }
}
```

Unlike the legacy grid format, regions scale with semantic complexity rather than pixel count. A 64x64 sprite takes the same space as an 8x8 sprite if they have similar structure.

## Shape Primitives

### Points

Individual pixels at specific coordinates.

```json5
eye: { points: [[2, 3], [5, 3]] }
```

### Line

Bresenham line between points.

```json5
mouth: { line: [[3, 6], [5, 6]] }

// Multiple segments
crack: { line: [[0, 0], [2, 3], [4, 2]] }
```

Optional: `thickness` (default: 1)

### Rectangle

Filled rectangle.

```json5
body: { rect: [2, 4, 12, 8] }  // [x, y, width, height]
```

Optional: `round` (corner radius)

### Stroke

Rectangle outline (unfilled).

```json5
outline: { stroke: [0, 0, 16, 16] }
```

Optional: `thickness`, `round`

### Ellipse

Filled ellipse.

```json5
head: { ellipse: [8, 4, 6, 4] }  // [cx, cy, rx, ry]
```

### Circle

Shorthand for equal-radius ellipse.

```json5
dot: { circle: [4, 4, 2] }  // [cx, cy, r]
```

### Polygon

Filled polygon from vertices. **Winding order doesn't matter** - clockwise or counter-clockwise produces identical results.

```json5
hair: {
  polygon: [[4, 0], [12, 0], [14, 4], [2, 4]]
}
```

> **Tip**: Keep polygons simple (3-5 vertices). Complex polygons with 6+ vertices may cause fill artifacts. Use `union` of simpler shapes instead.

### Path

SVG-lite path syntax.

```json5
complex: {
  path: "M2,0 L6,0 L8,2 L8,6 L6,8 L2,8 L0,6 L0,2 Z"
}
```

**Supported commands:**
- `M x,y` - Move to
- `L x,y` - Line to
- `H x` - Horizontal line to
- `V y` - Vertical line to
- `Z` - Close path

Curves (C, Q, A) are not supported as they don't make sense for pixel art.

### Fill

Flood fill inside a boundary.

```json5
skin: { fill: "inside(outline)" }
```

Optional: `seed: [x, y]` (starting point, auto-detected if omitted)

## Compound Operations

### Union

Combine multiple shapes.

```json5
hair: {
  union: [
    { rect: [2, 0, 12, 2] },
    { rect: [0, 2, 16, 2] }
  ]
}
```

### Subtract

Remove shapes from a base.

```json5
face: {
  base: { rect: [2, 4, 12, 8] },
  subtract: [
    { points: [[5, 6], [10, 6]] }  // eye holes
  ]
}
```

Or using token references:

```json5
skin: {
  fill: "inside(outline)",
  except: ["eye", "mouth"]
}
```

### Intersect

Keep only overlapping area.

```json5
visor: {
  intersect: [
    { rect: [2, 4, 12, 4] },
    { fill: "inside(helmet)" }
  ]
}
```

## Modifiers

### Symmetry

Auto-mirror across axis.

```json5
eye: {
  points: [[4, 6]],
  symmetric: "x"  // mirrors to [11, 6] for 16-wide sprite
}
```

**Values:**
- `"x"` - Horizontal mirror (around canvas center)
- `"y"` - Vertical mirror (around canvas center)
- `"xy"` - Both axes (4-way symmetry)
- `8` - Mirror around specific x-coordinate

### Range Constraints

Limit region to specific rows or columns.

```json5
hair: {
  fill: "inside(outline)",
  y: [0, 4]  // rows 0-4 inclusive
}

"left-arm": {
  fill: "inside(outline)",
  x: [0, 8]  // columns 0-8 inclusive
}
```

### Containment Validation

Validate that region stays within another.

```json5
pupil: {
  points: [[4, 6]],
  within: "eye"  // compiler validates this
}
```

Note: `within` is a validation constraint checked after all regions are resolved. It's distinct from `fill: "inside(X)"` which is a pixel-affecting operation.

### Adjacency Validation

Ensure region touches another.

```json5
shadow: {
  fill: "inside(outline)",
  "adjacent-to": "skin",
  y: [10, 14]
}
```

### Z-Order

Explicit render order (default: definition order).

```json5
detail: {
  points: [[8, 8]],
  z: 100  // renders on top
}
```

## Transform Modifiers

### Repeat

Tile a shape.

```json5
bricks: {
  rect: [0, 0, 4, 2],
  repeat: [8, 16],
  spacing: [1, 1],
  "offset-alternate": true
}
```

### Geometric Transform

Apply rotation, translation, scale.

```json5
sword: {
  line: [[0, 0], [0, 8]],
  transform: "rotate(45deg) translate(12, 4)"
}
```

**Supported transforms:**
- `translate(x, y)`
- `rotate(angle)` - supports `deg` or `rad`
- `scale(x, y)` or `scale(n)`
- `flip-x`, `flip-y`

### Jitter

Controlled randomness.

```json5
grass: {
  points: [[0, 15], [4, 15], [8, 15], [12, 15]],
  jitter: { y: [-2, 0] },
  seed: 42
}
```

## Auto-Generation

### Auto-Outline

Generate outline around a region.

```json5
outline: {
  "auto-outline": "body",
  thickness: 1
}
```

### Auto-Shadow

Generate drop shadow.

```json5
shadow: {
  "auto-shadow": "body",
  offset: [1, 1]
}
```

### Background

Shorthand to fill all unoccupied pixels.

```json5
_: "background"
```

## Region Resolution Order

Regions are processed in two passes:

1. **Shape resolution** (definition order): Each region's pixels are computed. Pixel-affecting operations (`fill: "inside(X)"`, `except: [X]`, `auto-outline: X`) require X to be defined earlier.

2. **Validation** (after all resolved): Constraints (`within`, `adjacent-to`) are checked. These can reference any region.

This enables streaming while catching errors:
```json5
regions: {
  outline: { stroke: [0, 0, 16, 16] },     // defined first
  skin: { fill: "inside(outline)" },        // OK: outline exists
  eye: { rect: [5, 6, 2, 2] },
  pupil: { points: [[6, 7]], within: "eye" } // OK: within is validation-only
}
```

**Error** (forward reference in pixel-affecting operation):
```json5
regions: {
  skin: { fill: "inside(outline)" },  // ERROR: outline not yet defined
  outline: { stroke: [0, 0, 16, 16] }
}
```

## Visual Reference

Here's how common shapes map to coordinates:

```
CIRCLE: circle: [cx, cy, r]
Example: circle: [16, 12, 8]

       0   4   8  12  16  20  24  28  32
     4 │         ●●●●●●●               │  ← radius 8
     8 │       ●●       ●●             │
    12 │      ●    (16,12)  ●          │  ← center
    16 │       ●●       ●●             │
    20 │         ●●●●●●●               │

RECTANGLE: rect: [x, y, w, h]
Example: rect: [8, 16, 16, 8]

       0   4   8  12  16  20  24  28  32
    16 │        ████████████████       │  ← y=16 (top)
    20 │        ████████████████       │
    24 │        ████████████████       │  ← y=24 (top + height)
               ↑               ↑
             x=8            x=24 (x + width)

POLYGON (Triangle): polygon: [[16,4], [4,28], [28,28]]

       0   4   8  12  16  20  24  28  32
     4 │               ▲ (16,4)        │  ← tip
    12 │             █████             │
    20 │           █████████           │
    28 │ (4,28)  █████████████ (28,28) │  ← base

COMBINED: Skull silhouette (circle + rect + polygon)

       0   4   8  12  16  20  24  28  32
     4 │         ●●●●●●●               │
     8 │       ●●●●●●●●●●●             │  ← circle (cranium)
    12 │      ●●●●●●●●●●●●●            │
    16 │       ●●●●●●●●●●●             │
    20 │        ████████████████       │
    24 │        ████████████████       │  ← rect (mid-face)
    28 │        ████████████████       │
    32 │         ██████████████        │
    36 │          ████████████         │  ← polygon (jaw taper)
```

> **Design tip**: Start with 2-3 basic shapes (circle + rect + polygon) for silhouettes. Get proportions right before adding detail.
