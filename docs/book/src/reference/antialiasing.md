# Antialiasing

Semantic-aware antialiasing for pixel art upscaling and edge smoothing.

## Overview

Pixelsrc provides optional antialiasing that uses semantic metadata to make intelligent smoothing decisions. Unlike traditional algorithms that treat all pixels equally, semantic-aware antialiasing respects the meaning of your pixel art - preserving crisp details like eyes while smoothing large fills.

By default, antialiasing is **disabled** to preserve the authentic pixel art aesthetic.

```json5
// Enable antialiasing in palette
{
  type: "palette",
  name: "hero",
  antialias: {
    enabled: true,
    algorithm: "hq2x",
    strength: 0.5
  }
}
```

## Algorithms

| Algorithm | Scale | Quality | Speed | Best For |
|-----------|-------|---------|-------|----------|
| `nearest` | 1x | - | Fastest | No antialiasing (default) |
| `scale2x` | 2x | Good | Fast | Simple upscaling, retro look |
| `hq2x` | 2x | Better | Medium | Balanced quality/performance |
| `hq4x` | 4x | Better | Medium | Higher resolution output |
| `xbr2x` | 2x | Best | Slower | Maximum quality at 2x |
| `xbr4x` | 4x | Best | Slower | Maximum quality at 4x |
| `aa-blur` | 1x | Subtle | Fast | Edge softening without scaling |

### nearest

No antialiasing applied. Preserves crisp pixel edges exactly as authored. This is the default behavior.

### scale2x

The Scale2x (EPX) algorithm performs 2x upscaling with edge-aware interpolation. It detects edges and curves to produce smoother diagonals while preserving sharp edges.

```text
Input pixel P with neighbors:
    A
  B P C
    D

Output 2x2 block:
  E0 E1
  E2 E3
```

The algorithm applies corner smoothing based on neighbor similarity patterns.

### hq2x / hq4x

HQ2x and HQ4x use pattern-based interpolation with YUV color comparison. They examine a 3x3 neighborhood around each pixel and apply interpolation rules based on which neighbors are "similar" in perceptual color space.

Key features:
- YUV color comparison for perceptually accurate edge detection
- Configurable similarity threshold
- Smooth diagonal handling

### xbr2x / xbr4x

xBR (Scale By Rules) algorithms provide the highest quality upscaling through edge direction and curvature analysis. They examine a 5x5 neighborhood to detect complex edge patterns.

Key features:
- Best visual quality for diagonal lines and curves
- Weighted edge direction detection
- Superior handling of complex pixel art patterns

### aa-blur

Gaussian blur with semantic masking. Unlike scaling algorithms, aa-blur operates at 1x resolution and selectively smooths edges based on semantic roles:

| Role | Blur Weight | Effect |
|------|-------------|--------|
| Anchor | 0% | No blur (crisp details) |
| Boundary | 25% | Light blur (soft edges) |
| Fill | 100% | Full blur (smooth fills) |
| Shadow/Highlight | 100% | Full blur (smooth gradients) |

## Configuration

### Basic Options

```json5
{
  antialias: {
    // Enable antialiasing (default: false)
    enabled: true,

    // Algorithm selection (default: "nearest")
    algorithm: "hq2x",

    // Smoothing strength 0.0-1.0 (default: 0.5)
    strength: 0.5
  }
}
```

### Anchor Mode

Controls how anchor pixels (important details like eyes) are handled:

```json5
{
  antialias: {
    enabled: true,
    algorithm: "xbr2x",

    // anchor_mode options:
    // - "preserve": No AA on anchors (default, keeps details crisp)
    // - "reduce": 25% AA strength on anchors
    // - "normal": Full AA on anchors (treat like any other region)
    anchor_mode: "preserve"
  }
}
```

### Semantic Options

```json5
{
  antialias: {
    enabled: true,
    algorithm: "hq4x",

    // Smooth transitions at DerivesFrom boundaries (default: true)
    // Creates gradual color blending between shadow/highlight and base
    gradient_shadows: true,

    // Respect ContainedWithin boundaries as hard edges (default: true)
    // Prevents color bleeding between regions like eye and skin
    respect_containment: true,

    // Use semantic role information for AA decisions (default: false)
    // When enabled, roles from palette guide smoothing choices
    semantic_aware: true
  }
}
```

### Per-Region Overrides

Fine-grained control for specific regions:

```json5
{
  antialias: {
    enabled: true,
    algorithm: "hq2x",

    regions: {
      // Preserve eyes completely (no AA)
      eye: { preserve: true },

      // Reduce AA on outline
      outline: { mode: "reduce" },

      // Disable gradient smoothing for hair shadow
      "hair-shadow": { gradient: false }
    }
  }
}
```

## Configuration Hierarchy

Antialiasing can be configured at multiple levels, with increasing precedence:

1. **pxl.toml defaults** - Project-wide defaults
2. **Atlas-level** - Per-atlas configuration
3. **Per-sprite** - Individual sprite settings
4. **CLI flags** - Highest priority (overrides everything)

### In pxl.toml

