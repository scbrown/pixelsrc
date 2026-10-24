# Stiwi: a hand-authored semantic portrait

![48-pixel portrait of Stiwi](stiwi-hand-48-preview.png)

Authored by **GPT-6.1-Sol running in Codex (model as declared by the
harness)**. This records the selected model; it is not an independent
attestation of the backend. Pixelsrc rendered the artwork.

The model placed every palette colour and geometric region by hand while
looking at a photo reference. There was no image import, quantizer or tracing.
The reference photo is intentionally not included or linked here.

The portrait went through **six revisions with human feedback**: softening the
side hair, curving the moustache, exposing a small philtrum, and closing a gap
in the right forelock. Named palette tokens, semantic roles and relationships,
and layered `rect`, `union`, `points`, `line` and `stroke` regions make those
changes editable rather than requiring a new pixel grid.

| Variant | Editable source | Native render | Nearest-neighbour preview |
| --- | --- | --- | --- |
| Detailed, 48 × 48 | [source](stiwi-hand-48.pxl) | [PNG](stiwi-hand-48.png) | [528 × 528](stiwi-hand-48-preview.png) |
| Compact, 32 × 32 | [source](stiwi-hand-32.pxl) | [PNG](stiwi-hand-32.png) | [512 × 512](stiwi-hand-32-preview.png) |

The 48-pixel portrait is independently authored, not an enlargement of the
32-pixel study.

## A renderer bug found during the revisions

A three-pixel forelock exposed that semantic `line` regions ignored their
`thickness`, including inside a `union`. [PR #25](https://github.com/scbrown/pixelsrc/pull/25)
fixed the renderer and added PNG/mask regression coverage. These examples
require a build containing that fix.

## Validate and render

From this directory, using `pxl` built from the repository:

```sh
pxl validate --strict stiwi-hand-48.pxl
pxl validate --strict stiwi-hand-32.pxl
pxl render stiwi-hand-48.pxl --strict --scale 1 -o stiwi-hand-48.png
pxl render stiwi-hand-48.pxl --strict --scale 11 -o stiwi-hand-48-preview.png
pxl render stiwi-hand-32.pxl --strict --scale 1 -o stiwi-hand-32.png
pxl render stiwi-hand-32.pxl --strict --scale 16 -o stiwi-hand-32-preview.png
```

`cargo test --test stiwi_example` validates both sources strictly and compares
all four newly rendered PNGs with the saved artwork.
