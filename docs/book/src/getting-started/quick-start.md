# Quick Start

Let's create your first sprite in under 5 minutes.

## Step 1: Create a Pixelsrc File

Create a file called `star.pxl` with this content:

```json5
// star.pxl - A simple 3x3 star
{
  type: "palette",
  name: "star",
  colors: {
    _: "transparent",
    y: "#FFD700",
  },
}

{
  type: "sprite",
  name: "star",
  size: [3, 3],
  palette: "star",
  regions: {
    y: {
      union: [
        { points: [[1, 0]] },           // Top
        { rect: [0, 1, 3, 1] },         // Middle row
        { points: [[1, 2]] },           // Bottom
      ],
    },
  },
}
```

This defines:
- A **palette** named "star" with two colors: transparent (`_`) and yellow (`y`)
- A **sprite** named "star" that uses regions to draw a 3x3 star shape

## Step 2: Render to PNG

Run the render command:

```bash
pxl render star.pxl -o star.png
```

You now have `star.png` - a 3x3 pixel star!

## Step 3: Scale It Up

For a larger preview, use the `--scale` flag:

```bash
pxl render star.pxl -o star_8x.png --scale 8
```

This creates an 8x scaled version (24x24 pixels).

## Step 4: Preview in Terminal

For quick iteration, use the `show` command to preview directly in your terminal:

```bash
pxl show star.pxl
```

This displays the sprite using ANSI true-color in your terminal.

## Step 5: Validate Your File

Check for common mistakes:

```bash
pxl validate star.pxl
```

If everything is correct, you'll see no output (success). If there are issues, you'll get helpful warnings.

## What's Next?

- Learn about [Core Concepts](concepts.md) like palettes, sprites, and regions
- Create [Your First Animation](first-animation.md)
- Explore the [Format Specification](../format/overview.md) for all available features

## Tips

- Use **semantic token names** like `skin`, `outline`, `shadow` instead of generic names
- The `_` token is the conventional name for transparent pixels
- Use shapes (`rect`, `circle`, `ellipse`) for efficiency over individual points
- Higher `z` values draw on top of lower ones
