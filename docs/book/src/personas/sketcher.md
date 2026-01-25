# The Sketcher

You want to **quickly visualize ideas**. Pixel-perfect polish can come later—right now you need to see if your concept works.

## Your Workflow

1. Create a `.pxl` file with minimal setup
2. Preview in terminal with `pxl show`
3. Iterate rapidly until the concept feels right
4. Export when ready

## Quick Setup

Create `sketch.pxl`:

```json5
{
  type: "palette",
  name: "sketch",
  colors: {
    _: "transparent",
    x: "#000000",
    o: "#FFFFFF",
  },
}
```

That's it. One palette, two colors. Now sketch:

```json5
{
  type: "sprite",
  name: "idea",
  size: [5, 5],
  palette: "sketch",
  regions: {
    x: {
      stroke: [0, 0, 5, 5],
      z: 0,
    },
    o: {
      union: [
        { rect: [1, 1, 3, 1] },
        { rect: [1, 3, 3, 1] },
        { points: [[1, 2], [3, 2]] },
      ],
      z: 1,
    },
  },
}
```

## Terminal Preview

Skip the PNG export—preview directly:

```bash
pxl show sketch.pxl
```

Your sprite appears in the terminal using ANSI colors. Fast feedback, no file clutter.

## Tips for Rapid Iteration

### Use Short Token Names

When sketching, single-character tokens are faster to type:

```json5
{
  _: "transparent",
  x: "#000",
  o: "#FFF",
  r: "#F00",
  b: "#00F",
}
```

### Keep Sprites Small

Start with 8x8 or 16x16. You can always upscale later:

```bash
pxl render sketch.pxl -o preview.png --scale 4
```

### Multiple Ideas in One File

Keep multiple sprites in a single file:

```json5
{ type: "sprite", name: "idea_v1", size: [8, 8], palette: "sketch", regions: { ... } }
{ type: "sprite", name: "idea_v2", size: [8, 8], palette: "sketch", regions: { ... } }
{ type: "sprite", name: "idea_v3", size: [8, 8], palette: "sketch", regions: { ... } }
```

Show a specific one:

```bash
pxl show sketch.pxl --name idea_v2
```

### Don't Worry About Mistakes

Pixelsrc is lenient by default. Missing tokens render as magenta, invalid shapes get skipped. The goal is momentum—fix issues later.

## When You're Ready for More

Once your sketch is solid:

- Add semantic token names (see [The Sprite Artist](sprite-artist.md))
- Create animation frames (see [The Animator](animator.md))
- Export to PNG: `pxl render sketch.pxl -o sketch.png`

## Example: Character Silhouette

Start with a simple silhouette to nail the proportions:

```json5
{
  type: "palette",
  name: "silhouette",
  colors: {
    _: "transparent",
    s: "#000000",
  },
}

{
  type: "sprite",
  name: "hero",
  size: [7, 7],
  palette: "silhouette",
  regions: {
    s: {
      union: [
        // Head
        { rect: [2, 0, 3, 2] },
        // Body
        { rect: [1, 2, 5, 2] },
        // Arms
        { points: [[0, 4], [6, 4]] },
        // Legs
        { rect: [2, 4, 1, 3] },
        { rect: [4, 4, 1, 3] },
      ],
      z: 0,
    },
  },
}
```

Try changing `s` to `#4169E1` (blue) to see color, or adjust the regions to make the character taller or add more detail.

Once the shape feels right, you can add detail colors, animate it, or hand it off to your sprite artist persona.
