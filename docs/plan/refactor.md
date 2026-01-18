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

**Priority**: Medium
**Current Size**: 3,982 lines

### Problem

`cli.rs` at nearly 4,000 lines is difficult to navigate. Each subcommand handler is self-contained and could be extracted.

### Proposed Structure

```
src/
├── cli.rs                  # Command enum, argument parsing, dispatch
├── cli/
│   ├── mod.rs              # Re-exports
│   ├── render.rs           # pxl render implementation
│   ├── import.rs           # pxl import implementation
│   ├── build.rs            # pxl build implementation
│   ├── watch.rs            # pxl watch implementation
│   ├── validate.rs         # pxl validate implementation
│   └── ...
```

### Implementation Tracking

- [ ] Create `src/cli/` directory
- [ ] Extract each subcommand handler
- [ ] Keep `cli.rs` as thin dispatch layer
- [ ] Verify all commands work correctly

---

## REF-4: Break Up transforms.rs

**Priority**: High
**Current Size**: 4,197 lines (159 inline tests)
**Target**: Multiple focused modules < 500 lines each

### Problem

This is the largest file in the codebase. It contains multiple distinct transform categories that could be separated:

### Current Structure

```
src/transforms.rs (4,197 lines)
├── DitherPattern enum + impl (~130 lines)
├── GradientDirection enum (~25 lines)
├── TransformError enum + Display (~60 lines)
├── Transform enum (~110 lines)
├── Parsing functions (~700 lines)
│   ├── parse_transform_str()
│   ├── parse_transform_value()
│   ├── parse_transform_object()
│   └── various parse_*_params() helpers
├── Animation transforms (~150 lines)
│   ├── apply_pingpong()
│   ├── apply_reverse()
│   ├── apply_frame_offset()
│   ├── apply_hold()
│   └── apply_animation_transform()
├── Pixel transforms (~1,200 lines)
│   ├── apply_selout()
│   ├── apply_scale()
│   ├── apply_outline()
│   ├── apply_shift()
│   ├── apply_shadow()
│   ├── apply_mirror_*()
│   ├── apply_rotate()
│   ├── apply_tile()
│   ├── apply_pad()
│   └── apply_crop()
└── Tests (~1,800 lines)
```

### Proposed Structure

```
src/
├── transforms/
│   ├── mod.rs              # Re-exports, Transform enum
│   ├── error.rs            # TransformError
│   ├── parse.rs            # All parsing functions
│   ├── dither.rs           # DitherPattern, GradientDirection
│   ├── animation.rs        # pingpong, reverse, hold, offset
│   ├── spatial.rs          # mirror, rotate, shift, crop, pad, tile
│   ├── effects.rs          # outline, shadow, selout, scale
│   └── apply.rs            # Main transform application logic
│
tests/
└── transform_tests.rs      # Move 159 tests out of src/
```

### Benefits

- **AI-friendly**: Each module fits comfortably in context windows
- **Focused concerns**: Dither math separate from spatial transforms
- **Easier testing**: Can test animation vs pixel transforms independently
- **Better discoverability**: Clear module names indicate functionality

### Implementation Tracking

- [ ] Create `src/transforms/` directory
- [ ] Extract `error.rs` with TransformError
- [ ] Extract `dither.rs` with DitherPattern, GradientDirection
- [ ] Extract `parse.rs` with all parsing logic
- [ ] Extract `animation.rs` with frame-level transforms
- [ ] Extract `spatial.rs` with geometric transforms
- [ ] Extract `effects.rs` with visual effect transforms
- [ ] Create `mod.rs` with re-exports
- [ ] Move tests to `tests/transform_tests.rs`
- [ ] Verify `cargo test` passes

---

## REF-5: Project Standards - Linting

**Priority**: High
**Status**: Not configured

### Problem

No clippy or rustfmt configuration exists. The CI runs clippy but doesn't fail on warnings. Code style may be inconsistent.

### Cargo.toml Lints

