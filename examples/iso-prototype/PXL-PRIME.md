# PXL Prime: Artistic Workflow for Pixel Art Generation

## The Core Insight

Pixelsrc's structured regions work like **pre-rendered 3D sprites** (Fallout 2, Donkey Kong Country), not hand-drawn pixel art. You define surfaces and their lighting relationships; the renderer produces the pixels.

```
Traditional: 3D Model → Render → 2D Sprite
Pixelsrc:    Regions (surfaces + shading) → Render → 2D Sprite
```

This means: **Think in surfaces and light, not individual pixels.**

---

## Key Rule: Region Names = Palette Keys

Every region name must have a matching palette entry. The region name IS the color lookup key.

```json
// CORRECT: region names match palette entries
"palette": { "suit": "#3070a8", "suit_lit": "#4888c0" },
"regions": { "suit": {...}, "suit_lit": {...} }

// WRONG: mismatched names cause "Unknown token" warnings
"palette": { "body": "#3070a8" },
"regions": { "suit": {...} }  // "suit" not in palette!
```

For multiple regions with the same color, create multiple palette entries:
```json
"palette": { "leg_l": "#3070a8", "leg_r": "#3070a8" },
"regions": { "leg_l": {...}, "leg_r": {...} }
```

---

## Region Types and When to Use Them

| Type | Syntax | Best For |
|------|--------|----------|
| **Polygon** | `[[x,y], ...]` | Tapered forms, organic edges, clothing, limbs |
| **Ellipse** | `[cx, cy, rx, ry]` | Heads, rounded objects, organic details |
| **Rect** | `[x, y, w, h]` | Mechanical parts, belts, boots, screens |
| **Points** | `[[x,y], ...]` | Eyes, highlights, single-pixel accents |

### Critical: Avoid Rectangles for Organic Forms

**The "outsider art" mistake**: Using only `rect` creates blocky, robotic shapes. Human figures need organic curves and tapers.

```json
// BAD: All rectangles = blocky robot
"head": { "rect": [5, 0, 6, 6] },
"body": { "rect": [4, 8, 8, 8] }

// GOOD: Ellipse head, polygon tapered body
"head": { "ellipse": [8, 4, 4, 4] },
"body": { "polygon": [[4, 8], [12, 8], [10, 15], [6, 15]] }
```

---

## The Three-Surface Rule

Every 3D form has three visible surfaces under directional light:

```
        [LIT]        ← Facing light (brightest)
       /    \
   [BASE]  [SHADOW]  ← Side (medium) and away from light (darkest)
```

Define these as overlapping regions with related colors:

```json
"palette": {
  "suit_lit": "#4888c0",    // Brightest (right side)
  "suit": "#3070a8",        // Base
  "suit_shd": "#205080"     // Darkest (left side)
},
"regions": {
  "suit": { "polygon": [[4, 8], [12, 8], [10, 15], [6, 15]] },
  "suit_lit": { "polygon": [[10, 8], [12, 8], [10, 15], [9, 15]] },
  "suit_shd": { "polygon": [[4, 8], [6, 8], [6, 15], [5, 14]] }
}
```

The **COLOR DIFFERENCE** defines the edge, not a black outline.

---

## Character Sprite Proportions

For small sprites (16-24px tall), use exaggerated proportions for readability:

| Body Part | Proportion | Example (24px sprite) |
|-----------|------------|----------------------|
| Head | 1/3 of height | 8 pixels |
| Torso | 1/3 of height | 8 pixels |
| Legs + Feet | 1/3 of height | 8 pixels |

**Head must be large** - at small scale, a proportionally-correct head becomes invisible.

---

## Light Direction Convention

Standard: **Top-right at ~45°**

```
     Light source
          ↘
    [Lit surface]
   [Base]  [Shadow]
```

This means:
- Right-facing surfaces = lightest
- Top surfaces = light
- Left-facing surfaces = darkest
- Bottom surfaces = shadow

