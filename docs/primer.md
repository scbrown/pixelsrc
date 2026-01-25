# Pixelsrc Primer

A concise guide for AI systems generating pixelsrc content.

## What is Pixelsrc?

Pixelsrc is a GenAI-native pixel art format. It's text-based JSON5 that you generate, then `pxl render` converts to PNG/GIF.

**Why Pixelsrc?**
- No binary data - pure text generation
- Semantic regions - describe shapes, not pixels
- Context-efficient - 64x64 sprites take same space as 8x8
- Lenient by default - small mistakes get corrected

## Format Quick Reference

### Object Types

| Type | Purpose | Required Fields |
|------|---------|-----------------|
| `palette` | Define named colors | `name`, `colors` |
| `sprite` | Pixel image via regions | `name`, `size`, `palette`, `regions` |
| `animation` | Frame sequence | `name`, `frames` |
| `composition` | Layer sprites | `name`, `size`, `layers` |

### Palette

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    skin: "#FFD5B4",
    hair: "#8B4513",
  },
}
```

- Token names: `_`, `skin`, `hair` (no braces in v2)
- Color format: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, or CSS colors
- `_` is the conventional transparent token

### Palette with Semantic Metadata

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
    outline: "boundary",  // Edge-defining
    skin: "fill",         // Interior mass
    eye: "anchor",        // Critical detail
    "skin-shadow": "shadow",
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" },
  },
}
```

- **Roles**: `boundary`, `anchor`, `fill`, `shadow`, `highlight`
- **Relationships**: `derives-from`, `contained-within`, `adjacent-to`, `paired-with`

### Sprite

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

- `size`: required `[width, height]`
- `palette`: name of previously defined palette
- `regions`: map of token names to shape definitions

### Shape Primitives

| Shape | Syntax | Example |
|-------|--------|---------|
| Points | `points: [[x, y], ...]` | `{ points: [[5, 6], [10, 6]] }` |
| Line | `line: [[x1, y1], [x2, y2], ...]` | `{ line: [[0, 0], [8, 8]] }` |
| Rectangle | `rect: [x, y, w, h]` | `{ rect: [2, 4, 12, 8] }` |
| Stroke | `stroke: [x, y, w, h]` | `{ stroke: [0, 0, 16, 16] }` |
| Circle | `circle: [cx, cy, r]` | `{ circle: [8, 8, 4] }` |
| Ellipse | `ellipse: [cx, cy, rx, ry]` | `{ ellipse: [8, 8, 6, 4] }` |
| Polygon | `polygon: [[x, y], ...]` | `{ polygon: [[0, 0], [4, 0], [2, 4]] }` |
| Fill | `fill: "inside(token)"` | `{ fill: "inside(outline)" }` |

### Modifiers

```json5
regions: {
  // Symmetry - auto-mirror
  eye: { points: [[5, 6]], symmetric: "x" },

  // Range constraints
  hair: { fill: "inside(outline)", y: [0, 4] },

  // Subtract other regions
  skin: { fill: "inside(outline)", except: ["eye"] },

  // Validation constraints
  pupil: { points: [[6, 6]], within: "eye" },
}
```

### Animation

```json5
{
  type: "animation",
  name: "walk",
  frames: ["walk1", "walk2", "walk3"],
  duration: 100,
  loop: true,
}
```

### Composition

```json5
{
  type: "composition",
  name: "scene",
  size: [64, 64],
  layers: [
    { sprite: "background", position: [0, 0] },
    { sprite: "hero", position: [24, 32] },
  ],
}
```

## Complete Example

A simple 8x8 coin sprite:

```json5
// coin.pxl
{
  type: "palette",
  name: "coin",
  colors: {
    _: "transparent",
    outline: "#8B6914",
    gold: "#FFD700",
    shine: "#FFFACD",
  },
  roles: {
    outline: "boundary",
    gold: "fill",
    shine: "highlight",
  },
}

{
  type: "sprite",
  name: "coin",
  size: [8, 8],
  palette: "coin",
  regions: {
    _: "background",
    outline: { stroke: [1, 1, 6, 6], round: 2 },
    gold: { fill: "inside(outline)" },
    shine: { points: [[3, 2], [4, 3]] },
  },
}
```

**Key observations:**
- JSON5 syntax (comments, trailing commas, unquoted keys)
- Semantic token names: `outline`, `gold`, `shine`
- Regions define shapes, not pixel grids
- Fill references earlier-defined outline

## Best Practices

### DO

1. **Use semantic token names**
   - Good: `skin`, `outline`, `hair_highlight`
   - Bad: `c1`, `a`, `color_1`

2. **Define outline first, then fill**
   - Outline region must exist before `fill: "inside(outline)"`

3. **Use symmetry for symmetric features**
   - `symmetric: "x"` for eyes, arms, etc.

4. **Keep sprites small**
   - 8x8, 16x16, 32x32 are common
   - Larger than 64x64 is unusual for pixel art

