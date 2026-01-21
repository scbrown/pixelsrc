# Nested Compositions

**Goal:** Allow compositions to reference other compositions in their sprite maps, enabling hierarchical scene building.

**Status:** Complete (NC-5 in progress)

**Depends on:** Phase 12 (Composition Tiling) complete

**Beads:**
- NC-1: TTP-3vsl (closed)
- NC-2: TTP-ic6c (closed)
- NC-3: TTP-4dfv (closed)
- NC-4: TTP-g624 (closed)
- NC-5: TTP-6wng (in_progress)
- NC-6: TTP-jvoe (closed)

---

## Motivation

### The Problem

Currently, a composition's `sprites` map can only reference sprites, not other compositions. This limits hierarchical scene building.

For example, you can't define a "building" composition and then tile multiple buildings into a "city" composition:

```jsonl
// This doesn't work today - "building_tall" is a composition, not a sprite
{"type": "composition", "name": "city", "size": [1280, 720], "cell_size": [24, 80],
  "sprites": {"B": "building_tall", "S": "building_short"},
  "layers": [{"map": ["B.S.B.S.B..."]}]}
```

### Current Workarounds

Users must:
1. **Flatten everything into sprites** - losing composition benefits like cell_size alignment
2. **Pre-render to PNG and import** - breaks the single-source workflow, loses editability
3. **Manually duplicate placements** - error-prone, hard to maintain

### The Solution

Allow compositions to reference other compositions in their sprite maps. The renderer pre-renders referenced compositions to images at placement time.

```jsonl
{"type": "composition", "name": "building_tall", "size": [24, 80], "cell_size": [8, 8],
  "sprites": {"T": "bldg_top", "W": "window", "D": "door"},
  "layers": [{"map": ["TTT", "WWW", "WWW", "DDD"]}]}

{"type": "composition", "name": "city", "size": [1280, 720], "cell_size": [24, 80],
  "sprites": {"B": "building_tall", "S": "building_short"},  // compositions work here!
  "layers": [{"map": ["B.S.B.S.B..."]}]}
```

---

## Design

### Resolution Order

When resolving a sprite reference in a composition's `sprites` map:

1. **Check sprite registry first** - existing behavior unchanged
2. **Check composition registry** - new: if no sprite found, look for a composition
3. **Pre-render composition to RgbaImage** - render the referenced composition to an image
4. **Use rendered image as sprite** - place it like any other sprite

This maintains backwards compatibility - sprites take precedence over compositions with the same name (though naming collisions should warn).

### Cycle Detection

Nested compositions introduce the possibility of infinite recursion:

```jsonl
{"type": "composition", "name": "A", "sprites": {"B": "B"}, ...}
{"type": "composition", "name": "B", "sprites": {"A": "A"}, ...}  // cycle!
```

The renderer must detect cycles and report an error. Implementation options:

1. **Stack-based detection** - Track composition names being rendered in a stack; error if name already in stack
2. **Pre-validation pass** - Build dependency graph before rendering; detect cycles via DFS

Recommended: **Stack-based detection** - simpler, catches cycles at render time with clear error context.

### Size Validation

When a composition references another composition:
- Referenced composition size should match or be <= the cell_size of the parent
- Same rules as sprite size validation (warn in lenient mode, error in strict)

```jsonl
{"type": "composition", "name": "small", "size": [16, 16], ...}
{"type": "composition", "name": "large", "cell_size": [32, 32],
  "sprites": {"S": "small"},  // OK: 16x16 fits in 32x32 cell
  ...}
```

### Caching

Rendering the same composition multiple times (e.g., many identical buildings) should cache the rendered image:

```rust
// Pseudocode
let rendered_compositions: HashMap<String, RgbaImage> = HashMap::new();

fn get_or_render_composition(name: &str, ...) -> &RgbaImage {
    if !rendered_compositions.contains_key(name) {
        let img = render_composition(name, ...);
        rendered_compositions.insert(name.to_string(), img);
    }
    rendered_compositions.get(name).unwrap()
}
```

---

## Examples

### Example 1: Building Blocks

Create reusable building compositions and arrange them into a city:

```jsonl
{"type": "palette", "name": "city", "colors": {
  "{_}": "#00000000", "{w}": "#808080", "{b}": "#404040", "{g}": "#c0c0c0"
}}

{"type": "sprite", "name": "window", "size": [8, 8], "palette": "city", "grid": [
  "{b}{b}{b}{b}{b}{b}{b}{b}",
  "{b}{g}{g}{b}{g}{g}{b}{b}",
  "{b}{g}{g}{b}{g}{g}{b}{b}",
  "{b}{b}{b}{b}{b}{b}{b}{b}",
  "{b}{g}{g}{b}{g}{g}{b}{b}",
  "{b}{g}{g}{b}{g}{g}{b}{b}",
  "{b}{b}{b}{b}{b}{b}{b}{b}",
  "{b}{b}{b}{b}{b}{b}{b}{b}"
]}

{"type": "sprite", "name": "roof", "size": [24, 8], "palette": "city", "grid": [...]}
{"type": "sprite", "name": "door", "size": [8, 16], "palette": "city", "grid": [...]}

{"type": "composition", "name": "building_3w", "size": [24, 48], "cell_size": [8, 8],
  "sprites": {"R": "roof", "W": "window", "D": "door", ".": null},
  "layers": [{"map": [
    "RRR",
    "WWW",
    "WWW",
    "WWW",
    "WDW",
    "WDW"
  ]}]}

{"type": "composition", "name": "city_block", "size": [120, 48], "cell_size": [24, 48],
  "sprites": {"B": "building_3w", ".": null},
  "layers": [{"map": ["B.B.B"]}]}
```

