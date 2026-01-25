# Spritesheet

Export animation frames as a single spritesheet image. Spritesheets are ideal for game engines that expect all frames in one file.

## Basic Usage

Render an animation as a spritesheet:

```bash
pxl render sprites.pxl --spritesheet
```

This creates a horizontal strip with all frames side by side.

## Layout

By default, spritesheets use a horizontal layout (single row):

```
┌────┬────┬────┬────┐
│ F1 │ F2 │ F3 │ F4 │
└────┴────┴────┴────┘
```

For a 16x16 sprite with 4 frames, the output is 64x16 pixels.

## Frame Dimensions

All frames are padded to match the largest frame:

```
┌────────┬────────┬────────┐
│        │  ████  │        │
│  ██    │ ██████ │    ██  │
│        │  ████  │        │
└────────┴────────┴────────┘
  8x12      12x12     8x12

→ Output: 36x12 (each cell 12x12)
```

This ensures consistent frame spacing for game engine playback.

## Command Options

### Select Animation

```bash
# Specific animation
pxl render sprites.pxl --spritesheet --animation walk

# First animation found (default)
pxl render sprites.pxl --spritesheet
```

### Scaling

```bash
# 4x scale for high-res assets
pxl render sprites.pxl --spritesheet --scale 4 -o walk_4x.png
```

### Output Path

```bash
pxl render sprites.pxl --spritesheet -o assets/walk_sheet.png
```

## Animation Definition

The spritesheet is generated from an animation object:

```json5
{
  type: "sprite",
  name: "walk_1",
  size: [16, 16],
  palette: "hero",
  regions: {
    body: { rect: [4, 0, 8, 12], z: 0 },
    legs: { rect: [4, 12, 8, 4], z: 0 },
  },
}

{
  type: "sprite",
  name: "walk_2",
  size: [16, 16],
  palette: "hero",
  regions: {
    body: { rect: [4, 0, 8, 12], z: 0 },
    legs: { rect: [2, 12, 12, 4], z: 0 },
  },
}

{
  type: "animation",
  name: "walk",
  frames: ["walk_1", "walk_2"],
}
```

## Game Engine Integration

### Frame Coordinates

For a 16x16 sprite with 4 frames in a horizontal strip:

| Frame | X | Y | Width | Height |
|-------|---|---|-------|--------|
| 0 | 0 | 0 | 16 | 16 |
| 1 | 16 | 0 | 16 | 16 |
| 2 | 32 | 0 | 16 | 16 |
| 3 | 48 | 0 | 16 | 16 |

### Example: Godot AnimatedSprite2D

```gdscript
var texture = load("res://sprites/walk.png")
var frames = SpriteFrames.new()
var frame_width = 16
var num_frames = 4

for i in range(num_frames):
    var region = Rect2(i * frame_width, 0, frame_width, texture.get_height())
    var atlas_tex = AtlasTexture.new()
    atlas_tex.atlas = texture
    atlas_tex.region = region
    frames.add_frame("walk", atlas_tex)
```

### Example: Unity

```csharp
Texture2D sheet = Resources.Load<Texture2D>("walk");
int frameWidth = 16;
int frameCount = 4;

Sprite[] frames = new Sprite[frameCount];
for (int i = 0; i < frameCount; i++) {
    Rect rect = new Rect(i * frameWidth, 0, frameWidth, sheet.height);
    frames[i] = Sprite.Create(sheet, rect, new Vector2(0.5f, 0.5f), 16);
}
```

## Spritesheet vs Atlas

| Feature | Spritesheet | [Atlas](atlas.md) |
|---------|------------|--------|
| Layout | Horizontal strip | Bin-packed |
| Metadata | None (manual calc) | JSON with coordinates |
| Space efficiency | Lower (fixed grid) | Higher (tight packing) |
| Simplicity | Higher | Lower |
| Multiple sprites | No | Yes |

Use **spritesheet** for quick exports and simple animation playback.
Use **atlas** for production game assets with multiple sprites and animations.

## Examples

### Basic Export

**Horizontal Spritesheet**

Export animation frames as a horizontal strip.

```json5
{
  type: "palette",
  name: "flag",
  colors: {
    _: "transparent",
    r: "#FF0000",
    w: "#FFFFFF",
    p: "#8B4513",
  },
}

{
  type: "sprite",
  name: "flag_1",
  size: [3, 3],
  palette: "flag",
  regions: {
    r: { rect: [0, 0, 2, 1], z: 1 },
    w: { rect: [0, 1, 2, 1], z: 1 },
    p: { points: [[0, 2]], z: 0 },
  },
}

{
  type: "sprite",
  name: "flag_2",
  size: [3, 3],
  palette: "flag",
  regions: {
    r: { rect: [1, 0, 2, 1], z: 1 },
    w: { rect: [1, 1, 2, 1], z: 1 },
    p: { points: [[0, 2]], z: 0 },
  },
}

{
  type: "sprite",
  name: "flag_3",
  size: [3, 3],
  palette: "flag",
  regions: {
    r: { rect: [0, 0, 2, 1], z: 1 },
    w: { rect: [0, 1, 2, 1], z: 1 },
    p: { points: [[0, 2]], z: 0 },
  },
}

{
  type: "animation",
  name: "wave",
  frames: ["flag_1", "flag_2", "flag_3"],
  duration: 150,
}
```

```bash
pxl render flag.pxl --spritesheet -o flag_sheet.png
```

### Scaled for Preview

**Scaled Spritesheet**

Scale up spritesheets for better visibility.

```json5
{
  type: "palette",
  name: "bounce",
  colors: {
    _: "transparent",
    b: "#3498DB",
    s: "#2980B9",
  },
}

{
  type: "sprite",
  name: "ball_1",
  size: [3, 3],
  palette: "bounce",
  regions: {
    b: {
      union: [
        { points: [[1, 0], [0, 1], [2, 1], [1, 2]] },
      ],
      z: 0,
    },
    s: { points: [[1, 1]], z: 1 },
  },
}

{
  type: "sprite",
  name: "ball_2",
  size: [3, 3],
  palette: "bounce",
  regions: {
    b: { rect: [0, 0, 3, 3], z: 0 },
    s: { points: [[1, 1]], z: 1 },
  },
}

{
  type: "sprite",
  name: "ball_3",
  size: [3, 3],
  palette: "bounce",
  regions: {
    b: {
      union: [
        { points: [[1, 0], [0, 1], [2, 1], [1, 2]] },
      ],
      z: 0,
    },
    s: { points: [[1, 1]], z: 1 },
  },
}

{
  type: "animation",
  name: "bounce",
  frames: ["ball_1", "ball_2", "ball_3", "ball_2"],
  duration: 100,
}
```

```bash
pxl render bounce.pxl --spritesheet --scale 4 -o bounce_4x.png
```

### Multiple Animations

```bash
# Export each animation as separate spritesheet
for anim in idle walk run attack; do
  pxl render character.pxl --spritesheet --animation "$anim" -o "sheets/${anim}.png"
done
```

## Related

- [PNG Export](png.md) - Single sprite export
- [GIF Animation](gif.md) - Animated preview
- [Atlas Formats](atlas.md) - Full game asset pipeline with metadata
