# Phase 0: MVP (Core)

**Goal:** Parse TTP JSONL files and render sprites to PNG

**Status:** Planning

---

## Scope

Phase 0 delivers the minimum viable product:
- Parse `.jsonl` files containing palette and sprite definitions
- Render sprites to PNG images
- Basic CLI: `pxl render input.jsonl`
- Lenient and strict error modes

**Not in scope:** Animation, built-in palettes, game engine export

---

## Task Dependency Diagram

```
                              PHASE 0 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    WAVE 1 (Sequential - Must Complete First)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐                                               │
    │  │   0.1        │                                               │
    │  │  Scaffold    │  cargo new, Cargo.toml, module stubs          │
    │  │              │                                               │
    │  └──────┬───────┘                                               │
    └─────────┼───────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - Run Simultaneously)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
    │  │   0.2        │  │   0.3        │  │   0.4        │          │
    │  │  Models      │  │  Color       │  │  Tokenizer   │          │
    │  │              │  │  Parsing     │  │              │          │
    │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
    └─────────┼─────────────────┼─────────────────┼───────────────────┘
              │                 │                 │
              ▼                 │                 │
    WAVE 3 (After Models)       │                 │
    ┌─────────────────────┐     │                 │
    │  ┌──────────────┐   │     │                 │
    │  │   0.5        │   │     │                 │
    │  │  JSONL       │   │     │                 │
    │  │  Parser      │   │     │                 │
    │  └──────┬───────┘   │     │                 │
    └─────────┼───────────┘     │                 │
              │                 │                 │
              ▼                 │                 │
    WAVE 4 (After Parser)       │                 │
    ┌─────────────────────┐     │                 │
    │  ┌──────────────┐   │     │                 │
    │  │   0.6        │   │     │                 │
    │  │  Palette     │   │     │                 │
    │  │  Registry    │   │     │                 │
    │  └──────┬───────┘   │     │                 │
    └─────────┼───────────┘     │                 │
              │                 │                 │
              └────────┬────────┴────────┬────────┘
                       │                 │
                       ▼                 ▼
    WAVE 5 (Convergence Point - Needs 0.3, 0.4, 0.6)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐                                               │
    │  │   0.7        │  Combines: color parsing + tokenizer +        │
    │  │  Renderer    │           palette resolution                  │
    │  │              │                                               │
    │  └──────┬───────┘                                               │
    └─────────┼───────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 6-8 (Sequential Pipeline)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
    │  │   0.8        │  │   0.9        │  │   0.10       │          │
    │  │  PNG Output  │─▶│  CLI         │─▶│  Integration │          │
    │  │              │  │              │  │  Tests       │          │
    │  └──────────────┘  └──────────────┘  └──────────────┘          │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 0.1                    (1 task)                        │
    │  Wave 2: 0.2 + 0.3 + 0.4        (3 tasks in parallel)           │
    │  Wave 3: 0.5                    (1 task, needs 0.2)             │
    │  Wave 4: 0.6                    (1 task, needs 0.5)             │
    │  Wave 5: 0.7                    (1 task, needs 0.3+0.4+0.6)     │
    │  Wave 6: 0.8                    (1 task)                        │
    │  Wave 7: 0.9                    (1 task)                        │
    │  Wave 8: 0.10                   (1 task)                        │
    └─────────────────────────────────────────────────────────────────┘

    CRITICAL PATH: 0.1 → 0.2 → 0.5 → 0.6 → 0.7 → 0.8 → 0.9 → 0.10
```

---

## Tasks

Tasks are sized for agent completion in ~60% of context window (learn → execute → test → iterate → done).

---

### Task 0.1: Project Scaffolding

**Wave:** 1 (must be first)

Set up the Rust project structure.

**Deliverables:**
- `Cargo.toml` with dependencies:
  ```toml
  [dependencies]
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  image = "0.24"
  clap = { version = "4.0", features = ["derive"] }
  ```
- `src/main.rs` - entry point stub
- `src/lib.rs` - module declarations
- Module stubs: `src/models.rs`, `src/color.rs`, `src/tokenizer.rs`, `src/parser.rs`, `src/registry.rs`, `src/renderer.rs`, `src/output.rs`, `src/cli.rs`

**Verification:**
```bash
cargo build                    # Must succeed
cargo test                     # Must run (0 tests OK)
cargo clippy                   # No errors
```

