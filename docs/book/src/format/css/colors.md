# CSS Colors

Pixelsrc supports the full range of CSS color formats, allowing you to use familiar web color syntax in your palette definitions.

## Hex Colors

<!-- DEMOS format/css/colors#hex -->
**Tests hex color formats: #rgb, #rrggbb, #rrggbbaa**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "hex_short", "colors": {"{_}": "#00000000", "{r}": "#F00", "{g}": "#0F0", "{b}": "#00F"}}
{"type": "palette", "name": "hex_full", "colors": {"{_}": "#00000000", "{r}": "#FF0000", "{g}": "#00FF00", "{b}": "#0000FF"}}
{"type": "palette", "name": "hex_alpha", "colors": {"{_}": "#00000000", "{a1}": "#FF000080", "{a2}": "#FF0000C0", "{a3}": "#FF0000FF"}}
{"type": "sprite", "name": "rgb_short", "palette": "hex_short", "size": [3, 1], "regions": {"r": {"points": [[0, 0]]}, "g": {"points": [[1, 0]]}, "b": {"points": [[2, 0]]}}}
{"type": "sprite", "name": "alpha_gradient", "palette": "hex_alpha", "size": [3, 1], "regions": {"a1": {"points": [[0, 0]]}, "a2": {"points": [[1, 0]]}, "a3": {"points": [[2, 0]]}}}
```

</div>

<div class="demo-container" data-demo="hex">
</div>
<!-- /DEMOS -->

Standard hexadecimal color notation:

| Format | Example | Description |
|--------|---------|-------------|
| `#RGB` | `#F00` | Short form (expands to #RRGGBB) |
| `#RRGGBB` | `#FF0000` | Full form |
| `#RRGGBBAA` | `#FF0000FF` | With alpha channel |

## RGB Colors

<!-- DEMOS format/css/colors#rgb -->
**Tests rgb() and rgba() color functions**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "rgb_basic", "colors": {"{_}": "#00000000", "{r}": "rgb(255, 0, 0)", "{g}": "rgb(0, 255, 0)", "{b}": "rgb(0, 0, 255)"}}
{"type": "palette", "name": "rgba_alpha", "colors": {"{_}": "#00000000", "{a1}": "rgba(255, 0, 0, 0.5)", "{a2}": "rgba(255, 0, 0, 0.75)", "{a3}": "rgba(255, 0, 0, 1.0)"}}
{"type": "sprite", "name": "rgb_demo", "palette": "rgb_basic", "size": [3, 1], "regions": {"r": {"points": [[0, 0]]}, "g": {"points": [[1, 0]]}, "b": {"points": [[2, 0]]}}}
{"type": "sprite", "name": "rgba_gradient", "palette": "rgba_alpha", "size": [3, 1], "regions": {"a1": {"points": [[0, 0]]}, "a2": {"points": [[1, 0]]}, "a3": {"points": [[2, 0]]}}}
```

</div>

<div class="demo-container" data-demo="rgb">
</div>
<!-- /DEMOS -->

Functional RGB notation:

```json
{
  "colors": {
    "{red}": "rgb(255, 0, 0)",
    "{green}": "rgb(0, 255, 0)",
    "{semi}": "rgba(255, 0, 0, 0.5)"
  }
}
```

| Format | Example | Description |
|--------|---------|-------------|
| `rgb(r, g, b)` | `rgb(255, 128, 0)` | RGB values (0-255) |
| `rgba(r, g, b, a)` | `rgba(255, 128, 0, 0.5)` | With alpha (0.0-1.0) |
| `rgb(r g b)` | `rgb(255 128 0)` | Space-separated (CSS4) |
| `rgb(r g b / a)` | `rgb(255 128 0 / 50%)` | CSS4 with alpha |

## HSL Colors

<!-- DEMOS format/css/colors#hsl -->
**Tests hsl() and hsla() color functions**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "hsl_basic", "colors": {"{_}": "#00000000", "{r}": "hsl(0, 100%, 50%)", "{g}": "hsl(120, 100%, 50%)", "{b}": "hsl(240, 100%, 50%)"}}
{"type": "palette", "name": "hsl_saturation", "colors": {"{_}": "#00000000", "{s0}": "hsl(0, 0%, 50%)", "{s50}": "hsl(0, 50%, 50%)", "{s100}": "hsl(0, 100%, 50%)"}}
{"type": "palette", "name": "hsl_lightness", "colors": {"{_}": "#00000000", "{l25}": "hsl(0, 100%, 25%)", "{l50}": "hsl(0, 100%, 50%)", "{l75}": "hsl(0, 100%, 75%)"}}
{"type": "sprite", "name": "hsl_demo", "palette": "hsl_basic", "size": [3, 1], "regions": {"r": {"points": [[0, 0]]}, "g": {"points": [[1, 0]]}, "b": {"points": [[2, 0]]}}}
{"type": "sprite", "name": "saturation_demo", "palette": "hsl_saturation", "size": [3, 1], "regions": {"s0": {"points": [[0, 0]]}, "s50": {"points": [[1, 0]]}, "s100": {"points": [[2, 0]]}}}
```

