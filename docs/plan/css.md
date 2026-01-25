---
phase: 22
title: CSS Integration
---

# Phase 22: CSS Integration

Adopt CSS syntax and semantics for colors, variables, easing, and styling.

**Personas:** All (GenAI benefits everyone)

**Status:** Complete

**Depends on:** Phase 21 (mdbook Documentation)

**Related:**
- [VISION.md](../VISION.md) - Tenets #3 and #5
- [refactor.md](./refactor.md) - composition.rs breakup (REF-1)

---

## Overview

| Component | Purpose |
|-----------|---------|
| CSS Colors | Replace hex-only with full CSS color syntax |
| CSS Variables | `--custom-properties` and `var()` resolution |
| CSS Easing | `cubic-bezier()`, `steps()`, named functions |
| CSS Blend Modes | Standardize composition layer blending |
| CSS Keyframes | Percentage-based animation definitions |
| CSS Transforms | `translate()`, `rotate()`, `scale()` |

---

## Strategic Vision

### The Problem

Pixelsrc reinvents concepts CSS has solved for decades:
- **Colors**: Hex-only vs CSS `rgb()`, `hsl()`, `oklch()`, `color-mix()`
- **Easing**: Custom `Interpolation` enum vs CSS `cubic-bezier()`, `steps()`
- **Animations**: Frame arrays vs CSS `@keyframes` percentages
- **Transforms**: Custom flags vs CSS `transform` functions

### Why This Matters

**GenAI familiarity is a force multiplier.** LLMs have seen millions of CSS examples. By adopting CSS:

1. **Lower cognitive load** - GenAI knows `hsl(30, 45%, 85%)` natively
2. **Proven semantics** - Battle-tested specifications
3. **Future-proof** - New CSS features come free
4. **Human-familiar** - Web developers know the syntax

### Alignment with Vision

- **Tenet #3**: "Don't reinvent the wheel"
- **Tenet #5**: "GenAI-first"

---

## Task Dependency Diagram

