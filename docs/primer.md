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

A composition places sprites on a canvas with character maps. `sprites` gives each sprite a one-character key; each
layer's `map` is a grid of those keys, one character per cell of `cell_size` pixels. A sprite is drawn with its
top-left corner at the top-left of the cell that holds its key. Layers draw first to last, so later layers cover
earlier ones.

```json5
{
  type: "composition",
  name: "scene",
  size: [32, 32],
  cell_size: [8, 8],
  sprites: { ".": null, S: "sky", G: "grass", T: "tree", H: "hero" },
  layers: [
    { name: "sky", map: ["S...", "....", "....", "...."] },
    { name: "ground", map: ["....", "....", "....", "GGGG"] },
    { name: "things", map: ["....", "....", ".T.H", "...."] },
  ],
}
```

- `sprites` is required. Map a key to `null` for an empty cell.
- `cell_size` defaults to `[1, 1]`. Pick the largest cell that still lets you place things where you need them.
- A sprite larger than a cell is fine: it is anchored at the cell's top-left and drawn at full size. A full-canvas
  background goes in the top-left cell of its own layer.
- Layers also take `blend` (`multiply`, `screen`, `add`, ...) and `opacity`.
- There is no `{ sprite, x, y }` or `position` form. It fails with `missing field sprites`.
- Render one with `pxl render scene.pxl -c scene -o scene.png`.

### Recolouring a Sprite

Two ways to draw the same shapes in other colours:

```json5
// Override a few tokens
{ type: "variant", name: "llama_brown", base: "llama", palette: { coat: "#7d5634", coat_shade: "#5c3d22" } }

// Swap the whole palette (the other palette must define the same token names), optionally with a transform
{ type: "sprite", name: "hut_grey", palette: "hut_grey_colors", source: "hut" }
{ type: "sprite", name: "llama_right", palette: "llama_colors", source: "llama", transform: ["mirror-h"] }
```

Name tokens by part or material (`wall`, `thatch`, `sleeve`), not by colour, so a second palette can reuse them.

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

### Building Scenes

A scene is a composition of many small sprites. What works:

1. **One sprite per thing, reused.** A hut, a tree, a figure, a clump of grass. Place each as often as needed.
2. **Draw far to near.** Put things in layers ordered from the back of the scene to the front. When one object must
   stand between two parts of another (an animal inside a fence), split the other into a far sprite and a near sprite.
3. **Give every sprite an anchor.** Decide which pixel stands on the ground (for example the midpoint of the feet)
   and place the sprite at `anchor position - anchor offset`. Keep positions on multiples of `cell_size`.
4. **Use full-canvas sprites for terrain.** Ground, roads and large buildings are easier to draw in canvas
   coordinates. Give the sprite the canvas size and put it in the top-left cell.
5. **Shadows and light are sprites too.** Colours take alpha (`#RRGGBBAA`). A `shadow: "#00000080"` region under an
   object, or a warm translucent polygon in front of a doorway, blends over whatever is below it.
6. **Split a large scene across files.** `cat` the files into one stream before rendering; palettes and sprites must
   come before the compositions that use them.

#### Isometric Scenes

For a 2:1 isometric view, choose one projection and compute every point from it:

```
screen_x = origin_x + (tile_w / 2) * (gx - gy)
screen_y = origin_y + (tile_h / 2) * (gx + gy) - height        // tile_w = 2 * tile_h, e.g. 16 x 8
```

- A box is three polygons: the top, the face looking down-left and the face looking down-right. Light one face and
  shade the other the same way on every object.
- A wall face is a parallelogram: two ground points and the same two points raised by the wall height. Doors, windows
  and courses of stone are drawn in the same plane (edges parallel to the wall's base, or vertical).
- A gabled roof is two parallelograms meeting at a ridge line raised above the middle of the building.
- Roads and paths are bands along a tile axis: lines with slope 1/2 or -1/2.
- Draw order is by depth: larger `gx + gy` is nearer, so it goes in a later layer.

## Common Mistakes

### Composition With Positions

```json5
// WRONG: there is no x/y or position on a layer
{ type: "composition", name: "scene", size: [32, 32], layers: [{ sprite: "hero", x: 8, y: 8 }] }
```

**Fix:** use `sprites` and a `map` (see Composition above). An object that fails to parse is skipped, and
`pxl render -c <name>` then reports that the composition was not found, so run `pxl validate` first.

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

### Mask (Read-Only Sprite Queries)

```bash
# List all sprites, animations, compositions in a file
pxl mask input.pxl --list --json

# Find all coordinates where a token appears
pxl mask input.pxl --sprite hero --query "{eye}" --json

# Get bounding box of a token
pxl mask input.pxl --sprite hero --bounds "{skin}" --json

# Sample token at a coordinate
pxl mask input.pxl --sprite hero --sample 6,6 --json

# Get 4-connected neighbors at a coordinate
pxl mask input.pxl --sprite hero --neighbors 6,6 --json

# Extract a rectangular region of tokens
pxl mask input.pxl --sprite hero --region 5,5,6,4 --json

# Token frequency counts
pxl mask input.pxl --sprite hero --count --json
```

## Workflow

1. **Generate** - Create JSON5 with palette and sprite definitions
2. **Validate** - Run `pxl validate file.pxl` to check for errors
3. **Render** - Run `pxl render file.pxl` to create PNG
4. **Inspect** - Run `pxl mask file.pxl --sprite name --count --json` to verify state
5. **Iterate** - Fix any issues and re-render

## Tips for Better Sprites

1. **Start with outline** - Define the boundary stroke first
2. **Fill inside** - Use `fill: "inside(outline)"` for interior
3. **Add details** - Eyes, highlights using points and small shapes
4. **Use symmetry** - `symmetric: "x"` for bilateral features
5. **Layer shadows** - Define shadow regions with y-constraints
6. **Test at 1x scale** - Ensure readability at native size

## Artisan Workflow

For autonomous art generation, use component-based iteration with quality gates.

**Full documentation:** See `docs/artistic-workflow.md` for the complete artisan workflow including:
- Component iteration pattern (Foundation → Component Loop → Integration → Submit)
- Quality gates (silhouette, scale, palette, pixels, lighting)
- Variant naming conventions
- RPG character quality targets
- Iteration logging format

### Quick Reference

**Phases:** Foundation → Component Iteration (per part) → Integration → Submit

**Quality Gates (pass/fail):**
- Silhouette: Shape recognizable when filled solid
- Scale: Readable at 1x zoom
- Palette: 100% colors from defined palette
- Pixels: No orphans, clean edges at 8x zoom
- Lighting: Consistent top-right 45°

**Variant naming:** `[component]_v[iteration][variant]` (e.g., `eyes_v1a`, `eyes_v2b`)
