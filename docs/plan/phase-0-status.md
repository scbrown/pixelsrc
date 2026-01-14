# Phase 0 Status

**Date:** 2026-01-14
**Status:** Complete

---

## Summary

Phase 0 MVP is complete. TTP can parse JSONL files and render sprites to PNG.

---

## Task Status

| Task | Description | Status |
|------|-------------|--------|
| 0.1 | Project Scaffolding | ✅ Complete |
| 0.2 | Data Models | ✅ Complete |
| 0.3 | Color Parsing | ✅ Complete |
| 0.4 | Token Parsing | ✅ Complete |
| 0.5 | JSONL Parser | ✅ Complete |
| 0.6 | Palette Registry | ✅ Complete |
| 0.7 | Sprite Renderer | ✅ Complete |
| 0.8 | PNG Output | ✅ Complete |
| 0.9 | CLI Implementation | ✅ Complete |
| 0.10 | Integration Tests | ✅ Complete |

---

## Capabilities

### CLI Usage
```bash
# Basic render
pxl render input.jsonl

# Output to specific file
pxl render input.jsonl -o output.png

# Output to directory
pxl render input.jsonl -o ./sprites/

# Strict mode (fail on warnings)
pxl render input.jsonl --strict
```

### Supported Formats
- **Colors:** `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`
- **Palettes:** Named references or inline definitions
- **Sprites:** Grid-based pixel definitions with size inference

### Error Handling
- **Lenient mode (default):** Warns and continues, unknown tokens render as magenta
- **Strict mode:** Fails on any warning

---

## Test Coverage

- Unit tests for all modules
- Integration tests for valid/invalid/lenient fixtures
- Doc tests for public APIs

```bash
cargo test                        # All tests
cargo test --test integration     # Integration only
cargo clippy -- -D warnings       # Lint check
```

---

## Examples

```bash
pxl render examples/heart.jsonl   # Simple 7x6 sprite
pxl render examples/coin.jsonl    # 8x8 with named palette
pxl render examples/hero.jsonl    # 16x16 character
```

---

## Next: Phase 1

Built-in palettes:
- `@gameboy` - Classic 4-color green
- `@nes` - NES system palette
- `@pico8` - PICO-8 16-color palette
- `@c64` - Commodore 64 palette
