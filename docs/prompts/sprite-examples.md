# Sprite Generation Examples

Example prompts and their expected outputs for generating pixel art with GenAI.

All examples use the recommended multi-line format for sprite grids.

---

## Simple Item: Heart

**Prompt:**
> Create a 7x6 pixel art heart icon with a red base color and pink highlight

**Output:**
```json
{"type": "sprite", "name": "heart", "palette": {"{_}": "#00000000", "{r}": "#FF0000", "{p}": "#FF6B6B"}, "grid": [
  "{_}{r}{r}{_}{r}{r}{_}",
  "{r}{p}{r}{r}{p}{r}{r}",
  "{r}{r}{r}{r}{r}{r}{r}",
  "{_}{r}{r}{r}{r}{r}{_}",
  "{_}{_}{r}{r}{r}{_}{_}",
  "{_}{_}{_}{r}{_}{_}{_}"
]}
```

---

## Collectible: Coin

**Prompt:**
> Create a 8x8 animated spinning coin with 4 frames showing rotation

**Output:**
```json
{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{g}": "#FFD700", "{y}": "#FFEC8B", "{o}": "#B8860B", "{d}": "#8B6914"}}

{"type": "sprite", "name": "coin_1", "palette": "coin", "grid": [
  "{_}{_}{g}{g}{g}{g}{_}{_}",
  "{_}{g}{y}{y}{y}{y}{g}{_}",
  "{g}{y}{y}{g}{g}{y}{y}{g}",
  "{g}{y}{g}{y}{y}{g}{y}{g}",
  "{g}{y}{g}{y}{y}{g}{y}{g}",
  "{g}{y}{y}{g}{g}{y}{y}{g}",
  "{_}{g}{o}{o}{o}{o}{g}{_}",
  "{_}{_}{d}{d}{d}{d}{_}{_}"
]}

{"type": "sprite", "name": "coin_2", "palette": "coin", "grid": [
  "{_}{_}{_}{g}{g}{_}{_}{_}",
  "{_}{_}{g}{y}{y}{g}{_}{_}",
  "{_}{g}{y}{y}{y}{y}{g}{_}",
  "{_}{g}{y}{g}{g}{y}{g}{_}",
  "{_}{g}{y}{g}{g}{y}{g}{_}",
  "{_}{g}{y}{y}{y}{y}{g}{_}",
  "{_}{_}{g}{o}{o}{g}{_}{_}",
  "{_}{_}{_}{d}{d}{_}{_}{_}"
]}

{"type": "sprite", "name": "coin_3", "palette": "coin", "grid": [
  "{_}{_}{_}{g}{g}{_}{_}{_}",
  "{_}{_}{_}{y}{y}{_}{_}{_}",
  "{_}{_}{g}{y}{y}{g}{_}{_}",
  "{_}{_}{g}{g}{g}{g}{_}{_}",
  "{_}{_}{g}{g}{g}{g}{_}{_}",
  "{_}{_}{g}{y}{y}{g}{_}{_}",
  "{_}{_}{_}{o}{o}{_}{_}{_}",
  "{_}{_}{_}{d}{d}{_}{_}{_}"
]}

{"type": "sprite", "name": "coin_4", "palette": "coin", "grid": [
  "{_}{_}{_}{g}{g}{_}{_}{_}",
  "{_}{_}{g}{y}{y}{g}{_}{_}",
  "{_}{g}{y}{y}{y}{y}{g}{_}",
  "{_}{g}{y}{g}{g}{y}{g}{_}",
  "{_}{g}{y}{g}{g}{y}{g}{_}",
  "{_}{g}{y}{y}{y}{y}{g}{_}",
  "{_}{_}{g}{o}{o}{g}{_}{_}",
  "{_}{_}{_}{d}{d}{_}{_}{_}"
]}

{"type": "animation", "name": "coin_spin", "frames": ["coin_1", "coin_2", "coin_3", "coin_4"], "duration": 150, "loop": true}
```

---

## Character: Hero

