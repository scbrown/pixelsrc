# Includes

The include system allows you to reference palettes from external files. This enables sharing color schemes across multiple Pixelsrc files and organizing large projects into modular components.

## Syntax

Use the `@include:` prefix followed by a file path:

```json5
{
  type: "sprite",
  name: "hero",
  size: [8, 8],
  palette: "@include:shared/colors.pxl",
  regions: { ... },
}
```

## How It Works

When Pixelsrc encounters an `@include:` palette reference:

1. Resolves the path relative to the current file's directory
2. Opens and parses the included file
3. Extracts the first palette object from that file
4. Uses that palette for the sprite

## Path Resolution

Paths are resolved relative to the file containing the `@include:` reference:

```
project/
├── sprites/
│   └── hero.pxl          ← Contains @include:../palettes/hero.pxl
└── palettes/
    └── hero.pxl          ← Palette file
```

## Extension Auto-Detection

If the specified file doesn't exist, Pixelsrc tries alternate extensions:

1. Exact path as specified
2. Path with `.pxl` extension
3. Path with `.jsonl` extension

```json5
// All of these work if "colors.pxl" exists:
"palette": "@include:colors"
"palette": "@include:colors.pxl"
```

## Example

**shared/palette.pxl**
```json5
{
  type: "palette",
  name: "game_colors",
  colors: {
    _: "transparent",
    skin: "#FFCC99",
    hair: "#8B4513",
    outline: "#000000",
  },
}
```

**sprites/hero.pxl**
```json5
{
  type: "sprite",
  name: "hero",
  size: [6, 4],
  palette: "@include:../shared/palette.pxl",
  regions: {
    hair: { rect: [2, 0, 2, 2], z: 0 },
    outline: {
      union: [
        { points: [[0, 2], [5, 2]] },
        { rect: [1, 3, 4, 1] },
      ],
      z: 0,
    },
    skin: { rect: [1, 2, 4, 1], z: 1 },
  },
}
```

## Use Cases

### Shared Color Schemes

Maintain a single source of truth for colors across multiple sprite files:

```
assets/
├── palettes/
│   ├── characters.pxl
│   ├── environment.pxl
│   └── ui.pxl
└── sprites/
    ├── hero.pxl          ← @include:../palettes/characters.pxl
    ├── enemy.pxl         ← @include:../palettes/characters.pxl
    └── button.pxl        ← @include:../palettes/ui.pxl
```

### Theme Variations

Create different color themes by swapping include paths:

```json5
// day_scene.pxl
{
  type: "sprite",
  name: "tree",
  size: [8, 12],
  palette: "@include:palettes/day.pxl",
  regions: { ... },
}

// night_scene.pxl
{
  type: "sprite",
  name: "tree",
  size: [8, 12],
  palette: "@include:palettes/night.pxl",
  regions: { ... },
}
```

### Library Palettes

Reference palettes from a central library:

```json5
{
  type: "sprite",
  name: "character",
  size: [16, 16],
  palette: "@include:../../lib/palettes/fantasy.pxl",
  regions: { ... },
}
```

## Error Handling

### File Not Found

If the include file doesn't exist:

- **Lenient mode**: Error (cannot substitute a missing palette)
- **Strict mode**: Error

### No Palette in File

If the included file doesn't contain a palette object:

- Error: "No palette found in included file"

The included file must contain at least one `{"type": "palette", ...}` object.

### Circular Includes

Pixelsrc detects circular include chains:

```
a.pxl includes b.pxl
b.pxl includes c.pxl
c.pxl includes a.pxl  ← Circular include error
```

## Comparison with Other Palette Methods

| Method | Best For |
|--------|----------|
| Inline palette | Simple, self-contained sprites |
| Named palette | Multiple sprites in the same file |
| Built-in (`@name`) | Quick prototyping with standard palettes |
| Include (`@include:`) | Shared palettes across files |

## Complete Example

**palettes/retro.pxl**
```json5
{
  type: "palette",
  name: "retro",
  colors: {
    _: "transparent",
    bg: "#0F380F",
    light: "#9BBC0F",
    mid: "#8BAC0F",
    dark: "#306230",
  },
}
```

**sprites/player.pxl**
```json5
{
  type: "sprite",
  name: "player",
  size: [4, 4],
  palette: "@include:../palettes/retro.pxl",
  regions: {
    light: { rect: [1, 0, 2, 1], z: 0 },
    mid: {
      union: [
        { points: [[0, 1], [3, 1]] },
        { rect: [0, 2, 4, 1] },
      ],
      z: 0,
    },
    dark: {
      union: [
        { rect: [1, 1, 2, 1] },
        { rect: [1, 3, 2, 1] },
      ],
      z: 1,
    },
  },
}
```

**sprites/enemy.pxl**
```json5
{
  type: "sprite",
  name: "enemy",
  size: [4, 4],
  palette: "@include:../palettes/retro.pxl",
  regions: {
    dark: { points: [[0, 0], [3, 0]], z: 0 },
    mid: {
      union: [
        { rect: [1, 1, 2, 1] },
        { rect: [0, 2, 4, 1] },
        { rect: [1, 3, 2, 1] },
      ],
      z: 0,
    },
    light: { rect: [1, 2, 2, 1], z: 1 },
  },
}
```

Both sprites share the same palette, and changing `retro.pxl` updates all sprites that include it.
