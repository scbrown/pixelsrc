# Color Formats

Pixelsrc supports a variety of color formats for defining palette colors. All formats are parsed using a combination of optimized hex parsing and the `lightningcss` library for CSS color notation.

<!-- DEMOS reference/colors#formats -->
<!-- /DEMOS -->

## Quick Reference

| Format | Example | Notes |
|--------|---------|-------|
| Hex (short) | `#RGB`, `#RGBA` | Each digit doubled |
| Hex (long) | `#RRGGBB`, `#RRGGBBAA` | Full precision |
| RGB | `rgb(255, 0, 0)` | Integer or percentage |
| HSL | `hsl(0, 100%, 50%)` | Hue, saturation, lightness |
| HWB | `hwb(0 0% 0%)` | Hue, whiteness, blackness |
| OKLCH | `oklch(0.628 0.258 29.23)` | Perceptually uniform |
| color-mix | `color-mix(in oklch, red 70%, black)` | Blend two colors |
| Named | `red`, `blue`, `transparent` | CSS named colors |

## Hex Colors

The most common format for pixel art. Fast to parse and easy to read.

### Short Form (`#RGB`, `#RGBA`)

Each digit is doubled to get the full value:

| Short | Expands To | Result |
|-------|------------|--------|
| `#F00` | `#FF0000` | Red (255, 0, 0) |
| `#0F0` | `#00FF00` | Green (0, 255, 0) |
| `#00F` | `#0000FF` | Blue (0, 0, 255) |
| `#ABC` | `#AABBCC` | (170, 187, 204) |
| `#F008` | `#FF000088` | Red with ~53% alpha |

### Full Form (`#RRGGBB`, `#RRGGBBAA`)

Full precision hex values:

```
#RRGGBB
 │ │ └── Blue (00-FF)
 │ └──── Green (00-FF)
 └────── Red (00-FF)
```

Without an alpha channel, colors default to fully opaque (alpha = 255).

## CSS Functional Notation

### RGB/RGBA

```json
{"colors": {
  "{red}": "rgb(255, 0, 0)",
  "{green}": "rgb(0, 255, 0)",
  "{semi}": "rgba(255, 0, 0, 0.5)",
  "{modern}": "rgb(255 0 0 / 50%)"
}}
```

### HSL/HSLA

Useful for creating color variations by adjusting lightness:

```json
{"colors": {
  "{red}": "hsl(0, 100%, 50%)",
  "{dark_red}": "hsl(0, 100%, 25%)",
  "{light_red}": "hsl(0, 100%, 75%)"
}}
```

### OKLCH

Perceptually uniform color space. Colors with the same lightness value appear equally bright:

```json
{"colors": {
  "{red}": "oklch(0.628 0.258 29.23)",
  "{green}": "oklch(0.866 0.295 142.5)"
}}
```

## color-mix() Function

Blend two colors in a specified color space. Ideal for generating shadow and highlight variants.

<!-- DEMOS reference/colors#color-mix -->
<!-- /DEMOS -->

### Syntax

```
color-mix(in <color-space>, <color1> [<percentage>], <color2> [<percentage>])
```

### Shadow Generation

Use `oklch` for perceptually uniform darkening:

```json
{"colors": {
  "--skin": "#FFCC99",
  "{skin}": "var(--skin)",
  "{skin_shadow}": "color-mix(in oklch, var(--skin) 70%, black)",
  "{skin_deep}": "color-mix(in oklch, var(--skin) 50%, black)"
}}
```

| Pattern | Effect |
|---------|--------|
| `color-mix(in oklch, <color> 70%, black)` | Light shadow (30% darker) |
| `color-mix(in oklch, <color> 50%, black)` | Medium shadow |
| `color-mix(in oklch, <color> 30%, black)` | Deep shadow |

### Highlight Generation

```json
{"colors": {
  "--primary": "#3366CC",
  "{primary}": "var(--primary)",
  "{primary_light}": "color-mix(in srgb, var(--primary) 70%, white)"
}}
```

### Complete Character Palette

```jsonl
{"type": "palette", "name": "character", "colors": {
  "--skin-base": "#FFCC99",
  "--hair-base": "#5D3A29",

  "{_}": "transparent",
  "{outline}": "#222034",

  "{skin}": "var(--skin-base)",
  "{skin_hi}": "color-mix(in srgb, var(--skin-base) 60%, white)",
  "{skin_sh}": "color-mix(in oklch, var(--skin-base) 70%, black)",

  "{hair}": "var(--hair-base)",
  "{hair_hi}": "color-mix(in srgb, var(--hair-base) 60%, white)",
  "{hair_sh}": "color-mix(in oklch, var(--hair-base) 70%, black)"
}}
```

**Why OKLCH for shadows?** OKLCH provides perceptually uniform blending - a 30% darkening looks equally dark across all hues. With sRGB, some colors darken more dramatically than others.

## Named Colors

All 147 CSS named colors are supported:

| Name | Hex Equivalent |
|------|----------------|
| `transparent` | `#00000000` |
| `black` | `#000000` |
| `white` | `#FFFFFF` |
| `red` | `#FF0000` |
| `green` | `#008000` (not `#00FF00`!) |
| `blue` | `#0000FF` |

**Warning**: CSS `green` is `#008000`. Use `lime` for pure green (`#00FF00`).

## Transparency

### Fully Transparent

Convention: use `{_}` token:

```json
{"colors": {
  "{_}": "transparent",
  "{_}": "#00000000",
  "{_}": "rgba(0, 0, 0, 0)"
}}
```

### Semi-Transparent

```json
{"colors": {
  "{glass}": "#FFFFFF80",
  "{shadow}": "rgba(0, 0, 0, 0.3)"
}}
```

## Error Messages

| Error | Cause | Example |
|-------|-------|---------|
| `empty color string` | Empty value | `""` |
| `color must start with '#'` | Missing hash (for hex) | `"FF0000"` |
| `invalid color length N` | Wrong digit count | `"#FF00"` |
| `invalid hex character 'X'` | Non-hex character | `"#GG0000"` |
| `CSS parse error: ...` | Invalid CSS syntax | `"notacolor"` |

Invalid colors in lenient mode render as magenta (`#FF00FF`) to make them visible.

## Best Practices

1. **Use hex for simple colors** - `#F00` is clearer than `rgb(255, 0, 0)`
2. **Use color-mix for shadows/highlights** - Consistent shading from base colors
3. **Use oklch for shadows** - Perceptually uniform darkening across all hues
4. **Use CSS variables with color-mix** - Define base colors once, derive variants
5. **Use `transparent` over `#00000000`** - More readable
6. **Test at 1x scale** - Ensure colors are distinguishable at native size

## Related

- [CSS Variables](../format/css-variables.md) - Custom properties and var()
- [Built-in Palettes](palettes.md) - Pre-defined color sets
- [Palette Format](../format/palette.md) - Defining custom palettes
