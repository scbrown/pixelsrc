# Phase 13: Theming & Branding Assets

**Goal:** Create cohesive visual identity with favicon, banners, and social preview assets

**Status:** Planning

**Depends on:** Phase 11 (Website improvements complete)

---

## Scope

Phase 13 adds:
- Favicon at multiple sizes for browser/PWA
- Social preview (Open Graph) image
- Banner assets for README and marketing
- Synthwave palette as built-in option
- Branding consistency across all touchpoints

**Not in scope:** Animated favicon, seasonal variants, community themes

---

## Branding Guidelines

### Naming Convention
- **Always lowercase**: `pixelsrc` (not "Pixelsrc" or "PixelSrc")
- **CLI tool**: `pxl`
- **Tagline**: "GenAI-native pixel art format"

### Logo
- Primary logo: lowercase `p` in synthwave colors
- The `p` represents both "pixel" and the descender mimics a pixel art brush stroke

### Synthwave Palette
```
{purple}: #BD93F9  (Dracula Purple)
{pink}:   #FF79C6  (Dracula Pink)
{cyan}:   #8BE9FD  (Dracula Cyan)
{glow}:   #E2B3FF  (Light purple glow)
{hot}:    #FF2D95  (Hot pink)
{neon}:   #00F7FF  (Neon cyan)
```

---

## Task Dependency Diagram

```
                              PHASE 13 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                    Phase 11 Complete                            │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Parallel - Foundation Assets)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
    │  │   13.1       │  │   13.2       │  │   13.3       │          │
    │  │  Synthwave   │  │  Favicon     │  │  Banner      │          │
    │  │  Palette     │  │  Sprite      │  │  Sprites     │          │
    │  └──────────────┘  └──────────────┘  └──────────────┘          │
    └─────────────────────────────────────────────────────────────────┘
              │                 │                 │
              └────────────────┬┴─────────────────┘
                               │
                               ▼
    WAVE 2 (Parallel - Asset Generation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────┐  ┌────────────────────────────┐  │
    │  │   13.4                   │  │   13.5                     │  │
    │  │  Favicon Generation      │  │  Social Preview Image      │  │
    │  │  (multi-size + ICO)      │  │  (1280x640 OG image)       │  │
    │  └──────────────────────────┘  └────────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘
              │                              │
              └──────────────┬───────────────┘
                             │
                             ▼
    WAVE 3 (Integration)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────┐  ┌────────────────────────────┐  │
    │  │   13.6                   │  │   13.7                     │  │
    │  │  Website Integration     │  │  README Polish             │  │
    │  │  (HTML meta tags)        │  │  (banner, badges)          │  │
    │  └──────────────────────────┘  └────────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 13.1 + 13.2 + 13.3      (3 tasks in parallel)         │
    │  Wave 2: 13.4 + 13.5             (2 tasks in parallel)         │
    │  Wave 3: 13.6 + 13.7             (2 tasks in parallel)         │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 13.1: Synthwave Built-in Palette

**Wave:** 1 (parallel with 13.2, 13.3)

Add synthwave colors as a built-in palette option.

**Deliverables:**
- Update `src/palettes/mod.rs`:
  ```rust
  // Add to BUILTIN_PALETTES
  ("synthwave", &[
      ("{_}", "#00000000"),
      ("{purple}", "#BD93F9"),
      ("{pink}", "#FF79C6"),
      ("{cyan}", "#8BE9FD"),
      ("{glow}", "#E2B3FF"),
      ("{hot}", "#FF2D95"),
      ("{neon}", "#00F7FF"),
      ("{bg}", "#282A36"),
  ])
  ```

**Verification:**
```bash
cargo test palettes
# Test: get_builtin("synthwave") returns correct colors
# Test: list_builtins() includes "synthwave"

