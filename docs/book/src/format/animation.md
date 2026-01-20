# Animation

Animations define sequences of sprites that play over time. Pixelsrc supports two animation formats:

- **CSS Keyframes** (recommended): Percentage-based keyframes with CSS timing functions
- **Frame Array** (legacy): Simple list of sprite names

Both formats support palette cycling, frame tags, and secondary motion (attachments).

---

## CSS Keyframes Format

The CSS keyframes format uses percentage-based keyframes, familiar to web developers and AI models. This is the recommended approach for new animations.

### Syntax

```json
{
  "type": "animation",
  "name": "string (required)",
  "keyframes": {
    "0%": { "sprite": "...", "opacity": 0.0 },
    "50%": { "sprite": "...", "opacity": 1.0 },
    "100%": { "sprite": "...", "opacity": 0.0 }
  },
  "duration": "500ms",
  "timing_function": "ease-in-out"
}
```

### Keyframe Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `type` | Yes | - | Must be `"animation"` |
| `name` | Yes | - | Unique identifier |
| `keyframes` | Yes | - | Map of percentage keys to keyframe objects |
| `duration` | No | `100` | Total animation duration (ms or CSS time string) |
| `timing_function` | No | `"linear"` | CSS timing function for easing |
| `loop` | No | `true` | Whether animation loops |

### Keyframe Object Fields

Each keyframe can specify any combination of these properties:

| Field | Description |
|-------|-------------|
| `sprite` | Sprite name to display at this keyframe |
| `opacity` | Opacity from 0.0 (transparent) to 1.0 (opaque) |
| `offset` | Position offset `[x, y]` in pixels |
| `transform` | CSS transform string (e.g., `"rotate(45deg)"`) |

### Percentage Keys

Keyframe keys are percentages of the total animation duration:

- `"0%"` - Start of animation
- `"50%"` - Halfway through
- `"100%"` - End of animation
- `"from"` - Alias for `"0%"`
- `"to"` - Alias for `"100%"`

### Duration Format

Duration accepts both raw milliseconds and CSS time strings:

```json
"duration": 500         // 500 milliseconds
"duration": "500ms"     // 500 milliseconds
"duration": "1s"        // 1000 milliseconds
"duration": "0.5s"      // 500 milliseconds
```

### Timing Functions

The `timing_function` field accepts CSS easing functions:

| Function | Description |
|----------|-------------|
| `linear` | Constant speed (default) |
| `ease` | Smooth acceleration and deceleration |
| `ease-in` | Slow start, fast end |
| `ease-out` | Fast start, slow end |
| `ease-in-out` | Slow start and end |
| `cubic-bezier(x1,y1,x2,y2)` | Custom bezier curve |
| `steps(n, position)` | Discrete steps |

### Examples

**Fade in animation:**

```json
{"type": "animation", "name": "fade_in", "keyframes": {"from": {"sprite": "dot", "opacity": 0.0}, "to": {"sprite": "dot", "opacity": 1.0}}, "duration": "1s", "timing_function": "ease"}
```

**Walk cycle with opacity:**

```json
{"type": "animation", "name": "fade_walk", "keyframes": {"0%": {"sprite": "walk_1", "opacity": 0.0}, "50%": {"sprite": "walk_2", "opacity": 1.0}, "100%": {"sprite": "walk_1", "opacity": 0.0}}, "duration": "500ms", "timing_function": "ease-in-out"}
```

**Rotating animation:**

```json
{"type": "animation", "name": "spin", "keyframes": {"0%": {"sprite": "star", "transform": "rotate(0deg)"}, "100%": {"sprite": "star", "transform": "rotate(360deg)"}}, "duration": 1000, "timing_function": "linear"}
```

**Pulsing scale effect:**

```json
{"type": "animation", "name": "pulse", "keyframes": {"0%": {"sprite": "star", "transform": "scale(1)", "opacity": 1.0}, "50%": {"sprite": "star", "transform": "scale(1.5)", "opacity": 0.5}, "100%": {"sprite": "star", "transform": "scale(1)", "opacity": 1.0}}, "duration": "2s", "timing_function": "ease-in-out"}
```

---

## Frame Array Format (Legacy)

The frame array format provides a simple list of sprite names. Use this for straightforward frame-by-frame animations.

### Syntax

```json
{
  "type": "animation",
  "name": "string (required)",
  "frames": ["sprite_name", ...],
  "duration": number,
  "loop": boolean
}
```

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `type` | Yes | - | Must be `"animation"` |
| `name` | Yes | - | Unique identifier |
| `frames` | Yes | - | Array of sprite names in order |
| `duration` | No | 100 | Milliseconds per frame |
| `loop` | No | true | Whether animation loops |