```
                           CSS INTEGRATION TASK FLOW
═══════════════════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Phase 21 Complete                                 │
│                      (mdbook for documentation home)                        │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 1 (Foundation - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │         CSS-1                  │  │         CSS-2                  │    │
│  │    lightningcss Setup          │  │    ColorError Extension        │    │
│  │    - Add Cargo dependency      │  │    - CssParse variant          │    │
│  │    - Verify builds             │  │    - Display impl              │    │
│  │    - Feature flags if needed   │  │    - From<lightningcss::Error> │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                    │
            └─────────────────┬──────────────────┘
                              ▼
WAVE 2 (CSS Colors)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-3                                    │    │
│  │               CSS Color Parsing                                     │    │
│  │               - Extend parse_color() in src/color.rs                │    │
│  │               - rgb(), hsl(), oklch(), hwb()                        │    │
│  │               - Named colors (gold, transparent)                    │    │
│  │               - Hex fast-path preserved                             │    │
│  │               Needs: CSS-1, CSS-2                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-4                                    │    │
│  │               Color Tests & Docs                                    │    │
│  │               - Unit tests for all color formats                    │    │
│  │               - Update docs/book/src/reference/colors.md            │    │
│  │               - Update primer with CSS color examples               │    │
│  │               Needs: CSS-3                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 3 (CSS Variables - Sequential)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-5                                    │    │
│  │               Variable Registry Core                                │    │
│  │               - Create src/variables.rs                             │    │
│  │               - VariableRegistry struct                             │    │
│  │               - define() and resolve() methods                      │    │
│  │               - Circular reference detection                        │    │
│  │               - Fallback value support                              │    │
│  │               Needs: CSS-3                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-6                                    │    │
│  │               Parser Integration                                    │    │
│  │               - Two-pass palette parsing                            │    │
│  │               - Lenient mode (magenta fallback)                     │    │
│  │               - Strict mode (error on undefined)                    │    │
│  │               - Integration with PaletteRegistry                    │    │
│  │               Needs: CSS-5                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-7                                    │    │
│  │               Variables Tests & Docs                                │    │
│  │               - Unit tests for resolution                           │    │
│  │               - Tests for circular refs, fallbacks                  │    │
│  │               - Create docs/book/src/reference/css-variables.md     │    │
│  │               - Update palette docs with var() examples             │    │
│  │               Needs: CSS-6                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 4 (Easing & Composition - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────────────────────────┐  ┌────────────────────────────────┐  │
│  │           CSS-8                   │  │           CSS-9                │  │
│  │    CSS Timing Functions           │  │    Composition Variables       │  │
│  │    - Add Steps variant            │  │    - var() in layer opacity    │  │
│  │    - parse_timing_function()      │  │    - var() in layer blend      │  │
│  │    - cubic-bezier() parsing       │  │    - Resolve before render     │  │
│  │    - steps() parsing              │  │    Needs: CSS-6                │  │
│  │    Needs: CSS-3                   │  │                                │  │
│  └───────────────────────────────────┘  └────────────────────────────────┘  │
│                                                                             │
│  ┌───────────────────────────────────┐  ┌────────────────────────────────┐  │
│  │           CSS-10                  │  │           CSS-11               │  │
│  │    Easing Tests & Docs            │  │    Composition Docs            │  │
│  │    - Tests for all functions      │  │    - Blend mode reference      │  │
│  │    - steps() semantic docs        │  │    - Variable examples         │  │
│  │    - timing-functions.md          │  │    - Update composition.md     │  │
│  │    Needs: CSS-8                   │  │    Needs: CSS-9                │  │
│  └───────────────────────────────────┘  └────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                     │
            └─────────────────┬───────────────────┘
                              ▼
WAVE 5 (Advanced - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐     │
│  │      CSS-12        │  │      CSS-13        │  │      CSS-14        │     │
│  │   color-mix()      │  │   @keyframes       │  │   CSS Transforms   │     │
│  │   - Parse function │  │   - New Animation  │  │   - translate()    │     │
│  │   - oklch interp   │  │     model          │  │   - rotate()       │     │
│  │   - Percentage     │  │   - % keyframes    │  │   - scale()        │     │
│  │     blending       │  │   - Parser update  │  │   - Parser         │     │
│  │   Needs: CSS-6     │  │   Needs: CSS-8     │  │   Needs: CSS-8     │     │
│  └────────────────────┘  └────────────────────┘  └────────────────────┘     │
│                                                                             │
│  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐     │
│  │      CSS-15        │  │      CSS-16        │  │      CSS-17        │     │
│  │   color-mix Docs   │  │   @keyframes Docs  │  │   Transform Docs   │     │
│  │   Needs: CSS-12    │  │   Needs: CSS-13    │  │   Needs: CSS-14    │     │
│  └────────────────────┘  └────────────────────┘  └────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘
            │                       │                       │
            └───────────────────────┴───────────────────────┘
                              │
                              ▼
WAVE 6 (Integration & Polish)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-18                                   │    │
│  │                    Integration Test Suite                           │    │
│  │                    - End-to-end CSS color tests                     │    │
│  │                    - Variable resolution tests                      │    │
│  │                    - Animation with keyframes tests                 │    │
│  │                    - Composition with vars tests                    │    │
│  │                    Needs: CSS-12, CSS-13, CSS-14                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-19                                   │    │
│  │                    Example Updates                                  │    │
│  │                    - Update examples/*.pxl with CSS syntax          │    │
│  │                    - Update primer with CSS-first examples          │    │
│  │                    - Update system prompts                          │    │
│  │                    Needs: CSS-18                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            CSS-20                                   │    │
│  │                    Documentation Finalization                       │    │
│  │                    - Update mdbook SUMMARY.md                       │    │
│  │                    - Cross-link all CSS docs                        │    │
│  │                    - Update format spec                             │    │
│  │                    - prime output updates                           │    │
│  │                    Needs: CSS-19                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Wave 1: CSS-1 + CSS-2                        (2 tasks in parallel)         │
│  Wave 2: CSS-3 → CSS-4                        (sequential)                  │
│  Wave 3: CSS-5 → CSS-6 → CSS-7                (sequential - core feature)   │
│  Wave 4: CSS-8 + CSS-9                        (2 parallel)                  │
│          CSS-10 + CSS-11 (after their deps)   (2 parallel)                  │
│  Wave 5: CSS-12 + CSS-13 + CSS-14             (3 parallel)                  │
│          CSS-15 + CSS-16 + CSS-17             (3 parallel docs)             │
│  Wave 6: CSS-18 → CSS-19 → CSS-20             (sequential)                  │
└─────────────────────────────────────────────────────────────────────────────┘

CRITICAL PATH: CSS-1 → CSS-3 → CSS-5 → CSS-6 → CSS-8 → CSS-13 → CSS-18 → CSS-20

BEADS CREATION ORDER:
  1. CSS-1, CSS-2 (no deps)
  2. CSS-3 (dep: CSS-1, CSS-2)
  3. CSS-4 (dep: CSS-3)
  4. CSS-5 (dep: CSS-3)
  5. CSS-6 (dep: CSS-5)
  6. CSS-7 (dep: CSS-6)
  7. CSS-8 (dep: CSS-3), CSS-9 (dep: CSS-6)
  8. CSS-10 (dep: CSS-8), CSS-11 (dep: CSS-9)
  9. CSS-12 (dep: CSS-6), CSS-13 (dep: CSS-8), CSS-14 (dep: CSS-8)
 10. CSS-15 (dep: CSS-12), CSS-16 (dep: CSS-13), CSS-17 (dep: CSS-14)
 11. CSS-18 (dep: CSS-12, CSS-13, CSS-14)
 12. CSS-19 (dep: CSS-18)
 13. CSS-20 (dep: CSS-19)
```

