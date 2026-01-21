# Pixelsrc

[![CI](https://github.com/scbrown/pixelsrc/actions/workflows/ci.yml/badge.svg)](https://github.com/scbrown/pixelsrc/actions/workflows/ci.yml)
[![Release](https://github.com/scbrown/pixelsrc/actions/workflows/release.yml/badge.svg)](https://github.com/scbrown/pixelsrc/actions/workflows/release.yml)
[![WASM](https://github.com/scbrown/pixelsrc/actions/workflows/wasm.yml/badge.svg)](https://github.com/scbrown/pixelsrc/actions/workflows/wasm.yml)
[![npm](https://img.shields.io/npm/v/@stiwi/pixelsrc-wasm)](https://www.npmjs.com/package/@stiwi/pixelsrc-wasm)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**The first pixel art format designed for GenAI.**

Pixelsrc is a semantic, human-readable text format for defining pixel art. Unlike traditional editors or hex-based formats, it's designed from the ground up for AI systems to generate reliably.

```jsonl
{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{shine}": "#FFFACD"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": [
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}",
  "{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{gold}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{gold}{gold}",
  "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}",
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}"
]}
```

## Why Pixelsrc?

- **Semantic tokens** - Use meaningful names like `{skin}`, `{gold}`, `{shadow}` instead of single characters or hex codes
- **GenAI-native** - No coordinate systems or spatial reasoning; just sequential rows that LLMs can generate reliably
- **Streaming-first** - JSONL format enables real-time parsing as AI generates each line
- **Lenient by default** - Fills gaps and continues on small errors; strict mode available for CI
- **Human-readable** - Git diffs of sprite changes are meaningful and reviewable

## Installation

### Homebrew (macOS/Linux)

```bash
brew install scbrown/tap/pixelsrc
```

### Cargo (from source)

```bash
cargo install --git https://github.com/scbrown/pixelsrc
```

### Download binaries

Pre-built binaries for Linux, macOS, and Windows are available on the [Releases](https://github.com/scbrown/pixelsrc/releases) page.

## Quick Start

1. Create a file `hero.pxl`:

```jsonl
{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFD5B4", "{hair}": "#8B4513", "{shirt}": "#4169E1"}}
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "hero", "grid": [
  "{_}{_}{hair}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}"
]}
```

2. Render it:

```bash
pxl render hero.pxl -o hero.png
```

3. Scale it up:

```bash
pxl render hero.pxl -o hero.png --scale 8
```

## Features

### CLI Tool (`pxl`)

| Command | Description |
|---------|-------------|
| `pxl render <file>` | Render .pxl/.jsonl to PNG |
| `pxl render --gif` | Export animations to GIF |
| `pxl render --spritesheet` | Generate sprite sheets |
| `pxl fmt <files>` | Format files for readability |
| `pxl palettes list` | List built-in palettes |
| `pxl import <image>` | Convert PNG to .pxl |

### Format Capabilities

- **Palettes** - Define reusable color schemes
- **Sprites** - Pixel grids with semantic tokens
- **Animations** - Frame sequences with timing
- **Compositions** - Layer sprites into scenes
- **Built-in palettes** - Gameboy, Dracula, and more

### Integrations

- **[Web Editor](https://scbrown.github.io/pixelsrc/)** - Live editor with real-time preview
- **[WASM Module](https://www.npmjs.com/package/@stiwi/pixelsrc-wasm)** - Use in JavaScript/TypeScript
- **[Obsidian Plugin](obsidian-pixelsrc/)** - Render sprites in your notes

## Development

This project uses [just](https://github.com/casey/just) as a command runner:

```bash
just --list      # Show all available commands
just build       # Build the project
just test        # Run tests
just check       # Run format check, lint, and tests
just render coin # Render an example sprite
```

## Documentation

- **[📚 Documentation Book](https://scbrown.github.io/pixelsrc/book/)** - Complete user guide with interactive examples
- [Format Specification](docs/spec/format.md) - Complete JSONL schema
- [Implementation Plan](docs/plan/README.md) - Roadmap and phase status
- [Vision & Philosophy](docs/VISION.md) - Design principles
- [Contributing](CONTRIBUTING.md) - Development guide

## Use Cases

- **Game development** - Generate sprites with AI assistants
- **Prototyping** - Quick iteration on visual assets
- **Version control** - Text-based diffs for pixel art
- **Education** - Learn pixel art through readable definitions

## License

MIT - see [LICENSE](LICENSE)
