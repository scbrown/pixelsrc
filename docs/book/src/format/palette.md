# Palette

A palette defines named color tokens for use in sprites. Palettes separate color definitions from sprite structure, making it easy to create color variations and maintain consistent color schemes.

## Basic Syntax

```json
{
  "type": "palette",
  "name": "string (required)",
  "colors": {
    "{token}": "#color (required, at least one)"
  }
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"palette"` |
| `name` | Yes | Unique identifier, referenced by sprites |
| `colors` | Yes | Map of token names to color values |

## Example

```json
{"type": "palette", "name": "hero", "colors": {
  "{_}": "#00000000",
  "{skin}": "#FFCC99",
  "{hair}": "#8B4513",
  "{shirt}": "#4169E1",
  "{outline}": "#000000"
}}
```

### Try It

Edit the palette colors and see how they affect the sprite:

<div class="pixelsrc-demo" data-pixelsrc-demo>
  <textarea id="palette-demo">{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{hair}": "#8B4513", "{shirt}": "#4169E1", "{outline}": "#000000"}}
{"type": "sprite", "name": "hero", "palette": "hero", "grid": ["{_}{hair}{hair}{hair}{_}", "{_}{skin}{skin}{skin}{_}", "{outline}{skin}{skin}{skin}{outline}", "{_}{shirt}{shirt}{shirt}{_}", "{_}{shirt}{_}{shirt}{_}"]}</textarea>
  <button onclick="pixelsrcDemo.renderFromTextarea('palette-demo', 'palette-demo-preview')">Try it</button>
  <div class="preview" id="palette-demo-preview"></div>
</div>

Try changing `{hair}` to `#FFD700` (blonde) or `{shirt}` to `#FF0000` (red).

## Color Formats

Palettes accept colors in these hexadecimal formats:

| Format | Example | Expansion |
|--------|---------|-----------|
| `#RGB` | `#F00` | `#FF0000` (red) |
| `#RGBA` | `#F008` | `#FF000088` (red, ~50% transparent) |
| `#RRGGBB` | `#FF0000` | Fully opaque |
| `#RRGGBBAA` | `#FF000080` | With alpha channel |

In addition to hex, all CSS color formats are supported: `rgb()`, `hsl()`, `hwb()`, named colors, and more. See [Color Formats](../reference/colors.md) for details.

## CSS Variables

Define reusable values with CSS custom properties:

```json
{"type": "palette", "name": "themed", "colors": {
  "--primary": "#4169E1",
  "--accent": "#FFD700",
  "{_}": "transparent",
  "{main}": "var(--primary)",
  "{highlight}": "var(--accent)",
  "{fallback}": "var(--missing, #FF6666)"
}}
```

Key features:
- **Define variables**: Use `--name` prefix for variable definitions
- **Reference variables**: Use `var(--name)` to reference
- **Fallback values**: Use `var(--name, fallback)` for undefined variables
- **Forward references**: Variables can reference others defined later

See [CSS Variables](css-variables.md) for complete documentation including theming patterns, nested references, and error handling.

## Reserved Tokens

- `{_}` - Conventional token for transparency (not enforced, but widely used)

## Color Ramps

Generate multiple shades from a base color. Shadows shift hue (toward warm/cool) rather than just darkening, creating more natural-looking color progressions.

```json
{
  "type": "palette",
  "name": "character",
  "ramps": {
    "skin": {
      "base": "#E8B89D",
      "steps": 5,
      "shadow_shift": {"lightness": -15, "hue": 10, "saturation": 5},
      "highlight_shift": {"lightness": 12, "hue": -5, "saturation": -10}
    }
  },
  "colors": {
    "{_}": "#00000000"
  }
}
```

### Ramp Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `ramps` | No | - | Map of ramp name to ramp definition |
| `ramps.{name}.base` | Yes | - | Base color in `#RRGGBB` format |
| `ramps.{name}.steps` | No | 3 | Total steps (odd numbers center on base) |
| `ramps.{name}.shadow_shift` | No | auto | Per-step shift toward shadows |
| `ramps.{name}.highlight_shift` | No | auto | Per-step shift toward highlights |

### Shift Parameters

| Parameter | Range | Description |
|-----------|-------|-------------|
| `lightness` | -100 to 100 | Lightness delta per step |
| `hue` | -180 to 180 | Hue rotation degrees per step |
| `saturation` | -100 to 100 | Saturation delta per step |

### Generated Tokens

For a ramp named `skin` with `steps: 5`:

| Token | Description |
|-------|-------------|
| `{skin_2}` | Darkest shadow (2 steps dark) |
| `{skin_1}` | Shadow (1 step dark) |
| `{skin}` | Base color |
| `{skin+1}` | Highlight (1 step light) |
| `{skin+2}` | Brightest (2 steps light) |

## Inline Color Derivation

Create single-color variants without defining full ramps:

```json
{
  "colors": {
    "{skin}": "#E8B89D",
    "{skin_shadow}": {"from": "{skin}", "shift": {"lightness": -20, "hue": 15}},
    "{skin_highlight}": {"from": "{skin}", "shift": {"lightness": 15, "hue": -10}}
  }
}
```

This derives `{skin_shadow}` and `{skin_highlight}` from the base `{skin}` color with specified shifts.

## Referencing Palettes

Sprites can reference palettes in three ways:

### Named Reference

Reference a palette defined earlier in the file:

```json
{"type": "palette", "name": "mono", "colors": {"{on}": "#FFF", "{off}": "#000"}}
{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}
```

### Inline Colors

Define colors directly in the sprite (no separate palette object):

```json
{"type": "sprite", "name": "dot", "palette": {"{on}": "#FFF", "{_}": "#0000"}, "grid": ["{on}"]}
```

### Built-in Palettes

Reference built-in palettes with the `@` prefix:

```json
{"type": "sprite", "name": "retro", "palette": "@gameboy", "grid": [...]}
```

See [Built-in Palettes](../reference/palettes.md) for available options.

## Order Matters

Palettes must be defined before sprites that reference them by name. Forward references cause errors:

```json
// ERROR: hero_sprite references "hero" palette before it's defined
{"type": "sprite", "name": "hero_sprite", "palette": "hero", "grid": [...]}
{"type": "palette", "name": "hero", "colors": {...}}
```

Correct order:

```json
{"type": "palette", "name": "hero", "colors": {...}}
{"type": "sprite", "name": "hero_sprite", "palette": "hero", "grid": [...]}
```

## Multiple Sprites, One Palette

A palette can be shared across multiple sprites:

```json
{"type": "palette", "name": "ui", "colors": {
  "{_}": "#0000",
  "{bg}": "#2D2D2D",
  "{border}": "#4A4A4A",
  "{text}": "#FFFFFF"
}}

{"type": "sprite", "name": "button", "palette": "ui", "grid": [...]}
{"type": "sprite", "name": "panel", "palette": "ui", "grid": [...]}
{"type": "sprite", "name": "icon", "palette": "ui", "grid": [...]}
```

Changing a color in the palette updates all sprites that use it.
