# Sprite

A sprite defines a pixel art image using a grid of color tokens. Sprites are the core building block of Pixelsrc.

## Basic Syntax

```json
{
  "type": "sprite",
  "name": "string (required)",
  "size": [width, height],
  "palette": "string | object (required)",
  "grid": ["row1", "row2", ...]
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"sprite"` |
| `name` | Yes | Unique identifier |
| `size` | No | `[width, height]` - inferred from grid if omitted |
| `palette` | Yes | Palette name (string) or inline colors (object) |
| `grid` | Yes | Array of strings, each string is one row of tokens |

## Example

```json
{"type": "sprite", "name": "coin", "palette": "gold", "grid": [
  "{_}{g}{g}{_}",
  "{g}{h}{g}{g}",
  "{g}{g}{g}{g}",
  "{_}{g}{g}{_}"
]}
```

### Try It

Modify the grid pattern and colors to create different sprites:

<div class="pixelsrc-demo" data-pixelsrc-demo>
  <textarea id="sprite-demo">{"type": "palette", "name": "gold", "colors": {"{_}": "#0000", "{g}": "#FFD700", "{h}": "#FFFACD"}}
{"type": "sprite", "name": "coin", "palette": "gold", "grid": ["{_}{g}{g}{_}", "{g}{h}{g}{g}", "{g}{g}{g}{g}", "{_}{g}{g}{_}"]}</textarea>
  <button onclick="pixelsrcDemo.renderFromTextarea('sprite-demo', 'sprite-demo-preview')">Try it</button>
  <div class="preview" id="sprite-demo-preview"></div>
</div>

Try adding more rows to make the coin larger, or change `{h}` (highlight) to create different shine effects.

<!-- DEMOS format/sprite#basic -->
<!-- /DEMOS -->

## Grid Format

The grid is an array of strings representing pixel rows:

- Each string is one row of the sprite
- Tokens are `{name}` format, concatenated: `"{a}{b}{c}"`
- Rows are ordered top-to-bottom
- Tokens within a row are left-to-right

### Example Grid

```json
"grid": [
  "{_}{_}{h}{h}{h}{_}{_}",
  "{_}{h}{h}{h}{h}{h}{_}",
  "{_}{s}{s}{s}{s}{s}{_}",
  "{_}{_}{b}{b}{b}{_}{_}"
]
```

This creates a 7x4 sprite with:
- Row 0: transparent, transparent, hair, hair, hair, transparent, transparent
- Row 1: transparent, hair across, transparent
- Row 2: transparent, skin across, transparent
- Row 3: transparent, body across, transparent

## Palette Options

### Named Palette

Reference a palette defined earlier in the file:

```json
{"type": "sprite", "name": "hero", "palette": "hero_colors", "grid": [...]}
```

<!-- DEMOS format/sprite#named_palette -->
<!-- /DEMOS -->

### Inline Palette

Define colors directly in the sprite:

```json
{"type": "sprite", "name": "dot", "palette": {
  "{_}": "#00000000",
  "{x}": "#FF0000"
}, "grid": ["{x}"]}
```

<!-- DEMOS format/sprite#inline_palette -->
<!-- /DEMOS -->

### Built-in Palette

Reference a built-in palette with `@` prefix:

```json
{"type": "sprite", "name": "retro", "palette": "@gameboy", "grid": [...]}
```

## Size Inference

If `size` is omitted:
- Width = maximum tokens in any row
- Height = number of rows

If `size` is provided:
- Rows shorter than width are padded with `{_}` (transparent)
- Rows longer than width are truncated
- Fewer rows than height: grid is padded
- More rows than height: grid is truncated

## Nine-Slice

Create scalable sprites where corners stay fixed while edges and center stretch. Useful for buttons, panels, and other UI elements.

```json
{
  "type": "sprite",
  "name": "button",
  "palette": "ui",
  "nine_slice": {
    "left": 4,
    "right": 4,
    "top": 4,
    "bottom": 4
  },
  "grid": [...]
}
```

### Nine-Slice Fields

| Field | Required | Description |
|-------|----------|-------------|
| `nine_slice` | No | Nine-slice region definition |
| `nine_slice.left` | Yes | Left border width in pixels |
| `nine_slice.right` | Yes | Right border width in pixels |
| `nine_slice.top` | Yes | Top border height in pixels |
| `nine_slice.bottom` | Yes | Bottom border height in pixels |

### Rendering Nine-Slice

```bash
pxl render button.pxl --nine-slice 64x32 -o button_wide.png
```

This scales the button to 64x32 pixels while preserving the 4-pixel borders.

## Metadata

Attach additional data for game engine integration:

```json
{
  "type": "sprite",
  "name": "player_attack",
  "palette": "hero",
  "grid": [...],
  "metadata": {
    "origin": [16, 32],
    "boxes": {
      "hurt": {"x": 4, "y": 0, "w": 24, "h": 32},
      "hit": {"x": 20, "y": 8, "w": 20, "h": 16}
    }
  }
}
```

### Metadata Fields

| Field | Required | Description |
|-------|----------|-------------|
| `metadata` | No | Sprite metadata object |
| `metadata.origin` | No | Sprite origin point `[x, y]` |
| `metadata.boxes` | No | Map of box name to rectangle |

### Box Rectangle Format

```json
{"x": 0, "y": 0, "w": 16, "h": 16}
```

### Common Box Types

| Name | Purpose |
|------|---------|
| `hurt` | Damage-receiving region |
| `hit` | Damage-dealing region |
| `collide` | Physics collision boundary |
| `trigger` | Interaction trigger zone |

<!-- DEMOS format/sprite#metadata -->
<!-- /DEMOS -->

## Transforms

Apply render-time modifications without changing the source:

```json
{
  "type": "sprite",
  "name": "hero_outlined",
  "source": "hero",
  "transform": [
    {"op": "sel-out", "fallback": "{outline}"}
  ]
}
```

See [Transforms](transforms.md) for all available operations.

## Complete Example

```json
{"type": "palette", "name": "coin", "colors": {
  "{_}": "#00000000",
  "{gold}": "#FFD700",
  "{shine}": "#FFFACD",
  "{shadow}": "#DAA520"
}}

{"type": "sprite", "name": "coin", "palette": "coin", "grid": [
  "{_}{gold}{gold}{_}",
  "{gold}{shine}{gold}{gold}",
  "{gold}{gold}{gold}{shadow}",
  "{_}{gold}{shadow}{_}"
], "metadata": {
  "origin": [2, 2],
  "boxes": {
    "collide": {"x": 0, "y": 0, "w": 4, "h": 4}
  }
}}
```

## Token Parsing

Tokens in grid strings follow this pattern:

```
\{[^}]+\}
```

**Parsing Algorithm:**
1. Scan string left-to-right
2. On `{`, begin token capture
3. On `}`, end token capture, emit token
4. Characters outside `{...}` generate warnings in lenient mode, errors in strict mode

**Token Examples:**

| Grid String | Parsed Tokens |
|-------------|---------------|
| `"{a}{b}{c}"` | `["{a}", "{b}", "{c}"]` |
| `"{_}{skin}{_}"` | `["{_}", "{skin}", "{_}"]` |
| `"{long_name}{x}"` | `["{long_name}", "{x}"]` |

## Error Handling

### Lenient Mode (Default)

| Error | Behavior |
|-------|----------|
| Row too short | Pad with `{_}` (transparent) |
| Row too long | Truncate with warning |
| Unknown token | Render as magenta `#FF00FF` |
| Empty grid | Create 1x1 transparent sprite |

### Strict Mode

All warnings become errors. Use `--strict` flag for CI validation.
