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

### Sprite

```json
{"type": "sprite", "name": "hero", "size": [16, 16], "palette": "hero", "grid": ["{_}{_}{hair}{hair}...", "..."]}
```

- `size`: optional, inferred from grid if omitted
- `palette`: name of previously defined palette, or inline colors object
- `grid`: array of strings, one per row, top to bottom

### Animation

```json
{"type": "animation", "name": "walk", "frames": ["frame1", "frame2", "frame3"], "duration": 100, "loop": true}
```

- `frames`: array of sprite names
- `duration`: milliseconds per frame (default: 100)
- `loop`: whether to loop (default: true)

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

A simple 8x8 coin sprite:

```jsonl
{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{shine}": "#FFFACD", "{shadow}": "#B8860B"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": [
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}",
  "{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{shadow}{shadow}{gold}",
  "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}",
  "{_}{_}{shadow}{shadow}{shadow}{shadow}{_}{_}"
]}
```

**Key observations:**
- Palette defined before sprite
- Semantic token names: `{gold}`, `{shine}`, `{shadow}`
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

## Output Commands

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
