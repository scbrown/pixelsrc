# Migrating from v1 to v2

This guide covers migrating from the v1 grid-based format to the v2 structured region format.

---

## Overview

Pixelsrc v2 replaces pixel grids with geometric regions:

| Aspect | v1 | v2 |
|--------|----|----|
| Sprite definition | `grid: ["{a}{b}{c}", ...]` | `regions: { token: shape }` |
| Token syntax | `{token}` with braces | `token` without braces |
| File format | Strict JSON | JSON5 |
| Semantic metadata | None | Roles, relationships |
| Size scaling | Scales with pixels | Scales with complexity |

---

## Quick Migration

The `pxl import` command can convert existing PNGs to the new format:

```bash
# Render old format to PNG
pxl render old_sprite.jsonl -o temp.png

# Import PNG to new format
pxl import temp.png --analyze -o new_sprite.pxl
```

For manual migration, follow the patterns below.

---

## Format Changes

### File Extension

| Before | After |
|--------|-------|
| `hero.jsonl` | `hero.pxl` |

The `.pxl` extension signals JSON5 format support.

### JSON5 Syntax

v2 uses JSON5, enabling cleaner syntax:

```json5
// v2: Comments allowed
{
  type: "palette",  // Unquoted keys
  name: "hero",
  colors: {
    _: "transparent",
    skin: "#FFD5B4",  // Trailing commas OK
  },
}
```

### Token Names

Tokens no longer use braces:

**v1:**
```json
{"colors": {"{_}": "#0000", "{skin}": "#FFD5B4"}}
```

**v2:**
```json5
{colors: {_: "transparent", skin: "#FFD5B4"}}
```

### Sprite Definition

The biggest change: `grid` is replaced by `regions`.

**v1 (grid-based):**
```json
{"type": "sprite", "name": "dot", "size": [3, 3], "palette": "mono", "grid": [
  "{_}{x}{_}",
  "{x}{x}{x}",
  "{_}{x}{_}"
]}
```

**v2 (region-based):**
```json5
{
  type: "sprite",
  name: "dot",
  size: [3, 3],
  palette: "mono",
  regions: {
    _: "background",
    x: { points: [[1, 0], [0, 1], [1, 1], [2, 1], [1, 2]] }
  }
}
```

Or using shapes:
```json5
{
  type: "sprite",
  name: "dot",
  size: [3, 3],
  palette: "mono",
  regions: {
    _: "background",
    x: { circle: [1, 1, 1] }  // cx=1, cy=1, r=1
  }
}
```

---

## Common Patterns

### Outlined Character

**v1:**
```json
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "hero", "grid": [
  "{_}{_}{o}{o}{o}{o}{_}{_}",
  "{_}{o}{s}{s}{s}{s}{o}{_}",
  "{o}{s}{s}{s}{s}{s}{s}{o}",
  "{o}{s}{e}{s}{s}{e}{s}{o}",
  "{o}{s}{s}{s}{s}{s}{s}{o}",
  "{_}{o}{s}{s}{s}{s}{o}{_}",
  "{_}{_}{o}{o}{o}{o}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}"
]}
```

**v2:**
```json5
{
  type: "sprite",
  name: "hero",
  size: [8, 8],
  palette: "hero",
  regions: {
    _: "background",
    o: { stroke: [2, 0, 4, 6], round: 1 },
    s: { fill: "inside(o)" },
    e: { points: [[2, 3]], symmetric: "x" }
  }
}
```

### Symmetric Features

**v1:** Manually place mirrored pixels
```json
"grid": ["{_}{e}{_}{_}{e}{_}"]  // eyes at x=1 and x=4
```

**v2:** Use `symmetric` modifier
```json5
regions: {
  e: { points: [[1, 3]], symmetric: "x" }  // auto-mirrors
}
```

### Fill with Exceptions

**v1:** Manually omit pixels in grid
```json
"grid": ["{s}{s}{e}{s}{s}"]  // eye hole in skin
```

**v2:** Use `except` modifier
```json5
regions: {
  outline: { stroke: [0, 0, 5, 1] },
  e: { points: [[2, 0]] },
  s: { fill: "inside(outline)", except: ["e"] }
}
```

### Row Constraints

**v1:** Different tokens per row
```json
"grid": [
  "{hair}{hair}{hair}",
  "{skin}{skin}{skin}",
  "{shirt}{shirt}{shirt}"
]
```

**v2:** Use `y` range constraint
```json5
regions: {
  outline: { stroke: [0, 0, 3, 3] },
  hair: { fill: "inside(outline)", y: [0, 0] },
  skin: { fill: "inside(outline)", y: [1, 1] },
  shirt: { fill: "inside(outline)", y: [2, 2] }
}
```

---

## Semantic Metadata (New in v2)

v2 adds optional semantic metadata:

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    eye: "#4169E1"
  },
  roles: {
    outline: "boundary",
    skin: "fill",
    eye: "anchor",
    "skin-shadow": "shadow"
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" }
  }
}
```

This metadata enables:
- Smarter scaling (anchors preserved, fill can shrink)
- Validation (contained-within, adjacent-to)
- Better import analysis

---

## Removed Features

| Feature | v1 | v2 Alternative |
|---------|----|----|
| `grid` field | Yes | `regions` |
| `{braces}` in tokens | Yes | Bare token names |
| RLE compression | Yes | Not needed (regions are compact) |
| `pxl alias` command | Yes | Removed |
| `pxl inline` command | Yes | Removed |
| `pxl sketch` command | Yes | Removed |
| `pxl grid` command | Yes | Removed |

---

## Workflow

1. **Export v1 sprites to PNG** (if not already)
   ```bash
   pxl render old.jsonl -o sprites/
   ```

2. **Import PNGs to v2 format**
   ```bash
   for png in sprites/*.png; do
     pxl import "$png" --analyze -o "${png%.png}.pxl"
   done
   ```

3. **Review and refine** - The import produces valid v2 files, but you may want to:
   - Add semantic roles
   - Simplify complex regions into shapes
   - Add symmetry where applicable

4. **Validate**
   ```bash
   pxl validate *.pxl --strict
   ```

---

## Troubleshooting

### "Unknown field: grid"

Your file still uses v1 format. Migrate using the patterns above or use `pxl import` on a rendered PNG.

### "Forward reference in fill"

In v2, regions must define dependencies before dependents:

**Wrong:**
```json5
regions: {
  skin: { fill: "inside(outline)" },  // ERROR
  outline: { stroke: [0, 0, 8, 8] }
}
```

**Right:**
```json5
regions: {
  outline: { stroke: [0, 0, 8, 8] },  // Define first
  skin: { fill: "inside(outline)" }   // Then reference
}
```

### "Invalid JSON5"

Check for:
- Missing commas between object entries
- Unmatched braces
- Invalid escape sequences

JSON5 is more lenient than JSON but still has rules.

---

## Benefits of v2

1. **Context efficiency** - 64x64 sprite takes same space as 8x8 with similar structure
2. **Edit friendly** - Change `rect: [2, 4, 12, 8]` instead of rewriting 96 tokens
3. **Semantic meaning** - Roles and relationships are explicit
4. **AI optimized** - Describe intent, compiler resolves pixels
5. **Better diffs** - Git shows "changed outline width" not "changed 50 tokens"
