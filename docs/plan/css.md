# CSS Integration Strategy

**Status**: 🟡 Planning
**Branch**: `claude/css-aliases-enhancement-dxxWz`
**Goal**: Adopt CSS syntax and semantics instead of reinventing styling primitives

## Strategic Vision

### The Problem

Pixelsrc is currently reinventing concepts that CSS has solved for decades:
- **Colors**: Custom hex-only palette format vs CSS `rgb()`, `hsl()`, `oklch()`, named colors, `--custom-properties`
- **Easing**: Custom `Interpolation` enum vs CSS `cubic-bezier()`, `ease-in`, `steps()`
- **Animations**: Custom timing system vs CSS `@keyframes`, `animation-duration`, `animation-timing-function`
- **Transforms**: (Future) vs CSS `transform`, `rotate()`, `scale()`, `matrix()`
- **Blending**: Custom `blend` field vs CSS `mix-blend-mode`
- **Opacity**: Already matches CSS `opacity` (1.0 = opaque)

### Why This Matters

**GenAI familiarity is a force multiplier.** LLMs have seen millions of CSS examples during training. By adopting CSS syntax:

1. **Lower cognitive load** - GenAI already knows `hsl(30, 45%, 85%)` and `cubic-bezier(0.68, -0.55, 0.265, 1.55)`
2. **Proven semantics** - CSS color spaces, easing functions, and transforms are battle-tested
3. **Future-proof** - As CSS evolves (e.g., `oklch()`, `color-mix()`), we get new features for free
4. **Familiar to humans** - Web developers already know this syntax

### Alignment with Vision

From `docs/VISION.md`:
- **Tenet #3**: "Don't reinvent the wheel" - Use proven standards instead of custom syntax
- **Tenet #5**: "GenAI-first" - Leverage existing LLM knowledge instead of teaching new formats

## Implementation Paths

### 1. CSS Color Functions (Phase 1)

**Impact**: High | **Risk**: Low | **Effort**: ~1 week

Replace hex-only palette colors with full CSS color syntax.

#### Current State
```jsonl
{"type": "palette", "name": "hero", "colors": {
  "{skin}": "#FFD5B4",
  "{shadow}": "#000000"
}}
```

#### Target State
```jsonl
{"type": "palette", "name": "hero", "colors": {
  "{skin}": "hsl(30, 45%, 85%)",
  "{shadow}": "oklch(0% 0 0 / 0.3)",
  "{primary}": "rgb(255 215 0)",
  "{highlight}": "#FFFACD"
}}
```

#### Supported CSS Color Formats
- Hex: `#FFD700`, `#FFD70080` (with alpha)
- RGB: `rgb(255, 215, 0)`, `rgb(255 215 0 / 0.5)`
- HSL: `hsl(51, 100%, 50%)`, `hsl(51deg 100% 50%)`
- OKLCH: `oklch(84% 0.15 103)` (perceptually uniform)
- Named colors: `gold`, `transparent`, `currentColor`

#### Implementation
```rust
// Use lightningcss for parsing
use lightningcss::values::color::CssColor;

pub fn parse_css_color(css_string: &str) -> Result<[u8; 4], ColorParseError> {
    let color = CssColor::parse_string(css_string)?;
    Ok(color.to_rgba())
}
```

**Dependencies**:
- `lightningcss = "1.0"` - Full CSS parser (used by Parcel, production-proven)
- OR `csscolorparser = "0.6"` - Simpler, color-only parser

**Backwards Compatibility**: Continue accepting hex colors as before.

---

### 2. CSS Custom Properties (Phase 2)

**Impact**: High | **Risk**: Medium | **Effort**: ~1 week

Enable CSS variable definitions and resolution in palettes.

