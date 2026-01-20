# Quick Start

Let's create your first sprite in under 5 minutes.

## Step 1: Create a Pixelsrc File

Create a file called `star.pxl` with this content:

```json
{"type": "palette", "name": "star", "colors": {"{_}": "#0000", "{y}": "#FFD700"}}
{"type": "sprite", "name": "star", "palette": "star", "grid": [
  "{_}{y}{_}",
  "{y}{y}{y}",
  "{_}{y}{_}"
]}
```

This defines:
- A **palette** named "star" with two colors: transparent (`{_}`) and yellow (`{y}`)
- A **sprite** named "star" that uses that palette to draw a 3x3 star shape

<!-- DEMOS getting-started/quick-start#star -->
**Your First Sprite**

A simple 3x3 star using semantic color tokens.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "star", "colors": {"{_}": "#0000", "{y}": "#FFD700"}}
{"type": "sprite", "name": "star", "palette": "star", "grid": ["{_}{y}{_}", "{y}{y}{y}", "{_}{y}{_}"]}
```

</div>

<div class="demo-container" data-demo="star">
</div>
<!-- /DEMOS -->

### Try It

Edit the colors below and click "Try it" to see your sprite:

<div class="pixelsrc-demo" data-pixelsrc-demo>
  <textarea id="quickstart-star">{"type": "palette", "name": "star", "colors": {"{_}": "#0000", "{y}": "#FFD700"}}
{"type": "sprite", "name": "star", "palette": "star", "grid": ["{_}{y}{_}", "{y}{y}{y}", "{_}{y}{_}"]}</textarea>
  <button onclick="pixelsrcDemo.renderFromTextarea('quickstart-star', 'quickstart-star-preview')">Try it</button>
  <div class="preview" id="quickstart-star-preview"></div>
</div>

Try changing `{y}` to use `#FF0000` (red) or `#00FF00` (green) to see how palette colors work.

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

- Learn about [Core Concepts](concepts.md) like palettes, sprites, and tokens
- Create [Your First Animation](first-animation.md)
- Explore the [Format Specification](../format/overview.md) for all available features

## Tips

- Use **semantic token names** like `{skin}`, `{outline}`, `{shadow}` instead of generic names
- The `{_}` token is the conventional name for transparent pixels
- Grid rows are defined top-to-bottom
- Each token in a row represents one pixel from left to right
