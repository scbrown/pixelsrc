# Contributing to Pixelsrc

Welcome! Pixelsrc is designed to be approachable for contributors, including AI agents.

---

## Quick Start

### Prerequisites

- Rust (stable, 1.70+)
- Cargo

### Setup

```bash
# Clone the repo
git clone <repo-url>
cd pixelsrc

# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- render examples/coin.jsonl -o coin.png
```

---

## Project Structure

```
pixelsrc/
├── docs/
│   ├── VISION.md          # Why we're building this, core tenets
│   ├── ANNOUNCEMENT.md    # Product positioning
│   ├── spec/
│   │   └── format.md      # Formal JSONL specification
│   └── plan/              # Implementation phases (see README.md)
├── CONTRIBUTING.md        # This file
├── Cargo.toml             # Rust package config
├── src/
│   ├── main.rs            # Entry point
│   ├── cli.rs             # Clap-based CLI
│   ├── parser.rs          # JSONL parsing + validation
│   ├── renderer.rs        # PNG/GIF generation via `image` crate
│   └── models.rs          # Serde structs for palette, sprite, animation
├── examples/              # Example .jsonl files
├── tests/
│   └── fixtures/
│       ├── valid/         # Files that should parse successfully
│       ├── invalid/       # Files that should fail (missing fields, bad JSON)
│       └── lenient/       # Files with warnings (work in default, fail in --strict)
```

---

## Key Documents

Before contributing, read these:

1. **docs/VISION.md** - Understand the "why" and core tenets
2. **docs/spec/format.md** - Formal specification for the JSONL format
3. **docs/plan/README.md** - See what phase we're in and what's planned
4. **docs/plan/phase-0-mvp.md** - Detailed task breakdown for MVP

---

## Code Conventions

### Rust Style

- Follow standard `rustfmt` formatting
- Use `clippy` for linting (see [Clippy](#clippy) section below)
- Prefer explicit error types over `unwrap()` in library code

### Error Handling

Pixelsrc has two modes:

- **Lenient (default)**: Fill gaps, warn, continue
- **Strict (`--strict`)**: Fail on first warning

When implementing error handling:
```rust
// Good: Return a warning that can be collected
fn parse_row(...) -> (Vec<Token>, Vec<Warning>) { ... }

// Let the caller decide: warn or fail based on mode
```

#### Error Types

Use `thiserror` for defining library error types:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid color format: {0}")]
    InvalidColor(String),
    #[error("unknown palette reference: {0}")]
    UnknownPalette(String),
}
```

Implement `From` traits for error conversions to enable the `?` operator:

```rust
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::Io(err)
    }
}
```

Define module-level `Result` type aliases for cleaner signatures:

```rust
pub type Result<T> = std::result::Result<T, ParseError>;
```

#### When to Use `unwrap()`, `expect()`, and `?`

| Method | When to Use |
|--------|-------------|
| `unwrap()` | **Only in tests.** Never in library or application code. |
| `expect("reason")` | Provably safe cases where failure indicates a bug. Document why it's safe. |
| `?` | All fallible operations in library code. Propagate errors to callers. |

```rust
// Bad: unwrap in library code
let value = map.get("key").unwrap();

// Good: expect with documented reason
let value = map.get("key").expect("key always present after validation");

// Best: propagate error to caller
let value = map.get("key").ok_or(ParseError::MissingKey)?;
```

See [TTP-0gb3](https://github.com/anthropics/pixelsrc/issues/TTP-0gb3) for the `unwrap()` cleanup epic.

### Type Safety

#### Newtypes for Domain Primitives

Wrap primitives in newtypes to prevent mixing up values:

```rust
// Bad: easy to mix up frame and layer indices
fn get_pixel(frame: usize, layer: usize, x: usize, y: usize) -> Color;

// Good: type-safe indices
pub struct FrameIndex(pub usize);
pub struct LayerIndex(pub usize);
fn get_pixel(frame: FrameIndex, layer: LayerIndex, x: usize, y: usize) -> Color;
```

#### Builder Pattern

Use the builder pattern for structs with many optional fields:

```rust
let sprite = SpriteBuilder::new("hero")
    .size(16, 16)
    .palette("gameboy")
    .build()?;
```

#### Attributes for Safety

- **`#[must_use]`**: Add to functions where ignoring the return value is likely a bug
- **`#[non_exhaustive]`**: Add to public enums that may gain variants

```rust
#[must_use]
pub fn validate(&self) -> ValidationResult { ... }

#[non_exhaustive]
pub enum ExportFormat {
    Png,
    Gif,
    Spritesheet,
}
```

### Code Organization

#### Module Structure

- Files under 500 lines: single `module.rs` file
- Files over 500 lines: directory with `mod.rs` and submodules

