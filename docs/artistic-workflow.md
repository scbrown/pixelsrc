# Artistic Workflow for Pixelsrc

A guide for creating quality pixel art with pixelsrc, including autonomous iteration patterns for AI agents.

## Core Insight

Pixelsrc's structured regions work like **pre-rendered 3D sprites** (Fallout 2, Donkey Kong Country), not hand-drawn pixel art. You define surfaces and their lighting relationships; the renderer produces the pixels.

```
Traditional: 3D Model → Render → 2D Sprite
Pixelsrc:    Regions (surfaces + shading) → Render → 2D Sprite
```

**Think in surfaces and light, not individual pixels.**

---

## Phase 0: Silhouette First

**Before adding detail, nail the outline.** A good silhouette is recognizable even when filled solid.

### The Mushroom Principle

Most organic forms can be built from 3 basic shapes:

```
     ●●●●●        ← Circle (head/cranium)
    ●●●●●●●
   ●●●●●●●●●
    ███████      ← Rectangle (torso/mid-section)
    ███████
     █████       ← Triangle (tapers - jaw/legs)
      ███
       █
```

**Start simple.** If you can't make the silhouette work with 3 shapes, the problem is proportions, not detail.

### Available Shape Primitives

| Shape | Syntax | Notes |
|-------|--------|-------|
| **Circle** | `circle: [cx, cy, r]` | Center + radius. Use for heads, round objects |
| **Ellipse** | `ellipse: [cx, cy, rx, ry]` | Center + x/y radii. Ovals, stretched circles |
| **Rectangle** | `rect: [x, y, w, h]` | Top-left corner + width/height |
| **Stroke** | `stroke: [x, y, w, h]` | Rectangle outline only (1px border) |
| **Polygon** | `polygon: [[x,y], ...]` | Arbitrary shape. Winding order doesn't matter |
| **Line** | `line: [[x1,y1], [x2,y2], ...]` | Connected line segments |
| **Points** | `points: [[x,y], ...]` | Individual pixels |
| **Union** | `union: [{ shape }, ...]` | Combine multiple shapes into one region |

### Shape Diagrams with Coordinates

```
CIRCLE: circle: [cx, cy, r]
Example: circle: [16, 12, 8]

       0   4   8  12  16  20  24  28  32
     0 ┌───────────────────────────────┐
     4 │         ●●●●●●●               │  ← radius 8
     8 │       ●●       ●●             │
    12 │      ●    (16,12)  ●          │  ← center at (16, 12)
    16 │       ●●       ●●             │
    20 │         ●●●●●●●               │
    24 └───────────────────────────────┘

RECTANGLE: rect: [x, y, w, h]
Example: rect: [8, 16, 16, 8]

       0   4   8  12  16  20  24  28  32
     0 ┌───────────────────────────────┐
    16 │        ████████████████       │  ← y=16 (top edge)
    20 │        ████████████████       │
    24 │        ████████████████       │  ← y=24 (top + height)
    28 └───────────────────────────────┘
                ↑               ↑
              x=8            x=24 (x + width)

POLYGON (Triangle): polygon: [[x1,y1], [x2,y2], [x3,y3]]
Example: polygon: [[16, 4], [4, 28], [28, 28]]

       0   4   8  12  16  20  24  28  32
     0 ┌───────────────────────────────┐
     4 │               ▲ (16,4)        │  ← vertex 1 (tip)
     8 │              ███              │
    12 │             █████             │
    16 │            ███████            │
    20 │           █████████           │
    24 │          ███████████          │
    28 │ (4,28)  █████████████ (28,28) │  ← vertices 2 & 3 (base)
    32 └───────────────────────────────┘

POLYGON (Trapezoid): polygon: [[6,28], [26,28], [24,38], [6,38]]
Example: Jaw shape - wider at top, narrower at bottom

       0   4   8  12  16  20  24  28  32
    24 ┌───────────────────────────────┐
    28 │      ████████████████████     │  ← top: (6,28) to (26,28)
    32 │      ██████████████████       │
    36 │       ████████████████        │
    38 │       ████████████████        │  ← bottom: (6,38) to (24,38)
    40 └───────────────────────────────┘
              ↑                 ↑
           x=6               x=24-26 (tapers)

COMBINED EXAMPLE: Skull silhouette (circle + rect + trapezoid)

       0   4   8  12  16  20  24  28  32
     0 ┌───────────────────────────────┐
     4 │         ●●●●●●●               │
     8 │       ●●●●●●●●●●●             │  ← circle: [14, 12, 12]
    12 │      ●●●●●●●●●●●●●            │     (cranium)
    16 │       ●●●●●●●●●●●             │
    20 │        ████████████████       │
    24 │        ████████████████       │  ← rect: [6, 18, 20, 10]
    28 │        ████████████████       │     (mid-face)
    32 │         ██████████████        │
    36 │          ████████████         │  ← polygon: trapezoid
    38 │           ██████████          │     (jaw taper)
    40 └───────────────────────────────┘

Tip: Shapes overlap where they connect - no gaps needed.
```