---

## Tasks

### Task CSS-1: lightningcss Setup

**Wave:** 1 (parallel with CSS-2)

Add lightningcss dependency and verify it builds.

**Deliverables:**
- Update `Cargo.toml`:
  ```toml
  [dependencies]
  lightningcss = "1.0"
  ```
- Verify `cargo build` succeeds
- Add feature flags if needed for WASM compatibility

**Verification:**
```bash
cargo build
cargo build --target wasm32-unknown-unknown
```

**Dependencies:** Phase 21 complete

---

### Task CSS-2: ColorError Extension

**Wave:** 1 (parallel with CSS-1)

Extend ColorError to handle CSS parsing errors.

**Deliverables:**
- Update `src/color.rs`:
  ```rust
  pub enum ColorError {
      Empty,
      MissingHash,
      InvalidLength(usize),
      InvalidHex(char),
      CssParse(String),  // NEW
  }

  impl From<lightningcss::error::Error<'_>> for ColorError {
      fn from(e: lightningcss::error::Error) -> Self {
          ColorError::CssParse(e.to_string())
      }
  }
  ```

**Verification:**
```bash
cargo test color
```

**Dependencies:** Phase 21 complete

---

### Task CSS-3: CSS Color Parsing

**Wave:** 2 (after CSS-1, CSS-2)

Extend parse_color() to handle full CSS color syntax.

**Deliverables:**
- Update `src/color.rs`:
  ```rust
  pub fn parse_color(s: &str) -> Result<Rgba<u8>, ColorError> {
      // Fast path for hex colors
      if s.starts_with('#') {
          return parse_hex_color(s);
      }

      // CSS color parsing via lightningcss
      let css_color = CssColor::parse_string(s)?;
      Ok(css_color_to_rgba(&css_color))
  }

  fn css_color_to_rgba(color: &CssColor) -> Rgba<u8> {
      // Convert lightningcss color to image::Rgba
  }
  ```

**Supported formats:**
- `rgb(255, 0, 0)`, `rgb(255 0 0)`, `rgb(255 0 0 / 0.5)`
- `hsl(0, 100%, 50%)`, `hsl(0deg 100% 50%)`
- `oklch(70% 0.15 30)`
- `hwb(0 0% 0%)`
- Named: `red`, `gold`, `transparent`

**Verification:**
```bash
cargo test color::parse
# Test: rgb(255, 0, 0) → Rgba([255, 0, 0, 255])
# Test: hsl(0, 100%, 50%) → Rgba([255, 0, 0, 255])
# Test: transparent → Rgba([0, 0, 0, 0])
```

**Dependencies:** CSS-1, CSS-2

---

### Task CSS-4: Color Tests & Docs

**Wave:** 2 (after CSS-3)

Comprehensive tests and documentation for CSS colors.

