# pixelsrc

Semantic pixel art format and compiler. Native Python bindings powered by Rust via PyO3.

## Installation

```bash
pip install pixelsrc
```

Requires Python 3.9+. Wheels are compiled from Rust source using [maturin](https://www.maturin.rs/).

## Quick start

```python
import pixelsrc

# Define a sprite in JSONL format
pxl = '{"type": "sprite", "name": "dot", "size": [3, 3], "palette": {"_": "#00000000", "x": "#ff0000"}, "regions": {"x": {"points": [[1, 1]], "z": 0}}}'

# Render to PNG
png_bytes = pixelsrc.render_to_png(pxl)
with open("dot.png", "wb") as f:
    f.write(png_bytes)

# Render to RGBA pixels
result = pixelsrc.render_to_rgba(pxl)
print(f"{result.width}x{result.height}, {len(result.pixels)} bytes")
```

## API overview

### Rendering

| Function | Description |
|----------|-------------|
| `render_to_png(pxl)` | Render the first sprite to PNG bytes |
| `render_to_rgba(pxl)` | Render the first sprite to a `RenderResult` with RGBA pixels |

### Parsing and listing

| Function | Description |
|----------|-------------|
| `parse(pxl)` | Parse PXL/JSONL and return a list of dicts |
| `list_sprites(pxl)` | Return sprite names found in the input |
| `list_palettes(pxl)` | Return palette names found in the input |
| `format_pxl(pxl)` | Reformat PXL/JSONL for readability |

### Validation

| Function | Description |
|----------|-------------|
| `validate(pxl)` | Validate a PXL string; returns a list of messages (empty = valid) |
| `validate_file(path)` | Validate a `.pxl` file on disk |

### Color utilities

| Function | Description |
|----------|-------------|
| `parse_color(color_str)` | Parse any CSS color to `#rrggbb` hex |
| `generate_ramp(from_color, to_color, steps)` | Interpolate between two colors |

### PNG import

| Function | Description |
|----------|-------------|
| `import_png(path, ...)` | Import a PNG file into Pixelsrc format |
| `import_png_analyzed(path, ...)` | Import a PNG with full analysis options |

### Stateful registry

For projects with multiple files or shared palettes, use the `Registry` class:

```python
from pixelsrc import Registry

reg = Registry()
reg.load_file("palettes.pxl")
reg.load_file("sprites.pxl")

print(reg.sprites())    # ['hero', 'enemy', ...]
print(reg.palettes())   # ['warm', 'cool', ...]

result = reg.render("hero")
png = reg.render_to_png("hero")
all_pngs = reg.render_all()  # dict[str, bytes]
```

## Result types

### `RenderResult`

Returned by `render_to_rgba()` and `Registry.render()`.

| Property | Type | Description |
|----------|------|-------------|
| `width` | `int` | Width in pixels |
| `height` | `int` | Height in pixels |
| `pixels` | `bytes` | Raw RGBA data (4 bytes per pixel) |
| `warnings` | `list[str]` | Warnings generated during rendering |

### `ImportResult`

Returned by `import_png()` and `import_png_analyzed()`.

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Sprite name |
| `width` | `int` | Width in pixels |
| `height` | `int` | Height in pixels |
| `palette` | `dict[str, str]` | Token-to-color mapping |
| `analysis` | `dict \| None` | Analysis results (if enabled) |

Methods: `to_pxl()` and `to_jsonl()` for serialization.

## License

MIT