### Polygon Reference

**Winding order doesn't matter** - clockwise or counter-clockwise vertices produce identical results.

Common polygon patterns:
```
Triangle (3 vertices):
  "polygon": [[16, 4], [4, 28], [28, 28]]

  Tip at top, base at bottom. Vertices can be listed in any order.

Trapezoid (4 vertices):
  "polygon": [[6, 28], [26, 28], [24, 38], [6, 38]]

  Top edge wider than bottom = jaw shape.
  Match top edge to rect above for seamless connection.

Pentagon+ (5+ vertices):
  ⚠️ May cause fill artifacts - prefer union of simpler shapes
```

### Union: Combining Shapes

Use `union` to combine multiple shapes into one region:

```json
"fill": {
  "union": [
    { "circle": [16, 12, 12] },
    { "rect": [6, 18, 20, 10] },
    { "polygon": [[6, 28], [26, 28], [24, 38], [6, 38]] }
  ],
  "z": 0
}
```

**When to use union:**
- Combining circle + rect + triangle for silhouettes
- Complex organic shapes that would need 6+ polygon vertices
- Reusing shape logic across regions

**Gotcha:** Union still counts toward the 4-shape limit (see Known Limitations)

### Silhouette Workflow

1. **Define the outline** with 2-3 basic shapes (circle + rect + triangle)
2. **Adjust proportions** until the shape reads correctly
3. **Add asymmetry** for 3/4 view (shift shapes, change angles)
4. **Only then** add internal detail, shading, features

### 3/4 View Orientation

To orient a face in 3/4 view:
- Shift the circle (head) slightly to one side
- Angle the triangle (chin/jaw) to point in the facing direction
- The direction the triangle points = the direction the face looks

```
Front view:       3/4 right:
    ●●●              ●●●
   █████            █████
     ▼                 ▶    ← Triangle points right = facing right
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

**Light Direction Convention:** Top-right at ~45°

---

## Region Types

| Type | Syntax | Best For |
|------|--------|----------|
| **Polygon** | `[[x,y], ...]` | Tapered forms, organic edges, clothing, limbs |
| **Ellipse** | `[cx, cy, rx, ry]` | Heads, rounded objects, organic details |
| **Rect** | `[x, y, w, h]` | Mechanical parts, belts, boots, screens |
| **Points** | `[[x,y], ...]` | Eyes, highlights, single-pixel accents |

**Critical:** Avoid rectangles for organic forms. Use ellipses for heads, polygons for bodies.

---

## Character Proportions

For small sprites (16-24px tall), use exaggerated proportions:

| Body Part | Proportion | Example (24px) |
|-----------|------------|----------------|
| Head | 1/3 of height | 8 pixels |
| Torso | 1/3 of height | 8 pixels |
| Legs + Feet | 1/3 of height | 8 pixels |

**Head must be large** - at small scale, proportionally-correct heads become invisible.

---

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| All rectangles | Blocky, robotic look | Use ellipses and polygons |
| Head too small | Invisible at small scale | Make head 1/3 of sprite height |
| No leg separation | Looks like single mass | Leave 1-2 pixel gap |
| Too many details | Noise at small scale | 2-3 emphasized features max |
| Shading outside bounds | Harsh rectangular edges | Keep shading within organic shapes |
| Floating head | Disconnected appearance | Add neck region |

---

## Artisan Workflow: Autonomous Iteration

When creating pixel art autonomously (without human feedback at each step), use this component-based iteration pattern.

### Philosophy

1. **Component isolation** - work on one part at a time
2. **Self-critique against criteria** - measurable evaluation, not "looks better"
3. **Variant generation** - try multiple approaches, pick the best
4. **Integration testing** - components that work alone may clash together

### Molecule Structure

```
ARTISAN MOLECULE: [artwork-name]

