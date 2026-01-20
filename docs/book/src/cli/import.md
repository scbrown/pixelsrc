# import

Import a PNG image and convert it to Pixelsrc format.

## Usage

```
pxl import [OPTIONS] <INPUT>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input PNG file to convert |

## Options

| Option | Description |
|--------|-------------|
| `-o, --output <OUTPUT>` | Output file (default: `{input}.jsonl`, use `.pxl` extension for new format) |
| `--max-colors <MAX_COLORS>` | Maximum number of colors in the palette (2-256, default: 16) |
| `-n, --name <NAME>` | Name for the generated sprite (default: derived from filename) |

## Description

The `import` command analyzes a PNG image and generates a Pixelsrc file containing:
- A palette with the detected colors
- A sprite with tokens referencing the palette

This is useful for converting existing pixel art into the Pixelsrc format for editing or animation.

## Examples

<!-- DEMOS cli/import#basic -->
**Import Workflow**

Convert a PNG image to Pixelsrc format. The import command detects colors and creates a palette and sprite definition.

<div class="demo-source">

```bash
# Input: hero.png (16x16 pixel art)
pxl import hero.png -o hero.pxl

# Output: hero.pxl contains:
# {"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", ...}}
# {"type": "sprite", "name": "hero", "palette": "hero", "grid": [...]}
```

</div>

<div class="demo-container" data-demo="basic">
</div>
<!-- /DEMOS -->

### Basic import

```bash
# Import with default settings
pxl import hero.png

# Creates hero.jsonl with detected colors and sprite data
```

### Custom output format

```bash
# Output as .pxl format (preferred)
pxl import hero.png -o hero.pxl

# Output to specific location
pxl import hero.png -o assets/sprites/hero.pxl
```

### Controlling colors

<!-- DEMOS cli/import#quantization -->
**Color Quantization**

Limit the palette size during import for retro-style constraints.

<div class="demo-source">

```bash
# Limit to 4 colors (Game Boy style)
pxl import hero.png --max-colors 4

# Result: palette has at most 4 colors
# Colors are quantized to fit the constraint
```

</div>

<div class="demo-container" data-demo="quantization">
</div>
<!-- /DEMOS -->

```bash
# Limit to 4 colors (Game Boy style)
pxl import hero.png --max-colors 4

# Allow more colors for detailed sprites
pxl import detailed.png --max-colors 64
```

### Custom sprite name

```bash
# Name the sprite instead of using filename
pxl import player-idle-frame1.png --name player_idle
```

## Color Quantization

When the source image has more colors than `--max-colors`, the importer will reduce the color count through quantization. This may result in slight color differences from the original.

For best results:
- Use source images that are already limited to your target palette
- Import at the original resolution (don't scale up before importing)
- Review the generated palette and adjust colors if needed

## Tips

- Import works best with clean pixel art (no anti-aliasing)
- Transparent pixels are preserved
- Very similar colors may be merged during import
- Use `pxl show` after import to preview the result

## See Also

- [render](render.md) - Render sprites back to PNG
- [palettes](palettes.md) - Use built-in palettes instead of importing colors
- [new](new.md) - Create new sprites from templates
