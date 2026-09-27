# Particle Systems

A particle system emits copies of a sprite with randomized, physically-simulated
motion — sparks, embers, dust, rain, snow. The simulation is **seeded and
deterministic**: the same definition always produces the same frames, so a
particle effect can be *baked* to a fixed frame-set and shipped like any other
animation.

## Basic Syntax

```json5
{
  type: "particle",
  name: "embers",
  sprite: "ember",        // the sprite emitted as each particle
  emitter: {
    rate: 1.5,            // particles spawned per frame (fractional accumulates)
    lifetime: [6, 10],    // frames each particle lives, [min, max]
    velocity: { x: [-0.4, 0.4], y: [-1.2, -0.6] },  // initial velocity range
    gravity: -0.02,       // per-frame acceleration on vy (negative = rises)
    fade: true,           // fade alpha to zero over the particle's lifetime
    rotation: [0, 0],     // rotation range in degrees (optional)
    seed: 7,              // RNG seed — fix it for reproducible bakes
  },
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"particle"` |
| `name` | Yes | Unique identifier (and the base name of the baked frame-set) |
| `sprite` | Yes | Name of the sprite to emit as each particle |
| `emitter` | Yes | Emitter configuration (below) |

### Emitter

| Field | Default | Description |
|-------|---------|-------------|
| `rate` | `1.0` | Particles emitted per frame; fractional rates accumulate across frames |
| `lifetime` | `[10, 20]` | Per-particle lifetime in frames, `[min, max]` |
| `velocity` | none | Initial velocity range `{ x: [min, max], y: [min, max] }` |
| `gravity` | `0` | Acceleration added to `vy` each frame (negative rises, positive falls) |
| `fade` | `false` | If true, alpha fades linearly to zero over the lifetime |
| `rotation` | `[0, 0]` | Rotation range in degrees |
| `seed` | `42` | RNG seed — **set this** so bakes are reproducible |

## Baking to Frames

Particle systems are runtime constructs, so `pxl render` does **not** draw them
statically by default. Pass `--frames N` to simulate the emitter and bake `N`
frames:

```bash
# A numbered PNG per frame: embers_00.png … embers_11.png
pxl render embers.pxl --frames 12 --canvas 16x32 --origin 8,31 -o frames/

# A single animated GIF instead
pxl render embers.pxl --frames 12 --canvas 16x32 --gif --fps 12 -o embers.gif
```

| Flag | Default | Description |
|------|---------|-------------|
| `--frames N` | — | Bake `N` frames (required to render a particle system) |
| `--canvas WxH` | `32x32` | Output canvas size |
| `--origin X,Y` | bottom-center | Emitter origin on the canvas |
| `--gif` | off | Emit one animated GIF instead of numbered PNGs |
| `--fps N` | `10` | GIF playback rate |
| `--sprite NAME` | all | Bake only the named particle system |

The PNG output follows the frame-set naming convention `{name}_NN.png`, so a
baked particle drops straight into an [animation](animation.md) `frames` array or
any pipeline that consumes numbered frames. Because the simulation is seeded,
re-baking the same file reproduces byte-identical frames — safe to commit and
diff.

> **Tip:** the emitted `sprite` is a normal sprite, so it can itself use
> [`extends`](sprite.md#region-inheritance-extends), transforms, or any palette.
