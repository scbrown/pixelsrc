# Phase 0 Gap Analysis

**Date:** 2026-01-14
**Status:** Incomplete — 4/10 tasks done

---

## Summary

Phase 0 foundation is solid (models, colors, tokens) but the entire pipeline from "read file" to "write PNG" is unimplemented.

---

## Task Status

| Task | Description | Status | Notes |
|------|-------------|--------|-------|
| 0.1 | Project Scaffolding | ✅ Complete | Cargo.toml, module stubs |
| 0.2 | Data Models | ✅ Complete | Palette, Sprite, TtpObject with tests |
| 0.3 | Color Parsing | ✅ Complete | #RGB, #RGBA, #RRGGBB, #RRGGBBAA |
| 0.4 | Token Parsing | ✅ Complete | Extracts {tokens}, warns on extras |
| 0.5 | JSONL Parser | ❌ Empty stub | `parse_line()`, `parse_stream()` |
| 0.6 | Palette Registry | ❌ Empty stub | `PaletteRegistry`, resolve refs |
| 0.7 | Sprite Renderer | ❌ Empty stub | `render_sprite()` → RgbaImage |
| 0.8 | PNG Output | ❌ Empty stub | `save_png()`, path generation |
| 0.9 | CLI Implementation | ❌ Empty stub | `pxl render` command |
| 0.10 | Integration Tests | ❌ Missing | End-to-end test suite |

---

## What's Working

### Models (`src/models.rs`)
- `Palette`, `Sprite`, `TtpObject` structs
- Serde serialization/deserialization
- 8 unit tests passing

### Color Parsing (`src/color.rs`)
- All hex formats: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`
- Case insensitive
- Proper error types
- 12 unit tests passing

### Tokenizer (`src/tokenizer.rs`)
- Extracts `{name}` tokens from grid strings
- Warns on unexpected characters
- Handles unclosed braces
- 7 unit tests passing

### Test Fixtures
```
tests/fixtures/
├── valid/           # 7 files - should parse and render
├── invalid/         # 7 files - should fail to parse
└── lenient/         # 5 files - should warn but succeed
```

---

## What's Missing

### 0.5 JSONL Parser (`src/parser.rs`)
```rust
// Required:
pub fn parse_line(line: &str) -> Result<TtpObject, ParseError>
pub fn parse_stream<R: BufRead>(reader: R) -> ParseResult
pub struct ParseResult { objects: Vec<TtpObject>, warnings: Vec<Warning> }
```
- Skip blank lines
- Collect warnings in lenient mode

### 0.6 Palette Registry (`src/registry.rs`)
```rust
// Required:
pub struct PaletteRegistry { palettes: HashMap<String, Palette> }
impl PaletteRegistry {
    pub fn register(&mut self, palette: Palette)
    pub fn resolve(&self, sprite: &Sprite) -> Result<ResolvedPalette, Warning>
}
pub struct ResolvedPalette { colors: HashMap<String, Rgba<u8>> }
```
- Inline palettes pass through
- Missing refs → magenta fallback + warning

### 0.7 Sprite Renderer (`src/renderer.rs`)
```rust
// Required:
pub fn render_sprite(sprite: &Sprite, palette: &ResolvedPalette) -> (RgbaImage, Vec<Warning>)
```
- Unknown tokens → magenta `#FF00FF`
- Size inference when `size` omitted
- Row padding/truncation

### 0.8 PNG Output (`src/output.rs`)
```rust
// Required:
pub fn save_png(image: &RgbaImage, path: &Path) -> Result<(), OutputError>
pub fn generate_output_path(input: &Path, sprite_name: &str, output_arg: Option<&Path>) -> PathBuf
```
- Default: `{input_stem}_{sprite_name}.png`
- Handle `-o file.png` vs `-o directory/`

### 0.9 CLI (`src/cli.rs`)
```rust
// Required:
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
- Warnings to stderr
- Exit codes per spec

### 0.10 Integration Tests
```rust
// Required in tests/integration_tests.rs:
#[test] fn test_valid_fixtures_render() { /* all valid/* files */ }
#[test] fn test_invalid_fixtures_error() { /* all invalid/* files */ }
#[test] fn test_lenient_fixtures_warn() { /* all lenient/* files */ }
#[test] fn test_strict_mode_fails_on_warnings() { /* --strict flag */ }
```

---

## Critical Path

```
0.5 Parser ──► 0.6 Registry ──► 0.7 Renderer ──► 0.8 Output ──► 0.9 CLI ──► 0.10 Tests
```

All 6 remaining tasks are sequential dependencies.

---

## Verification Checklist

After completion, all must pass:

```bash
cargo build --release
cargo test
cargo clippy -- -D warnings
cargo test --test integration_tests
./demo.sh
```
