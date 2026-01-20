# Transforms

Transforms modify sprites at render time without changing the source definition. Pixelsrc supports two transform systems:

- **CSS Transforms**: Translate, rotate, scale, and flip with CSS-like syntax (used in keyframe animations)
- **Effect Transforms**: Dithering, selective outline, squash/stretch (applied via transform arrays)

---

## CSS Transforms

CSS transforms use familiar web syntax and are primarily used within keyframe animations. They're designed for pixel art with specific considerations for crisp rendering.

### Syntax

```json
{
  "type": "animation",
  "name": "example",
  "keyframes": {
    "0%": { "sprite": "star", "transform": "translate(0, 0) rotate(0deg)" },
    "100%": { "sprite": "star", "transform": "translate(10, -5) rotate(90deg)" }
  },
  "duration": "1s"
}
```

Multiple transforms can be combined in a single string, separated by spaces.

### Transform Functions

| Function | Syntax | Description |
|----------|--------|-------------|
| `translate` | `translate(x, y)` | Move by x, y pixels |
| `rotate` | `rotate(deg)` | Rotate clockwise |
| `scale` | `scale(n)` or `scale(x, y)` | Scale uniformly or non-uniformly |
| `flip` | `flip(x)` or `flip(y)` | Horizontal or vertical flip |

### translate(x, y)

Move the sprite by the specified pixel offset.

```json
"transform": "translate(10, 5)"      // Move right 10, down 5
"transform": "translate(-5, 0)"      // Move left 5
"transform": "translate(10px, 5px)"  // Optional px suffix
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `x` | integer | Horizontal offset (positive = right) |
| `y` | integer | Vertical offset (positive = down, optional, defaults to 0) |

<!-- DEMOS format/transforms#translate -->
<!-- /DEMOS -->

### rotate(deg)

Rotate the sprite clockwise by the specified angle.

```json
"transform": "rotate(90deg)"   // Quarter turn clockwise
"transform": "rotate(180)"     // Half turn (deg suffix optional)
"transform": "rotate(270deg)"  // Three-quarter turn
```

**Pixel Art Considerations:**

For crisp pixel art, use 90-degree increments (0, 90, 180, 270). Other angles work but may produce anti-aliased edges:

| Angle | Result |
|-------|--------|
| 0, 90, 180, 270 | Pixel-perfect rotation |
| 45, 135, 225, 315 | Diagonal, some blurring |
| Other values | Arbitrary rotation with anti-aliasing |

<!-- DEMOS format/transforms#rotate -->
<!-- /DEMOS -->

### scale(n) or scale(x, y)

Scale the sprite uniformly or non-uniformly.

```json
"transform": "scale(2)"        // Double size (uniform)
"transform": "scale(2, 1)"     // Double width only
"transform": "scale(1, 0.5)"   // Half height
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `n` | float | Uniform scale factor (must be positive) |
| `x, y` | float | Non-uniform scale factors |

Alternative syntax:

```json
"transform": "scaleX(2)"       // Scale width only
"transform": "scaleY(0.5)"     // Scale height only
```

**Pixel Art Considerations:**

Integer scale factors (2, 3, 4) maintain pixel-perfect appearance. Fractional scales may blur:

| Scale | Result |
|-------|--------|
| 1, 2, 3, 4... | Pixel-perfect scaling |
| 1.5, 2.5... | Blended pixels |
| 0.5, 0.25... | Reduced resolution |

<!-- DEMOS format/transforms#scale -->
<!-- /DEMOS -->

### flip(x) or flip(y)

Mirror the sprite horizontally or vertically.

```json
"transform": "flip(x)"         // Horizontal mirror
"transform": "flip(y)"         // Vertical mirror
"transform": "flip(x) flip(y)" // Both axes
```

Alternative syntax:

```json
"transform": "flipX()"         // Same as flip(x)
"transform": "flipY()"         // Same as flip(y)
"transform": "flip(h)"         // Horizontal (alias)
"transform": "flip(v)"         // Vertical (alias)
```

Flipping is always pixel-perfect with no quality loss.

<!-- DEMOS format/transforms#flip -->
<!-- /DEMOS -->

### Transform Order

Transforms apply in CSS order: translate → rotate → scale → flip.

```json
"transform": "translate(10, 0) rotate(90deg) scale(2)"
```

This translates first, then rotates, then scales.

### Complete Examples

**Spinning animation:**

```json
{"type": "animation", "name": "spin", "keyframes": {"0%": {"sprite": "star", "transform": "rotate(0deg)"}, "100%": {"sprite": "star", "transform": "rotate(360deg)"}}, "duration": 1000, "timing_function": "linear"}
```

**Pulsing scale effect:**

```json
{"type": "animation", "name": "pulse", "keyframes": {"0%": {"sprite": "star", "transform": "scale(1)", "opacity": 1.0}, "50%": {"sprite": "star", "transform": "scale(1.5)", "opacity": 0.5}, "100%": {"sprite": "star", "transform": "scale(1)", "opacity": 1.0}}, "duration": "2s", "timing_function": "ease-in-out"}
```

**Bouncing with translation:**

```json
{"type": "animation", "name": "bounce", "keyframes": {"0%": {"sprite": "ball", "transform": "translate(0, 0)"}, "50%": {"sprite": "ball", "transform": "translate(0, -10)"}, "100%": {"sprite": "ball", "transform": "translate(0, 0)"}}, "duration": "500ms", "timing_function": "ease-in-out"}
```

**Flip on hover (walk cycle):**

```json
{"type": "sprite", "name": "walk_left", "source": "walk_right", "transform": [{"op": "mirror-h"}]}
```

---

## Sprite Transforms (Derived Sprites)

Create new sprites derived from existing ones using transforms. These are specified via the `transform` array on a sprite definition.

### String Syntax (Simple Operations)

For common geometric operations, use string syntax:

```json
{"type": "sprite", "name": "hero_left", "source": "hero_right", "transform": ["mirror-h"]}
{"type": "sprite", "name": "hero_down", "source": "hero_right", "transform": ["rotate:90"]}
{"type": "sprite", "name": "hero_big", "source": "hero", "transform": ["scale:2,2"]}
{"type": "sprite", "name": "hero_shadow", "source": "hero", "transform": ["shadow:1,1:{shadow}"]}
```

| Operation | String Syntax | Description |
|-----------|---------------|-------------|
| Mirror H | `"mirror-h"` | Flip horizontally (left-right) |
| Mirror V | `"mirror-v"` | Flip vertically (top-bottom) |
| Rotate | `"rotate:90"` | Rotate 90°, 180°, or 270° clockwise |
| Scale | `"scale:2,2"` | Scale by X,Y factors |
| Shift | `"shift:1,1"` | Shift pixels by X,Y offset |
| Shadow | `"shadow:1,1:{token}"` | Add drop shadow at offset with token |
| Sel-out | `"sel-out"` or `"sel-out:{fallback}"` | Selective outline |

Aliases: `flip-h` = `mirror-h`, `flip-v` = `mirror-v`, `rot` = `rotate`

### Object Syntax (Advanced Operations)

For operations with multiple parameters, use object syntax:

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

---

## Effect Transforms

Effect transforms modify sprite appearance through arrays of operations.

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
