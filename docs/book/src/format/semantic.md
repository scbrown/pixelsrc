# Semantic Metadata

Semantic metadata adds meaning to your sprites through roles and relationships. This enables smarter tooling, better transforms, and clearer intent.

## Overview

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    eye: "#4169E1"
  },
  roles: {
    outline: "boundary",
    eye: "anchor",
    skin: "fill",
    "skin-shadow": "shadow"
  },
  relationships: {
    "skin-shadow": {
      type: "derives-from",
      target: "skin"
    },
    pupil: {
      type: "contained-within",
      target: "eye"
    }
  }
}
```

## Roles

Roles classify tokens by their visual/functional purpose. They're defined in the palette and inform how tools handle each region during transforms, scaling, and analysis.

### Available Roles

| Role | Meaning | Transform Behavior |
|------|---------|-------------------|
| `boundary` | Edge-defining (outlines) | High priority, preserve connectivity |
| `anchor` | Critical details (eyes, buttons) | Must survive transforms (min 1px) |
| `fill` | Interior mass (skin, clothes) | Can shrink, low priority |
| `shadow` | Depth indicators | Derives from parent |
| `highlight` | Light indicators | Derives from parent |

### Defining Roles

```json5
{
  type: "palette",
  name: "character",
  colors: {
    outline: "#000000",
    skin: "#FFD5B4",
    eye: "#4169E1",
    "skin-shadow": "#D4A574",
    shine: "#FFFFFF"
  },
  roles: {
    outline: "boundary",
    skin: "fill",
    eye: "anchor",
    "skin-shadow": "shadow",
    shine: "highlight"
  }
}
```

### Role Guidelines

**boundary**: Use for outlines and edges that define the sprite's silhouette. These pixels maintain connectivity even during aggressive scaling.

**anchor**: Use for small, critical details that identify the sprite - eyes, buttons, logos. These are protected during transforms and will be preserved even at minimum 1px size.

**fill**: Use for large interior regions - skin, clothing, backgrounds. These can be safely reduced when scaling down.

**shadow**: Use for darker variants that add depth. Tooling understands these derive from a base color.

**highlight**: Use for lighter variants that add emphasis. Same relationship understanding as shadows.

## Relationships

Relationships define semantic connections between tokens. They help tooling understand how colors relate and validate that sprites maintain their intended structure.

### Relationship Types

| Relationship | Meaning |
|--------------|---------|
| `derives-from` | Color derived from another token |
| `contained-within` | Spatially inside another region |
| `adjacent-to` | Must touch specified region |
| `paired-with` | Symmetric relationship |

### derives-from

Indicates a color is a variant of another. Used for shadow/highlight relationships.

```json5
relationships: {
  "skin-shadow": {
    type: "derives-from",
    target: "skin"
  },
  "skin-highlight": {
    type: "derives-from",
    target: "skin"
  }
}
```

### contained-within

Indicates a region should be spatially inside another. Tooling validates this constraint.

```json5
relationships: {
  pupil: {
    type: "contained-within",
    target: "eye"
  },
  "button-icon": {
    type: "contained-within",
    target: "button"
  }
}
```

### adjacent-to

Indicates a region should touch another. Useful for outlines and borders.

```json5
relationships: {
  outline: {
    type: "adjacent-to",
    target: "skin"
  }
}
```

### paired-with

Indicates symmetric or complementary regions. Tooling maintains their relationship during transforms.

```json5
relationships: {
  "left-eye": {
    type: "paired-with",
    target: "right-eye"
  },
  "left-arm": {
    type: "paired-with",
    target: "right-arm"
  }
}
```

## Usage in CLI

### Show Roles

Display role annotations on a sprite:

```bash
pxl show sprite.pxl --roles
```

### Validate Relationships

Check that all relationships are satisfied:

```bash
pxl validate sprite.pxl --strict
```

In strict mode, relationship violations become errors:
- `contained-within` checked against actual pixel positions
- `adjacent-to` verified that regions share at least one edge
- `paired-with` validated for structural consistency

### Import with Analysis

Auto-detect roles and relationships during import:

```bash
pxl import sprite.png --analyze
```

This uses heuristics to infer semantic metadata:
- 1px edges become `boundary`
- Small isolated regions (< 4px) become `anchor`
- Large interior regions become `fill`
- Darker variants detected as `derives-from` shadows

## Example

Complete palette with semantic metadata:

```json5
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
    pupil: "#000000",
    shirt: "#E74C3C",
    "shirt-shadow": "#C0392B"
  },
  roles: {
    outline: "boundary",
    skin: "fill",
    "skin-shadow": "shadow",
    hair: "fill",
    eye: "anchor",
    pupil: "anchor",
    shirt: "fill",
    "shirt-shadow": "shadow"
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" },
    "shirt-shadow": { type: "derives-from", target: "shirt" },
    pupil: { type: "contained-within", target: "eye" }
  }
}
```

This metadata enables:
- **Smarter scaling**: Eyes preserved, skin can shrink
- **Color variant generation**: Tool knows shadow relationships
- **Validation**: Pupil must stay inside eye region
- **Better imports**: Heuristics can suggest this structure automatically
