# Phase 2.5: Output Upscaling

**Goal:** Integer scaling for pixel-perfect output enlargement

**Status:** Planning

**Depends on:** Phase 1 complete (can run before or parallel to Phase 2)

---

## Scope

Phase 2.5 adds output scaling - a standard feature for pixel art tools that enlarges rendered sprites by an integer factor while preserving crisp pixel edges.

**Features:**
- `--scale N` CLI flag for integer upscaling (1-16)
- Nearest-neighbor interpolation (no blur/smoothing)
- Works with all sprite types

**Not in scope:** Source upscaling, non-integer scaling, interpolation modes

---

## Rationale

Pixel art is designed at small resolutions (8x8, 16x16, 32x32) but often needs to be displayed larger:
- Game engines may need 2x or 4x assets for different display resolutions
- Preview/sharing images benefit from larger sizes
- Web display typically needs at least 2x-4x scaling

This is a standard feature in pixel art tools (Aseprite, PICO-8, Pixelorama) and a common user request.

---

## Task Dependency Diagram

```
                          PHASE 2.5 TASK FLOW
═══════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────┐
│                      Phase 1 Complete                           │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 1 (Foundation)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   2.5.1 Scale Implementation                             │   │
│  │   - Add --scale flag to CLI                              │   │
│  │   - Implement scaling in output module                   │   │
│  │   - Nearest-neighbor via image crate                     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (Integration)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   2.5.2 Tests & Examples                                 │   │
│  │   - Unit tests for scaling                               │   │
│  │   - CLI integration tests                                │   │
│  │   - Update demo.sh with scaling examples                 │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 2.5.1              (1 task - foundation)               │
│  Wave 2: 2.5.2              (1 task - needs 2.5.1)              │
│                                                                 │
│  No parallelization: 2.5.2 depends on 2.5.1 completion          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 2.5.1: Scale Implementation

**Wave:** 1 (no parallelization possible)

Add `--scale` flag and implement nearest-neighbor scaling.

**Deliverables:**

1. Update `src/cli.rs` - add `--scale` argument:
   ```rust
   /// Render sprites from a Pixelsrc JSONL file to PNG
   Render {
       // ... existing args ...

       /// Scale output by integer factor (1-16, default: 1)
       #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=16))]
       scale: u8,
   }
   ```

2. Update `src/output.rs` - add scaling function:
   ```rust
   use image::imageops::FilterType;

   /// Scale image by integer factor using nearest-neighbor interpolation
   pub fn scale_image(image: RgbaImage, factor: u8) -> RgbaImage {
       if factor <= 1 {
           return image;
       }
       let (w, h) = image.dimensions();
       let new_w = w * factor as u32;
       let new_h = h * factor as u32;
       image::imageops::resize(&image, new_w, new_h, FilterType::Nearest)
   }
   ```

3. Update `run_render()` in `src/cli.rs`:
   ```rust
   // After render_sprite(), before save_png():
   let image = if scale > 1 {
       scale_image(image, scale)
   } else {
       image
   };
   ```

**Verification:**
```bash
# Build
cargo build --release

# Test scaling factors
./target/release/pxl render examples/hero.jsonl --scale 1 -o /tmp/hero_1x.png
./target/release/pxl render examples/hero.jsonl --scale 2 -o /tmp/hero_2x.png
./target/release/pxl render examples/hero.jsonl --scale 4 -o /tmp/hero_4x.png

# Verify dimensions (assuming 16x16 source)
file /tmp/hero_1x.png  # Should show 16x16
file /tmp/hero_2x.png  # Should show 32x32
file /tmp/hero_4x.png  # Should show 64x64

# Test bounds
./target/release/pxl render examples/hero.jsonl --scale 0  # Should error
./target/release/pxl render examples/hero.jsonl --scale 17 # Should error

