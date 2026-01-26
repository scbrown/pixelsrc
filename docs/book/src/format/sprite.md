# Sprite

A sprite defines a pixel art image using named regions. Each region maps to a color token from the palette and describes its shape geometrically.

## Basic Syntax

```json5
{
  type: "sprite",
  name: "string (required)",
  size: [width, height],
  palette: "string (required)",
  regions: {
    token: { shape_definition },
    token: { shape_definition },
  },
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"sprite"` |
| `name` | Yes | Unique identifier |
| `size` | Yes | `[width, height]` in pixels |
| `palette` | Yes | Palette name to use for colors |
| `regions` | Yes | Map of token names to region definitions |

### Optional Fields

| Field | Description |
|-------|-------------|
| `background` | Token to fill empty pixels (default: `_`) |
| `origin` | Anchor point `[x, y]` for transforms |
| `metadata` | Custom data passthrough for game engines |
| `state-rules` | Name of state rules to apply |

## Example

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

## Regions

The `regions` field maps token names to shape definitions. Tokens must exist in the referenced palette.

### Simple Shapes

```json5
regions: {
  // Individual pixels
  eye: { points: [[5, 6], [10, 6]] },

  // Filled rectangle
  body: { rect: [2, 4, 12, 8] },

  // Rectangle outline
  outline: { stroke: [0, 0, 16, 16] },

  // Filled circle
  head: { circle: [8, 4, 3] },

  // Line
  mouth: { line: [[5, 10], [10, 10]] },
}
```

### Fill Operations

Fill inside a boundary:

```json5
regions: {
  outline: { stroke: [0, 0, 16, 16] },
  skin: { fill: "inside(outline)" },
}
```

Fill with exclusions:

```json5
regions: {
  outline: { stroke: [0, 0, 16, 16] },
  eye: { rect: [5, 5, 2, 2], symmetric: "x" },
  skin: {
    fill: "inside(outline)",
    except: ["eye"],
  },
}
```

### Symmetry

Auto-mirror regions across an axis:

```json5
regions: {
  // Creates eyes at [5, 6] and [10, 6] for 16-wide sprite
  eye: {
    points: [[5, 6]],
    symmetric: "x",
  },
}
```

### Background

The special `"background"` value fills all unoccupied pixels:

```json5
regions: {
  _: "background",
  // ... other regions ...
}
```

See [Regions & Shapes](regions.md) for complete documentation of all shape primitives and modifiers.

## Palette Options

### Named Palette

Reference a palette defined earlier in the file:

```json5
{
  type: "palette",
  name: "hero_colors",
  colors: { /* ... */ },
}

{
  type: "sprite",
  name: "hero",
  palette: "hero_colors",
  size: [16, 16],
  regions: { /* ... */ },
}
```

### Built-in Palette

Reference a built-in palette with `@` prefix:

```json5
{
  type: "sprite",
  name: "retro",
  palette: "@gameboy",
  size: [8, 8],
  regions: { /* ... */ },
}
```

## Metadata

Attach additional data for game engine integration:

```json5
{
  type: "sprite",
  name: "player_attack",
  size: [32, 32],
  palette: "hero",
  regions: { /* ... */ },
  origin: [16, 32],
  metadata: {
    boxes: {
      hurt: { x: 4, y: 0, w: 24, h: 32 },
      hit: { x: 20, y: 8, w: 20, h: 16 },
    },
  },
}
```

### Common Metadata Fields

| Field | Purpose |
|-------|---------|
| `origin` | Sprite anchor point `[x, y]` |
| `boxes.hurt` | Damage-receiving region |
| `boxes.hit` | Damage-dealing region |
| `boxes.collide` | Physics collision boundary |
| `boxes.trigger` | Interaction trigger zone |

## Nine-Slice

Create scalable sprites where corners stay fixed while edges and center stretch:

```json5
{
  type: "sprite",
  name: "button",
  size: [16, 16],
  palette: "ui",
  regions: { /* ... */ },
  nine_slice: {
    left: 4,
    right: 4,
    top: 4,
    bottom: 4,
  },
}
```

Render at different sizes:

```bash
pxl render button.pxl --nine-slice 64x32 -o button_wide.png
```

## Transforms (Derived Sprites)

Create derived sprites by applying op-style transforms to an existing sprite:

```json5
{
  type: "sprite",
  name: "hero_outlined",
  source: "hero",
  transform: [
    { op: "sel-out", fallback: "outline" },
  ],
}
```

Op-style transforms support both geometric operations (`mirror-h`, `rotate:90`) and effects (`sel-out`, `dither`, `shadow`).

See [Transforms](transforms.md#op-style-transforms-derived-sprites) for the full list of operations.

> **Note:** For animated transforms (in keyframes), use CSS transform strings instead. See [Animation](animation.md).

## Complete Example

```json5
// hero.pxl
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    hair: "#8B4513",
    eye: "#4169E1",
    shirt: "#E74C3C",
  },
  roles: {
    outline: "boundary",
    eye: "anchor",
    skin: "fill",
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

    // Head
    "head-outline": { stroke: [4, 0, 8, 10], round: 2 },
    hair: { fill: "inside(head-outline)", y: [0, 4] },
    skin: {
      fill: "inside(head-outline)",
      y: [4, 10],
      except: ["eye"],
    },

    // Eyes (symmetric)
    eye: { rect: [5, 5, 2, 2], symmetric: "x" },

    // Body
    "body-outline": { stroke: [3, 10, 10, 14] },
    shirt: { fill: "inside(body-outline)" },
  },
  origin: [8, 24],
  metadata: {
    boxes: {
      collide: { x: 4, y: 10, w: 8, h: 14 },
    },
  },
}
```

## Error Handling

### Lenient Mode (Default)

| Error | Behavior |
|-------|----------|
| Unknown token | Render as magenta `#FF00FF` |
| Region outside canvas | Clip to canvas with warning |
| Forward reference in fill | Error (must define dependencies first) |
| Missing palette | All regions render white with warning |

### Strict Mode

All warnings become errors. Use `--strict` flag for CI validation.