**Updates demo.sh:** Creates initial demo.sh that shows "Phase 0 in progress"

**Dependencies:** None

---

### Task 0.2: Data Models

**Wave:** 2 (parallel with 0.3, 0.4)

Define Rust structs for TTP objects.

**Deliverables:**
- `src/models.rs`:
  ```rust
  pub struct Palette { name: String, colors: HashMap<String, Color> }
  pub struct Sprite { name: String, size: Option<[u32; 2]>, palette: PaletteRef, grid: Vec<String> }
  pub enum PaletteRef { Named(String), Inline(HashMap<String, Color>) }
  pub enum TtpObject { Palette(Palette), Sprite(Sprite) }
  pub struct Warning { message: String, line: usize }
  ```
- All structs derive `Serialize`, `Deserialize`, `Debug`, `Clone`

**Verification:**
```bash
cargo test models              # Unit tests pass
# Test: JSON round-trip for each struct
# Test: Deserialize from fixture files
```

**Test Fixtures:**
- `tests/fixtures/valid/minimal_dot.jsonl`
- `tests/fixtures/valid/named_palette.jsonl`

**Reference:** `docs/spec/format.md` - Object Types section

**Dependencies:** Task 0.1

---

### Task 0.3: Color Parsing

**Wave:** 2 (parallel with 0.2, 0.4)

Parse hex color strings to RGBA.

**Deliverables:**
- `src/color.rs`:
  ```rust
  pub fn parse_color(s: &str) -> Result<Rgba<u8>, ColorError>
  // Supports: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
  ```

**Verification:**
```bash
cargo test color               # Unit tests pass
# Test: #F00 → Rgba([255, 0, 0, 255])
# Test: #FF0000 → Rgba([255, 0, 0, 255])
# Test: #FF000080 → Rgba([255, 0, 0, 128])
# Test: #F00F → Rgba([255, 0, 0, 255])
# Test: "invalid" → ColorError
```

**Test Fixtures:** `tests/fixtures/valid/color_formats.jsonl`

**Reference:** `docs/spec/format.md` - Color Formats section

**Dependencies:** Task 0.1

---

### Task 0.4: Token Parsing

**Wave:** 2 (parallel with 0.2, 0.3)

Extract tokens from grid strings.

**Deliverables:**
- `src/tokenizer.rs`:
  ```rust
  pub fn tokenize(row: &str) -> (Vec<String>, Vec<Warning>)
  // Extracts {name} tokens, warns on extra characters
  ```

**Verification:**
```bash
cargo test tokenizer           # Unit tests pass
# Test: "{a}{b}{c}" → ["{a}", "{b}", "{c}"], []
# Test: "x{a}y{b}z" → ["{a}", "{b}"], [Warning("x"), Warning("y"), Warning("z")]
# Test: "{unclosed" → [], [Warning("unclosed brace")]
```

**Test Fixtures:** `tests/fixtures/lenient/extra_chars_in_grid.jsonl`

**Reference:** `docs/spec/format.md` - Token Parsing section

**Dependencies:** Task 0.1

---

### Task 0.5: JSONL Parser

**Wave:** 3 (after 0.2)

Parse JSONL stream into TTP objects.

**Deliverables:**
- `src/parser.rs`:
  ```rust
  pub fn parse_line(line: &str) -> Result<TtpObject, ParseError>
  pub fn parse_stream<R: BufRead>(reader: R) -> ParseResult
  pub struct ParseResult { objects: Vec<TtpObject>, warnings: Vec<Warning> }
  ```
- Skip blank lines
- Collect warnings in lenient mode

**Verification:**
```bash
cargo test parser              # Unit tests pass
# Test: All files in tests/fixtures/valid/ parse successfully
# Test: All files in tests/fixtures/invalid/ produce ParseError
# Test: Blank lines are skipped
```

**Test Fixtures:** `tests/fixtures/valid/*`, `tests/fixtures/invalid/*`

**Reference:** `docs/spec/format.md` - Stream Processing section

**Dependencies:** Task 0.2

---

### Task 0.6: Palette Resolution

**Wave:** 4 (after 0.5)

Resolve palette references for sprites.