#### Target State
```jsonl
{"type": "palette", "name": "theme", "colors": {
  "--primary": "oklch(60% 0.2 250)",
  "--secondary": "oklch(70% 0.15 180)",
  "--accent": "hsl(15, 80%, 60%)",
  "{hero-shirt}": "var(--primary)",
  "{hero-pants}": "var(--secondary)",
  "{hero-shirt-shadow}": "color-mix(in oklch, var(--primary) 80%, black)"
}}
```

#### Benefits
- **Theme variations**: Override `--primary` to create color schemes
- **Palette composition**: Palettes can reference other palettes' variables
- **Computed colors**: `color-mix()`, `color-contrast()` for automatic shading

#### Implementation Challenges
- **Variable scope**: Do variables scope to palette, file, or globally?
- **Inheritance**: Can palettes inherit from base palettes?
- **Resolution order**: When are variables resolved (parse time vs render time)?

**Recommended Scope**: Variables are file-scoped, resolved at parse time.

---

### 3. CSS Easing Functions (Phase 3)

**Impact**: Medium | **Risk**: Low | **Effort**: ~3 days

Replace custom `Interpolation` enum with CSS timing function strings.

#### Current State (`src/motion.rs`)
```rust
pub enum Interpolation {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Bezier { p1: (f64, f64), p2: (f64, f64) },
}
```

#### Target State
```jsonl
{"type": "animation", "name": "bounce", "frames": ["f1", "f2"],
  "timing-function": "cubic-bezier(0.68, -0.55, 0.265, 1.55)"
}

{"type": "animation", "name": "walk", "frames": ["w1", "w2", "w3"],
  "timing-function": "steps(3, jump-end)"
}
```

#### Supported CSS Timing Functions
- **Named**: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`
- **Cubic Bezier**: `cubic-bezier(x1, y1, x2, y2)`
- **Steps**: `steps(n, jump-start|jump-end|jump-both)`
- **Linear** (new): `linear(0, 0.25, 0.5 50%, 0.75, 1)`

#### Implementation
```rust
// Parse CSS timing function string to internal enum
pub fn parse_timing_function(css: &str) -> Result<Interpolation, ParseError> {
    match css {
        "linear" => Ok(Interpolation::Linear),
        "ease-in" => Ok(Interpolation::EaseIn),
        s if s.starts_with("cubic-bezier(") => {
            // Parse cubic-bezier(x1, y1, x2, y2)
            let (p1, p2) = parse_bezier_params(s)?;
            Ok(Interpolation::Bezier { p1, p2 })
        }
        // ... more cases
    }
}
```

**Backwards Compatibility**: Accept both enum strings (`"ease-in"`) and CSS functions (`"cubic-bezier(...)"`).

---

### 4. CSS Transform Functions (Phase 4)

**Impact**: High | **Risk**: Medium | **Effort**: ~1-2 weeks

When sprite transforms are implemented, use CSS transform syntax directly.

#### Target State
```jsonl
{"type": "sprite", "name": "rotated_hero", "base": "hero",
  "transform": "rotate(45deg) scale(1.5) translateX(10px)"
}

