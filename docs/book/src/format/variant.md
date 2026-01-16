# Variant

A variant creates a color variation of an existing sprite without duplicating the pixel grid. This is useful for creating enemy color swaps, team colors, alternate costumes, and similar variations.

## Basic Syntax

```json
{
  "type": "variant",
  "name": "string (required)",
  "base": "string (required)",
  "palette": { "{token}": "#color", ... }
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

```json
{"type": "sprite", "name": "hero", "palette": {
  "{skin}": "#FFCC99",
  "{hair}": "#8B4513",
  "{shirt}": "#4169E1"
}, "grid": [...]}

{"type": "variant", "name": "hero_red", "base": "hero", "palette": {
  "{shirt}": "#DC143C"
}}

{"type": "variant", "name": "hero_green", "base": "hero", "palette": {
  "{shirt}": "#228B22"
}}
```

All three sprites share the same grid. Only the shirt color differs.

## Behavior

- **Inherits grid and size** from the base sprite
- **Only specified tokens are overridden** - unspecified tokens keep their base colors
- **Base sprite must be defined first** - forward references are errors

## Partial Overrides

You don't need to override all colors. Only specify the tokens you want to change:

```json
{"type": "sprite", "name": "knight", "palette": {
  "{armor}": "#808080",
  "{plume}": "#FF0000",
  "{skin}": "#FFCC99",
  "{eyes}": "#0000FF"
}, "grid": [...]}

{"type": "variant", "name": "knight_gold", "base": "knight", "palette": {
  "{armor}": "#FFD700",
  "{plume}": "#FFFFFF"
}}
```

The `knight_gold` variant has gold armor and white plume, but keeps the same skin and eye colors.

## Use Cases

### Team Colors

```json
{"type": "sprite", "name": "player", "palette": {"{team}": "#FF0000"}, "grid": [...]}
{"type": "variant", "name": "player_blue", "base": "player", "palette": {"{team}": "#0000FF"}}
{"type": "variant", "name": "player_green", "base": "player", "palette": {"{team}": "#00FF00"}}
```

### Enemy Variants

```json
{"type": "sprite", "name": "slime", "palette": {"{body}": "#00FF00", "{eyes}": "#FFFFFF"}, "grid": [...]}
{"type": "variant", "name": "slime_fire", "base": "slime", "palette": {"{body}": "#FF4500"}}
{"type": "variant", "name": "slime_ice", "base": "slime", "palette": {"{body}": "#00CED1"}}
{"type": "variant", "name": "slime_poison", "base": "slime", "palette": {"{body}": "#9400D3"}}
```

### Day/Night Variations

```json
{"type": "sprite", "name": "house_day", "palette": {
  "{sky}": "#87CEEB",
  "{window}": "#FFFF00",
  "{wall}": "#DEB887"
}, "grid": [...]}

{"type": "variant", "name": "house_night", "base": "house_day", "palette": {
  "{sky}": "#191970",
  "{window}": "#FFA500"
}}
```

## Chaining Variants

Variants can be based on other variants:

```json
{"type": "sprite", "name": "base_character", "palette": {...}, "grid": [...]}
{"type": "variant", "name": "character_evil", "base": "base_character", "palette": {"{eyes}": "#FF0000"}}
{"type": "variant", "name": "character_evil_boss", "base": "character_evil", "palette": {"{armor}": "#4B0082"}}
```

The boss inherits both the red eyes from `character_evil` and the base grid from `base_character`.

## Variants vs Inline Palettes

When to use each approach:

**Use variants when:**
- You have multiple color swaps of the same sprite
- The base sprite is complex and you don't want to duplicate the grid
- You want to maintain a single source of truth for the pixel layout

**Use inline palettes when:**
- Each sprite is unique
- You're defining a one-off sprite
- The sprite is simple (a few pixels)

## Order Matters

The base sprite must be defined before the variant:

```json
// ERROR: variant before base
{"type": "variant", "name": "hero_red", "base": "hero", "palette": {...}}
{"type": "sprite", "name": "hero", "palette": {...}, "grid": [...]}
```

Correct order:

```json
{"type": "sprite", "name": "hero", "palette": {...}, "grid": [...]}
{"type": "variant", "name": "hero_red", "base": "hero", "palette": {...}}
```

## Complete Example

```json
{"type": "palette", "name": "gem_base", "colors": {
  "{_}": "#00000000",
  "{shine}": "#FFFFFF",
  "{body}": "#FF0000",
  "{shadow}": "#8B0000"
}}

{"type": "sprite", "name": "gem", "palette": "gem_base", "grid": [
  "{_}{shine}{body}{_}",
  "{shine}{body}{body}{shadow}",
  "{body}{body}{shadow}{shadow}",
  "{_}{shadow}{shadow}{_}"
]}

{"type": "variant", "name": "gem_blue", "base": "gem", "palette": {
  "{body}": "#0000FF",
  "{shadow}": "#00008B"
}}

{"type": "variant", "name": "gem_green", "base": "gem", "palette": {
  "{body}": "#00FF00",
  "{shadow}": "#006400"
}}

{"type": "variant", "name": "gem_purple", "base": "gem", "palette": {
  "{body}": "#9400D3",
  "{shadow}": "#4B0082"
}}
```

Four gem sprites from one grid definition.
