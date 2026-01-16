# Includes

The include system allows you to reference palettes from external files. This enables sharing color schemes across multiple Pixelsrc files and organizing large projects into modular components.

## Syntax

Use the `@include:` prefix followed by a file path:

```json
{"type": "sprite", "name": "hero", "palette": "@include:shared/colors.pxl", "grid": [...]}
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

```json
// All of these work if "colors.pxl" exists:
"palette": "@include:colors"
"palette": "@include:colors.pxl"
```

## Example

**shared/palette.pxl**
```json
{"type": "palette", "name": "game_colors", "colors": {
  "{_}": "#00000000",
  "{skin}": "#FFCC99",
  "{hair}": "#8B4513",
  "{outline}": "#000000"
}}
```

**sprites/hero.pxl**
```json
{"type": "sprite", "name": "hero", "palette": "@include:../shared/palette.pxl", "grid": [
  "{_}{_}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{_}",
  "{outline}{skin}{skin}{skin}{skin}{outline}",
  "{_}{outline}{outline}{outline}{outline}{_}"
]}
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

```json
// day_scene.pxl
{"type": "sprite", "name": "tree", "palette": "@include:palettes/day.pxl", "grid": [...]}

// night_scene.pxl
{"type": "sprite", "name": "tree", "palette": "@include:palettes/night.pxl", "grid": [...]}
```

### Library Palettes

Reference palettes from a central library:

```json
{"type": "sprite", "name": "character", "palette": "@include:../../lib/palettes/fantasy.pxl", "grid": [...]}
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
```json
{"type": "palette", "name": "retro", "colors": {
  "{_}": "#00000000",
  "{bg}": "#0f380f",
  "{light}": "#9bbc0f",
  "{mid}": "#8bac0f",
  "{dark}": "#306230"
}}
```

**sprites/player.pxl**
```json
{"type": "sprite", "name": "player", "palette": "@include:../palettes/retro.pxl", "grid": [
  "{_}{light}{light}{_}",
  "{light}{mid}{mid}{light}",
  "{mid}{dark}{dark}{mid}",
  "{_}{dark}{dark}{_}"
]}
```

**sprites/enemy.pxl**
```json
{"type": "sprite", "name": "enemy", "palette": "@include:../palettes/retro.pxl", "grid": [
  "{dark}{_}{_}{dark}",
  "{_}{mid}{mid}{_}",
  "{mid}{light}{light}{mid}",
  "{_}{mid}{mid}{_}"
]}
```

Both sprites share the same palette, and changing `retro.pxl` updates all sprites that include it.
