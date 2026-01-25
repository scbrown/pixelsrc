---
phase: 19
title: Advanced Transforms & Animation Features
---

# Phase 19: Advanced Transforms & Animation Features

> **STATUS UPDATE (2026-01-25)**
>
> Major refactoring since original plan. Many features now implemented via:
> - CSS transforms for keyframe animations
> - Op-style transforms for sprite derivation
> - Build system for atlas export
>
> See "Implementation Status" below for current state.

**Depends on:** Phase 18 (CSS transforms implemented)

---

## Implementation Status

### Complete (Documented & Implemented)

| Feature | Location | Notes |
|---------|----------|-------|
| **Palette cycling** | animation.md, `palette_cycle` attr | Rotates colors over time |
| **Frame tags** | animation.md, `tags` attr | Semantic frame ranges |
| **Atlas export** | Build system | `pxl build` with `pxl.toml` |
| **Dithering patterns** | transforms.md, `dither` op | Checker, Bayer, noise patterns |
| **Dither gradient** | transforms.md, `dither-gradient` op | Directional gradient dither |
| **Selective outline** | transforms.md, `sel-out` op | Color-aware outline |
| **Squash & stretch** | transforms.md, CSS `scale()` | Via Scale transform |
| **Secondary motion** | animation.md, `attachments` | Hair, capes, tails |
| **Hit/hurt boxes** | animation.md, `frame_metadata` | Per-frame collision regions |
| **Sub-pixel animation** | Transform::Subpixel | Color blending for < 1px motion |

### Remaining Work

| Feature | Priority | Implementation | Notes |
|---------|----------|----------------|-------|
| **Color ramps** | ★★★ High | Palette attribute | Auto-generate shadow/highlight colors with hue shift |
| **Nine-slice** | ★★☆ Medium | Sprite attribute | Scalable UI elements |
| **Blend modes** | ★★☆ Medium | Composition layer | multiply, screen, overlay, add |
| **Onion skinning** | ★★☆ Medium | CLI flag | Preview prev/next frames |
| **Particle systems** | ★☆☆ Lower | New type | Sparks, dust, effects |

### Deferred (Not Planned)

| Feature | Reason |
|---------|--------|
| **Arc motion paths** | CSS `cubic-bezier()` timing functions cover most curved motion needs. True bezier paths would add complexity for marginal benefit in pixel art contexts. |
| **Parallax hints** | Simple metadata addition. Can be added as sprite/layer `metadata` field when needed. Not worth dedicated syntax. |
| **Hue-shifted shadows** | Subsumed by Color Ramps feature which provides complete solution. |

---

## Remaining Features

### Color Ramps

**Priority:** ★★★ High
**Persona:** Pixel Artist, Motion Designer

Auto-generate palette colors along a ramp with hue shifting. Shadows aren't just darker - they shift toward cool/warm tones.

**Implementation:** Palette attribute `ramps`

```json
{
  "type": "palette",
  "name": "skin",
  "ramps": {
    "skin": {
      "base": "#E8B89D",
      "steps": 5,
      "shadow_shift": {"lightness": -15, "hue": 10, "saturation": 5},
      "highlight_shift": {"lightness": 12, "hue": -5, "saturation": -10}
    }
  }
}
```

**Generated tokens:**
- `{skin_2}` (darkest shadow)
- `{skin_1}` (shadow)
- `{skin}` (base)
- `{skin+1}` (highlight)
- `{skin+2}` (brightest highlight)

**Simpler syntax:**
```json
{
  "ramps": {
    "skin": {"base": "#E8B89D", "steps": 3}
  }
}
```
Uses sensible defaults for shift values.

---

### Nine-Slice

**Priority:** ★★☆ Medium
**Persona:** Game Developer

Scalable sprites where corners stay fixed and edges/center stretch. Essential for UI buttons, panels, dialog boxes.

**Implementation:** Sprite attribute `nine_slice`

```json
{
  "type": "sprite",
  "name": "button",
  "size": [24, 16],
  "palette": "ui",
  "nine_slice": {
    "left": 4,
    "right": 4,
    "top": 4,
    "bottom": 4
  },
  "regions": {
    "corner": {"rect": [0, 0, 4, 4]},
    "edge_h": {"rect": [4, 0, 16, 4]},
    "center": {"rect": [4, 4, 16, 8]}
  }
}
```

**CLI usage:**
```bash
pxl render button.pxl --nine-slice 64x32 -o button_wide.png
```

**In compositions:**
```json
{
  "layers": [
    {"sprite": "button", "position": [0, 0], "nine_slice_size": [100, 40]}
  ]
}
```

---

### Blend Modes

**Priority:** ★★☆ Medium
**Persona:** Pixel Artist, Motion Designer

Layer blending for compositions. Enables shadows, glows, and lighting effects.

**Implementation:** Composition layer attributes `blend` and `opacity`

```json
{
  "type": "composition",
  "name": "scene",
  "size": [64, 64],
  "layers": [
    {"sprite": "background", "position": [0, 0]},
    {"sprite": "shadow", "position": [10, 20], "blend": "multiply", "opacity": 0.5},
    {"sprite": "glow", "position": [5, 5], "blend": "screen"},
    {"sprite": "player", "position": [16, 16]}
  ]
}
```

**Blend modes:**
| Mode | Effect |
|------|--------|
| `normal` | Default, no blending |
| `multiply` | Darken (shadow) |
| `screen` | Lighten (glow) |
| `overlay` | Contrast |
| `add` | Additive (fire, energy) |
| `subtract` | Subtractive |

---

### Onion Skinning

**Priority:** ★★☆ Medium
**Persona:** Animator