### Example

```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3", "walk_4"], "duration": 100, "loop": true}
```

<!-- DEMOS format/animation#basic_frames -->
<!-- /DEMOS -->

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

<!-- DEMOS format/animation#frame_tags -->
<!-- /DEMOS -->

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

<!-- DEMOS format/animation#timing -->
<!-- /DEMOS -->

## Complete Examples

### CSS Keyframes Example (Recommended)

A blinking light with fade effect:

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

{"type": "animation", "name": "blink_fade", "keyframes": {"0%": {"sprite": "light_on", "opacity": 1.0}, "50%": {"sprite": "light_off", "opacity": 0.5}, "100%": {"sprite": "light_on", "opacity": 1.0}}, "duration": "1s", "timing_function": "ease-in-out"}
```

### Frame Array Example (Legacy)

A simple blinking light:

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

### Try It

Click to render the first frame of the blinking animation:

<div class="pixelsrc-demo" data-pixelsrc-demo>
  <textarea id="animation-demo">{"type": "palette", "name": "blink", "colors": {"{_}": "#0000", "{on}": "#FFFF00", "{off}": "#888800"}}
{"type": "sprite", "name": "light_on", "palette": "blink", "grid": ["{_}{on}{on}{_}", "{on}{on}{on}{on}", "{on}{on}{on}{on}", "{_}{on}{on}{_}"]}
{"type": "sprite", "name": "light_off", "palette": "blink", "grid": ["{_}{off}{off}{_}", "{off}{off}{off}{off}", "{off}{off}{off}{off}", "{_}{off}{off}{_}"]}
{"type": "animation", "name": "blink", "frames": ["light_on", "light_off"], "duration": 500, "loop": true}</textarea>
  <button onclick="pixelsrcDemo.renderFromTextarea('animation-demo', 'animation-demo-preview', {spriteName: 'light_on'})">Show "On" Frame</button>
  <button onclick="pixelsrcDemo.renderFromTextarea('animation-demo', 'animation-demo-preview', {spriteName: 'light_off'})">Show "Off" Frame</button>
  <div class="preview" id="animation-demo-preview"></div>
</div>

Try changing the `{on}` color to `#00FF00` (green) or `#FF0000` (red).

## Rendering Animations

```bash
# Render as animated GIF
pxl render animation.pxl -o output.gif

# Render as spritesheet
pxl render animation.pxl --format spritesheet -o sheet.png

# Preview with onion skinning
pxl show animation.pxl --onion 2
```

---

## Migrating from Frames to Keyframes

Converting from the legacy frame array format to CSS keyframes is straightforward:

### Basic Migration

**Before (frames format):**

```json
{
  "type": "animation",
  "name": "walk",
  "frames": ["walk_1", "walk_2", "walk_3", "walk_4"],
  "duration": 100,
  "loop": true
}
```

**After (keyframes format):**

```json
{
  "type": "animation",
  "name": "walk",
  "keyframes": {
    "0%": {"sprite": "walk_1"},
    "25%": {"sprite": "walk_2"},
    "50%": {"sprite": "walk_3"},
    "75%": {"sprite": "walk_4"}
  },
  "duration": "400ms",
  "loop": true
}
```

### Key Differences

| Aspect | Frames Format | Keyframes Format |
|--------|--------------|------------------|
| Timing | `duration` is per-frame | `duration` is total animation time |
| Structure | Flat sprite array | Percentage-keyed objects |
| Properties | Sprite only | Sprite, opacity, offset, transform |
| Easing | N/A | `timing_function` for interpolation |

### Migration Steps

1. **Calculate total duration**: Multiply per-frame duration by frame count
   - 4 frames × 100ms = 400ms total

2. **Convert to percentages**: Divide frame index by total frames
   - Frame 0 → 0%
   - Frame 1 → 25% (1/4)
   - Frame 2 → 50% (2/4)
   - Frame 3 → 75% (3/4)

3. **Wrap sprites in keyframe objects**: `"walk_1"` becomes `{"sprite": "walk_1"}`

4. **Add timing function** (optional): Use `"timing_function": "linear"` for frame-accurate timing

### When to Migrate

Migrate to keyframes when you need:

- **Opacity changes**: Fade effects between frames
- **Position offsets**: Screen shake, bouncing
- **Transforms**: Rotation, scaling effects
- **CSS timing**: Easing curves for smoother motion

Keep using frames format for:

- Simple frame-by-frame animations with no interpolation
- Quick prototypes
- Backwards compatibility with existing files
