# Phase 3: Animation

**Goal:** Multi-frame sprites, spritesheet and GIF export

**Status:** Planning

**Depends on:** Phase 2 complete

---

## Scope

Phase 3 adds:
- Animation type parsing
- Spritesheet generation (grid of frames)
- Animated GIF export
- Frame timing support

**Not in scope:** Game engine export, video formats

---

## Task Dependency Diagram

```
                              PHASE 3 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                      Phase 2 Complete                           │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Foundation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐                                               │
    │  │   3.1        │                                               │
    │  │  Animation   │  Add Animation struct to models               │
    │  │  Model       │                                               │
    │  └──────┬───────┘                                               │
    └─────────┼───────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - After Model)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
    │  │   3.2        │  │   3.3        │  │   3.4        │          │
    │  │  Animation   │  │  Spritesheet │  │  GIF         │          │
    │  │  Validation  │  │  Renderer    │  │  Renderer    │          │
    │  └──────────────┘  └──────────────┘  └──────────────┘          │
    └─────────────────────────────────────────────────────────────────┘
              │                 │                 │
              └────────────────┬┴─────────────────┘
                               │
                               ▼
    WAVE 3 (CLI Integration)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐                                               │
    │  │   3.5        │  --gif, --spritesheet, --animation flags      │
    │  │  Animation   │                                               │
    │  │  CLI         │                                               │
    │  └──────────────┘                                               │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 3.1                   (1 task)                         │
    │  Wave 2: 3.2 + 3.3 + 3.4       (3 tasks in parallel)            │
    │  Wave 3: 3.5                   (1 task, needs 3.2-3.4)          │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 3.1: Animation Model

**Wave:** 1

Add animation type to data models.

**Deliverables:**
- Update `src/models.rs`:
  ```rust
  pub struct Animation {
      pub name: String,
      pub frames: Vec<String>,  // Sprite names
      pub duration: u32,        // ms per frame, default 100
      pub loop_anim: bool,      // default true
  }

  pub enum TtpObject {
      Palette(Palette),
      Sprite(Sprite),
      Animation(Animation),
  }
  ```

**Verification:**
```bash
cargo test models
# Test: Animation JSON deserializes correctly
# Test: Default duration is 100
# Test: Default loop is true
```

**Test Fixture:** `tests/fixtures/valid/animation.jsonl`

**Dependencies:** Phase 0 complete

---

### Task 3.2: Animation Validation

**Wave:** 2 (parallel with 3.3, 3.4)

Validate animation references.

**Deliverables:**
- `src/animation.rs`:
  ```rust
  pub fn validate_animation(anim: &Animation, sprites: &[Sprite]) -> Vec<Warning>
  // Warns if frame references unknown sprite
  // Warns if no frames
  ```

**Verification:**
```bash
cargo test animation
# Test: Valid animation with existing sprites → no warnings
# Test: Animation with missing sprite → warning
# Test: Empty frames → warning
```

**Dependencies:** Task 3.1

---

### Task 3.3: Spritesheet Renderer

**Wave:** 2 (parallel with 3.2, 3.4)

Render multiple sprites into a grid.

**Deliverables:**
- `src/spritesheet.rs`:
  ```rust
  pub fn render_spritesheet(frames: &[RgbaImage], cols: Option<u32>) -> RgbaImage
  // Horizontal layout by default
  // All frames same size (pad smaller ones)
  ```

**Verification:**
```bash
cargo test spritesheet
# Test: 4 frames → 4x1 spritesheet
# Test: Different sized frames → padded to largest
# Test: Custom columns (cols=2) → 2x2 grid
```

**Dependencies:** Task 3.1

---

### Task 3.4: GIF Renderer

**Wave:** 2 (parallel with 3.2, 3.3)

Render animation as GIF.

**Deliverables:**
- `src/gif.rs`:
  ```rust
  pub fn render_gif(frames: &[RgbaImage], duration_ms: u32, loop_anim: bool, path: &Path) -> Result<()>
  // Uses image crate's GIF encoder
  ```

**Verification:**
```bash
cargo test gif
# Test: Creates valid GIF file
# Test: Frame duration matches input
# Test: Loop setting is respected
```

**Dependencies:** Task 3.1

---

### Task 3.5: Animation CLI

**Wave:** 3 (after 3.2, 3.3, 3.4)

Add animation CLI options.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  Render {
      // existing fields...
      #[arg(long)]
      gif: bool,
      #[arg(long)]
      spritesheet: bool,
      #[arg(long)]
      animation: Option<String>,  // Select specific animation
  }
  ```

**Verification:**
```bash
./target/release/pxl render examples/walk_cycle.jsonl --spritesheet -o /tmp/sheet.png
file /tmp/sheet.png  # Should show wide image

./target/release/pxl render examples/walk_cycle.jsonl --gif -o /tmp/walk.gif
file /tmp/walk.gif   # Should be GIF

./target/release/pxl render examples/walk_cycle.jsonl --animation walk --gif -o /tmp/walk.gif
# Renders only the "walk" animation
```

**Updates demo.sh:** Add animation examples

**Dependencies:** Tasks 3.2, 3.3, 3.4

---

## demo.sh Updates

After Phase 3, demo.sh shows:

```bash
echo "── Phase 3: Animation ─────────────────────────────────────────"
echo "Input: examples/walk_cycle.jsonl"
cat examples/walk_cycle.jsonl | head -5
echo "..."
echo ""
$PXL render examples/walk_cycle.jsonl --spritesheet -o /tmp/demo_sheet.png
echo "Spritesheet: /tmp/demo_sheet.png"
$PXL render examples/walk_cycle.jsonl --gif -o /tmp/demo_walk.gif
echo "Animation: /tmp/demo_walk.gif"
```

---

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. Animation fixture parses
./target/release/pxl render tests/fixtures/valid/animation.jsonl

# 3. Spritesheet output
./target/release/pxl render examples/walk_cycle.jsonl --spritesheet -o /tmp/sheet.png
open /tmp/sheet.png

# 4. GIF output
./target/release/pxl render examples/walk_cycle.jsonl --gif -o /tmp/walk.gif
open /tmp/walk.gif  # Should animate

# 5. Demo updated
./demo.sh
```
