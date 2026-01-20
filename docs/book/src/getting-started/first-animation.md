# Your First Animation

Let's create a simple blinking star animation.

## Step 1: Define the Palette

First, create a file called `blink.pxl` and add a palette:

```json
{"type": "palette", "name": "star", "colors": {"{_}": "#0000", "{y}": "#FFD700", "{w}": "#FFFFFF"}}
```

We have three colors:
- `{_}` - transparent
- `{y}` - yellow (gold)
- `{w}` - white (for the bright frame)

## Step 2: Create Animation Frames

Add two sprite frames - one normal, one bright:

```json
{"type": "sprite", "name": "star_normal", "palette": "star", "grid": [
  "{_}{y}{_}",
  "{y}{y}{y}",
  "{_}{y}{_}"
]}
{"type": "sprite", "name": "star_bright", "palette": "star", "grid": [
  "{_}{w}{_}",
  "{w}{w}{w}",
  "{_}{w}{_}"
]}
```

## Step 3: Define the Animation

Add the animation object that sequences the frames:

```json
{"type": "animation", "name": "blink", "frames": ["star_normal", "star_normal", "star_normal", "star_bright"], "duration": 150}
```

This creates a blink effect:
- Three frames of normal yellow (450ms total)
- One frame of bright white (150ms)
- Then loops

## Complete File

Your `blink.pxl` should look like:

<!-- DEMOS getting-started/first-animation#blink -->
**Blinking Star Animation**

A simple 4-frame blink effect with timing control.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "star", "colors": {"{_}": "#0000", "{y}": "#FFD700", "{w}": "#FFFFFF"}}
{"type": "sprite", "name": "star_normal", "palette": "star", "grid": ["{_}{y}{_}", "{y}{y}{y}", "{_}{y}{_}"]}
{"type": "sprite", "name": "star_bright", "palette": "star", "grid": ["{_}{w}{_}", "{w}{w}{w}", "{_}{w}{_}"]}
{"type": "animation", "name": "blink", "frames": ["star_normal", "star_normal", "star_normal", "star_bright"], "duration": 150}
```

</div>

<div class="demo-container" data-demo="blink">
</div>
<!-- /DEMOS -->

## Step 4: Render as GIF

Export the animation as a GIF:

```bash
pxl render blink.pxl --gif --animation blink -o blink.gif --scale 8
```

This creates `blink.gif` - an 8x scaled animated GIF of your blinking star!

## Step 5: Export as Spritesheet

For game engines, export as a horizontal spritesheet:

```bash
pxl render blink.pxl --spritesheet --animation blink -o blink_sheet.png --scale 4
```

This creates a horizontal strip with all frames side by side.

## Animation Options

The animation object supports these fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Animation identifier |
| `frames` | array | required | Sprite names in sequence |
| `duration` | number | 100 | Milliseconds per frame |
| `loop` | boolean | true | Whether to loop |

## Tips

- **Reuse frames** - Repeat sprite names in the `frames` array for timing control
- **Frame duration** - Lower values = faster animation
- **Scale** - Always scale up for previews; pixel art is small!
- **Validate first** - Run `pxl validate` before rendering to catch issues

## Next Steps

- Learn about [Variants](../format/variant.md) to create frame variations efficiently
- Explore [Compositions](../format/composition.md) to layer animated sprites
- Check the [GIF Export](../exports/gif.md) guide for advanced options