Phase 1: FOUNDATION
├── Define components: [list parts to iterate]
├── Define criteria per component
├── Gather references (existing examples, style targets)
└── Create initial draft v0

Phase 2: COMPONENT ITERATION
│
│  For EACH component (sequentially):
│  │
│  │  Component Loop (max 5-10 rounds):
│  │  ├── Generate 2-3 variants with different approaches
│  │  ├── Render all variants (use naming: component_v1a, component_v1b, etc.)
│  │  ├── Evaluate against component quality gates
│  │  ├── Promote highest scorer as new baseline
│  │  ├── Log: what changed, why it scored better
│  │  └── All gates passed? → next component : continue
│  │
│  └── Output: [component]_final + [component]_evolution.png

Phase 3: INTEGRATION ITERATION
│
│  Integration Loop (max 5 rounds):
│  ├── Compose all components
│  ├── Evaluate against integration quality gates
│  │
│  │  If weak component identified:
│  │  └── DRILL DOWN → return to Phase 2 for that component
│  │
│  │  If composition issue (layering, z-order, color clash):
│  │  └── Adjust composition parameters, re-render
│  │
│  └── All integration gates passed? → Phase 4 : continue

Phase 4: SUBMIT
├── Final composed render
├── All component evolution logs
├── Integration evolution + drill-down log
└── Comparison sheet (v0 → final side-by-side)
```

---

## Quality Gates

Quality gates are **binary pass/fail criteria** that agents can evaluate objectively.

### Component-Level Gates

| Gate | Test | Pass Criteria |
|------|------|---------------|
| **Silhouette** | Fill component with solid black | Shape is recognizable |
| **Scale** | Render at 1x zoom | Details still readable |
| **Palette** | Check all colors used | 100% from defined palette |
| **Pixels** | Zoom to 800% and inspect | No orphan pixels, clean edges |
| **Lighting** | Check shading direction | Consistent with top-right 45° convention |

### Integration Gates

| Gate | Test | Pass Criteria |
|------|------|---------------|
| **Layer Order** | Render full composition | No unexpected occlusion |
| **Proportion** | Compare component sizes | Match defined ratios (head 1/3, etc.) |
| **Color Harmony** | Convert to grayscale | Values distinct, no muddy areas |
| **Animation** | Check detail density | No detail that would blur in motion |

### How to Test Each Gate

**Silhouette Test:**
```bash
# Render with solid fill (conceptually - fill all regions with same color)
# Ask: "Can I tell what this is?"
```

**Scale Test:**
```bash
pxl render file.pxl --scale 1 -o test_1x.png
# View at actual size - is it readable?
```

**Palette Test:**
```bash
pxl validate file.pxl
# Check for "Unknown token" warnings
```

**Pixel Test:**
```bash
pxl render file.pxl --scale 8 -o test_8x.png
# Inspect for orphan pixels, jagged edges
```

---

## Variant Naming Convention

When generating variants, use consistent naming:

```
[component]_v[iteration][variant]

Examples:
  eyes_v1a.png    # Eyes, iteration 1, variant A
  eyes_v1b.png    # Eyes, iteration 1, variant B
  eyes_v2a.png    # Eyes, iteration 2, variant A (based on v1 winner)

  head_integrated_v1.png    # Full head, integration iteration 1
  head_integrated_v2.png    # Full head, integration iteration 2
