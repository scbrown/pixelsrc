# Core Concepts

Understanding these core concepts will help you work effectively with Pixelsrc.

## Objects and Types

Pixelsrc files contain **objects**, one per line. Each object has a `type` field:

| Type | Purpose | Required Fields |
|------|---------|-----------------|
| `palette` | Define named colors | `name`, `colors` |
| `sprite` | Pixel grid | `name`, `palette`, `grid` |
| `animation` | Frame sequence | `name`, `frames` |
| `composition` | Layer sprites | `name`, `size`, `layers` |
| `variant` | Modify existing sprite | `name`, `base`, changes |

## Palettes

A **palette** defines colors with semantic names:

<!-- DEMOS getting-started/concepts#palette -->
**Semantic Color Palette**

Palette with meaningful token names for character colors.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{hair}": "#8B4513", "{outline}": "#000000"}}
```

</div>

<div class="demo-container" data-demo="palette">
</div>
<!-- /DEMOS -->

Key points:
- Token names are wrapped in curly braces: `{name}`
- Colors use hex format: `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA`
- `{_}` is the conventional token for transparency
- Palettes must be defined before sprites that reference them

## Tokens

**Tokens** are the heart of Pixelsrc. They're multi-character identifiers that represent colors:

```
{_}         → transparent (convention)
{skin}      → semantic name for skin color
{outline}   → semantic name for outline
{dark_hair} → underscores OK for multi-word names
```

Benefits of semantic tokens:
- **Readable**: `{skin}{skin}{hair}` is clearer than `#FFCC99#FFCC99#8B4513`
- **Maintainable**: Change a color in one place (the palette), update everywhere
- **AI-friendly**: LLMs can reason about `{shadow}` more reliably than hex values

## Sprites

A **sprite** is a pixel grid defined using tokens:

<!-- DEMOS getting-started/concepts#sprite -->
**Basic Sprite Grid**

A simple cross pattern showing how tokens map to pixels.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "colors", "colors": {"{_}": "#00000000", "{r}": "#FF0000"}}
{"type": "sprite", "name": "dot", "palette": "colors", "grid": ["{_}{r}{_}", "{r}{r}{r}", "{_}{r}{_}"]}
```

</div>

<div class="demo-container" data-demo="sprite">
</div>
<!-- /DEMOS -->

Key points:
- `grid` is an array of strings, one per row (top to bottom)
- Each token in a row represents one pixel (left to right)
- `size` is optional - inferred from the grid
- `palette` can be a name (referencing a defined palette) or an inline colors object

## Inline Palettes

For simple sprites, define colors inline:

```json
{"type": "sprite", "name": "dot", "palette": {"{r}": "#FF0000", "{_}": "#0000"}, "grid": [
  "{_}{r}{_}",
  "{r}{r}{r}",
  "{_}{r}{_}"
]}
```

## Animations

An **animation** sequences multiple sprites:

```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

Key points:
- `frames`: Array of sprite names in order
- `duration`: Milliseconds per frame (default: 100)
- `loop`: Whether to loop (default: true)

## Compositions

A **composition** layers multiple sprites:

<!-- DEMOS getting-started/concepts#composition -->
**Layered Scene**

Combining multiple sprites into a single scene with positioning.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "scene", "colors": {"{_}": "#00000000", "{bg}": "#87CEEB", "{g}": "#228B22", "{r}": "#FF0000"}}
{"type": "sprite", "name": "background", "palette": "scene", "grid": ["{bg}{bg}{bg}{bg}", "{bg}{bg}{bg}{bg}", "{g}{g}{g}{g}", "{g}{g}{g}{g}"]}
{"type": "sprite", "name": "hero", "palette": "scene", "grid": ["{r}{r}", "{r}{r}"]}
{"type": "composition", "name": "scene", "size": [4, 4], "layers": [{"sprite": "background", "x": 0, "y": 0}, {"sprite": "hero", "x": 1, "y": 1}]}
```

</div>

<div class="demo-container" data-demo="composition">
</div>
<!-- /DEMOS -->

Layers are rendered bottom-to-top (first layer is background).

## File Format

Pixelsrc uses **JSONL** (JSON Lines) format:
- One JSON object per line
- No commas between objects
- Files typically use `.pxl` or `.jsonl` extension

This streaming format means:
- Each line is self-contained and valid JSON
- AI can generate line-by-line
- Easy to parse incrementally

## Lenient Mode

By default, Pixelsrc is **lenient**:
- Missing tokens render as magenta (visible but not breaking)
- Row length mismatches are padded/truncated with warnings
- Small mistakes don't halt rendering

Use `--strict` mode for validation in CI/CD pipelines.