**Deliverables:**
- Add tests to `src/color.rs`:
  ```rust
  #[test]
  fn test_css_rgb() { ... }
  #[test]
  fn test_css_hsl() { ... }
  #[test]
  fn test_css_oklch() { ... }
  #[test]
  fn test_css_named() { ... }
  ```
- Update `docs/book/src/reference/colors.md`:
  - All supported formats with examples
  - Color space explanations
  - When to use oklch vs hsl
- Update primer/system prompts with CSS examples

**Verification:**
```bash
cargo test color
mdbook build docs/book
```

**Dependencies:** CSS-3

---

### Task CSS-5: Variable Registry Core

**Wave:** 3 (after CSS-3)

Create VariableRegistry for CSS custom property resolution.

**Deliverables:**
- New file `src/variables.rs`:
  ```rust
  pub struct VariableRegistry {
      variables: HashMap<String, String>,
  }

  pub enum VariableError {
      Undefined(String),
      Circular(String),
      Syntax(String),
  }

  impl VariableRegistry {
      pub fn new() -> Self;
      pub fn define(&mut self, name: &str, value: &str);
      pub fn resolve(&self, value: &str, strict: bool) -> Result<String, VariableError>;
  }
  ```
- Circular reference detection via HashSet tracking
- Fallback support: `var(--missing, #fff)`
- Update `src/lib.rs` to add `pub mod variables;`

**Verification:**
```bash
cargo test variables
# Test: basic resolution
# Test: chained variables
# Test: circular detection
# Test: fallback values
```

**Dependencies:** CSS-3

---

### Task CSS-6: Parser Integration

**Wave:** 3 (after CSS-5)

Integrate VariableRegistry into palette parsing.

**Deliverables:**
- Update `src/parser.rs` (or relevant parsing code):
  ```rust
  fn parse_palette(&mut self, json: &Value) -> Result<Palette, ParseError> {
      let mut var_registry = VariableRegistry::new();
      let colors = json["colors"].as_object()?;

      // Pass 1: Collect --variable definitions
      for (key, value) in colors {
          if key.starts_with("--") {
              var_registry.define(key, value.as_str()?);
          }
      }

      // Pass 2: Resolve var() references
      let mut resolved_colors = HashMap::new();
      for (key, value) in colors {
          if key.starts_with('{') {
              let resolved = var_registry.resolve(value.as_str()?, self.strict)?;
              resolved_colors.insert(key.clone(), resolved);
          }
      }

      Ok(Palette { name, colors: resolved_colors })
  }
  ```
- Lenient mode: undefined vars → `#FF00FF` + warning
- Strict mode: undefined vars → error

**Verification:**
```bash
cargo test parser::palette
pxl render tests/fixtures/css/variables.pxl
pxl render tests/fixtures/css/variables.pxl --strict
```

**Dependencies:** CSS-5

---

### Task CSS-7: Variables Tests & Docs

**Wave:** 3 (after CSS-6)

Tests and documentation for CSS variables.

**Deliverables:**
- Integration tests for variable resolution
- Create `docs/book/src/reference/css-variables.md`:
  - Syntax reference
  - Scoping rules (file-scoped)
  - Error handling (strict vs lenient)
  - Examples with common patterns
- Update `docs/book/src/format/palette.md` with variable examples

**Verification:**
```bash
cargo test variables
mdbook build docs/book
```

**Dependencies:** CSS-6

---

### Task CSS-8: CSS Timing Functions

**Wave:** 4 (parallel with CSS-9)

Add CSS timing function parsing to motion.rs.

**Deliverables:**
- Update `src/motion.rs`:
  ```rust
  pub enum Interpolation {
      Linear,
      EaseIn,
      EaseOut,
      EaseInOut,
      Bounce,
      Elastic,
      Bezier { p1: (f64, f64), p2: (f64, f64) },
      Steps { count: u32, position: StepPosition },  // NEW
  }

  pub enum StepPosition {
      JumpStart,
      JumpEnd,
      JumpNone,
      JumpBoth,
  }

  pub fn parse_timing_function(css: &str) -> Result<Interpolation, ParseError>;
  ```
- Parse: `cubic-bezier(x1, y1, x2, y2)`
- Parse: `steps(n, jump-end)`
- Parse: named functions (`ease`, `ease-in-out`, etc.)

**Verification:**
```bash
cargo test motion::parse_timing
```

