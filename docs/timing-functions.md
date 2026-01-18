# Timing Functions Reference

Pixelsrc supports CSS-compatible timing functions for animations. These control how animated values change over time, enabling smooth motion, discrete steps, and custom easing curves.

## Quick Reference

| Function | Syntax | Use Case |
|----------|--------|----------|
| `linear` | `linear` | Constant speed |
| `ease` | `ease` | Natural motion (default CSS) |
| `ease-in` | `ease-in` | Slow start, fast end |
| `ease-out` | `ease-out` | Fast start, slow end |
| `ease-in-out` | `ease-in-out` | Slow start and end |
| `step-start` | `step-start` | Instant jump at start |
| `step-end` | `step-end` | Instant jump at end |
| `cubic-bezier` | `cubic-bezier(x1, y1, x2, y2)` | Custom curve |
| `steps` | `steps(n)` or `steps(n, position)` | Discrete frames |
| `bounce` | `bounce` | Overshoot and settle |
| `elastic` | `elastic` | Spring oscillation |

## Named Timing Functions

### linear

Constant speed from start to end. No acceleration or deceleration.

```json
{
  "animation-timing-function": "linear"
}
```

**Best for**: Mechanical motion, progress bars, scrolling backgrounds.

### ease

The CSS default. Starts slow, accelerates, then decelerates to a stop.

```json
{
  "animation-timing-function": "ease"
}
```

Equivalent to `cubic-bezier(0.25, 0.1, 0.25, 1.0)`.

### ease-in

Slow start, accelerating toward the end. Creates a sense of building momentum.

```json
{
  "animation-timing-function": "ease-in"
}
```

**Best for**: Objects starting from rest, fade-outs, elements leaving the screen.

### ease-out

Fast start, decelerating toward the end. Creates a sense of settling.

```json
{
  "animation-timing-function": "ease-out"
}
```

**Best for**: Objects coming to rest, fade-ins, elements entering the screen.

### ease-in-out

Slow start and slow end with faster middle. Symmetric S-curve.

```json
{
  "animation-timing-function": "ease-in-out"
}
```

**Best for**: Elements that both enter and exit smoothly, pendulum motion.

### step-start

Instant jump to end value at the start of the animation.

```json
{
  "animation-timing-function": "step-start"
}
```

Equivalent to `steps(1, jump-start)`.

### step-end

Holds at start value until the end, then jumps to end value.

```json
{
  "animation-timing-function": "step-end"
}
```

Equivalent to `steps(1, jump-end)`.

## cubic-bezier()

Define a custom easing curve with two control points.

### Syntax

```
cubic-bezier(x1, y1, x2, y2)
```

- **x1, x2**: Must be between 0 and 1 (time axis)
- **y1, y2**: Can be any value (allows overshoot/undershoot)

### Standard Curves

| Name | Bezier | Behavior |
|------|--------|----------|
| ease | `cubic-bezier(0.25, 0.1, 0.25, 1.0)` | Default CSS timing |
| ease-in | `cubic-bezier(0.42, 0, 1.0, 1.0)` | Slow start |
| ease-out | `cubic-bezier(0, 0, 0.58, 1.0)` | Slow end |
| ease-in-out | `cubic-bezier(0.42, 0, 0.58, 1.0)` | Slow both ends |

### Custom Examples

```json
{
  "// Fast acceleration, slow deceleration": "",
  "animation-timing-function": "cubic-bezier(0.1, 0.9, 0.2, 1.0)"
}
```

```json
{
  "// Overshoot (spring-like)": "",
  "animation-timing-function": "cubic-bezier(0.5, 1.5, 0.5, 1.0)"
}
```

```json
{
  "// Anticipation (pulls back before moving)": "",
  "animation-timing-function": "cubic-bezier(0.5, -0.5, 0.5, 1.0)"
}
```

### Overshoot and Undershoot

Y values outside [0, 1] create dynamic effects:

- **y1 > 1** or **y2 > 1**: Overshoot (value goes past target, settles back)
- **y1 < 0** or **y2 < 0**: Undershoot (value pulls back before moving forward)

```json
{
  "// Spring bounce effect": "",
  "animation-timing-function": "cubic-bezier(0.68, -0.55, 0.265, 1.55)"
}
```

## steps()

Creates discrete jumps between values. Essential for sprite sheet animations.

### Syntax

```
steps(count)
steps(count, position)
```

- **count**: Number of steps (must be >= 1)
- **position**: When the step occurs (optional, defaults to `jump-end`)

### Step Positions

| Position | Aliases | Behavior |
|----------|---------|----------|
| `jump-end` | `end` | Step at interval end (default) |
| `jump-start` | `start` | Step at interval start |
| `jump-none` | — | No step at 0% or 100% |
| `jump-both` | — | Step at both 0% and 100% |

### Visual Guide

For `steps(4, ...)` with input 0.0 → 1.0:

```
jump-end (default):     jump-start:
  1.0 ──────┐             1.0 ┌──────
            │                 │
 0.75 ────┐ │            0.75 │ ┌────
          │ │                 │ │
  0.5 ──┐ │ │             0.5 │ │ ┌──
        │ │ │                 │ │ │
 0.25 ┐ │ │ │            0.25 │ │ │ ┌
      │ │ │ │                 │ │ │ │
  0.0 └─┴─┴─┴─             0.0 ┴─┴─┴─┘
      0   0.5  1              0   0.5  1
```

### jump-end (default)

The most common for sprite animations. Each frame shows for a full interval, then advances at the interval boundary.

```json
{
  "animation-timing-function": "steps(8, jump-end)"
}
```

**Output values**: 0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, then 1.0 at the end.

