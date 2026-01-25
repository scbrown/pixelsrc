# Variant

A variant creates a color variation of an existing sprite without duplicating the structure. This is useful for creating enemy color swaps, team colors, alternate costumes, and similar variations.

## Basic Syntax

```json5
{
  type: "variant",
  name: "hero_red",
  base: "hero",
  palette: { shirt: "#DC143C" },
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"variant"` |
| `name` | Yes | Unique identifier for this variant |
| `base` | Yes | Name of the sprite to derive from |
| `palette` | Yes | Color overrides - replaces matching tokens from base |

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
  },
}

{
  type: "sprite",
  name: "hero",
  size: [8, 12],
  palette: "hero",
  regions: {
    hair: { rect: [2, 0, 4, 2] },
    skin: { rect: [2, 2, 4, 3] },
    shirt: { rect: [1, 5, 6, 5] },
  },
}

{
  type: "variant",
  name: "hero_red",
  base: "hero",
  palette: { shirt: "#DC143C" },
}

{
  type: "variant",
  name: "hero_green",
  base: "hero",
  palette: { shirt: "#228B22" },
}
```

All three sprites share the same regions. Only the shirt color differs.

## Behavior

- **Inherits regions and size** from the base sprite
- **Only specified tokens are overridden** - unspecified tokens keep their base colors
- **Base sprite must be defined first** - forward references are errors

## Partial Overrides

You don't need to override all colors. Only specify the tokens you want to change:

```json5
{
  type: "sprite",
  name: "knight",
  size: [16, 16],
  palette: {
    armor: "#808080",
    plume: "#FF0000",
    skin: "#FFCC99",
    eyes: "#0000FF",
  },
  regions: {
    armor: { rect: [4, 4, 8, 10] },
    plume: { rect: [6, 0, 4, 4] },
    skin: { rect: [5, 2, 6, 3] },
    eyes: { points: [[6, 3], [9, 3]] },
  },
}

{
  type: "variant",
  name: "knight_gold",
  base: "knight",
  palette: {
    armor: "#FFD700",
    plume: "#FFFFFF",
  },
}
```

The `knight_gold` variant has gold armor and white plume, but keeps the same skin and eye colors.

## Use Cases

### Team Colors

```json5
{ type: "sprite", name: "player", size: [8, 8], palette: { team: "#FF0000" }, regions: { team: { rect: [2, 2, 4, 4] } } }
{ type: "variant", name: "player_blue", base: "player", palette: { team: "#0000FF" } }
{ type: "variant", name: "player_green", base: "player", palette: { team: "#00FF00" } }
```

### Enemy Variants

```json5
{
  type: "sprite",
  name: "slime",
  size: [8, 8],
  palette: { body: "#00FF00", eyes: "#FFFFFF" },
  regions: {
    body: { ellipse: [4, 5, 3, 2] },
    eyes: { points: [[3, 4], [5, 4]] },
  },
}

{ type: "variant", name: "slime_fire", base: "slime", palette: { body: "#FF4500" } }
{ type: "variant", name: "slime_ice", base: "slime", palette: { body: "#00CED1" } }
{ type: "variant", name: "slime_poison", base: "slime", palette: { body: "#9400D3" } }
```

### Day/Night Variations

```json5
{
  type: "sprite",
  name: "house_day",
  size: [16, 16],
  palette: {
    sky: "#87CEEB",
    window: "#FFFF00",
    wall: "#DEB887",
  },
  regions: {
    sky: { rect: [0, 0, 16, 8] },
    wall: { rect: [4, 8, 8, 8] },
    window: { rect: [6, 10, 4, 3] },
  },
}

{
  type: "variant",
  name: "house_night",
  base: "house_day",
  palette: {
    sky: "#191970",
    window: "#FFA500",
  },
}
```

## Chaining Variants

Variants can be based on other variants:

```json5
{ type: "sprite", name: "base_character", size: [8, 12], palette: { ... }, regions: { ... } }
{ type: "variant", name: "character_evil", base: "base_character", palette: { eyes: "#FF0000" } }
{ type: "variant", name: "character_evil_boss", base: "character_evil", palette: { armor: "#4B0082" } }
```

The boss inherits both the red eyes from `character_evil` and the base structure from `base_character`.

## Variants vs Inline Palettes

When to use each approach:

**Use variants when:**
- You have multiple color swaps of the same sprite
- The base sprite is complex and you don't want to duplicate the regions
- You want to maintain a single source of truth for the structure

**Use inline palettes when:**
- Each sprite is unique
- You're defining a one-off sprite
- The sprite is simple

## Order Matters

The base sprite must be defined before the variant:

```json5
// ERROR: variant before base
{ type: "variant", name: "hero_red", base: "hero", palette: { ... } }
{ type: "sprite", name: "hero", ... }
```

Correct order:

```json5
{ type: "sprite", name: "hero", ... }
{ type: "variant", name: "hero_red", base: "hero", palette: { ... } }
```

## Complete Example

```json5
// Palette
{
  type: "palette",
  name: "gem_base",
  colors: {
    _: "transparent",
    shine: "#FFFFFF",
    body: "#FF0000",
    shadow: "#8B0000",
  },
}

// Base gem sprite
{
  type: "sprite",
  name: "gem",
  size: [4, 4],
  palette: "gem_base",
  regions: {
    body: {
      union: [
        { rect: [1, 0, 2, 1] },
        { rect: [0, 1, 4, 2] },
        { rect: [1, 3, 2, 1] },
      ],
      z: 0,
    },
    shine: { points: [[1, 0], [0, 1]], z: 1 },
    shadow: { points: [[3, 2], [2, 3]], z: 1 },
  },
}

// Color variants
{
  type: "variant",
  name: "gem_blue",
  base: "gem",
  palette: {
    body: "#0000FF",
    shadow: "#00008B",
  },
}

{
  type: "variant",
  name: "gem_green",
  base: "gem",
  palette: {
    body: "#00FF00",
    shadow: "#006400",
  },
}

{
  type: "variant",
  name: "gem_purple",
  base: "gem",
  palette: {
    body: "#9400D3",
    shadow: "#4B0082",
  },
}
```

Four gem sprites from one structure definition.
