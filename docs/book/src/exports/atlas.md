# Atlas Formats

Texture atlases combine multiple sprites into a single image with metadata describing frame locations. Pixelsrc supports several atlas formats for different game engines.

## Basic Usage

Export an atlas with generic JSON metadata:

```bash
pxl render sprites.pxl --format atlas
```

This creates:
- `sprites.png` - The packed texture atlas image
- `sprites.json` - Metadata with frame coordinates

## Supported Formats

| Format | Flag | Output | Description |
|--------|------|--------|-------------|
| JSON | `--format atlas` | `.json` | Generic JSON, works with any engine |
| Godot | `--format atlas-godot` | `.tres` | Godot resource files |
| Unity | `--format atlas-unity` | `.json` | Unity sprite import format |
| libGDX | `--format atlas-libgdx` | `.atlas` | TextureAtlas format |

## Atlas Configuration

### Maximum Size

Limit atlas dimensions (useful for platform constraints):

```bash
pxl render sprites.pxl --format atlas --max-size 512x512
```

If sprites don't fit, multiple atlas pages are generated.

### Padding

Add space between sprites to prevent texture bleeding:

```bash
pxl render sprites.pxl --format atlas --padding 2
```

### Power of Two

Force dimensions to power-of-two (required by some platforms):

```bash
pxl render sprites.pxl --format atlas --power-of-two
```

## Generic JSON Format

The default JSON format includes all metadata for custom integration:

```json
{
  "image": "sprites.png",
  "size": [256, 256],
  "frames": {
    "hero_idle": {
      "x": 0,
      "y": 0,
      "w": 32,
      "h": 32,
      "origin": [16, 32]
    },
    "hero_walk_1": {
      "x": 32,
      "y": 0,
      "w": 32,
      "h": 32,
      "origin": [16, 32]
    }
  },
  "animations": {
    "walk": {
      "frames": ["hero_walk_1", "hero_walk_2", "hero_walk_3"],
      "fps": 10
    }
  }
}
```

### Frame Metadata

Sprites with `metadata` in the source file include:
- `origin` - Pivot point `[x, y]`
- `boxes` - Collision boxes (hit, hurt, collide, trigger)

```json5
{
  type: "sprite",
  name: "hero",
  size: [32, 32],
  palette: "colors",
  regions: {
    body: { rect: [8, 0, 16, 32], z: 0 },
  },
  metadata: {
    origin: [16, 32],
    boxes: {
      hit: { x: 4, y: 0, w: 24, h: 32 },
    },
  },
}
```

## Godot Format

```bash
pxl render sprites.pxl --format atlas-godot
```

Generates Godot `.tres` resource files:
- Individual `AtlasTexture` resources for each frame
- `SpriteFrames` resource for animations

Example AtlasTexture:

```text
[gd_resource type="AtlasTexture" load_steps=2 format=3]

[ext_resource type="Texture2D" path="res://assets/atlas.png" id="1"]

[resource]
atlas = ExtResource("1")
region = Rect2(0, 0, 32, 32)
```

## Unity Format

```bash
pxl render sprites.pxl --format atlas-unity
```

Generates Unity-compatible JSON:

```json
{
  "texture": "atlas.png",
  "textureSize": { "w": 256, "h": 256 },
  "pixelsPerUnit": 16,
  "filterMode": "Point",
  "sprites": [
    {
      "name": "hero_idle",
      "rect": { "x": 0, "y": 0, "w": 32, "h": 32 },
      "pivot": { "x": 0.5, "y": 0.0 },
      "border": { "x": 0, "y": 0, "z": 0, "w": 0 }
    }
  ],
  "animations": [
    {
      "name": "walk",
      "frameRate": 10,
      "sprites": ["walk_1", "walk_2", "walk_3"]
    }
  ]
}
```

Import using Unity's TextureImporter API or custom editor scripts.

## libGDX Format

```bash
pxl render sprites.pxl --format atlas-libgdx
```

Generates libGDX TextureAtlas format:

```text
atlas.png
size: 256, 256
format: RGBA8888
filter: Nearest, Nearest
repeat: none
hero_idle
  rotate: false
  xy: 0, 0
  size: 32, 32
  orig: 32, 32
  offset: 0, 0
  index: -1
hero_walk
  rotate: false
  xy: 32, 0
  size: 32, 32
  orig: 32, 32
  offset: 0, 0
  index: 0
```

Load directly with libGDX's `TextureAtlas` class:

