---
name: pixelsrc
description: >
  Draw pixel art as pixelsrc source (.pxl) and render it with the `pxl` CLI. Use this skill when asked for pixel art,
  a sprite, an icon, a portrait, a tile map, a scene or blog/README artwork "in pixelsrc" or "with pxl", when editing
  an existing .pxl file, or when a pxl render or validate fails. Covers the authoring loop, compositions and scenes,
  recolouring, isometric views and the mistakes that cost the most time.
---

# Pixelsrc Skill — pixel art as source

Pixelsrc art is text: palettes of named colours, sprites made of named regions (shapes, not pixel grids), and
compositions that place sprites on a canvas. The source is the deliverable. Someone will read it, change a palette or
move a sprite, and re-render.

## Start here

```bash
pxl prime            # the full format guide; read it before writing a first sprite
pxl prime --brief    # the short version
pxl --version        # regions need 0.3.1 or later
```

## The loop

1. Write or edit the `.pxl` file.
2. `pxl validate file.pxl` — fix every error; read the warnings.
3. `pxl render file.pxl -s <sprite> --scale 4 -o out.png` (or `-c <composition>`).
4. Look at the PNG. Do not judge a drawing from its source.
5. Repeat. Small edits and frequent renders beat one large write.

## Rules that matter

- **Author shapes with meaning.** Regions are named for the part they draw (`wall`, `thatch`, `sleeve`, `shadow`),
  built from `rect`, `polygon`, `line`, `ellipse`, `points` and `union`. Do not draw the picture in another tool and
  export pixel runs into a `.pxl`; that produces source nobody can edit.
- **Name tokens by part or material, not by colour.** Then a second palette recolours the sprite without touching
  its shapes.
- **Order regions with `z`.** Higher `z` draws over lower.
- **Palettes and sprites come before whatever uses them** in the file or stream.
- **Compose, don't repeat.** One sprite per thing; place it many times in a composition.

## Compositions

A composition needs a `sprites` key map and `map` layers. There is no `{ sprite, x, y }` layer.

```json5
{
  type: "composition", name: "scene", size: [32, 32], cell_size: [8, 8],
  sprites: { ".": null, S: "sky", G: "grass", H: "hero" },
  layers: [
    { name: "sky", map: ["S...", "....", "....", "...."] },
    { name: "ground", map: ["....", "....", "....", "GGGG"] },
    { name: "things", map: ["....", "....", ".H..", "...."] },
  ],
}
```

- A sprite is drawn from the top-left of the cell holding its key, at full size even if larger than the cell.
- A full-canvas sprite (ground, sky) goes in the top-left cell of its own layer.
- `cell_size` sets how finely things can be placed. Keep sprite positions on multiples of it.
- Layers draw first to last: order them back to front.
- `exceeds cell size` warnings are expected for scenes.

## Scenes

- Give each sprite an anchor (the pixel that stands on the ground) and place it at `anchor position - anchor offset`.
- To put something between two parts of another object, split that object into a far sprite and a near sprite.
- Shadows and light are regions with alpha colours (`#00000080`), or a layer with `blend: "multiply"`.
- Large scenes can be split across files and joined with `cat` before rendering.
- Recolour with a `variant` (override some tokens) or with `{ type: "sprite", name, palette, source }` (swap the
  whole palette, same token names). Add `transform: ["mirror-h"]` to flip.

## Isometric views

Pick one projection and compute every vertex from it:

```
screen_x = origin_x + (tile_w / 2) * (gx - gy)
screen_y = origin_y + (tile_h / 2) * (gx + gy) - height      // tile_w = 2 * tile_h
```

Boxes are a top and two faces (one lit, one shaded, the same on every object). Walls are parallelograms; openings and
stone courses stay in the wall's plane. Roads run along a tile axis at slope 1/2. Nearer things (larger `gx + gy`)
go in later layers.

## When it goes wrong

| Symptom | Cause |
|---------|-------|
| `missing field sprites` | Composition written with `{ sprite, x, y }` layers. Use `sprites` + `map`. |
| `No composition named ...` on render | Some composition in the file failed to parse. Run `pxl validate`. |
| A cell renders empty, `Sprite or variant ... not found` | The key names a composition or a sprite that does not exist. |
| `Palette tokens never used` on a palette used only through `source:` | False alarm; the check does not follow `source`. |
| `pixel(s) outside the canvas` | A shape's coordinates pass the sprite's `size`; they are clipped. |
| Shapes cover each other wrongly | Missing or equal `z` values. |
