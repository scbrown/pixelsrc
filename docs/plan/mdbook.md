---
phase: 21
title: mdbook Documentation
---

# Phase 21: mdbook Documentation

**Status:** Not Started

**Depends on:** Phase 16 (.pxl Format - for accurate format documentation)

**Related:**
- [Personas](../personas.md) - Content source for persona guides
- [Format Spec](../spec/format.md) - Content source for format reference

---

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

## Task Dependency Diagram

```
                           MDBOOK DOCUMENTATION TASK FLOW
═══════════════════════════════════════════════════════════════════════════════

WAVE 1 (Foundation - Sequential)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            MDB-1                                    │    │
│  │               Book Structure Setup                                  │    │
│  │               - Create docs/book/ directories                       │    │
│  │               - Configure book.toml                                 │    │
│  │               - Create SUMMARY.md scaffold                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            MDB-2                                    │    │
│  │               Getting Started Section                               │    │
│  │               - Introduction/README                                 │    │
│  │               - Installation guide                                  │    │
│  │               - Quick-start tutorial                                │    │
│  │               - Core concepts                                       │    │
│  │               Needs: MDB-1                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 2 (Core Content - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │         MDB-3                  │  │         MDB-4                  │    │
│  │    Format Specification        │  │    CLI Reference               │    │
│  │    (8 files)                   │  │    (15 files)                  │    │
│  │    - Palette spec              │  │    - render command            │    │
│  │    - Sprite spec               │  │    - fmt command               │    │
│  │    - Animation spec            │  │    - validate command          │    │
│  │    - Composition spec          │  │    - All 14 CLI commands       │    │
│  │    - Variant spec              │  │    Needs: MDB-1                │    │
│  │    Needs: MDB-1                │  │                                │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                    │
            └─────────────────┬──────────────────┘
                              ▼
WAVE 3 (Guides - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐          │
│  │      MDB-5       │  │      MDB-6       │  │      MDB-7       │          │
│  │  Persona Guides  │  │  Export Guides   │  │  Integration     │          │
│  │  (6 files)       │  │  (5 files)       │  │  Guides          │          │
│  │  - Sketcher      │  │  - PNG export    │  │  (4 files)       │          │
│  │  - Pixel Artist  │  │  - GIF export    │  │  - WASM module   │          │
│  │  - Animator      │  │  - Spritesheet   │  │  - Obsidian      │          │
│  │  - Motion Dsgn   │  │  - Atlas         │  │  - Web editor    │          │
│  │  - Game Dev      │  │  - JSON          │  │  - Tooling       │          │
│  │  Needs: MDB-2    │  │  Needs: MDB-3    │  │  Needs: MDB-3    │          │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘          │
└─────────────────────────────────────────────────────────────────────────────┘
            │                   │                   │
            └───────────────────┴───────────────────┘
                              │
                              ▼
WAVE 4 (Reference & AI - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │         MDB-8                  │  │         MDB-9                  │    │
│  │    AI Generation Guides        │  │    Reference Section           │    │
│  │    (4 files)                   │  │    (4 files)                   │    │
│  │    - System prompts            │  │    - Built-in palettes         │    │
│  │    - Best practices            │  │    - Color formats             │    │
│  │    - Example gallery           │  │    - Exit codes                │    │
│  │    - Troubleshooting           │  │    - Error messages            │    │
│  │    Needs: MDB-5                │  │    Needs: MDB-4                │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                    │
            └─────────────────┬──────────────────┘
                              ▼
WAVE 5 (WASM Demos - Sequential)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            MDB-10                                   │    │
│  │               WASM Demo Infrastructure                              │    │
│  │               - custom/js/wasm-demo.js                              │    │
│  │               - custom/css/custom.css (Dracula theme)               │    │
│  │               - theme/head.hbs for WASM loading                     │    │
│  │               Needs: MDB-3                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                        │
│                                    ▼                                        │
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │        MDB-11                  │  │        MDB-12                  │    │
│  │   Embedded Demos               │  │   Standalone Playground        │    │
│  │   - Add "try it" to           │  │   - Sandbox page               │    │
│  │     getting-started           │  │   - Example gallery            │    │
│  │   - Add to format pages       │  │   - Shareable links            │    │
│  │   - Add to persona guides     │  │   Needs: MDB-10                │    │
│  │   Needs: MDB-10               │  │                                │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                    │
            └─────────────────┬──────────────────┘
                              ▼
WAVE 6 (CI/CD & Polish)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            MDB-13                                   │    │
│  │               CI/CD Integration                                     │    │
│  │               - Update website.yml workflow                         │    │
│  │               - Install mdbook step                                 │    │
│  │               - Copy WASM assets to book                            │    │
│  │               - Build mdbook                                        │    │
│  │               - Combine website + /book/                            │    │
│  │               Needs: MDB-11, MDB-12                                 │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                        │
│                                    ▼                                        │
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │        MDB-14                  │  │        MDB-15                  │    │
│  │   Local Testing               │  │   Cross-Browser Testing        │    │
│  │   - mdbook serve works        │  │   - Chrome verification        │    │
│  │   - WASM demos functional     │  │   - Firefox verification       │    │
│  │   - Link check passes         │  │   - Safari verification        │    │
│  │   Needs: MDB-13               │  │   Needs: MDB-13                │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            MDB-16                                   │    │
│  │               Final Deployment & Review                             │    │
│  │               - Deploy to GitHub Pages                              │    │
│  │               - Verify at /book/ path                               │    │
│  │               - Final content review                                │    │
│  │               - Update project README with book link                │    │
│  │               Needs: MDB-14, MDB-15                                 │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Wave 1: MDB-1 → MDB-2                         (sequential)                 │
│  Wave 2: MDB-3 + MDB-4                         (2 tasks in parallel)        │
│  Wave 3: MDB-5 + MDB-6 + MDB-7                 (3 tasks in parallel)        │
│  Wave 4: MDB-8 + MDB-9                         (2 tasks in parallel)        │
│  Wave 5: MDB-10 → MDB-11 + MDB-12              (then 2 in parallel)         │
│  Wave 6: MDB-13 → MDB-14 + MDB-15 → MDB-16     (mixed)                      │
└─────────────────────────────────────────────────────────────────────────────┘

CRITICAL PATH: MDB-1 → MDB-2 → MDB-5 → MDB-8 → MDB-10 → MDB-11 → MDB-13 → MDB-16

BEADS CREATION ORDER:
  1. MDB-1 (no deps)
  2. MDB-2 (dep: MDB-1)
  3. MDB-3, MDB-4 (dep: MDB-1)
  4. MDB-5 (dep: MDB-2), MDB-6, MDB-7 (dep: MDB-3)
  5. MDB-8 (dep: MDB-5), MDB-9 (dep: MDB-4)
  6. MDB-10 (dep: MDB-3)
  7. MDB-11, MDB-12 (dep: MDB-10)
  8. MDB-13 (dep: MDB-11, MDB-12)
  9. MDB-14, MDB-15 (dep: MDB-13)
  10. MDB-16 (dep: MDB-14, MDB-15)
```

