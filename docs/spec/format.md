# Pixelsrc Format Specification

**Version:** 0.2.0 (Draft)

---

## Overview

Pixelsrc (Text To Pixel) is a text-based format for defining pixel art sprites using JSON objects with a `type` field.

**File Extensions:**
- `.pxl` - Preferred extension (supports multi-line JSON)
- `.jsonl` - Legacy extension (supported for backward compatibility)

**Format Support:**
- **Single-line JSONL**: Traditional one-object-per-line format
- **Multi-line JSON**: Objects can span multiple lines for readability

**Design Philosophy:** Lenient by default, strict when requested. When GenAI makes small mistakes, fill the gaps and keep going.

---

## Object Types

### Palette

Defines named color tokens for use in sprites.

```json
{
  "type": "palette",
  "name": "string (required)",
  "colors": {
    "{token}": "#RRGGBB | #RRGGBBAA | #RGB | #RGBA (required, at least one)"
  }
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"palette"` |
| name | Yes | Unique identifier, referenced by sprites |
| colors | Yes | Map of token → color. Tokens must be `{name}` format |

**Color Formats:**
- `#RGB` → expands to `#RRGGBB` (e.g., `#F00` → `#FF0000`)
- `#RGBA` → expands to `#RRGGBBAA` (e.g., `#F00F` → `#FF0000FF`)
- `#RRGGBB` → fully opaque
- `#RRGGBBAA` → with alpha channel

**Reserved Tokens:**
- `{_}` → Recommended for transparency, but not enforced

---

### Sprite

Defines a pixel art image.

```json
{
  "type": "sprite",
  "name": "string (required)",
  "size": [width, height] (optional),
  "palette": "string | object (required)",
  "grid": ["row1", "row2", ...] (required)
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| type | Yes | Must be `"sprite"` |
| name | Yes | Unique identifier |
| size | No | `[width, height]` - inferred from grid if omitted |
| palette | Yes | Palette name (string) or inline colors (object) |
| grid | Yes | Array of strings, each string is one row of tokens |

**Palette Reference Options:**
- Named: `"palette": "hero_colors"` → references palette defined earlier in stream
- Inline: `"palette": {"{_}": "#00000000", "{skin}": "#FFCC99"}`
- Built-in: `"palette": "@gameboy"` → references built-in palette (Phase 1)

**Grid Format:**
- Each string is one row of the sprite
- Tokens are `{name}` format, concatenated: `"{a}{b}{c}"`
- Rows are ordered top-to-bottom
- Tokens within row are left-to-right

---

### Animation (Phase 2)

Defines a sequence of sprites as an animation.

```json
{
  "type": "animation",
  "name": "string (required)",
  "frames": ["sprite_name", ...] (required),
  "duration": number (optional, default 100),
  "loop": boolean (optional, default true)
}
```

**Fields:**
| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| type | Yes | - | Must be `"animation"` |
| name | Yes | - | Unique identifier |
| frames | Yes | - | Array of sprite names in order |
| duration | No | 100 | Milliseconds per frame |
| loop | No | true | Whether animation loops |

---

## Token Parsing

Tokens in grid strings follow this pattern:

```
\{[^}]+\}
```

**Parsing Algorithm:**
1. Scan string left-to-right
2. On `{`, begin token capture
3. On `}`, end token capture, emit token
4. Characters outside `{...}` are errors (see Error Handling)

**Examples:**
| Grid String | Parsed Tokens |
|-------------|---------------|
| `"{a}{b}{c}"` | `["{a}", "{b}", "{c}"]` |
| `"{_}{skin}{_}"` | `["{_}", "{skin}", "{_}"]` |
| `"{long_name}{x}"` | `["{long_name}", "{x}"]` |

**Token Names:**
- Case sensitive: `{Skin}` ≠ `{skin}`
- Whitespace preserved: `{ skin }` is a valid (but discouraged) token
- Recommended: lowercase, underscores: `{dark_skin}`, `{hair_highlight}`

---

## Error Handling

Pixelsrc has two modes: **lenient** (default) and **strict**.

### Lenient Mode (Default)

Fill gaps, warn, continue. Designed for GenAI iteration.

| Error | Behavior | Warning |
|-------|----------|---------|
| Row too short | Pad with `{_}` (transparent) | "Row N has M tokens, expected W" |
| Row too long | Truncate | "Row N has M tokens, expected W, truncating" |
| Unknown token in grid | Render as magenta `#FF00FF` | "Unknown token {foo} in sprite X" |
| Undefined palette reference | Error if no inline fallback | "Palette 'X' not found" |
| Duplicate name | Last definition wins | "Duplicate sprite name 'X', using latest" |
| Invalid color format | Use magenta `#FF00FF` | "Invalid color 'X', using magenta" |
| Characters outside tokens | Ignore | "Unexpected character 'X' in grid row" |
| Empty grid | Create 1x1 transparent | "Empty grid in sprite X" |
| Missing required field | Error (cannot fill) | "Missing required field 'X'" |

### Strict Mode (`--strict`)

Fail on first error. Designed for CI/validation.

All warnings in lenient mode become errors in strict mode. Processing stops at first error with non-zero exit code.

---

## Size Inference

If `size` is omitted:
- Width = max tokens in any row
- Height = number of rows

If `size` is provided:
- Rows are padded/truncated to match width
- Grid is padded/truncated to match height

