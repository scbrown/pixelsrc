# Gaps found while drawing and animating the skull

These are places where the compiler couldn't express what the art needed, or
where it didn't behave the way the docs describe. Each one has a minimal repro
and a note on how `skull.pxl` works around it. Checked against `pxl` built from
this branch.

## 1. Region paint order is random unless every region has `z`

```json5
{ type: "palette", name: "p", colors: { _: "transparent", a: "#ff0000", b: "#0000ff", c: "#00ff00", d: "#ffff00" } }
{ type: "sprite", name: "order", size: [4, 4], palette: "p", regions: {
    a: { rect: [0, 0, 4, 4] },
    b: { rect: [1, 1, 3, 3] },
    c: { rect: [2, 2, 2, 2] },
    d: { points: [[3, 3]] },
} }
```

The docs (format/regions.md, "Z-Order") say the default is definition order.
`Sprite.regions` is a `HashMap`, though, and `render_structured` stable-sorts
by z, so equal-z regions paint in hash order. That order changes from process
to process: rendering the repro 6 times gave 3 different PNGs. **Workaround:**
every region in `skull.pxl` has an explicit, unique `z`. The fix is probably an
`IndexMap` for `regions`, which would also make `fill`/`except` dependency
order match the source.

## 2. `auto-outline` (and several other region modifiers) does nothing

```json5
{ type: "palette", name: "p", colors: { _: "transparent", body: "#e6dccd", outline: "#000000" } }
{ type: "sprite", name: "blob", size: [8, 8], palette: "p", regions: {
    body: { rect: [2, 2, 4, 4], z: 0 },
    outline: { "auto-outline": "body", thickness: 1, z: 1 },
} }
```

`pxl mask --count` reports 16 `body` pixels and 0 `outline` pixels.
`RegionDef` parses `auto-outline`, `repeat`/`spacing`/`offset-alternate`,
`transform`, `jitter`, `round` and the `x`/`y` range limits, but
`structured.rs::rasterize_region` never reads any of them, so they are dropped
without a warning. **Workaround:** the outline is drawn explicitly as a closed
polyline. The teeth are listed one by one instead of using `repeat`.

## 3. `fill: "inside(X)"` can flood the outside of a concave shape

```json5
{ type: "palette", name: "p", colors: { _: "transparent", bone: "#e6dccd", outline: "#000000" } }
{ type: "sprite", name: "cup", size: [10, 10], palette: "p", regions: {
    outline: { z: 1, line: [[0, 0], [3, 0], [3, 6], [6, 6], [6, 0], [9, 0], [9, 9], [0, 9], [0, 0]] },
    bone: { z: 0, fill: "inside(outline)" },
} }
```

The seed is the centre of the boundary's bounding box, or the nearest free
pixel to it. For the "U" that pixel is in the notch, so `bone` fills the notch
and leaks out of the top of the canvas, and the inside of the U stays empty.
The docs offer `seed: [x, y]`, but `RegionDef.seed` is the `u32` jitter seed
and `flood_fill` is always called with `None`. A related problem: a `fill`
nested inside `union`/`intersect` isn't deferred the way a top-level `fill` is,
so it can run before its boundary exists and fill the whole canvas.
**Workaround:** the cranium keeps its `fill` (its centre is inside). The
mandible is concave, so its interior is a `path` polygon with the same vertices
as its outline. Every `fill` is top-level.

## 4. Compositions can't offset a layer

```json5
{ type: "palette", name: "p", colors: { _: "transparent", x: "#ff0000" } }
{ type: "sprite", name: "dot", size: [4, 4], palette: "p", regions: { x: { points: [[0, 0]] } } }
{ type: "sprite", name: "dot_moved", palette: "p", source: "dot", transform: ["shift:2,2"] }
{ type: "composition", name: "scene", size: [4, 4], cell_size: [4, 4],
  sprites: { D: "dot_moved" }, layers: [{ map: ["D"] }] }
```

`pxl render -s dot_moved` puts the dot at (2,2), but `pxl render -c scene` puts
it at (0,0). The composition path (`render_composition_to_image`) renders the
resolved regions and skips the derived sprite's image `transform`. The
`{ sprite, x, y }` layer form in format/composition.md doesn't help either: it
renders an empty canvas. The only offset left is `cell_size` with a
mismatched map, and that warns on every render. **Workaround:** the talking
poses `skull_jaw_ajar` and `skull_jaw_open` are written out in full as their
own sprites, not as `shift`-ed copies of `skull_jaw`.

GIF frames have the same problem. With `frames: ["dot", "dot_moved"]`,
`pxl render --gif` draws the second frame empty and warns that `dot_moved`
"uses deprecated grid format". **Workaround in `skull_anim.pxl`:** the 1px
head float needs the whole skull at two heights, so every sprite exists twice
(`*_rest` and `*_up`) with its coordinates written out.

## 5. Derived sprites still need `palette`

The docs example `{ type: "sprite", name: "hero_outlined", source: "hero",
transform: [...] }` fails with `missing field 'palette'`. In `skull.pxl`,
the composition after that line then wasn't found either. **Workaround:** not needed after gap 4, but any
`source:` sprite has to repeat `palette`.

## 6. No per-frame duration in a frame-list animation

A `frames: [...]` animation has one `duration` that applies to every frame.
`gif.rs::render_gif` takes a single `duration_ms`, and there's no per-frame
timing field (`frame_metadata` only carries hitboxes). Speech timing and idle
holds need uneven frame lengths. **Workaround:** a hold is the same
composition repeated (`skull_talking` repeats `skull` for its pauses). Nothing
is lost in the GIF, but the frame lists get long.

## 7. A palette `@include` breaks when a sprite is used directly as a GIF frame

```json5
// base.pxl
{ type: "palette", name: "p", colors: { _: "transparent", x: "#ff0000" } }
// anim.pxl
{ type: "sprite", name: "a", size: [4, 4], palette: "@include:base.pxl", regions: { x: { points: [[0, 0]] } } }
{ type: "animation", name: "an", frames: ["a", "a"], duration: 100 }
```

The first frame is right, but the second renders magenta with `Circular include detected: .../base.pxl` and
`Unknown token x`. The include-visited set seems to be shared across frames,
so the second lookup of the same file is treated as a cycle. The same include
works through a composition. **Workaround:** `skull_anim.pxl` repeats the
`bone` palette instead of including it from `skull.pxl`.
