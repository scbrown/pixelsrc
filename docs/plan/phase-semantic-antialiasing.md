# Phase: Semantic-Aware Antialiasing & Upscaling

**Goal:** Optional antialiasing that uses semantic roles/relationships for intelligent smoothing

**Status:** Planning

**Depends on:** Phase 2.5 (Output Upscaling) complete

**Tasks:** `docs/plan/tasks/phase-aa.md`

---

## Summary

Add optional antialiasing/smooth upscaling that leverages pixelsrc's semantic role system to make intelligent smoothing decisions. Unlike traditional AA that treats all pixels equally, semantic-aware AA knows which pixels to preserve, smooth, or blend based on their meaning.

**Key Innovation:** Semantic context guides per-pixel AA decisions:
- **Anchors** (eyes, details) → Configurable preservation
- **Boundaries** (outlines) → Smart edge smoothing
- **Shadow/Highlight + DerivesFrom** → Gradient interpolation
- **ContainedWithin** → Prevents color bleeding across regions

---

## Architecture

### Processing Pipeline

```
Parse .pxl
  → Render regions to RGBA pixels
  → Extract SemanticContext from regions + palette
  → Apply Antialiasing (algorithms query SemanticContext)
  → Apply Scale (existing --scale)
  → Save PNG
```

### SemanticContext Structure

The bridge between semantic definitions and pixel-level algorithms:

```rust
pub struct SemanticContext {
    /// Pixels by role - for per-pixel decisions
    pub role_masks: HashMap<Role, HashSet<(i32, i32)>>,

    /// Anchor bounds for preservation (from existing extract_anchor_bounds)
    pub anchor_bounds: Vec<AnchorBounds>,

    /// Hard edges from ContainedWithin - never blend across these
    pub containment_edges: HashSet<(i32, i32)>,

    /// Pixel pairs that should gradient-blend (from DerivesFrom)
    pub gradient_pairs: Vec<GradientPair>,

    /// Adjacent region boundaries for smart blending
    pub adjacencies: HashMap<(String, String), HashSet<(i32, i32)>>,
}

pub struct GradientPair {
    pub source_token: String,      // e.g., "skin_shadow"
    pub target_token: String,      // e.g., "skin"
    pub source_color: Rgba<u8>,
    pub target_color: Rgba<u8>,
    pub boundary_pixels: Vec<(i32, i32)>,  // Where they meet
}
```

### How Algorithms Use SemanticContext

```rust
fn apply_algorithm(image: &RgbaImage, ctx: &SemanticContext, config: &AAConfig) -> RgbaImage {
    let mut result = RgbaImage::new(/* scaled size */);

    for (x, y) in image.pixels() {
        let pos = (x as i32, y as i32);

        // 1. Check anchor - respect anchor_mode setting
        if ctx.is_anchor(pos) {
            match config.anchor_mode {
                AnchorMode::Preserve => { copy_exact(pos); continue; }
                AnchorMode::Reduce => { apply_at_25_percent(pos); }
                AnchorMode::Normal => { /* fall through */ }
            }
        }

        // 2. Check containment boundary - hard edge
        if ctx.is_containment_edge(pos) {
            apply_without_cross_blend(pos);
            continue;
        }

        // 3. Check gradient pair - smooth interpolation
        if let Some(gradient) = ctx.get_gradient_pair(pos) {
            apply_gradient_blend(pos, gradient);
            continue;
        }

        // 4. Normal algorithm processing
        apply_standard_algorithm(pos);
    }

    result
}
```

---

## Algorithms

### Available Algorithms

| Algorithm | Scale Factor | Description | Best For |
|-----------|--------------|-------------|----------|
| `nearest` | Any | No AA (default) | Preserving pixel art |
| `scale2x` | 2x | Pattern-based edge detection | General pixel art |
| `hq2x` | 2x | 3x3 neighborhood analysis with LUT | Smooth curves |
| `hq4x` | 4x | Higher quality version of hq2x | Detailed upscaling |
| `xbr2x` | 2x | Edge direction/curvature detection | Best quality 2x |
| `xbr4x` | 4x | Higher quality xBR | Best quality 4x |
| `aa-blur` | Any | Gaussian blur with semantic mask | Subtle smoothing |

### Semantic Enhancement Per Algorithm

| Role/Relationship | scale2x | hq2x/4x | xBR | aa-blur |
|-------------------|---------|---------|-----|---------|
| **Anchor** | Skip pixel | Skip pixel | Skip pixel | Mask = 0% |
| **Boundary** | Enhanced edge detect | Edge priority | Edge preservation | Mask = 25% |
| **Fill** | Normal processing | Normal | Normal | Mask = 100% |
| **DerivesFrom** | Post-process gradient | Gradient blend | Gradient blend | Gradient blend |
| **ContainedWithin** | Hard edge | No cross-blend | Hard edge | Mask boundary |
| **AdjacentTo** | Blend candidates | Color similarity | Blend candidates | Blend zone |