</div>

<div class="demo-container" data-demo="hsl">
</div>
<!-- /DEMOS -->

Hue-saturation-lightness notation:

```json
{
  "colors": {
    "{red}": "hsl(0, 100%, 50%)",
    "{muted}": "hsl(0, 50%, 50%)",
    "{dark}": "hsl(0, 100%, 25%)"
  }
}
```

| Parameter | Range | Description |
|-----------|-------|-------------|
| Hue | 0-360 | Color wheel angle (0=red, 120=green, 240=blue) |
| Saturation | 0-100% | Color intensity (0=gray, 100=vivid) |
| Lightness | 0-100% | Brightness (0=black, 50=normal, 100=white) |

## OKLCH Colors

<!-- DEMOS format/css/colors#oklch -->
**Tests oklch() color function (perceptually uniform)**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "oklch_basic", "colors": {"{_}": "#00000000", "{r}": "oklch(0.6 0.2 30)", "{g}": "oklch(0.8 0.2 150)", "{b}": "oklch(0.5 0.2 270)"}}
{"type": "palette", "name": "oklch_lightness", "colors": {"{_}": "#00000000", "{l25}": "oklch(0.25 0.2 30)", "{l50}": "oklch(0.5 0.2 30)", "{l75}": "oklch(0.75 0.2 30)"}}
{"type": "palette", "name": "oklch_chroma", "colors": {"{_}": "#00000000", "{c0}": "oklch(0.6 0 30)", "{c1}": "oklch(0.6 0.1 30)", "{c2}": "oklch(0.6 0.2 30)"}}
{"type": "sprite", "name": "oklch_demo", "palette": "oklch_basic", "size": [3, 1], "regions": {"r": {"points": [[0, 0]]}, "g": {"points": [[1, 0]]}, "b": {"points": [[2, 0]]}}}
```

</div>

<div class="demo-container" data-demo="oklch">
</div>
<!-- /DEMOS -->

Perceptually uniform color space for consistent brightness:

```json
{
  "colors": {
    "{vibrant}": "oklch(70% 0.15 30)",
    "{muted}": "oklch(70% 0.05 30)"
  }
}
```

| Parameter | Range | Description |
|-----------|-------|-------------|
| Lightness | 0-100% | Perceived brightness |
| Chroma | 0-0.4 | Color intensity |
| Hue | 0-360 | Color angle |

OKLCH is ideal for generating consistent color palettes where colors have the same perceived brightness.

## HWB Colors

<!-- DEMOS format/css/colors#hwb -->
**Tests hwb() color function (hue-whiteness-blackness)**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "hwb_basic", "colors": {"{_}": "#00000000", "{r}": "hwb(0 0% 0%)", "{g}": "hwb(120 0% 0%)", "{b}": "hwb(240 0% 0%)"}}
{"type": "palette", "name": "hwb_whiteness", "colors": {"{_}": "#00000000", "{w0}": "hwb(0 0% 0%)", "{w25}": "hwb(0 25% 0%)", "{w50}": "hwb(0 50% 0%)"}}
{"type": "palette", "name": "hwb_blackness", "colors": {"{_}": "#00000000", "{b0}": "hwb(0 0% 0%)", "{b25}": "hwb(0 0% 25%)", "{b50}": "hwb(0 0% 50%)"}}
{"type": "sprite", "name": "hwb_demo", "palette": "hwb_basic", "size": [3, 1], "regions": {"r": {"points": [[0, 0]]}, "g": {"points": [[1, 0]]}, "b": {"points": [[2, 0]]}}}
```

