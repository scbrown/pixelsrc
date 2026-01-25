# Palette

A palette defines named color tokens for use in sprites. Palettes separate color definitions from sprite structure, making it easy to create color variations and maintain consistent color schemes.

## Basic Syntax

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    skin: "#FFCC99",
    hair: "#8B4513",
  },
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"palette"` |
| `name` | Yes | Unique identifier, referenced by sprites |
| `colors` | Yes | Map of token names to color values |
| `roles` | No | Semantic roles for tokens |
| `relationships` | No | Token relationships |

## Example

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    skin: "#FFCC99",
    hair: "#8B4513",
    shirt: "#4169E1",
    outline: "#000000",
  },
  roles: {
    outline: "boundary",
    skin: "fill",
    hair: "fill",
    shirt: "fill",
  },
}
```

## Color Formats

Palettes accept colors in these formats:

| Format | Example | Description |
|--------|---------|-------------|
| `#RGB` | `#F00` | Short hex (expands to #FF0000) |
| `#RGBA` | `#F008` | Short hex with alpha |
| `#RRGGBB` | `#FF0000` | Full hex |
| `#RRGGBBAA` | `#FF000080` | Full hex with alpha |
| `transparent` | - | Fully transparent |

CSS color formats are also supported: `rgb()`, `hsl()`, `hwb()`, named colors, etc. See [Color Formats](../reference/colors.md) for details.

## Semantic Roles

Define the semantic purpose of each token:

```json5
{
  type: "palette",
  name: "character",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    eye: "#4169E1",
  },
  roles: {
    outline: "boundary",     // Edge-defining
    skin: "fill",            // Interior mass
    "skin-shadow": "shadow", // Shading
    eye: "anchor",           // Critical detail
  },
}
```

Available roles:
- `boundary` - Edge-defining outlines
- `anchor` - Critical details (eyes, etc.)
- `fill` - Large interior areas
- `shadow` - Shading regions
- `highlight` - Light regions

## Relationships

Define how tokens relate to each other:

```json5
{
  type: "palette",
  name: "character",
  colors: {
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    "skin-hi": "#FFF0E0",
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" },
    "skin-hi": { type: "derives-from", target: "skin" },
  },
}
```

Relationship types:
- `derives-from` - Color derived from another
- `contained-within` - Region contained in another
- `adjacent-to` - Regions share boundary
- `paired-with` - Symmetric regions

## Reserved Tokens

- `_` - Conventional token for transparency (widely used but not enforced)

## Referencing Palettes

Sprites reference palettes by name:

```json5
{ type: "palette", name: "mono", colors: { on: "#FFF", off: "#000" } }
{ type: "sprite", name: "dot", size: [1, 1], palette: "mono", regions: { on: { rect: [0, 0, 1, 1] } } }
```

### Inline Colors

Define colors directly in the sprite:

```json5
{
  type: "sprite",
  name: "dot",
  size: [1, 1],
  palette: { on: "#FFF", _: "transparent" },
  regions: { on: { rect: [0, 0, 1, 1] } },
}
```

### Built-in Palettes

Reference built-in palettes with the `@` prefix:

```json5
{ type: "sprite", name: "retro", palette: "@gameboy", ... }
```

See [Built-in Palettes](../reference/palettes.md) for available options.

## Order Matters

Palettes must be defined before sprites that reference them:

```json5
// Correct order
{ type: "palette", name: "hero", colors: { ... } }
{ type: "sprite", name: "hero_sprite", palette: "hero", ... }
```

## Multiple Sprites, One Palette

A palette can be shared across multiple sprites:

```json5
{
  type: "palette",
  name: "ui",
  colors: {
    _: "transparent",
    bg: "#2D2D2D",
    border: "#4A4A4A",
    text: "#FFFFFF",
  },
}

{ type: "sprite", name: "button", palette: "ui", ... }
{ type: "sprite", name: "panel", palette: "ui", ... }
{ type: "sprite", name: "icon", palette: "ui", ... }
```

Changing a color in the palette updates all sprites that use it.
