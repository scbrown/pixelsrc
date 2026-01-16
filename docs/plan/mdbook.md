# Pixelsrc mdbook Documentation Plan

## Overview

Add mdbook-based documentation to pixelsrc with:
- Exhaustive project documentation
- Persona-specific getting started guides
- Interactive WASM demos embedded in the book
- GitHub Pages deployment at `/book/` path (alongside existing website)

## Directory Structure

```
docs/book/
├── book.toml                 # mdbook configuration
├── custom/
│   ├── css/custom.css        # Dracula theme, demo styling
│   └── js/wasm-demo.js       # WASM initialization + demo API
├── theme/
│   └── head.hbs              # Custom head for WASM loading
├── src/
│   ├── SUMMARY.md            # Table of contents
│   ├── README.md             # Introduction
│   ├── getting-started/      # Installation, quick-start, concepts
│   ├── personas/             # 5 persona guides with setup workflows
│   ├── format/               # Palette, sprite, animation, variant, composition specs
│   ├── cli/                  # All 14 CLI commands documented
│   ├── exports/              # PNG, GIF, spritesheet, atlas formats
│   ├── integrations/         # WASM, Obsidian, web editor
│   ├── ai-generation/        # System prompts, best practices, examples
│   ├── playground/           # Standalone interactive playground/sandbox
│   ├── reference/            # Built-in palettes, color formats, exit codes
│   └── appendix/             # Vision, changelog, contributing
└── wasm-assets/              # WASM files (copied during build)
```

**Note:** "Try it" demos embedded throughout pages AND standalone playground section.

## WASM Demo Integration

**Approach:** Custom JavaScript that loads WASM module and exposes `window.pixelsrcDemo` API.

**Philosophy:** "Try it" examples should be embedded contextually throughout the documentation, not isolated to a demo section. When explaining a concept, include an interactive example right there.

**Example - embedded in format/sprite.md:**
```markdown
## Grid Structure

The `grid` array defines pixel rows using token references:

<div class="pixelsrc-demo">
  <textarea id="sprite-grid-demo">{"type":"sprite","name":"star","palette":{"{_}":"#0000","{y}":"#FFD700"},"grid":["{_}{y}{_}","{y}{y}{y}","{_}{y}{_}"]}</textarea>
  <button onclick="pixelsrcDemo.render(...)">Try it</button>
  <div id="sprite-grid-preview"></div>
</div>

Try changing `{y}` to `{r}` with `"#FF0000"` to make a red star.
```

**Where to embed demos:**
- `getting-started/quick-start.md` - First sprite example
- `format/palette.md` - Color definition examples
- `format/sprite.md` - Grid structure examples
- `format/animation.md` - Frame sequence preview
- `personas/*.md` - Persona-specific workflow examples
- `cli/render.md` - Show what render produces

**API exposed:**
- `pixelsrcDemo.render(jsonl, containerId)` - Render sprite to container
- `pixelsrcDemo.validate(jsonl)` - Return validation messages
- `pixelsrcDemo.listSprites(jsonl)` - List sprite names

## Content Outline (SUMMARY.md)

1. **Introduction** - What is pixelsrc, why use it
2. **Getting Started** (4 pages) - Installation, quick-start, concepts [+ "try it" demos]
3. **Persona Guides** (6 pages) - Sketcher through Game Developer [+ workflow demos]
4. **Format Specification** (8 pages) - All format types with examples [+ interactive demos]
5. **CLI Reference** (15 pages) - Every command documented [+ output previews]
6. **Export Formats** (5 pages) - PNG, GIF, spritesheet, atlas
7. **Integrations** (4 pages) - WASM, Obsidian, web editor
8. **AI Generation** (4 pages) - Prompts and best practices
9. **Playground** (2 pages) - Standalone sandbox, example gallery
10. **Reference** (4 pages) - Palettes, colors, exit codes
11. **Appendix** (3 pages) - Vision, changelog, contributing

**Total: ~55 documentation pages**

**Demo strategy:** "Try it" examples embedded contextually throughout docs + standalone playground for free-form experimentation

## book.toml Configuration

```toml
[book]
title = "Pixelsrc Documentation"
authors = ["Steve Brown"]
description = "Complete documentation for Pixelsrc - the GenAI-native pixel art format"
src = "src"
language = "en"

[build]
build-dir = "book"

[output.html]
default-theme = "coal"
preferred-dark-theme = "coal"
git-repository-url = "https://github.com/stiwi/pixelsrc"
edit-url-template = "https://github.com/stiwi/pixelsrc/edit/main/docs/book/{path}"
additional-css = ["custom/css/custom.css"]
additional-js = ["custom/js/wasm-demo.js"]
site-url = "/book/"

[output.html.search]
enable = true

[output.linkcheck]
warning-policy = "warn"
```