```

This naming allows:
- Easy comparison between variants
- Clear evolution tracking
- Automated comparison sheet generation

---

## Variant Generation Strategies

When creating variants, try different approaches:

| Strategy | Description | When to Use |
|----------|-------------|-------------|
| **Contrast** | Vary light/dark ratio | Readability issues |
| **Proportion** | Vary size/shape ratios | Shape clarity issues |
| **Detail level** | More vs. fewer details | Scale readability issues |
| **Color temperature** | Warmer vs. cooler | Color harmony issues |
| **Line weight** | Thicker vs. thinner outlines | Definition issues |

---

## RPG Character Quality Targets

For RPG-style character heads (the reference-head use case):

| Target | Criteria |
|--------|----------|
| **Party Portrait Ready** | Readable at 32x40, distinct silhouette, hair/face distinguishable |
| **Battle Ready** | Status overlays (poison/damage) visible without obscuring identity |
| **Dialogue Ready** | Expression readable, mouth positions distinct, blink animation smooth |

### Component Breakdown for Heads

```
head/
├── base (face shape, skin shading)
├── eyes (white, iris, pupil, highlights)
├── mouth (lips, teeth if visible)
├── brows (expression control)
├── hair (style, highlights, shadows)
├── ears (if visible)
└── extras (glasses, scars, status effects)
```

---

## Self-Critique Prompts

During evaluation, ask:

- "At 1x zoom, can I tell what this is?"
- "Does the expression read clearly?"
- "Are there any pixels that seem out of place?"
- "Does this match the style of the reference?"
- "Would this animate well, or is there too much detail?"
- "Is the silhouette distinct from other characters?"

---

## Iteration Logging

For each iteration, record:

```
Round N:
- Approach tried: [what you changed]
- Gate results: silhouette=PASS, scale=PASS, pixels=FAIL (orphan at 12,8)
- Winner: variant [A/B/C]
- Reason: [why this scored highest]
- Next iteration focus: [what to fix/try next]
```

This log helps:
1. Avoid repeating failed approaches
2. Identify patterns in what works
3. Provide human reviewer with context
4. Enable drill-down decisions during integration

---

## Example Bead Structure

For tracking artistic work with beads:

```
artisan-[artwork] (epic)
├── foundation (task) - references, criteria, v0
├── iterate-[component-1] (task) - component iteration loop
├── iterate-[component-2] (task) - component iteration loop
├── ...
├── integrate (task) - integration loop, can reopen component tasks
└── submit (task) - final render, logs, comparison
```

---

## Known Rendering Limitations

Some combinations of shapes can cause rendering artifacts. Work around these:

| Issue | Trigger | Workaround |
|-------|---------|------------|
| **Stripe artifacts** | 5+ shape primitives with same color | Keep to 4 shapes max, or use different z-levels |
| **Polygon fill gaps** | Complex polygons (6+ vertices) | Break into simpler shapes, use union |
| **Overlap artifacts** | Multiple regions overlapping at same z | Use different z-levels for overlapping regions |

**Safe patterns:**
- 3-4 simple shapes (circle, rect, triangle) = reliable
- Polygons with 3-5 vertices = reliable
- `union` of simple shapes = usually works

**Risky patterns:**
- 5+ overlapping regions with same color
- Single polygon with 6+ vertices
- Complex unions with many shapes

When in doubt, **keep it simple**. If you hit artifacts, reduce complexity.

See bead TTP-vi3r for detailed bug analysis.

---

## Sources

Techniques synthesized from:
- [Derek Yu's Pixel Art Tutorial](https://www.derekyu.com/makegames/pixelart.html)
- [SLYNYRD Pixelblog](https://www.slynyrd.com/blog)
- [Game Art Pipeline from Idea to Polish](https://pixune.com/blog/game-art-pipeline/)
- Pre-rendered sprite analysis from Fallout 2, Donkey Kong Country