**Deliverables:**
- `src/registry.rs`:
  ```rust
  pub struct PaletteRegistry { palettes: HashMap<String, Palette> }
  impl PaletteRegistry {
      pub fn register(&mut self, palette: Palette)
      pub fn resolve(&self, sprite: &Sprite) -> Result<ResolvedPalette, Warning>
  }
  pub struct ResolvedPalette { colors: HashMap<String, Rgba<u8>> }
  ```
- Inline palettes pass through
- Missing references → Warning + magenta fallback (lenient) or Error (strict)

**Verification:**
```bash
cargo test registry            # Unit tests pass
# Test: Named reference resolves correctly
# Test: Inline palette works
# Test: Missing palette → magenta fallback + warning
```

**Test Fixtures:**
- `tests/fixtures/valid/named_palette.jsonl`
- `tests/fixtures/invalid/unknown_palette_ref.jsonl`

**Reference:** `docs/spec/format.md` - Palette Reference Options

**Dependencies:** Task 0.5

---

### Task 0.7: Sprite Renderer

**Wave:** 5 (convergence - needs 0.3, 0.4, 0.6)

Render a sprite to an image buffer.

**Deliverables:**
- `src/renderer.rs`:
  ```rust
  pub fn render_sprite(sprite: &Sprite, palette: &ResolvedPalette) -> (RgbaImage, Vec<Warning>)
  ```
- Use `image::RgbaImage` for output
- Unknown tokens → magenta `#FF00FF`
- Size inference when `size` omitted
- Row padding (short rows) / truncation (long rows)

**Verification:**
```bash
cargo test renderer            # Unit tests pass
# Test: Render minimal_dot.jsonl → 1x1 image
# Test: Render simple_heart.jsonl → correct dimensions
# Test: Unknown token → magenta pixel
# Test: Short row → padded with transparent
```

**Test Fixtures:**
- `tests/fixtures/valid/minimal_dot.jsonl`
- `tests/fixtures/valid/simple_heart.jsonl`
- `tests/fixtures/lenient/row_too_short.jsonl`
- `tests/fixtures/lenient/unknown_token.jsonl`

**Reference:** `docs/spec/format.md` - Size Inference, Error Handling

**Dependencies:** Task 0.3, Task 0.4, Task 0.6

---

### Task 0.8: PNG Output

**Wave:** 6 (after 0.7)

Save rendered images to PNG files.

**Deliverables:**
- `src/output.rs`:
  ```rust
  pub fn save_png(image: &RgbaImage, path: &Path) -> Result<(), OutputError>
  pub fn generate_output_path(input: &Path, sprite_name: &str, output_arg: Option<&Path>) -> PathBuf
  ```
- Default naming: `{input_stem}_{sprite_name}.png`
- Handle `-o file.png` vs `-o directory/`

**Verification:**
```bash
cargo test output              # Unit tests pass
# Test: generate_output_path("input.jsonl", "hero", None) → "input_hero.png"
# Test: generate_output_path("input.jsonl", "hero", Some("out.png")) → "out.png"
# Test: generate_output_path("input.jsonl", "hero", Some("dir/")) → "dir/hero.png"
```

**Reference:** `docs/spec/format.md` - Output Behavior

**Dependencies:** Task 0.7

---

### Task 0.9: CLI Implementation

**Wave:** 7 (after 0.8)

Implement the `pxl` command-line interface.

**Deliverables:**
- `src/cli.rs` with clap:
  ```rust
  #[derive(Parser)]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }
  enum Commands {
      Render { input: PathBuf, output: Option<PathBuf>, sprite: Option<String>, strict: bool }
  }
  ```
- Wire up: parse → resolve → render → save
- Warnings to stderr, exit codes per spec

**Verification:**
```bash
cargo build --release
./target/release/pxl render examples/coin.jsonl
ls examples/coin_coin.png      # File exists
./target/release/pxl render examples/coin.jsonl -o /tmp/test.png
ls /tmp/test.png               # File exists
./target/release/pxl render tests/fixtures/lenient/unknown_token.jsonl 2>&1 | grep -i warning
# Should print warning to stderr
./target/release/pxl render tests/fixtures/lenient/unknown_token.jsonl --strict && echo "FAIL" || echo "PASS"
# Should exit non-zero in strict mode
```

**Updates demo.sh:** Demo now shows `pxl render` working

