# TTP (Text To Pixel) - Vision

## Mission

Create a GenAI-native pixel art format that prioritizes readability, expressability, and reliability over complexity.

## Core Tenets

1. **Expressability over brevity** - Multi-char tokens (`{skin}`, `{hair}`) over cryptic single chars. Code should read like prose.

2. **Implicit over explicit** - Grid position defines location; no coordinate systems. This sidesteps GenAI's weakness with spatial reasoning.

3. **Don't reinvent the wheel** - Use proven libraries (`image` crate for rendering, `serde` for parsing). TTP is a thin semantic layer, not a rendering engine.

4. **Two-way doors** - Keep decisions reversible where possible. Start with JSON syntax, but design for future flexibility.

5. **GenAI-first** - Every design choice should make LLM generation more reliable. If a human finds it readable, an LLM can generate it.

6. **Streaming-native** - JSONL format enables real-time parsing as GenAI generates. Each line is self-contained and immediately usable.

7. **Lenient by default** - Fill gaps, warn, continue. When GenAI makes small mistakes, keep going. Strict mode available for CI/validation.

## What TTP Is

- A semantic, human-readable format for defining pixel art
- A thin layer that compiles to images via existing tools
- Designed for GenAI to generate reliably
- A bridge between text and pixels

## What TTP Is Not

- A full image editor
- A replacement for Aseprite/Photoshop
- A binary format
- A rendering engine

## Target Users

- **GenAI systems** (Claude, GPT, etc.) generating game assets
- **Indie game developers** wanting quick prototyping
- **Pixel artists** wanting text-based version control
- **Roguelike/retro game developers**

## Competitive Context

See [DIFFERENTIATOR.md](./DIFFERENTIATOR.md) for full competitive analysis.

**Key insight**: No existing text-based *source* format is designed for AI-generatable, human-readable pixel art. Existing tools are either:
- Visual editors with text *exports* (REXPaint, Aseprite)
- Hex-based fantasy console formats (PICO-8, TIC-80)
- AI image generators that skip text entirely (PixelLab, diffusion models)

TTP fills the gap: a **source format** (not export), **semantic** (not hex), **GenAI-native** (not visual-first).

## Quality Target

**REXPaint-level output**. REXPaint is the gold standard for roguelike/ASCII art tooling. A compiled `.ttp` file should produce output indistinguishable from what a skilled REXPaint artist would create.

## Success Criteria

TTP succeeds when:
- An LLM can generate valid `.ttp` files on the first attempt, consistently
- A human can read a `.ttp` file and understand the sprite without rendering it
- Git diffs of sprite changes are meaningful and reviewable
- The toolchain stays simple and composable
