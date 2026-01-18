# Built-in Palettes

Pixelsrc includes several built-in palettes that can be referenced by name using the `@name` syntax. These palettes provide curated color sets for common pixel art styles.

## Using Built-in Palettes

Reference a built-in palette with the `@` prefix:

```json
{"type": "sprite", "name": "retro_heart", "palette": "@gameboy", "grid": ["{_}{dark}{_}", "{dark}{light}{dark}", "{_}{dark}{_}"]}
```

List available palettes:

```bash
pxl palettes
```

Show palette colors:

```bash
pxl palettes --show gameboy
```

## Available Palettes

### gameboy

Classic Game Boy 4-color green palette.

| Token | Color | Hex |
|-------|-------|-----|
| `{_}` | Transparent | `#00000000` |
| `{lightest}` | ![#9BBC0F](https://via.placeholder.com/16/9BBC0F/9BBC0F.png) | `#9BBC0F` |
| `{light}` | ![#8BAC0F](https://via.placeholder.com/16/8BAC0F/8BAC0F.png) | `#8BAC0F` |
| `{dark}` | ![#306230](https://via.placeholder.com/16/306230/306230.png) | `#306230` |
| `{darkest}` | ![#0F380F](https://via.placeholder.com/16/0F380F/0F380F.png) | `#0F380F` |

**Reference:** [Nintendo Game Boy (BGB) on Lospec](https://lospec.com/palette-list/nintendo-gameboy-bgb)

### nes

NES-inspired palette with key representative colors.

| Token | Color | Hex |
|-------|-------|-----|
| `{_}` | Transparent | `#00000000` |
| `{black}` | ![#000000](https://via.placeholder.com/16/000000/000000.png) | `#000000` |
| `{white}` | ![#FCFCFC](https://via.placeholder.com/16/FCFCFC/FCFCFC.png) | `#FCFCFC` |
| `{red}` | ![#A80020](https://via.placeholder.com/16/A80020/A80020.png) | `#A80020` |
| `{green}` | ![#00A800](https://via.placeholder.com/16/00A800/00A800.png) | `#00A800` |
| `{blue}` | ![#0058F8](https://via.placeholder.com/16/0058F8/0058F8.png) | `#0058F8` |
| `{cyan}` | ![#00B8D8](https://via.placeholder.com/16/00B8D8/00B8D8.png) | `#00B8D8` |
| `{yellow}` | ![#F8D800](https://via.placeholder.com/16/F8D800/F8D800.png) | `#F8D800` |
| `{orange}` | ![#F83800](https://via.placeholder.com/16/F83800/F83800.png) | `#F83800` |
| `{pink}` | ![#F878F8](https://via.placeholder.com/16/F878F8/F878F8.png) | `#F878F8` |
| `{brown}` | ![#503000](https://via.placeholder.com/16/503000/503000.png) | `#503000` |
| `{gray}` | ![#7C7C7C](https://via.placeholder.com/16/7C7C7C/7C7C7C.png) | `#7C7C7C` |
| `{skin}` | ![#FCB8B8](https://via.placeholder.com/16/FCB8B8/FCB8B8.png) | `#FCB8B8` |

**Reference:** [Nintendo Entertainment System on Lospec](https://lospec.com/palette-list/nintendo-entertainment-system)

### pico8

PICO-8 fantasy console 16-color palette.

| Token | Color | Hex |
|-------|-------|-----|
| `{_}` | Transparent | `#00000000` |
| `{black}` | ![#000000](https://via.placeholder.com/16/000000/000000.png) | `#000000` |
| `{dark_blue}` | ![#1D2B53](https://via.placeholder.com/16/1D2B53/1D2B53.png) | `#1D2B53` |
| `{dark_purple}` | ![#7E2553](https://via.placeholder.com/16/7E2553/7E2553.png) | `#7E2553` |
| `{dark_green}` | ![#008751](https://via.placeholder.com/16/008751/008751.png) | `#008751` |
| `{brown}` | ![#AB5236](https://via.placeholder.com/16/AB5236/AB5236.png) | `#AB5236` |
| `{dark_gray}` | ![#5F574F](https://via.placeholder.com/16/5F574F/5F574F.png) | `#5F574F` |
| `{light_gray}` | ![#C2C3C7](https://via.placeholder.com/16/C2C3C7/C2C3C7.png) | `#C2C3C7` |
| `{white}` | ![#FFF1E8](https://via.placeholder.com/16/FFF1E8/FFF1E8.png) | `#FFF1E8` |
| `{red}` | ![#FF004D](https://via.placeholder.com/16/FF004D/FF004D.png) | `#FF004D` |
| `{orange}` | ![#FFA300](https://via.placeholder.com/16/FFA300/FFA300.png) | `#FFA300` |
| `{yellow}` | ![#FFEC27](https://via.placeholder.com/16/FFEC27/FFEC27.png) | `#FFEC27` |
| `{green}` | ![#00E436](https://via.placeholder.com/16/00E436/00E436.png) | `#00E436` |
| `{blue}` | ![#29ADFF](https://via.placeholder.com/16/29ADFF/29ADFF.png) | `#29ADFF` |
| `{indigo}` | ![#83769C](https://via.placeholder.com/16/83769C/83769C.png) | `#83769C` |
| `{pink}` | ![#FF77A8](https://via.placeholder.com/16/FF77A8/FF77A8.png) | `#FF77A8` |
| `{peach}` | ![#FFCCAA](https://via.placeholder.com/16/FFCCAA/FFCCAA.png) | `#FFCCAA` |

**Reference:** [PICO-8 on Lospec](https://lospec.com/palette-list/pico-8)

### grayscale

8-shade grayscale palette from white to black.

| Token | Color | Hex |
|-------|-------|-----|
| `{_}` | Transparent | `#00000000` |
| `{white}` | ![#FFFFFF](https://via.placeholder.com/16/FFFFFF/FFFFFF.png) | `#FFFFFF` |
| `{gray1}` | ![#DFDFDF](https://via.placeholder.com/16/DFDFDF/DFDFDF.png) | `#DFDFDF` |
| `{gray2}` | ![#BFBFBF](https://via.placeholder.com/16/BFBFBF/BFBFBF.png) | `#BFBFBF` |
| `{gray3}` | ![#9F9F9F](https://via.placeholder.com/16/9F9F9F/9F9F9F.png) | `#9F9F9F` |
| `{gray4}` | ![#7F7F7F](https://via.placeholder.com/16/7F7F7F/7F7F7F.png) | `#7F7F7F` |
| `{gray5}` | ![#5F5F5F](https://via.placeholder.com/16/5F5F5F/5F5F5F.png) | `#5F5F5F` |
| `{gray6}` | ![#3F3F3F](https://via.placeholder.com/16/3F3F3F/3F3F3F.png) | `#3F3F3F` |
| `{black}` | ![#000000](https://via.placeholder.com/16/000000/000000.png) | `#000000` |

### 1bit

Minimal 1-bit black and white palette.

| Token | Color | Hex |
|-------|-------|-----|
| `{_}` | Transparent | `#00000000` |
| `{black}` | ![#000000](https://via.placeholder.com/16/000000/000000.png) | `#000000` |
| `{white}` | ![#FFFFFF](https://via.placeholder.com/16/FFFFFF/FFFFFF.png) | `#FFFFFF` |

### dracula

Dracula theme palette for code editor aesthetics.

| Token | Color | Hex |
|-------|-------|-----|
| `{_}` | Transparent | `#00000000` |
| `{background}` | ![#282A36](https://via.placeholder.com/16/282A36/282A36.png) | `#282A36` |
| `{current}` | ![#44475A](https://via.placeholder.com/16/44475A/44475A.png) | `#44475A` |
| `{foreground}` | ![#F8F8F2](https://via.placeholder.com/16/F8F8F2/F8F8F2.png) | `#F8F8F2` |
| `{comment}` | ![#6272A4](https://via.placeholder.com/16/6272A4/6272A4.png) | `#6272A4` |
| `{cyan}` | ![#8BE9FD](https://via.placeholder.com/16/8BE9FD/8BE9FD.png) | `#8BE9FD` |
| `{green}` | ![#50FA7B](https://via.placeholder.com/16/50FA7B/50FA7B.png) | `#50FA7B` |
| `{orange}` | ![#FFB86C](https://via.placeholder.com/16/FFB86C/FFB86C.png) | `#FFB86C` |
| `{pink}` | ![#FF79C6](https://via.placeholder.com/16/FF79C6/FF79C6.png) | `#FF79C6` |
| `{purple}` | ![#BD93F9](https://via.placeholder.com/16/BD93F9/BD93F9.png) | `#BD93F9` |
| `{red}` | ![#FF5555](https://via.placeholder.com/16/FF5555/FF5555.png) | `#FF5555` |
| `{yellow}` | ![#F1FA8C](https://via.placeholder.com/16/F1FA8C/F1FA8C.png) | `#F1FA8C` |

**Reference:** [Dracula Theme](https://draculatheme.com/contribute)

## Transparent Color

All built-in palettes include the special transparent token `{_}` mapped to `#00000000`. This is the conventional token for transparency in Pixelsrc sprites.

## Extending Built-in Palettes

You can use a built-in palette and add custom colors:

```json
{"type": "palette", "name": "custom_gb", "extends": "@gameboy", "colors": {"{highlight}": "#FFFF00"}}
```

Or override existing colors:

```json
{"type": "palette", "name": "warm_gb", "extends": "@gameboy", "colors": {"{lightest}": "#E8D8A0"}}
```

## Related

- [Palette Format](../format/palette.md) - Custom palette definition
- [palettes command](../cli/palettes.md) - Palette management CLI
- [Color Formats](colors.md) - Supported color notation