```
src/
├── small_module.rs          # < 500 lines, single file
└── large_module/            # > 500 lines, directory
    ├── mod.rs               # Public API, re-exports, tests
    ├── parser.rs            # Implementation details
    └── types.rs             # Types used by the module
```

#### Test Organization

- Keep unit tests in `mod.rs` or the main module file
- Keep implementation in submodules
- Re-export public types at module root

```rust
// In mod.rs
mod parser;
mod types;

pub use types::{Sprite, Frame, Animation};

#[cfg(test)]
mod tests {
    use super::*;
    // tests here
}
```

### Documentation

#### Doc Comments

All public items need doc comments:

```rust
/// Parses a JSONL file and returns sprites.
///
/// # Arguments
///
/// * `input` - The JSONL content to parse
///
/// # Errors
///
/// Returns `ParseError` if the input is malformed.
///
/// # Examples
///
/// ```
/// let sprites = parse_jsonl(r#"{"type":"sprite","name":"dot"}"#)?;
/// ```
pub fn parse_jsonl(input: &str) -> Result<Vec<Sprite>> { ... }
```

#### Module-Level Documentation

Add `//!` docs at the top of each module explaining its purpose:

```rust
//! # Parser Module
//!
//! Handles parsing of JSONL sprite definitions into internal representations.
//! Supports both lenient and strict parsing modes.

use crate::models::Sprite;
```

### Clippy

#### Zero Warnings Policy

All code must pass clippy with warnings as errors:

```bash
cargo clippy -- -D warnings
```

Run this before committing. CI will fail on clippy warnings.

#### Allowing Lints

If you must suppress a clippy lint, document the reason:

```rust
#[allow(clippy::too_many_arguments)] // Builder pattern not suitable for hot path
fn render_frame(/* many args */) { ... }
```