---

## Tasks

### Task MDB-1: Book Structure Setup

**Wave:** 1

Create the mdbook directory structure and configuration.

**Deliverables:**
- Create directory structure:
  ```
  docs/book/
  ├── book.toml
  ├── custom/
  │   ├── css/
  │   └── js/
  ├── theme/
  └── src/
      └── SUMMARY.md
  ```

- Configure `book.toml`:
  ```toml
  [book]
  title = "Pixelsrc Documentation"
  authors = ["Steve Brown"]
  description = "Complete documentation for Pixelsrc"
  src = "src"
  language = "en"

  [build]
  build-dir = "book"

  [output.html]
  default-theme = "coal"
  preferred-dark-theme = "coal"
  site-url = "/book/"
  additional-css = ["custom/css/custom.css"]
  additional-js = ["custom/js/wasm-demo.js"]

  [output.html.search]
  enable = true
  ```

- Create `SUMMARY.md` scaffold with all planned sections

**Verification:**
```bash
cd docs/book && mdbook build
# Should build without errors (empty pages OK)
```

**Dependencies:** None

---

### Task MDB-2: Getting Started Section

**Wave:** 1 (after MDB-1)

Write the introduction and getting started documentation.

**Deliverables:**
- `src/README.md` - Introduction: what is pixelsrc, why use it
- `src/getting-started/installation.md` - All installation methods
- `src/getting-started/quick-start.md` - First sprite tutorial
- `src/getting-started/concepts.md` - Core concepts (palettes, sprites, tokens)

**Verification:**
```bash
cd docs/book && mdbook serve
# Navigate to getting-started, verify content is complete
```

**Dependencies:** Task MDB-1

---

### Task MDB-3: Format Specification

**Wave:** 2 (parallel with MDB-4)

Split and expand format.md into 8 separate documentation files.