## GitHub Actions Workflow

Update existing `.github/workflows/website.yml` to build both website and book:

```yaml
# Updated website.yml - adds book build step
jobs:
  build:
    steps:
      # ... existing WASM and website build steps ...

      # NEW: Build mdbook documentation
      - name: Install mdbook
        run: |
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v0.4.40/mdbook-v0.4.40-x86_64-unknown-linux-gnu.tar.gz | tar -xz
          chmod +x mdbook
          sudo mv mdbook /usr/local/bin/

      - name: Copy WASM assets to book
        run: |
          mkdir -p docs/book/wasm-assets
          cp wasm/pkg/pixelsrc_wasm.js docs/book/wasm-assets/
          cp wasm/pkg/pixelsrc_wasm_bg.wasm docs/book/wasm-assets/

      - name: Build mdbook
        run: mdbook build docs/book

      - name: Combine website and book
        run: |
          mkdir -p website/dist/book
          cp -r docs/book/book/* website/dist/book/

      # Deploy combined site (website + /book/)
      - uses: actions/upload-pages-artifact@v3
        with:
          path: website/dist
```

## Content Migration

| Source | Destination |
|--------|-------------|
| `docs/spec/format.md` | Split into `format/*.md` |
| `docs/personas.md` | Split into `personas/*.md` |
| `docs/prompts/*.md` | Adapt to `ai-generation/*.md` |
| `docs/VISION.md` | `appendix/vision.md` |
| `docs/primer.md` | Reference for getting-started |

## Implementation Phases

### Phase 1: Foundation
- [ ] Create `docs/book/` directory structure
- [ ] Configure `book.toml` (theme: coal, site-url, linkcheck)
- [ ] Create `SUMMARY.md` with complete TOC
- [ ] Write introduction and getting-started section

### Phase 2: Core Documentation
- [ ] Format specification (8 files from format.md)
- [ ] CLI reference (15 files from cli.rs help)

### Phase 3: Persona Guides
- [ ] Split and expand `docs/personas.md` into 6 guides
- [ ] Add complete setup workflows for each persona

### Phase 4: Exports & Integrations
- [ ] Export format documentation (5 files)
- [ ] Integration guides (WASM, Obsidian, web editor)

### Phase 5: AI & Reference
- [ ] AI generation guides from `docs/prompts/`
- [ ] Reference section (palettes, colors, exit codes)

### Phase 6: WASM Demos & Playground
- [ ] Create `custom/js/wasm-demo.js`
- [ ] Create `custom/css/custom.css` (Dracula theme)
- [ ] Add "try it" demos to key pages (getting-started, format, personas)
- [ ] Build standalone playground section (sandbox + example gallery)

### Phase 7: CI/CD & Polish
- [ ] Update `.github/workflows/website.yml` with mdbook build steps
- [ ] Test full site build locally
- [ ] Deploy and verify at /book/ path
- [ ] Final review

## Key Files to Modify/Create

**New files:**
- `docs/plan/mdbook.md` - This plan document
- `docs/book/book.toml` - mdbook configuration
- `docs/book/src/SUMMARY.md` - Table of contents
- `docs/book/src/**/*.md` (~55 documentation files)
- `docs/book/custom/js/wasm-demo.js` - WASM demo loader
- `docs/book/custom/css/custom.css` - Dracula theme styling

**Modified files:**
- `.github/workflows/website.yml` - Add mdbook build steps

**Reference files (content to migrate):**
- `docs/spec/format.md` - Format specification source
- `docs/personas.md` - Persona documentation source
- `docs/prompts/*.md` - AI prompt content
- `src/cli.rs` - CLI command definitions
- `wasm/README.md` - WASM API documentation

## Verification

1. **Local testing:** `mdbook serve` in `docs/book/` - verify at http://localhost:3000
2. **WASM demos:** Click "Render" buttons, verify sprites display correctly
3. **Link check:** Run `mdbook-linkcheck` - no broken links
4. **Full site build:** Run website build locally, verify `/book/` path works
5. **Deployment:** Push to main, verify at https://stiwi.github.io/pixelsrc/book/
6. **Cross-browser:** Test demos in Chrome, Firefox, Safari
