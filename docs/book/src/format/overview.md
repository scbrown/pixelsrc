# Format Overview

Pixelsrc uses a text-based format for defining pixel art. This section provides complete documentation of the format specification.

## File Format

Pixelsrc files contain JSON objects, one per line (JSONL format). Each object has a `type` field that identifies what kind of element it defines.

**File Extensions:**
- `.pxl` - Preferred extension (supports multi-line JSON)
- `.jsonl` - Legacy extension (supported for backward compatibility)

**Format Support:**
- **Single-line JSONL**: Traditional one-object-per-line format
- **Multi-line JSON**: Objects can span multiple lines for readability

## Object Types

Every Pixelsrc object requires a `type` field. The supported types are:

| Type | Purpose | Learn More |
|------|---------|------------|
| `palette` | Define named color tokens | [Palette](palette.md) |
| `sprite` | Define a pixel grid | [Sprite](sprite.md) |
| `animation` | Sequence sprites over time | [Animation](animation.md) |
| `variant` | Create color variations | [Variant](variant.md) |
| `composition` | Layer sprites together | [Composition](composition.md) |

## Stream Processing

Pixelsrc files use streaming JSON parsing:

1. Objects are parsed as complete JSON values (may span multiple lines)
2. Objects are processed in order of appearance
3. Palettes must be defined before sprites that reference them (by name)
4. Forward references produce errors (lenient: magenta placeholder, strict: fail)

## Token System

Tokens are named color identifiers wrapped in curly braces: `{name}`. They're the core of Pixelsrc's design.

```
{_}         → transparent (conventional)
{skin}      → semantic color name
{dark_hair} → underscores for multi-word names
```

**Parsing Rules:**
- Tokens match the pattern `\{[^}]+\}`
- Case sensitive: `{Skin}` ≠ `{skin}`
- Whitespace preserved: `{ skin }` is valid (but discouraged)
- Recommended style: lowercase with underscores

## Design Philosophy

**Lenient by default, strict when requested.**

When AI makes small mistakes, Pixelsrc fills the gaps and continues. This design choice makes the format reliable for AI generation while allowing strict validation for production pipelines.

### Lenient Mode (Default)

| Error | Behavior |
|-------|----------|
| Row too short | Pad with transparent pixels |
| Row too long | Truncate with warning |
| Unknown token | Render as magenta `#FF00FF` |
| Duplicate name | Last definition wins |
| Invalid color | Use magenta placeholder |
| Empty grid | Create 1x1 transparent sprite |

### Strict Mode (`--strict`)

All warnings become errors. Processing stops at first error with non-zero exit code. Use this mode in CI/CD pipelines.

## Example File

A complete Pixelsrc file with multiple object types:

```json
{"type": "palette", "name": "hero", "colors": {"{_}": "#0000", "{skin}": "#FFCC99", "{hair}": "#8B4513", "{shirt}": "#4169E1"}}

{"type": "sprite", "name": "hero_stand", "palette": "hero", "grid": [
  "{_}{_}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{skin}{_}{_}"
]}

{"type": "sprite", "name": "hero_walk", "palette": "hero", "grid": [
  "{_}{_}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{skin}{_}{_}{_}{skin}{_}"
]}

{"type": "animation", "name": "hero_walk_cycle", "frames": ["hero_stand", "hero_walk"], "duration": 200}
```

## Color Formats

Pixelsrc supports multiple color formats:

| Format | Example | Description |
|--------|---------|-------------|
| `#RGB` | `#F00` | Expands to `#RRGGBB` (red) |
| `#RGBA` | `#F00F` | Expands to `#RRGGBBAA` (red, opaque) |
| `#RRGGBB` | `#FF0000` | Fully opaque color |
| `#RRGGBBAA` | `#FF000080` | With alpha channel |
| `rgb()` | `rgb(255, 0, 0)` | CSS RGB notation |
| `hsl()` | `hsl(0, 100%, 50%)` | CSS HSL notation |
| Named | `red`, `transparent` | CSS named colors |

The alpha channel controls transparency: `00` = fully transparent, `FF` = fully opaque.

See [Color Formats Reference](../reference/colors.md) for full documentation including `oklch()` and `color-mix()`.

## CSS Variables

Palettes support CSS custom properties for dynamic theming:

```json
{"type": "palette", "name": "themed", "colors": {
  "--primary": "#4169E1",
  "{_}": "transparent",
  "{main}": "var(--primary)",
  "{shadow}": "color-mix(in oklch, var(--primary) 70%, black)"
}}
```

- **Define variables**: `"--name": "value"`
- **Reference variables**: `var(--name)` or `var(--name, fallback)`
- **Generate variants**: `color-mix(in oklch, color 70%, black)` for shadows

See [CSS Variables](css-variables.md) for full documentation.

## Size Inference

When `size` is omitted from a sprite:
- Width = maximum tokens in any row
- Height = number of rows

When `size` is provided:
- Rows are padded or truncated to match width
- Grid is padded or truncated to match height

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (lenient: may have warnings) |
| 1 | Error (strict: any warning; lenient: fatal error) |
| 2 | Invalid arguments |
