# Format Overview

Pixelsrc uses a structured format for defining pixel art. Sprites are described using geometric regions rather than pixel grids, making them context-efficient and edit-friendly.

## File Format

Pixelsrc files use JSON5 syntax with one or more objects per file. Each object has a `type` field that identifies what kind of element it defines.

**File Extension:** `.pxl`

**Format Features:**
- Comments (`// ...` and `/* ... */`)
- Trailing commas
- Unquoted keys
- Multi-line strings

```json5
// hero.pxl - A complete character definition
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
  },
}

{
  type: "sprite",
  name: "hero",
  size: [16, 16],
  palette: "hero",
  regions: {
    outline: { stroke: [0, 0, 16, 16] },
    skin: { fill: "inside(outline)" },
  },
}
```

## Object Types

Every Pixelsrc object requires a `type` field. The supported types are:

| Type | Purpose | Learn More |
|------|---------|------------|
| `palette` | Define named color tokens with roles and relationships | [Palette](palette.md) |
| `sprite` | Define a pixel image using regions | [Sprite](sprite.md) |
| `state_rules` | Define visual states and effects | [State Rules](state-rules.md) |
| `animation` | Sequence sprites over time | [Animation](animation.md) |
| `variant` | Create color variations | [Variant](variant.md) |
| `composition` | Layer sprites together | [Composition](composition.md) |

## Structured Format

The core innovation of Pixelsrc is the **region-based** approach:

```json5
// Instead of pixel grids like this:
// grid: ["{o}{o}{o}", "{o}{s}{o}", "{o}{o}{o}"]

// We define semantic regions:
regions: {
  outline: { stroke: [0, 0, 3, 3] },
  skin: { fill: "inside(outline)" }
}
```

**Advantages:**
- Scales with semantic complexity, not pixel count
- Easy to edit (change a number, not rewrite rows)
- Semantic meaning is explicit (roles, relationships)
- AI-optimized (describe intent, compiler resolves pixels)

## Token System

Tokens are named color identifiers defined in palettes:

```json5
{
  type: "palette",
  name: "example",
  colors: {
    _: "transparent",        // conventional transparent
    skin: "#FFD5B4",         // semantic color name
    dark_hair: "#4A3728"     // underscores for multi-word
  }
}
```

Token names are referenced directly in regions (no braces):

```json5
regions: {
  skin: { fill: "inside(outline)" },  // token name = region name
  dark_hair: { rect: [2, 0, 12, 4] }
}
```

## Design Philosophy

**Lenient by default, strict when requested.**

When AI makes small mistakes, Pixelsrc fills the gaps and continues. This design choice makes the format reliable for AI generation while allowing strict validation for production pipelines.

### Lenient Mode (Default)

| Error | Behavior |
|-------|----------|
| Unknown token | Render as magenta `#FF00FF` |
| Region outside canvas | Clipped with warning |
| Duplicate name | Last definition wins |
| Invalid color | Use magenta placeholder |
| Missing palette | All regions render white with warning |

### Strict Mode (`--strict`)

All warnings become errors. Processing stops at first error with non-zero exit code. Use this mode in CI/CD pipelines.

## Example File

A complete Pixelsrc file demonstrating the structured format:

```json5
// character.pxl
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    hair: "#8B4513",
    eye: "#4169E1",
  },
  roles: {
    outline: "boundary",
    skin: "fill",
    eye: "anchor",
    "skin-shadow": "shadow",
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" },
  },
}

{
  type: "sprite",
  name: "hero",
  size: [16, 16],
  palette: "hero",
  regions: {
    // Background
    _: "background",

    // Head outline
    "head-outline": { stroke: [4, 0, 8, 10], round: 2 },

    // Hair at top
    hair: { fill: "inside(head-outline)", y: [0, 4] },

    // Face below hair
    skin: {
      fill: "inside(head-outline)",
      y: [4, 10],
      except: ["eye"],
    },

    // Eyes (symmetric)
    eye: {
      rect: [5, 5, 2, 2],
      symmetric: "x",
    },
  },
}
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

```json5
{
  type: "palette",
  name: "themed",
  colors: {
    "--primary": "#4169E1",
    _: "transparent",
    main: "var(--primary)",
    shadow: "color-mix(in oklch, var(--primary) 70%, black)",
  },
}
```

See [CSS Variables](css-variables.md) for full documentation.

## Stream Processing

Pixelsrc files use streaming JSON5 parsing:

1. Objects are parsed as complete JSON5 values
2. Objects are processed in order of appearance
3. Palettes must be defined before sprites that reference them
4. Regions within a sprite must define dependencies before dependents

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (lenient: may have warnings) |
| 1 | Error (strict: any warning; lenient: fatal error) |
| 2 | Invalid arguments |
