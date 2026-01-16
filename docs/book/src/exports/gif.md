# GIF Animation

Export animated sprites as GIF files for previews, social media, or web use.

## Basic Usage

Render an animation to GIF:

```bash
pxl render sprite.pxl --gif
```

This requires an `animation` object in your input file:

```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

## Animation Definition

Animations reference sprite names in sequence:

```json
{"type": "sprite", "name": "walk_1", "palette": "hero", "grid": ["..."]}
{"type": "sprite", "name": "walk_2", "palette": "hero", "grid": ["..."]}
{"type": "sprite", "name": "walk_3", "palette": "hero", "grid": ["..."]}
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

### Animation Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `frames` | array | required | Sprite names in sequence |
| `duration` | number | 100 | Milliseconds per frame |
| `loop` | boolean | true | Whether animation loops |

## Command Options

### Select Animation

When a file contains multiple animations:

```bash
# Use specific animation
pxl render sprites.pxl --gif --animation walk

# Without --animation, uses first animation found
pxl render sprites.pxl --gif
```

### Scaling

```bash
# 4x scale for better visibility
pxl render sprites.pxl --gif --scale 4 -o preview.gif
```

### Output Path

```bash
pxl render sprites.pxl --gif -o hero_walk.gif
```

## Frame Timing

GIF uses centiseconds (1/100 second) for frame delays. Pixelsrc converts `duration` from milliseconds:

| duration (ms) | GIF delay | Effective FPS |
|--------------|-----------|---------------|
| 16 | 2cs | ~50 fps |
| 33 | 3cs | ~33 fps |
| 100 | 10cs | 10 fps |
| 200 | 20cs | 5 fps |

Minimum duration is 10ms (1 centisecond). Shorter values are clamped.

## Palette Cycling

Create animated effects without multiple frames using palette cycling:

```json
{"type": "sprite", "name": "water", "palette": "ocean", "grid": [
  "{w1}{w2}{w3}",
  "{w2}{w3}{w1}",
  "{w3}{w1}{w2}"
]}
{"type": "animation", "name": "shimmer", "frames": ["water"], "duration": 150,
 "palette_cycle": [{"tokens": ["{w1}", "{w2}", "{w3}"]}]}
```

Palette cycling rotates colors through the specified tokens, creating animated effects like:
- Shimmering water
- Flickering fire
- Pulsing energy

## Looping Behavior

Control whether animations loop:

```json
{"type": "animation", "name": "death", "frames": ["die_1", "die_2", "die_3"], "loop": false}
```

- `"loop": true` (default): Animation repeats infinitely
- `"loop": false`: Animation plays once and stops on final frame

## Examples

### Basic Walk Cycle

```bash
# Input file with 4-frame walk cycle
pxl render character.pxl --gif --animation walk -o walk.gif
```

### High-Res Social Media Preview

```bash
# 8x scale with 200ms frames for slow-motion
pxl render character.pxl --gif --scale 8 -o preview.gif
```

### Batch Animation Export

```bash
# Export all animations from a file
for anim in idle walk run attack; do
  pxl render character.pxl --gif --animation "$anim" -o "output/${anim}.gif"
done
```

## Limitations

GIF format has inherent limitations:
- **256 color limit**: Complex sprites may have color banding
- **Binary transparency**: Pixels are fully transparent or opaque (no semi-transparency)
- **Large file size**: Uncompressed frame data can be large

For better quality and smaller files, consider [Spritesheet](spritesheet.md) export for game use.

## Related

- [PNG Export](png.md) - Static sprite export
- [Spritesheet](spritesheet.md) - Combine frames into a single image
- [Atlas Formats](atlas.md) - Game engine integration with metadata