Preview previous/next frames as transparent overlays. Essential for animation workflow.

**Implementation:** CLI flag on `pxl show`

```bash
pxl show walk_cycle.pxl --onion 2
```

Shows current frame with 2 previous and 2 next frames as ghosts.

**Options:**
- `--onion <count>` - Number of frames before/after
- `--onion-opacity <0-1>` - Ghost opacity (default 0.3)
- `--onion-prev-color <hex>` - Tint for previous frames (default: red)
- `--onion-next-color <hex>` - Tint for next frames (default: green)

---

### Particle Systems

**Priority:** ★☆☆ Lower
**Persona:** Motion Designer

Define particle emitters for effects like sparks, dust, rain.

**Implementation:** New type `particle`

```json
{
  "type": "particle",
  "name": "sparkle",
  "sprite": "spark",
  "emitter": {
    "rate": 5,
    "lifetime": [10, 20],
    "velocity": {"x": [-2, 2], "y": [-4, -1]},
    "gravity": 0.2,
    "fade": true,
    "rotation": [0, 360]
  }
}
```

**Use in composition:**
```json
{
  "type": "composition",
  "layers": [
    {"sprite": "gem"},
    {"particle": "sparkle", "position": [8, 8]}
  ]
}
```

---

## Task Dependency Diagram

```
                     PHASE 19 REMAINING WORK
═══════════════════════════════════════════════════════════════════

WAVE 1 (High Priority)
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                        ATF-R1                           │    │
│  │                     Color Ramps                         │    │
│  │                  (Palette attribute)                    │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 2 (Medium Priority - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │
│  │    ATF-R2     │  │    ATF-R3     │  │    ATF-R4     │        │
│  │  Nine-Slice   │  │  Blend Modes  │  │    Onion      │        │
│  │   (Sprite)    │  │ (Composition) │  │   Skinning    │        │
│  └───────────────┘  └───────────────┘  └───────────────┘        │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 3 (Lower Priority)
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                        ATF-R5                           │    │
│  │                   Particle Systems                      │    │
│  │                    (New type)                           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 4 (Testing & Docs)
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                        ATF-R6                           │    │
│  │              Test Suite & Documentation                 │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

SUMMARY: 6 tasks remaining (down from 17 in original plan)
```

---

## Tasks

### Task ATF-R1: Color Ramps

**Wave:** 1 (High Priority)

Implement automatic color ramp generation with hue-shifted shadows/highlights.

**Deliverables:**
- Add `ramps` attribute to Palette in `src/models.rs`
- Implement HSL shifting algorithm
- Generate tokens: `{name_2}`, `{name_1}`, `{name}`, `{name+1}`, `{name+2}`
- Support configurable shift values
- Sensible defaults for common pixel art use cases

**Verification:**
```bash
pxl render examples/color_ramps.pxl -o output.png
cargo test color_ramps
```

---

### Task ATF-R2: Nine-Slice

**Wave:** 2 (Parallel with ATF-R3, ATF-R4)

Implement scalable sprite slicing for UI elements.

**Deliverables:**
- Add `nine_slice` attribute to Sprite in `src/models.rs`
- Implement nine-slice rendering algorithm
- Support `--nine-slice WxH` CLI option
- Support `nine_slice_size` in composition layers

**Verification:**
```bash
pxl render button.pxl --nine-slice 64x32 -o button_wide.png
cargo test nine_slice
```

---

### Task ATF-R3: Blend Modes

**Wave:** 2 (Parallel with ATF-R2, ATF-R4)

Implement layer blending for compositions.

**Deliverables:**
- Add `blend` and `opacity` attributes to CompositionLayer
- Implement modes: normal, multiply, screen, overlay, add, subtract
- Update composition renderer

**Verification:**
```bash
pxl render composition_blend.pxl -o output.png
cargo test blend_modes
```

---

### Task ATF-R4: Onion Skinning

**Wave:** 2 (Parallel with ATF-R2, ATF-R3)

Implement animation preview with frame ghosts.

**Deliverables:**
- Add `--onion N` flag to `pxl show` command
- Render previous/next frames as transparent overlays
- Support `--onion-opacity` and color tint options
- Terminal-friendly rendering

**Verification:**
```bash
pxl show walk_cycle.pxl --onion 2
```

---

### Task ATF-R5: Particle Systems

**Wave:** 3 (Lower Priority)

Implement particle emitters for effects.

**Deliverables:**
- Add new `particle` type to format
- Implement emitter with rate, lifetime, velocity, gravity
- Support fade, rotation, and random seed
- Composition layer integration

**Verification:**
```bash
pxl render sparkle.pxl -o sparkle.gif
cargo test particle
```

---

### Task ATF-R6: Test Suite & Documentation

**Wave:** 4 (Final)

Comprehensive tests and documentation for all remaining Phase 19 features.

**Deliverables:**
- Unit tests for color ramps, nine-slice, blend modes, particles
- Integration tests for CLI commands
- Update MDbook documentation
- Update `pxl prime` output

**Verification:**
```bash
cargo test phase19
mdbook build docs/book
```

---

## Success Criteria

1. Color ramps generate correct hue-shifted palettes
2. Nine-slice scales UI elements correctly
3. Blend modes produce expected visual results
4. Onion skinning shows frame ghosts in terminal
5. Particle systems render to GIF
6. All tests pass
7. MDbook documentation complete

---

## Historical Note

The original Phase 19 plan (17 tasks) was written before:
- CSS transforms were implemented
- The format moved from grids to structured regions
- The build system was completed

This revision (6 tasks) reflects current reality: most "advanced" features are already in place. What remains are genuinely new capabilities not yet implemented.