---

## Workflow: Character Sprite

### Phase 1: Silhouette Block-in

Establish the readable shape first. Test: fill with solid color - can you tell what it is?

```json
// Start with base shapes only
"skin": { "ellipse": [8, 4, 4, 4] },           // Head
"suit": { "polygon": [[4, 8], [12, 8], [10, 15], [6, 15]] },  // Tapered torso
"leg_l": { "polygon": [[5, 15], [7, 15], [7, 21], [5, 21]] }, // Left leg
"leg_r": { "polygon": [[9, 15], [11, 15], [11, 21], [9, 21]] } // Right leg (gap!)
```

**Key**: Leave a gap between legs for separation.

### Phase 2: Three-Surface Shading

Add lit and shadow regions over the base shapes:

```json
"skin_lit": { "polygon": [[9, 1], [11, 3], [11, 6], [9, 6]] },  // Face right
"skin_shd": { "polygon": [[5, 3], [7, 2], [7, 6], [5, 6]] },    // Face left
"suit_lit": { "polygon": [[10, 8], [12, 8], [10, 15], [9, 15]] }, // Body right
"suit_shd": { "polygon": [[4, 8], [6, 8], [6, 15], [5, 14]] }     // Body left
```

### Phase 3: Minimal Details

Only add what's essential for recognition (2-3 details max):

```json
"hair": { "ellipse": [8, 1, 4, 2] },      // Hair on top of head
"eye": { "points": [[6, 4], [10, 4]] },   // Just 2 pixels for eyes
"belt": { "rect": [5, 12, 6, 1] }         // Breaks up the torso
```

---

## Common Mistakes

### ❌ All Rectangles
Creates "outsider art" blocky look. Use ellipses for heads, polygons for bodies.

### ❌ Head Too Small
At small scale, proportionally-correct heads disappear. Make head 1/3 of sprite height.

### ❌ No Leg Separation
Legs that touch look like a single mass. Leave a gap (1-2 pixels) between legs.

### ❌ Too Many Details
At small scale, details become noise. 2-3 emphasized features max.

### ❌ Mismatched Names
Region names must exactly match palette keys or you get "Unknown token" warnings.

### ❌ Missing Z-Order and Roles
Without explicit `z` values or semantic `role` tags, regions render in undefined order. Important details (eyes, belt) get covered.

---

## Z-Ordering: Layer Control

### Semantic Roles: Automatic Z-Ordering

**The simplest approach**: Use semantic `role` tags and let the renderer infer z-order automatically.

| Role | Default Z | Purpose |
|------|-----------|---------|
| `anchor` | 100 | Critical details (eyes, belt buckle) - always on top |
| `boundary` | 80 | Outlines, edges - high priority |
| `shadow` / `highlight` | 60 | Shading overlays |
| `fill` | 40 | Interior mass (skin, clothing) |
| (no role) | 0 | Untagged regions |

```json
"regions": {
  // Fill renders at z=40 (automatic)
  "suit": { "polygon": [...], "role": "fill" },

  // Shadow renders at z=60 (above fill)
  "suit_shd": { "polygon": [...], "role": "shadow" },

  // Anchor renders at z=100 (on top of everything)
  "buckle": { "rect": [...], "role": "anchor" }
}
```

### Explicit Z-Order (Override)

When you need precise control, explicit `z` values override role-based defaults:

```json
"regions": {
  // Base layer (explicit z: 0)
  "suit": { "polygon": [...], "z": 0 },

  // Shading overlays (explicit z: 1)
  "suit_hi": { "polygon": [...], "z": 1 },
  "suit_shd": { "polygon": [...], "z": 1 },

  // Details on top (explicit z: 5+)
  "belt": { "polygon": [...], "z": 5 },
  "buckle": { "rect": [...], "z": 6 }
}
```

### Recommended Explicit Z Ranges

