# Pixelsrc Implementation Plan

This directory contains the phased implementation plan for Pixelsrc.

## Overview

| Phase | Goal | Status |
|-------|------|--------|
| [Phase 0](./phase-0-mvp.md) | MVP - Parse and render sprites to PNG | Complete |
| [Phase 1](./phase-1-palettes.md) | Built-in palette library | Complete |
| [Phase 2](./phase-2-composition.md) | Unified composition system (sprites & scenes) | Complete |
| [Phase 2.5](./phase-2.5-upscaling.md) | Output upscaling (integer scale factor) | Complete |
| [Phase 3](./phase-3-animation.md) | Animation and spritesheet export | Complete |
| [Phase 4](./phase-4-rename.md) | Project rename (TTP → Pixelsrc) | Complete |
| [Phase 5](./phase-5-cli-extras.md) | CLI extras (PNG import, prompts, emoji) | Complete |
| [Phase 6](./phase-6-wasm.md) | **WASM Foundation** | Complete |
| [Phase 7](./phase-7-website.md) | **Interactive Website** | Complete |
| [Phase 8](./phase-8-obsidian.md) | **Obsidian Plugin** | Complete |
| [Phase 9](./phase-9-packages.md) | **Package Distribution** | Complete |
| [Phase 10](./phase-10-github-migration.md) | **GitHub Migration** | Complete |
| [Phase 11](./phase-11-website-improvements.md) | **Website Improvements** (Dracula theme, loading states, polish) | Planning |
| [Phase 12](./phase-12-tiling.md) | **Composition Tiling** (`cell_size` for large images) | Complete |
| [Phase 13](./phase-13-theming.md) | **Theming & Branding** (favicon, banners, social preview) | Planning |
| [Phase 14](./phase-14-analyze.md) | **Corpus Analysis** (`pxl analyze` for usage metrics) | Planning |
| [Phase 15](./phase-15-ai-tools.md) | **AI Assistance Tools** (`pxl prime`, `validate`, `suggest`) | Planning |

### Future Ideas

| Idea | Description |
|------|-------------|
| VS Code Extension | Syntax highlighting + live preview |
| Token Efficiency | Run-length encoding, row repetition, compression |
| Inheritance | Scene variants, extends, day/night themes |
| Edge Constraints | Tile connectivity validation (see BACKLOG) |
| Metadata/Frontmatter | Optional metadata convention separate from spec (see BACKLOG) |

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Name | Pixelsrc | Clear, describes source format nature |
| Syntax | JSONL (JSON Lines) | Streaming-friendly, each line self-describing |
| Tokens | Multi-char (`{skin}`) | Readability, expressability |
| Coordinates | Implicit (grid position) | Avoids GenAI weakness |
| Transparency | `{_}` | Short, minimal, common "empty" convention |
| CLI command | `pxl` | Short and punchy |
| Language | Rust | Single binary, WASM target, fast streaming |
| Image rendering | Native via `image` crate | Pure Rust, no external deps |
| Error handling | Lenient default, strict opt-in | GenAI-friendly: fill gaps, warn, continue |
| File extension | Any (`.pxs` or `.jsonl`) | CLI accepts any file, extension is convention |

## Project Structure

```
pixelsrc/
├── docs/
│   ├── VISION.md              # Project vision and tenets
│   ├── ANNOUNCEMENT.md        # Product positioning
│   ├── BACKLOG.md             # Deferred features (game integration, etc.)
│   ├── spec/
│   │   └── format.md          # Formal JSONL specification
│   └── plan/
│       ├── README.md          # This file
│       ├── phase-0-mvp.md
│       ├── phase-1-palettes.md
│       ├── phase-2-composition.md
│       ├── phase-2.5-upscaling.md
│       ├── phase-3-animation.md
│       ├── phase-4-rename.md
│       ├── phase-5-cli-extras.md
│       ├── phase-6-wasm.md
│       ├── phase-7-website.md
│       ├── phase-8-obsidian.md
│       ├── phase-9-packages.md
│       ├── phase-10-github-migration.md
│       ├── phase-11-website-improvements.md
│       ├── phase-12-tiling.md
│       ├── phase-13-theming.md
│       ├── phase-14-analyze.md
│       └── phase-15-ai-tools.md
├── CONTRIBUTING.md            # Dev setup, conventions
├── Cargo.toml                 # Rust package config
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── parser.rs
│   ├── renderer.rs
│   └── models.rs
├── examples/
│   ├── coin.jsonl
│   ├── hero.jsonl
│   └── walk_cycle.jsonl
└── tests/
    └── fixtures/
        ├── valid/
        ├── invalid/
        └── lenient/
```

## Task Management

Tasks are managed via [Beads](https://github.com/steveyegge/beads). Each phase document contains:
- Goal and scope
- Discrete tasks sized for agent completion (~60% context window)
- Dependencies between tasks
- Acceptance criteria per task
- Parallelization notes

## Key Documents

- [VISION.md](../VISION.md) - Why we're building this
- [spec/format.md](../spec/format.md) - Formal specification
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - How to contribute
