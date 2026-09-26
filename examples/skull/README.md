# Skull

A 3/4-view skull facing right, drawn in pixelsrc after `reference-skull.jpg`.
It's a 40x40 sprite on a transparent background, and the jaw is a separate
sprite so the skull can talk.

![reference vs render](skull_comparison.png)

![talking](skull_talking.gif)

## Files

| File | What it is |
|------|------------|
| `skull.pxl` | Source: the `bone` palette, the `skull_cranium` and `skull_jaw` sprites, two open-jaw poses (`skull_jaw_ajar`, `skull_jaw_open`), the `skull` / `skull_talk_*` compositions and the `skull_talking` animation |
| `skull.png` | The still at 1x (40x40) |
| `skull_8x.png` | The still at 8x (320x320) |
| `skull_talking.gif` | The talking loop at 8x |
| `reference-skull.jpg` | The target art |
| `skull_comparison.png` | Reference and render side by side, both skulls scaled to the same height |
| `compare.py` | Builds `skull_comparison.png` (needs Pillow) |
| `GAPS.md` | Things the compiler couldn't express, with minimal repros |

## Render

From the repo root, after `cargo build --release`:

```sh
pxl=target/release/pxl
$pxl render examples/skull/skull.pxl -c skull -o examples/skull/skull.png
$pxl render examples/skull/skull.pxl -c skull --scale 8 -o examples/skull/skull_8x.png
$pxl render examples/skull/skull.pxl --gif --animation skull_talking --scale 8 -o examples/skull/skull_talking.gif
python3 examples/skull/compare.py
```

## How it's built

- **Silhouette.** `outline` is a closed polyline (`line` with many points),
  and the cranium's `bone` is `fill: "inside(outline)"`.
- **Shading.** The back-of-skull shadow is `mid: { fill: "inside(outline)",
  except: ["lit"] }`, where `lit` is a transparent mask polygon covering the
  lit side. Other planes are small `path` polygons: the shadowed temple, the
  cheekbone underside and the far side of the face. The dome highlight is a
  `path` too.
- **Detail.** The angular orbits are `path` polygons with a lighter inner `rim`
  polygon. The nasal notch, tooth roots, ramus edge and chin ridge are `points`
  and `line`s. The teeth are two pixels each: a lit column (`tooth`) over the
  beige base. A transparent `gap` region at z 200 punches the see-through hole
  at the corner of the mouth.
- **Jaw.** `skull_jaw` is drawn over the cranium. Where it meets the skull, its
  top edge is a soft `seam`/`gum` line instead of a hard outline. The cranium
  has a dark mouth interior hidden behind the jaw, and that interior shows when
  the jaw opens. The open poses swing the mandible around the hinge: the chin
  drops 1px (ajar) or 2px (open) and the hinge stays put.
- **Order.** Every region has an explicit `z` (see GAPS.md).
