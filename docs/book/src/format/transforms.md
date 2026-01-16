# Transforms

Transforms modify sprites at render time without changing the source definition. They're applied via the `transform` array on a sprite.

## Basic Syntax

```json
{
  "type": "sprite",
  "name": "hero_outlined",
  "source": "hero",
  "transform": [
    {"op": "operation_name", ...options}
  ]
}
```

Transforms are applied in array order.

## Dithering Patterns

Apply dithering patterns for gradients, transparency effects, and texture.

```json
{
  "type": "sprite",
  "name": "gradient",
  "source": "solid",
  "transform": [
    {"op": "dither", "pattern": "checker", "tokens": ["{dark}", "{light}"], "threshold": 0.5}
  ]
}
```

### Dither Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `op` | Yes | - | Must be `"dither"` |
| `pattern` | Yes | - | Dither pattern name |
| `tokens` | Yes | - | Two-element array `[dark, light]` |
| `threshold` | No | 0.5 | Blend threshold (0.0-1.0) |
| `seed` | No | auto | Random seed for noise pattern |

### Built-in Patterns

| Pattern | Description |
|---------|-------------|
| `checker` | 2x2 checkerboard |
| `ordered-2x2` | 2x2 Bayer matrix (4 levels) |
| `ordered-4x4` | 4x4 Bayer matrix (16 levels) |
| `ordered-8x8` | 8x8 Bayer matrix (64 levels) |
| `diagonal` | Diagonal line pattern |
| `horizontal` | Horizontal line pattern |
| `vertical` | Vertical line pattern |
| `noise` | Random dither (seeded) |

### Gradient Dither

Create smooth gradients across the sprite:

```json
{
  "op": "dither-gradient",
  "direction": "vertical",
  "from": "{sky_light}",
  "to": "{sky_dark}",
  "pattern": "ordered-4x4"
}
```

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `op` | Yes | - | Must be `"dither-gradient"` |
| `direction` | Yes | - | `"vertical"`, `"horizontal"`, or `"radial"` |
| `from` | Yes | - | Starting color token |
| `to` | Yes | - | Ending color token |
| `pattern` | No | `"ordered-4x4"` | Dither pattern to use |

## Selective Outline (Sel-out)

Selective outline varies the outline color based on the adjacent fill color, creating softer, more natural edges.

```json
{
  "transform": [
    {"op": "sel-out", "fallback": "{outline}"}
  ]
}
```

### Sel-out Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `op` | Yes | - | Must be `"sel-out"` |
| `fallback` | No | `"{_}"` | Default outline color |
| `auto_darken` | No | 0.3 | Auto-darken factor (0.0-1.0) |
| `mapping` | No | auto | Explicit fill→outline mapping |

### Auto-Darken Mode

By default, sel-out automatically darkens each fill color to create its outline:

```json
{"op": "sel-out", "auto_darken": 0.3}
```

Skin-colored pixels get a darker skin outline, hair pixels get a darker hair outline, etc.

### Explicit Mapping

Define exactly which outline color to use for each fill:

```json
{
  "op": "sel-out",
  "mapping": {
    "{skin}": "{skin_dark}",
    "{hair}": "{hair_dark}",
    "*": "{outline}"
  }
}
```

The `*` key is the fallback for any unspecified fill colors.

## Squash & Stretch

Deform sprites for impact and bounce effects. Classic animation technique.

```json
{
  "transform": [
    {"op": "squash", "amount": 0.3}
  ]
}
```

### Squash/Stretch Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `op` | Yes | - | `"squash"` or `"stretch"` |
| `amount` | Yes | - | Deformation amount (0.0-1.0) |
| `anchor` | No | `"center"` | Transform anchor point |
| `preserve_area` | No | `true` | Maintain sprite area |

### Anchor Points

| Value | Description |
|-------|-------------|
| `"center"` | Center of sprite |
| `"bottom"` | Bottom center |
| `"top"` | Top center |
| `[x, y]` | Custom coordinates |

### Squash vs Stretch

- **Squash**: Compress vertically, expand horizontally (landing, impact)
- **Stretch**: Expand vertically, compress horizontally (jumping, anticipation)

```json
{"type": "sprite", "name": "ball_land", "source": "ball", "transform": [
  {"op": "squash", "amount": 0.4, "anchor": "bottom"}
]}

{"type": "sprite", "name": "ball_jump", "source": "ball", "transform": [
  {"op": "stretch", "amount": 0.3, "anchor": "bottom"}
]}
```

## Chaining Transforms

Apply multiple transforms in sequence:

```json
{
  "type": "sprite",
  "name": "hero_processed",
  "source": "hero",
  "transform": [
    {"op": "sel-out", "auto_darken": 0.25},
    {"op": "dither", "pattern": "checker", "tokens": ["{shadow}", "{_}"], "threshold": 0.3}
  ]
}
```

Transforms apply in array order: first sel-out, then dither.

## Complete Example

```json
{"type": "palette", "name": "character", "colors": {
  "{_}": "#00000000",
  "{skin}": "#FFCC99",
  "{skin_dark}": "#CC9966",
  "{hair}": "#8B4513",
  "{hair_dark}": "#5C2E0A",
  "{outline}": "#000000"
}}

{"type": "sprite", "name": "hero_raw", "palette": "character", "grid": [
  "{_}{_}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{_}{_}{_}{_}{_}"
]}

{"type": "sprite", "name": "hero", "source": "hero_raw", "transform": [
  {"op": "sel-out", "mapping": {
    "{skin}": "{skin_dark}",
    "{hair}": "{hair_dark}",
    "*": "{outline}"
  }}
]}
```

The `hero` sprite has automatic selective outlining based on the fill colors.

## Use Cases

### Retro-Style Gradients

```json
{"op": "dither-gradient", "direction": "vertical", "from": "{sky_top}", "to": "{sky_bottom}", "pattern": "ordered-4x4"}
```

### Soft Shadows

```json
{"op": "dither", "pattern": "ordered-2x2", "tokens": ["{shadow}", "{_}"], "threshold": 0.6}
```

### Impact Effects

```json
{"op": "squash", "amount": 0.5, "anchor": "bottom", "preserve_area": true}
```

### Professional Outlines

```json
{"op": "sel-out", "auto_darken": 0.3}
```