# Test with strict mode
./target/release/pxl render examples/hero.jsonl --scale 2 --strict -o /tmp/
```

**Dependencies:** Phase 1 complete

---

### Task 2.5.2: Tests & Examples

**Wave:** 2 (after 2.5.1)

Add tests and update demo.

**Deliverables:**

1. Add unit tests in `src/output.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_scale_1x_returns_same() {
           let img = RgbaImage::new(16, 16);
           let scaled = scale_image(img.clone(), 1);
           assert_eq!(scaled.dimensions(), (16, 16));
       }

       #[test]
       fn test_scale_2x_doubles_dimensions() {
           let img = RgbaImage::new(16, 16);
           let scaled = scale_image(img, 2);
           assert_eq!(scaled.dimensions(), (32, 32));
       }

       #[test]
       fn test_scale_4x_quadruples_dimensions() {
           let img = RgbaImage::new(8, 12);
           let scaled = scale_image(img, 4);
           assert_eq!(scaled.dimensions(), (32, 48));
       }

       #[test]
       fn test_scale_preserves_pixel_colors() {
           let mut img = RgbaImage::new(2, 2);
           img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));   // Red
           img.put_pixel(1, 0, Rgba([0, 255, 0, 255]));   // Green
           img.put_pixel(0, 1, Rgba([0, 0, 255, 255]));   // Blue
           img.put_pixel(1, 1, Rgba([255, 255, 0, 255])); // Yellow

           let scaled = scale_image(img, 2);

           // Each original pixel becomes 2x2 block
           assert_eq!(scaled.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
           assert_eq!(scaled.get_pixel(1, 1), &Rgba([255, 0, 0, 255]));
           assert_eq!(scaled.get_pixel(2, 0), &Rgba([0, 255, 0, 255]));
           assert_eq!(scaled.get_pixel(3, 1), &Rgba([0, 255, 0, 255]));
       }
   }
   ```

2. Add CLI integration test in `tests/cli_scale.rs`:
   ```rust
   use assert_cmd::Command;
   use predicates::prelude::*;
   use tempfile::tempdir;

   #[test]
   fn test_scale_flag_accepted() {
       let dir = tempdir().unwrap();
       let output = dir.path().join("test.png");

       Command::cargo_bin("pxl")
           .unwrap()
           .args(["render", "examples/hero.jsonl", "--scale", "2", "-o"])
           .arg(&output)
           .assert()
           .success();

       assert!(output.exists());
   }

   #[test]
   fn test_scale_out_of_range_errors() {
       Command::cargo_bin("pxl")
           .unwrap()
           .args(["render", "examples/hero.jsonl", "--scale", "0"])
           .assert()
           .failure();

       Command::cargo_bin("pxl")
           .unwrap()
           .args(["render", "examples/hero.jsonl", "--scale", "20"])
           .assert()
           .failure();
   }
   ```

3. Update `demo.sh`:
   ```bash
   echo ""
   echo "── Phase 2.5: Output Scaling ───────────────────────────────────"
   echo ""
   echo "Scaling sprites for display:"
   echo "  pxl render hero.jsonl --scale 1  → 16x16 (original)"
   echo "  pxl render hero.jsonl --scale 2  → 32x32 (2x)"
   echo "  pxl render hero.jsonl --scale 4  → 64x64 (4x)"
   echo ""
   $PXL render examples/hero.jsonl --scale 4 -o /tmp/hero_4x.png
   echo "Rendered: /tmp/hero_4x.png (4x scale)"
   ```

**Verification:**
```bash
# Run tests
cargo test scale
cargo test cli_scale

# Run demo
./demo.sh
```

**Dependencies:** Task 2.5.1

---

## Example Usage

```bash
# Default (1x) - original pixel dimensions
pxl render sprite.jsonl -o sprite.png

# 2x scale - common for retina/HiDPI displays
pxl render sprite.jsonl --scale 2 -o sprite_2x.png

# 4x scale - good for previews and sharing
pxl render sprite.jsonl --scale 4 -o sprite_4x.png

# 8x scale - large display/thumbnails
pxl render sprite.jsonl --scale 8 -o sprite_8x.png

# Works with all other flags
pxl render sprite.jsonl --scale 4 --sprite hero --strict -o hero_4x.png
```

---

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. Scale flag works
./target/release/pxl render examples/hero.jsonl --scale 2 -o /tmp/hero_2x.png
./target/release/pxl render examples/hero.jsonl --scale 4 -o /tmp/hero_4x.png

# 3. Verify output dimensions
file /tmp/hero_2x.png
file /tmp/hero_4x.png

# 4. Invalid scale values rejected
./target/release/pxl render examples/hero.jsonl --scale 0 2>&1 | grep -i error
./target/release/pxl render examples/hero.jsonl --scale 100 2>&1 | grep -i error

# 5. Demo updated
./demo.sh
```

---

## Future Considerations

Features considered but not included in this phase:

| Feature | Rationale for Deferral |
|---------|------------------------|
| Non-integer scaling | Pixel art rarely needs 1.5x; adds interpolation complexity |
| Interpolation modes | Nearest-neighbor is correct for pixel art; other modes blur |
| Per-sprite scaling | Can be added later if needed; CLI flag is simpler |
| Source upscaling | Defeats compact format purpose; not a real use case |
| Max scale > 16 | 16x is already 256x256 from 16x16; larger is rarely needed |