**Dependencies:** CSS-3

---

### Task CSS-9: Composition Variables

**Wave:** 4 (parallel with CSS-8)

Enable var() in composition layer properties.

**Deliverables:**
- Update composition rendering to resolve variables:
  ```rust
  // Before applying opacity/blend, resolve any var() references
  let opacity = match layer.opacity {
      Some(val) if val.contains("var(") => {
          let resolved = var_registry.resolve(&val, strict)?;
          resolved.parse::<f64>()?
      }
      Some(val) => val.parse::<f64>()?,
      None => 1.0,
  };
  ```
- Support `var()` in:
  - `opacity` field
  - `blend` field

**Verification:**
```bash
cargo test composition::variables
pxl render tests/fixtures/css/composition_vars.pxl
```

**Dependencies:** CSS-6

---

### Task CSS-10: Easing Tests & Docs

**Wave:** 4 (after CSS-8)

Tests and documentation for CSS timing functions.

**Deliverables:**
- Tests for all timing functions
- Create `docs/book/src/reference/timing-functions.md`:
  - Named functions reference
  - cubic-bezier() explanation
  - steps() semantics for pixel art (important clarification)
  - When to use each type
- ASCII diagram explaining steps() applies to tweened properties, not frame selection

**Verification:**
```bash
cargo test motion
mdbook build docs/book
```

**Dependencies:** CSS-8

---

### Task CSS-11: Composition Docs

**Wave:** 4 (after CSS-9)

Documentation for composition styling.

**Deliverables:**
- Update `docs/book/src/format/composition.md`:
  - Blend mode reference (matching CSS mix-blend-mode)
  - Opacity usage
  - Variable examples in layers
- Add blend mode visual examples

**Verification:**
```bash
mdbook build docs/book
```

**Dependencies:** CSS-9

---

### Task CSS-12: color-mix() Support

**Wave:** 5 (parallel with CSS-13, CSS-14)

Implement color-mix() function for computed colors.

**Deliverables:**
- Update color parsing to handle `color-mix()`:
  ```rust
  // color-mix(in oklch, var(--primary) 70%, black)
  pub fn resolve_color_mix(expr: &str, var_registry: &VariableRegistry) -> Result<Rgba<u8>, ColorError>;
  ```
- Resolve variables first, then parse color-mix
- Support `in oklch`, `in srgb`, `in hsl` color spaces
- Percentage blending

**Verification:**
```bash
cargo test color::color_mix
pxl render tests/fixtures/css/color_mix.pxl
```

**Dependencies:** CSS-6

---

### Task CSS-13: @keyframes Animation Model

**Wave:** 5 (parallel with CSS-12, CSS-14)

Replace frame array with percentage-based keyframes.

**Deliverables:**
- Update `src/models.rs`:
  ```rust
  pub struct Animation {
      pub name: String,
      pub keyframes: HashMap<String, Keyframe>,  // "0%", "50%", "100%"
      pub duration: String,                       // "500ms"
      pub timing_function: Option<String>,
  }

  pub struct Keyframe {
      pub sprite: Option<String>,
      pub transform: Option<String>,
      pub opacity: Option<f64>,
      pub offset: Option<[i32; 2]>,
  }
  ```
- Update parser for new format
- Update animation renderer

**Note:** This is a breaking change. Old `frames` format removed entirely.

**Verification:**
```bash
cargo test animation::keyframes
pxl render tests/fixtures/css/keyframes.pxl --gif
```

**Dependencies:** CSS-8

---

### Task CSS-14: CSS Transforms

**Wave:** 5 (parallel with CSS-12, CSS-13)

Parse and apply CSS transform functions.

**Deliverables:**
- New file or section for transform parsing:
  ```rust
  pub fn parse_transform(css: &str) -> Result<Transform, ParseError>;

  pub struct Transform {
      pub translate: Option<(i32, i32)>,
      pub rotate: Option<f64>,        // degrees
      pub scale: Option<(f64, f64)>,
      pub flip_x: bool,
      pub flip_y: bool,
  }
  ```
- Support: `translate(x, y)`, `rotate(deg)`, `scale(n)`, `flip(x)`, `flip(y)`
- Apply transforms during rendering

**Verification:**
```bash
cargo test transforms::css
pxl render tests/fixtures/css/transforms.pxl
```

