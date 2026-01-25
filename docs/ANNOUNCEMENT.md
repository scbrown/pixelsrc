# Introducing Pixelsrc

**The first pixel art format designed for GenAI.**

---

## The Problem

Ask any LLM to create pixel art and watch it struggle:

- "Place a red pixel at coordinates (7, 3)" — spatial reasoning is hard
- ASCII art approximations that render differently everywhere
- Color codes scattered through output with no semantic meaning
- One mistake invalidates the entire sprite definition

Text-to-image models can generate pixel art, but you can't edit the output. You get what you get.

Existing text-based formats (XPM, PPM) were designed for storage, not generation. They assume you already have the image.

**There's no good way to go from "describe a sprite" to "usable game asset" through text alone.**

---

## The Solution

Pixelsrc is a pixel art format built from the ground up for LLM generation.

```jsonl
{"type": "palette", "name": "coin", "colors": {"_": "#00000000", "gold": "#FFD700", "shine": "#FFFACD", "shadow": "#B8860B"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "regions": {
  "gold": {"union": [{"rect": [2, 0, 4, 1]}, {"rect": [1, 1, 6, 1]}, {"rect": [0, 2, 8, 4]}, {"rect": [1, 6, 6, 1]}, {"rect": [2, 7, 4, 1]}], "z": 0},
  "shine": {"union": [{"rect": [2, 1, 2, 1]}, {"points": [[1, 2], [1, 3]]}], "z": 1},
  "shadow": {"union": [{"points": [[6, 2], [6, 3], [6, 4]]}, {"points": [[5, 5], [6, 5]]}], "z": 1}
}}
```

Then:
```bash
pxl render coin.jsonl -o coin.png
```

That's it. Text in, pixels out.

---

## Why It Works for GenAI

**1. Semantic tokens, not coordinates**

Instead of "place #FFD700 at (2, 0)", name your colors `gold`, `shine`, `shadow`. The token names carry meaning, and shapes describe regions with simple geometry:

```json5
shine: { points: [[2, 1], [2, 2]] },  // highlight pixels
shadow: { points: [[6, 2], [6, 3]] }, // depth pixels
gold: { fill: "background" }           // everything else
```

**2. No spatial reasoning required**

Describe shapes geometrically. `rect: [2, 0, 4, 1]` is "4-pixel wide rectangle at the top". No pixel-by-pixel enumeration. No counting errors.

**3. Streaming-friendly**

JSONL format means each line is independently valid JSON. If line 5 has an error, lines 1-4 still work. Parse as you generate.

**4. Palette-first thinking**

Define colors once with meaningful names. Reference them everywhere. This is how artists think: "the skin color" not "that hex code I used earlier."

---

## What You Can Do With It

**Rapid prototyping**
```
You: "Generate a 16x16 hero sprite, idle pose"
AI: [outputs Pixelsrc]
You: pxl render hero.jsonl -o hero.png
You: "Add a sword"
AI: [modifies Pixelsrc]
```

**Consistent asset packs**
```jsonl
{"type": "palette", "name": "dungeon", "colors": {...}}
{"type": "sprite", "name": "floor", "palette": "dungeon", ...}
{"type": "sprite", "name": "wall", "palette": "dungeon", ...}
{"type": "sprite", "name": "door", "palette": "dungeon", ...}
```

Same palette = visual coherence across all assets.

**Animation**
```jsonl
{"type": "sprite", "name": "walk_1", ...}
{"type": "sprite", "name": "walk_2", ...}
{"type": "sprite", "name": "walk_3", ...}
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

```bash
pxl render walk.jsonl -o walk.gif --gif
```

**Version control**

It's text. Git diff shows exactly what changed:
```diff
- body: { rect: [4, 8, 8, 4] }
+ body: { rect: [4, 8, 10, 4] }  // made body wider
```

---

## Design Principles

1. **Expressability over brevity** — `skin` beats `S`. Readable beats compact.

2. **Shapes over pixels** — Describe regions geometrically, let the compiler render pixels.

3. **Don't reinvent the wheel** — Pixelsrc defines sprites. ImageMagick renders them.

4. **Streaming-native** — Each line stands alone. Parse incrementally.

5. **GenAI-first** — Every decision optimizes for LLM reliability.

---

## Who It's For

- **GenAI systems** generating game assets
- **Indie developers** prototyping quickly
- **Pixel artists** wanting text-based version control
- **Roguelike/retro developers** needing simple asset pipelines

---

## What It's Not

- Not an image editor
- Not a replacement for Aseprite
- Not a rendering engine

Pixelsrc is a thin semantic layer. A bridge between text and pixels.

---

## Get Started

```bash
# Install
cargo install pixelsrc

# Render a sprite
pxl render sprite.jsonl -o sprite.png

# List built-in palettes
pxl palettes list

# Use a classic palette
# {"type": "sprite", "palette": "@gameboy", ...}
```

---

## The Pitch in One Sentence

**Pixelsrc lets you describe pixel art in text and get actual pixels out — designed so AI can do it reliably.**

---

*Pixelsrc is open source. Contributions welcome.*