| Layer Type | Z Range | Examples |
|------------|---------|----------|
| Base shapes | 0-3 | Body, limbs, clothing base |
| Shading | 1-4 | Highlights, shadows |
| Accessories | 5-9 | Belt, pockets, straps |
| Head base | 10-11 | Skin, face shading |
| Hair | 12-13 | Hair base, highlights |
| Facial features | 14-17 | Eyes (white→iris→pupil), brows |
| Top details | 18+ | Glasses, hats, overlays |

### Eye Layering Example

Eyes require precise z-order. Use roles for simplicity, or explicit z for fine control:

```json
// Option 1: Using roles (simple)
"eye_l_w": { "ellipse": [25, 18, 5, 3], "role": "fill" },       // z=40 auto
"eye_l_iris": { "ellipse": [26, 18, 3, 3], "role": "boundary" }, // z=80 auto
"eye_l_pupil": { "ellipse": [26, 18, 1, 2], "role": "anchor" },  // z=100 auto

// Option 2: Explicit z (fine control)
"eye_l_w": { "ellipse": [25, 18, 5, 3], "z": 14 },      // White (back)
"eye_l_iris": { "ellipse": [26, 18, 3, 3], "z": 15 },   // Iris (middle)
"eye_l_pupil": { "ellipse": [26, 18, 1, 2], "z": 16 },  // Pupil (front)
```

### Debugging Z-Order Issues

If details are invisible:
1. Add a semantic `role` tag (anchor for critical details)
2. Or use explicit `z` value higher than overlapping regions
3. Verify region coordinates are within canvas bounds

---

## Testing Your Sprite

1. **Silhouette test**: Fill with solid black. Is it recognizable?
2. **Grayscale test**: Convert to grayscale. Does form read clearly?
3. **Scale test**: View at 1x. Does it read at actual size?
4. **Context test**: Place on intended background. Does it pop?

---

## Complete Example: Vault Dweller (64x96)

