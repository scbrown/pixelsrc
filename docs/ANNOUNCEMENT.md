# Introducing TTP: Text To Pixel

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

TTP is a pixel art format built from the ground up for LLM generation.

```jsonl
{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{shine}": "#FFFACD", "{shadow}": "#B8860B"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "grid": [
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}",
  "{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{shadow}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{shadow}{shadow}{gold}",
  "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}",
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}"
]}
```

Then:
```bash
pxl render coin.jsonl -o coin.png
```

That's it. Text in, pixels out.

---

## Why It Works for GenAI

**1. Semantic tokens, not coordinates**

Instead of "place #FFD700 at (2, 0)", write `{gold}`. The token name carries meaning. An LLM can "read" the sprite as it generates:

```
{_}{_}{gold}{gold}{_}{_}    ← clearly the top of something round
{_}{gold}{shine}{shine}...  ← highlight on upper-left, makes sense
```

**2. No spatial reasoning required**

Write rows top-to-bottom, left-to-right. Sequential text generation. No coordinate math. No "where was I?" confusion.

**3. Streaming-friendly**

JSONL format means each line is independently valid JSON. If line 5 has an error, lines 1-4 still work. Parse as you generate.

**4. Palette-first thinking**

Define colors once with meaningful names. Reference them everywhere. This is how artists think: "the skin color" not "that hex code I used earlier."

---

## What You Can Do With It

**Rapid prototyping**
```
You: "Generate a 16x16 hero sprite, idle pose"
AI: [outputs TTP]
You: pxl render hero.jsonl -o hero.png
You: "Add a sword"
AI: [modifies TTP]
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
- "{_}{skin}{skin}{_}"
+ "{_}{skin}{armor}{_}"
```

---

## Design Principles

1. **Expressability over brevity** — `{skin}` beats `S`. Readable beats compact.

2. **Implicit over explicit** — Position comes from grid location, not coordinates.

3. **Don't reinvent the wheel** — TTP defines sprites. ImageMagick renders them.

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

TTP is a thin semantic layer. A bridge between text and pixels.

---

## Get Started

```bash
# Install
cargo install pxl

# Render a sprite
pxl render sprite.jsonl -o sprite.png

# List built-in palettes
pxl palettes list

# Use a classic palette
# {"type": "sprite", "palette": "@gameboy", ...}
```

---

## The Pitch in One Sentence

**TTP lets you describe pixel art in text and get actual pixels out — designed so AI can do it reliably.**

---

*TTP is open source. Contributions welcome.*
