# Color Format Reference

Pixelsrc supports a variety of color formats for defining palette colors. All formats are parsed using a combination of optimized hex parsing and the `lightningcss` library for CSS color notation.

## Quick Reference

| Format | Example | Notes |
|--------|---------|-------|
| Hex (short) | `#RGB`, `#RGBA` | Each digit doubled |
| Hex (long) | `#RRGGBB`, `#RRGGBBAA` | Full precision |
| RGB | `rgb(255, 0, 0)` | Integer or percentage |
| RGBA | `rgba(255, 0, 0, 0.5)` | With alpha |
| HSL | `hsl(0, 100%, 50%)` | Hue, saturation, lightness |
| HSLA | `hsla(0, 100%, 50%, 0.5)` | With alpha |
| HWB | `hwb(0 0% 0%)` | Hue, whiteness, blackness |
| OKLCH | `oklch(0.628 0.258 29.23)` | Perceptually uniform |
| color-mix | `color-mix(in oklch, red 70%, black)` | Blend two colors |
| Named | `red`, `blue`, `transparent` | CSS named colors |

## Hex Colors

The most common format for pixel art. Fast to parse and easy to read.

### Short Form (`#RGB`, `#RGBA`)

Each digit is doubled to get the full value:
- `#F00` → `#FF0000` (red)
- `#0F0` → `#00FF00` (green)
- `#00F` → `#0000FF` (blue)
- `#F00F` → `#FF0000FF` (red, opaque)
- `#F008` → `#FF000088` (red, ~50% alpha)

### Long Form (`#RRGGBB`, `#RRGGBBAA`)

Full precision hex values:
- `#FF0000` → red (alpha defaults to 255)
- `#00FF00` → green
- `#0000FF` → blue
- `#FF000080` → red, 50% alpha
- `#00000000` → fully transparent

**Case insensitive**: `#ff0000`, `#FF0000`, and `#Ff0000` are equivalent.

## RGB/RGBA Functional Notation

Standard CSS RGB notation with integer or percentage values.

### Basic Syntax

```
rgb(red, green, blue)
rgba(red, green, blue, alpha)
```

### Integer Values (0-255)

```json
{"colors": {
  "{red}": "rgb(255, 0, 0)",
  "{green}": "rgb(0, 255, 0)",
  "{blue}": "rgb(0, 0, 255)",
  "{semi}": "rgba(255, 0, 0, 0.5)"
}}
```

### Percentage Values

```json
{"colors": {
  "{red}": "rgb(100%, 0%, 0%)",
  "{half}": "rgb(50%, 50%, 50%)"
}}
```

### Modern Space Syntax

CSS Color Level 4 space-separated syntax with optional alpha:

```json
{"colors": {
  "{red}": "rgb(255 0 0)",
  "{semi}": "rgb(255 0 0 / 50%)",
  "{quarter}": "rgb(255 0 0 / 0.25)"
}}
```

## HSL/HSLA Functional Notation

Hue-Saturation-Lightness model. Useful for creating color variations.

### Parameters

- **Hue**: Angle on the color wheel (0-360)
  - 0° = red
  - 120° = green
  - 240° = blue
- **Saturation**: 0% (gray) to 100% (full color)
- **Lightness**: 0% (black) to 100% (white), 50% is pure color

### Examples

```json
{"colors": {
  "{red}": "hsl(0, 100%, 50%)",
  "{green}": "hsl(120, 100%, 50%)",
  "{blue}": "hsl(240, 100%, 50%)",
  "{dark_red}": "hsl(0, 100%, 25%)",
  "{light_red}": "hsl(0, 100%, 75%)",
  "{muted_red}": "hsl(0, 50%, 50%)"
}}
```

### Modern Syntax

```json
{"colors": {
  "{red}": "hsl(0deg 100% 50%)",
  "{semi}": "hsl(0 100% 50% / 50%)"
}}
```

## HWB Functional Notation

Hue-Whiteness-Blackness model. Intuitive for mixing colors with white/black.

### Parameters

- **Hue**: Same as HSL (0-360)
- **Whiteness**: 0% (no white) to 100% (pure white)
- **Blackness**: 0% (no black) to 100% (pure black)

### Examples

```json
{"colors": {
  "{red}": "hwb(0 0% 0%)",
  "{pink}": "hwb(0 50% 0%)",
  "{dark_red}": "hwb(0 0% 50%)",
  "{gray}": "hwb(0 50% 50%)",
  "{white}": "hwb(0 100% 0%)",
  "{black}": "hwb(0 0% 100%)"
}}
```

## OKLCH Functional Notation

Perceptually uniform color space. Colors with the same lightness value appear equally bright.

### Parameters

- **Lightness**: 0 (black) to 1 (white)
- **Chroma**: 0 (gray) to ~0.4 (saturated)
- **Hue**: Angle in degrees (0-360)

### Examples

```json
{"colors": {
  "{red}": "oklch(0.628 0.258 29.23)",
  "{green}": "oklch(0.866 0.295 142.5)",
  "{blue}": "oklch(0.452 0.313 264.1)"
}}
```

**Note**: OKLCH values may have slight rounding differences when converted to sRGB.

## Named Colors

All 147 CSS named colors are supported, plus `transparent`.

### Common Colors

| Name | Hex Equivalent |
|------|----------------|
| `transparent` | `#00000000` |
| `black` | `#000000` |
| `white` | `#FFFFFF` |
| `red` | `#FF0000` |
| `green` | `#008000` (not `#00FF00`!) |
| `blue` | `#0000FF` |
| `yellow` | `#FFFF00` |
| `cyan` | `#00FFFF` |
| `magenta` | `#FF00FF` |

