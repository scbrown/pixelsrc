# show

Display sprites with colored terminal output using ANSI true-color.

## Usage

```
pxl show [OPTIONS] <FILE>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Input file containing sprite definitions |

## Options

| Option | Description |
|--------|-------------|
| `--sprite <SPRITE>` | Sprite name (if file contains multiple sprites) |
| `--animation <ANIMATION>` | Animation name to show with onion skinning |
| `--frame <FRAME>` | Frame index to display (for animations, default: 0) |
| `--onion <ONION>` | Number of frames before/after to show as onion skin |
| `--onion-opacity <OPACITY>` | Ghost frame opacity (0.0-1.0, default: 0.3) |
| `--onion-prev-color <COLOR>` | Tint color for previous frames (default: #0000FF blue) |
| `--onion-next-color <COLOR>` | Tint color for next frames (default: #00FF00 green) |
| `--onion-fade` | Decrease opacity for frames farther from current |
| `-o, --output <OUTPUT>` | Output file (PNG) for onion skin preview |

## Description

The `show` command renders sprites directly in the terminal using ANSI true-color escape codes. This provides instant visual feedback without generating files.

For animations, onion skinning displays ghost frames before and after the current frame, helping visualize motion.

## Examples

### Basic preview

```bash
# Show a sprite in terminal
pxl show sprite.pxl

# Show a specific sprite from multi-sprite file
pxl show sprites.pxl --sprite hero
```

### Animation preview

```bash
# Show first frame of animation
pxl show animation.pxl --animation walk

# Show specific frame
pxl show animation.pxl --animation walk --frame 2
```

### Onion skinning

```bash
# Show with 2 frames of onion skin on each side
pxl show animation.pxl --animation walk --onion 2

# Custom onion skin colors
pxl show animation.pxl --animation walk --onion 2 \
    --onion-prev-color "#FF0000" \
    --onion-next-color "#00FF00"

# Fading onion skin (farther frames more transparent)
pxl show animation.pxl --animation walk --onion 3 --onion-fade

# Higher opacity ghost frames
pxl show animation.pxl --animation walk --onion 2 --onion-opacity 0.5
```

### Export onion skin view

```bash
# Save onion skin preview as PNG
pxl show animation.pxl --animation walk --onion 2 -o preview.png
```

## Terminal Requirements

This command requires a terminal that supports:
- ANSI true-color (24-bit color)
- Unicode block characters (▀, ▄)

Most modern terminals support this, including:
- iTerm2, Terminal.app (macOS)
- Windows Terminal
- GNOME Terminal, Konsole (Linux)
- VS Code integrated terminal

## See Also

- [grid](grid.md) - Show grid with coordinates
- [render](render.md) - Render to image files
- [inline](inline.md) - Expand grid spacing for readability
