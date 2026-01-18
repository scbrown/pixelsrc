# Color Formats

Pixelsrc uses hex color notation for defining colors in palettes. All color values must start with `#` and use hexadecimal digits (0-9, A-F, case-insensitive).

## Supported Formats

| Format | Length | Example | Description |
|--------|--------|---------|-------------|
| `#RGB` | 3 digits | `#F00` | Short form, each digit doubled |
| `#RGBA` | 4 digits | `#F00F` | Short form with alpha |
| `#RRGGBB` | 6 digits | `#FF0000` | Full form, opaque |
| `#RRGGBBAA` | 8 digits | `#FF0000FF` | Full form with alpha |

## Short Form (#RGB, #RGBA)

In short form, each hex digit is doubled to produce the full value:

| Short | Expands To | Result |
|-------|------------|--------|
| `#F00` | `#FF0000` | Red (255, 0, 0) |
| `#0F0` | `#00FF00` | Green (0, 255, 0) |
| `#00F` | `#0000FF` | Blue (0, 0, 255) |
| `#ABC` | `#AABBCC` | (170, 187, 204) |
| `#F008` | `#FF000088` | Red with ~53% alpha |

The digit doubling rule: `#RGB` → `#RRGGBB` where each letter represents the same digit repeated.

## Full Form (#RRGGBB, #RRGGBBAA)

Full form specifies each color channel with two hex digits (00-FF):

```
#RRGGBB
 │ │ └── Blue (00-FF)
 │ └──── Green (00-FF)
 └────── Red (00-FF)
```

Without an alpha channel, colors default to fully opaque (alpha = 255).

## Alpha Channel

The alpha channel controls transparency:

| Alpha | Hex | Meaning |
|-------|-----|---------|
| 0 | `00` | Fully transparent |
| 128 | `80` | 50% transparent |
| 255 | `FF` | Fully opaque |

```json
{"type": "palette", "name": "alpha_demo", "colors": {
  "{solid}": "#FF0000",
  "{half}": "#FF000080",
  "{clear}": "#FF000000"
}}
```

## Transparency Convention

The token `{_}` is conventionally used for transparent pixels:

```json
{"type": "palette", "name": "example", "colors": {
  "{_}": "#00000000",
  "{fg}": "#FFFFFF"
}}
```

Using `#0000` (short form) or `#00000000` (full form) creates fully transparent pixels that will not be rendered.

## Case Insensitivity

Color values are case-insensitive:

```json
{"type": "palette", "name": "case_demo", "colors": {
  "{a}": "#ff0000",
  "{b}": "#FF0000",
  "{c}": "#Ff0000"
}}
```

All three colors above are identical (red).

## Examples

### Basic Colors

| Color | Hex | Short |
|-------|-----|-------|
| Black | `#000000` | `#000` |
| White | `#FFFFFF` | `#FFF` |
| Red | `#FF0000` | `#F00` |
| Green | `#00FF00` | `#0F0` |
| Blue | `#0000FF` | `#00F` |
| Yellow | `#FFFF00` | `#FF0` |
| Cyan | `#00FFFF` | `#0FF` |
| Magenta | `#FF00FF` | `#F0F` |

### Common Pixel Art Colors

```json
{"type": "palette", "name": "pixel_basics", "colors": {
  "{_}": "#0000",
  "{outline}": "#222034",
  "{skin}": "#EABF9C",
  "{skin_shade}": "#C38B6E",
  "{hair}": "#5D3A29",
  "{highlight}": "#FFFBF0"
}}
```

## Error Messages

Invalid color formats produce these errors:

| Error | Cause | Example |
|-------|-------|---------|
| `empty color string` | Empty value | `""` |
| `color must start with '#'` | Missing hash | `"FF0000"` |
| `invalid color length N, expected 3, 4, 6, or 8` | Wrong digit count | `"#FF00"` |
| `invalid hex character 'X'` | Non-hex character | `"#GG0000"` |

## Related

- [Built-in Palettes](palettes.md) - Pre-defined color sets
- [Palette Format](../format/palette.md) - Defining custom palettes
- [validate command](../cli/validate.md) - Check color validity
