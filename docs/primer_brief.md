# Pixelsrc Quick Reference

Text-based format for pixel art. Define sprites with structured regions, render to PNG with `pxl render`.

## Object Types

**Palette** - Define colors:
```json5
{
  type: "palette",
  name: "coin",
  colors: {
    _: "transparent",
    gold: "#FFD700",
    shine: "#FFE766",
    shadow: "#B8960B",
  },
}
```

**Sprite** - Structured regions:
```json5
{
  type: "sprite",
  name: "coin",
  size: [8, 8],
  palette: "coin",
  regions: {
    gold: { rect: [1, 1, 6, 6], z: 0 },
    shine: { points: [[2, 1], [1, 2]], z: 1 },
    shadow: { points: [[5, 5], [6, 4]], z: 1 },
  },
}
```

**Animation** - CSS Keyframes:
```json5
{
  type: "animation",
  name: "bounce",
  keyframes: {
    "0%": { sprite: "coin" },
    "50%": { sprite: "coin", transform: "translate(0, -4)" },
  },
  duration: "400ms",
  loop: true,
}
```

## Shape Primitives

- `rect: [x, y, w, h]` - Filled rectangle
- `stroke: [x, y, w, h]` - Rectangle outline (1px)
- `points: [[x, y], ...]` - Individual pixels
- `line: [[x1, y1], [x2, y2], ...]` - Connected line
- `ellipse: [cx, cy, rx, ry]` - Filled ellipse
- `circle: [cx, cy, r]` - Filled circle
- `polygon: [[x, y], ...]` - Filled polygon

## Compound Shapes

```json5
regions: {
  body: {
    union: [
      { rect: [2, 0, 4, 1] },
      { rect: [1, 1, 6, 3] },
    ],
    z: 0,
  },
}
```

Operations: `union`, `subtract`, `intersect`

## Example (8x8 coin)

```json5
// coin.pxl
{
  type: "palette",
  name: "coin",
  colors: {
    _: "transparent",
    gold: "#FFD700",
    shine: "#FFE766",
    shadow: "#B8960B",
  },
}

{
  type: "sprite",
  name: "coin",
  size: [8, 8],
  palette: "coin",
  regions: {
    gold: {
      union: [
        { rect: [2, 0, 4, 1] },
        { rect: [1, 1, 6, 6] },
        { rect: [2, 7, 4, 1] },
      ],
      z: 0,
    },
    shine: { points: [[2, 1], [3, 1], [1, 2], [1, 3]], z: 1 },
    shadow: { points: [[6, 4], [6, 5], [5, 5]], z: 1 },
  },
}
```

## Rules

1. **Palette before sprite** - Define colors first
2. **Semantic tokens** - `skin`, `hair`, not `a`, `b`
3. **Use `_` for transparency**
4. **Z-order matters** - Higher z draws on top
5. **Shapes, not pixels** - Use rect, ellipse, polygon when possible

## Commands

```bash
pxl render file.pxl             # Render to PNG
pxl render file.pxl --scale 4   # 4x upscale
pxl render file.pxl --gif       # Animated GIF
pxl validate file.pxl           # Check for errors
pxl show file.pxl --sprite x    # Terminal display
pxl import image.png -o out.pxl # Import PNG
pxl fmt file.pxl                # Format file
```
