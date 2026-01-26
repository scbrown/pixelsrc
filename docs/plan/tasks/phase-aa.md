# Phase AA: Semantic-Aware Antialiasing

**Goal:** Optional antialiasing using semantic roles for intelligent smoothing

**Status:** Planning

**Spec:** `docs/plan/phase-semantic-antialiasing.md`

---

## Algorithm Review

### Selected Algorithms

| Algorithm | Scale | Complexity | Notes |
|-----------|-------|------------|-------|
| `nearest` | Any | Trivial | Default, no AA (passthrough) |
| `scale2x` | 2x | Simple | EPX variant, pattern-based edge detection |
| `hq2x` | 2x | Medium | LUT-based, 256 patterns, smooth curves |
| `hq4x` | 4x | Medium | HQ2x applied twice with refinement |
| `xbr2x` | 2x | Complex | Edge direction/curvature, best quality |
| `xbr4x` | 4x | Complex | XBR2x with 4x output |
| `aa-blur` | Any | Simple | Gaussian blur with semantic mask |

### Considered But Deferred

| Algorithm | Reason |
|-----------|--------|
| Eagle | Superseded by Scale2x |
| 2xSaI | Similar quality to HQ2x, more complex |
| Super2xSaI | Diminishing returns over HQ4x |
| AdvMAME | Essentially Scale2x variants |
| Neural (ESRGAN) | Requires ML runtime, out of scope |

### Semantic Enhancement Matrix

| Semantic Info | scale2x | hq2x/4x | xbr2x/4x | aa-blur |
|---------------|---------|---------|----------|---------|
| **Anchor role** | Skip pixel | Skip pixel | Skip pixel | Mask=0% |
| **Boundary role** | Edge hint | Edge priority | Edge lock | Mask=25% |
| **Fill role** | Normal | Normal | Normal | Mask=100% |
| **DerivesFrom** | Post-gradient | Blend hint | Blend hint | Gradient |
| **ContainedWithin** | Hard edge | No cross-blend | Hard edge | Mask edge |
| **AdjacentTo** | Blend candidate | Color group | Blend zone | Blend zone |
| **PairedWith** | Mirror AA | Mirror AA | Mirror AA | Mirror AA |

---

## Task Dependency Diagram

```
                          PHASE AA TASK FLOW
═══════════════════════════════════════════════════════════════════════════

WAVE 1 (Foundation) - No dependencies
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌────────────────┐  ┌────────────────┐                               │
│   │   AA-1         │  │   AA-2         │                               │
│   │   Core Types   │  │   Config       │                               │
│   │   & Enums      │  │   Schema       │                               │
│   └───────┬────────┘  └───────┬────────┘                               │
│           │                   │                                         │
│           └─────────┬─────────┘                                         │
│                     │                                                   │
└─────────────────────┼───────────────────────────────────────────────────┘
                      │
                      ▼
WAVE 2 (Context & Pipeline) - Depends on Wave 1
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
│   │   AA-3         │  │   AA-4         │  │   AA-5         │           │
│   │   Semantic     │  │   CLI Flags    │  │   Pipeline     │           │
│   │   Context      │  │                │  │   Integration  │           │
│   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
│           │                   │                   │                     │
│           └───────────────────┼───────────────────┘                     │
│                               │                                         │
└───────────────────────────────┼─────────────────────────────────────────┘
                                │
                                ▼
WAVE 3 (Algorithms - Parallel) - Depends on Wave 2
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
│   │   AA-6         │  │   AA-7         │  │   AA-8         │           │
│   │   aa-blur      │  │   scale2x      │  │   hq2x/hq4x    │           │
│   │   Algorithm    │  │   Algorithm    │  │   Algorithms   │           │
│   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
│           │                   │                   │                     │
│           │           ┌────────────────┐          │                     │
│           │           │   AA-9         │          │                     │
│           │           │   xbr2x/xbr4x  │          │                     │
│           │           │   Algorithms   │          │                     │
│           │           └───────┬────────┘          │                     │
│           │                   │                   │                     │
│           └───────────────────┼───────────────────┘                     │
│                               │                                         │
└───────────────────────────────┼─────────────────────────────────────────┘
                                │
                                ▼
WAVE 4 (Advanced Features) - Depends on Wave 3
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌────────────────┐  ┌────────────────┐                               │
│   │   AA-10        │  │   AA-11        │                               │
│   │   Gradient     │  │   Per-Sprite   │                               │
│   │   Smoothing    │  │   Config       │                               │
│   └───────┬────────┘  └───────┬────────┘                               │
│           │                   │                                         │
│           └─────────┬─────────┘                                         │
│                     │                                                   │
└─────────────────────┼───────────────────────────────────────────────────┘
                      │
                      ▼
WAVE 5 (Demo & Docs) - Depends on Wave 4
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
│   │   AA-12        │  │   AA-13        │  │   AA-14        │           │
│   │   Demo Tests   │  │   Example      │  │   mdbook       │           │
│   │   (tests/)     │  │   Files        │  │   Docs         │           │
│   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
│           │                   │                   │                     │
│           └───────────────────┼───────────────────┘                     │
│                               │                                         │
│                               ▼                                         │
│                       ┌────────────────┐                                │
│                       │   AA-15        │                                │
│                       │   Integration  │                                │
│                       │   Tests        │                                │
│                       └────────────────┘                                │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────────────┐
│  Wave 1:  AA-1 ∥ AA-2                    (2 tasks in parallel)         │
│  Wave 2:  AA-3 ∥ AA-4 ∥ AA-5             (3 tasks in parallel)         │
│  Wave 3:  AA-6 ∥ AA-7 ∥ AA-8 ∥ AA-9      (4 tasks in parallel)         │
│  Wave 4:  AA-10 ∥ AA-11                  (2 tasks in parallel)         │
│  Wave 5:  AA-12 ∥ AA-13 ∥ AA-14 → AA-15                                │
│                                                                         │
│  Legend: ∥ = parallel, → = sequential dependency                        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Wave 1: Foundation

### AA-1 Core Types & Enums

**Parallel:** Yes (with AA-2)
**Files:** `src/antialias/mod.rs` (new)

Define core types for the antialiasing system.

**Deliverables:**
```rust
// src/antialias/mod.rs

