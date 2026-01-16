# The Game Developer

You're building a game and need **production-ready assets**. Spritesheets, atlas formats, and smooth integration with your engine.

## Your Workflow

1. Create sprites and animations in Pixelsrc
2. Export to spritesheets or atlas formats
3. Import into your game engine
4. Iterate without breaking references

## Spritesheet Export

### Basic Spritesheet

Export all sprites to a single spritesheet:

```bash
pxl render characters.pxl -o spritesheet.png --spritesheet
```

This arranges sprites in a grid and generates metadata.

### With Metadata

Generate JSON atlas alongside the PNG:

```bash
pxl render characters.pxl -o characters.png --spritesheet --atlas json
```

Output:
- `characters.png` - The spritesheet image
- `characters.json` - Frame coordinates and metadata

### Atlas Formats

Pixelsrc supports common game engine formats:

```bash
# Generic JSON (works with most engines)
pxl render sprites.pxl -o atlas.png --spritesheet --atlas json

# Aseprite format
pxl render sprites.pxl -o atlas.png --spritesheet --atlas aseprite

# TexturePacker format
pxl render sprites.pxl -o atlas.png --spritesheet --atlas texturepacker
```

## Atlas JSON Structure

The JSON atlas contains everything your engine needs:

```json
{
  "frames": {
    "hero_idle": {
      "frame": {"x": 0, "y": 0, "w": 16, "h": 16},
      "sourceSize": {"w": 16, "h": 16},
      "spriteSourceSize": {"x": 0, "y": 0, "w": 16, "h": 16}
    },
    "hero_walk_1": {
      "frame": {"x": 16, "y": 0, "w": 16, "h": 16},
      "sourceSize": {"w": 16, "h": 16},
      "spriteSourceSize": {"x": 0, "y": 0, "w": 16, "h": 16}
    }
  },
  "animations": {
    "hero_walk": {
      "frames": ["hero_walk_1", "hero_walk_2", "hero_walk_3", "hero_walk_4"],
      "duration": 100,
      "loop": true
    }
  },
  "meta": {
    "image": "characters.png",
    "size": {"w": 256, "h": 256},
    "scale": 1
  }
}
```

## Engine Integration Examples

### Godot

Load the atlas in GDScript:

```gdscript
var atlas_data = load("res://assets/characters.json")
var texture = load("res://assets/characters.png")

func get_sprite_rect(sprite_name: String) -> Rect2:
    var frame = atlas_data.frames[sprite_name].frame
    return Rect2(frame.x, frame.y, frame.w, frame.h)
```

### Unity

Use with Unity's Sprite Atlas or custom importer:

```csharp
[System.Serializable]
public class PixelsrcAtlas {
    public Dictionary<string, FrameData> frames;
    public Dictionary<string, AnimationData> animations;
}

// Load and create sprites
var json = Resources.Load<TextAsset>("characters");
var atlas = JsonUtility.FromJson<PixelsrcAtlas>(json.text);
```

### Phaser

```javascript
this.load.json('atlas_data', 'assets/characters.json');
this.load.image('atlas_image', 'assets/characters.png');

// In create()
const data = this.cache.json.get('atlas_data');
// Use frame coordinates from data.frames
```

### Love2D

```lua
local json = require("json")
local atlas_data = json.decode(love.filesystem.read("characters.json"))
local atlas_image = love.graphics.newImage("characters.png")

function draw_sprite(name, x, y)
    local frame = atlas_data.frames[name].frame
    local quad = love.graphics.newQuad(
        frame.x, frame.y, frame.w, frame.h,
        atlas_image:getDimensions()
    )
    love.graphics.draw(atlas_image, quad, x, y)
end
```

## Spritesheet Options

### Grid Configuration

Control sprite arrangement:

```bash
# Fixed columns
pxl render sprites.pxl -o sheet.png --spritesheet --columns 8

# With padding between sprites
pxl render sprites.pxl -o sheet.png --spritesheet --padding 2

# Power-of-two dimensions (GPU friendly)
pxl render sprites.pxl -o sheet.png --spritesheet --pot
```

### Scaling

Export at different scales:

```bash
# 1x for mobile
pxl render sprites.pxl -o sheet_1x.png --spritesheet --scale 1

# 2x for retina
pxl render sprites.pxl -o sheet_2x.png --spritesheet --scale 2
```

## Animation Integration

### Frame Timing

Access animation data from the atlas:

```javascript
const anim = atlas.animations["hero_walk"];
// anim.frames = ["hero_walk_1", "hero_walk_2", ...]
// anim.duration = 100 (ms per frame)
// anim.loop = true
```

### Playing Animations

```javascript
class AnimatedSprite {
    constructor(animData) {
        this.frames = animData.frames;
        this.duration = animData.duration;
        this.loop = animData.loop;
        this.currentFrame = 0;
        this.elapsed = 0;
    }

    update(dt) {
        this.elapsed += dt;
        if (this.elapsed >= this.duration) {
            this.elapsed -= this.duration;
            this.currentFrame++;
            if (this.currentFrame >= this.frames.length) {
                this.currentFrame = this.loop ? 0 : this.frames.length - 1;
            }
        }
    }

    getCurrentFrameName() {
        return this.frames[this.currentFrame];
    }
}
```

## Build Pipeline

### Watch and Rebuild

During development, auto-rebuild on changes:

```bash
pxl build src/ -o assets/ --watch --spritesheet
```

### Multiple Spritesheets

Organize by category for memory management:

```bash
pxl render src/characters/*.pxl -o assets/characters.png --spritesheet --atlas json
pxl render src/items/*.pxl -o assets/items.png --spritesheet --atlas json
pxl render src/ui/*.pxl -o assets/ui.png --spritesheet --atlas json
```

## Best Practices

### Naming Conventions

Use consistent naming for easy engine integration:

```
hero_idle_1
hero_idle_2
hero_walk_1
hero_walk_2
hero_attack_1
hero_attack_2
```

This allows pattern-based loading: `hero_walk_*` gets all walk frames.

### Separate Source and Output

```
game/
├── sprites/          # Pixelsrc source files (.pxl)
│   ├── characters/
│   └── items/
├── assets/           # Generated output (in .gitignore)
│   ├── characters.png
│   ├── characters.json
│   └── items.png
└── pxl.toml
```

### Stable References

Keep sprite names stable. Your game code references these names—changing them breaks things. Use variants for different versions:

```json
{"type": "sprite", "name": "hero_idle", ...}
{"type": "variant", "name": "hero_idle_v2", "base": "hero_idle", ...}
```

Test `v2` without breaking `hero_idle` references.

## Next Steps

- Set up CI/CD for asset building (see [The Tool Builder](tool-builder.md))
- Learn about animation best practices (see [The Animator](animator.md))
- Export to specific formats (see [Export Formats](../exports/spritesheet.md))