**Prompt:**
> Create a 16x16 pixel art character with brown hair, skin tone face, blue shirt, and dark pants. Include an outline for definition.

**Output:**
```json
{"type": "palette", "name": "hero", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{hair}": "#8B4513", "{eye}": "#000000", "{shirt}": "#4169E1", "{pants}": "#1E3A5F", "{shoes}": "#000000", "{outline}": "#2C1810"}}

{"type": "sprite", "name": "hero_idle", "palette": "hero", "grid": [
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{outline}{outline}{outline}{outline}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{outline}{hair}{hair}{hair}{hair}{outline}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{outline}{hair}{hair}{hair}{hair}{hair}{hair}{outline}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{outline}{hair}{hair}{hair}{hair}{hair}{hair}{outline}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{outline}{skin}{skin}{skin}{skin}{skin}{skin}{outline}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{outline}{skin}{eye}{skin}{skin}{eye}{skin}{outline}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{outline}{skin}{skin}{skin}{skin}{skin}{skin}{outline}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{outline}{skin}{skin}{skin}{skin}{outline}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{outline}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{outline}{_}{_}{_}{_}",
  "{_}{_}{_}{outline}{skin}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{skin}{outline}{_}{_}{_}",
  "{_}{_}{_}{outline}{skin}{outline}{shirt}{shirt}{shirt}{shirt}{outline}{skin}{outline}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{outline}{shirt}{shirt}{shirt}{shirt}{outline}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{outline}{pants}{pants}{pants}{pants}{outline}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{outline}{pants}{_}{_}{pants}{outline}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{shoes}{shoes}{_}{_}{shoes}{shoes}{_}{_}{_}{_}{_}"
]}
```

---

## Weapon: Sword

**Prompt:**
> Create a 16x16 pixel art sword with a silver blade, golden crossguard, and brown leather handle

**Output:**
```json
{"type": "sprite", "name": "sword", "palette": {"{_}": "#00000000", "{blade}": "#C0C0C0", "{edge}": "#E8E8E8", "{guard}": "#FFD700", "{handle}": "#8B4513", "{pommel}": "#DAA520"}, "grid": [
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{edge}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{edge}{blade}{blade}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{guard}{guard}{blade}{blade}{guard}{guard}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{handle}{handle}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{handle}{handle}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{handle}{handle}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{handle}{handle}{_}{_}{_}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{pommel}{pommel}{_}{_}{_}{_}{_}{_}{_}{_}{_}"
]}
```

---

## Tile: Grass

**Prompt:**
> Create a 16x16 seamless grass tile for a top-down RPG with varied green shades

**Output:**
```json
{"type": "sprite", "name": "grass", "palette": {"{g1}": "#228B22", "{g2}": "#32CD32", "{g3}": "#3CB371", "{d}": "#1E5F1E"}, "grid": [
  "{g1}{g2}{g1}{g1}{g3}{g1}{g2}{g1}{g1}{g1}{g2}{g3}{g1}{g1}{g2}{g1}",
  "{g1}{g1}{g3}{g1}{g1}{g2}{g1}{g1}{g3}{g1}{g1}{g1}{g2}{g1}{g1}{g3}",
  "{g2}{g1}{g1}{g2}{g1}{g1}{g1}{g3}{g1}{g2}{g1}{g1}{g1}{g3}{g1}{g1}",
  "{g1}{g1}{g1}{g1}{g3}{g1}{g2}{g1}{g1}{g1}{g1}{g2}{g1}{g1}{g1}{g2}",
  "{g1}{g3}{g1}{g1}{g1}{g1}{g1}{g1}{d}{g1}{g3}{g1}{g1}{g1}{g2}{g1}",
  "{g1}{g1}{g2}{g1}{g2}{g1}{g3}{g1}{g1}{g1}{g1}{g1}{g2}{g1}{g1}{g1}",
  "{g3}{g1}{g1}{g1}{g1}{g1}{g1}{g2}{g1}{g2}{g1}{g1}{g1}{g3}{g1}{g1}",
  "{g1}{g1}{g1}{g3}{g1}{g2}{g1}{g1}{g1}{g1}{g1}{g3}{g1}{g1}{g2}{g1}",
  "{g1}{g2}{g1}{g1}{g1}{g1}{g1}{g3}{g1}{g1}{g2}{g1}{g1}{g1}{g1}{g1}",
  "{g1}{g1}{g1}{g2}{g1}{g1}{g2}{g1}{g1}{g1}{g1}{g1}{g2}{g1}{g3}{g1}",
  "{g1}{g1}{g3}{g1}{g1}{g1}{g1}{g1}{g2}{g1}{g3}{g1}{g1}{g1}{g1}{g2}",
  "{g2}{g1}{g1}{g1}{g3}{g1}{g1}{g1}{g1}{g1}{g1}{g2}{g1}{g2}{g1}{g1}",
  "{g1}{g1}{g2}{g1}{g1}{g2}{g1}{g3}{g1}{g2}{g1}{g1}{g1}{g1}{g1}{g1}",
  "{g1}{g3}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g3}{g1}{g2}{g1}",
  "{g1}{g1}{g1}{g2}{g1}{g3}{g1}{g2}{g1}{g3}{g1}{g1}{g1}{g1}{g1}{g3}",
  "{g2}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g1}{g2}{g1}{g2}{g1}{g1}{g1}"
]}
```