---

## Stream Processing

Pixelsrc files use streaming JSON parsing:

1. Objects are parsed as complete JSON values (may span multiple lines)
2. Objects are processed in order of appearance
3. Palettes must be defined before sprites that reference them (by name)
4. Forward references are errors (lenient: use magenta, strict: fail)

**Whitespace:** Ignored between objects
**Comments:** Not supported in JSON (use separate documentation)

### Single-Line Format (JSONL)

Traditional format with one object per line:

```jsonl
{"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF"}}
{"type": "sprite", "name": "dot", "palette": "mono", "grid": ["{on}"]}
```

### Multi-Line Format

Objects can span multiple lines for improved readability, especially for sprite grids:

```json
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "colors", "grid": [
  "{_}{_}{hair}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}"
]}
```

Both formats parse identically—the renderer handles concatenated JSON objects regardless of whitespace.

---

## Output Behavior

### Default Output Naming

```bash
pxl render input.pxl      # or input.jsonl
```

| Scenario | Output |
|----------|--------|
| Single sprite "hero" | `input_hero.png` |
| Multiple sprites | `input_{name}.png` for each |
| With `-o output.png` (single sprite) | `output.png` |
| With `-o output.png` (multiple) | `output_{name}.png` |
| With `-o dir/` | `dir/{name}.png` |
| With `--sprite hero` | Only render "hero" |

Both `.pxl` and `.jsonl` extensions produce identical output—the extension only affects the source file naming convention.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (lenient: may have warnings) |
| 1 | Error (strict: any warning; lenient: fatal error) |
| 2 | Invalid arguments |

---

## Formatting

The `pxl fmt` command formats pixelsrc files for improved readability.

### Command Options

```bash
pxl fmt <files...>         # Format files in-place
pxl fmt <files> --check    # Check formatting without writing (exit 1 if changes needed)
pxl fmt <files> --stdout   # Write formatted output to stdout
```

### Formatting Rules

The formatter applies these rules:

**Sprites** - Grid arrays expanded for visual alignment:
```json
{"type": "sprite", "name": "hero", "size": [16, 16], "palette": "colors", "grid": [
  "{_}{_}{_}{_}{o}{o}{o}{o}{o}{o}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{o}{skin}{skin}{skin}{skin}{skin}{skin}{o}{_}{_}{_}{_}{_}",
  ...
]}
```

**Compositions** - Layer maps expanded for visual clarity:
```json
{"type": "composition", "name": "scene", "size": [64, 64], "sprites": {"H": "hero", "T": "tree"}, "layers": [
  {"name": "background", "fill": "grass"},
  {"name": "objects", "map": [
    "T......T",
    "........",
    "...H....",
    "........"
  ]}
]}
```

**Palettes** - Single line for compactness:
```json
{"type": "palette", "name": "colors", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{o}": "#000000"}}
```

**Animations** - Single line:
```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

### Round-Trip Safety

Formatting is lossless—rendered output is identical before and after formatting:

```bash
pxl render input.jsonl -o before.png
pxl fmt input.jsonl --stdout > formatted.pxl
pxl render formatted.pxl -o after.png
diff before.png after.png  # Identical
```

---

## Examples

### Minimal Sprite (Inline Palette)

```jsonl
{"type": "sprite", "name": "dot", "palette": {"{_}": "#00000000", "{x}": "#FF0000"}, "grid": ["{x}"]}
```

### Sprite with Named Palette (Multi-Line)

```json
{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFD5B4", "{hair}": "#8B4513", "{shirt}": "#4169E1"}}

{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "hero", "grid": [
  "{_}{_}{hair}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}"
]}
```

### Checker Pattern (Single-Line)

```jsonl
{"type": "palette", "name": "mono", "colors": {"{_}": "#00000000", "{on}": "#FFFFFF", "{off}": "#000000"}}
{"type": "sprite", "name": "checker", "palette": "mono", "grid": ["{on}{off}{on}{off}", "{off}{on}{off}{on}", "{on}{off}{on}{off}", "{off}{on}{off}{on}"]}
```

### Animation

```jsonl
{"type": "palette", "name": "blink", "colors": {"{_}": "#00000000", "{o}": "#FFFF00"}}
{"type": "sprite", "name": "on", "palette": "blink", "grid": ["{o}{o}", "{o}{o}"]}
{"type": "sprite", "name": "off", "palette": "blink", "grid": ["{_}{_}", "{_}{_}"]}
{"type": "animation", "name": "blink_anim", "frames": ["on", "off"], "duration": 500, "loop": true}
```

---

## Implementation Notes

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `serde`, `serde_json` | JSON parsing |
| `image` | PNG/GIF/WebP generation (native, no ImageMagick) |
| `clap` | CLI argument parsing |
| `regex` | Token extraction (or manual parser) |

### Rendering Pipeline

1. Parse JSONL line-by-line
2. Build palette registry (name → colors map)
3. For each sprite:
   a. Resolve palette (named or inline)
   b. Parse grid into 2D token array
   c. Map tokens to RGBA colors
   d. Create `RgbaImage` and set pixels
   e. Save to output format

---

## Version History

| Version | Changes |
|---------|---------|
| 0.2.0 | Add `.pxl` extension, multi-line JSON support, `pxl fmt` command |
| 0.1.0 | Initial draft |