Add to `Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
# Specific high-value lints
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
todo = "warn"
unimplemented = "warn"
dbg_macro = "warn"
print_stdout = "warn"
print_stderr = "warn"

# Pedantic overrides (too noisy)
missing_errors_doc = "allow"
missing_panics_doc = "allow"
module_name_repetitions = "allow"
must_use_candidate = "allow"
too_many_lines = "allow"
```

### Implementation Tracking

- [ ] Add `[lints.rust]` section to Cargo.toml
- [ ] Add `[lints.clippy]` section to Cargo.toml
- [ ] Run `cargo clippy` and fix violations
- [ ] Update CI to fail on clippy warnings

---

## REF-6: Project Standards - Formatting

**Priority**: High
**Status**: Not configured

### Problem

No `rustfmt.toml` exists. Default rustfmt settings may not match project preferences.

### rustfmt.toml

Create `rustfmt.toml`:

```toml
# Stable options
max_width = 100
use_small_heuristics = "Max"
edition = "2021"

# Keep imports organized
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

### Implementation Tracking

- [ ] Create `rustfmt.toml`
- [ ] Run `cargo fmt` to apply formatting
- [ ] Verify CI formatting check passes

---

## REF-7: Test Organization

**Priority**: High
**Current State**: 962 inline `#[test]` functions in src/

### Problem

Tests mixed with implementation code:
- Makes src/ files larger than necessary
- Harder to find tests vs implementation
- Some files are 80%+ tests

### Target State

| File | Current Tests | Action |
|------|--------------|--------|
| transforms.rs | 159 | → tests/transform_tests.rs |
| composition.rs | ~100 | → tests/composition_tests.rs |
| registry.rs | ~80 | → tests/registry_tests.rs |
| models.rs | ~60 | → tests/models_tests.rs |
| Others | ~560 | Evaluate per-file |

### Guidelines

- **Unit tests** that need private access → stay in `src/` as `#[cfg(test)]` modules
- **Integration tests** that only use public API → move to `tests/`
- **Target**: < 200 lines of tests per src/ file

### Implementation Tracking

- [ ] Audit test access patterns (private vs public API)
- [ ] Move integration-style tests to `tests/`
- [ ] Keep true unit tests in `src/` with `#[cfg(test)]`
- [ ] Verify all tests pass after reorganization

---

## REF-8: Test Coverage

**Priority**: Medium
**Status**: Not configured

### Problem

No visibility into test coverage. Unknown which code paths are tested.

### Solution

Add coverage tooling:

```bash
# Install cargo-tarpaulin (Linux) or cargo-llvm-cov (all platforms)
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html
```

### Coverage Targets

| Category | Target |
|----------|--------|
| Overall | 80%+ |
| Core rendering | 90%+ |
| CLI handlers | 70%+ |
| Error paths | 80%+ |

### CI Integration

```yaml
- name: Generate coverage
  run: cargo llvm-cov --lcov --output-path lcov.info

- name: Upload coverage
  uses: codecov/codecov-action@v4
  with:
    files: lcov.info
```

### Implementation Tracking

- [ ] Install coverage tooling locally
- [ ] Generate baseline coverage report
- [ ] Add coverage to CI workflow
- [ ] Set up Codecov or similar reporting
- [ ] Establish minimum coverage thresholds

---

## REF-9: CI Pipeline Hardening

**Priority**: High
**Status**: CI exists but may be failing

### Current State

`.github/workflows/ci.yml` runs:
- `cargo test --verbose`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets`

### Problems

1. Clippy doesn't fail on warnings (`-D warnings` missing)
2. No coverage reporting
3. No WASM build verification in main CI
4. No caching optimization for faster builds

### Updated CI Workflow

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Build
        run: cargo build --all-features

      - name: Run tests
        run: cargo test --all-features --verbose

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: false
```

### Implementation Tracking

- [ ] Add `-D warnings` to clippy invocation
- [ ] Switch to `Swatinem/rust-cache@v2` for better caching
- [ ] Add coverage job
- [ ] Add WASM build verification
- [ ] Fix any existing CI failures

---

## REF-10: Error Handling Standardization

