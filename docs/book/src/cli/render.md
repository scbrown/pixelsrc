# render

Render sprites from a Pixelsrc file to PNG, GIF, or atlas formats.

## Usage

```
pxl render [OPTIONS] <INPUT>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input file containing palette and sprite definitions (`.pxl` or `.jsonl`) |

## Options

| Option | Description |
|--------|-------------|
| `-o, --output <OUTPUT>` | Output file or directory (see below) |
| `-s, --sprite <SPRITE>` | Only render the sprite with this name |
| `-c, --composition <COMPOSITION>` | Only render the composition with this name |
| `--scale <SCALE>` | Scale output by integer factor (1-16, default: 1) |
| `--strict` | Treat warnings as errors |
| `--gif` | Output as animated GIF (requires animation in input) |
| `--spritesheet` | Output as spritesheet (horizontal strip of all frames) |
| `--emoji` | Output as emoji art to terminal (for quick preview) |
| `--animation <ANIMATION>` | Select a specific animation by name |
| `--format <FORMAT>` | Atlas format (see below) |
| `--max-size <MAX_SIZE>` | Maximum atlas size (e.g., "512x512") |
| `--padding <PADDING>` | Padding between sprites in atlas (pixels, default: 0) |
| `--power-of-two` | Force power-of-two dimensions for atlas |
| `--nine-slice <WxH>` | Render nine-slice sprite to target size (e.g., "64x32") |

## Output Naming

If `--output` is omitted:
- Single sprite: `{input}_{sprite}.png`
- Multiple sprites: `{input}_{sprite}.png` for each

If `--output` is a file:
- Single sprite: uses the exact filename
- Multiple sprites: `{output}_{sprite}.png` for each

If `--output` ends with `/`:
- Each sprite is written as `{dir}/{sprite}.png`

## Atlas Formats

The `--format` option supports:

| Format | Description |
|--------|-------------|
| `atlas` | Generic JSON atlas |
| `atlas-aseprite` | Aseprite-compatible JSON |
| `atlas-godot` | Godot engine format |
| `atlas-unity` | Unity sprite atlas |
| `atlas-libgdx` | LibGDX texture atlas |

## Examples

<!-- DEMOS cli/render#basic -->
**Basic Render Example**

Render a simple sprite to PNG output.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "simple", "colors": {"{_}": "#00000000", "{b}": "#4a90d9", "{w}": "#ffffff"}}
{"type": "sprite", "name": "icon", "palette": "simple", "grid": ["{_}{b}{_}", "{b}{w}{b}", "{_}{b}{_}"]}
```

</div>

<div class="demo-container" data-demo="basic">
</div>

```bash
pxl render icon.pxl -o icon.png
```
<!-- /DEMOS -->

### Basic rendering

```bash
# Render all sprites to PNG files
pxl render character.pxl

# Render to a specific output file
pxl render character.pxl -o hero.png

# Render a specific sprite
pxl render sprites.pxl --sprite hero -o hero.png
```

### Scaling

<!-- DEMOS cli/render#scaled -->
**Scaled Render Example**

Render sprites at larger sizes with integer scaling.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "pixel", "colors": {"{_}": "#00000000", "{p}": "#e43b44", "{d}": "#a82b3a"}}
{"type": "sprite", "name": "heart", "palette": "pixel", "grid": ["{_}{p}{_}{p}{_}", "{p}{d}{p}{d}{p}", "{p}{d}{d}{d}{p}", "{_}{p}{d}{p}{_}", "{_}{_}{p}{_}{_}"]}
```

</div>

<div class="demo-container" data-demo="scaled">
</div>

```bash
pxl render heart.pxl --scale 4 -o heart_4x.png
```
<!-- /DEMOS -->

```bash
# Scale up 4x (16x16 becomes 64x64)
pxl render character.pxl --scale 4

# Maximum scale factor is 16
pxl render character.pxl --scale 16
```

### Animated output

```bash
# Render animation as GIF
pxl render animation.pxl --gif -o walk.gif

# Render specific animation
pxl render character.pxl --animation walk --gif -o walk.gif

# Render as spritesheet (horizontal strip)
pxl render animation.pxl --spritesheet -o walk-strip.png
```

### Quick preview

```bash
# Preview as emoji in terminal
pxl render character.pxl --emoji
```

### Atlas generation

```bash
# Create a texture atlas
pxl render sprites.pxl --format atlas -o sprites-atlas.png

# Godot-compatible atlas with padding
pxl render sprites.pxl --format atlas-godot --padding 2 -o atlas.png

# Force power-of-two dimensions
pxl render sprites.pxl --format atlas --power-of-two -o atlas.png

# Limit maximum atlas size
pxl render sprites.pxl --format atlas --max-size 512x512 -o atlas.png
```

### Strict mode

```bash
# Fail on any warnings
pxl render character.pxl --strict
```

### Nine-slice rendering

Nine-slice (9-patch) sprites are scalable UI elements where corners stay fixed while
edges and center stretch. The sprite must have a `nine_slice` attribute defined.

```bash
# Render a button sprite stretched to 64x32 pixels
pxl render button.pxl --nine-slice 64x32 -o button_wide.png

# Render at 128x48 for a dialog box
pxl render panel.pxl --nine-slice 128x48 -o panel_large.png
```

The `nine_slice` attribute on the sprite defines the border widths:
```json
{"nine_slice": {"left": 4, "right": 4, "top": 4, "bottom": 4}}
```

See the [format specification](../../spec/format.md) for complete details on the nine-slice format.

## See Also

- [import](import.md) - Convert PNG images to Pixelsrc format
- [show](show.md) - Preview sprites in terminal
- [build](build.md) - Build multiple assets according to project config
