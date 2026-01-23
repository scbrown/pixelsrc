# Competitive Analysis & Differentiation

> Research conducted January 2025

## Executive Summary

**Pixelsrc fills a genuine gap**: there is no existing text-based *source* format designed for AI-generatable, human-readable pixel art definition. Existing tools are either visual editors with text *exports*, or AI image generators that skip text entirely.

**End goal**: REXPaint-quality output. REXPaint sets the bar for roguelike/ASCII art tooling - our compiled output should match that level of polish.

---

## The Current Landscape

### What Developers Actually Use

#### REXPaint (The Gold Standard)
- **Website**: https://www.gridsagegames.com/rexpaint/
- **What it is**: Visual ASCII art editor, de facto standard for roguelike development
- **Native format**: `.xp` (binary, zlib compressed)
- **Exports**: PNG, ANSI, TXT, CSV, XML, XPM, BBCode, C:DDA
- **Ecosystem**: Parsers in 17+ languages (Rust, Python, C++, C#, Nim, Godot, etc.)
- **Key insight**: Text formats are *exports*, not sources. Devs draw visually → export → use.

#### Fantasy Consoles (PICO-8, TIC-80)
- **PICO-8**: `.p8` files have `__gfx__` section with hex digits (0-F), 16-color limit
- **TIC-80**: `.tic` binary chunks, hex-based sprite data
- **Limitation**: Hex characters, not semantic names. `A` means color 10, not `{armor}`.

#### Legacy Text Formats
| Format | Year | Pros | Cons |
|--------|------|------|------|
| XPM | 1989 | Valid C code, text-based | No animation, dated, no semantic naming |
| PPM/PBM | 1988 | Simple, universal | Raw numbers, no palette names |

### AI/LLM Pixel Art Tools

These exist but take a **fundamentally different approach**:

| Tool | Approach | Why It's Not Pixelsrc |
|------|----------|------------------|
| [PixelLab](https://www.pixellab.ai/) | Diffusion model → image | Generates images directly, no text format |
| [pixel-plugin](https://github.com/willibrandon/pixel-plugin) | Natural language → Aseprite API | Procedural API calls, no file format |
| [Ludo.ai](https://ludo.ai/features/sprite-generator) | Text prompt → sprite sheet | AI image gen, not deterministic text |

### Procedural Generation Tools

Visual randomizers, not text source formats:

- **[CryPixels](https://crypixels.com/)** - Grid-based procedural generator with brushes
- **[Lospec Generator](https://lospec.com/procedural-pixel-art-generator/)** - Random sprite generator
- **[SPARTAN](https://pnjeffries.itch.io/spartan-procjam-edition)** - Node-based parametric pixel art

---

## The Workflow Gap

```
EXISTING WORKFLOW (Visual-First):
  Draw in REXPaint/Aseprite → Export to text/image → Use in game

Pixelsrc WORKFLOW (Text-First):
  Write/Generate .pxs → Compile to image → Use in game
```

| Capability | REXPaint | PICO-8 | XPM | AI Gen | Pixelsrc |
|------------|----------|--------|-----|--------|-----|
| Human-readable source | Export only | Hex-based | Yes | N/A | **Yes** |
| Semantic color names | No | No | No | N/A | **Yes** |
| AI-generatable | No | Difficult | Difficult | Different | **Yes** |
| Animation support | Yes | Yes | No | Varies | **Yes** |
| Git-diffable | Exports only | Yes | Yes | No | **Yes** |
| Streaming-friendly | No | No | No | No | **Yes** |

---

## Why This Gap Exists

1. **Small community** - Roguelike/retro devs are niche; not enough demand for new tooling
2. **Visual tools work** - REXPaint is excellent for manual creation
3. **No GenAI driver (until now)** - Before LLMs, no compelling reason for text SOURCE format
4. **Inertia** - "We've always drawn sprites, not written them"

---

## Pixelsrc's Unique Value Proposition

### 1. Source Format, Not Export Format
- Edit the text, recompile → changes reflected
- Version control diffs are meaningful
- No round-trip through visual editor

### 2. GenAI-Native Design
- Semantic tokens (`{skin}`, `{hair}`) over cryptic chars
- Grid position = implicit coordinates (sidesteps LLM spatial weakness)
- JSONL streaming for real-time generation

### 3. Human-Readable
- A developer can read a `.pxs` file and understand the sprite without rendering
- Comments and named palettes aid comprehension

### 4. Thin Layer Philosophy
- Pixelsrc → ImageMagick → PNG (don't reinvent rendering)
- Leverage existing tools, focus on the semantic gap

---

## Target Quality: REXPaint-Level Output

REXPaint produces stunning ASCII art and roguelike graphics. Examples from their gallery demonstrate:
- Complex multi-layer compositions
- Rich color palettes (full RGB, not just 16 colors)
- UI mockups, game maps, character art
- Professional-quality results

**Our goal**: A `.pxs` file, when compiled, should produce output indistinguishable from what a skilled REXPaint artist would create.

---

## Market Opportunity

### Primary Users
1. **GenAI systems** generating game assets (Claude, GPT, etc.)
2. **Indie game developers** wanting text-based version control
3. **Roguelike/retro developers** already comfortable with text
4. **Pixel artists** wanting git-diffable sprite sources

### Adjacent Markets
- Fantasy console developers (PICO-8, TIC-80 users who want better tooling)
- ASCII art community
- Demoscene artists

---

## Competitive Risks

1. **Adoption friction** - Devs comfortable with REXPaint may not switch
2. **AI image gen improves** - If diffusion models get good enough, text formats become less compelling
3. **Fantasy consoles add features** - PICO-8/TIC-80 could add semantic naming

### Mitigation
- Focus on GenAI use case (our unique strength)
- Provide import/export for REXPaint `.xp` files (interoperability)
- Keep format simple and stable

---

## Key Sources

- [REXPaint](https://www.gridsagegames.com/rexpaint/) - ASCII art editor
- [REXPaint Resources](https://www.gridsagegames.com/rexpaint/resources.html) - Parser libraries
- [PICO-8 File Format](https://pico-8.fandom.com/wiki/P8FileFormat) - Fantasy console format
- [TIC-80 File Format](https://github.com/nesbox/TIC-80/wiki/.tic-File-Format) - Fantasy console format
- [XPM Wikipedia](https://en.wikipedia.org/wiki/X_PixMap) - Legacy text format
- [LLM Pixel Art Experiment](https://ljvmiranda921.github.io/notebook/2025/07/20/draw-me-a-swordsman/) - Tool-calling approach
- [pixel-plugin](https://github.com/willibrandon/pixel-plugin) - Claude Code Aseprite integration
- [PixelLab](https://www.pixellab.ai/) - AI sprite generator
- [CryPixels](https://crypixels.com/) - Procedural pixel art
- [Lospec Generator](https://lospec.com/procedural-pixel-art-generator/) - Random sprites

---

## Why Not Just Use SVG + CSS?

> Added January 2026, addressing Format v2 design questions

### What overlaps with web tech

- Shape primitives (`rect`, `ellipse`, `polygon`, `path`) → basically SVG
- CSS variables, `color-mix()` → already CSS
- JSON-based format → nothing new
- Animations with keyframes → CSS animations

### What's genuinely different

#### 1. Semantic tokens, not just shapes

SVG:
```xml
<rect fill="#FFD5B4"/>
```
It's just a color.

Pixelsrc:
```json5
skin: { rect: [...] }
```
With:
```json5
roles: { skin: "fill" }
relationships: { "skin-shadow": { "derives-from": "skin" } }
```

The token carries meaning through the entire pipeline—theming, transforms, game logic.

#### 2. Pixel-art output, not vectors

SVG renders anti-aliased vectors. Getting crisp pixels requires fighting the format (`image-rendering: pixelated`, careful coordinate alignment). Pixelsrc outputs actual pixel grids—every shape rasterizes to discrete coordinates. No anti-aliasing, no subpixel ambiguity.

#### 3. AI context efficiency

A 32×32 sprite in SVG might be 2KB of verbose XML. The structured format describes the same sprite in ~400 chars of semantic intent. LLMs can reason about "skin region" vs "eye region" rather than parsing coordinate soup.

| Format | 32×32 sprite |
|--------|--------------|
| SVG | ~2,000 chars |
| Pixelsrc v1 (grid) | ~8,000 chars |
| Pixelsrc v2 (structured) | ~400 chars |

#### 4. Game engine integration

Designed for Unity/Godot/etc workflows, not browsers:

- Export spritesheets with animation metadata
- Hit regions and collision shapes from semantic tokens
- Role classifications for runtime effects
- State rules for damage/poison/freeze without separate sprites

#### 5. Constraint validation

```json5
pupil: {
  points: [[4, 6]],
  inside: "eye"  // validated at compile time
}
```

The compiler verifies that `pupil` pixels fall within `eye` bounds. SVG has no equivalent—you'd need external tooling.

#### 6. Relationship-aware transforms

Roles inform transform algorithms:

| Role | Transform Behavior |
|------|-------------------|
| `boundary` | High priority, preserve connectivity |
| `anchor` | Must survive (min 1px) |
| `fill` | Can shrink, low priority |

Future semantic rotation will use this to keep eyes visible and outlines connected when rotating sprites—something SVG rotation destroys.

### Honest assessment

**If you're making web content, use SVG + CSS.** It's mature, universal, and has great tooling.

Pixelsrc's value is narrower:

- **Pixel art specifically** (not vectors)
- **AI generation workflows** (context-efficient, semantic)
- **Game asset pipelines** (spritesheets, metadata, state rules)
- **Semantic meaning** that survives through transforms and theming

If those aren't your use cases, web tech is probably better.

### The niche

Pixelsrc exists because **pixel art + AI + games** is a specific combination where the semantic approach pays off:

1. AI needs compact, meaningful formats to generate assets
2. Games need metadata (hit regions, animation data, state effects)
3. Pixel art needs discrete coordinates, not vector approximations
4. Artists need semantic theming (change skin color, all shadows update)

Web tech solves none of these well. Pixelsrc solves all of them in one format.