---

## Animation: Walk Cycle

**Prompt:**
> Create a 16x16 character walk animation with 4 frames showing left foot forward, standing, right foot forward, standing

**Output:**
```json
{"type": "palette", "name": "walker", "colors": {"{_}": "#00000000", "{h}": "#654321", "{s}": "#FFCC99", "{b}": "#4169E1", "{p}": "#2F4F4F", "{f}": "#1C1C1C"}}

{"type": "sprite", "name": "walk_1", "palette": "walker", "grid": [
  "{_}{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{p}{p}{_}{_}{_}{_}{p}{p}{_}{_}{_}{_}",
  "{_}{_}{_}{p}{p}{_}{_}{_}{_}{_}{_}{p}{p}{_}{_}{_}",
  "{_}{_}{_}{f}{f}{_}{_}{_}{_}{_}{_}{f}{f}{_}{_}{_}"
]}

{"type": "sprite", "name": "walk_2", "palette": "walker", "grid": [
  "{_}{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{f}{f}{_}{_}{f}{f}{_}{_}{_}{_}{_}"
]}

{"type": "sprite", "name": "walk_3", "palette": "walker", "grid": [
  "{_}{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{_}{p}{p}{p}{p}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{f}{f}{_}{_}{f}{f}{_}{_}{_}{_}{_}"
]}

{"type": "sprite", "name": "walk_4", "palette": "walker", "grid": [
  "{_}{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{h}{h}{h}{h}{h}{h}{h}{h}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{s}{s}{s}{s}{s}{s}{s}{s}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{b}{b}{b}{b}{b}{b}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{p}{p}{_}{_}{p}{p}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{_}{_}{f}{f}{_}{_}{f}{f}{_}{_}{_}{_}{_}"
]}

{"type": "animation", "name": "walk_cycle", "frames": ["walk_1", "walk_2", "walk_3", "walk_4"], "duration": 150, "loop": true}
```

---

## Tips for Writing Prompts

1. **Be specific about size**: "16x16", "32x32", "8x8"
2. **Name colors explicitly**: "silver blade", "golden hilt", "brown handle"
3. **Reference real games**: "like Zelda items", "SNES-era style"
4. **Request semantic tokens**: "use descriptive token names like {skin} and {hair}"
5. **Ask for palettes separately**: "first create a palette, then the sprite"
6. **Request multi-line format**: "output grids with one row per line for readability"

## Formatting Generated Output

Use `pxl fmt` to ensure consistent formatting:

```bash
# Format a generated file
pxl fmt generated.pxl

# Check if formatting is needed
pxl fmt --check generated.pxl
```