5. **Use `_` for transparency**
   - Convention throughout pixelsrc ecosystem
   - Map to `"transparent"`

6. **Use roles for semantic meaning**
   - `boundary` for outlines
   - `anchor` for critical details like eyes
   - `fill` for large areas

### DON'T

1. **Don't use pixel coordinates directly**
   - Use shapes: `rect`, `circle`, `fill`
   - Not individual pixel coordinates

2. **Don't forward-reference in fill operations**
   - Wrong: `skin: { fill: "inside(outline)" }` then `outline: { ... }`
   - Right: `outline: { ... }` then `skin: { fill: "inside(outline)" }`

3. **Don't use single-character tokens**
   - Bad: `a`, `b`, `1`
   - Semantic names are more maintainable

4. **Don't generate huge sprites**
   - Keep under 64x64 for typical pixel art
   - Large sprites lose the pixel art aesthetic

## Common Mistakes

### Forward Reference

**Wrong:**
```json5
regions: {
  skin: { fill: "inside(outline)" },  // ERROR: outline not defined yet
  outline: { stroke: [0, 0, 16, 16] }
}
```

**Right:**
```json5
regions: {
  outline: { stroke: [0, 0, 16, 16] },  // Define first
  skin: { fill: "inside(outline)" }     // Then reference
}
```

### Missing Palette Definition

**Wrong:**
```json5
{ type: "sprite", palette: "hero", ... }
// No palette named "hero" defined before this
```

**Right:**
```json5
{ type: "palette", name: "hero", colors: { ... } }
{ type: "sprite", palette: "hero", ... }
```

### Size Omitted

**Wrong:**
```json5
{
  type: "sprite",
  name: "coin",
  // size is required in v2!
  regions: { ... }
}
```

**Right:**
```json5
{
  type: "sprite",
  name: "coin",
  size: [8, 8],  // Required
  regions: { ... }
}
```

## Commands

### Rendering

```bash
# Render all sprites to PNG
pxl render input.pxl

# Render specific sprite
pxl render input.pxl --sprite hero

# Render with scaling (2x, 4x, etc.)
pxl render input.pxl --scale 4

# Render animation as GIF
pxl render input.pxl --gif

# Strict mode (fail on warnings)
pxl render input.pxl --strict
```

### Validation & Analysis

```bash
# Validate file
pxl validate input.pxl

# Validate with strict mode
pxl validate input.pxl --strict

# Show sprite with roles annotated
pxl show input.pxl --roles

# Analyze structure
pxl analyze input.pxl
```

### Import

```bash
# Import PNG to pixelsrc (structured format)
pxl import image.png -o output.pxl

# Import with role inference
pxl import image.png --analyze -o output.pxl
```

## Workflow

1. **Generate** - Create JSON5 with palette and sprite definitions
2. **Validate** - Run `pxl validate file.pxl` to check for errors
3. **Render** - Run `pxl render file.pxl` to create PNG
4. **Iterate** - Fix any issues and re-render

## Tips for Better Sprites

1. **Start with outline** - Define the boundary stroke first
2. **Fill inside** - Use `fill: "inside(outline)"` for interior
3. **Add details** - Eyes, highlights using points and small shapes
4. **Use symmetry** - `symmetric: "x"` for bilateral features
5. **Layer shadows** - Define shadow regions with y-constraints
6. **Test at 1x scale** - Ensure readability at native size

## Artisan Workflow: Autonomous Art Iteration

For autonomous art generation (without human feedback each step), use component-based iteration:

### Phases

1. **Foundation** - Define components, criteria, gather references, create v0
2. **Component Iteration** - For each component (eyes, mouth, hair, etc.):
   - Generate 2-3 variants with different approaches
   - Evaluate against criteria (readability, clarity, style match)
   - Promote highest scorer, log reasoning
   - Repeat until criteria satisfied (max 5-10 rounds)
3. **Integration** - Compose all components, evaluate as whole:
   - If weak component identified → drill back to that component
   - If composition issue → adjust layering/z-order
   - Repeat until integration criteria satisfied
4. **Submit** - Final render + evolution log + comparison sheet

### Evaluation Criteria

| Criterion | Test |
|-----------|------|
| Readability | Can you tell what it is at 1x? |
| Shape clarity | Is silhouette clean when filled solid? |
| Color coherence | All colors from defined palette? |
| Technical quality | Clean lines? No orphan pixels? |
| Style match | Does it match reference? |

### Variant Strategies

- **Contrast** - Vary light/dark ratio
- **Proportion** - Vary size/shape ratios
- **Detail level** - More vs. fewer details
- **Color temperature** - Warmer vs. cooler
- **Line weight** - Thicker vs. thinner outlines

### Logging

For each iteration, record: approach tried, scores, winner, reason, next focus.
This prevents repeating failed approaches and provides context for review.