```toml
[defaults]
antialias_algorithm = "scale2x"
antialias_strength = 0.5

[atlases.characters]
sources = ["sprites/characters/**"]
antialias = { algorithm = "hq4x", strength = 0.7 }
```

### Per-Sprite in Palette

```json5
{
  type: "palette",
  name: "hero",
  antialias: {
    enabled: true,
    algorithm: "xbr2x",
    semantic_aware: true
  }
}
```

## CLI Usage

### Basic Rendering

```bash
# Render with antialiasing
pxl render sprite.pxl --antialias hq2x

# Render with strength control
pxl render sprite.pxl --antialias xbr4x --aa-strength 0.8

# Disable semantic awareness
pxl render sprite.pxl --antialias hq2x --no-semantic-aa
```

### Available Flags

| Flag | Description |
|------|-------------|
| `--antialias <algo>` | Select algorithm (scale2x, hq2x, hq4x, xbr2x, xbr4x, aa-blur) |
| `--aa-strength <0.0-1.0>` | Smoothing intensity |
| `--anchor-mode <mode>` | preserve, reduce, or normal |
| `--no-semantic-aa` | Disable semantic-aware processing |
| `--no-gradient-shadows` | Disable gradient smoothing |

### Build Command

```bash
# Build all with default AA settings from pxl.toml
pxl build

# Override AA for this build
pxl build --antialias hq4x
```

## Semantic Integration

Antialiasing integrates with [semantic metadata](../format/semantic.md) to make intelligent smoothing decisions.

### Roles Affect Smoothing

```json5
{
  type: "palette",
  colors: {
    outline: "#000000",
    skin: "#FFD5B4",
    eye: "#4169E1"
  },
  roles: {
    outline: "boundary",  // Light smoothing (25%)
    skin: "fill",         // Full smoothing
    eye: "anchor"         // Preserved (0% by default)
  },
  antialias: {
    enabled: true,
    algorithm: "hq2x",
    semantic_aware: true
  }
}
```

### Relationships Guide Blending

```json5
{
  relationships: {
    // DerivesFrom: Creates smooth gradient at boundary
    "skin-shadow": {
      type: "derives-from",
      target: "skin"
    },

    // ContainedWithin: Hard edge, no blending across boundary
    pupil: {
      type: "contained-within",
      target: "eye"
    }
  }
}
```

**DerivesFrom** relationships enable gradient smoothing - the transition between `skin-shadow` and `skin` will be gradual rather than a hard edge.

**ContainedWithin** relationships create hard boundaries - the `pupil` will never blend into surrounding `eye` pixels, maintaining crisp definition.

## Examples

### Retro Upscaling

For a classic scaled-up look with minimal smoothing:

```json5
{
  antialias: {
    enabled: true,
    algorithm: "scale2x",
    strength: 0.3,
    anchor_mode: "preserve"
  }
}
```

### High-Quality Character Sprites

For smooth character art at high resolution:

```json5
{
  antialias: {
    enabled: true,
    algorithm: "xbr4x",
    strength: 0.7,
    semantic_aware: true,
    gradient_shadows: true
  },
  roles: {
    outline: "boundary",
    eye: "anchor",
    skin: "fill",
    "skin-shadow": "shadow"
  },
  relationships: {
    "skin-shadow": { type: "derives-from", target: "skin" }
  }
}
```

### Subtle Edge Softening

For slight smoothing without scaling:

```json5
{
  antialias: {
    enabled: true,
    algorithm: "aa-blur",
    strength: 0.4,
    semantic_aware: true,
    anchor_mode: "preserve"
  }
}
```

### Pure Algorithm (No Semantic Awareness)

For traditional upscaling behavior:

```json5
{
  antialias: {
    enabled: true,
    algorithm: "hq4x",
    semantic_aware: false,
    gradient_shadows: false,
    respect_containment: false
  }
}
```

## Technical Notes

### Scale Factors

Algorithms produce fixed output scales:

- `nearest`, `aa-blur`: 1x (no scaling)
- `scale2x`, `hq2x`, `xbr2x`: 2x
- `hq4x`, `xbr4x`: 4x

When combining with manual scaling (`--scale`), antialiasing is applied first, then additional scaling uses nearest-neighbor.

### Color Comparison

HQ and xBR algorithms use YUV color space for similarity comparisons:
- More perceptually accurate than RGB
- Green channel weighted most heavily (human eye sensitivity)
- Configurable threshold affects edge detection sensitivity

### Performance Considerations

| Algorithm | Relative Speed |
|-----------|---------------|
| nearest | 1x (baseline) |
| aa-blur | ~2x |
| scale2x | ~3x |
| hq2x | ~5x |
| hq4x | ~8x |
| xbr2x | ~10x |
| xbr4x | ~20x |

For large atlases, consider using `scale2x` or `hq2x` for faster builds, reserving `xbr4x` for final production renders.

## Related

- [Semantic Metadata](../format/semantic.md) - Roles and relationships
- [render command](../cli/render.md) - Rendering options
- [Configuration](config.md) - pxl.toml reference
