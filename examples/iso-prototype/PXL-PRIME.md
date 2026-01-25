# PXL Prime: Artistic Workflow for Pixel Art Generation

## The Core Insight

Pixelsrc's structured regions work like **pre-rendered 3D sprites** (Fallout 2, Donkey Kong Country), not hand-drawn pixel art. You define surfaces and their lighting relationships; the renderer produces the pixels.

```
Traditional: 3D Model → Render → 2D Sprite
Pixelsrc:    Regions (surfaces + shading) → Render → 2D Sprite
```

This means: **Think in surfaces and light, not individual pixels.**

---

## The Two Approaches Combined

### From Hand-Drawn Pixel Art:
- **Selective outlining** - Don't outline everything in black
- **Exaggerated proportions** - Head = 1/3 to 1/2 of sprite height
- **2-3 emphasized details** - Not everything needs definition
- **Clear silhouette** - Must read as solid black shape
- **2:1 line pattern** - Foundation of isometric angles

### From Pre-Rendered 3D Sprites:
- **Define lit/base/shadow surfaces** - Three tones create form
- **Consistent light direction** - Top-right is standard
- **Grayscale readability test** - If it reads in grayscale, colors will work
- **Lighting creates form** - Not outlines

---

## The Three-Surface Rule

Every 3D form has three visible surfaces under directional light:

```
        [LIT]        ← Facing light (brightest)
       /    \
   [BASE]  [SHADOW]  ← Side (medium) and away from light (darkest)
```

In pixelsrc, define these as separate regions with related colors:

```json
"palette": {
  "suit_lit": "#4080c0",    // Brightest
  "suit": "#2868a8",        // Base
  "suit_shadow": "#184870"  // Darkest
}
```

The COLOR DIFFERENCE defines the edge, not a black outline.

---

## Workflow: Isometric Environment

### Phase 1: Base Diamond
64×32 with 2:1 isometric ratio.

```json
"ground": { "polygon": [[32, 0], [63, 15], [32, 31], [0, 15]] }
```

### Phase 2: Three-Surface Shading
Light from top-right. Use irregular polygons, not straight splits.

```json
"ground_lit": { "polygon": [[32, 2], [55, 12], [45, 18], [32, 14]] },
"ground_shadow": { "polygon": [[12, 14], [32, 20], [20, 26], [5, 16]] }
```

### Phase 3: Surface Details
Cracks, texture, objects - using organic shapes.

```json
"crack": { "polygon": [[18, 8], [26, 12], [25, 13], [17, 9]] },
"pebble": { "ellipse": [40, 15, 2, 1] }
```

---

## Workflow: Character Sprite

### Phase 1: Silhouette Block-in
Establish the readable shape. Test: fill with solid color - can you tell what it is?

**Proportions for readability:**
- Head: 1/3 of total height
- Body: Simple tapered form
- Limbs: Suggested, not fully defined

### Phase 2: Three-Surface Form
Apply lit/base/shadow to major masses.

```json
// Torso has three surfaces
"torso": { "polygon": [...] },           // Base color
"torso_lit": { "polygon": [...] },       // Right side (toward light)
"torso_shadow": { "polygon": [...] }     // Left side (away from light)
```

### Phase 3: Minimal Details
Only add what's essential for recognition:
- Eyes: 1-2 pixels each
- Belt/accessories: Break up large areas
- Equipment: Small accent shapes

---

## Common Mistakes

### ❌ Black Outlines Everywhere
Creates "outsider art" look. Let color contrast define edges.

### ❌ Too Many Details
At small scale, details become noise. 2-3 emphasized features max.

### ❌ Flat Shading
Without lit/base/shadow, forms look flat. Always use 3+ values.

### ❌ Ignoring Silhouette
If the solid shape isn't readable, no amount of detail helps.

### ❌ One-Shot Complexity
Build incrementally. Render and review after each phase.

---

## Testing Your Sprite

1. **Silhouette test**: Fill with solid black. Is it recognizable?
2. **Grayscale test**: Convert to grayscale. Does form read clearly?
3. **Scale test**: View at 1x. Does it read at actual size?
4. **Context test**: Place on intended background. Does it pop?

---

## Region Types and When to Use Them

| Type | Syntax | Best For |
|------|--------|----------|
| **Polygon** | `[[x,y], ...]` | Irregular surfaces, tapered forms, organic edges |
| **Ellipse** | `[cx, cy, rx, ry]` | Heads, rounded objects, organic details |
| **Rect** | `[x, y, w, h]` | Mechanical parts, belts, screens |
| **Points** | `[[x,y], ...]` | Eyes, highlights, texture accents |

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

## Sources

Techniques synthesized from:
- [Derek Yu's Pixel Art Tutorial](https://www.derekyu.com/makegames/pixelart.html) - Selective outlining, anti-aliasing
- [SLYNYRD Pixelblog](https://www.slynyrd.com/blog/2022/11/28/pixelblog-41-isometric-pixel-art) - Isometric foundations
- [SLYNYRD Character Sprites](https://www.slynyrd.com/blog/2019/10/21/pixelblog-22-top-down-character-sprites) - Proportions
- [Pixel Parmesan](https://pixelparmesan.com/blog/fundamentals-of-isometric-pixel-art) - 2:1 line patterns
- Pre-rendered sprite analysis from Fallout 2, Donkey Kong Country workflows
