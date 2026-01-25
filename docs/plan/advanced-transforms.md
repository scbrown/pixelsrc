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
| **Color ramps** | palette.rs, color.rs, registry.rs | Auto-generate hue-shifted shadow/highlight (TTP-57ti) |
| **Nine-slice** | sprite.rs, renderer.rs | Scalable UI sprites with `--nine-slice WxH` (TTP-fzq9) |

### Remaining Work

| Feature | Priority | Implementation | Notes |
|---------|----------|----------------|-------|
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

## Transform Architecture

Pixelsrc has multiple levels at which transforms can operate:

### Current Transform Levels

| Level | When Applied | Operates On | Location |
|-------|--------------|-------------|----------|
| **Post-render** | After regions → pixels | `RgbaImage` | `src/transforms/apply.rs` |
| **Animation-time** | During keyframe interpolation | Rendered frames | `src/transforms/css.rs` |

**Post-render transforms** (op-style): mirror, rotate, scale, dither, sel-out, outline, shadow.
These operate on the rendered pixel buffer.

**Animation-time transforms** (CSS-style): translate, rotate, scale, flip, skew.
These are specified in keyframes and interpolated during animation.

### Future: Semantic/Pre-render Transforms

A third level could operate on the **structured region definitions** before rendering:

| Level | When Applied | Operates On | Benefits |
|-------|--------------|-------------|----------|
| **Pre-render / Semantic** | Before regions → pixels | Region definitions | Pixel-perfect results |

**Example: Semantic Rotation**

Instead of rotating pixels (which can blur), transform the region coordinates:
- Input: `{"rect": [2, 4, 8, 6]}` rotated 90°
- Output: `{"rect": [4, 2, 6, 8]}` (coordinates transformed)
- Rendering produces pixel-perfect output from the transformed regions

This would enable:
- Pixel-perfect geometric transforms (no interpolation artifacts)
- Transforms that understand sprite structure (e.g., rotate "head" region independently)
- Region-aware operations impossible at the pixel level

**Not yet implemented.** Would require new transform system operating on the AST/model level.

---

## Remaining Features

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

WAVE 1 (Medium Priority - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────┐  ┌─────────────────────────┐       │
│  │        ATF-R1           │  │        ATF-R2           │       │
│  │     Blend Modes         │  │    Onion Skinning       │       │
│  │    (Composition)        │  │       (CLI)             │       │
│  └─────────────────────────┘  └─────────────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 2 (Lower Priority)
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                        ATF-R3                           │    │
│  │                   Particle Systems                      │    │
│  │                    (New type)                           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 3 (Testing & Docs)
┌─────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                        ATF-R4                           │    │
│  │              Test Suite & Documentation                 │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

COMPLETED: Color Ramps (TTP-57ti), Nine-Slice (TTP-fzq9)
SUMMARY: 4 tasks remaining (down from 17 in original plan)
```

---

## Tasks

### Task ATF-R1: Blend Modes

**Wave:** 1 (Parallel with ATF-R2)

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

### Task ATF-R2: Onion Skinning

**Wave:** 1 (Parallel with ATF-R1)

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

### Task ATF-R3: Particle Systems

**Wave:** 2 (Lower Priority)

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

### Task ATF-R4: Test Suite & Documentation

**Wave:** 3 (Final)

Comprehensive tests and documentation for remaining Phase 19 features.

**Deliverables:**
- Unit tests for blend modes, onion skinning, particles
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

1. ~~Color ramps generate correct hue-shifted palettes~~ ✓ Complete (TTP-57ti)
2. ~~Nine-slice scales UI elements correctly~~ ✓ Complete (TTP-fzq9)
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

This revision (4 tasks remaining) reflects current reality: most "advanced" features are already in place, including Color Ramps and Nine-Slice which were completed by polecats (TTP-57ti, TTP-fzq9). What remains are genuinely new capabilities not yet implemented.
