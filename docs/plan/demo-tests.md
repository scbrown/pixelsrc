---
phase: 23
title: Demo Tests
---

# Phase 23: Demo Tests

Integration tests that double as product documentation and regression prevention.

**Personas:** All (documentation benefits everyone)

**Status:** Complete

**Depends on:** Phase 21 (mdbook Documentation), Phase 22 (CSS Integration)

**Related:**
- [css.md](css.md) - CSS features to cover
- [refactor.md](refactor.md) - Code structure improvements
- [mdbook.md](mdbook.md) - Documentation home

---

## Overview

| Component | Purpose |
|-----------|---------|
| Test Harness | Rust utilities for text-based verification |
| Demo Fixtures | JSONL files demonstrating each feature |
| Doc Generator | Extract test metadata → mdbook pages |
| CI Integration | Coverage checks, auto-regeneration |

---

## Strategic Vision

### The Problem

- Tests verify implementation, not user-visible capabilities
- Documentation and tests are separate, leading to drift
- No regression protection for "does the product still work?"
- Features without tests are undocumented (and effectively don't exist)

### Solution

**Every feature gets a demo test that:**
1. Verifies the feature works (regression protection)
2. Generates documentation (embedded in mdbook)
3. Provides runnable examples (WASM playground)

### Principles

1. **Tests are demos**: Every test showcases a user-visible capability
2. **Demos are docs**: Every demo generates mdbook documentation
3. **No binary files**: All verification is text-based (hashes, dimensions, metadata)
4. **Feature-first organization**: Group by capability, then by complexity

---

## Task Dependency Diagram

```
                          DEMO TESTS TASK FLOW
===============================================================================

PREREQUISITE
+-----------------------------------------------------------------------------+
|                     Phase 21 + Phase 22 Complete                            |
|              (mdbook for doc home, CSS for full feature set)                |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 1 (Foundation - Parallel)
+-----------------------------------------------------------------------------+
|  +--------------------------------+  +--------------------------------+     |
|  |           DT-1                 |  |           DT-2                 |     |
|  |    Test Harness Core           |  |    Demo Fixture Structure      |     |
|  |    - tests/demos/mod.rs        |  |    - examples/demos/ dirs      |     |
|  |    - RenderInfo struct         |  |    - basic.jsonl templates     |     |
|  |    - assert_* functions        |  |    - Mirror mdbook structure   |     |
|  |    - Hash + fallback verify    |  |                                |     |
|  +--------------------------------+  +--------------------------------+     |
+-----------------------------------------------------------------------------+
            |                                    |
            +------------------+-----------------+
                               v
WAVE 2 (Core Feature Demos - Parallel)
+-----------------------------------------------------------------------------+
|  +----------------------+  +----------------------+  +--------------------+ |
|  |        DT-3          |  |        DT-4          |  |        DT-5        | |
|  |   Sprite Demos       |  |   Animation Demos    |  |   Composition      | |
|  |   - basic            |  |   - basic_frames     |  |   Demos            | |
|  |   - named_palette    |  |   - frame_tags       |  |   - basic_layers   | |
|  |   - inline_palette   |  |   - timing           |  |   - blend_modes    | |
|  |   - metadata         |  |   - attachments      |  |   - positioning    | |
|  |   - transforms       |  |                      |  |   - fills          | |
|  |   Needs: DT-1, DT-2  |  |   Needs: DT-1, DT-2  |  |   Needs: DT-1,DT-2 | |
|  +----------------------+  +----------------------+  +--------------------+ |
+-----------------------------------------------------------------------------+
            |                        |                        |
            +------------------------+------------------------+
                               v
WAVE 3 (Export Demos - Parallel)
+-----------------------------------------------------------------------------+
|  +----------------------+  +----------------------+  +--------------------+ |
|  |        DT-6          |  |        DT-7          |  |        DT-8        | |
|  |   PNG/GIF Demos      |  |   Spritesheet Demos  |  |   Atlas Demos      | |
|  |   - png_basic        |  |   - horizontal       |  |   - godot          | |
|  |   - png_scaled       |  |   - grid             |  |   - unity          | |
|  |   - gif_animated     |  |   - padding          |  |   - libgdx         | |
|  |   Needs: DT-3        |  |   Needs: DT-4        |  |   Needs: DT-3      | |
|  +----------------------+  +----------------------+  +--------------------+ |
+-----------------------------------------------------------------------------+
            |                        |                        |
            +------------------------+------------------------+
                               v
WAVE 4 (CSS Feature Demos - After Phase 22)
+-----------------------------------------------------------------------------+
|  +----------------------+  +----------------------+  +--------------------+ |
|  |        DT-9          |  |        DT-10         |  |        DT-11       | |
|  |   CSS Color Demos    |  |   CSS Variable Demos |  |   CSS Timing Demos | |
|  |   - hex formats      |  |   - definition       |  |   - named          | |
|  |   - rgb/hsl/oklch    |  |   - var() resolve    |  |   - cubic-bezier   | |
|  |   - named colors     |  |   - fallbacks        |  |   - steps          | |
|  |   - color-mix        |  |   - chaining         |  |                    | |
|  |   Needs: DT-1, CSS   |  |   Needs: DT-1, CSS   |  |   Needs: DT-1, CSS | |
|  +----------------------+  +----------------------+  +--------------------+ |
|                                                                             |
|  +----------------------+  +----------------------+  +--------------------+ |
|  |        DT-12         |  |        DT-13         |  |        DT-14       | |
|  |   CSS Transform      |  |   CSS Blend Mode     |  |   @keyframes       | |
|  |   Demos              |  |   Demos              |  |   Demos            | |
|  |   - translate        |  |   - each mode        |  |   - percentage     | |
|  |   - rotate           |  |   - opacity          |  |   - from/to        | |
|  |   - scale            |  |                      |  |   - transforms     | |
|  |   Needs: DT-1, CSS   |  |   Needs: DT-5, CSS   |  |   Needs: DT-4, CSS | |
|  +----------------------+  +----------------------+  +--------------------+ |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 5 (Unsupported Documentation)
+-----------------------------------------------------------------------------+
|  +-----------------------------------------------------------------------+  |
|  |                              DT-15                                    |  |
|  |                    "Not Supported" Documentation                      |  |
|  |                    - format/css/unsupported.md                        |  |
|  |                    - Explicit list of CSS features NOT supported      |  |
|  |                    - Rationale for each exclusion                     |  |
|  |                    Needs: DT-9 through DT-14                          |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 6 (Doc Generator)
+-----------------------------------------------------------------------------+
|  +--------------------------------+  +--------------------------------+     |
|  |           DT-16                |  |           DT-17                |     |
|  |    Doc Generator Script        |  |    mdbook Integration          |     |
|  |    - Parse @demo annotations   |  |    - <!-- DEMOS --> markers    |     |
|  |    - Extract JSONL content     |  |    - WASM playground hooks     |     |
|  |    - Generate markdown         |  |    - SUMMARY.md updates        |     |
|  |    Needs: DT-3 through DT-14   |  |    Needs: DT-16                |     |
|  +--------------------------------+  +--------------------------------+     |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 7 (CI & Coverage)
+-----------------------------------------------------------------------------+
|  +--------------------------------+  +--------------------------------+     |
|  |           DT-18                |  |           DT-19                |     |
|  |    CI Integration              |  |    Coverage Enforcement        |     |
|  |    - Run demos in CI           |  |    - Feature coverage check    |     |
|  |    - Auto-regen docs           |  |    - Warn on missing demos     |     |
|  |    - Fail on regression        |  |    - Report uncovered features |     |
|  |    Needs: DT-17                |  |    Needs: DT-18                |     |
|  +--------------------------------+  +--------------------------------+     |
+-----------------------------------------------------------------------------+

===============================================================================

PARALLELIZATION SUMMARY:
+-----------------------------------------------------------------------------+
|  Wave 1: DT-1 + DT-2                          (2 tasks in parallel)         |
|  Wave 2: DT-3 + DT-4 + DT-5                   (3 tasks in parallel)         |
|  Wave 3: DT-6 + DT-7 + DT-8                   (3 tasks in parallel)         |
|  Wave 4: DT-9 + DT-10 + DT-11 + DT-12 + DT-13 + DT-14 (6 parallel, CSS req) |
|  Wave 5: DT-15                                (sequential - needs wave 4)   |
|  Wave 6: DT-16 → DT-17                        (sequential)                  |
|  Wave 7: DT-18 → DT-19                        (sequential)                  |
+-----------------------------------------------------------------------------+

CRITICAL PATH: DT-1 → DT-3 → DT-6 → DT-16 → DT-17 → DT-18 → DT-19

BEADS CREATION ORDER:
  1. DT-1, DT-2 (no deps besides phase prereqs)
  2. DT-3, DT-4, DT-5 (dep: DT-1, DT-2)
  3. DT-6, DT-7, DT-8 (dep: DT-3/DT-4)
  4. DT-9 through DT-14 (dep: DT-1, Phase 22 complete)
  5. DT-15 (dep: DT-9 through DT-14)
  6. DT-16 (dep: DT-3 through DT-14)
  7. DT-17 (dep: DT-16)
  8. DT-18 (dep: DT-17)
  9. DT-19 (dep: DT-18)
```

---

## Tasks

### Task DT-1: Test Harness Core

**Wave:** 1 (parallel with DT-2)

Create the Rust test harness with text-based verification utilities.

**Deliverables:**
- Create `tests/demos/mod.rs`:
  ```rust
  pub struct RenderInfo {
      pub width: u32,
      pub height: u32,
      pub frame_count: usize,
      pub palette_name: Option<String>,
      pub color_count: usize,
      pub sha256: String,
  }

  /// Verify sprite renders with expected dimensions
  pub fn assert_dimensions(jsonl: &str, sprite: &str, width: u32, height: u32);

  /// Verify output hash (with fallback to dimensions on platform mismatch)
  pub fn assert_output_hash(jsonl: &str, sprite: &str, expected_sha256: &str);

  /// Verify frame count for animations
  pub fn assert_frame_count(jsonl: &str, animation: &str, count: usize);

  /// Verify validation passes/fails
  pub fn assert_validates(jsonl: &str, should_pass: bool);

  /// Capture structured render info
  pub fn capture_render_info(jsonl: &str, sprite: &str) -> RenderInfo;
  ```

**Verification:**
```bash
cargo test demos::harness
```

**Dependencies:** Phase 21 complete

---

### Task DT-2: Demo Fixture Structure

**Wave:** 1 (parallel with DT-1)

Set up the demo fixture directory structure mirroring mdbook sections.

**Deliverables:**
- Create directory structure:
  ```
  examples/demos/
  ├── sprites/
  │   └── basic.jsonl         # Template with comments
  ├── animation/
  ├── composition/
  ├── exports/
  └── css/
      ├── colors/
      ├── variables/
      ├── timing/
      └── transforms/
  ```
- Each directory gets a README explaining the demo category
- Basic template files showing the expected structure

**Verification:**
```bash
ls -la examples/demos/*/
```

**Dependencies:** Phase 21 complete

---

### Task DT-3: Sprite Demos

**Wave:** 2 (parallel with DT-4, DT-5)

Create demo tests for all sprite features.

**Deliverables:**
- Create `tests/demos/sprites/`:
  - `basic.rs` - Minimal 3x3 sprite
  - `named_palette.rs` - Using palette references
  - `inline_palette.rs` - Inline color definitions
  - `metadata.rs` - Origin, hitboxes, attachments
  - `transforms.rs` - Flip, rotate, scale, recolor
- Each test uses `@demo` annotation for doc generation:
  ```rust
  /// @demo format/sprite#basic
  /// @title Basic Sprite
  /// @description The simplest valid sprite.
  #[test]
  fn basic_sprite() { ... }
  ```
- Corresponding JSONL files in `examples/demos/sprites/`

**Verification:**
```bash
cargo test demos::sprites
```

**Dependencies:** DT-1, DT-2

---

### Task DT-4: Animation Demos

**Wave:** 2 (parallel with DT-3, DT-5)

Create demo tests for animation features.

**Deliverables:**
- Create `tests/demos/animation/`:
  - `basic_frames.rs` - Simple frame sequence
  - `frame_tags.rs` - Named animation ranges
  - `timing.rs` - FPS and duration
  - `attachments.rs` - Attachment chains
- Corresponding JSONL files in `examples/demos/animation/`

**Verification:**
```bash
cargo test demos::animation
```

**Dependencies:** DT-1, DT-2

---

### Task DT-5: Composition Demos

**Wave:** 2 (parallel with DT-3, DT-4)

Create demo tests for composition features.

**Deliverables:**
- Create `tests/demos/composition/`:
  - `basic_layers.rs` - Simple sprite stacking
  - `blend_modes.rs` - Normal, multiply, screen, overlay
  - `positioning.rs` - Offsets and anchors
  - `fills.rs` - Background fills
- Corresponding JSONL files in `examples/demos/composition/`

**Verification:**
```bash
cargo test demos::composition
```

**Dependencies:** DT-1, DT-2

---

### Task DT-6: PNG/GIF Export Demos

**Wave:** 3 (parallel with DT-7, DT-8)

Create demo tests for basic export functionality.

**Deliverables:**
- Create `tests/demos/exports/`:
  - `png_basic.rs` - Simple PNG output
  - `png_scaled.rs` - Scaled output (2x, 4x, 8x)
  - `gif_animated.rs` - Animated GIF from animation

**Verification:**
```bash
cargo test demos::exports::png
cargo test demos::exports::gif
```

**Dependencies:** DT-3

---

### Task DT-7: Spritesheet Demos

**Wave:** 3 (parallel with DT-6, DT-8)

Create demo tests for spritesheet export.

**Deliverables:**
- Create `tests/demos/exports/`:
  - `spritesheet_horizontal.rs` - Horizontal strip
  - `spritesheet_grid.rs` - Grid layout
  - `spritesheet_padding.rs` - With spacing

**Verification:**
```bash
cargo test demos::exports::spritesheet
```

**Dependencies:** DT-4

---

### Task DT-8: Atlas Export Demos

**Wave:** 3 (parallel with DT-6, DT-7)

Create demo tests for game engine atlas export.

**Deliverables:**
- Create `tests/demos/exports/`:
  - `atlas_godot.rs` - Godot .tres format
  - `atlas_unity.rs` - Unity JSON format
  - `atlas_libgdx.rs` - LibGDX .atlas format

**Verification:**
```bash
cargo test demos::exports::atlas
```

**Dependencies:** DT-3

---

### Task DT-9: CSS Color Demos

**Wave:** 4 (parallel with DT-10 through DT-14, requires Phase 22)

Create demo tests for all supported CSS color formats.

**Deliverables:**
- Create `tests/demos/css/colors/`:
  - `hex.rs` - #rgb, #rrggbb, #rrggbbaa
  - `rgb.rs` - rgb(), rgba() variants
  - `hsl.rs` - hsl(), hsla() variants
  - `oklch.rs` - oklch() format
  - `hwb.rs` - hwb() format
  - `named.rs` - Named colors (red, gold, transparent)
  - `color_mix.rs` - color-mix() function

**Verification:**
```bash
cargo test demos::css::colors
```

**Dependencies:** DT-1, Phase 22 complete

---

### Task DT-10: CSS Variable Demos

**Wave:** 4 (parallel with DT-9, DT-11 through DT-14, requires Phase 22)

Create demo tests for CSS variable functionality.

**Deliverables:**
- Create `tests/demos/css/variables/`:
  - `definition.rs` - Defining --custom-properties
  - `resolution.rs` - Using var()
  - `fallbacks.rs` - var(--name, fallback)
  - `chaining.rs` - Variables referencing variables
  - `in_layers.rs` - Variables in composition opacity/blend

**Verification:**
```bash
cargo test demos::css::variables
```

**Dependencies:** DT-1, Phase 22 complete

---

### Task DT-11: CSS Timing Demos

**Wave:** 4 (parallel with DT-9, DT-10, DT-12 through DT-14, requires Phase 22)

Create demo tests for CSS timing functions.

**Deliverables:**
- Create `tests/demos/css/timing/`:
  - `named.rs` - ease, ease-in, ease-out, ease-in-out
  - `cubic_bezier.rs` - cubic-bezier(x1, y1, x2, y2)
  - `steps.rs` - steps(n), steps(n, jump-end)

**Verification:**
```bash
cargo test demos::css::timing
```

**Dependencies:** DT-1, Phase 22 complete

---

### Task DT-12: CSS Transform Demos

**Wave:** 4 (parallel, requires Phase 22)

Create demo tests for CSS transform functions.

**Deliverables:**
- Create `tests/demos/css/transforms/`:
  - `translate.rs` - translate(), translateX(), translateY()
  - `rotate.rs` - rotate(deg)
  - `scale.rs` - scale(), scaleX(), scaleY()
  - `flip.rs` - flip(x), flip(y) extension

**Verification:**
```bash
cargo test demos::css::transforms
```

**Dependencies:** DT-1, Phase 22 complete

---

### Task DT-13: CSS Blend Mode Demos

**Wave:** 4 (parallel, requires Phase 22)

Create demo tests for CSS blend modes in composition.

**Deliverables:**
- Create `tests/demos/css/blend/`:
  - `normal.rs` - Normal blending
  - `multiply.rs` - Multiply mode
  - `screen.rs` - Screen mode
  - `overlay.rs` - Overlay mode
  - `others.rs` - darken, lighten, color-dodge, color-burn, etc.

**Verification:**
```bash
cargo test demos::css::blend
```

**Dependencies:** DT-5, Phase 22 complete

---

### Task DT-14: @keyframes Demos

**Wave:** 4 (parallel, requires Phase 22)

Create demo tests for CSS @keyframes animation format.

**Deliverables:**
- Create `tests/demos/css/keyframes/`:
  - `percentage.rs` - 0%, 50%, 100% keyframes
  - `from_to.rs` - from/to aliases
  - `sprite_changes.rs` - Changing sprites at keyframes
  - `transforms.rs` - Animating transforms

**Verification:**
```bash
cargo test demos::css::keyframes
```

**Dependencies:** DT-4, Phase 22 complete

---

### Task DT-15: "Not Supported" Documentation

**Wave:** 5 (after Wave 4)

Document CSS features that are explicitly NOT supported.

**Deliverables:**
- Create `docs/book/src/format/css/unsupported.md`:
  - **Colors**: lab(), lch(), color(), currentColor, system colors, relative color syntax
  - **Variables**: :root scope, @property, variables in grids/names, calc()
  - **Timing**: linear(...) with stops, spring()
  - **Transforms**: skew(), matrix(), 3D, transform-origin
  - **Blend**: hue, saturation, color, luminosity, plus-lighter/darker
  - **Animation**: multiple animations, delay, direction, fill-mode, play-state
- Each exclusion includes rationale

**Verification:**
```bash
mdbook build docs/book
# Check unsupported.md renders correctly
```

**Dependencies:** DT-9 through DT-14

---

### Task DT-16: Doc Generator Script

**Wave:** 6 (after Wave 4 + 5)

Create script to extract demo metadata and generate markdown.

**Deliverables:**
- Create `scripts/generate-demos.sh` (or Rust binary):
  - Parse `tests/demos/**/*.rs` for `@demo`, `@title`, `@description` annotations
  - Extract corresponding JSONL from `examples/demos/`
  - Generate markdown fragments with:
    - Demo title and description
    - JSONL source in code block
    - CLI command example
    - WASM playground div
  - Output to temp location for integration

**Verification:**
```bash
./scripts/generate-demos.sh
# Check output in target/demos/*.md
```

**Dependencies:** DT-3 through DT-14

---

### Task DT-17: mdbook Integration

**Wave:** 6 (after DT-16)

Integrate generated demos into mdbook pages.

**Deliverables:**
- Add `<!-- DEMOS -->` markers to existing format/ and cli/ pages
- Update `scripts/generate-demos.sh` to insert demos at markers
- Add `format/css/` section to SUMMARY.md:
  ```markdown
  - [CSS Colors](format/css/colors.md)
  - [CSS Variables](format/css/variables.md)
  - [CSS Timing](format/css/timing.md)
  - [CSS Transforms](format/css/transforms.md)
  - [Unsupported CSS](format/css/unsupported.md)
  ```
- Add `demos/` landing page linking to all demo locations
- Update WASM demo JS to recognize new demo containers

**Verification:**
```bash
./scripts/generate-demos.sh
mdbook build docs/book
mdbook serve docs/book  # Manual verification
```

**Dependencies:** DT-16

---

### Task DT-18: CI Integration

**Wave:** 7 (after DT-17)

Add demo tests to CI pipeline.

**Deliverables:**
- Update `.github/workflows/ci.yml`:
  ```yaml
  - name: Run demo tests
    run: cargo test --test demos

  - name: Regenerate demo docs
    run: ./scripts/generate-demos.sh

  - name: Check demo docs are current
    run: git diff --exit-code docs/book/src/
  ```
- Demo test failures block merge
- Stale generated docs block merge

**Verification:**
```bash
# Manually verify CI runs pass
```

**Dependencies:** DT-17

---

### Task DT-19: Coverage Enforcement

**Wave:** 7 (after DT-18)

Add tooling to track and report demo coverage.

**Deliverables:**
- Create `scripts/demo-coverage.sh`:
  - Parse Feature Coverage Checklist from this doc
  - Cross-reference with existing demo tests
  - Report uncovered features
  - Exit non-zero if coverage below threshold
- Add to CI as informational (warn, don't fail initially)
- Create `docs/plan/demo-coverage.md` tracking coverage over time

**Verification:**
```bash
./scripts/demo-coverage.sh
# Should show current coverage percentage
```

**Dependencies:** DT-18

---

## Architecture

### Test Structure

```
tests/
├── demos/
│   ├── mod.rs                    # Test harness utilities
│   ├── sprites/
│   │   ├── mod.rs
│   │   ├── basic.rs
│   │   ├── named_palette.rs
│   │   └── ...
│   ├── animation/
│   ├── composition/
│   ├── exports/
│   └── css/
│       ├── colors/
│       ├── variables/
│       ├── timing/
│       ├── transforms/
│       ├── blend/
│       └── keyframes/

examples/demos/                   # Demo source files (JSONL)
├── sprites/
├── animation/
├── composition/
├── exports/
└── css/
```

### Verification Methods (No Binaries)

```rust
// tests/demos/mod.rs

pub struct RenderInfo {
    pub width: u32,
    pub height: u32,
    pub frame_count: usize,
    pub palette_name: Option<String>,
    pub color_count: usize,
    pub sha256: String,
}

/// Hash-based verification with fallback
pub fn assert_output_hash(jsonl: &str, sprite: &str, expected: &str) {
    let info = capture_render_info(jsonl, sprite);
    if info.sha256 != expected {
        // Fallback: verify dimensions and color count instead
        // (handles cross-platform PNG compression differences)
    }
}
```

### Demo Annotation Format

```rust
/// @demo format/sprite#basic
/// @title Basic Sprite
/// @description The simplest valid pixelsrc sprite.
/// @cli pxl render basic.jsonl -o sprite.png
#[test]
fn basic_sprite() {
    let jsonl = include_str!("../../../examples/demos/sprites/basic.jsonl");
    assert_validates(jsonl, true);
    assert_dimensions(jsonl, "square", 3, 3);
}
```

### mdbook Integration

Demos embed into existing pages via markers:

```markdown
# Sprite

[Existing specification content...]

<!-- DEMOS -->
<!-- Generated content inserted here by scripts/generate-demos.sh -->

## Basic Sprite

The simplest valid pixelsrc sprite.

<div class="demo-source">
```jsonl
{"palette":{...}}
{"sprite":{...}}
```
</div>

<div class="demo-container" data-demo="sprite-basic">
</div>

**CLI equivalent:**
```bash
pxl render basic.jsonl -o sprite.png
```
<!-- /DEMOS -->
```

---

## Feature Coverage Checklist

### Sprites
- [ ] Basic sprite (minimal valid example)
- [ ] Named palette reference
- [ ] Inline palette definition
- [ ] Multi-character color keys
- [ ] Transparency (. character)
- [ ] Origin point
- [ ] Collision boxes
- [ ] Attachment points

### Transforms
- [ ] Horizontal flip
- [ ] Vertical flip
- [ ] Rotation (90, 180, 270)
- [ ] Scale (2x, 3x, 4x)
- [ ] Recolor (palette swap)

### Animation
- [ ] Basic frame sequence
- [ ] Frame timing (FPS)
- [ ] Frame tags (named ranges)
- [ ] Looping modes
- [ ] Attachment chains
- [ ] Frame-specific metadata

### Composition
- [ ] Basic layer stacking
- [ ] Layer positioning (offsets)
- [ ] Blend modes (normal, multiply, screen, overlay)
- [ ] Background fills
- [ ] Multi-sprite scenes

### Palette Cycling
- [ ] Single color cycle
- [ ] Multiple cycle groups
- [ ] Cycle timing
- [ ] Ping-pong mode

### Imports
- [ ] PNG to JSONL conversion
- [ ] Palette detection
- [ ] Multi-frame import
- [ ] Transparent color handling

### Exports
- [ ] PNG (static)
- [ ] PNG (scaled)
- [ ] GIF (animated)
- [ ] Spritesheet (horizontal)
- [ ] Spritesheet (grid)
- [ ] Atlas (Godot)
- [ ] Atlas (Unity)
- [ ] Atlas (LibGDX)
- [ ] Atlas (Aseprite)

### Build System
- [ ] Basic pxl.toml configuration
- [ ] Multi-target builds
- [ ] Incremental rebuilds
- [ ] Watch mode
- [ ] Build variants

### CLI Commands
- [ ] render
- [ ] import
- [ ] validate
- [ ] fmt
- [ ] show
- [ ] explain
- [ ] diff
- [ ] suggest
- [ ] inline
- [ ] alias
- [ ] grid
- [ ] build
- [ ] new
- [ ] init
- [ ] analyze
- [ ] prime
- [ ] prompts
- [ ] palettes

---

## CSS Coverage (Phase 22)

CSS integration introduces complexity that must be explicitly documented. Each supported feature needs a demo; unsupported features need explicit documentation.

### CSS Colors

**Supported:**
- [ ] `#rgb` / `#rrggbb` / `#rrggbbaa` (hex)
- [ ] `rgb(r, g, b)` / `rgb(r g b)`
- [ ] `rgb(r, g, b, a)` / `rgb(r g b / a)`
- [ ] `hsl(h, s%, l%)` / `hsl(h s% l%)`
- [ ] `hsl(h, s%, l%, a)` / `hsl(h s% l% / a)`
- [ ] `oklch(l% c h)` / `oklch(l% c h / a)`
- [ ] `hwb(h w% b%)` / `hwb(h w% b% / a)`
- [ ] Named colors (`red`, `gold`, `transparent`)
- [ ] `color-mix(in oklch, color1 %, color2)`
- [ ] `color-mix(in srgb, ...)` / `color-mix(in hsl, ...)`

**NOT Supported:**
- [ ] `lab()`, `lch()` (use oklch instead)
- [ ] `color()` function
- [ ] `currentColor` keyword
- [ ] System colors (`Canvas`, `ButtonText`)
- [ ] Relative color syntax `rgb(from var(--x) r g b)`

### CSS Variables

**Supported:**
- [ ] `--custom-property` definitions in palette colors
- [ ] `var(--name)` resolution
- [ ] `var(--name, fallback)` with fallback values
- [ ] Chained variables `var(--a)` where `--a: var(--b)`
- [ ] Variables in layer opacity
- [ ] Variables in layer blend mode

**NOT Supported:**
- [ ] Global `:root` scope (file-scoped only)
- [ ] `@property` registered properties
- [ ] Variables in grid strings
- [ ] Variables in sprite names
- [ ] Computed values `calc(var(--x) + 10)`

### CSS Timing Functions

**Supported:**
- [ ] `linear`
- [ ] `ease`, `ease-in`, `ease-out`, `ease-in-out`
- [ ] `cubic-bezier(x1, y1, x2, y2)`
- [ ] `steps(n)` / `steps(n, jump-end)` / `steps(n, jump-start)`

**NOT Supported:**
- [ ] `linear(...)` with stops (CSS linear easing)
- [ ] `spring()` (not standard CSS)

### CSS Transforms

**Supported:**
- [ ] `translate(x, y)` / `translateX(x)` / `translateY(y)`
- [ ] `rotate(deg)` / `rotate(90deg)`
- [ ] `scale(n)` / `scale(x, y)` / `scaleX(x)` / `scaleY(y)`
- [ ] `flip(x)` / `flip(y)` (extension)

**NOT Supported:**
- [ ] `skew()`, `skewX()`, `skewY()`
- [ ] `matrix()`, `matrix3d()`
- [ ] 3D transforms (`perspective`, `rotateX`, etc.)
- [ ] `transform-origin` (always center)

### CSS Blend Modes

**Supported:**
- [ ] `normal`
- [ ] `multiply`
- [ ] `screen`
- [ ] `overlay`
- [ ] `darken` / `lighten`
- [ ] `color-dodge` / `color-burn`
- [ ] `hard-light` / `soft-light`
- [ ] `difference` / `exclusion`

**NOT Supported:**
- [ ] `hue`, `saturation`, `color`, `luminosity`
- [ ] `plus-lighter`, `plus-darker`

### @keyframes (Animation)

**Supported:**
- [ ] Percentage keyframes `"0%": {...}, "100%": {...}`
- [ ] Sprite changes at keyframes
- [ ] Opacity changes at keyframes
- [ ] Transform changes at keyframes
- [ ] `from` / `to` aliases for 0%/100%

**NOT Supported:**
- [ ] Multiple animations on same element
- [ ] `animation-delay` (start immediately)
- [ ] `animation-direction: alternate`
- [ ] `animation-fill-mode`
- [ ] `animation-play-state`

---

## Coordination with Other Work

### Refactor (REF-1)
The composition.rs breakup creates a cleaner structure for blend mode demos. When `src/composition/blend.rs` exists, blend mode demos can reference it directly. Demo tests don't block on the refactor.

### CSS Integration (Phase 22)
CSS demos (Wave 4) depend on CSS implementation being complete. Key coordination:
- CSS-4 (Color Tests) aligns with DT-9
- CSS-7 (Variables Tests) aligns with DT-10
- CSS-18 (Integration Tests) may share fixtures with demos
- CSS-20 (Doc Finalization) should integrate DT-17 output

---

## Related Documents

- [refactor.md](refactor.md) - Code structure improvements (REF-1)
- [css.md](css.md) - CSS integration plan (Phase 22)
- [mdbook.md](mdbook.md) - Documentation structure (Phase 21)
