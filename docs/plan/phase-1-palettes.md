# Phase 1: Palette Library

**Goal:** Built-in palettes and palette sharing system

**Status:** Planning

**Depends on:** Phase 0 complete

---

## Scope

Phase 1 adds:
- Built-in palettes bundled in binary (gameboy, nes, pico-8, etc.)
- `@name` syntax to reference built-in palettes
- CLI commands for palette discovery
- External palette file inclusion

**Not in scope:** Animation, game engine export

---

## Task Dependency Diagram

```
                              PHASE 1 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                      Phase 0 Complete                           │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Parallel - Can Run Simultaneously)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐                            │
    │  │   1.1        │  │   1.4        │                            │
    │  │  Built-in    │  │  External    │                            │
    │  │  Palette     │  │  Include     │                            │
    │  │  Data        │  │              │                            │
    │  └──────┬───────┘  └──────────────┘                            │
    └─────────┼───────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (After Built-in Data)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐                            │
    │  │   1.2        │  │   1.3        │                            │
    │  │  @name       │  │  Palette     │  Can run in parallel       │
    │  │  Resolution  │  │  CLI         │                            │
    │  └──────────────┘  └──────────────┘                            │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 1.1 + 1.4             (2 tasks in parallel)            │
    │  Wave 2: 1.2 + 1.3             (2 tasks in parallel, need 1.1)  │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 1.1: Built-in Palette Data

**Wave:** 1 (parallel with 1.4)

Define built-in palette data.

**Deliverables:**
- `src/palettes/mod.rs`:
  ```rust
  pub fn get_builtin(name: &str) -> Option<Palette>
  pub fn list_builtins() -> Vec<&'static str>
  ```
- Embedded palette definitions:
  - `gameboy` - 4-color green (#9BBC0F, #8BAC0F, #306230, #0F380F)
  - `nes` - NES color palette (subset of key colors)
  - `pico8` - PICO-8 16-color palette
  - `grayscale` - 8 shades from white to black
  - `1bit` - black and white only

**Verification:**
```bash
cargo test palettes
# Test: get_builtin("gameboy") returns correct colors
# Test: list_builtins() includes all palettes
# Test: get_builtin("nonexistent") returns None
```

**Reference:** https://lospec.com/palette-list for authentic color values

**Dependencies:** Phase 0 complete

---

### Task 1.2: Built-in Palette Resolution

**Wave:** 2 (after 1.1)

Support `@name` syntax in palette references.

**Deliverables:**
- Update `src/registry.rs`:
  ```rust
  // In resolve():
  // If palette starts with "@", look up in builtins
  pub fn resolve(&self, sprite: &Sprite) -> Result<ResolvedPalette, Warning>
  ```

**Verification:**
```bash
cargo test registry
# Test: "palette": "@gameboy" resolves correctly
# Test: "palette": "@nonexistent" → warning + fallback
```

**Test fixture needed:**
```jsonl
{"type": "sprite", "name": "test", "palette": "@gameboy", "grid": ["{lightest}{dark}"]}
```

**Dependencies:** Task 1.1

---

### Task 1.3: Palette CLI Commands

**Wave:** 2 (parallel with 1.2, after 1.1)

Add palette discovery commands.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  enum Commands {
      Render { ... },
      Palettes { #[command(subcommand)] action: PaletteAction },
  }
  enum PaletteAction {
      List,
      Show { name: String },
  }
  ```
- `pxl palettes list` - list available built-in palettes
- `pxl palettes show <name>` - show palette colors with hex values

**Verification:**
```bash
./target/release/pxl palettes list
# Should list: gameboy, nes, pico8, grayscale, 1bit
./target/release/pxl palettes show gameboy
# Should show color names and hex values
```

**Updates demo.sh:** Add palette listing to demo

**Dependencies:** Task 1.1

---

### Task 1.4: External Palette Include

**Wave:** 1 (parallel with 1.1)

Support including external palette files.

**Deliverables:**
- Update palette resolution to handle `@include:path`:
  ```rust
  // "palette": "@include:./shared/colors.jsonl"
  pub fn resolve_include(path: &Path, base: &Path) -> Result<Palette, Error>
  ```
- Relative paths resolved from including file's directory
- Detect and error on circular includes

**Verification:**
```bash
# Create test file structure:
# shared/palette.jsonl: {"type": "palette", "name": "shared", ...}
# test.jsonl: {"type": "sprite", "palette": "@include:shared/palette.jsonl", ...}
cargo test include
./target/release/pxl render test.jsonl
```

**Dependencies:** Phase 0 complete

---

## demo.sh Updates

After Phase 1, demo.sh shows:

```bash
echo "── Phase 1: Built-in Palettes ─────────────────────────────────"
$PXL palettes list
echo ""
$PXL palettes show gameboy
echo ""
echo "Using @gameboy palette:"
echo '{"type": "sprite", "palette": "@gameboy", "grid": ["{lightest}{dark}"]}'
# Render and show
```

---

## Verification Summary

```bash
# 1. All Phase 0 tests still pass
cargo test

# 2. Palette commands work
./target/release/pxl palettes list
./target/release/pxl palettes show gameboy

# 3. @name syntax works
echo '{"type": "sprite", "name": "test", "palette": "@gameboy", "grid": ["{lightest}{dark}{lightest}"]}' | \
  ./target/release/pxl render - -o /tmp/test.png

# 4. Demo updated
./demo.sh
```