{"type": "animation", "name": "spin", "frames": ["sprite"],
  "keyframes": {
    "0%": {"transform": "rotate(0deg)"},
    "100%": {"transform": "rotate(360deg)"}
  }
}
```

#### Supported CSS Transform Functions
- `translateX(px)`, `translateY(px)`, `translate(x, y)`
- `rotate(deg)`, `rotateX(deg)`, `rotateY(deg)`
- `scale(n)`, `scaleX(n)`, `scaleY(n)`, `scale(x, y)`
- `skewX(deg)`, `skewY(deg)`
- `matrix(a, b, c, d, tx, ty)`

**Note**: This is future work. Current project has `src/transforms.rs` with flip/rotate but not CSS syntax.

---

### 5. CSS @keyframes Syntax (Phase 5)

**Impact**: High | **Risk**: High | **Effort**: ~2 weeks

Replace custom animation format with CSS-style keyframe definitions.

#### Current State
```jsonl
{"type": "animation", "name": "bounce", "frames": ["f1", "f2", "f1"], "duration": 100}
```

#### Target State
```jsonl
{"type": "keyframes", "name": "bounce",
  "keyframes": {
    "0%": {"sprite": "ball_squash", "transform": "translateY(0)"},
    "50%": {"sprite": "ball_round", "transform": "translateY(-20px)"},
    "100%": {"sprite": "ball_squash", "transform": "translateY(0)"}
  },
  "duration": "500ms",
  "timing-function": "cubic-bezier(0.68, -0.55, 0.265, 1.55)"
}
```

#### Benefits
- **Familiar syntax** - Matches CSS animations exactly
- **Percentage-based timing** - More intuitive than frame indices
- **Per-keyframe easing** - Different easing between keyframe pairs

**Risk**: This changes the animation format significantly. Requires migration path for existing animations.

---

## Migration Strategy

### Phase 1: CSS Colors (Week 1) ⭐ **START HERE**

**Goal**: Support CSS color functions in palettes alongside existing hex colors.

**Tasks**:
1. Add `lightningcss` or `csscolorparser` dependency
2. Update `parse_color()` in color parsing module to accept CSS syntax
3. Add tests for `rgb()`, `hsl()`, `oklch()`, named colors
4. Update docs with CSS color examples
5. Maintain backwards compatibility with hex-only colors

**Success Criteria**:
- All existing fixtures still pass
- Can parse `hsl(30, 45%, 85%)` to RGBA
- Can parse `oklch(84% 0.15 103)` to RGBA
- Named colors like `gold` work

**Files to Modify**:
- `src/color.rs` (or create it) - Color parsing logic
- `src/models.rs` - Palette color field parsing
- `tests/` - Add CSS color parsing tests

---

### Phase 2: CSS Custom Properties (Week 2)

**Goal**: Enable `--variables` and `var()` resolution in palettes.

**Tasks**:
1. Extend palette parsing to recognize `--variable` definitions
2. Implement `var()` resolution during palette expansion
3. Support `color-mix()` for computed colors
4. Add variable scope tracking (file-scoped recommended)
5. Update error messages for undefined variables

**Success Criteria**:
- Can define `"--primary": "oklch(60% 0.2 250)"`
- Can reference `"{shirt}": "var(--primary)"`
- Undefined variables produce clear error messages
- Variables can reference other variables

---

### Phase 3: CSS Easing (Week 3)

**Goal**: Accept CSS timing function strings in animations.

**Tasks**:
1. Add CSS timing function parser
2. Map CSS functions to existing `Interpolation` enum
3. Support `steps()` for pixel-perfect frame stepping
4. Update animation docs with CSS timing examples
5. Deprecate custom easing names in favor of CSS

**Success Criteria**:
- `"timing-function": "ease-in"` works
- `"timing-function": "cubic-bezier(0.68, -0.55, 0.265, 1.55)"` works
- `"timing-function": "steps(4, jump-end)"` works
- Backwards compatible with existing animations

---

### Phase 4: Documentation (Week 4)

**Goal**: Update all docs to emphasize CSS syntax and GenAI familiarity.

**Tasks**:
1. Update format spec with CSS color examples
2. Add "CSS Integration" section to README
3. Create migration guide for existing projects
4. Update GenAI system prompts to use CSS syntax
5. Add CSS syntax examples to web editor

---

## Technical Decisions

### Parser Library: `lightningcss`

**Chosen**: `lightningcss = "1.0"`

**Why**:
- Complete CSS parser written in Rust
- Used in production by Parcel bundler (high confidence)
- Handles colors, functions, custom properties, `calc()`, etc.
- Can parse individual values (don't need full stylesheets)
- Active maintenance and CSS spec compliance

**Alternative**: `csscolorparser = "0.6"` (simpler, but colors-only)

---

### Variable Scope: File-Scoped

**Decision**: CSS custom properties are scoped to the current `.pxl` file.

**Rationale**:
- Simple mental model: variables defined in a file are available in that file
- No cross-file dependency tracking needed
- Matches how most programming languages handle variables
- Can extend to global scope later if needed

**Alternative**: Global scope (more complex, harder to reason about)

---

### Resolution Timing: Parse Time

**Decision**: Resolve CSS variables during parsing, not during rendering.

**Rationale**:
- Simpler implementation: resolve once, render many times
- Clearer error messages: undefined variables fail immediately
- No runtime variable tracking needed
- Matches how CSS preprocessors (Sass, Less) work

**Alternative**: Render-time resolution (more flexible, but complex)

---

### Backwards Compatibility: Strict

**Decision**: All existing `.pxl` files must continue to work without changes.

**Rationale**:
- CSS syntax is additive, not replacement
- Hex colors like `#FFD700` are valid CSS (no breaking changes)
- Enum-style easing names (`"ease-in"`) map directly to CSS
- No forced migration needed

