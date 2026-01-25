# The Animator

You bring sprites to life with **motion**. Walk cycles, attack animations, idle bobbing—you think in frames and timing.

## Your Workflow

1. Create keyframe sprites
2. Define animation sequences
3. Preview and adjust timing
4. Export to GIF or spritesheet

## Animation Basics

An animation references a sequence of sprites:

```json5
{
  type: "sprite",
  name: "coin_1",
  size: [6, 6],
  palette: "coin",
  regions: { body: { ellipse: [3, 3, 3, 3], z: 0 } },
}

{
  type: "sprite",
  name: "coin_2",
  size: [6, 6],
  palette: "coin",
  regions: { body: { ellipse: [3, 3, 2, 3], z: 0 } },
}

{
  type: "animation",
  name: "coin_spin",
  frames: ["coin_1", "coin_2"],
  duration: 100,
}
```

Key properties:
- **frames**: Array of sprite names in play order
- **duration**: Milliseconds per frame (default: 100)
- **loop**: Whether to loop (default: true)

## Example: Coin Spin

A classic 4-frame coin rotation:

```json5
{
  type: "palette",
  name: "coin",
  colors: {
    _: "transparent",
    gold: "#FFD700",
    gold_light: "#FFEC8B",
    gold_dark: "#DAA520",
    shine: "#FFFFFF",
  },
}

// Frame 1: Full face
{
  type: "sprite",
  name: "coin_1",
  size: [6, 6],
  palette: "coin",
  regions: {
    gold: {
      union: [
        { rect: [1, 0, 4, 1] },
        { rect: [0, 1, 6, 4] },
        { rect: [1, 5, 4, 1] },
      ],
      z: 0,
    },
    gold_light: { rect: [1, 1, 2, 1], z: 1 },
    shine: { points: [[1, 2]], z: 2 },
    gold_dark: { points: [[4, 4]], z: 1 },
  },
}

// Frame 2: Turning
{
  type: "sprite",
  name: "coin_2",
  size: [6, 6],
  palette: "coin",
  regions: {
    gold: {
      union: [
        { rect: [2, 0, 2, 1] },
        { rect: [1, 1, 4, 4] },
        { rect: [2, 5, 2, 1] },
      ],
      z: 0,
    },
    gold_light: { rect: [2, 1, 1, 1], z: 1 },
    shine: { points: [[2, 2]], z: 2 },
    gold_dark: { points: [[3, 4]], z: 1 },
  },
}

// Frame 3: Edge view
{
  type: "sprite",
  name: "coin_3",
  size: [6, 6],
  palette: "coin",
  regions: {
    gold: { rect: [2, 0, 2, 6], z: 0 },
    gold_light: { points: [[2, 1]], z: 1 },
    gold_dark: { points: [[3, 4]], z: 1 },
  },
}

// Frame 4: Returning
{
  type: "sprite",
  name: "coin_4",
  size: [6, 6],
  palette: "coin",
  regions: {
    gold: {
      union: [
        { rect: [2, 0, 2, 1] },
        { rect: [1, 1, 4, 4] },
        { rect: [2, 5, 2, 1] },
      ],
      z: 0,
    },
    gold_light: { rect: [3, 1, 1, 1], z: 1 },
    shine: { points: [[3, 2]], z: 2 },
    gold_dark: { points: [[2, 4]], z: 1 },
  },
}

{
  type: "animation",
  name: "coin_spin",
  frames: ["coin_1", "coin_2", "coin_3", "coin_4"],
  duration: 120,
  loop: true,
}
```

## Preview Animations

Preview in terminal:

```bash
pxl show coin.pxl --name coin_spin
```

The animation plays in your terminal using ANSI colors.

## Export to GIF

```bash
pxl render coin.pxl --name coin_spin -o coin.gif
```

For scaled output:

```bash
pxl render coin.pxl --name coin_spin -o coin.gif --scale 4
```

## Timing Tips

### Frame Duration

- **Fast action** (attacks, impacts): 50-80ms per frame
- **Standard motion** (walking, running): 80-120ms per frame
- **Slow motion** (idle breathing, floating): 150-250ms per frame

### Frame Count Guidelines

| Animation Type | Typical Frames |
|----------------|----------------|
| Idle breathing | 2-4 frames |
| Walk cycle | 4-8 frames |
| Run cycle | 6-8 frames |
| Attack | 3-6 frames |
| Jump | 4-6 frames |
| Death | 4-8 frames |

### Ease-In/Ease-Out

Hold keyframes longer than in-between frames:

```json5
{
  type: "animation",
  name: "attack",
  frames: [
    "attack_windup",
    "attack_windup",
    "attack_swing",
    "attack_impact",
    "attack_impact",
    "attack_recover",
  ],
  duration: 80,
}
```

By repeating `attack_windup` and `attack_impact`, you create anticipation and follow-through.

## Walk Cycle Example

A basic 4-frame walk:

```json5
{
  type: "sprite",
  name: "walk_1",
  size: [8, 12],
  palette: "character",
  regions: {
    body: { rect: [2, 0, 4, 8], z: 0 },
    leg_l: { rect: [2, 8, 2, 4], z: 1 },
    leg_r: { rect: [4, 8, 2, 4], z: 1 },
  },
}

{
  type: "sprite",
  name: "walk_2",
  size: [8, 12],
  palette: "character",
  regions: {
    body: { rect: [2, 0, 4, 8], z: 0 },
    leg_l: { rect: [1, 8, 2, 4], z: 1 },
    leg_r: { rect: [5, 8, 2, 4], z: 1 },
  },
}

{
  type: "animation",
  name: "walk_right",
  frames: ["walk_1", "walk_2", "walk_1", "walk_2"],
  duration: 100,
}
```

For mirrored walk (walking left), you can use transforms in compositions or create separate sprites.

## Non-Looping Animations

For one-shot animations like death or victory:

```json5
{
  type: "animation",
  name: "death",
  frames: ["death_1", "death_2", "death_3", "death_final"],
  duration: 120,
  loop: false,
}
```

## Organizing Animation Files

Structure for a character with multiple animations:

```
hero/
├── hero_palette.pxl
├── hero_idle.pxl
├── hero_walk.pxl
├── hero_attack.pxl
└── hero_animations.pxl
```

The `hero_animations.pxl` file includes all sprites and defines animations:

```json5
{ type: "include", path: "hero_palette.pxl" }
{ type: "include", path: "hero_idle.pxl" }
{ type: "include", path: "hero_walk.pxl" }
{ type: "include", path: "hero_attack.pxl" }

{ type: "animation", name: "hero_idle", frames: ["idle_1", "idle_2"], duration: 300 }
{ type: "animation", name: "hero_walk", frames: ["walk_1", "walk_2", "walk_3", "walk_4"], duration: 100 }
{ type: "animation", name: "hero_attack", frames: ["attack_1", "attack_2", "attack_3"], duration: 80 }
```

## Next Steps

- Export to spritesheets for game engines (see [The Game Developer](game-developer.md))
- Learn about transforms for flipping and rotating (see [Format Specification](../format/transforms.md))