```json
{
  "type": "sprite",
  "name": "vault-dweller-large",
  "size": [64, 96],
  "palette": {
    "_": "#00000000",
    "skin": "#d8b088", "skin_hi": "#f0d0a8", "skin_shd": "#b08860",
    "hair": "#483020", "hair_hi": "#5a4030",
    "eye_l_w": "#f0f0f0", "eye_l_iris": "#4a7090", "eye_l_pupil": "#101010",
    "eye_r_w": "#f0f0f0", "eye_r_iris": "#4a7090", "eye_r_pupil": "#101010",
    "brow_l": "#3a2818", "brow_r": "#3a2818",
    "nose": "#c0a078",
    "mouth_top": "#a07058", "mouth_bot": "#8a5a48",
    "suit": "#3070a8", "suit_hi": "#4888c0", "suit_shd": "#205080",
    "leg_l": "#3070a8", "leg_l_shd": "#205080",
    "leg_r": "#3070a8", "leg_r_hi": "#4888c0",
    "belt": "#d0a040", "belt_hi": "#e8c060", "buckle": "#c0c0c0",
    "boot_l": "#403830", "boot_l_shd": "#302820",
    "boot_r": "#403830", "boot_r_hi": "#504840"
  },
  "regions": {
    // HEAD (z: 10-17)
    "skin": { "ellipse": [32, 18, 14, 16], "z": 10 },
    "skin_hi": { "polygon": [[38, 10], [44, 14], [44, 22], [40, 24], [36, 18]], "z": 11 },
    "skin_shd": { "polygon": [[20, 14], [24, 12], [26, 20], [24, 26], [20, 22]], "z": 11 },
    "hair": { "ellipse": [32, 6, 14, 8], "z": 12 },
    "hair_hi": { "polygon": [[36, 2], [42, 4], [44, 8], [38, 10], [34, 6]], "z": 13 },
    // EYES (z: 14-16, must layer correctly)
    "eye_l_w": { "ellipse": [25, 18, 5, 3], "z": 14 },
    "eye_l_iris": { "ellipse": [26, 18, 3, 3], "z": 15 },
    "eye_l_pupil": { "ellipse": [26, 18, 1, 2], "z": 16 },
    "eye_r_w": { "ellipse": [39, 18, 5, 3], "z": 14 },
    "eye_r_iris": { "ellipse": [38, 18, 3, 3], "z": 15 },
    "eye_r_pupil": { "ellipse": [38, 18, 1, 2], "z": 16 },
    "brow_l": { "polygon": [[20, 14], [30, 13], [30, 15], [20, 16]], "z": 17 },
    "brow_r": { "polygon": [[34, 13], [44, 14], [44, 16], [34, 15]], "z": 17 },
    "nose": { "polygon": [[30, 22], [34, 22], [33, 26], [31, 26]], "z": 13 },
    "mouth_top": { "polygon": [[27, 28], [37, 28], [36, 30], [28, 30]], "z": 13 },
    "mouth_bot": { "polygon": [[28, 30], [36, 30], [35, 32], [29, 32]], "z": 13 },
    // TORSO (z: 0-1)
    "suit": { "polygon": [[18, 36], [46, 36], [48, 40], [48, 56], [46, 62], [44, 64], [20, 64], [18, 62], [16, 56], [16, 40]], "z": 0 },
    "suit_hi": { "polygon": [[38, 38], [46, 38], [48, 44], [48, 54], [44, 60], [38, 60], [38, 48]], "z": 1 },
    "suit_shd": { "polygon": [[16, 40], [22, 40], [22, 60], [18, 62], [16, 56]], "z": 1 },
    // BELT (z: 5-7, on top of suit)
    "belt": { "polygon": [[18, 52], [46, 52], [46, 56], [18, 56]], "z": 5 },
    "belt_hi": { "polygon": [[36, 52], [46, 52], [46, 56], [36, 56]], "z": 6 },
    "buckle": { "rect": [29, 52, 6, 4], "z": 7 },
    // LEGS (z: 0-1)
    "leg_l": { "polygon": [[20, 64], [30, 64], [28, 84], [18, 84]], "z": 0 },
    "leg_l_shd": { "polygon": [[20, 64], [24, 64], [22, 84], [18, 84]], "z": 1 },
    "leg_r": { "polygon": [[34, 64], [44, 64], [46, 84], [36, 84]], "z": 0 },
    "leg_r_hi": { "polygon": [[40, 64], [44, 64], [46, 84], [42, 84]], "z": 1 },
    // BOOTS (z: 2-3)
    "boot_l": { "polygon": [[16, 84], [30, 84], [30, 94], [16, 94]], "z": 2 },
    "boot_l_shd": { "polygon": [[16, 84], [22, 84], [22, 94], [16, 94]], "z": 3 },
    "boot_r": { "polygon": [[34, 84], [48, 84], [48, 94], [34, 94]], "z": 2 },
    "boot_r_hi": { "polygon": [[42, 84], [48, 84], [48, 94], [42, 94]], "z": 3 }
  }
}
```

---

## Sources

Techniques synthesized from:
- [Derek Yu's Pixel Art Tutorial](https://www.derekyu.com/makegames/pixelart.html) - Selective outlining, anti-aliasing
- [SLYNYRD Pixelblog](https://www.slynyrd.com/blog/2022/11/28/pixelblog-41-isometric-pixel-art) - Isometric foundations
- [SLYNYRD Character Sprites](https://www.slynyrd.com/blog/2019/10/21/pixelblog-22-top-down-character-sprites) - Proportions
- [Pixel Parmesan](https://pixelparmesan.com/blog/fundamentals-of-isometric-pixel-art) - 2:1 line patterns
- Pre-rendered sprite analysis from Fallout 2, Donkey Kong Country workflows
