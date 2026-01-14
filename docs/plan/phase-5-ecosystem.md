# Phase 5: Ecosystem

**Goal:** Developer tooling and ecosystem support

**Status:** Planning

**Depends on:** Phase 0 complete (some tasks), Phase 3 complete (others)

---

## Scope

Phase 5 adds developer experience improvements:
- VS Code extension
- Web-based previewer
- PNG import (reverse engineering)
- GenAI prompt templates
- Emoji art output mode

**Not in scope:** IDE plugins for other editors

---

## Tasks

### Task 5.1: VS Code Extension
**Parallelizable:** Yes

Syntax highlighting and preview for TTP files.

**Deliverables:**
- TextMate grammar for `.jsonl` TTP files
- Syntax highlighting for tokens, colors
- Preview panel showing rendered sprite
- Publish to VS Code marketplace

**Acceptance Criteria:**
- Syntax highlighting works
- Preview updates on save
- Extension installs from marketplace

---

### Task 5.2: Web Previewer
**Parallelizable:** Yes

Browser-based TTP editor and previewer.

**Deliverables:**
- WASM build of renderer
- Simple web UI with:
  - Text editor for TTP input
  - Live preview of rendered sprites
  - Download PNG button
- Host on GitHub Pages or similar

**Acceptance Criteria:**
- Paste TTP, see rendered output
- Download works
- Mobile-friendly

---

### Task 5.3: PNG Import
**Parallelizable:** Yes

Convert PNG images to TTP format.

**Deliverables:**
- `pxl import image.png -o sprite.jsonl`
- Analyze colors, generate palette
- Generate grid with auto-named tokens
- Optional: quantize to N colors

**Acceptance Criteria:**
- Simple images convert correctly
- Round-trip: import → render matches original
- Color quantization works

---

### Task 5.4: GenAI Prompt Templates
**Parallelizable:** Yes

Templates and guides for GenAI sprite generation.

**Deliverables:**
- `docs/prompts/` directory with:
  - System prompt for sprite generation
  - Example prompts with outputs
  - Best practices guide
- Include in CLI: `pxl prompts show <template>`

**Acceptance Criteria:**
- Templates produce valid TTP output
- Guide is clear and helpful

---

### Task 5.5: Emoji Art Output
**Parallelizable:** Yes (after Phase 0)

Text-based output using emoji for quick preview.

**Deliverables:**
- `pxl render input.jsonl --emoji`
- Map colors to closest emoji
- Output text grid to stdout
- Simplified palette (limited emoji colors)

**Acceptance Criteria:**
- Output is visually recognizable
- Works in terminals that support emoji

---

## Dependency Graph

```
Phase 0 complete ──┬── 5.3 (PNG import)
                   ├── 5.4 (prompts)
                   └── 5.5 (emoji)

Phase 3 complete ──┬── 5.1 (VS Code)
                   └── 5.2 (web)
```

---

## Verification

1. VS Code extension works with example files
2. Web previewer renders examples correctly
3. PNG import produces valid TTP
4. GenAI (Claude) can generate sprites using templates
5. Emoji output is recognizable
