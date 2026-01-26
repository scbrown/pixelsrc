# CSS Keyframes

Keyframes define the states of an animation at specific points in time. Pixelsrc uses CSS-style keyframe syntax for animations.

## Percentage Keyframes

<!-- DEMOS format/css/keyframes#percentage -->
**Percentage Keyframes**

Animation using 0%, 50%, 100% keyframes with opacity and sprite changes.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "walk_colors", "colors": {"{_}": "#00000000", "{c}": "#FF0000"}}
{"type": "sprite", "name": "walk_1", "palette": "walk_colors", "size": [2, 2], "regions": {"c": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "walk_2", "palette": "walk_colors", "size": [2, 2], "regions": {"c": {"rect": [0, 0, 2, 2]}}}
{"type": "animation", "name": "fade_walk", "duration": "1s", "timing_function": "ease-in-out", "keyframes": {"0%": {"sprite": "walk_1", "opacity": 0.0}, "50%": {"sprite": "walk_2", "opacity": 1.0}, "100%": {"sprite": "walk_1", "opacity": 0.0}}}
```

</div>

<div class="demo-container" data-demo="percentage">
</div>
<!-- /DEMOS -->

Define keyframes using percentage values:

```json
{
  "type": "animation",
  "name": "bounce",
  "duration": "1s",
  "keyframes": {
    "0%": { "sprite": "ball", "transform": "translate(0, 0)" },
    "50%": { "sprite": "ball", "transform": "translate(0, -10)" },
    "100%": { "sprite": "ball", "transform": "translate(0, 0)" }
  }
}
```

| Percentage | Meaning |
|------------|---------|
| `0%` | Animation start |
| `50%` | Animation midpoint |
| `100%` | Animation end |
| Any value | Intermediate state |

## from/to Keywords

<!-- DEMOS format/css/keyframes#from_to -->
**From/To Keyframes**

Animation using from/to aliases (equivalent to 0%/100%).

<div class="demo-source">

```jsonl
{"type": "palette", "name": "dot_colors", "colors": {"{_}": "#00000000", "{d}": "#FF0000"}}
{"type": "sprite", "name": "dot", "palette": "dot_colors", "size": [1, 1], "regions": {"d": {"points": [[0, 0]]}}}
{"type": "animation", "name": "fade_in", "duration": "1s", "keyframes": {"from": {"sprite": "dot", "opacity": 0.0}, "to": {"sprite": "dot", "opacity": 1.0}}}
```

</div>

<div class="demo-container" data-demo="from_to">
</div>
<!-- /DEMOS -->

Use `from` and `to` as aliases for `0%` and `100%`:

```json
{
  "type": "animation",
  "name": "fade",
  "duration": "500ms",
  "keyframes": {
    "from": { "sprite": "icon", "opacity": 0 },
    "to": { "sprite": "icon", "opacity": 1 }
  }
}
```

| Keyword | Equivalent |
|---------|------------|
| `from` | `0%` |
| `to` | `100%` |

## Sprite Changes

<!-- DEMOS format/css/keyframes#sprite_changes -->
**Sprite Changes at Keyframes**

Animation that changes sprites at different keyframes (idle -> jump -> land -> idle).

<div class="demo-source">

```jsonl
{"type": "palette", "name": "char_colors", "colors": {"{_}": "#00000000", "{c}": "#0000FF"}}
{"type": "sprite", "name": "char_idle", "palette": "char_colors", "size": [2, 2], "regions": {"c": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "char_jump", "palette": "char_colors", "size": [2, 2], "regions": {"c": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "char_land", "palette": "char_colors", "size": [2, 2], "regions": {"c": {"rect": [0, 0, 2, 2]}}}
{"type": "animation", "name": "jump_cycle", "duration": "800ms", "keyframes": {"0%": {"sprite": "char_idle"}, "25%": {"sprite": "char_jump"}, "75%": {"sprite": "char_land"}, "100%": {"sprite": "char_idle"}}}
```

</div>

<div class="demo-container" data-demo="sprite_changes">
</div>
<!-- /DEMOS -->

Switch between different sprites at keyframes:

```json
{
  "type": "animation",
  "name": "walk",
  "duration": "400ms",
  "timing_function": "steps(4)",
  "keyframes": {
    "0%": { "sprite": "walk_1" },
    "25%": { "sprite": "walk_2" },
    "50%": { "sprite": "walk_3" },
    "75%": { "sprite": "walk_4" },
    "100%": { "sprite": "walk_1" }
  }
}
```

Use `steps()` timing when switching sprites to prevent blending between frames.

## Transform Animations

<!-- DEMOS format/css/keyframes#transforms -->
**Transform Animations**

Animations using CSS transforms (rotate, scale) at keyframes.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "shape_colors", "colors": {"{_}": "#00000000", "{s}": "#FF00FF"}}
{"type": "sprite", "name": "shape", "palette": "shape_colors", "size": [2, 2], "regions": {"s": {"rect": [0, 0, 2, 2]}}}
{"type": "animation", "name": "spin", "duration": "1s", "timing_function": "linear", "keyframes": {"0%": {"sprite": "shape", "transform": "rotate(0deg)"}, "100%": {"sprite": "shape", "transform": "rotate(360deg)"}}}
{"type": "animation", "name": "pulse", "duration": "500ms", "timing_function": "ease-in-out", "keyframes": {"0%": {"sprite": "shape", "transform": "scale(1)", "opacity": 1.0}, "50%": {"sprite": "shape", "transform": "scale(1.5)", "opacity": 0.5}, "100%": {"sprite": "shape", "transform": "scale(1)", "opacity": 1.0}}}
```

</div>

<div class="demo-container" data-demo="transforms">
</div>
<!-- /DEMOS -->

Animate CSS transforms across keyframes:

```json
{
  "type": "animation",
  "name": "spin",
  "duration": "1s",
  "timing_function": "linear",
  "keyframes": {
    "0%": { "sprite": "star", "transform": "rotate(0deg)" },
    "100%": { "sprite": "star", "transform": "rotate(360deg)" }
  }
}
```

### Supported Transform Properties

| Property | Example | Description |
|----------|---------|-------------|
| `translate` | `translate(10, 5)` | Move position |
| `rotate` | `rotate(90deg)` | Rotate clockwise |
| `scale` | `scale(2)` | Scale size |
| `flip` | `flip(x)` | Mirror horizontally/vertically |

### Combined Transforms

Multiple transforms can be combined:

```json
{
  "keyframes": {
    "0%": {
      "sprite": "ship",
      "transform": "translate(0, 0) rotate(0deg) scale(1)"
    },
    "100%": {
      "sprite": "ship",
      "transform": "translate(20, -10) rotate(45deg) scale(1.5)"
    }
  }
}
```

## Opacity

Animate transparency:

```json
{
  "keyframes": {
    "0%": { "sprite": "ghost", "opacity": 1.0 },
    "50%": { "sprite": "ghost", "opacity": 0.3 },
    "100%": { "sprite": "ghost", "opacity": 1.0 }
  }
}
```

| Value | Description |
|-------|-------------|
| `1.0` | Fully opaque |
| `0.5` | 50% transparent |
| `0.0` | Fully transparent |

## Keyframe Interpolation

Values between keyframes are interpolated based on the timing function:

- **Transforms**: Position, rotation, and scale interpolate smoothly
- **Opacity**: Interpolates linearly
- **Sprites**: Use `steps()` timing to avoid blending

## Best Practices

1. **Use `steps()` for sprite swaps**: Prevents blurry transitions
2. **Keep transform chains consistent**: Same transforms in same order across keyframes
3. **Use `from`/`to` for simple animations**: More readable than `0%`/`100%`
4. **Add intermediate keyframes for complex motion**: Control the path, not just start/end
5. **Match loop points**: For looping animations, ensure `100%` matches `0%`
