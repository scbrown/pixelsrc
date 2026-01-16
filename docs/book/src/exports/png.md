# PNG Export

PNG is the default output format for Pixelsrc. Each sprite renders to a transparent PNG file with optional scaling.

## Basic Usage

Render a single sprite to PNG:

```bash
pxl render sprite.pxl
```

This creates `sprite_<name>.png` for each sprite in the file.

## Output Options

### Specifying Output Path

```bash
# Single file output
pxl render sprite.pxl -o hero.png

# Output to directory (must end with /)
pxl render sprite.pxl -o output/

# Specific sprite only
pxl render sprite.pxl --sprite hero -o hero.png
```

### Scaling

Scale output by integer factors (1-16):

```bash
# 2x scale (each pixel becomes 2x2)
pxl render sprite.pxl --scale 2

# 4x scale for high-res preview
pxl render sprite.pxl --scale 4 -o preview.png
```

Scaling uses nearest-neighbor interpolation to preserve pixel art crispness.

## Output Naming

Without `-o`, output follows this pattern:

| Input | Sprites | Output |
|-------|---------|--------|
| `hero.pxl` | 1 sprite | `hero_<sprite_name>.png` |
| `hero.pxl` | Multiple | `hero_<sprite_name>.png` per sprite |
| `hero.jsonl` | 1 sprite | `hero_<sprite_name>.png` |

## Composition Rendering

Render compositions (layered sprites):

```bash
# Render all compositions
pxl render scene.pxl

# Render specific composition
pxl render scene.pxl --composition battle_scene
```

Compositions flatten all layers into a single PNG, respecting:
- Layer order (first layer = background)
- Layer positions (x, y offsets)
- Transparency blending

## Error Handling

### Lenient Mode (Default)

By default, Pixelsrc is lenient:
- Unknown tokens render as magenta (`#FF00FF`)
- Row length mismatches are padded/truncated
- Warnings are printed but rendering continues

```bash
# Normal render (lenient)
pxl render sprite.pxl
# Warning: Unknown token {foo} in sprite "hero"
# Rendered hero.png (with magenta pixels for unknown tokens)
```

### Strict Mode

For CI/CD validation, use strict mode:

```bash
pxl render sprite.pxl --strict
# Error: Unknown token {foo} in sprite "hero"
# Exit code: 1
```

## File Format Details

PNG files are saved with:
- RGBA color (32-bit with alpha channel)
- True transparency (alpha = 0 for `{_}` tokens)
- No compression artifacts (lossless)
- No embedded metadata

## Examples

### Basic Sprite Export

```bash
# Input: characters.pxl with sprites "hero", "villain"
pxl render characters.pxl -o sprites/

# Output:
# sprites/hero.png
# sprites/villain.png
```

### High-Resolution Preview

```bash
# Generate 8x scaled preview for documentation
pxl render icon.pxl --scale 8 -o docs/icon_preview.png
```

### Batch Processing

```bash
# Render all .pxl files in a directory
for f in assets/*.pxl; do
  pxl render "$f" -o output/
done
```

## Related

- [GIF Animation](gif.md) - Export animated sprites
- [Spritesheet](spritesheet.md) - Combine frames into a single image
- [Atlas Formats](atlas.md) - Game engine integration