---

## Configuration

### 1. CLI Flags

```bash
pxl render sprite.pxl [OPTIONS]

Antialiasing Options:
  --antialias <ALGO>       Algorithm: nearest, scale2x, hq2x, hq4x, xbr2x, xbr4x, aa-blur
  --aa-strength <FLOAT>    Intensity 0.0-1.0 (default: 0.5)
  --anchor-mode <MODE>     How to handle anchors: preserve, reduce, normal (default: preserve)
  --no-semantic-aa         Disable semantic awareness (global algorithm)
  --gradient-shadows       Enable gradient smoothing for DerivesFrom (default: true)
```

**Examples:**
```bash
# Basic antialiasing
pxl render hero.pxl --antialias hq4x --scale 4

# Maximum smoothing, still preserve eyes
pxl render hero.pxl --antialias xbr4x --aa-strength 1.0 --anchor-mode preserve

# Subtle blur
pxl render hero.pxl --antialias aa-blur --aa-strength 0.3

# Ignore semantic info (traditional AA)
pxl render hero.pxl --antialias scale2x --no-semantic-aa
```

### 2. pxl.toml Project Config

```toml
[project]
name = "my-game"

[defaults]
scale = 2

# Global antialiasing defaults
[defaults.antialias]
enabled = false                 # Off by default - opt-in
algorithm = "hq2x"
strength = 0.5
anchor_mode = "preserve"        # preserve | reduce | normal
gradient_shadows = true
respect_containment = true

# Per-atlas override
[atlases.characters]
sources = ["sprites/characters/**"]
antialias = { enabled = true, algorithm = "hq4x", strength = 0.7 }

[atlases.ui]
sources = ["sprites/ui/**"]
antialias = { enabled = false }  # UI stays crisp
```

### 3. Per-Sprite .pxl Config

```jsonc
{
  "type": "sprite",
  "name": "hero_face",
  "size": [16, 16],
  "palette": "hero_pal",

  "antialias": {
    "enabled": true,
    "algorithm": "xbr4x",
    "strength": 0.8,
    "anchor_mode": "preserve",
    "gradient_shadows": true,

    // Per-region overrides
    "regions": {
      "eye_left": { "preserve": true },
      "eye_right": { "preserve": true },
      "outline": { "mode": "smooth" },
      "skin_shadow": { "gradient": true }
    }
  },

  "regions": {
    "outline": { "stroke": [1, 1, 14, 14] },
    "skin": { "fill": "inside(outline)" },
    "skin_shadow": { "rect": [4, 10, 8, 2] },
    "eye_left": { "points": [[5, 6]] },
    "eye_right": { "points": [[10, 6]] }
  }
}
```

### Configuration Precedence

1. Built-in defaults (antialias disabled)
2. pxl.toml `[defaults.antialias]`
3. pxl.toml per-atlas `[atlases.X.antialias]`
4. Per-sprite .pxl `"antialias": {}`
5. CLI flags (highest priority)

---

## Implementation

### New Module Structure

```
src/
  antialias/
    mod.rs                 # Public API, AntialiasConfig, AAAlgorithm enum
    context.rs             # SemanticContext extraction
    algorithms/
      mod.rs               # Algorithm trait definition
      nearest.rs           # No-op (passthrough)
      scale2x.rs           # Scale2x implementation
      hqx.rs               # HQ2x/HQ4x implementation
      xbr.rs               # xBR2x/xBR4x implementation
      blur.rs              # Gaussian blur with mask
    gradient.rs            # Gradient interpolation for DerivesFrom
```

### Core Types

```rust
// src/antialias/mod.rs

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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnchorMode {
    #[default]
    Preserve,   // No AA on anchors
    Reduce,     // 25% AA strength on anchors
    Normal,     // Treat anchors like other pixels
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntialiasConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub algorithm: AAAlgorithm,

    #[serde(default = "default_strength")]
    pub strength: f32,  // 0.0-1.0

    #[serde(default)]
    pub anchor_mode: AnchorMode,

    #[serde(default = "default_true")]
    pub gradient_shadows: bool,

    #[serde(default = "default_true")]
    pub respect_containment: bool,

    #[serde(default)]
    pub regions: Option<HashMap<String, RegionAAOverride>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAAOverride {
    pub preserve: Option<bool>,
    pub mode: Option<AnchorMode>,
    pub gradient: Option<bool>,
}

fn default_strength() -> f32 { 0.5 }
fn default_true() -> bool { true }
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/config/schema.rs` | Add `AntialiasConfig` to `DefaultsConfig`, `AtlasConfig` |
| `src/cli/mod.rs` | Add `--antialias`, `--aa-strength`, `--anchor-mode`, etc. |
| `src/cli/render.rs` | Integrate AA into pipeline between render and scale |
| `src/models/sprite.rs` | Add `antialias: Option<AntialiasConfig>` |
| `src/output.rs` | Add `apply_antialias()` entry point |
| `src/structured.rs` | Extend to export region pixel sets for SemanticContext |