**Dependencies:** CSS-8

---

### Task CSS-15: color-mix() Docs

**Wave:** 5 (after CSS-12)

Documentation for color-mix() function.

**Deliverables:**
- Update `docs/book/src/reference/colors.md` with color-mix section
- Examples of shadow generation, highlights, etc.

**Dependencies:** CSS-12

---

### Task CSS-16: @keyframes Docs

**Wave:** 5 (after CSS-13)

Documentation for keyframes animation format.

**Deliverables:**
- Update `docs/book/src/format/animation.md`:
  - New keyframes syntax
  - Migration from old frames format
  - Examples
- Update all animation examples in docs

**Dependencies:** CSS-13

---

### Task CSS-17: Transform Docs

**Wave:** 5 (after CSS-14)

Documentation for CSS transforms.

**Deliverables:**
- Create `docs/book/src/reference/transforms.md`:
  - All supported functions
  - Pixel art considerations (nearest-neighbor scaling)
  - Examples

**Dependencies:** CSS-14

---

### Task CSS-18: Integration Test Suite

**Wave:** 6 (after CSS-12, CSS-13, CSS-14)

Comprehensive integration tests for all CSS features.

**Deliverables:**
- Create `tests/css_integration_tests.rs`:
  - End-to-end color parsing tests
  - Variable resolution across palettes
  - Keyframe animations with transforms
  - Compositions with variable blend/opacity
- Fixture files in `tests/fixtures/css/`

**Verification:**
```bash
cargo test --test css_integration_tests
```

**Dependencies:** CSS-12, CSS-13, CSS-14

---

### Task CSS-19: Example Updates

**Wave:** 6 (after CSS-18)

Update all examples to use CSS syntax.

**Deliverables:**
- Update `examples/*.pxl` files:
  - Use `hsl()` or `oklch()` instead of hex where appropriate
  - Add variable examples
  - Update animations to keyframe format
- Update primer documents
- Update system prompts to suggest CSS syntax

**Verification:**
```bash
pxl render examples/*.pxl
pxl validate examples/
```

**Dependencies:** CSS-18

---

### Task CSS-20: Documentation Finalization

**Wave:** 6 (after CSS-19)

Final documentation pass and cross-linking.

**Deliverables:**
- Update `docs/book/src/SUMMARY.md` with new pages:
  ```markdown
  - [CSS Variables](reference/css-variables.md)
  - [Timing Functions](reference/timing-functions.md)
  - [Transforms](reference/transforms.md)
  ```
- Update `docs/spec/format.md` with CSS syntax
- Update `pxl prime` output
- Cross-link all CSS-related docs

**Verification:**
```bash
mdbook build docs/book
pxl prime --topic colors
pxl prime --topic variables
```

**Dependencies:** CSS-19

---

## Technical Decisions

### Parser Library: lightningcss

**Chosen**: `lightningcss = "1.0"`

**Why**:
- Complete CSS parser in Rust
- Production-proven (Parcel bundler)
- Handles colors, functions, custom properties
- Active maintenance

### Variable Scope: File-Scoped, Parse-Time

Variables are scoped to the parse unit and resolved during parsing.

**Rationale**:
- Matches existing Registry pattern
- Simple mental model
- No cross-file tracking
- Clear error messages

### Breaking Changes: Accepted

The @keyframes format replaces frames format entirely.

**Rationale**:
- No external users yet
- Documentation updates sufficient
- Cleaner format without legacy code

---

## Success Metrics

1. **LLMs generate CSS syntax naturally** - GenAI uses `hsl()` without prompting
2. **Variables work intuitively** - `var(--primary)` resolves with clear errors
3. **steps() understood** - Users know it's for property tweening
4. **Consistent styling** - blend, opacity, transform use same syntax everywhere
5. **Docs emphasize CSS** - All examples show CSS syntax first

---

## Related Documents

- [VISION.md](../VISION.md) - Core design principles
- [format.md](../spec/format.md) - Format specification
- [refactor.md](./refactor.md) - composition.rs breakup
- [motion.rs](../../src/motion.rs) - Current easing
- [color.rs](../../src/color.rs) - Current color parsing
- [registry.rs](../../src/registry.rs) - Registry pattern
