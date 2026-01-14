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
- Use `clippy` for linting: `cargo clippy`
- Prefer explicit error types over `unwrap()` in library code
- `unwrap()` is acceptable in tests and examples

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

### Testing

- Add fixtures for new features in `tests/fixtures/`
- Valid fixtures go in `valid/`
- Invalid fixtures (should error) go in `invalid/`
- Lenient fixtures (warn but succeed) go in `lenient/`

Each fixture should be a minimal reproduction of the case it tests.

---

## Development Workflow

### Adding a Feature

1. Check if there's a task/issue for it
2. Read relevant spec in `spec/format.md`
3. Write failing tests first
4. Implement the feature
5. Ensure `cargo test` passes
6. Run `cargo clippy` and fix warnings
7. Submit PR

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

---

## Questions?

- Check existing issues
- Read VISION.md for design philosophy
- Open an issue for clarification