---

## Open Questions

### 1. Should palettes support CSS cascade/inheritance?

**Option A**: Simple `var()` resolution only
**Option B**: Full inheritance with base palettes

**Recommendation**: Start with Option A, add inheritance if users need it.

---

### 2. Should we support `calc()`?

Example: `"{size}": "calc(8px * 2)"`

**Pros**: Extremely powerful for computed values
**Cons**: Adds complexity, requires unit tracking

**Recommendation**: Defer to Phase 3+. Colors and easing are higher priority.

---

### 3. How deep should transform support go?

**Option A**: Basic `rotate(deg)` and `scale(n)` only
**Option B**: Full matrix operations and 3D transforms

**Recommendation**: Start with Option A. Pixel art rarely needs 3D transforms.

---

### 4. Should compositions use CSS Grid syntax?

Current compositions already layer sprites. Could they use CSS Grid positioning?

```jsonl
{"type": "composition", "name": "scene",
  "grid-template": "repeat(3, 8px) / repeat(4, 8px)",
  "sprites": {
    "hero": {"grid-area": "1 / 2 / 2 / 3"},
    "enemy": {"grid-area": "2 / 3 / 3 / 4"}
  }
}
```

**Pros**: Powerful, familiar to web devs
**Cons**: Complex, may be overkill for pixel art

**Recommendation**: Defer. Current map-based system works well.

---

## Success Metrics

Pixelsrc CSS integration succeeds when:

1. **LLMs generate CSS syntax naturally** - No need to teach hex colors, GenAI uses `hsl()` by default
2. **Human readability improves** - `oklch(84% 0.15 103)` is more intuitive than `#FFD700` for color relationships
3. **Zero breaking changes** - All existing `.pxl` files work unchanged
4. **Docs emphasize CSS** - README and tutorials show CSS examples first, hex as fallback

---

## Related Documents

- [VISION.md](../VISION.md) - Core design principles
- [format.md](../spec/format.md) - Format specification
- [motion.rs](../../src/motion.rs) - Current easing implementation
- [models.rs](../../src/models.rs) - Data structures (Palette, Animation, etc.)

---

## Implementation Tracking

**Phase 1: CSS Colors**
- [ ] Add `lightningcss` dependency
- [ ] Implement CSS color parsing
- [ ] Add test coverage for all CSS color formats
- [ ] Update format spec documentation
- [ ] Verify backwards compatibility

**Phase 2: CSS Custom Properties**
- [ ] Implement variable definition parsing (`--var`)
- [ ] Implement `var()` resolution
- [ ] Add `color-mix()` support
- [ ] Add variable scope tracking
- [ ] Update error messages

**Phase 3: CSS Easing**
- [ ] Implement timing function parser
- [ ] Map to existing `Interpolation` enum
- [ ] Add `steps()` support
- [ ] Update animation docs
- [ ] Deprecation plan for custom names

**Phase 4: Documentation**
- [ ] Update README with CSS examples
- [ ] Update format spec
- [ ] Create migration guide
- [ ] Update GenAI system prompts
- [ ] Add examples to web editor
