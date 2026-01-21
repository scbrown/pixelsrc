# Pixelsrc Primer

A concise guide for AI systems generating pixelsrc content.

## What is Pixelsrc?

Pixelsrc is a GenAI-native pixel art format. It's text-based JSONL that you generate, then `pxl render` converts to PNG/GIF.

**Why Pixelsrc?**
- No binary data - pure text generation
- Semantic tokens (`{skin}`, `{hair}`) not hex coordinates
- One line per object - easy to iterate
- Lenient by default - small mistakes get corrected

## Format Quick Reference

### Object Types

| Type | Purpose | Required Fields |
|------|---------|-----------------|
| `palette` | Define named colors | `name`, `colors` |
| `sprite` | Pixel grid | `name`, `palette`, `grid` |
| `animation` | Frame sequence | `name`, `frames` |
| `composition` | Layer sprites | `name`, `size`, `layers` |

### Palette

```json
{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{hair}": "#8B4513"}}
```

- Token format: `{name}` - always use curly braces
- Color format: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, or CSS colors (see [colors.md](colors.md))
- `{_}` is the conventional transparent token

### Palette with CSS Variables

```json
{"type": "palette", "name": "themed", "colors": {
  "--primary": "#4169E1",
  "--accent": "#FFD700",
  "{_}": "transparent",
  "{main}": "var(--primary)",
  "{highlight}": "var(--accent)",
  "{shadow}": "color-mix(in oklch, var(--primary) 70%, black)",
  "{glow}": "color-mix(in oklch, var(--primary) 60%, white)"
}}
```

- **CSS Variables**: Define with `--name`, reference with `var(--name)` or `var(--name, fallback)`
- **color-mix()**: Generate shadows/highlights automatically: `color-mix(in oklch, color 70%, black)`
- Common patterns:
  - Shadow: `color-mix(in oklch, var(--color) 70%, black)`
  - Highlight: `color-mix(in oklch, var(--color) 60%, white)`
  - Muted: `color-mix(in srgb, color1 50%, color2)`

### Sprite

```json
{"type": "sprite", "name": "hero", "size": [16, 16], "palette": "hero", "grid": ["{_}{_}{hair}{hair}...", "..."]}
```

- `size`: optional, inferred from grid if omitted
- `palette`: name of previously defined palette, or inline colors object
- `grid`: array of strings, one per row, top to bottom

### Sprite Transforms (Derived Sprites)

Create derived sprites from a source with pixel-level transformations:

```json
{"type": "sprite", "name": "hero_left", "source": "hero_right", "transform": ["mirror-h"]}
{"type": "sprite", "name": "hero_down", "source": "hero_right", "transform": ["rotate:90"]}
{"type": "sprite", "name": "hero_big", "source": "hero", "transform": ["scale:2,2"]}
{"type": "sprite", "name": "hero_outlined", "source": "hero", "transform": [{"op": "sel-out", "fallback": "{outline}"}]}
```

**Transform operations** (string or object syntax):

| Operation | String Syntax | Description |
|-----------|--------------|-------------|
| Mirror H | `"mirror-h"` | Flip horizontally |
| Mirror V | `"mirror-v"` | Flip vertically |
| Rotate | `"rotate:90"` | Rotate 90°, 180°, or 270° |
| Scale | `"scale:2,2"` | Scale by X,Y factors |
| Sel-out | `{"op": "sel-out"}` | Auto-outline based on fill colors |
| Dither | `{"op": "dither", "pattern": "checker", "tokens": ["{dark}", "{light}"]}` | Apply dither pattern |
| Shadow | `"shadow:1,1:{shadow}"` | Add drop shadow |

**Chaining transforms:**
```json
{"type": "sprite", "name": "hero_processed", "source": "hero", "transform": ["mirror-h", "rotate:90"]}
```

Transforms apply in array order.

### Animation (Frame Array)

```json
{"type": "animation", "name": "walk", "frames": ["frame1", "frame2", "frame3"], "duration": 100, "loop": true}
```

- `frames`: array of sprite names
- `duration`: milliseconds per frame (default: 100)
- `loop`: whether to loop (default: true)

### Animation (CSS Keyframes) - Recommended

```json
{"type": "animation", "name": "pulse", "keyframes": {
  "0%": {"sprite": "star", "opacity": 1.0},
  "50%": {"sprite": "star", "opacity": 0.5, "transform": "scale(1.2)"},
  "100%": {"sprite": "star", "opacity": 1.0}
}, "duration": "1s", "timing_function": "ease-in-out", "loop": true}
```

- `keyframes`: map of percentage keys to keyframe objects
- `duration`: CSS time string (`"500ms"`, `"1s"`) or milliseconds
- `timing_function`: CSS easing (`linear`, `ease`, `ease-in-out`, `steps(4)`)
- Keyframe fields: `sprite`, `opacity`, `offset`, `transform`