./target/release/pxl palettes list | grep synthwave
./target/release/pxl palettes show synthwave
```

**Test Fixture:** `tests/fixtures/valid/palette_synthwave.jsonl`
```jsonl
{"type": "sprite", "name": "test", "palette": "@synthwave", "size": [4, 1], "grid": ["{purple}{pink}{cyan}{neon}"]}
```

**Dependencies:** Phase 11 complete

---

### Task 13.2: Favicon Sprite Definition

**Wave:** 1 (parallel with 13.1, 13.3)

Create the favicon sprite using pixelsrc format (dogfooding).

**Deliverables:**
- `examples/favicon_p.jsonl`:
  ```jsonl
  {"type": "palette", "name": "favicon", "colors": {"{_}": "#00000000", "{p}": "#BD93F9", "{g}": "#E2B3FF", "{bg}": "#282A36"}}
  {"type": "sprite", "name": "favicon_p", "size": [8, 8], "palette": "favicon", "grid": [
    "{_}{_}{_}{_}{_}{_}{_}{_}",
    "{_}{_}{_}{_}{_}{_}{_}{_}",
    "{_}{p}{p}{p}{_}{_}{_}{_}",
    "{_}{p}{g}{p}{_}{_}{_}{_}",
    "{_}{p}{p}{p}{_}{_}{_}{_}",
    "{_}{p}{_}{_}{_}{_}{_}{_}",
    "{_}{p}{_}{_}{_}{_}{_}{_}",
    "{_}{_}{_}{_}{_}{_}{_}{_}"
  ]}
  ```

**Verification:**
```bash
./target/release/pxl render examples/favicon_p.jsonl -o /tmp/
ls /tmp/favicon_p.png
file /tmp/favicon_p.png  # Should be 8x8 PNG
```

**Dependencies:** Phase 11 complete

---

### Task 13.3: Banner Sprite Definitions

**Wave:** 1 (parallel with 13.1, 13.2)

Create banner sprites for "pxl" text.

**Deliverables:**
- `examples/banner_pxl.jsonl` - Individual letter sprites + composition:
  ```jsonl
  {"type": "palette", "name": "banner", "colors": {"{_}": "#00000000", "{c}": "#8BE9FD", "{p}": "#BD93F9", "{x}": "#50FA7B", "{l}": "#FFB86C", "{k}": "#FF79C6"}}
  {"type": "sprite", "name": "bracket_l", "size": [3, 8], "palette": "banner", "grid": [...]}
  {"type": "sprite", "name": "letter_p", "size": [3, 8], "palette": "banner", "grid": [...]}
  {"type": "sprite", "name": "letter_x", "size": [3, 8], "palette": "banner", "grid": [...]}
  {"type": "sprite", "name": "letter_l", "size": [4, 8], "palette": "banner", "grid": [...]}
  {"type": "sprite", "name": "bracket_r", "size": [3, 8], "palette": "banner", "grid": [...]}
  {"type": "composition", "name": "banner_pxl", "size": [18, 8], "sprites": {"{": "bracket_l", "p": "letter_p", "x": "letter_x", "l": "letter_l", "}": "bracket_r", ".": null}, "layers": [{"map": ["{.p.x.l.}"]}]}
  ```

**Verification:**
```bash
./target/release/pxl render examples/banner_pxl.jsonl -o /tmp/
ls /tmp/banner_pxl.png
file /tmp/banner_pxl.png  # Should be 18x8 PNG
```

**Dependencies:** Phase 11 complete

---

### Task 13.4: Favicon Multi-Size Generation

**Wave:** 2 (after 13.2)

Generate favicon at all required sizes and create ICO file.

**Deliverables:**
- Script or CLI command to generate multiple sizes:
  ```bash
  # Generate scaled versions
  ./target/release/pxl render examples/favicon_p.jsonl --scale 2 -o website/favicon-16.png
  ./target/release/pxl render examples/favicon_p.jsonl --scale 4 -o website/favicon-32.png
  ./target/release/pxl render examples/favicon_p.jsonl --scale 6 -o website/favicon-48.png
  ./target/release/pxl render examples/favicon_p.jsonl --scale 24 -o website/favicon-192.png
  ./target/release/pxl render examples/favicon_p.jsonl --scale 64 -o website/favicon-512.png
  ```
- `website/favicon.ico` - Multi-resolution ICO (16, 32, 48)
- `website/manifest.json` - PWA manifest with icon references:
  ```json
  {
    "name": "pixelsrc",
    "icons": [
      {"src": "/favicon-192.png", "sizes": "192x192", "type": "image/png"},
      {"src": "/favicon-512.png", "sizes": "512x512", "type": "image/png"}
    ]
  }
  ```

**Verification:**
```bash
ls website/favicon*.png
file website/favicon.ico  # Should be ICO format
cat website/manifest.json | jq '.icons'
```

**Dependencies:** Task 13.2

---

### Task 13.5: Social Preview Image

**Wave:** 2 (after 13.3)

Create Open Graph image for social media link previews.

**Deliverables:**
- `examples/social_preview.jsonl` - 1280x640 composition:
  ```jsonl
  {"type": "composition", "name": "og_image", "size": [1280, 640], "cell_size": [64, 64], ...}
  ```
  Including:
  - pixelsrc logo/banner
  - Sample pixel art sprite
  - Tagline: "GenAI-native pixel art format"
  - Dark background (Dracula theme)
- Generated `website/og-image.png` (1280x640)

**Verification:**
```bash
./target/release/pxl render examples/social_preview.jsonl -o website/
file website/og_image.png
# Verify dimensions
identify website/og_image.png  # Should show 1280x640
```

**Dependencies:** Task 13.3

---

### Task 13.6: Website HTML Integration

**Wave:** 3 (after 13.4, 13.5)

Add favicon and OG meta tags to website HTML.

**Deliverables:**
- Update `website/index.html` `<head>` section:
  ```html
  <!-- Favicon -->
  <link rel="icon" type="image/x-icon" href="/favicon.ico">
  <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png">
  <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16.png">
  <link rel="apple-touch-icon" sizes="180x180" href="/favicon-180.png">
  <link rel="manifest" href="/manifest.json">

  <!-- Open Graph / Social -->
  <meta property="og:title" content="pixelsrc">
  <meta property="og:description" content="GenAI-native pixel art format">
  <meta property="og:image" content="https://pixelsrc.dev/og-image.png">
  <meta property="og:url" content="https://pixelsrc.dev">
  <meta property="og:type" content="website">

  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:title" content="pixelsrc">
  <meta name="twitter:description" content="GenAI-native pixel art format">
  <meta name="twitter:image" content="https://pixelsrc.dev/og-image.png">
  ```
- Fix casing: `Pixel<span>src</span>` → `pixel<span>src</span>`

**Verification:**
```bash
grep -c 'og:image' website/index.html  # Should be 1
grep -c 'favicon.ico' website/index.html  # Should be 1
grep 'pixelsrc' website/index.html  # Verify lowercase
```

**Dependencies:** Tasks 13.4, 13.5

---

### Task 13.7: README Polish

**Wave:** 3 (parallel with 13.6)

Improve README visual presentation.

**Deliverables:**
- Update `README.md`:
  - Add banner image at top: `![pixelsrc banner](docs/assets/banner.png)`
  - Add badges:
    ```markdown
    ![Version](https://img.shields.io/crates/v/pxl)
    ![License](https://img.shields.io/crates/l/pxl)
    ![Build](https://img.shields.io/github/actions/workflow/status/...)
    ```
  - Fix casing to lowercase `pixelsrc`
  - Add GIF demo of workflow (optional stretch goal)
  - Add "Made with pixelsrc" badge for community:
    ```markdown
    [![Made with pixelsrc](https://img.shields.io/badge/made%20with-pixelsrc-BD93F9)](https://pixelsrc.dev)
    ```
- Copy scaled banner to `docs/assets/banner.png`

**Verification:**
```bash
head -20 README.md  # Should show banner image
grep -c 'shields.io' README.md  # Should be >= 3
grep 'Pixelsrc\|PixelSrc' README.md  # Should be 0 (all lowercase)
```

**Dependencies:** Task 13.3

---

## Branding Checklist

After Phase 13, verify branding consistency:

| Location | Check |
|----------|-------|
| Website header | lowercase `pixelsrc` |
| README.md title | lowercase `pixelsrc` |
| package.json description | lowercase reference |
| GitHub repo description | lowercase + tagline |
| Favicon | `p` logo visible |
| Social preview | Shows on link share |

---

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. Synthwave palette available
./target/release/pxl palettes show synthwave

# 3. Favicon renders correctly
./target/release/pxl render examples/favicon_p.jsonl --scale 4 -o /tmp/favicon.png
open /tmp/favicon.png

# 4. Banner renders correctly
./target/release/pxl render examples/banner_pxl.jsonl --scale 4 -o /tmp/banner.png
open /tmp/banner.png

# 5. Website has meta tags
grep 'og:image' website/index.html

# 6. README has banner
head -5 README.md | grep banner

# 7. Test social preview (manual)
# Share link on Twitter/Slack and verify card appears
```

---

## Future Ideas

Not in scope for Phase 13:
- Animated favicon (GIF or SVG animation)
- Dark/light theme variants
- Holiday/seasonal variants
- Community-contributed themes
