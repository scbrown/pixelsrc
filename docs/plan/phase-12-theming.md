# Phase 12: Theming & Branding Assets

**Goal:** Create cohesive visual identity with favicon, banners, and social preview assets

**Status:** Planning

**Depends on:** Phase 11 (Website improvements complete)

---

## Branding Guidelines

### Naming Convention
- **Always lowercase**: `pixelsrc` (not "Pixelsrc" or "PixelSrc")
- **CLI tool**: `pxl`
- **Tagline**: "GenAI-native pixel art format"

### Logo
- Primary logo: lowercase `p` in synthwave colors
- The `p` represents both "pixel" and the descender mimics a pixel art brush stroke

### Where to Update
- [ ] Website header: `Pixel<span>src</span>` → `pixel<span>src</span>`
- [ ] README.md title and references
- [ ] package.json description
- [ ] GitHub repo description
- [ ] Any documentation headers

---

## Overview

Establish a consistent visual identity for Pixelsrc across:
- Favicon (browser tab icon)
- Social preview image (Open Graph / Twitter cards)
- GitHub README visual improvements
- Banner assets for documentation/marketing

**Design Direction:** Dracula theme colors with synthwave accents

---

## Tasks

### Task 12.1: Favicon

Create and integrate favicon using pixelsrc format (dogfooding).

**Assets Created:**
- `examples/favicon_p.jsonl` - 8x8 lowercase `p` with synthwave glow

**Deliverables:**
1. Generate favicon at multiple sizes (16, 32, 48, 180, 192, 512)
2. Create `favicon.ico` for legacy browser support
3. Add favicon links to `website/index.html`
4. Add PWA manifest icons

---

### Task 12.2: Social Preview Image

Create Open Graph image for link previews on social media.

**Deliverables:**
1. Design 1280x640 preview image showing:
   - Pixelsrc logo/branding
   - Sample pixel art (rendered from format)
   - Tagline: "GenAI-native pixel art format"
2. Add OG meta tags to `website/index.html`
3. Upload to GitHub repo for social preview

---

### Task 12.3: Banner Assets

Create banner variations for different contexts.

**Assets Created:**
- `examples/banner_pxl.jsonl` - "pxl" text with synthwave gradient

**Deliverables:**
1. Wide banner for README header
2. Square banner for social avatars
3. Animated GIF banner showing format in action

---

### Task 12.4: GitHub README Polish

Improve README visual presentation.

**Deliverables:**
1. Add banner image at top
2. Add badges (version, license, build status)
3. Add relevant GitHub topics/tags
4. Include GIF demo of workflow
5. Add "Made with Pixelsrc" badge for community use

---

### Task 12.5: Synthwave Palette

Formalize the synthwave color palette as a built-in palette option.

**Colors:**
```
{purple}: #BD93F9  (Dracula Purple)
{pink}:   #FF79C6  (Dracula Pink)
{cyan}:   #8BE9FD  (Dracula Cyan)
{glow}:   #E2B3FF  (Light purple glow)
{hot}:    #FF2D95  (Hot pink)
{neon}:   #00F7FF  (Neon cyan)
```

**Deliverables:**
1. Add "synthwave" to built-in palettes in `src/palettes/mod.rs`
2. Document in palette list

---

## Assets Reference

| Asset | Location | Status |
|-------|----------|--------|
| lowercase p favicon (8x8) | `examples/favicon_p.jsonl` | Created |
| pxl banner sprite | `examples/banner_pxl.jsonl` | Created |
| Synthwave palette | (inline in sprites) | Defined |

---

## Future Ideas

- Animated favicon (GIF or SVG animation)
- Dark/light theme variants
- Holiday/seasonal variants
- Community-contributed themes
