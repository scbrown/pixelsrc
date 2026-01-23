# Pixelsrc Structured Format Specification

> **STATUS: SUPERSEDED**
>
> This document has been merged into the unified Format v2 specification.
> See: [`format2.md`](./format2.md)
>
> Key concepts from this document that made it into Format v2:
> - Structured regions replace grid entirely (no grid format in v2)
> - All shape primitives (points, line, rect, stroke, ellipse, circle, polygon, path, fill)
> - Compound operations (union, subtract, intersect)
> - Modifiers (symmetric, repeat, transform, jitter)
> - Constraints renamed: `inside` → `within` (validation), `fill: "inside(X)"` (pixel-affecting)
> - Auto-generation (auto-outline, auto-shadow, background)
> - Forward-reference requirement for pixel-affecting operations
>
> Key changes from this document:
> - Token syntax simplified: `{token}` → `token` (no braces)
> - `format: "structured"` discriminator removed (only one format)
> - `role` removed from RegionDef (roles defined in Palette only)
> - Symmetric syntax: `[4]` → `4` (number, not array)

---

## Original Overview (Historical)

The structured format is an alternative to grid-based sprite definition that describes **semantic regions** rather than individual pixels. It mirrors how artists actually work: outline first, block in regions, add details, refine.

---

## What Made It Into Format v2 MVP

### Core Structure

```json5
{
  type: "sprite",
  name: "hero",
  size: [16, 16],
  palette: "hero_palette",
  regions: {
    outline: { stroke: [0, 0, 16, 16] },
    skin: { fill: "inside(outline)", y: [4, 12] },
    eye: { points: [[5, 6]], symmetric: "x" }
  }
}
```

Note: No `format` field needed (only structured format exists in v2).

### Shape Primitives (All Included)

| Primitive | Syntax | MVP |
|-----------|--------|-----|
| `points` | `[[x,y], ...]` | Yes |
| `line` | `[[x1,y1], [x2,y2], ...]` | Yes |
| `rect` | `[x, y, w, h]` | Yes |
| `stroke` | `[x, y, w, h]` | Yes |
| `ellipse` | `[cx, cy, rx, ry]` | Yes |
| `circle` | `[cx, cy, r]` | Yes |
| `polygon` | `[[x,y], ...]` | Yes |
| `path` | `"M0,0 L5,5 Z"` | Yes |
| `fill` | `"inside(token)"` | Yes |

### Compound Operations (All Included)

- `union: [...]`
- `base` + `subtract: [...]`
- `intersect: [...]`
- `except: [...]` (shorthand for subtract)

### Modifiers (All Included)

- `symmetric: "x" | "y" | "xy" | number`
- `repeat: [nx, ny]` with `spacing`, `offset-alternate`
- `transform: "rotate(45deg) translate(10, 5)"`
- `jitter: { x: [min, max], y: [min, max] }` with `seed`
- `z: number` (render order)
- `round: number` (corner radius)
- `thickness: number` (for line/stroke)

### Constraints

Renamed for clarity:
- `within: "token"` - validation constraint (was `inside`)
- `adjacent-to: "token"` - validation constraint
- `x: [start, end]` - range constraint
- `y: [start, end]` - range constraint

### Auto-Generation (All Included)

- `auto-outline: "token"`
- `auto-shadow: "token"` with `offset`
- `_: "background"` shorthand

### Region Resolution Order

Added explicit two-pass model:
1. Shape resolution (definition order) - pixel-affecting ops require forward definition
2. Validation (after all resolved) - constraints can reference any region

---

## What Changed

### Token Syntax Simplified

```json5
// This document (original)
"{outline}": { "stroke": [0, 0, 8, 8] }
"fill": "inside({outline})"

// Format v2
outline: { stroke: [0, 0, 8, 8] }
fill: "inside(outline)"
```

### No Format Discriminator

```json5
// This document (original)
{ type: "sprite", format: "structured", ... }

// Format v2 (only one format)
{ type: "sprite", ... }
```

### Roles in Palette Only

```json5
// This document (original)
"{eye}": { points: [[2, 3]], role: "anchor" }

// Format v2 (roles in Palette)
// RegionDef has no role field
// Palette.roles: { eye: "anchor" }
```

### Symmetric Syntax

```json5
// This document (original)
symmetric: [4]  // array

// Format v2
symmetric: 4    // number
```

---

## What Was Deferred

### Procedural Shapes
```json5
"{terrain}": { procedural: "noise", params: { octaves: 3 } }
```

### Shape Libraries
```json5
"{head}": { use: "primitives/humanoid-head", scale: 0.5 }
```

### Expressions
```json5
"{bar}": { rect: [0, 0, "var(--health) * 8", 2] }
```

---

## See Also

- [Format v2 Specification](./format2.md) - Unified spec
- [Phase 24 Tasks](./tasks/phase24.md) - Implementation plan