### Example 2: Animated Character in Scene

Define a character as a composition (for easier part swapping), then place in scenes:

```jsonl
{"type": "sprite", "name": "head", "size": [16, 16], ...}
{"type": "sprite", "name": "body", "size": [16, 24], ...}
{"type": "sprite", "name": "legs", "size": [16, 16], ...}

{"type": "composition", "name": "hero", "size": [16, 56],
  "sprites": {"H": "head", "B": "body", "L": "legs"},
  "layers": [{"map": ["H", "B", "L"]}]}

{"type": "composition", "name": "scene", "size": [256, 256],
  "sprites": {"P": "hero", "T": "tree", "G": "grass"},
  "layers": [
    {"map": ["GGGGGGGG...", ...]},  // background
    {"map": ["....P...T..", ...]}   // characters (hero is a composition!)
  ]}
```

### Example 3: UI Components

Build complex UI from simpler components:

```jsonl
{"type": "composition", "name": "button", "size": [64, 24], ...}
{"type": "composition", "name": "checkbox", "size": [24, 24], ...}
{"type": "composition", "name": "slider", "size": [128, 24], ...}

{"type": "composition", "name": "settings_panel", "size": [200, 300],
  "sprites": {"B": "button", "C": "checkbox", "S": "slider", ".": null},
  "layers": [{"map": [
    "C...Sound",
    "S........",
    "C...Music",
    "S........",
    "....B...."
  ]}]}
```

---

## Task Dependency Diagram