pub mod algorithms;
pub mod context;
pub mod gradient;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AAAlgorithm {
    #[default]
    Nearest,
    Scale2x,
    Hq2x,
    Hq4x,
    Xbr2x,
    Xbr4x,
    AaBlur,
}

impl AAAlgorithm {
    pub fn scale_factor(&self) -> u8 {
        match self {
            AAAlgorithm::Nearest => 1,
            AAAlgorithm::Scale2x => 2,
            AAAlgorithm::Hq2x => 2,
            AAAlgorithm::Hq4x => 4,
            AAAlgorithm::Xbr2x => 2,
            AAAlgorithm::Xbr4x => 4,
            AAAlgorithm::AaBlur => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnchorMode {
    #[default]
    Preserve,  // No AA on anchors
    Reduce,    // 25% AA strength
    Normal,    // Full AA
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AntialiasConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub algorithm: AAAlgorithm,
    #[serde(default = "default_strength")]
    pub strength: f32,
    #[serde(default)]
    pub anchor_mode: AnchorMode,
    #[serde(default = "default_true")]
    pub gradient_shadows: bool,
    #[serde(default = "default_true")]
    pub respect_containment: bool,
    #[serde(default)]
    pub semantic_aware: bool,  // true by default, --no-semantic-aa disables
    #[serde(default)]
    pub regions: Option<HashMap<String, RegionAAOverride>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegionAAOverride {
    pub preserve: Option<bool>,
    pub mode: Option<AnchorMode>,
    pub gradient: Option<bool>,
}

fn default_strength() -> f32 { 0.5 }
fn default_true() -> bool { true }
```

**Verification:**
```bash
cargo build
# Types compile without errors
```

---

### AA-2 Config Schema

**Parallel:** Yes (with AA-1)
**Files:** `src/config/schema.rs`

Add antialiasing to project configuration.

**Deliverables:**
```rust
// In src/config/schema.rs

// Add to DefaultsConfig:
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DefaultsConfig {
    pub scale: Option<u8>,
    pub padding: Option<u32>,
    #[serde(default)]
    pub antialias: Option<AntialiasConfig>,  // ADD THIS
}

// Add to AtlasConfig:
#[derive(Debug, Clone, Deserialize)]
pub struct AtlasConfig {
    pub sources: Vec<String>,
    // ... existing fields ...
    #[serde(default)]
    pub antialias: Option<AntialiasConfig>,  // ADD THIS
}
```

**Verification:**
```toml
# Test pxl.toml parses correctly:
[defaults.antialias]
enabled = true
algorithm = "hq2x"
strength = 0.7
```

```bash
pxl validate examples/test.pxl
# Should not error on antialias config
```

---

## Wave 2: Context & Pipeline

### AA-3 Semantic Context Extraction

**Parallel:** Yes (with AA-4, AA-5)
**Depends:** AA-1
**Files:** `src/antialias/context.rs` (new)

Extract semantic information into a format algorithms can query.

**Deliverables:**
```rust
// src/antialias/context.rs

use crate::models::{Role, RelationshipType};
use image::Rgba;
use std::collections::{HashMap, HashSet};

pub struct SemanticContext {
    /// Pixels belonging to each role
    pub role_masks: HashMap<Role, HashSet<(i32, i32)>>,

    /// Quick lookup: is this pixel an anchor?
    pub anchor_pixels: HashSet<(i32, i32)>,

    /// Containment boundaries - hard edges
    pub containment_edges: HashSet<(i32, i32)>,

    /// Gradient pairs from DerivesFrom relationships
    pub gradient_pairs: Vec<GradientPair>,

    /// Adjacent region boundaries
    pub adjacencies: Vec<AdjacencyInfo>,
}

pub struct GradientPair {
    pub source_token: String,
    pub target_token: String,
    pub source_color: Rgba<u8>,
    pub target_color: Rgba<u8>,
    pub boundary_pixels: Vec<(i32, i32)>,
}

pub struct AdjacencyInfo {
    pub region_a: String,
    pub region_b: String,
    pub shared_edges: Vec<(i32, i32)>,
}

impl SemanticContext {
    pub fn empty() -> Self { ... }

    pub fn is_anchor(&self, pos: (i32, i32)) -> bool {
        self.anchor_pixels.contains(&pos)
    }

    pub fn is_containment_edge(&self, pos: (i32, i32)) -> bool {
        self.containment_edges.contains(&pos)
    }

    pub fn get_role(&self, pos: (i32, i32)) -> Option<Role> { ... }

    pub fn get_gradient_at(&self, pos: (i32, i32)) -> Option<&GradientPair> { ... }
}

/// Extract semantic context from rendered regions
pub fn extract_semantic_context(
    regions: &HashMap<String, RenderedRegion>,
    palette: &Palette,
    canvas_size: (u32, u32),
) -> SemanticContext { ... }
```

**Verification:**
```rust
#[test]
fn test_extract_anchor_pixels() {
    // Create regions with anchor role
    // Verify anchor_pixels contains correct positions
}

#[test]
fn test_extract_containment_edges() {
    // Create ContainedWithin relationship
    // Verify boundary pixels detected
}
```

---

### AA-4 CLI Flags

**Parallel:** Yes (with AA-3, AA-5)
**Depends:** AA-1, AA-2
**Files:** `src/cli/mod.rs`

Add CLI flags for antialiasing control.

**Deliverables:**
```rust
// In Commands::Render

/// Apply antialiasing algorithm
#[arg(long, value_enum)]
antialias: Option<AAAlgorithm>,

/// Antialiasing strength 0.0-1.0
#[arg(long, default_value = "0.5")]
aa_strength: f32,

/// How to handle anchor regions
#[arg(long, value_enum, default_value = "preserve")]
anchor_mode: AnchorMode,

/// Disable semantic awareness (apply algorithm globally)
#[arg(long)]
no_semantic_aa: bool,

/// Enable gradient smoothing for shadow/highlight
#[arg(long)]
gradient_shadows: bool,
```

**Verification:**
```bash
pxl render test.pxl --antialias hq2x --aa-strength 0.8 --help
# Should show all AA options

pxl render test.pxl --antialias invalid
# Should error with valid options list
```

---

### AA-5 Pipeline Integration

**Parallel:** Yes (with AA-3, AA-4)
**Depends:** AA-1, AA-2
**Files:** `src/cli/render.rs`, `src/output.rs`

Integrate antialiasing into the render pipeline.

**Deliverables:**
```rust
// src/output.rs

/// Apply antialiasing to an image with semantic awareness
pub fn apply_antialias(
    image: &RgbaImage,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> Result<RgbaImage, AAError> {
    if !config.enabled || config.algorithm == AAAlgorithm::Nearest {
        return Ok(image.clone());
    }

    match config.algorithm {
        AAAlgorithm::Nearest => Ok(image.clone()),
        AAAlgorithm::AaBlur => algorithms::blur::apply(image, config, context),
        AAAlgorithm::Scale2x => algorithms::scale2x::apply(image, config, context),
        AAAlgorithm::Hq2x => algorithms::hqx::apply_2x(image, config, context),
        AAAlgorithm::Hq4x => algorithms::hqx::apply_4x(image, config, context),
        AAAlgorithm::Xbr2x => algorithms::xbr::apply_2x(image, config, context),
        AAAlgorithm::Xbr4x => algorithms::xbr::apply_4x(image, config, context),
    }
}
```

```rust
// src/cli/render.rs - in render loop

let (mut image, warnings) = render_resolved(&sprite_data);

// Apply antialiasing if configured
let aa_config = resolve_aa_config(&config, &sprite, &cli_args);
if aa_config.enabled {
    let ctx = if aa_config.semantic_aware {
        extract_semantic_context(&rendered_regions, &palette, image.dimensions())
    } else {
        SemanticContext::empty()
    };
    image = apply_antialias(&image, &aa_config, &ctx)?;
}

// Then apply existing scale
if scale > 1 {
    image = scale_image(image, scale);
}
```

**Verification:**
```bash
# Pipeline runs without AA (default)
pxl render test.pxl -o out.png

# Pipeline runs with AA flag (even if algorithm not yet implemented)
pxl render test.pxl --antialias nearest -o out.png
```

---

## Wave 3: Algorithms

### AA-6 aa-blur Algorithm

**Parallel:** Yes (with AA-7, AA-8, AA-9)
**Depends:** AA-3, AA-4, AA-5
**Files:** `src/antialias/algorithms/blur.rs` (new)

Implement Gaussian blur with semantic masking.

**Deliverables:**
```rust
// src/antialias/algorithms/blur.rs

pub fn apply(
    image: &RgbaImage,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> Result<RgbaImage, AAError> {
    let (w, h) = image.dimensions();
    let mut result = image.clone();

    // Generate blur mask from semantic context
    let mask = generate_blur_mask(w, h, config, context);

    // Apply gaussian blur weighted by mask
    for y in 0..h {
        for x in 0..w {
            let mask_value = mask[(x, y)] as f32 / 255.0;
            if mask_value < 0.01 {
                continue; // No blur for this pixel
            }

            let effective_strength = config.strength * mask_value;
            let blurred = gaussian_sample(image, x, y, effective_strength);
            result.put_pixel(x, y, blurred);
        }
    }

    Ok(result)
}

fn generate_blur_mask(
    w: u32, h: u32,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> GrayImage {
    let mut mask = GrayImage::from_pixel(w, h, Luma([128])); // 50% default

    // Anchor: 0% blur (or reduced based on anchor_mode)
    for &pos in &context.anchor_pixels {
        let value = match config.anchor_mode {
            AnchorMode::Preserve => 0,
            AnchorMode::Reduce => 64,  // 25%
            AnchorMode::Normal => 128, // 50%
        };
        if pos.0 >= 0 && pos.1 >= 0 {
            mask.put_pixel(pos.0 as u32, pos.1 as u32, Luma([value]));
        }
    }

    // Boundary: 25% blur
    if let Some(boundary_pixels) = context.role_masks.get(&Role::Boundary) {
        for &pos in boundary_pixels {
            if pos.0 >= 0 && pos.1 >= 0 {
                mask.put_pixel(pos.0 as u32, pos.1 as u32, Luma([64]));
            }
        }
    }

    // Fill: 100% blur
    if let Some(fill_pixels) = context.role_masks.get(&Role::Fill) {
        for &pos in fill_pixels {
            if pos.0 >= 0 && pos.1 >= 0 {
                mask.put_pixel(pos.0 as u32, pos.1 as u32, Luma([255]));
            }
        }
    }

    // Containment edges: 0% blur
    for &pos in &context.containment_edges {
        if pos.0 >= 0 && pos.1 >= 0 {
            mask.put_pixel(pos.0 as u32, pos.1 as u32, Luma([0]));
        }
    }

    mask
}
```

**Verification:**
```bash
pxl render hero.pxl --antialias aa-blur --aa-strength 0.5 -o blur.png
# Should produce subtly smoothed output
# Anchor regions should remain crisp
```

---

### AA-7 scale2x Algorithm

**Parallel:** Yes (with AA-6, AA-8, AA-9)
**Depends:** AA-3, AA-4, AA-5
**Files:** `src/antialias/algorithms/scale2x.rs` (new)

Implement Scale2x (EPX) with semantic awareness.

**Deliverables:**
```rust
// src/antialias/algorithms/scale2x.rs

/// Scale2x algorithm with semantic context
///
/// For each pixel P with neighbors:
///   A
/// C P B
///   D
///
/// Output 2x2 block:
/// E0 E1
/// E2 E3
///
/// Rules:
/// - E0 = (C==A && C!=D && A!=B) ? A : P
/// - E1 = (A==B && A!=C && B!=D) ? B : P
/// - E2 = (D==C && D!=B && C!=A) ? C : P
/// - E3 = (B==D && B!=A && D!=C) ? D : P

pub fn apply(
    image: &RgbaImage,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> Result<RgbaImage, AAError> {
    let (w, h) = image.dimensions();
    let mut result = RgbaImage::new(w * 2, h * 2);

    for y in 0..h {
        for x in 0..w {
            let pos = (x as i32, y as i32);
            let p = *image.get_pixel(x, y);

            // Anchor preservation
            if context.is_anchor(pos) && config.anchor_mode == AnchorMode::Preserve {
                fill_2x2(&mut result, x * 2, y * 2, p);
                continue;
            }

            // Get neighbors (clamped to edges)
            let a = get_pixel_clamped(image, x, y.saturating_sub(1));
            let b = get_pixel_clamped(image, x + 1, y);
            let c = get_pixel_clamped(image, x.saturating_sub(1), y);
            let d = get_pixel_clamped(image, x, y + 1);

            // Check containment - don't blend across boundaries
            let (e0, e1, e2, e3) = if context.is_containment_edge(pos) {
                (p, p, p, p)  // No interpolation
            } else {
                scale2x_interpolate(p, a, b, c, d)
            };

            result.put_pixel(x * 2, y * 2, e0);
            result.put_pixel(x * 2 + 1, y * 2, e1);
            result.put_pixel(x * 2, y * 2 + 1, e2);
            result.put_pixel(x * 2 + 1, y * 2 + 1, e3);
        }
    }

    Ok(result)
}

fn scale2x_interpolate(p: Rgba<u8>, a: Rgba<u8>, b: Rgba<u8>, c: Rgba<u8>, d: Rgba<u8>) -> (Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>) {
    let e0 = if colors_equal(c, a) && !colors_equal(c, d) && !colors_equal(a, b) { a } else { p };
    let e1 = if colors_equal(a, b) && !colors_equal(a, c) && !colors_equal(b, d) { b } else { p };
    let e2 = if colors_equal(d, c) && !colors_equal(d, b) && !colors_equal(c, a) { c } else { p };
    let e3 = if colors_equal(b, d) && !colors_equal(b, a) && !colors_equal(d, c) { d } else { p };
    (e0, e1, e2, e3)
}
```

**Verification:**
```bash
pxl render hero.pxl --antialias scale2x -o scale2x.png
# Output should be 2x size with smoothed edges
# Compare with nearest-neighbor 2x for difference
```

---

### AA-8 hq2x/hq4x Algorithms

**Parallel:** Yes (with AA-6, AA-7, AA-9)
**Depends:** AA-3, AA-4, AA-5
**Files:** `src/antialias/algorithms/hqx.rs` (new)

Implement HQ2x and HQ4x algorithms.

**Deliverables:**
- HQ2x lookup table (256 patterns → interpolation rules)
- YUV color comparison for similarity detection
- Semantic context integration
- HQ4x as HQ2x applied with refinement

```rust
// src/antialias/algorithms/hqx.rs

/// HQ2x uses a 3x3 neighborhood to determine 256 patterns
/// Each pattern maps to a specific interpolation for the 2x2 output

pub fn apply_2x(
    image: &RgbaImage,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> Result<RgbaImage, AAError> {
    let (w, h) = image.dimensions();
    let mut result = RgbaImage::new(w * 2, h * 2);

    for y in 0..h {
        for x in 0..w {
            let pos = (x as i32, y as i32);

            // Anchor handling
            if context.is_anchor(pos) && config.anchor_mode == AnchorMode::Preserve {
                let p = *image.get_pixel(x, y);
                fill_2x2(&mut result, x * 2, y * 2, p);
                continue;
            }

            // Get 3x3 neighborhood
            let neighbors = get_3x3_neighborhood(image, x, y);

            // Calculate pattern index based on color similarity
            let pattern = calculate_pattern(&neighbors, context, pos);

            // Apply interpolation from lookup table
            let (e0, e1, e2, e3) = HQ2X_LUT[pattern as usize](&neighbors);

            result.put_pixel(x * 2, y * 2, e0);
            result.put_pixel(x * 2 + 1, y * 2, e1);
            result.put_pixel(x * 2, y * 2 + 1, e2);
            result.put_pixel(x * 2 + 1, y * 2 + 1, e3);
        }
    }

    Ok(result)
}

pub fn apply_4x(
    image: &RgbaImage,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> Result<RgbaImage, AAError> {
    // Apply 2x twice with context scaling
    let intermediate = apply_2x(image, config, context)?;
    let scaled_context = context.scale(2);
    apply_2x(&intermediate, config, &scaled_context)
}
```

**Verification:**
```bash
pxl render hero.pxl --antialias hq2x -o hq2x.png
pxl render hero.pxl --antialias hq4x -o hq4x.png
# Should produce smoother results than scale2x
```

---

### AA-9 xbr2x/xbr4x Algorithms

**Parallel:** Yes (with AA-6, AA-7, AA-8)
**Depends:** AA-3, AA-4, AA-5
**Files:** `src/antialias/algorithms/xbr.rs` (new)

Implement xBR algorithms (highest quality).

**Deliverables:**
- Edge direction detection
- Edge curvature analysis
- Sub-pixel positioning
- Semantic boundary respect

```rust
// src/antialias/algorithms/xbr.rs

/// xBR (scale By Rules) analyzes edge direction and curvature
/// for the highest quality upscaling

pub fn apply_2x(
    image: &RgbaImage,
    config: &AntialiasConfig,
    context: &SemanticContext,
) -> Result<RgbaImage, AAError> {
    let (w, h) = image.dimensions();
    let mut result = RgbaImage::new(w * 2, h * 2);

    for y in 0..h {
        for x in 0..w {
            let pos = (x as i32, y as i32);

            // Anchor handling
            if context.is_anchor(pos) && config.anchor_mode == AnchorMode::Preserve {
                let p = *image.get_pixel(x, y);
                fill_2x2(&mut result, x * 2, y * 2, p);
                continue;
            }

            // Get 5x5 neighborhood for edge analysis
            let neighbors = get_5x5_neighborhood(image, x, y);

            // Detect edge direction
            let edge_info = detect_edge(&neighbors, context, pos);

            // Apply xBR interpolation based on edge
            let (e0, e1, e2, e3) = xbr_interpolate(&neighbors, &edge_info);

            result.put_pixel(x * 2, y * 2, e0);
            result.put_pixel(x * 2 + 1, y * 2, e1);
            result.put_pixel(x * 2, y * 2 + 1, e2);
            result.put_pixel(x * 2 + 1, y * 2 + 1, e3);
        }
    }

    Ok(result)
}
```

**Verification:**
```bash
pxl render hero.pxl --antialias xbr2x -o xbr2x.png
pxl render hero.pxl --antialias xbr4x -o xbr4x.png
# Should produce highest quality results
# Diagonal lines should be particularly smooth
```

---

## Wave 4: Advanced Features

### AA-10 Gradient Smoothing

**Parallel:** Yes (with AA-11)
**Depends:** Wave 3
**Files:** `src/antialias/gradient.rs` (new)

Implement gradient smoothing for DerivesFrom relationships.

**Deliverables:**
```rust
// src/antialias/gradient.rs

/// Apply gradient smoothing at shadow/highlight boundaries
pub fn apply_gradient_smoothing(
    image: &mut RgbaImage,
    context: &SemanticContext,
    config: &AntialiasConfig,
) {
    if !config.gradient_shadows {
        return;
    }

    for gradient in &context.gradient_pairs {
        // Find boundary pixels between the two colors
        for &(x, y) in &gradient.boundary_pixels {
            // Calculate blend factor based on neighborhood
            let blend = calculate_blend_factor(image, x, y, gradient);

            // Interpolate color using the derives-from relationship
            let interpolated = interpolate_gradient(
                &gradient.source_color,
                &gradient.target_color,
                blend,
            );

            image.put_pixel(x as u32, y as u32, interpolated);
        }
    }
}

/// Interpolate between colors preserving hue relationship
fn interpolate_gradient(from: &Rgba<u8>, to: &Rgba<u8>, t: f32) -> Rgba<u8> {
    // Use LAB or OKLCH for perceptually uniform interpolation
    let from_lab = rgb_to_lab(from);
    let to_lab = rgb_to_lab(to);
    let result_lab = lerp_lab(&from_lab, &to_lab, t);
    lab_to_rgb(&result_lab)
}
```

**Verification:**
```bash
pxl render hero.pxl --antialias hq4x --gradient-shadows -o gradient.png
# Shadow/highlight transitions should be smooth
# Compare with --no-gradient-shadows for difference
```

---

### AA-11 Per-Sprite Config

**Parallel:** Yes (with AA-10)
**Depends:** Wave 3
**Files:** `src/models/sprite.rs`

Add per-sprite antialiasing configuration.

**Deliverables:**
```rust
// src/models/sprite.rs

#[derive(Debug, Clone, Deserialize)]
pub struct Sprite {
    pub name: String,
    pub size: [u32; 2],
    pub palette: String,
    // ... existing fields ...

    /// Per-sprite antialiasing configuration
    #[serde(default)]
    pub antialias: Option<AntialiasConfig>,
}
```

Add config resolution logic:
```rust
// src/cli/render.rs

fn resolve_aa_config(
    project_config: &PxlConfig,
    atlas_config: Option<&AtlasConfig>,
    sprite: &Sprite,
    cli_args: &RenderArgs,
) -> AntialiasConfig {
    // Start with defaults
    let mut config = project_config.defaults.antialias.clone()
        .unwrap_or_default();

    // Override with atlas config
    if let Some(atlas) = atlas_config {
        if let Some(ref atlas_aa) = atlas.antialias {
            config.merge(atlas_aa);
        }
    }

    // Override with sprite config
    if let Some(ref sprite_aa) = sprite.antialias {
        config.merge(sprite_aa);
    }

    // Override with CLI args
    if let Some(algo) = cli_args.antialias {
        config.algorithm = algo;
        config.enabled = true;
    }
    if cli_args.aa_strength != 0.5 {
        config.strength = cli_args.aa_strength;
    }
    // ... etc

    config
}
```

**Verification:**
```json
// Test sprite with AA override
{
  "type": "sprite",
  "name": "test",
  "antialias": {
    "enabled": true,
    "algorithm": "xbr4x",
    "regions": {
      "eye": { "preserve": true }
    }
  }
}
```

```bash
pxl render test.pxl -o out.png
# Should use sprite's AA config
```

---

## Wave 5: Demo & Docs

### AA-12 Demo Tests

**Parallel:** Yes (with AA-13, AA-14)
**Depends:** Wave 4
**Files:** `tests/demos/antialias/` (new directory)

Create demo tests for antialiasing features.

**Deliverables:**

```
tests/demos/antialias/
  mod.rs
  algorithms.rs      # Test each algorithm
  semantic.rs        # Test semantic awareness
  config.rs          # Test config resolution
```

```rust
// tests/demos/antialias/algorithms.rs

//! Antialiasing Algorithm Demo Tests
//!
//! Tests for each antialiasing algorithm: scale2x, hq2x, hq4x, xbr2x, xbr4x, aa-blur.

use crate::demos::{assert_validates, capture_render_info, parse_content};

/// @demo render/antialias#scale2x
/// @title Scale2x Algorithm
/// @description 2x upscaling with edge-aware interpolation.
#[test]
fn test_scale2x() {
    let jsonl = include_str!("../../../../examples/demos/antialias/algorithms.jsonl");
    assert_validates(jsonl, true);

    // Verify Scale2x produces 2x output dimensions
    let info = capture_render_info(jsonl, "test_sprite", Some("scale2x"));
    assert_eq!(info.width, 32);  // 16 * 2
    assert_eq!(info.height, 32);
}

/// @demo render/antialias#hq4x
/// @title HQ4x Algorithm
/// @description High-quality 4x upscaling with smooth curves.
#[test]
fn test_hq4x() {
    let jsonl = include_str!("../../../../examples/demos/antialias/algorithms.jsonl");
    assert_validates(jsonl, true);

    let info = capture_render_info(jsonl, "test_sprite", Some("hq4x"));
    assert_eq!(info.width, 64);  // 16 * 4
    assert_eq!(info.height, 64);
}

// ... tests for each algorithm
```

```rust
// tests/demos/antialias/semantic.rs

/// @demo render/antialias#anchor_preservation
/// @title Anchor Preservation
/// @description Anchor regions (eyes, details) are protected from smoothing.
#[test]
fn test_anchor_preservation() {
    let jsonl = include_str!("../../../../examples/demos/antialias/semantic.jsonl");
    // Test that anchor pixels remain unchanged after AA
}

/// @demo render/antialias#containment_edges
/// @title Containment Boundaries
/// @description ContainedWithin relationships create hard edges that prevent color bleeding.
#[test]
fn test_containment_edges() {
    // Test no blending across containment boundaries
}

/// @demo render/antialias#gradient_shadows
/// @title Gradient Smoothing
/// @description DerivesFrom relationships enable smooth shadow/highlight transitions.
#[test]
fn test_gradient_shadows() {
    // Test gradient interpolation
}
```

**Verification:**
```bash
cargo test demos::antialias
# All demo tests pass
```

---

### AA-13 Example Files

**Parallel:** Yes (with AA-12, AA-14)
**Depends:** Wave 4
**Files:** `examples/demos/antialias/` (new directory)

Create example JSONL files for demos.

**Deliverables:**

```
examples/demos/antialias/
  algorithms.jsonl    # Basic sprite for algorithm comparison
  semantic.jsonl      # Sprite with roles/relationships
  config.jsonl        # Config override examples
```

```jsonc
// examples/demos/antialias/semantic.jsonl

// Palette with semantic roles and relationships
{
  "type": "palette",
  "name": "semantic_demo",
  "colors": {
    "_": "transparent",
    "outline": "#2C1810",
    "skin": "#FFCC99",
    "skin_shadow": "#D4A574",
    "eye": "#000000"
  },
  "roles": {
    "outline": "boundary",
    "skin": "fill",
    "skin_shadow": "shadow",
    "eye": "anchor"
  },
  "relationships": {
    "skin_shadow": { "type": "derives-from", "target": "skin" },
    "eye": { "type": "contained-within", "target": "skin" }
  }
}

// Sprite demonstrating semantic AA
{
  "type": "sprite",
  "name": "semantic_face",
  "size": [16, 16],
  "palette": "semantic_demo",
  "regions": {
    "outline": { "stroke": [1, 1, 14, 14], "z": 10 },
    "skin": { "ellipse": [8, 8, 6, 7], "z": 0 },
    "skin_shadow": { "polygon": [[4, 10], [8, 12], [12, 10], [10, 8], [6, 8]], "z": 1 },
    "eye": { "points": [[6, 6], [10, 6]], "symmetric": "x", "z": 5 }
  }
}
```

**Verification:**
```bash
pxl validate examples/demos/antialias/*.jsonl
# All example files valid

pxl render examples/demos/antialias/semantic.jsonl --antialias hq4x -o /tmp/test.png
# Renders successfully
```

---

### AA-14 mdbook Documentation

**Parallel:** Yes (with AA-12, AA-13)
**Depends:** Wave 4
**Files:** `docs/book/src/format/antialias.md` (new), `docs/book/src/SUMMARY.md`

Create mdbook documentation for antialiasing.

**Deliverables:**

```markdown
<!-- docs/book/src/format/antialias.md -->

# Antialiasing

Pixelsrc supports optional antialiasing for smoother upscaled output.
By default, antialiasing is **disabled** to preserve crisp pixel art.

## Quick Start

```bash
# Apply HQ4x antialiasing
pxl render sprite.pxl --antialias hq4x --scale 4 -o smooth.png
```

## Algorithms

| Algorithm | Scale | Quality | Speed |
|-----------|-------|---------|-------|
| `nearest` | Any | - | Fastest |
| `scale2x` | 2x | Good | Fast |
| `hq2x` | 2x | Better | Medium |
| `hq4x` | 4x | Better | Medium |
| `xbr2x` | 2x | Best | Slower |
| `xbr4x` | 4x | Best | Slower |
| `aa-blur` | Any | Subtle | Fast |

## Semantic Awareness

Pixelsrc uses semantic roles to make intelligent antialiasing decisions:

### Anchor Preservation

Regions marked with `role: "anchor"` (like eyes) are protected:

```json
{
  "roles": { "eye": "anchor" }
}
```

Control anchor handling with `--anchor-mode`:
- `preserve` (default) - Never antialias anchors
- `reduce` - Apply 25% strength
- `normal` - Full antialiasing

### Gradient Smoothing

Colors with `derives-from` relationships blend smoothly:

```json
{
  "relationships": {
    "skin_shadow": { "type": "derives-from", "target": "skin" }
  }
}
```

Enable with `--gradient-shadows` (on by default).

### Containment Boundaries

`contained-within` relationships create hard edges:

```json
{
  "relationships": {
    "eye": { "type": "contained-within", "target": "face" }
  }
}
```

## Configuration

### CLI Flags

```bash
--antialias <ALGO>      # Algorithm selection
--aa-strength <0.0-1.0> # Intensity (default: 0.5)
--anchor-mode <MODE>    # preserve|reduce|normal
--no-semantic-aa        # Disable semantic awareness
--gradient-shadows      # Enable gradient smoothing
```

### pxl.toml

```toml
[defaults.antialias]
enabled = true
algorithm = "hq4x"
strength = 0.7
anchor_mode = "preserve"
```

### Per-Sprite

```json
{
  "type": "sprite",
  "name": "hero",
  "antialias": {
    "enabled": true,
    "algorithm": "xbr4x",
    "regions": {
      "eye": { "preserve": true }
    }
  }
}
```

## Examples

[Include rendered comparison images]
```

Update SUMMARY.md:
```markdown
- [Antialias](./format/antialias.md)
```

**Verification:**
```bash
cd docs/book && mdbook build
# No errors, antialias page renders correctly
```

---

### AA-15 Integration Tests

**Parallel:** No
**Depends:** AA-12, AA-13, AA-14
**Files:** `tests/antialias_integration.rs` (new)

End-to-end integration tests.

**Deliverables:**
```rust
// tests/antialias_integration.rs

#[test]
fn test_aa_disabled_by_default() {
    // Render without AA flag
    // Verify output matches nearest-neighbor scaling
}

#[test]
fn test_aa_cli_override() {
    // pxl.toml sets algorithm
    // CLI flag overrides
    // Verify CLI takes precedence
}

#[test]
fn test_aa_preserves_anchors() {
    // Render sprite with anchor regions
    // Verify anchor pixels unchanged after AA
}

#[test]
fn test_aa_respects_containment() {
    // Render sprite with containment relationship
    // Verify no color bleeding across boundary
}

#[test]
fn test_aa_gradient_smoothing() {
    // Render sprite with derives-from relationship
    // Verify gradient interpolation at boundary
}

#[test]
fn test_all_algorithms_produce_correct_scale() {
    for algo in [Scale2x, Hq2x, Hq4x, Xbr2x, Xbr4x, AaBlur] {
        // Verify output dimensions match expected scale factor
    }
}
```

**Verification:**
```bash
cargo test antialias_integration
# All integration tests pass
```

---

## Final Verification Checklist

```bash
# 1. Default behavior unchanged
pxl render test.pxl --scale 4 -o default.png
# Should match current output exactly (no AA)

# 2. All algorithms work
for algo in scale2x hq2x hq4x xbr2x xbr4x aa-blur; do
  pxl render hero.pxl --antialias $algo -o /tmp/$algo.png
done
# All produce valid output

# 3. Semantic features work
pxl render semantic.pxl --antialias hq4x --anchor-mode preserve -o preserved.png
pxl render semantic.pxl --antialias hq4x --gradient-shadows -o gradient.png
# Anchors preserved, gradients smooth

# 4. Config precedence works
# With pxl.toml algorithm=scale2x:
pxl render hero.pxl --antialias hq4x -o override.png
# Should use hq4x (CLI wins)

# 5. Demo tests pass
cargo test demos::antialias

# 6. Integration tests pass
cargo test antialias_integration

# 7. Documentation builds
cd docs/book && mdbook build
```