### jump-start

Immediately advances to the first step. Useful when you don't want to show the initial frame.

```json
{
  "animation-timing-function": "steps(4, jump-start)"
}
```

At t=0: output is 0 (but immediately jumps to 0.25 for any t > 0).

### jump-none

No jump at the beginning or end. The output smoothly holds at 0 and 1.

```json
{
  "animation-timing-function": "steps(4, jump-none)"
}
```

**Note**: Requires count >= 2.

**Output values**: 0, 0.333..., 0.666..., 1.0

### jump-both

Jumps at both ends. Never outputs exactly 0 or 1 except at the boundaries.

```json
{
  "animation-timing-function": "steps(4, jump-both)"
}
```

**Output values**: 0.2, 0.4, 0.6, 0.8, then 1.0

## Sprite Sheet Animation

The `steps()` function is designed for frame-by-frame animation.

### Basic Sprite Animation

For an 8-frame walk cycle:

```json
{
  "animation": "walk 1s steps(8) infinite"
}
```

This cycles through frames 0-7, spending 125ms on each frame.

### Frame Index Calculation

Given animation progress `t` (0.0 to 1.0) and `steps(n, jump-end)`:

```
frame_index = floor(t * n)
```

For 8 frames, each 1/8 of the animation shows one frame:
- t in [0, 0.125) → frame 0
- t in [0.125, 0.25) → frame 1
- ...
- t in [0.875, 1.0) → frame 7
- t = 1.0 → frame 7 (end of animation)

### Choosing Step Position

| Position | Use When |
|----------|----------|
| `jump-end` | Default choice. Frame 0 shows first. |
| `jump-start` | Skip the first frame (e.g., idle pose at start of action) |
| `jump-none` | Hold first and last frames longer |
| `jump-both` | Never show pure start/end states |

### Example: Character States

```json
{
  "idle": {
    "animation": "idle-sprite 2s steps(4) infinite",
    "// Shows frames 0,1,2,3 in a loop": ""
  },
  "run": {
    "animation": "run-sprite 0.5s steps(8) infinite",
    "// Fast 8-frame run cycle": ""
  },
  "attack": {
    "animation": "attack-sprite 0.3s steps(6, jump-start) forwards",
    "// Skip idle frame, play attack frames": ""
  }
}
```

## Pixel Art Considerations

### Prefer Discrete Steps

For pixel art animations, `steps()` often looks better than smooth easing because:

1. **Matches the medium**: Pixel art is inherently discrete
2. **Cleaner motion**: No sub-pixel interpolation artifacts
3. **Predictable**: Each frame is a complete, hand-crafted image

### When to Use Smooth Easing

Smooth easing (`ease`, `cubic-bezier`) is appropriate for:

- Position/movement interpolation (when rendering handles sub-pixel)
- Opacity fades
- Scale transforms
- Camera motion

### Motion Path + Steps

Combine discrete frame display with smooth position:

```json
{
  "// Frame display is stepped, but position is eased": "",
  "animation": "walk-frames 1s steps(8) infinite",
  "transform": "translateX(var(--walk-progress))",
  "transition": "transform 1s ease-out"
}
```

## Extension Functions

Pixelsrc includes additional easing functions beyond CSS:

### bounce

Overshoots the target and settles back. Like a ball hitting a surface.

```json
{
  "animation-timing-function": "bounce"
}
```

**Best for**: Landing impacts, button presses, notifications appearing.

### elastic

Spring-like oscillation that overshoots and oscillates before settling.

```json
{
  "animation-timing-function": "elastic"
}
```

**Best for**: Springy UI elements, character expressions, emphasis animations.

**Note**: Output can exceed [0, 1] range due to oscillation.

## Parsing and Errors

### Valid Formats

```
linear
ease-in-out
cubic-bezier(0.25, 0.1, 0.25, 1.0)
cubic-bezier( 0.25 , 0.1 , 0.25 , 1.0 )   // Extra whitespace OK
steps(4)
steps(4, jump-end)
steps(4, end)         // Shorthand
STEPS(4, JUMP-END)    // Case insensitive
```

### Error Conditions

| Input | Error |
|-------|-------|
| `""` | Empty timing function |
| `unknown` | Unknown timing function |
| `cubic-bezier(0.25, 0.1)` | Expected 4 values |
| `cubic-bezier(-0.1, 0, 1, 1)` | x1 must be between 0 and 1 |
| `steps(0)` | Step count must be at least 1 |
| `steps(1, jump-none)` | jump-none requires at least 2 steps |
| `steps(4, invalid)` | Unknown step position |

## API Reference

### parse_timing_function

Parse a CSS timing function string:

```rust
use pixelsrc::motion::{parse_timing_function, Interpolation};

let linear = parse_timing_function("linear")?;
let bezier = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)")?;
let steps = parse_timing_function("steps(4, jump-start)")?;
```

### ease

Apply easing to a normalized time value:

```rust
use pixelsrc::motion::{ease, Interpolation};

let t = 0.5; // Halfway through animation
let eased = ease(t, &Interpolation::EaseInOut);
// eased ≈ 0.5 for ease-in-out
```

### interpolate_value

Interpolate between two values with easing:

```rust
use pixelsrc::motion::{interpolate_value, Interpolation};

let start = 0.0;
let end = 100.0;
let t = 0.5;
let result = interpolate_value(start, end, t, &Interpolation::EaseOut);
// result > 50.0 (ease-out is faster at start)
```

## See Also

- [CSS Easing Functions Level 2](https://www.w3.org/TR/css-easing-2/)
- [Animation Keyframes](./keyframes.md)
- [Transform Functions](./transforms.md)
