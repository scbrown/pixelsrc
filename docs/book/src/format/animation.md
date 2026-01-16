# Animation

Animations define sequences of sprites that play over time. Pixelsrc supports frame-based animations, palette cycling, frame tags, and secondary motion (attachments).

## Basic Syntax

```json
{
  "type": "animation",
  "name": "string (required)",
  "frames": ["sprite_name", ...],
  "duration": number,
  "loop": boolean
}
```

## Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `type` | Yes | - | Must be `"animation"` |
| `name` | Yes | - | Unique identifier |
| `frames` | Yes | - | Array of sprite names in order |
| `duration` | No | 100 | Milliseconds per frame |
| `loop` | No | true | Whether animation loops |

## Example

```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3", "walk_4"], "duration": 100, "loop": true}
```

## Frame References

Frames reference sprites by name. The sprites must be defined earlier in the file:

```json
{"type": "palette", "name": "hero", "colors": {...}}
{"type": "sprite", "name": "idle_1", "palette": "hero", "grid": [...]}
{"type": "sprite", "name": "idle_2", "palette": "hero", "grid": [...]}
{"type": "animation", "name": "idle", "frames": ["idle_1", "idle_2"], "duration": 500}
```

## Palette Cycling

Animate by rotating palette colors instead of changing sprites. This classic technique creates efficient water, fire, and energy effects.

```json
{
  "type": "animation",
  "name": "waterfall",
  "sprite": "water_static",
  "palette_cycle": {
    "tokens": ["{water1}", "{water2}", "{water3}", "{water4}"],
    "fps": 8,
    "direction": "forward"
  }
}
```

### Palette Cycle Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `sprite` | Yes* | - | Single sprite to cycle (*required if no `frames`) |
| `palette_cycle` | Yes | - | Cycle definition object or array |
| `palette_cycle.tokens` | Yes | - | Ordered list of tokens to rotate |
| `palette_cycle.fps` | No | 10 | Frames per second for cycling |
| `palette_cycle.direction` | No | `"forward"` | `"forward"` or `"reverse"` |

### Multiple Cycles

Run several palette cycles simultaneously:

```json
{
  "palette_cycle": [
    {"tokens": ["{water1}", "{water2}", "{water3}"], "fps": 8},
    {"tokens": ["{glow1}", "{glow2}"], "fps": 4}
  ]
}
```

## Frame Tags

Mark frame ranges with semantic names for game engine integration:

```json
{
  "type": "animation",
  "name": "player",
  "frames": ["idle1", "idle2", "run1", "run2", "run3", "run4", "jump", "fall"],
  "fps": 10,
  "tags": {
    "idle": {"start": 0, "end": 1, "loop": true},
    "run": {"start": 2, "end": 5, "loop": true},
    "jump": {"start": 6, "end": 6, "loop": false},
    "fall": {"start": 7, "end": 7, "loop": false}
  }
}
```

### Tag Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `tags` | No | - | Map of tag name to tag definition |
| `tags.{name}.start` | Yes | - | Starting frame index (0-based) |
| `tags.{name}.end` | Yes | - | Ending frame index (inclusive) |
| `tags.{name}.loop` | No | true | Whether this segment loops |
| `tags.{name}.fps` | No | inherit | Override FPS for this tag |

Tags allow game engines to play specific sub-animations by name (e.g., "play the run tag").

## Per-Frame Metadata

Define hitboxes and metadata that vary across frames:

```json
{
  "type": "animation",
  "name": "attack",
  "frames": ["attack_1", "attack_2", "attack_3"],
  "frame_metadata": [
    {"boxes": {"hit": null}},
    {"boxes": {"hit": {"x": 20, "y": 8, "w": 20, "h": 16}}},
    {"boxes": {"hit": {"x": 24, "y": 4, "w": 24, "h": 20}}}
  ]
}
```

Frame 1 has no hitbox (`null`), while frames 2 and 3 have active hit regions.

## Secondary Motion (Attachments)

Animate attached elements like hair, capes, or tails that follow the parent animation with configurable delay:

```json
{
  "type": "animation",
  "name": "hero_walk",
  "frames": ["walk_1", "walk_2", "walk_3", "walk_4"],
  "duration": 100,
  "attachments": [
    {
      "name": "hair",
      "anchor": [12, 4],
      "chain": ["hair_1", "hair_2", "hair_3"],
      "delay": 1,
      "follow": "position"
    },
    {
      "name": "cape",
      "anchor": [8, 8],
      "chain": ["cape_top", "cape_mid", "cape_bottom"],
      "delay": 2,
      "follow": "velocity",
      "z_index": -1
    }
  ]
}
```

### Attachment Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `attachments` | No | - | Array of attachment definitions |
| `attachments[].name` | Yes | - | Identifier for this attachment |
| `attachments[].anchor` | Yes | - | Attachment point `[x, y]` on parent sprite |
| `attachments[].chain` | Yes | - | Array of sprite names forming the chain |
| `attachments[].delay` | No | 1 | Frame delay between chain segments |
| `attachments[].follow` | No | `"position"` | `"position"`, `"velocity"`, or `"rotation"` |
| `attachments[].damping` | No | 0.8 | Oscillation damping (0.0-1.0) |
| `attachments[].stiffness` | No | 0.5 | Spring stiffness (0.0-1.0) |
| `attachments[].z_index` | No | 0 | Render order (negative = behind parent) |

### Follow Modes

| Mode | Behavior |
|------|----------|
| `position` | Chain follows parent position directly |
| `velocity` | Chain responds to movement velocity |
| `rotation` | Chain responds to rotation changes |

## Duration vs FPS

You can specify timing using either `duration` (ms per frame) or `fps` (frames per second):

```json
{"type": "animation", "name": "fast", "frames": [...], "duration": 50}
{"type": "animation", "name": "fast", "frames": [...], "fps": 20}
```

Both examples create the same 20 FPS animation.

## Complete Example

```json
{"type": "palette", "name": "blink", "colors": {"{_}": "#0000", "{on}": "#FF0", "{off}": "#880"}}

{"type": "sprite", "name": "light_on", "palette": "blink", "grid": [
  "{_}{on}{on}{_}",
  "{on}{on}{on}{on}",
  "{on}{on}{on}{on}",
  "{_}{on}{on}{_}"
]}

{"type": "sprite", "name": "light_off", "palette": "blink", "grid": [
  "{_}{off}{off}{_}",
  "{off}{off}{off}{off}",
  "{off}{off}{off}{off}",
  "{_}{off}{off}{_}"
]}

{"type": "animation", "name": "blink", "frames": ["light_on", "light_off"], "duration": 500, "loop": true}
```

## Rendering Animations

```bash
# Render as animated GIF
pxl render animation.pxl -o output.gif

# Render as spritesheet
pxl render animation.pxl --format spritesheet -o sheet.png

# Preview with onion skinning
pxl show animation.pxl --onion 2
```
