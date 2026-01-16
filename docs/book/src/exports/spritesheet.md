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

```json
{"type": "sprite", "name": "walk_1", "palette": "hero", "grid": [...]}
{"type": "sprite", "name": "walk_2", "palette": "hero", "grid": [...]}
{"type": "sprite", "name": "walk_3", "palette": "hero", "grid": [...]}
{"type": "sprite", "name": "walk_4", "palette": "hero", "grid": [...]}
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3", "walk_4"]}
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

```bash
pxl render character.pxl --spritesheet -o character_walk.png
```

### Multiple Animations

```bash
# Export each animation as separate spritesheet
for anim in idle walk run attack; do
  pxl render character.pxl --spritesheet --animation "$anim" -o "sheets/${anim}.png"
done
```

### Scaled for Preview

```bash
# 8x scale for documentation/preview
pxl render character.pxl --spritesheet --scale 8 -o docs/walk_preview.png
```

## Related

- [PNG Export](png.md) - Single sprite export
- [GIF Animation](gif.md) - Animated preview
- [Atlas Formats](atlas.md) - Full game asset pipeline with metadata