**Deliverables:**
- `src/format/overview.md` - Format introduction, JSONL basics
- `src/format/palette.md` - Palette definition, color formats
- `src/format/sprite.md` - Sprite structure, grid syntax
- `src/format/animation.md` - Animation frames, timing
- `src/format/composition.md` - Layer composition, tiling
- `src/format/variant.md` - Sprite variants
- `src/format/transform.md` - Transform operations
- `src/format/metadata.md` - Metadata fields, nine-slice, hitboxes

**Source:** `docs/spec/format.md`

**Verification:**
```bash
mdbook build && mdbook-linkcheck
# All format pages render, no broken links
```

**Dependencies:** Task MDB-1

---

### Task MDB-4: CLI Reference

**Wave:** 2 (parallel with MDB-3)

Document all CLI commands with examples.

**Deliverables:**
- `src/cli/overview.md` - CLI introduction, common patterns
- `src/cli/render.md` - render command (PNG, GIF, spritesheet)
- `src/cli/fmt.md` - Format command
- `src/cli/validate.md` - Validation command
- `src/cli/import.md` - PNG import command
- `src/cli/palettes.md` - Palette management commands
- `src/cli/analyze.md` - Corpus analysis command
- `src/cli/prime.md` - AI priming command
- `src/cli/suggest.md` - Suggestion command
- `src/cli/show.md` - Terminal preview command
- `src/cli/transform.md` - Transform command
- `src/cli/init.md` - Project init command
- `src/cli/build.md` - Build command
- `src/cli/diff.md` - Diff command
- `src/cli/explain.md` - Explain command

**Source:** `src/cli.rs` help text

**Verification:**
```bash
# Each command page shows: synopsis, options, examples
mdbook build
```

**Dependencies:** Task MDB-1

---

### Task MDB-5: Persona Guides

**Wave:** 3 (after MDB-2)

Create comprehensive guides for each persona.

**Deliverables:**
- `src/personas/overview.md` - Persona introduction
- `src/personas/sketcher.md` - Quick prototyping workflow
- `src/personas/pixel-artist.md` - Static art creation workflow
- `src/personas/animator.md` - Animation workflow
- `src/personas/motion-designer.md` - Advanced animation workflow
- `src/personas/game-developer.md` - Game integration workflow

Each guide includes:
- Target use case and skill level
- Recommended setup
- Step-by-step tutorial
- Common patterns
- Tips and best practices

**Source:** `docs/personas.md`

**Verification:**
```bash
# Each persona guide has complete workflow example
mdbook build
```

**Dependencies:** Task MDB-2

---

### Task MDB-6: Export Format Guides

**Wave:** 3 (parallel with MDB-5, MDB-7)

Document all export formats.

**Deliverables:**
- `src/exports/overview.md` - Export format comparison
- `src/exports/png.md` - PNG export options
- `src/exports/gif.md` - Animated GIF export
- `src/exports/spritesheet.md` - Spritesheet generation
- `src/exports/atlas.md` - Atlas packing and JSON output

**Verification:**
```bash
# Each export page shows: command, options, output example
mdbook build
```

**Dependencies:** Task MDB-3

---

### Task MDB-7: Integration Guides

**Wave:** 3 (parallel with MDB-5, MDB-6)

Document platform integrations.

**Deliverables:**
- `src/integrations/overview.md` - Integration options
- `src/integrations/wasm.md` - WASM module usage in JavaScript
- `src/integrations/obsidian.md` - Obsidian plugin setup
- `src/integrations/web-editor.md` - Using the web editor
- `src/integrations/vscode.md` - VS Code setup (syntax highlighting)

**Source:** `wasm/README.md`, `obsidian-pixelsrc/README.md`

**Verification:**
```bash
mdbook build
# Integration pages have working code examples
```

**Dependencies:** Task MDB-3

---

### Task MDB-8: AI Generation Guides

**Wave:** 4 (after MDB-5)

Document AI-assisted sprite generation.

**Deliverables:**
- `src/ai-generation/overview.md` - AI generation introduction
- `src/ai-generation/system-prompts.md` - Crafting effective prompts
- `src/ai-generation/best-practices.md` - Tips for reliable generation
- `src/ai-generation/examples.md` - Example prompts and outputs

**Source:** `docs/prompts/`

**Verification:**
```bash
mdbook build
# AI pages include actual prompt examples
```

**Dependencies:** Task MDB-5

---

### Task MDB-9: Reference Section

**Wave:** 4 (parallel with MDB-8)

Create reference documentation.

