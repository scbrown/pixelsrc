# Terminal Output

Display sprites directly in the terminal using ANSI true-color escape sequences. Ideal for quick previews without generating image files.

## Basic Usage

Display a sprite in the terminal:

```bash
pxl show sprite.pxl
```

This renders the sprite using colored backgrounds, with each pixel shown as a 3-character cell:

```
 _  _  _  _
 _  r  r  _
 r  r  r  r
 _  r  r  _

Legend:
  _ = transparent       (#00000000)
  r = red               (#FF0000)
```

## Requirements

Terminal output requires:
- **True-color support**: 24-bit color (most modern terminals)
- **Unicode support**: For box-drawing characters in grid mode

Supported terminals: iTerm2, Alacritty, kitty, Windows Terminal, VS Code terminal, and most modern terminal emulators.

## Commands

### pxl show

Display sprite with colored backgrounds:

```bash
# Show first sprite
pxl show sprite.pxl

# Show specific sprite
pxl show sprite.pxl --sprite hero

# Show animation frame
pxl show sprite.pxl --animation walk --frame 2
```

### pxl render --emoji

Quick ASCII art preview:

```bash
pxl render sprite.pxl --emoji
```

## Sprite Selection

When a file contains multiple sprites:

```bash
# Show specific sprite by name
pxl show characters.pxl --sprite hero

# Without --sprite, shows first sprite found
pxl show characters.pxl
```

## Animation Preview

View individual animation frames:

```bash
# Show frame 0 (default)
pxl show sprite.pxl --animation walk

# Show specific frame
pxl show sprite.pxl --animation walk --frame 3
```

## Onion Skinning

Overlay previous/next frames for animation review:

```bash
# Show 2 frames before and after current
pxl show sprite.pxl --animation walk --frame 3 --onion 2
```

### Onion Skin Options

| Option | Default | Description |
|--------|---------|-------------|
| `--onion N` | - | Show N frames before/after |
| `--onion-opacity` | 0.3 | Ghost frame transparency (0.0-1.0) |
| `--onion-prev-color` | `#0000FF` | Tint for previous frames (blue) |
| `--onion-next-color` | `#00FF00` | Tint for next frames (green) |
| `--onion-fade` | false | Fade opacity for distant frames |

### Onion Skin to PNG

Save onion skin preview to file:

```bash
pxl show sprite.pxl --animation walk --frame 3 --onion 2 -o preview.png
```

## Coordinate Grid

Use `pxl grid` to display with row/column coordinates:

```bash
pxl grid sprite.pxl --sprite hero
```

Output:

```
     0  1  2  3
   ┌────────────
 0 │ _  _  _  _
 1 │ _  r  r  _
 2 │ r  r  r  r
 3 │ _  r  r  _
```

Useful for:
- Identifying pixel positions
- Debugging sprite alignment
- Documenting coordinates for code

## Color Display

### Transparent Pixels

Transparent pixels (`{_}` or alpha=0) display with a dark gray background to distinguish them from black:

```
┌──────────┬─────────────────────┐
│ Actual   │ Terminal Display    │
├──────────┼─────────────────────┤
│ #00000000│ Dark gray (visible) │
│ #000000FF│ Black               │
└──────────┴─────────────────────┘
```

### Color Legend

A legend is displayed after the sprite showing:
- Token alias (single character)
- Semantic name
- Hex color value

```
Legend:
  _ = transparent       (#00000000)
  s = skin              (#FFCC99)
  h = hair              (#8B4513)
  o = outline           (#000000)
```

## Inline View

Expand tokens with aligned spacing:

```bash
pxl inline sprite.pxl --sprite hero
```

Output:

```
{_}    {_}    {skin} {skin} {_}    {_}
{_}    {skin} {skin} {skin} {skin} {_}
{skin} {skin} {hair} {hair} {skin} {skin}
{_}    {skin} {skin} {skin} {skin} {_}
```

Useful for copy-pasting and editing grid rows.

## Examples

### Quick Preview During Development

```bash
# Fast visual check without creating files
pxl show hero.pxl
```

### Animation Review

```bash
# Review walk cycle frame by frame
for i in {0..3}; do
  echo "Frame $i:"
  pxl show character.pxl --animation walk --frame $i
  echo
done
```

### Onion Skin for Animation Polish

```bash
# See previous/next frames overlaid for smooth animation
pxl show character.pxl \
  --animation walk \
  --frame 3 \
  --onion 2 \
  --onion-fade \
  -o onion_preview.png
```

### Debug Pixel Positions

```bash
# Get coordinates for a specific pixel
pxl grid sprite.pxl --sprite hero
# Then reference in code: pixel at row 2, column 3
```

## Limitations

- **Cell size**: Each pixel is 3 characters wide, limiting resolution
- **Color accuracy**: Terminal color rendering varies by emulator
- **No animation playback**: Shows static frames only (use GIF for animation)

For accurate color review, export to [PNG](png.md) and view in an image editor.

## Related

- [PNG Export](png.md) - Export to image files
- [GIF Animation](gif.md) - Animated export
