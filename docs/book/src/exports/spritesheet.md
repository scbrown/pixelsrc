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

<!-- DEMOS exports/spritesheet#basic -->
**Horizontal Spritesheet**

Export animation frames as a horizontal strip.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "flag", "colors": {"{_}": "#00000000", "{r}": "#ff0000", "{w}": "#ffffff", "{p}": "#8b4513"}}
{"type": "sprite", "name": "flag_1", "palette": "flag", "grid": ["{r}{r}{_}", "{w}{w}{_}", "{p}{_}{_}"]}
{"type": "sprite", "name": "flag_2", "palette": "flag", "grid": ["{_}{r}{r}", "{_}{w}{w}", "{p}{_}{_}"]}
{"type": "sprite", "name": "flag_3", "palette": "flag", "grid": ["{r}{r}{_}", "{w}{w}{_}", "{p}{_}{_}"]}
{"type": "animation", "name": "wave", "frames": ["flag_1", "flag_2", "flag_3"], "duration": 150}
```

</div>

<div class="demo-container" data-demo="basic">
</div>

```bash
pxl render flag.pxl --spritesheet -o flag_sheet.png
```
<!-- /DEMOS -->

### Scaled for Preview

<!-- DEMOS exports/spritesheet#scaled -->
**Scaled Spritesheet**

Scale up spritesheets for better visibility.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "bounce", "colors": {"{_}": "#00000000", "{b}": "#3498db", "{s}": "#2980b9"}}
{"type": "sprite", "name": "ball_1", "palette": "bounce", "grid": ["{_}{b}{_}", "{b}{s}{b}", "{_}{b}{_}"]}
{"type": "sprite", "name": "ball_2", "palette": "bounce", "grid": ["{b}{b}{b}", "{b}{s}{b}", "{b}{b}{b}"]}
{"type": "sprite", "name": "ball_3", "palette": "bounce", "grid": ["{_}{b}{_}", "{b}{s}{b}", "{_}{b}{_}"]}
{"type": "animation", "name": "bounce", "frames": ["ball_1", "ball_2", "ball_3", "ball_2"], "duration": 100}
```

</div>

<div class="demo-container" data-demo="scaled">
</div>

```bash
pxl render bounce.pxl --spritesheet --scale 4 -o bounce_4x.png
```
<!-- /DEMOS -->

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
