# Composition

A composition layers multiple sprites onto a canvas. This is useful for building scenes, tile maps, and complex images from smaller sprite components.

## Basic Syntax

```json5
{
  type: "composition",
  name: "scene",
  size: [width, height],
  layers: [
    { sprite: "background", x: 0, y: 0 },
    { sprite: "hero", x: 16, y: 16 },
  ],
}
```

## Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `type` | Yes | - | Must be `"composition"` |
| `name` | Yes | - | Unique identifier |
| `size` | Yes | - | Canvas size `[width, height]` in pixels |
| `layers` | Yes | - | Array of layers, rendered bottom-to-top |

## Layer Fields

| Field | Required | Description |
|-------|----------|-------------|
| `sprite` | Yes | Sprite name to place |
| `x` | No | X position (default: 0) |
| `y` | No | Y position (default: 0) |
| `blend` | No | Blend mode (default: `"normal"`) |
| `opacity` | No | Layer opacity 0.0-1.0 (default: 1.0) |

## Simple Example

```json5
{
  type: "palette",
  name: "scene",
  colors: {
    _: "transparent",
    g: "#228B22",
    h: "#FFD700",
    t: "#8B4513",
  },
}

{
  type: "sprite",
  name: "grass",
  size: [8, 8],
  palette: "scene",
  regions: { g: { rect: [0, 0, 8, 8] } },
}

{
  type: "sprite",
  name: "hero",
  size: [8, 8],
  palette: "scene",
  regions: {
    h: { rect: [2, 2, 4, 6] },
  },
}

{
  type: "sprite",
  name: "tree",
  size: [8, 8],
  palette: "scene",
  regions: {
    g: { ellipse: [4, 2, 3, 2] },
    t: { rect: [3, 4, 2, 4] },
  },
}

{
  type: "composition",
  name: "scene",
  size: [32, 32],
  layers: [
    { sprite: "grass", x: 0, y: 0 },
    { sprite: "grass", x: 8, y: 0 },
    { sprite: "grass", x: 16, y: 0 },
    { sprite: "grass", x: 24, y: 0 },
    { sprite: "tree", x: 4, y: 8 },
    { sprite: "hero", x: 12, y: 16 },
    { sprite: "tree", x: 20, y: 8 },
  ],
}
```

## Multiple Layers

Layers are rendered bottom-to-top. The first layer is the background:

```json5
{
  type: "composition",
  name: "scene",
  size: [64, 64],
  layers: [
    // Background layer
    { sprite: "sky_background", x: 0, y: 0 },
    // Middle layer - ground
    { sprite: "ground", x: 0, y: 48 },
    // Foreground - characters
    { sprite: "hero", x: 24, y: 32 },
    { sprite: "enemy", x: 40, y: 32 },
  ],
}
```

## Blend Modes

Layer blending controls how colors combine when layers overlap:

```json5
{
  type: "composition",
  name: "effects",
  size: [32, 32],
  layers: [
    { sprite: "background", x: 0, y: 0 },
    { sprite: "shadow", x: 8, y: 8, blend: "multiply", opacity: 0.5 },
    { sprite: "glow", x: 12, y: 4, blend: "add" },
  ],
}
```

### Blend Mode Reference

| Mode | Description |
|------|-------------|
| `normal` | Standard alpha compositing |
| `multiply` | Darkens - good for shadows |
| `screen` | Lightens - good for glows |
| `overlay` | Enhances contrast |
| `add` | Additive blending - fire, particles |
| `subtract` | Subtractive blending |
| `difference` | Absolute difference |
| `darken` | Keeps darker color |
| `lighten` | Keeps lighter color |

### When to Use Each Mode

**`multiply`** - Shadows, tinting, darkening
```json5
{ sprite: "shadow", x: 4, y: 4, blend: "multiply", opacity: 0.3 }
```

**`screen`** - Glows, highlights, fog
```json5
{ sprite: "glow", x: 8, y: 8, blend: "screen" }
```

**`add`** - Fire, particles, magical effects
```json5
{ sprite: "particles", x: 0, y: 0, blend: "add" }
```

## CSS Variables in Compositions

Composition layers support CSS variable references:

```json5
{
  type: "composition",
  name: "themed_scene",
  size: [32, 32],
  layers: [
    { sprite: "background", x: 0, y: 0 },
    { sprite: "shadow", x: 4, y: 4, blend: "var(--shadow-blend)", opacity: "var(--shadow-opacity)" },
    { sprite: "glow", x: 8, y: 8, blend: "var(--glow-mode, add)" },
  ],
}
```

Variables are resolved from the palette's variable registry at render time.

## Nested Compositions

Compositions can reference other compositions, enabling hierarchical scene construction:

```json5
// Forest tile composition
{
  type: "composition",
  name: "forest_tile",
  size: [16, 16],
  layers: [
    { sprite: "grass", x: 0, y: 0 },
    { sprite: "tree", x: 4, y: 0 },
  ],
}

// Main scene using forest_tile
{
  type: "composition",
  name: "scene",
  size: [32, 32],
  layers: [
    { sprite: "forest_tile", x: 0, y: 0 },
    { sprite: "forest_tile", x: 16, y: 0 },
    { sprite: "hero", x: 12, y: 16 },
  ],
}
```

### Cycle Detection

Circular references are detected and produce an error:

```json5
// ERROR: Cycle detected
{ type: "composition", name: "A", layers: [{ sprite: "B" }] }
{ type: "composition", name: "B", layers: [{ sprite: "A" }] }
```

## Complete Example

```json5
// Palette
{
  type: "palette",
  name: "island",
  colors: {
    _: "transparent",
    grass: "#228B22",
    water: "#4169E1",
    sand: "#F4A460",
  },
}

// Tile sprites
{
  type: "sprite",
  name: "grass_tile",
  size: [8, 8],
  palette: "island",
  regions: { grass: { rect: [0, 0, 8, 8] } },
}

{
  type: "sprite",
  name: "water_tile",
  size: [8, 8],
  palette: "island",
  regions: { water: { rect: [0, 0, 8, 8] } },
}

{
  type: "sprite",
  name: "sand_tile",
  size: [8, 8],
  palette: "island",
  regions: { sand: { rect: [0, 0, 8, 8] } },
}

// Island composition
{
  type: "composition",
  name: "island",
  size: [32, 32],
  layers: [
    // Water background
    { sprite: "water_tile", x: 0, y: 0 },
    { sprite: "water_tile", x: 8, y: 0 },
    { sprite: "water_tile", x: 16, y: 0 },
    { sprite: "water_tile", x: 24, y: 0 },
    { sprite: "water_tile", x: 0, y: 24 },
    { sprite: "water_tile", x: 8, y: 24 },
    { sprite: "water_tile", x: 16, y: 24 },
    { sprite: "water_tile", x: 24, y: 24 },
    // Sand border
    { sprite: "sand_tile", x: 8, y: 8 },
    { sprite: "sand_tile", x: 16, y: 8 },
    { sprite: "sand_tile", x: 8, y: 16 },
    { sprite: "sand_tile", x: 16, y: 16 },
    // Grass center
    { sprite: "grass_tile", x: 12, y: 12 },
  ],
}
```

This creates a 32x32 pixel island with water around the edges, a sand border, and grass in the center.
