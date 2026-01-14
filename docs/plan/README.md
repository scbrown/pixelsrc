# TTP Implementation Plan

This directory contains the phased implementation plan for TTP (Text To Pixel).

## Overview

| Phase | Goal | Status |
|-------|------|--------|
| [Phase 0](./phase-0-mvp.md) | MVP - Parse and render sprites to PNG | Complete |
| [Phase 1](./phase-1-palettes.md) | Built-in palette library | Complete |
| [Phase 2](./phase-2-composition.md) | Unified composition system (sprites & scenes) | Planning |
| [Phase 2.5](./phase-2.5-upscaling.md) | Output upscaling (integer scale factor) | Planning |
| [Phase 3](./phase-3-animation.md) | Animation and spritesheet export | Planning |
| [Phase 4](./phase-4-game-integration.md) | Game engine format export | Planning |
| [Phase 5](./phase-5-ecosystem.md) | Developer tooling and ecosystem | Planning |

### Future Ideas

| Idea | Description |
|------|-------------|
| Phase 6: Token Efficiency | Run-length encoding, row repetition, compression |
| Phase 7: Inheritance | Scene variants, extends, day/night themes |

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Name | TTP (Text To Pixel) | Clear, memorable |
| Syntax | JSONL (JSON Lines) | Streaming-friendly, each line self-describing |
| Tokens | Multi-char (`{skin}`) | Readability, expressability |
| Coordinates | Implicit (grid position) | Avoids GenAI weakness |
| Transparency | `{_}` | Short, minimal, common "empty" convention |
| CLI command | `pxl` | Short and punchy |
| Language | Rust | Single binary, WASM target, fast streaming |
| Image rendering | Native via `image` crate | Pure Rust, no external deps |
| Error handling | Lenient default, strict opt-in | GenAI-friendly: fill gaps, warn, continue |
| File extension | Any (`.ttp` or `.jsonl`) | CLI accepts any file, extension is convention |

## Project Structure

```
ttp/
├── docs/
│   ├── VISION.md              # Project vision and tenets
│   ├── ANNOUNCEMENT.md        # Product positioning
│   ├── spec/
│   │   └── format.md          # Formal JSONL specification
│   └── plan/
│       ├── README.md          # This file
│       ├── phase-0-mvp.md
│       ├── phase-1-palettes.md
│       ├── phase-2-composition.md
│       ├── phase-3-animation.md
│       ├── phase-4-game-integration.md
│       └── phase-5-ecosystem.md
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