See [TTP-rkab](https://github.com/anthropics/pixelsrc/issues/TTP-rkab) for the Rust idioms improvement epic.

### Testing

- Add fixtures for new features in `tests/fixtures/`
- Valid fixtures go in `valid/`
- Invalid fixtures (should error) go in `invalid/`
- Lenient fixtures (warn but succeed) go in `lenient/`

Each fixture should be a minimal reproduction of the case it tests.

### Coverage

We track test coverage via [Codecov](https://codecov.io). Coverage targets:
- **Project coverage**: 70% (overall codebase)
- **Patch coverage**: 80% (new/modified code in PRs)

To generate coverage locally:

```bash
# Install cargo-llvm-cov (one-time)
cargo install cargo-llvm-cov

# Generate coverage report (text summary)
just coverage

# Generate and open HTML report
just coverage-html

# Generate LCOV format (for tooling)
just coverage-lcov
```

Coverage reports are automatically generated in CI and uploaded to Codecov.

---

## Development Workflow

### Adding a Feature

1. Check if there's a task/issue for it
2. Read relevant spec in `spec/format.md`
3. Write failing tests first
4. Implement the feature
5. Ensure `cargo test` passes
6. Run `cargo clippy` and fix warnings
7. If you changed any demo tests in `tests/demos/`, regenerate demo docs (see [Demo Documentation](#demo-documentation) below)
8. Submit PR

### Demo Documentation

Demo tests in `tests/demos/` are the source of truth for the generated book documentation in `docs/book/src/demos/`. CI enforces that these stay in sync.

**If you add or modify any file under `tests/demos/`**, you must regenerate the docs:

```bash
# Regenerate demo documentation
./scripts/generate-demos.sh --book

# Verify it's up to date (this is what CI runs)
./scripts/generate-demos.sh --book --check
```

Commit the regenerated docs alongside your test changes. CI will fail if demo docs are stale.

### Modifying the Spec

If you need to change `spec/format.md`:

1. Discuss the change first (open an issue)
2. Update the spec
3. Update affected code
4. Update/add fixtures to cover the change

---

## Test Fixtures Reference

### Valid Fixtures (`tests/fixtures/valid/`)

| File | Tests |
|------|-------|
| `minimal_dot.jsonl` | Smallest valid sprite (1x1) |
| `simple_heart.jsonl` | Basic multi-row sprite |
| `named_palette.jsonl` | Palette defined separately, referenced by name |
| `with_size.jsonl` | Explicit size declaration |
| `multiple_sprites.jsonl` | Multiple sprites sharing a palette |
| `color_formats.jsonl` | All supported color formats (#RGB, #RRGGBB, etc.) |
| `animation.jsonl` | Animation with frames and timing |

### Invalid Fixtures (`tests/fixtures/invalid/`)

| File | Expected Error |
|------|----------------|
| `missing_type.jsonl` | Missing required `type` field |
| `missing_name.jsonl` | Missing required `name` field |
| `missing_grid.jsonl` | Missing required `grid` field |
| `missing_palette.jsonl` | Missing required `palette` field |
| `invalid_json.jsonl` | Malformed JSON |
| `unknown_palette_ref.jsonl` | References undefined palette |
| `invalid_color.jsonl` | Color value not a valid hex |

### Lenient Fixtures (`tests/fixtures/lenient/`)

| File | Warning | Lenient Behavior |
|------|---------|------------------|
| `row_too_short.jsonl` | Row has fewer tokens than width | Pad with `{_}` |
| `row_too_long.jsonl` | Row has more tokens than width | Truncate |
| `unknown_token.jsonl` | Token not in palette | Render as magenta |
| `duplicate_name.jsonl` | Two sprites with same name | Last wins |
| `extra_chars_in_grid.jsonl` | Characters outside `{...}` | Ignore |

---

## CLI Reference

```bash
# Render sprites to PNG
pxl render input.jsonl                    # Output: input_{name}.png
pxl render input.jsonl -o output.png      # Output: output.png (single) or output_{name}.png
pxl render input.jsonl -o dir/            # Output: dir/{name}.png
pxl render input.jsonl --sprite hero      # Render only "hero"

# Strict mode (fail on warnings)
pxl render input.jsonl --strict

# Animation (Phase 2)
pxl render input.jsonl --gif -o anim.gif
pxl render input.jsonl --spritesheet -o sheet.png

# Palettes (Phase 1)
pxl palettes list
pxl palettes show gameboy
```

---

## For AI Agents

If you're an AI agent working on Pixelsrc:

1. **Read the spec first** - `spec/format.md` has all the rules
2. **Check fixtures** - They show expected behavior for edge cases
3. **Lenient by default** - When in doubt, warn and continue
4. **Minimal changes** - Don't over-engineer; simple solutions preferred
5. **Test your changes** - Add fixtures for new cases

The codebase is designed to be straightforward. Most tasks are isolated to single files.

### Artistic Work: Artisan Workflow

When creating or improving pixel art examples, use the **Artisan Workflow** documented in `docs/artistic-workflow.md`.

**Key principles:**
- Work on components in isolation (eyes, mouth, hair, etc.)
- Generate 2-3 variants per iteration, evaluate against quality gates
- Use consistent naming: `[component]_v[iteration][variant]` (e.g., `eyes_v1a`)
- Log each iteration: approach tried, gate results, winner, reasoning

**Quality Gates (binary pass/fail):**

| Gate | Test |
|------|------|
| Silhouette | Fill with solid color - is shape recognizable? |
| Scale | View at 1x - is it readable? |
| Palette | `pxl validate` - any unknown token warnings? |
| Pixels | View at 8x - any orphan pixels or jagged edges? |
| Lighting | Is shading consistent with top-right 45° light? |

**Bead structure for art tasks:**
```
artisan-[artwork] (epic)
├── foundation (task)
├── iterate-[component] (task) - one per component
├── integrate (task)
└── submit (task)
```

See `docs/artistic-workflow.md` for complete documentation including variant strategies, integration gates, and RPG character quality targets.

---

## Creating Releases

Releases are automated via GitHub Actions. To create a new release:

### 1. Update Version

Update the version in `Cargo.toml`:
```toml
[package]
version = "0.2.0"  # Bump appropriately
```

### 2. Commit and Tag

```bash
git add Cargo.toml
git commit -m "Bump version to v0.2.0"
git tag v0.2.0
git push origin main --tags
```

### 3. What Happens

**CLI Binaries** (release.yml):
- Builds binaries for 6 platforms:
  - Linux (x86_64, aarch64)
  - macOS (x86_64, aarch64)
  - Windows (x86_64, aarch64)
- Generates SHA256 checksums
- Creates a GitHub Release with all assets

**WASM Package** (wasm.yml):
- Builds the WASM module with wasm-pack
- Syncs version from Cargo.toml to package.json
- Publishes `@stiwi/pixelsrc-wasm` to npm with provenance

### 4. Verify

After pushing the tag:
1. Check [Actions](../../actions) tab for workflow progress
2. Once complete, verify the [Releases](../../releases) page has all artifacts
3. Download and test a binary on your platform
4. Verify npm package: `npm view @stiwi/pixelsrc-wasm`

### Release Assets

**GitHub Release** includes:
- `pxl-v{version}-x86_64-unknown-linux-gnu.tar.gz`
- `pxl-v{version}-aarch64-unknown-linux-gnu.tar.gz`
- `pxl-v{version}-x86_64-apple-darwin.tar.gz`
- `pxl-v{version}-aarch64-apple-darwin.tar.gz`
- `pxl-v{version}-x86_64-pc-windows-msvc.zip`
- `pxl-v{version}-aarch64-pc-windows-msvc.zip`
- `SHA256SUMS.txt`

**npm** publishes:
- `@stiwi/pixelsrc-wasm@{version}`

---

## Questions?

- Check existing issues
- Read VISION.md for design philosophy
- Open an issue for clarification
