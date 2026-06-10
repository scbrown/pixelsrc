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

## Region Inheritance (`extends`)

`extends` builds a sprite from another sprite's regions, then patches them
key-by-key. It is the right tool for **animation frame-sets**, where most
regions are identical across frames and only a few change — instead of copying
every region into every frame (and risking a frame-tear when one copy drifts),
each frame inherits the shared regions and overrides only what moves.

```json5
// Base frame: defines the full sprite.
{
  type: "sprite",
  name: "fire_a",
  size: [16, 32],
  palette: "fire",
  regions: {
    ring:  { rect: [3, 26, 10, 4], z: 0 },   // stone ring — shared by every frame
    flame: { rect: [7, 14, 2, 12], z: 1 },
  },
}

// Next frame: inherits `ring` and the palette; overrides only `flame`.
{
  type: "sprite",
  name: "fire_b",
  extends: "fire_a",
  regions: {
    flame: { rect: [8, 12, 2, 14], z: 1 },
  },
}
```

### Fields

| Field | Description |
|-------|-------------|
| `extends` | Name of the base sprite to inherit regions, palette, and size from |
| `regions` | Region patches: each key **replaces** the inherited region of that name (or **adds** a new one if the key is not inherited) |
| `remove` | List of inherited region keys to **delete** |

### Semantics

- **Override** — a key in `regions` that the base also defines replaces the
  whole region definition. Its `z` and `role` come from the override, not the
  inherited region, so an override is free to change its own z-order. (Other
  regions keep theirs; nothing else re-stacks.)
- **Add** — a key not present in the base is simply inserted at its declared `z`.
- **Remove** — `remove: ["key"]` deletes an inherited region. Removing a key the
  base doesn't define is a warning, not an error.
- **Palette** — the base's resolved palette is inherited. You may omit `palette`
  entirely (it is required on every *non-extending* sprite); if you do declare
  one, its colors override inherited tokens key-by-key, exactly like a
  [variant](variant.md).
- **Size** — inherited from the base unless the extending sprite declares its own.
- **Chaining** — `extends` may point at another extending sprite; the chain
  resolves base-first, so the deepest override of a key wins.

### `extends` vs. `source`

Both inherit from another sprite, but they operate at different levels:

| | `source` | `extends` |
|---|----------|-----------|
| Inherits | The base sprite's **rendered image** | The base sprite's **region map** |
| Can change individual regions? | No — only whole-image `transform`s | Yes — override/add/remove by key |
| Use for | Mirrored / rotated / filtered variants | Animation frames that share most regions |

Inspect the resolved result of an extending sprite the same way as any sprite —
`pxl mask <file> --sprite fire_b` shows the merged token grid, and
`pxl render --sprite fire_b` renders the inherited + overridden regions together.

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