**Priority**: Low
**Status**: Mixed patterns

### Current State

Multiple error handling patterns across the codebase:
- Custom error enums with manual `Display` impls
- String errors in some places
- `Box<dyn Error>` in others

### Proposed Standardization

Consider adopting `thiserror` for consistent error handling:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransformError {
    #[error("invalid rotation {degrees}: must be 0, 90, 180, or 270")]
    InvalidRotation { degrees: u16 },

    #[error("unknown transform: {name}")]
    UnknownTransform { name: String },

    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
}
```

### Benefits

- Less boilerplate than manual `Display` impls
- Automatic `From` implementations
- Consistent error formatting
- Source chaining support

### Implementation Tracking

- [ ] Add `thiserror` dependency
- [ ] Migrate TransformError to thiserror
- [ ] Migrate CompositionError to thiserror
- [ ] Migrate other error types
- [ ] Remove manual Display impls

---

## REF-11: Documentation Coverage

**Priority**: Low
**Status**: Partial

### Problem

Public API lacks consistent documentation. Users can't easily understand function purposes.

### Solution

Enable `missing_docs` lint and add documentation:

```rust
#![warn(missing_docs)]

/// Applies a horizontal mirror transform to the pixel grid.
///
/// # Arguments
/// * `grid` - The pixel grid as a vector of row strings
///
/// # Returns
/// A new grid with each row reversed
pub fn apply_mirror_horizontal(grid: &[String]) -> Vec<String> {
```

### Targets

| Category | Documentation Level |
|----------|-------------------|
| Public functions | Required |
| Public structs | Required |
| Public enums | Required + variant docs |
| Internal functions | Optional |

### Implementation Tracking

- [ ] Add `#![warn(missing_docs)]` to lib.rs
- [ ] Document all public functions
- [ ] Document all public types
- [ ] Add module-level documentation
- [ ] Generate and review `cargo doc`

---

## REF-12: Benchmark Suite

**Priority**: Medium
**Status**: Not implemented

### Problem

No performance benchmarks exist. Can't detect performance regressions.

### Solution

Add criterion benchmarks for critical paths:

```rust
// benches/rendering.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixelsrc::*;

fn bench_render_sprite(c: &mut Criterion) {
    let sprite = load_test_sprite();
    c.bench_function("render 32x32 sprite", |b| {
        b.iter(|| render_sprite(black_box(&sprite)))
    });
}

criterion_group!(benches, bench_render_sprite);
criterion_main!(benches);
```

### Benchmark Targets

| Operation | Why |
|-----------|-----|
| Sprite rendering | Core functionality |
| Transform application | Used heavily |
| Composition | Multi-layer rendering |
| Build (incremental) | Developer experience |

### Implementation Tracking

- [ ] Add criterion dev-dependency
- [ ] Create `benches/` directory
- [ ] Add rendering benchmarks
- [ ] Add transform benchmarks
- [ ] Add composition benchmarks
- [ ] Set up benchmark comparison in CI

---

## Summary: Priority Order

| Priority | Items | Focus |
|----------|-------|-------|
| **High** | REF-4, REF-5, REF-6, REF-9 | Large file breakup, linting, CI |
| **Medium** | REF-1, REF-3, REF-7, REF-8, REF-12 | Module extraction, testing |
| **Low** | REF-2, REF-10, REF-11 | Polish, standardization |

---

## General Refactoring Principles

1. **Don't refactor speculatively** - Only break up files when they cause real pain
2. **Maintain public API** - Internal restructuring shouldn't break external usage
3. **Tests follow code** - If tests are 80%+ of a file, consider moving them
4. **One concern per file** - BlendMode is distinct from composition rendering
5. **Re-exports preserve ergonomics** - `use pixelsrc::composition::BlendMode` should still work
6. **AI context friendly** - Target < 500 lines per file for optimal GenAI assistance

---

## Related Documents

- [css.md](css.md) - CSS integration (motivation for blend.rs extraction)
- [build-system.md](build-system.md) - Build system documentation