**Reference:** `docs/spec/format.md` - Output Behavior, Exit Codes

**Dependencies:** Task 0.8

---

### Task 0.10: Integration Tests & Demo

**Wave:** 8 (final)

End-to-end tests and demo script.

**Deliverables:**
- `tests/integration_tests.rs`:
  ```rust
  #[test] fn test_valid_fixtures_render() { /* all valid/* files */ }
  #[test] fn test_invalid_fixtures_error() { /* all invalid/* files */ }
  #[test] fn test_lenient_fixtures_warn() { /* all lenient/* files succeed with warnings */ }
  #[test] fn test_strict_mode_fails_on_warnings() { /* lenient/* files fail with --strict */ }
  ```
- `demo.sh` - executable demo showing Phase 0 features

**Verification:**
```bash
cargo test --test integration_tests
./demo.sh                      # Runs successfully, shows output
```

**Dependencies:** Task 0.9

---

## demo.sh

Create at project root - updated by each phase:

```bash
#!/bin/bash
# TTP Demo Script - Shows current capabilities
# Updated: Phase 0

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    TTP (Text To Pixel)                       ║"
echo "║                    Demo - Phase 0 MVP                        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Build if needed
if [ ! -f target/release/pxl ]; then
    echo "Building pxl..."
    cargo build --release --quiet
fi

PXL="./target/release/pxl"

echo "── Example 1: Simple Coin Sprite ──────────────────────────────"
echo "Input: examples/coin.jsonl"
head -2 examples/coin.jsonl
echo "..."
echo ""
$PXL render examples/coin.jsonl -o /tmp/demo_coin.png
echo "Output: /tmp/demo_coin.png"
echo "Dimensions: $(file /tmp/demo_coin.png | grep -oE '[0-9]+ x [0-9]+')"
echo ""

echo "── Example 2: Character Sprite ────────────────────────────────"
echo "Input: examples/hero.jsonl"
$PXL render examples/hero.jsonl -o /tmp/demo_hero.png
echo "Output: /tmp/demo_hero.png"
echo "Dimensions: $(file /tmp/demo_hero.png | grep -oE '[0-9]+ x [0-9]+')"
echo ""

echo "── Example 3: Multiple Sprites ────────────────────────────────"
$PXL render tests/fixtures/valid/multiple_sprites.jsonl -o /tmp/demo_
ls /tmp/demo_*.png 2>/dev/null | head -5
echo ""

echo "── Example 4: Lenient Mode (with warnings) ────────────────────"
echo "Input has unknown token - lenient mode renders as magenta:"
$PXL render tests/fixtures/lenient/unknown_token.jsonl -o /tmp/demo_lenient.png 2>&1 || true
echo ""

echo "── Example 5: Strict Mode ─────────────────────────────────────"
echo "Same input with --strict flag - should fail:"
$PXL render tests/fixtures/lenient/unknown_token.jsonl --strict -o /tmp/demo_strict.png 2>&1 || echo "(Expected failure)"
echo ""

echo "══════════════════════════════════════════════════════════════"
echo "Phase 0 Complete! Features:"
echo "  ✓ Parse JSONL palette and sprite definitions"
echo "  ✓ Render sprites to PNG"
echo "  ✓ Named and inline palettes"
echo "  ✓ Lenient mode (fill gaps, warn, continue)"
echo "  ✓ Strict mode (fail on warnings)"
echo ""
echo "Coming in Phase 1: Built-in palettes (@gameboy, @nes, @pico8)"
echo "══════════════════════════════════════════════════════════════"
```

---

## Verification Summary

After Phase 0 completion, verify:

```bash
# 1. Build
cargo build --release

# 2. Unit tests
cargo test

# 3. Lint
cargo clippy -- -D warnings

# 4. Integration tests
cargo test --test integration_tests

# 5. Demo
chmod +x demo.sh
./demo.sh

# 6. Manual inspection
open /tmp/demo_coin.png        # or: xdg-open, display, etc.
```

---

## Agent Checklist

Before marking a task complete, the agent MUST:

1. ✅ Read referenced spec sections
2. ✅ Implement all deliverables
3. ✅ Run verification commands - all must pass
4. ✅ Ensure `cargo clippy` has no warnings
5. ✅ Test with relevant fixtures
6. ✅ Update demo.sh if noted in task
