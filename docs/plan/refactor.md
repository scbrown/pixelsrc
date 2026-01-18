# Refactoring Plan

**Status**: 🟡 Planning

## Overview

This document tracks structural refactoring needs in the codebase that improve maintainability without changing functionality.

---

## REF-1: Break Up composition.rs

**Priority**: Medium
**Current Size**: 2,585 lines
**Target**: Multiple focused modules < 500 lines each

### Current Structure

```
src/composition.rs (2,585 lines)
├── BlendMode enum + impl (~80 lines)
├── Warning struct (~15 lines)
├── CompositionError enum + Display (~100 lines)
├── render_composition() function (~310 lines)
└── Tests (~2,055 lines)
```

### Problem

The file is dominated by tests (~80%), making navigation difficult. The `render_composition` function is also large and handles multiple concerns:
- Layer parsing and validation
- Sprite lookup and placement
- Blend mode application
- Size/cell calculations
- Error handling (strict/lenient modes)

### Proposed Structure

```
src/
├── composition/
│   ├── mod.rs              # Re-exports, render_composition()
│   ├── blend.rs            # BlendMode enum and pixel operations
│   ├── layer.rs            # Layer processing logic
│   └── error.rs            # CompositionError, Warning
│
tests/
└── composition_tests.rs    # Move tests out of src/ (2,055 lines)
```

### Migration Steps

1. **Create `src/composition/` directory**
2. **Extract `blend.rs`**
   - Move `BlendMode` enum
   - Move `BlendMode::blend_channel()` and related functions
   - ~100 lines
3. **Extract `error.rs`**
   - Move `CompositionError` enum
   - Move `Warning` struct
   - Move Display impl
   - ~100 lines
4. **Create `mod.rs`**
   - Keep `render_composition()` function
   - Re-export BlendMode, CompositionError, Warning
   - ~350 lines
5. **Move tests to `tests/composition_tests.rs`**
   - Integration tests don't need to be in src/
   - Reduces cognitive load when reading implementation
   - 2,055 lines → separate file

### Benefits

- **Easier navigation**: Each file has a single responsibility
- **Parallel development**: Different modules can be worked on independently
- **CSS integration ready**: `blend.rs` becomes natural home for CSS blend modes
- **Test isolation**: Implementation and tests in separate locations

### Breaking Changes

None - only file structure changes, public API unchanged.

### Implementation Tracking

- [ ] Create `src/composition/` directory
- [ ] Extract `src/composition/blend.rs`
- [ ] Extract `src/composition/error.rs`
- [ ] Create `src/composition/mod.rs` with re-exports
- [ ] Move tests to `tests/composition_tests.rs`
- [ ] Update imports in other files
- [ ] Verify `cargo test` passes
- [ ] Update any documentation references

---

## REF-2: Standardize Registry Pattern

**Priority**: Low
**Depends on**: CSS Variables (Phase 2)

### Current State

Multiple registries with similar patterns but slightly different APIs:

```rust
// src/registry.rs
PaletteRegistry::register(palette)
PaletteRegistry::resolve_strict(sprite) -> Result
PaletteRegistry::resolve_lenient(sprite) -> LenientResult

SpriteRegistry::register_sprite(sprite)
SpriteRegistry::resolve(name, palette_reg, strict) -> Result
```

### Proposed Standardization

After CSS variables are added, consider unifying:

```rust
pub trait Registry<T, R> {
    fn register(&mut self, item: T);
    fn resolve(&self, name: &str, strict: bool) -> Result<R, RegistryError>;
}

impl Registry<Palette, ResolvedPalette> for PaletteRegistry { ... }
impl Registry<Sprite, ResolvedSprite> for SpriteRegistry { ... }
impl Registry<String, String> for VariableRegistry { ... }
```

**Note**: This is low priority - the current code works fine. Only consider if registry proliferation becomes a maintenance burden.

---

## REF-3: Extract CLI Subcommand Handlers

**Priority**: Low
**Current Size**: src/cli.rs - examine when needed

### Observation

As CLI grows, individual command handlers may benefit from extraction:

```
src/
├── cli.rs                  # Command enum, argument parsing
├── cli/
│   ├── render.rs           # pxl render implementation
│   ├── import.rs           # pxl import implementation
│   ├── build.rs            # pxl build implementation
│   └── ...
```

**Note**: Only pursue if cli.rs becomes unwieldy. Current structure may be fine.

---

## General Refactoring Principles

1. **Don't refactor speculatively** - Only break up files when they cause real pain
2. **Maintain public API** - Internal restructuring shouldn't break external usage
3. **Tests follow code** - If tests are 80%+ of a file, consider moving them
4. **One concern per file** - BlendMode is distinct from composition rendering
5. **Re-exports preserve ergonomics** - `use pixelsrc::composition::BlendMode` should still work

---

## Related Documents

- [css.md](css.md) - CSS integration (motivation for blend.rs extraction)
- [build-system.md](build-system.md) - Build system documentation