```
                          NESTED COMPOSITIONS TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                    Phase 12 Complete                            │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Foundation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐                                               │
    │  │   NC.1       │  Add CompositionRegistry, unified lookup      │
    │  │  Registry    │                                               │
    │  └──────┬───────┘                                               │
    └─────────┼───────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - After Registry)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐                             │
    │  │   NC.2       │  │   NC.3       │                             │
    │  │  Cycle       │  │  Caching     │                             │
    │  │  Detection   │  │  Layer       │                             │
    │  └──────────────┘  └──────────────┘                             │
    └─────────────────────────────────────────────────────────────────┘
              │                 │
              └────────┬────────┘
                       │
                       ▼
    WAVE 3 (Integration)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐                                               │
    │  │   NC.4       │  Update render_composition to use unified     │
    │  │  Rendering   │  lookup with cycle detection + caching        │
    │  └──────┬───────┘                                               │
    └─────────┼───────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 4 (Completion)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐                             │
    │  │   NC.5       │  │   NC.6       │                             │
    │  │  Examples    │  │  Docs        │                             │
    │  │  & Tests     │  │              │                             │
    │  └──────────────┘  └──────────────┘                             │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: NC.1                 (1 task)                          │
    │  Wave 2: NC.2 + NC.3          (2 tasks in parallel)             │
    │  Wave 3: NC.4                 (1 task, needs NC.1-NC.3)         │
    │  Wave 4: NC.5 + NC.6          (2 tasks in parallel)             │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task NC.1: Composition Registry

**Wave:** 1

Add a registry for compositions and a unified lookup function.

**Deliverables:**
- Add `CompositionRegistry` to `src/registry.rs`:
  ```rust
  pub struct CompositionRegistry {
      compositions: HashMap<String, Composition>,
  }

  impl CompositionRegistry {
      pub fn register(&mut self, comp: Composition);
      pub fn get(&self, name: &str) -> Option<&Composition>;
      pub fn contains(&self, name: &str) -> bool;
  }
  ```

- Add unified lookup type:
  ```rust
  pub enum Renderable {
      Sprite(RgbaImage),
      Composition(Composition),
  }
  ```

**Verification:**
```bash
cargo test registry::composition
# Test: Register and retrieve compositions
# Test: Check contains() works
```

**Dependencies:** Phase 12 complete

---

### Task NC.2: Cycle Detection

**Wave:** 2 (parallel with NC.3)

Implement cycle detection for nested composition references.

**Deliverables:**
- Add cycle detection to rendering context:
  ```rust
  pub struct RenderContext {
      render_stack: Vec<String>,  // compositions currently being rendered
  }

  impl RenderContext {
      pub fn push(&mut self, name: &str) -> Result<(), CycleError>;
      pub fn pop(&mut self);
  }
  ```

- Add error type:
  ```rust
  #[error("Cycle detected in compositions: {cycle}")]
  CycleDetected { cycle: String }  // e.g., "A -> B -> C -> A"
  ```

**Verification:**
```bash
cargo test composition::cycle
# Test: A -> B -> C renders OK
# Test: A -> B -> A returns CycleDetected error
# Test: A -> A (self-reference) returns CycleDetected error
```

**Dependencies:** Task NC.1

---

### Task NC.3: Caching Layer

**Wave:** 2 (parallel with NC.2)

Add caching for rendered compositions to avoid redundant rendering.

**Deliverables:**
- Add cache to render context:
  ```rust
  pub struct RenderContext {
      composition_cache: HashMap<String, RgbaImage>,
  }

  impl RenderContext {
      pub fn get_cached(&self, name: &str) -> Option<&RgbaImage>;
      pub fn cache(&mut self, name: String, image: RgbaImage);
  }
  ```

**Verification:**
```bash
cargo test composition::cache
# Test: Same composition rendered twice uses cache
# Test: Cache invalidation when composition changes (if needed)
```

**Dependencies:** Task NC.1

---

### Task NC.4: Unified Rendering

**Wave:** 3

Update `render_composition` to support nested composition references.

**Deliverables:**
- Modify `render_composition` in `src/composition.rs`:
  ```rust
  pub fn render_composition(
      comp: &Composition,
      sprites: &HashMap<String, RgbaImage>,
      compositions: &CompositionRegistry,  // NEW
      ctx: &mut RenderContext,             // NEW: for cycle detection + caching
      strict: bool,
      variables: Option<&VariableRegistry>,
  ) -> Result<(RgbaImage, Vec<Warning>), CompositionError>
  ```

- Update sprite lookup logic:
  ```rust
  // In the layer rendering loop:
  let sprite_image = if let Some(img) = sprites.get(sprite_name) {
      img.clone()
  } else if let Some(comp) = compositions.get(sprite_name) {
      // Check for cycle
      ctx.push(sprite_name)?;
      // Check cache
      let img = if let Some(cached) = ctx.get_cached(sprite_name) {
          cached.clone()
      } else {
          let (rendered, _) = render_composition(comp, sprites, compositions, ctx, strict, variables)?;
          ctx.cache(sprite_name.to_string(), rendered.clone());
          rendered
      };
      ctx.pop();
      img
  } else {
      // warning: sprite not found
      continue;
  };
  ```

- Update CLI and WASM entry points to pass composition registry

**Verification:**
```bash
cargo test composition::nested
# Test: Composition referencing sprite works (regression)
# Test: Composition referencing composition works
# Test: Two-level nesting (A -> B -> sprite) works
# Test: Cycle detection triggers error
# Test: Cache is used for repeated references
```

**Dependencies:** Tasks NC.1, NC.2, NC.3

---

### Task NC.5: Examples and Tests

**Wave:** 4 (parallel with NC.6)

Create comprehensive examples and tests.

**Deliverables:**
- `examples/nested_building.jsonl` - building blocks example
- `examples/nested_ui.jsonl` - UI components example
- `tests/fixtures/valid/nested_composition.jsonl`
- `tests/fixtures/invalid/composition_cycle.jsonl`
- Integration tests in `tests/integration/nested.rs`

**Verification:**
```bash
cargo test
pxl render examples/nested_building.jsonl -o /tmp/building.png
pxl render examples/nested_ui.jsonl -o /tmp/ui.png
```

**Dependencies:** Task NC.4

---

### Task NC.6: Documentation

**Wave:** 4 (parallel with NC.5)

Update documentation for nested compositions.

**Deliverables:**
- Update `docs/spec/format.md` with nested composition specification
- Update system prompts to mention nested compositions
- Add examples to website gallery (if applicable)

**Verification:**
- Documentation builds without errors
- Examples in documentation are accurate

**Dependencies:** Task NC.4

---

## Related Features

### Compositions as Animation Frames

A closely related feature (also in BACKLOG) would allow animations to reference compositions as frames:

```jsonl
{"type": "composition", "name": "frame_01", "size": [32, 32], ...}
{"type": "composition", "name": "frame_02", "size": [32, 32], ...}
{"type": "animation", "frames": ["frame_01", "frame_02"], ...}
```

This uses the same resolution pattern - animation frame lookup would check both sprite and composition registries. Could be implemented as a follow-on task using the same infrastructure.

---

## Open Questions

1. **Name collision handling** - If a sprite and composition have the same name, sprite wins. Should we warn about this?

2. **Depth limit** - Should there be a maximum nesting depth (e.g., 10 levels) to prevent stack overflow even without cycles?

3. **Transform support** - Can a composition reference with transforms work? e.g., `"B": "building:flip_h"`. This would require rendering the composition, then applying transforms.

4. **Performance** - For deeply nested compositions with many references, caching is essential. Should we consider lazy evaluation or parallel rendering?

---

## Success Criteria

1. Existing compositions (no nested references) render identically (backwards compatible)
2. Compositions can reference other compositions in their `sprites` map
3. Cycles are detected and reported with clear error messages
4. Repeated references are cached for performance
5. Examples demonstrate practical nested composition use cases
6. Documentation explains the feature and its limitations