</div>

<div class="demo-container" data-demo="hwb">
</div>
<!-- /DEMOS -->

Hue-whiteness-blackness notation:

```json
{
  "colors": {
    "{pure}": "hwb(0 0% 0%)",
    "{tinted}": "hwb(0 20% 0%)",
    "{shaded}": "hwb(0 0% 20%)"
  }
}
```

| Parameter | Range | Description |
|-----------|-------|-------------|
| Hue | 0-360 | Color wheel angle |
| Whiteness | 0-100% | Amount of white mixed in |
| Blackness | 0-100% | Amount of black mixed in |

## Named Colors

<!-- DEMOS format/css/colors#named -->
**Tests CSS named colors (red, blue, coral, etc.)**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "named_basic", "colors": {"{_}": "#00000000", "{r}": "red", "{g}": "green", "{b}": "blue"}}
{"type": "palette", "name": "named_warm", "colors": {"{_}": "#00000000", "{c}": "coral", "{o}": "orange", "{g}": "gold"}}
{"type": "palette", "name": "named_cool", "colors": {"{_}": "#00000000", "{t}": "teal", "{c}": "cyan", "{a}": "aqua"}}
{"type": "palette", "name": "named_neutral", "colors": {"{_}": "#00000000", "{w}": "white", "{s}": "silver", "{g}": "gray"}}
{"type": "sprite", "name": "named_demo", "palette": "named_basic", "size": [3, 1], "regions": {"r": {"points": [[0, 0]]}, "g": {"points": [[1, 0]]}, "b": {"points": [[2, 0]]}}}
{"type": "sprite", "name": "grayscale", "palette": "named_neutral", "size": [3, 1], "regions": {"w": {"points": [[0, 0]]}, "s": {"points": [[1, 0]]}, "g": {"points": [[2, 0]]}}}
```

</div>

<div class="demo-container" data-demo="named">
</div>
<!-- /DEMOS -->

CSS named colors are supported:

```json
{
  "colors": {
    "{primary}": "royalblue",
    "{accent}": "gold",
    "{bg}": "darkslategray"
  }
}
```

All 147 CSS named colors are available, including `transparent`.

## color-mix()

<!-- DEMOS format/css/colors#color_mix -->
**Tests color-mix() function for blending colors**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "color_mix_basic", "colors": {"{_}": "#00000000", "{m}": "color-mix(in srgb, red 50%, blue 50%)"}}
{"type": "palette", "name": "shadows_oklch", "colors": {"{_}": "#00000000", "{s}": "color-mix(in oklch, black 30%, blue)"}}
{"type": "palette", "name": "highlights_srgb", "colors": {"{_}": "#00000000", "{h}": "color-mix(in srgb, white 30%, red)"}}
{"type": "palette", "name": "skin_tones", "colors": {"{_}": "#00000000", "{b}": "color-mix(in srgb, #f5d0c5 80%, #8b4513)"}}
{"type": "sprite", "name": "shaded_square", "palette": "color_mix_basic", "size": [2, 2], "regions": {"m": {"rect": [0, 0, 2, 2]}}}
```

</div>

<div class="demo-container" data-demo="color_mix">
</div>
<!-- /DEMOS -->

Blend two colors together:

```json
{
  "colors": {
    "{blend}": "color-mix(in srgb, red 50%, blue)"
  }
}
```

| Syntax | Description |
|--------|-------------|
| `color-mix(in srgb, color1, color2)` | 50/50 blend |
| `color-mix(in srgb, color1 70%, color2)` | 70% color1, 30% color2 |
| `color-mix(in oklch, color1, color2)` | Blend in OKLCH space |

Blending in OKLCH often produces more visually pleasing results for gradients and transitions.

## Best Practices

1. **Use hex for exact colors**: When you know the exact RGB values
2. **Use HSL for variations**: Easy to create lighter/darker/muted versions
3. **Use OKLCH for palettes**: Consistent perceived brightness across colors
4. **Use color-mix for blends**: Create intermediate shades programmatically
5. **Use named colors for readability**: When the name matches your intent