```java
TextureAtlas atlas = new TextureAtlas(Gdx.files.internal("sprites.atlas"));
TextureRegion idle = atlas.findRegion("hero_idle");
Animation<TextureRegion> walk = new Animation<>(0.1f, atlas.findRegions("hero_walk"));
```

## Animation Tags

Define named sub-ranges within animations for game logic:

```json5
{
  type: "animation",
  name: "attack",
  frames: ["attack_1", "attack_2", "attack_3", "attack_4", "attack_5"],
  duration: 100,
  tags: {
    windup: { start: 0, end: 1 },
    active: { start: 2, end: 3, loop: false },
    recovery: { start: 4, end: 4 },
  },
}
```

Tags are included in atlas metadata for runtime use.

## Bin Packing

Pixelsrc uses shelf bin packing for efficient atlas layout:

```
┌─────────────────────────────┐
│ ████ │ ██████ │ ████████ │
├──────┴────────┴──────────┤
│ ██████████ │ ██████ │    │
├────────────┴────────┴────┤
│ ████████████████████████ │
└─────────────────────────────┘
```

Sprites are sorted by height and packed into horizontal shelves for efficient space usage.

## Examples

### Basic Atlas Export

**JSON Atlas Export**

Pack multiple sprites into a single atlas with JSON metadata.

```json5
{
  type: "palette",
  name: "items",
  colors: {
    _: "transparent",
    g: "#FFD700",
    s: "#C0C0C0",
    r: "#FF4444",
  },
}

{
  type: "sprite",
  name: "coin",
  size: [3, 3],
  palette: "items",
  regions: {
    g: {
      union: [
        { points: [[1, 0]] },
        { rect: [0, 1, 3, 1] },
        { points: [[1, 2]] },
      ],
      z: 0,
    },
  },
}

{
  type: "sprite",
  name: "key",
  size: [3, 3],
  palette: "items",
  regions: {
    s: {
      union: [
        { rect: [0, 0, 2, 1] },
        { rect: [1, 1, 1, 2] },
        { points: [[2, 2]] },
      ],
      z: 0,
    },
  },
}

{
  type: "sprite",
  name: "heart",
  size: [3, 3],
  palette: "items",
  regions: {
    r: {
      union: [
        { points: [[0, 0], [2, 0]] },
        { rect: [0, 1, 3, 1] },
        { points: [[1, 2]] },
      ],
      z: 0,
    },
  },
}
```

```bash
pxl render items.pxl --format atlas -o items
```

### Godot Export

**Godot Atlas Export**

Generate Godot-compatible atlas resources.

```json5
{
  type: "palette",
  name: "ui",
  colors: {
    _: "transparent",
    w: "#FFFFFF",
    g: "#888888",
    b: "#333333",
  },
}

{
  type: "sprite",
  name: "btn_normal",
  size: [4, 3],
  palette: "ui",
  regions: {
    b: { points: [[0, 0], [3, 0], [0, 2], [3, 2]], z: 0 },
    g: {
      union: [
        { rect: [1, 0, 2, 1] },
        { points: [[0, 1], [3, 1]] },
        { rect: [1, 2, 2, 1] },
      ],
      z: 0,
    },
    w: { rect: [1, 1, 2, 1], z: 1 },
  },
}

{
  type: "sprite",
  name: "btn_hover",
  size: [4, 3],
  palette: "ui",
  regions: {
    g: { points: [[0, 0], [3, 0], [0, 2], [3, 2]], z: 0 },
    w: { rect: [0, 0, 4, 3], z: 1 },
  },
}

{
  type: "sprite",
  name: "btn_pressed",
  size: [4, 3],
  palette: "ui",
  regions: {
    b: { rect: [0, 0, 4, 3], z: 0 },
    g: { rect: [1, 1, 2, 1], z: 1 },
  },
}
```

```bash
pxl render ui.pxl --format atlas-godot --padding 1 -o ui_atlas
```

### Production Game Assets

```bash
pxl render characters.pxl \
  --format atlas-godot \
  --max-size 2048x2048 \
  --padding 2 \
  --power-of-two \
  -o assets/characters
```

### Multi-Format Export

```bash
# Generate for all supported engines
pxl render sprites.pxl --format atlas -o assets/sprites
pxl render sprites.pxl --format atlas-godot -o assets/godot/sprites
pxl render sprites.pxl --format atlas-unity -o assets/unity/sprites
pxl render sprites.pxl --format atlas-libgdx -o assets/libgdx/sprites
```

## Related

- [PNG Export](png.md) - Individual sprite export
- [GIF Animation](gif.md) - Animated preview
- [Spritesheet](spritesheet.md) - Simple horizontal strip format
