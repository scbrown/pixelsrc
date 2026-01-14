# Pixel Art DSL - Planning Document

**Project Codename:** TBD
**Status:** Planning Phase
**Last Updated:** 2025-01-13

---

## 1. Problem Statement

Current text-based image formats (XPM, PPM, PBM) were designed for general image storage, not pixel art creation. They lack:
- Semantic understanding of pixel art concepts (sprites, frames, palettes)
- GenAI-friendly structure (LLMs struggle with coordinate systems)
- Modern game dev workflow integration
- Animation as a first-class citizen

**Target Users:**
- GenAI systems (Claude, GPT, etc.) generating game assets
- Indie game developers wanting quick prototyping
- Pixel artists wanting text-based version control
- Roguelike/retro game developers

---

## 2. Competitive Analysis

### Existing Solutions

| Tool/Format | Pros | Cons |
|-------------|------|------|
| **XPM** | Text-based, C-includable | No animation, dated syntax, no layers |
| **PPM/PBM** | Simple, universal | Raw numbers, no palette names, no structure |
| **Aseprite** | Industry standard editor | Binary format, not text-based, not GenAI friendly |
| **PICO-8** | Constrained, charming | Proprietary, specific palette, not extensible |
| **Emoji art** | Colorful, GenAI can do it | Platform-dependent rendering, limited resolution |

### Differentiation Opportunities

1. **GenAI-Native** - Designed for LLMs to generate reliably
2. **Semantic Structure** - Named regions, animation states, hitboxes
3. **Palette-First** - Color themes as reusable, named entities
4. **Multi-Resolution** - Same definition can render at different scales
5. **Game Engine Integration** - Direct export to Unity/Godot/etc formats

---

## 3. Feature Set (Prioritized)

### P0 - Core (MVP)
- [ ] Simple grid-based sprite definition
- [ ] Named color palettes
- [ ] Single sprite → PNG export
- [ ] Human-readable, GenAI-generatable syntax

### P1 - Animation
- [ ] Multiple frames in single file
- [ ] Animation timing/metadata
- [ ] Spritesheet export
- [ ] GIF export

### P2 - Advanced Sprites
- [ ] Layers with blend modes
- [ ] Named regions (hitbox, origin, etc.)
- [ ] Sprite variants (recolor, flip)
- [ ] Tile/pattern definitions

### P3 - Game Integration
- [ ] Unity spritesheet + metadata export
- [ ] Godot resource export
- [ ] Tiled tileset export
- [ ] CSS sprite export

### P4 - Ecosystem
- [ ] VS Code extension (syntax highlighting, preview)
- [ ] Web-based editor/previewer
- [ ] CLI tool
- [ ] Import from PNG (reverse engineer)

---

## 4. DSL Design Considerations

### Design Goals
1. **Readable** - A human should understand it at a glance
2. **Generatable** - LLMs should produce valid output reliably
3. **Diffable** - Git diffs should be meaningful
4. **Extensible** - New features shouldn't break old files
5. **Minimal** - Simple things should be simple

### Syntax Style Options

**Option A: YAML-like**
```yaml
sprite: hero_idle
size: 16x16
palette:
  skin: #FFCC99
  hair: #8B4513
  outline: #000000
frames:
  - |
    ....HHHH....
    ...HHHHHH...
    ...SSSSSS...
    ...S.SS.S...
```

**Option B: Custom minimal**
```
@sprite hero_idle 16x16
@palette
  . = transparent
  H = #8B4513  ; hair
  S = #FFCC99  ; skin
  O = #000000  ; outline

@frame idle_1
....HHHH....
...HHHHHH...
...SSSSSS...
...S.SS.S...
```

**Option C: JSON-based**
```json
{
  "sprite": "hero_idle",
  "size": [16, 16],
  "palette": {
    ".": "transparent",
    "H": "#8B4513"
  },
  "frames": {
    "idle_1": [
      "....HHHH....",
      "...HHHHHH..."
    ]
  }
}
```

**Option D: S-expression (Lisp-like)**
```lisp
(sprite hero_idle
  (size 16 16)
  (palette
    (. transparent)
    (H #8B4513))
  (frame idle_1
    "....HHHH...."
    "...HHHHHH..."))
```

### Syntax Decision Factors
- YAML: Familiar, but whitespace-sensitive (LLM risk)
- Custom: Most control, but needs parser from scratch
- JSON: Universal, but verbose and no comments
- S-expr: Elegant, unfamiliar to most users

**Current Leaning:** TBD - need to evaluate GenAI reliability

---

## 5. Implementation Language Options

| Language | Pros | Cons |
|----------|------|------|
| **Python** | Fast prototyping, PIL/Pillow, wide adoption | Slower, packaging complexity |
| **Rust** | Fast, single binary, WASM target | Slower dev, steeper learning |
| **TypeScript** | Web-native, good tooling, WASM via AssemblyScript | Node dependency, not great for CLI |
| **Go** | Single binary, fast, good CLI tooling | Less expressive, smaller ecosystem |
| **Zig** | Blazing fast, simple, C interop | Young ecosystem, niche |

### Considerations
- **Prototyping phase:** Python (fastest iteration)
- **Production CLI:** Rust or Go (single binary distribution)
- **Web playground:** TypeScript/WASM
- **Game engine plugins:** Depends on target engine

**Recommendation:** Start with Python for MVP, port to Rust for v1.0

---

## 6. Output Formats

### Image Outputs
- PNG (single sprite, spritesheet)
- GIF (animated)
- WebP (modern alternative)
- SVG (vector upscale)

### Data Outputs
- JSON (sprite metadata, animation data)
- Unity .meta files
- Godot .tres resources
- Tiled .tsx tilesets

### Code Outputs
- C header (XPM-style embedding)
- Rust const arrays
- Python byte arrays

---

## 7. Open Questions

1. **Naming:** What do we call this? (PixelScript? SpriteML? DotLang? 8BitDSL?)

2. **Character set:** Single char per pixel, or allow multi-char tokens?
   - Single: `H` = hair
   - Multi: `{hair}` or `[H]` for more colors than ASCII allows

3. **Coordinate system:** Top-left origin (standard) or bottom-left (OpenGL)?

4. **Alpha/transparency:** How to represent? (special char? RGBA in palette?)

5. **Subpixel/antialiasing:** Support or explicitly ban for purity?

6. **Animation format:** Inline frames or separate files with references?

7. **Palette sharing:** Global palettes across files? Import system?

---

## 8. Next Steps

1. [ ] Decide on syntax style (A/B/C/D or hybrid)
2. [ ] Define MVP feature scope precisely
3. [ ] Create formal grammar/spec for DSL
4. [ ] Build Python prototype renderer
5. [ ] Test with Claude/GPT generation
6. [ ] Iterate based on GenAI reliability
7. [ ] Define file extension (.pxl? .sprite? .dot?)

---

## 9. References & Inspiration

- XPM Format: https://en.wikipedia.org/wiki/X_PixMap
- Netpbm/PPM: https://en.wikipedia.org/wiki/Netpbm
- Aseprite file format: https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md
- PICO-8 sprite format
- Rivals of Aether sprite style guide
- Lospec palettes: https://lospec.com/palette-list

---

## Appendix A: Example Target Output

What we want Claude to be able to generate reliably:

```
[EXAMPLE DSL HERE - TBD after syntax decision]
```

Rendered output: 32x32 fighter character, idle animation, 4 frames, Rivals-of-Aether style with:
- Clean outlines
- 2-3 color shading per material
- Readable silhouette
- ~16-32 color palette



