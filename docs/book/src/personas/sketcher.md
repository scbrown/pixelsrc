# The Sketcher

You want to **quickly visualize ideas**. Pixel-perfect polish can come later—right now you need to see if your concept works.

## Your Workflow

1. Create a `.pxl` file with minimal setup
2. Preview in terminal with `pxl show`
3. Iterate rapidly until the concept feels right
4. Export when ready

## Quick Setup

Create `sketch.pxl`:

```json
{"type": "palette", "name": "sketch", "colors": {"{_}": "#0000", "{x}": "#000000", "{o}": "#FFFFFF"}}
```

That's it. One line, two colors. Now sketch:

```json
{"type": "sprite", "name": "idea", "palette": "sketch", "grid": [
  "{_}{x}{x}{x}{_}",
  "{x}{o}{o}{o}{x}",
  "{x}{o}{x}{o}{x}",
  "{x}{o}{o}{o}{x}",
  "{_}{x}{x}{x}{_}"
]}
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

```json
{"{_}": "#0000", "{x}": "#000", "{o}": "#FFF", "{r}": "#F00", "{b}": "#00F"}
```

### Keep Sprites Small

Start with 8x8 or 16x16. You can always upscale later:

```bash
pxl render sketch.pxl -o preview.png --scale 4
```

### Multiple Ideas in One File

JSONL lets you keep multiple sprites in a single file:

```json
{"type": "sprite", "name": "idea_v1", "palette": "sketch", "grid": ["..."]}
{"type": "sprite", "name": "idea_v2", "palette": "sketch", "grid": ["..."]}
{"type": "sprite", "name": "idea_v3", "palette": "sketch", "grid": ["..."]}
```

Show a specific one:

```bash
pxl show sketch.pxl --name idea_v2
```

### Don't Worry About Mistakes

Pixelsrc is lenient by default. Missing tokens render as magenta, row mismatches get padded. The goal is momentum—fix issues later.

## When You're Ready for More

Once your sketch is solid:

- Add semantic token names (see [The Sprite Artist](sprite-artist.md))
- Create animation frames (see [The Animator](animator.md))
- Export to PNG: `pxl render sketch.pxl -o sketch.png`

## Example: Character Silhouette

Start with a simple silhouette to nail the proportions:

```json
{"type": "palette", "name": "silhouette", "colors": {"{_}": "#0000", "{s}": "#000000"}}
{"type": "sprite", "name": "hero", "palette": "silhouette", "grid": [
  "{_}{_}{s}{s}{s}{_}{_}",
  "{_}{s}{s}{s}{s}{s}{_}",
  "{_}{_}{s}{s}{s}{_}{_}",
  "{_}{s}{s}{s}{s}{s}{_}",
  "{s}{_}{s}{s}{s}{_}{s}",
  "{_}{_}{s}{_}{s}{_}{_}",
  "{_}{_}{s}{_}{s}{_}{_}"
]}
```

### Try It

Experiment with the silhouette shape—adjust the pose, add arms, or change proportions:

<div class="pixelsrc-demo" data-pixelsrc-demo>
  <textarea id="sketcher-demo">{"type": "palette", "name": "silhouette", "colors": {"{_}": "#0000", "{s}": "#000000"}}
{"type": "sprite", "name": "hero", "palette": "silhouette", "grid": ["{_}{_}{s}{s}{s}{_}{_}", "{_}{s}{s}{s}{s}{s}{_}", "{_}{_}{s}{s}{s}{_}{_}", "{_}{s}{s}{s}{s}{s}{_}", "{s}{_}{s}{s}{s}{_}{s}", "{_}{_}{s}{_}{s}{_}{_}", "{_}{_}{s}{_}{s}{_}{_}"]}</textarea>
  <button onclick="pixelsrcDemo.renderFromTextarea('sketcher-demo', 'sketcher-demo-preview')">Try it</button>
  <div class="preview" id="sketcher-demo-preview"></div>
</div>

Try changing `{s}` to `#4169E1` (blue) to see color, or add more rows to make the character taller.

Once the shape feels right, you can add detail colors, animate it, or hand it off to your sprite artist persona.