### CSS Transforms in Keyframes

Use CSS transform strings within keyframe animations:

```json
"transform": "translate(0, -5)"        // Move up 5 pixels
"transform": "rotate(90deg)"           // Rotate 90 degrees clockwise
"transform": "scale(2)"                // Double size
"transform": "scale(1.5, 1)"           // Scale width only
"transform": "flip(x)"                 // Horizontal mirror
"transform": "flip(y)"                 // Vertical mirror
"transform": "translate(5, 0) rotate(45deg) scale(1.2)"  // Combined
```

**Pixel art tips**: Use 90° rotation increments and integer scale factors for crisp results.

### Composition

```json
{"type": "composition", "name": "scene", "size": [32, 32], "layers": [{"sprite": "bg", "x": 0, "y": 0}, {"sprite": "hero", "x": 8, "y": 8}]}
```

- `layers`: array of sprite placements
- Each layer: `{"sprite": "name", "x": N, "y": N}`

## Token Syntax

Tokens are multi-character identifiers wrapped in curly braces:

```
{_}         -> transparent (convention)
{skin}      -> semantic name for skin color
{outline}   -> semantic name for outline color
{dark_hair} -> underscores OK for multi-word
```

**In grid strings**, tokens are concatenated:

```
"{_}{_}{skin}{skin}{hair}{hair}{_}{_}"
```

This is one row with 8 tokens (8 pixels wide).

## Complete Example

A simple 8x8 coin sprite using CSS variables and color-mix:

```jsonl
{"type": "palette", "name": "coin", "colors": {
  "--gold": "#FFD700",
  "{_}": "transparent",
  "{gold}": "var(--gold)",
  "{shine}": "color-mix(in oklch, var(--gold) 60%, white)",
  "{shadow}": "color-mix(in oklch, var(--gold) 70%, black)",
  "{dark}": "color-mix(in oklch, var(--gold) 50%, black)"
}}
{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": [
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}",
  "{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{shadow}{shadow}{gold}",
  "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}",
  "{_}{_}{dark}{dark}{dark}{dark}{_}{_}"
]}
{"type": "animation", "name": "coin_spin", "keyframes": {
  "0%": {"sprite": "coin", "transform": "scale(1, 1)"},
  "25%": {"sprite": "coin", "transform": "scale(0.3, 1)"},
  "50%": {"sprite": "coin", "transform": "scale(1, 1) flip(x)"},
  "75%": {"sprite": "coin", "transform": "scale(0.3, 1)"},
  "100%": {"sprite": "coin", "transform": "scale(1, 1)"}
}, "duration": "600ms", "timing_function": "ease-in-out", "loop": true}
```

**Key observations:**
- CSS variable `--gold` defines base color
- `color-mix()` derives `{shine}`, `{shadow}`, `{dark}` automatically
- Semantic token names: `{gold}`, `{shine}`, `{shadow}`
- CSS keyframes animation with transform for spin effect
- Each row has exactly 8 tokens (matches width)
- Transparent `{_}` for background

## Best Practices

### DO

1. **Use semantic token names**
   - Good: `{skin}`, `{outline}`, `{hair_highlight}`
   - Bad: `{c1}`, `{a}`, `{color_1}`

2. **Keep sprites small**
   - 8x8, 16x16, 32x32 are common
   - Larger than 64x64 is unusual for pixel art

3. **Define palette before sprite**
   - Palettes must appear before sprites that reference them
   - Exception: inline palettes in sprite definition

4. **Use `{_}` for transparency**
   - Convention throughout pixelsrc ecosystem
   - Map to `#00000000` (fully transparent)

5. **Maintain row consistency**
   - Every row should have same token count
   - Token count should match declared width

6. **Use outlines for definition**
   - Dark outline around character silhouette
   - Makes sprites readable at small sizes

7. **Use CSS variables for theming**
   - Define base colors as `--name` variables
   - Reference with `var(--name)` in tokens
   - Enables easy palette swapping

8. **Use color-mix() for derived colors**
   - Shadows: `color-mix(in oklch, base 70%, black)`
   - Highlights: `color-mix(in oklch, base 60%, white)`
   - Avoids manually calculating shade values

9. **Use CSS keyframes for animations**
   - Percentage-based timing is intuitive
   - Supports opacity, transforms, offsets
   - CSS timing functions for smooth easing

### DON'T

1. **Don't use coordinates**
   - Pixel positions are implicit from grid
   - Token order = pixel order (left to right, top to bottom)

2. **Don't use single-character tokens**
   - Bad: `{a}`, `{b}`, `{1}`
   - Semantic names are more maintainable

3. **Don't generate huge sprites**
   - Keep under 64x64 for typical pixel art
   - Large sprites lose the pixel art aesthetic