**Deliverables:**
- `src/reference/palettes.md` - Built-in palette catalog with samples
- `src/reference/colors.md` - Color format reference (hex, named, etc.)
- `src/reference/exit-codes.md` - CLI exit codes and meanings
- `src/reference/errors.md` - Common errors and solutions

**Verification:**
```bash
mdbook build
# Reference pages are complete and accurate
```

**Dependencies:** Task MDB-4

---

### Task MDB-10: WASM Demo Infrastructure

**Wave:** 5

Create the JavaScript and CSS infrastructure for interactive demos.

**Deliverables:**
- `custom/js/wasm-demo.js`:
  ```javascript
  // Load WASM module
  // Expose pixelsrcDemo.render(jsonl, containerId)
  // Expose pixelsrcDemo.validate(jsonl)
  // Expose pixelsrcDemo.listSprites(jsonl)
  ```

- `custom/css/custom.css`:
  - Dracula theme styling
  - Demo container styles
  - Textarea and button styles
  - Preview canvas styles

- `theme/head.hbs`:
  - WASM module loading
  - Demo initialization

**Verification:**
```bash
mdbook serve
# Open browser console, verify pixelsrcDemo object exists
```

**Dependencies:** Task MDB-3

---

### Task MDB-11: Embedded Demos

**Wave:** 5 (after MDB-10)

Add "try it" demos to existing pages.

**Deliverables:**
Add demo blocks to:
- `src/getting-started/quick-start.md` - First sprite demo
- `src/format/palette.md` - Color definition demo
- `src/format/sprite.md` - Grid structure demo
- `src/format/animation.md` - Frame preview demo
- `src/personas/*.md` - Workflow demos for each persona

Demo format:
```markdown
<div class="pixelsrc-demo">
  <textarea id="demo-id">{"type":"sprite",...}</textarea>
  <button onclick="pixelsrcDemo.render(...)">Try it</button>
  <div id="demo-id-preview"></div>
</div>
```

**Verification:**
```bash
mdbook serve
# Click "Try it" buttons, verify sprites render
```

**Dependencies:** Task MDB-10

---

### Task MDB-12: Standalone Playground

**Wave:** 5 (parallel with MDB-11)

Create the interactive playground section.

**Deliverables:**
- `src/playground/sandbox.md` - Full-featured editor sandbox
  - Large textarea for editing
  - Live preview panel
  - Error display
  - Download buttons (PNG, GIF)

- `src/playground/gallery.md` - Example gallery
  - Curated sprite examples
  - Click to load into sandbox
  - Category filtering (characters, items, UI, etc.)

**Verification:**
```bash
mdbook serve
# Playground loads, can edit and preview sprites
```

**Dependencies:** Task MDB-10

---

### Task MDB-13: CI/CD Integration

**Wave:** 6

Update GitHub Actions workflow to build and deploy mdbook.

**Deliverables:**
- Update `.github/workflows/website.yml`:
  ```yaml
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
  ```

**Verification:**
```bash
# Push to branch, verify workflow runs successfully
# Check Actions log for mdbook build step
```

**Dependencies:** Tasks MDB-11, MDB-12

---

### Task MDB-14: Local Testing

**Wave:** 6 (after MDB-13)

Comprehensive local testing of the documentation.

**Deliverables:**
- Run `mdbook serve` and verify all pages render
- Test all WASM demos work correctly
- Run `mdbook-linkcheck` and fix any broken links
- Verify search functionality works
- Test navigation and TOC

**Verification:**
```bash
cd docs/book
mdbook serve &
# Manual testing of all sections
mdbook-linkcheck
```

**Dependencies:** Task MDB-13

---

### Task MDB-15: Cross-Browser Testing

**Wave:** 6 (parallel with MDB-14)

Test WASM demos across browsers.

**Deliverables:**
- Test in Chrome: all demos render correctly
- Test in Firefox: all demos render correctly
- Test in Safari: all demos render correctly
- Document any browser-specific issues
- Add polyfills if needed

**Verification:**
```bash
# Manual testing in each browser
# Verify WASM loads and demos work
```

**Dependencies:** Task MDB-13

---

### Task MDB-16: Final Deployment & Review

**Wave:** 6 (after MDB-14, MDB-15)

Deploy and perform final review.

**Deliverables:**
- Push to main branch
- Verify deployment at `/book/` path works
- Final content review for accuracy
- Update project README.md with book link
- Add book link to website navigation

**Verification:**
```bash
# After merge to main:
curl -I https://scbrown.github.io/pixelsrc/book/
# Should return 200 OK

# README has book link
grep "book" README.md
```

**Dependencies:** Tasks MDB-14, MDB-15

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
