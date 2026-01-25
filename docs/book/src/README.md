# Introduction

**Pixelsrc** is a GenAI-native pixel art format. It's text-based JSONL that you (or an AI) generate, then the `pxl` command converts to PNG, GIF, spritesheets, and more.

## Why Pixelsrc?

Traditional pixel art tools produce binary files that are difficult for AI to generate and humans to version control. Pixelsrc takes a different approach:

- **Pure text** - No binary data. Every sprite is human-readable JSONL
- **Semantic tokens** - Use names like `{skin}` and `{hair}` instead of hex coordinates
- **Streaming-friendly** - JSONL format means each line is self-contained
- **Lenient by default** - Small mistakes get corrected automatically
- **AI-first design** - Every feature is designed for reliable AI generation

## Quick Example

Here's a simple coin sprite in Pixelsrc format:

```json5
{"type": "palette", "name": "coin", "colors": {"_": "#0000", "gold": "#FFD700", "shine": "#FFFACD"}}
{"type": "sprite", "name": "coin", "size": [4, 4], "palette": "coin", "regions": {
  "gold": {"union": [{"rect": [1, 0, 2, 1]}, {"rect": [0, 1, 4, 2]}, {"rect": [1, 3, 2, 1]}], "z": 0},
  "shine": {"points": [[1, 1]], "z": 1}
}}
```

Render it with:

```bash
pxl render coin.pxl -o coin.png
```

## What You Can Do

With Pixelsrc, you can:

- **Create sprites** using semantic color tokens
- **Build animations** from frame sequences
- **Compose scenes** by layering sprites
- **Generate assets** with AI assistance
- **Export** to PNG, GIF, spritesheets, and game engine formats
- **Validate** your files for common mistakes
- **Format** for consistent, readable code

## Who Is This For?

Pixelsrc is designed for:

- **AI systems** (Claude, GPT, etc.) generating game assets
- **Indie developers** wanting quick prototyping
- **Pixel artists** wanting text-based version control
- **Roguelike developers** needing procedural assets

## Getting Started

Head to [Installation](getting-started/installation.md) to set up Pixelsrc, then try the [Quick Start](getting-started/quick-start.md) guide.

If you prefer learning by example, check out the [Persona Guides](personas/sketcher.md) for workflow examples tailored to your use case.
