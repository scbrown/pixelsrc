# Composition

A composition places sprites on a canvas. This is how scenes, tile maps and other large images are built from small sprites.

## Basic Syntax

```json5
{
  type: "composition",
  name: "scene",
  size: [width, height],
  cell_size: [cell_width, cell_height],
  sprites: { ".": null, G: "grass", H: "hero" },
  layers: [
    { map: ["GGGG", "GGGG"] },
    { map: ["....", ".H.."] },
  ],
}
```

`sprites` gives each sprite a one-character key. Each layer's `map` is a grid of those keys, one character per cell.
A sprite is drawn with its top-left corner at the top-left of the cell that holds its key.

## Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `type` | Yes | - | Must be `"composition"` |
| `name` | Yes | - | Unique identifier |
| `size` | No | - | Canvas size `[width, height]` in pixels |
| `cell_size` | No | `[1, 1]` | Size of one map cell in pixels |
| `sprites` | Yes | - | Map of one-character keys to sprite names; `null` marks an empty cell |
| `layers` | Yes | - | Array of layers, drawn first to last |

## Layer Fields

| Field | Required | Description |
|-------|----------|-------------|
| `map` | No | Rows of sprite keys, one character per cell |
| `name` | No | A label for the layer |
| `transform` | No | Transforms applied to the layer |
| `blend` | No | Blend mode (default: `"normal"`) |
| `opacity` | No | Layer opacity 0.0-1.0 (default: 1.0) |

A layer has no `sprite`, `x`, `y` or `position` field. A composition written that way fails to parse with
`missing field sprites`.

## Example

```json5
{
  type: "palette",
  name: "scene",
  colors: {
    _: "transparent",
    grass: "#228B22",
    gold: "#FFD700",
    trunk: "#8B4513",
    leaf: "#2e7d32",
    sky: "#87ceeb",
    shade: "#00000080",
  },
}

{ type: "sprite", name: "sky", size: [32, 32], palette: "scene", regions: { sky: { rect: [0, 0, 32, 32] } } }
{ type: "sprite", name: "grass", size: [8, 8], palette: "scene", regions: { grass: { rect: [0, 0, 8, 8] } } }
{ type: "sprite", name: "hero", size: [8, 8], palette: "scene", regions: { gold: { rect: [2, 2, 4, 6] } } }
{ type: "sprite", name: "shadow", size: [8, 8], palette: "scene", regions: { shade: { ellipse: [4, 6, 3, 1] } } }
{
  type: "sprite",
  name: "tree",
  size: [8, 8],
  palette: "scene",
  regions: {
    leaf: { ellipse: [4, 2, 3, 2] },
    trunk: { rect: [3, 4, 2, 4] },
  },
}

{
  type: "composition",
  name: "scene",
  size: [32, 32],
  cell_size: [8, 8],
  sprites: { ".": null, S: "sky", G: "grass", T: "tree", H: "hero", s: "shadow" },
  layers: [
    { name: "sky", map: ["S...", "....", "....", "...."] },
    { name: "ground", map: ["....", "....", "....", "GGGG"] },
    { name: "shadows", map: ["....", "....", ".s.s", "...."], blend: "multiply", opacity: 0.5 },
    { name: "things", map: ["....", "....", ".T.H", "...."] },
  ],
}
```

Render it with:

```bash
pxl render scene.pxl -c scene --scale 6 -o scene.png
```

## Cells and Placement

- **Cell size sets placement precision.** With `cell_size: [8, 8]` a sprite can sit at x = 0, 8, 16 and so on. Use a
  smaller cell (down to the default `[1, 1]`) when sprites need finer placement; the maps get larger in proportion.
- **A sprite may be larger than a cell.** It is anchored at the cell's top-left corner and drawn at full size. The
  renderer prints an `exceeds cell size` warning for each one; this is expected for scenes. The `sky` sprite above
  is the usual case: a full-canvas sprite in the top-left cell of its own layer.
- **Rows may be shorter than the grid.** Missing cells are empty, and the renderer warns that the map dimensions do
  not match the grid. Under `--strict` warnings are errors, so write full rows if you use it.
- **Put things that must cover each other in separate layers.** Layer order is the only draw order to rely on.

## Multiple Layers

Layers draw first to last. For a scene, order them from the back to the front:

```json5
layers: [
  { name: "sky", map: [...] },
  { name: "far buildings", map: [...] },
  { name: "people", map: [...] },
  { name: "foreground", map: [...] },
]
```

When one object must stand between two parts of another, for example an animal inside a fence, split the other into
a far sprite and a near sprite and put the object in a layer between them.

## Blend Modes

A layer's `blend` controls how its colors combine with the layers below:

```json5
{ name: "shadows", map: ["....", ".s.s"], blend: "multiply", opacity: 0.5 }
```

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

Palette colors also take alpha (`#RRGGBBAA`), so a translucent shadow or glow can be an ordinary region in an
ordinary layer.

## CSS Variables in Compositions

`blend` and `opacity` accept CSS variable references:

```json5
{ map: ["s..."], blend: "var(--shadow-blend)", opacity: "var(--shadow-opacity)" }
```

Variables are resolved from the palette's variable registry at render time.

## Known Limits

- **A composition cannot be used as a sprite in another composition** with `pxl render` 0.3.1. The key resolves to
  nothing, the cell is left empty, and the renderer warns `Sprite or variant '<name>' not found`.
- **One composition that fails to parse hides the others.** `pxl render -c <name>` then reports that the named
  composition was not found. Run `pxl validate` first; it names the object and the parse error.