**Warning**: CSS `green` is `#008000`, not `#00FF00`. Use `lime` for pure green.

### Extended Colors

```json
{"colors": {
  "{skin}": "peachpuff",
  "{hair}": "saddlebrown",
  "{eyes}": "steelblue",
  "{lips}": "coral",
  "{blush}": "hotpink"
}}
```

**Case insensitive**: `Red`, `RED`, and `red` are equivalent.

## Usage in Palettes

All color formats can be used anywhere a color value is expected:

```jsonl
{"type": "palette", "name": "mixed", "colors": {
  "{_}": "transparent",
  "{skin}": "#FFCC99",
  "{hair}": "saddlebrown",
  "{outline}": "rgb(0, 0, 0)",
  "{highlight}": "hsl(60, 100%, 90%)",
  "{shadow}": "hwb(0 0% 70%)"
}}
```

## Transparency

### Fully Transparent

Convention: use `{_}` token mapped to transparent:

```json
{"colors": {"{_}": "#00000000"}}
{"colors": {"{_}": "transparent"}}
{"colors": {"{_}": "rgba(0, 0, 0, 0)"}}
```

### Semi-Transparent

```json
{"colors": {
  "{glass}": "#FFFFFF80",
  "{shadow}": "rgba(0, 0, 0, 0.3)",
  "{ghost}": "hsl(200 50% 50% / 50%)"
}}
```

## color-mix() Function

The `color-mix()` function blends two colors in a specified color space. This is particularly useful for generating shadow and highlight variants from a base color.

### Syntax

```
color-mix(in <color-space>, <color1> [<percentage>], <color2> [<percentage>])
```

**Color spaces:**
- `srgb` - Standard RGB (linear blending)
- `oklch` - Perceptually uniform (recommended for shadows/highlights)
- `hsl` - Hue-saturation-lightness
- `hwb` - Hue-whiteness-blackness

### Basic Examples

```json
{"colors": {
  "{purple}": "color-mix(in srgb, red 50%, blue)",
  "{gray}": "color-mix(in srgb, white, black)",
  "{teal}": "color-mix(in oklch, blue 60%, green)"
}}
```

### Shadow Generation

Create darker variants by mixing with black. Use `oklch` for perceptually uniform darkening:

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

Create lighter variants by mixing with white:

```json
{"colors": {
  "--primary": "#3366CC",
  "{primary}": "var(--primary)",
  "{primary_light}": "color-mix(in srgb, var(--primary) 70%, white)",
  "{primary_bright}": "color-mix(in srgb, var(--primary) 50%, white)"
}}
```

| Pattern | Effect |
|---------|--------|
| `color-mix(in srgb, <color> 70%, white)` | Subtle highlight |
| `color-mix(in srgb, <color> 50%, white)` | Medium highlight |
| `color-mix(in srgb, <color> 30%, white)` | Bright highlight |

### Pixel Art Character Palette Example

Complete palette using color-mix for consistent shading:

```jsonl
{"type": "palette", "name": "character", "colors": {
  "--skin-base": "#FFCC99",
  "--hair-base": "#5D3A29",
  "--shirt-base": "#3366CC",

  "{_}": "transparent",
  "{outline}": "#222034",

  "{skin}": "var(--skin-base)",
  "{skin_hi}": "color-mix(in srgb, var(--skin-base) 60%, white)",
  "{skin_sh}": "color-mix(in oklch, var(--skin-base) 70%, black)",

  "{hair}": "var(--hair-base)",
  "{hair_hi}": "color-mix(in srgb, var(--hair-base) 60%, white)",
  "{hair_sh}": "color-mix(in oklch, var(--hair-base) 70%, black)",

  "{shirt}": "var(--shirt-base)",
  "{shirt_hi}": "color-mix(in srgb, var(--shirt-base) 60%, white)",
  "{shirt_sh}": "color-mix(in oklch, var(--shirt-base) 70%, black)"
}}
```

### Why OKLCH for Shadows?

OKLCH provides perceptually uniform blending - a 30% darkening looks equally dark across all hues. With sRGB, some colors (like yellow) appear to darken more dramatically than others.

```json
{"colors": {
  "{yellow_srgb}": "color-mix(in srgb, yellow 70%, black)",
  "{yellow_oklch}": "color-mix(in oklch, yellow 70%, black)",
  "{blue_srgb}": "color-mix(in srgb, blue 70%, black)",
  "{blue_oklch}": "color-mix(in oklch, blue 70%, black)"
}}
```

**Recommendation:** Use `oklch` for shadows, `srgb` for highlights.

## Error Handling

Invalid colors produce a `ColorError`:

| Error | Example | Message |
|-------|---------|---------|
| Empty | `""` | "empty color string" |
| Invalid length | `#FF` | "invalid color length 2, expected 3, 4, 6, or 8" |
| Invalid hex | `#GGG` | "invalid hex character 'G'" |
| CSS parse error | `notacolor` | "CSS parse error: ..." |

Invalid colors in lenient mode render as magenta (`#FF00FF`) to make them visible.

## Best Practices

1. **Use hex for simple colors** - `#F00` is clearer than `rgb(255, 0, 0)`
2. **Use HSL for variations** - Easy to create light/dark/muted versions
3. **Use color-mix for shadows/highlights** - Consistent shading from base colors
4. **Use oklch for shadows** - Perceptually uniform darkening across all hues
5. **Use CSS variables with color-mix** - Define base colors once, derive variants
6. **Use named colors for semantics** - `coral` is more memorable than `#FF7F50`
7. **Use `transparent` over `#00000000`** - More readable
8. **Test at 1x scale** - Ensure colors are distinguishable at native size
