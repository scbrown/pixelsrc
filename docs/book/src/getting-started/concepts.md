# Core Concepts

Understanding these core concepts will help you work effectively with Pixelsrc.

## Objects and Types

Pixelsrc files contain **objects** in JSON5 format. Each object has a `type` field:

| Type | Purpose | Required Fields |
|------|---------|-----------------|
| `palette` | Define named colors | `name`, `colors` |
| `sprite` | Structured regions | `name`, `size`, `palette`, `regions` |
| `animation` | Frame sequence | `name`, `frames` |
| `composition` | Layer sprites | `name`, `size`, `layers` |
| `variant` | Modify existing sprite | `name`, `base`, changes |

## Palettes

A **palette** defines colors with semantic names:

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    skin: "#FFCC99",
    hair: "#8B4513",
    outline: "#000000",
  },
}
```

Key points:
- Token names are simple identifiers: `skin`, `hair`, `outline`
- Colors use hex format: `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA`
- `_` is the conventional token for transparency
- Palettes must be defined before sprites that reference them

## Tokens

**Tokens** are identifiers that represent colors:

```
_          → transparent (convention)
skin       → semantic name for skin color
outline    → semantic name for outline
dark_hair  → underscores OK for multi-word names
```

Benefits of semantic tokens:
- **Readable**: Region definitions reference meaningful names
- **Maintainable**: Change a color in one place (the palette), update everywhere
- **AI-friendly**: LLMs can reason about `shadow` more reliably than hex values

## Sprites

A **sprite** is defined using structured **regions**:

```json5
{
  type: "sprite",
  name: "cross",
  size: [3, 3],
  palette: "colors",
  regions: {
    r: {
      union: [
        { points: [[1, 0], [1, 2]] },  // Vertical
        { rect: [0, 1, 3, 1] },         // Horizontal
      ],
    },
  },
}
```

Key points:
- `size`: Required `[width, height]`
- `regions`: Map of token names to shape definitions
- `z`: Controls draw order (higher draws on top)
- `palette`: Name of a defined palette or inline colors object

## Shape Primitives

| Shape | Syntax | Description |
|-------|--------|-------------|
| `rect` | `[x, y, w, h]` | Filled rectangle |
| `stroke` | `[x, y, w, h]` | Rectangle outline |
| `points` | `[[x, y], ...]` | Individual pixels |
| `line` | `[[x1, y1], ...]` | Connected line |
| `circle` | `[cx, cy, r]` | Filled circle |
| `ellipse` | `[cx, cy, rx, ry]` | Filled ellipse |
| `polygon` | `[[x, y], ...]` | Filled polygon |

## Compound Shapes

Combine shapes using `union`, `subtract`, or `intersect`:

```json5
regions: {
  body: {
    union: [
      { rect: [2, 0, 4, 2] },
      { rect: [0, 2, 8, 4] },
    ],
    z: 0,
  },
}
```

## Animations

An **animation** sequences multiple sprites:

```json5
{
  type: "animation",
  name: "walk",
  frames: ["walk_1", "walk_2", "walk_3"],
  duration: 100,
}
```

Key points:
- `frames`: Array of sprite names in order
- `duration`: Milliseconds per frame (default: 100)
- `loop`: Whether to loop (default: true)

## Compositions

A **composition** layers multiple sprites:

```json5
{
  type: "composition",
  name: "scene",
  size: [16, 16],
  layers: [
    { sprite: "background", x: 0, y: 0 },
    { sprite: "hero", x: 4, y: 4 },
  ],
}
```

Layers are rendered bottom-to-top (first layer is background).

## File Format

Pixelsrc uses **JSON5** format:
- Supports comments (`// comment`)
- Trailing commas allowed
- Unquoted keys
- Multiple objects separated by whitespace
- Files typically use `.pxl` extension

This format is:
- Human-readable with comments
- AI-friendly for generation
- Easy to edit manually

## Lenient Mode

By default, Pixelsrc is **lenient**:
- Missing tokens render as magenta (visible but not breaking)
- Small mistakes don't halt rendering
- Provides helpful warnings

Use `--strict` mode for validation in CI/CD pipelines.
