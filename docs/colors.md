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
3. **Use named colors for semantics** - `coral` is more memorable than `#FF7F50`
4. **Use `transparent` over `#00000000`** - More readable
5. **Stick to one format per palette** - Consistency aids readability
6. **Test at 1x scale** - Ensure colors are distinguishable at native size