4. **Don't forget row consistency**
   - If width is 16, every row needs 16 tokens
   - Mismatch causes padding/truncation

5. **Don't use coordinates in grid**
   - Wrong: thinking in (x, y) terms
   - Right: thinking in rows of tokens

## Common Mistakes

### Row Length Mismatch

**Wrong:**
```json
{"grid": ["{_}{_}{_}", "{_}{_}"]}
```
Row 1 has 3 tokens, row 2 has 2. This will cause a warning.

**Right:**
```json
{"grid": ["{_}{_}{_}", "{_}{_}{_}"]}
```

### Undefined Token

**Wrong:**
```json
{"type": "palette", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "grid": ["{a}{b}"]}
```
Token `{b}` is used but never defined. Will render as magenta.

**Right:**
```json
{"type": "palette", "colors": {"{a}": "#FF0000", "{b}": "#00FF00"}}
{"type": "sprite", "grid": ["{a}{b}"]}
```

### Palette Reference Before Definition

**Wrong:**
```json
{"type": "sprite", "palette": "hero", "grid": ["..."]}
{"type": "palette", "name": "hero", "colors": {...}}
```
Sprite references palette that hasn't been defined yet.

**Right:**
```json
{"type": "palette", "name": "hero", "colors": {...}}
{"type": "sprite", "palette": "hero", "grid": ["..."]}
```

### Invalid Color Format

**Wrong:**
```json
{"colors": {"{bad}": "#GGG", "{worse}": "notacolor"}}
```
Invalid hex characters and unknown color names render as magenta in lenient mode.

**Right:**
```json
{"colors": {"{red}": "#FF0000", "{blue}": "blue", "{skin}": "peachpuff"}}
```
Hex colors and [CSS named colors](colors.md) both work.

## Commands

### Rendering

```bash
# Render all sprites to PNG
pxl render input.jsonl

# Render specific sprite
pxl render input.jsonl --sprite hero

# Render with scaling (2x, 4x, etc.)
pxl render input.jsonl --scale 4

# Render animation as GIF
pxl render input.jsonl --gif

# Render animation as spritesheet
pxl render input.jsonl --spritesheet

# Strict mode (fail on warnings)
pxl render input.jsonl --strict
```

### Validation & Analysis

```bash
# Validate file for common mistakes
pxl validate input.jsonl

# Suggest fixes for issues (missing tokens, row completion)
pxl suggest input.jsonl

# Analyze corpus metrics (token frequency, dimensions)
pxl analyze input.jsonl

# Compare sprites between files
pxl diff file1.jsonl file2.jsonl --sprite hero
```

### Inspection & Display

```bash
# Show sprite with colored terminal output (ANSI)
pxl show input.jsonl --sprite hero

# Explain sprite structure in human-readable format
pxl explain input.jsonl --sprite hero

# Display grid with row/column coordinates
pxl grid input.jsonl --sprite hero
```

### Import & Export

```bash
# Import PNG image to pixelsrc format
pxl import image.png -o output.jsonl

# Import with max color limit
pxl import image.png --max-colors 16

# Build all assets per pxl.toml config
pxl build
```

### Formatting & Editing

```bash
# Format pixelsrc file for readability
pxl fmt input.jsonl

# Expand grid with column-aligned spacing
pxl inline input.jsonl --sprite hero

# Extract repeated patterns into aliases (JSON output)
pxl alias input.jsonl --sprite hero

# Transform sprites (mirror, rotate, etc.)
pxl transform input.jsonl --sprite hero --mirror-h -o flipped.jsonl
```

### Project Setup

```bash
# Initialize new pixelsrc project
pxl init my-project

# Create new asset from template
pxl new sprite hero

# List built-in palettes
pxl palette list

# Show details of built-in palette
pxl palette show gameboy
```

### AI Integration

```bash
# Print format guide for AI context injection
pxl prime

# Print brief format guide
pxl prime --brief

# Print specific section (format, examples, tips)
pxl prime --section format

# Verify content (JSON API for AI agents)
pxl verify input.jsonl

# GenAI prompt templates
pxl prompt
```

## Workflow

1. **Generate** - Create JSONL with palette and sprite definitions
2. **Validate** - Run `pxl validate file.jsonl` to check for errors
3. **Render** - Run `pxl render file.jsonl` to create PNG
4. **Iterate** - Fix any issues and re-render

## Tips for Better Sprites

1. **Start with silhouette** - Define the outline first
2. **Add base colors** - Fill main areas
3. **Add shading** - Darker colors for shadows, lighter for highlights
4. **Use limited palette** - 4-16 colors is typical for pixel art
5. **Think in layers** - Background, character, details
6. **Test at 1x scale** - Ensure readability at native size
