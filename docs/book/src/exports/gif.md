# GIF Animation

Export animated sprites as GIF files for previews, social media, or web use.

## Basic Usage

Render an animation to GIF:

```bash
pxl render sprite.pxl --gif
```

This requires an `animation` object in your input file:

```json5
{
  type: "animation",
  name: "walk",
  frames: ["walk_1", "walk_2", "walk_3"],
  duration: 100,
}
```

## Animation Definition

Animations reference sprite names in sequence:

```json5
{
  type: "sprite",
  name: "walk_1",
  size: [8, 12],
  palette: "hero",
  regions: {
    body: { rect: [2, 4, 4, 8], z: 0 },
    leg_left: { rect: [2, 10, 2, 2], z: 1 },
    leg_right: { rect: [4, 10, 2, 2], z: 1 },
  },
}

{
  type: "sprite",
  name: "walk_2",
  size: [8, 12],
  palette: "hero",
  regions: {
    body: { rect: [2, 4, 4, 8], z: 0 },
    leg_left: { rect: [1, 10, 2, 2], z: 1 },
    leg_right: { rect: [5, 10, 2, 2], z: 1 },
  },
}

{
  type: "animation",
  name: "walk",
  frames: ["walk_1", "walk_2"],
  duration: 100,
}
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

```json5
{
  type: "sprite",
  name: "water",
  size: [6, 3],
  palette: "ocean",
  regions: {
    w1: { points: [[0, 0], [3, 1], [0, 2]], z: 0 },
    w2: { points: [[1, 0], [4, 1], [1, 2]], z: 0 },
    w3: { points: [[2, 0], [5, 1], [2, 2]], z: 0 },
  },
}

{
  type: "animation",
  name: "shimmer",
  frames: ["water"],
  duration: 150,
  palette_cycle: [{ tokens: ["w1", "w2", "w3"] }],
}
```

Palette cycling rotates colors through the specified tokens, creating animated effects like:
- Shimmering water
- Flickering fire
- Pulsing energy

## Looping Behavior

Control whether animations loop:

```json5
{
  type: "animation",
  name: "death",
  frames: ["die_1", "die_2", "die_3"],
  loop: false,
}
```

- `"loop": true` (default): Animation repeats infinitely
- `"loop": false`: Animation plays once and stops on final frame

## Examples

### Basic Walk Cycle

**Animated GIF Export**

Export a simple animation as an animated GIF.

```json5
{
  type: "palette",
  name: "blink",
  colors: {
    _: "transparent",
    w: "#FFFFFF",
    b: "#000000",
  },
}

{
  type: "sprite",
  name: "eye_open",
  size: [4, 3],
  palette: "blink",
  regions: {
    w: {
      union: [
        { rect: [1, 0, 2, 1] },
        { rect: [0, 1, 4, 1] },
        { rect: [1, 2, 2, 1] },
      ],
      z: 0,
    },
    b: { rect: [1, 1, 2, 1], z: 1 },
  },
}

{
  type: "sprite",
  name: "eye_closed",
  size: [4, 3],
  palette: "blink",
  regions: {
    w: { rect: [0, 1, 4, 1], z: 0 },
  },
}

{
  type: "animation",
  name: "blink",
  frames: ["eye_open", "eye_open", "eye_open", "eye_closed"],
  duration: 200,
}
```

```bash
pxl render blink.pxl --gif -o blink.gif
```

### High-Res Social Media Preview

**Scaled GIF Export**

Scale up animations for better visibility in previews.

```json5
{
  type: "palette",
  name: "spinner",
  colors: {
    _: "transparent",
    a: "#FF6B6B",
    b: "#4ECDC4",
  },
}

{
  type: "sprite",
  name: "spin_1",
  size: [2, 2],
  palette: "spinner",
  regions: {
    a: { points: [[0, 0]], z: 0 },
    b: { points: [[1, 1]], z: 0 },
  },
}

{
  type: "sprite",
  name: "spin_2",
  size: [2, 2],
  palette: "spinner",
  regions: {
    a: { points: [[1, 0]], z: 0 },
    b: { points: [[0, 1]], z: 0 },
  },
}

{
  type: "sprite",
  name: "spin_3",
  size: [2, 2],
  palette: "spinner",
  regions: {
    b: { points: [[0, 0]], z: 0 },
    a: { points: [[1, 1]], z: 0 },
  },
}

{
  type: "sprite",
  name: "spin_4",
  size: [2, 2],
  palette: "spinner",
  regions: {
    b: { points: [[1, 0]], z: 0 },
    a: { points: [[0, 1]], z: 0 },
  },
}

{
  type: "animation",
  name: "spin",
  frames: ["spin_1", "spin_2", "spin_3", "spin_4"],
  duration: 100,
}
```

```bash
pxl render spinner.pxl --gif --scale 4 -o spinner_4x.gif
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