### Pipeline Integration

```rust
// In src/cli/render.rs

// After rendering sprite to image
let (mut image, warnings) = render_resolved(&sprite_data);

// Get effective AA config (merge toml + sprite + cli)
let aa_config = resolve_aa_config(&config, &sprite, &cli_args);

// Apply antialiasing if enabled
if aa_config.enabled && aa_config.algorithm != AAAlgorithm::Nearest {
    // Extract semantic context from rendered regions
    let semantic_ctx = extract_semantic_context(
        &resolved_regions,
        &palette,
        (image.width(), image.height()),
    );

    // Apply the algorithm
    image = apply_antialias(&image, &aa_config, &semantic_ctx)?;
}

// Existing scaling
if scale > 1 {
    image = scale_image(image, scale);
}
```

---

## Task Breakdown

### Wave 1: Infrastructure (Foundation)

**Task AA-1: Core types and config**
- Add `AntialiasConfig`, `AAAlgorithm`, `AnchorMode` to new `src/antialias/mod.rs`
- Add to `src/config/schema.rs` DefaultsConfig
- Add CLI flags to `src/cli/mod.rs`
- Wire up config resolution

**Task AA-2: SemanticContext extraction**
- Create `src/antialias/context.rs`
- Extract role masks from rendered regions
- Extract gradient pairs from DerivesFrom relationships
- Extract containment edges from ContainedWithin

### Wave 2: Simplest Algorithm

**Task AA-3: aa-blur implementation**
- Gaussian blur with semantic mask
- Anchor pixels → mask = 0
- Boundary pixels → mask = 25%
- Fill pixels → mask = 100%
- Containment edges → mask = 0

**Task AA-4: Pipeline integration**
- Integrate into `src/cli/render.rs`
- Add `apply_antialias()` to `src/output.rs`
- Test with aa-blur

### Wave 3: Edge-Based Algorithms

**Task AA-5: scale2x implementation**
- Standard scale2x algorithm
- Semantic modifications: skip anchors, respect containment

**Task AA-6: hq2x/hq4x implementation**
- Lookup table based
- Semantic color similarity adjustments

**Task AA-7: xBR implementation**
- Edge direction detection
- Most complex, best quality

### Wave 4: Advanced Features

**Task AA-8: Gradient smoothing**
- DerivesFrom relationship detection
- Color interpolation at shadow/highlight boundaries
- Post-process pass after main algorithm

**Task AA-9: Per-sprite config**
- Add `antialias` field to sprite model
- Per-region overrides
- Config precedence resolution

### Wave 5: Polish

**Task AA-10: Documentation**
- Update docs/book with AA reference
- Add examples demonstrating each algorithm
- Document semantic role effects

**Task AA-11: Tests**
- Unit tests for each algorithm
- Integration tests for config resolution
- Visual regression tests

---

## Verification

```bash
# 1. Default behavior unchanged (AA disabled)
pxl render test.pxl -o crisp.png --scale 4
# Should match current output exactly

# 2. Basic AA works
pxl render hero.pxl --antialias aa-blur --aa-strength 0.5 -o blur.png
# Should show subtle smoothing

# 3. Semantic preservation works
pxl render hero.pxl --antialias hq4x --anchor-mode preserve -o preserved.png
# Eyes/anchors should remain crisp

# 4. Gradient smoothing works
pxl render hero.pxl --antialias xbr4x --gradient-shadows -o gradient.png
# Shadow/highlight transitions should be smooth

# 5. Config precedence works
# With pxl.toml setting algorithm=scale2x:
pxl render hero.pxl --antialias hq4x -o override.png
# Should use hq4x (CLI overrides toml)

# 6. No-semantic mode works
pxl render hero.pxl --antialias scale2x --no-semantic-aa -o global.png
# Should apply algorithm uniformly
```

---

## Dependencies

- `image` crate (already used) - pixel manipulation
- Consider `imageproc` for convolution kernels (optional)
- No major new dependencies - algorithms implemented in pure Rust

---

## Future Considerations

| Feature | Notes |
|---------|-------|
| Custom algorithm plugins | Allow user-defined AA algorithms |
| ML-based upscaling | ESRGAN, etc. - would need external deps |
| Per-frame animation AA | Cache SemanticContext across frames |
| Real-time preview | WebAssembly AA for playground |
