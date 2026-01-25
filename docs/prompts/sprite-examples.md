# Sprite Generation Examples

Example prompts and their expected outputs for generating pixel art with GenAI.

All examples use the regions format with shape primitives.

---

## Simple Item: Heart

**Prompt:**
> Create a 7x6 pixel art heart icon with a red base color and pink highlight

**Output:**
```json
{"type": "sprite", "name": "heart", "size": [7, 6], "palette": {"_": "#00000000", "r": "#FF0000", "p": "#FF6B6B"}, "regions": {
  "r": {"union": [{"rect": [1, 0, 2, 1]}, {"rect": [4, 0, 2, 1]}, {"rect": [0, 1, 7, 2]}, {"rect": [1, 3, 5, 1]}, {"rect": [2, 4, 3, 1]}, {"points": [[3, 5]]}], "z": 0},
  "p": {"points": [[1, 1], [4, 1]], "z": 1}
}}
```

---

## Collectible: Coin

**Prompt:**
> Create a 8x8 animated spinning coin with 4 frames showing rotation

**Output:**
```json
{"type": "palette", "name": "coin", "colors": {"_": "#00000000", "g": "#FFD700", "y": "#FFEC8B", "o": "#B8860B", "d": "#8B6914"}}

{"type": "sprite", "name": "coin_1", "size": [8, 8], "palette": "coin", "regions": {
  "g": {"union": [{"rect": [2, 0, 4, 1]}, {"points": [[1, 1], [6, 1]]}, {"points": [[0, 2], [7, 2], [0, 3], [7, 3], [0, 4], [7, 4], [0, 5], [7, 5]]}, {"points": [[1, 6], [6, 6]]}], "z": 0},
  "y": {"union": [{"rect": [2, 1, 4, 1]}, {"rect": [1, 2, 2, 1]}, {"rect": [5, 2, 2, 1]}, {"points": [[1, 3], [4, 3], [5, 3]]}, {"points": [[1, 4], [4, 4], [5, 4]]}, {"rect": [1, 5, 2, 1]}, {"rect": [5, 5, 2, 1]}], "z": 1},
  "o": {"rect": [2, 6, 4, 1], "z": 0},
  "d": {"rect": [2, 7, 4, 1], "z": 0}
}}

{"type": "sprite", "name": "coin_2", "size": [8, 8], "palette": "coin", "regions": {
  "g": {"union": [{"rect": [3, 0, 2, 1]}, {"points": [[2, 1], [5, 1]]}, {"points": [[1, 2], [6, 2], [1, 3], [6, 3], [1, 4], [6, 4], [1, 5], [6, 5]]}, {"points": [[2, 6], [5, 6]]}], "z": 0},
  "y": {"union": [{"rect": [3, 1, 2, 1]}, {"rect": [2, 2, 4, 1]}, {"rect": [2, 5, 4, 1]}], "z": 1},
  "o": {"rect": [3, 6, 2, 1], "z": 0},
  "d": {"rect": [3, 7, 2, 1], "z": 0}
}}

{"type": "sprite", "name": "coin_3", "size": [8, 8], "palette": "coin", "regions": {
  "g": {"union": [{"rect": [3, 0, 2, 1]}, {"rect": [2, 2, 4, 2]}, {"rect": [2, 5, 4, 1]}], "z": 0},
  "y": {"union": [{"rect": [3, 1, 2, 1]}, {"rect": [3, 5, 2, 1]}], "z": 1},
  "o": {"rect": [3, 6, 2, 1], "z": 0},
  "d": {"rect": [3, 7, 2, 1], "z": 0}
}}

{"type": "sprite", "name": "coin_4", "size": [8, 8], "palette": "coin", "regions": {
  "g": {"union": [{"rect": [3, 0, 2, 1]}, {"points": [[2, 1], [5, 1]]}, {"points": [[1, 2], [6, 2], [1, 3], [6, 3], [1, 4], [6, 4], [1, 5], [6, 5]]}, {"points": [[2, 6], [5, 6]]}], "z": 0},
  "y": {"union": [{"rect": [3, 1, 2, 1]}, {"rect": [2, 2, 4, 1]}, {"rect": [2, 5, 4, 1]}], "z": 1},
  "o": {"rect": [3, 6, 2, 1], "z": 0},
  "d": {"rect": [3, 7, 2, 1], "z": 0}
}}

{"type": "animation", "name": "coin_spin", "frames": ["coin_1", "coin_2", "coin_3", "coin_4"], "duration": 150, "loop": true}
```

---

## Character: Hero

**Prompt:**
> Create a 16x16 pixel art character with brown hair, skin tone face, blue shirt, and dark pants. Include an outline for definition.

**Output:**
```json
{"type": "palette", "name": "hero", "colors": {"_": "#00000000", "skin": "#FFCC99", "hair": "#8B4513", "eye": "#000000", "shirt": "#4169E1", "pants": "#1E3A5F", "shoes": "#000000", "outline": "#2C1810"}}

{"type": "sprite", "name": "hero_idle", "size": [16, 16], "palette": "hero", "regions": {
  "outline": {"union": [{"rect": [6, 1, 4, 1]}, {"points": [[5, 2], [10, 2]]}, {"rect": [4, 3, 1, 6]}, {"rect": [11, 3, 1, 6]}, {"points": [[5, 8], [10, 8]]}, {"rect": [4, 9, 1, 4]}, {"rect": [11, 9, 1, 4]}, {"points": [[3, 10], [12, 10], [3, 11], [12, 11]]}, {"rect": [5, 12, 1, 3]}, {"rect": [10, 12, 1, 3]}], "z": 0},
  "hair": {"rect": [5, 2, 6, 3], "z": 1},
  "skin": {"union": [{"rect": [5, 5, 6, 3]}, {"rect": [6, 8, 4, 1]}, {"points": [[4, 10], [11, 10], [4, 11], [11, 11]]}], "z": 1},
  "eye": {"points": [[6, 6], [9, 6]], "z": 2},
  "shirt": {"union": [{"rect": [5, 9, 6, 3]}, {"rect": [6, 12, 4, 1]}], "z": 1},
  "pants": {"union": [{"rect": [6, 13, 4, 1]}, {"points": [[6, 14], [9, 14]]}], "z": 1},
  "shoes": {"points": [[5, 15], [6, 15], [9, 15], [10, 15]], "z": 0}
}}
```

---

## Weapon: Sword

**Prompt:**
> Create a 16x16 pixel art sword with a silver blade, golden crossguard, and brown leather handle

**Output:**
```json
{"type": "sprite", "name": "sword", "size": [16, 16], "palette": {"_": "#00000000", "blade": "#C0C0C0", "edge": "#E8E8E8", "guard": "#FFD700", "handle": "#8B4513", "pommel": "#DAA520"}, "regions": {
  "edge": {"points": [[14, 0], [13, 1], [12, 2], [11, 3], [10, 4], [9, 5], [8, 6], [7, 7], [6, 8], [5, 9]], "z": 0},
  "blade": {"union": [{"points": [[14, 1]]}, {"rect": [13, 2, 2, 1]}, {"rect": [12, 3, 2, 1]}, {"rect": [11, 4, 2, 1]}, {"rect": [10, 5, 2, 1]}, {"rect": [9, 6, 2, 1]}, {"rect": [8, 7, 2, 1]}, {"rect": [7, 8, 2, 1]}, {"rect": [6, 9, 2, 1]}, {"rect": [5, 10, 2, 1]}], "z": 0},
  "guard": {"union": [{"rect": [3, 10, 2, 1]}, {"rect": [7, 10, 2, 1]}], "z": 1},
  "handle": {"rect": [5, 11, 2, 4], "z": 0},
  "pommel": {"rect": [5, 15, 2, 1], "z": 0}
}}
```

---

## Tile: Grass

**Prompt:**
> Create a 16x16 seamless grass tile for a top-down RPG with varied green shades

**Output:**
```json
{"type": "sprite", "name": "grass", "size": [16, 16], "palette": {"g1": "#228B22", "g2": "#32CD32", "g3": "#3CB371", "d": "#1E5F1E"}, "regions": {
  "g1": {"rect": [0, 0, 16, 16], "z": 0},
  "g2": {"points": [[1, 0], [6, 0], [10, 0], [14, 0], [2, 1], [5, 1], [12, 1], [0, 2], [3, 2], [9, 2], [6, 3], [11, 3], [15, 3], [2, 4], [14, 4], [2, 5], [4, 5], [12, 5], [7, 6], [9, 6], [14, 7], [1, 8], [10, 8], [3, 9], [6, 9], [12, 9], [8, 10], [15, 10], [11, 11], [13, 11], [2, 12], [5, 12], [9, 12], [14, 13], [3, 14], [7, 14], [10, 15], [12, 15]], "z": 1},
  "g3": {"points": [[4, 0], [11, 0], [15, 1], [2, 1], [8, 1], [7, 2], [13, 2], [4, 3], [1, 4], [10, 4], [6, 5], [0, 6], [13, 6], [3, 7], [11, 7], [7, 8], [14, 9], [2, 10], [10, 10], [4, 11], [7, 12], [1, 13], [12, 13], [5, 14], [9, 14], [15, 14]], "z": 1},
  "d": {"points": [[8, 4]], "z": 1}
}}
```

---

## Animation: Walk Cycle

**Prompt:**
> Create a 16x16 character walk animation with 4 frames showing left foot forward, standing, right foot forward, standing

**Output:**
```json
{"type": "palette", "name": "walker", "colors": {"_": "#00000000", "h": "#654321", "s": "#FFCC99", "b": "#4169E1", "p": "#2F4F4F", "f": "#1C1C1C"}}

{"type": "sprite", "name": "walk_1", "size": [16, 12], "palette": "walker", "regions": {
  "h": {"union": [{"rect": [5, 0, 6, 1]}, {"rect": [4, 1, 8, 1]}], "z": 0},
  "s": {"rect": [4, 2, 8, 2], "z": 0},
  "b": {"union": [{"rect": [5, 4, 6, 1]}, {"rect": [4, 5, 8, 2]}, {"rect": [5, 7, 6, 1]}], "z": 0},
  "p": {"union": [{"rect": [5, 8, 2, 1]}, {"rect": [9, 8, 2, 1]}, {"rect": [4, 9, 2, 1]}, {"rect": [10, 9, 2, 1]}, {"rect": [3, 10, 2, 1]}, {"rect": [11, 10, 2, 1]}], "z": 0},
  "f": {"union": [{"rect": [3, 11, 2, 1]}, {"rect": [11, 11, 2, 1]}], "z": 0}
}}

{"type": "sprite", "name": "walk_2", "size": [16, 12], "palette": "walker", "regions": {
  "h": {"union": [{"rect": [5, 0, 6, 1]}, {"rect": [4, 1, 8, 1]}], "z": 0},
  "s": {"rect": [4, 2, 8, 2], "z": 0},
  "b": {"union": [{"rect": [5, 4, 6, 1]}, {"rect": [4, 5, 8, 2]}, {"rect": [5, 7, 6, 1]}], "z": 0},
  "p": {"union": [{"rect": [5, 8, 2, 1]}, {"rect": [9, 8, 2, 1]}, {"rect": [5, 9, 2, 1]}, {"rect": [9, 9, 2, 1]}, {"rect": [5, 10, 2, 1]}, {"rect": [9, 10, 2, 1]}], "z": 0},
  "f": {"union": [{"rect": [5, 11, 2, 1]}, {"rect": [9, 11, 2, 1]}], "z": 0}
}}

{"type": "sprite", "name": "walk_3", "size": [16, 12], "palette": "walker", "regions": {
  "h": {"union": [{"rect": [5, 0, 6, 1]}, {"rect": [4, 1, 8, 1]}], "z": 0},
  "s": {"rect": [4, 2, 8, 2], "z": 0},
  "b": {"union": [{"rect": [5, 4, 6, 1]}, {"rect": [4, 5, 8, 2]}, {"rect": [5, 7, 6, 1]}], "z": 0},
  "p": {"union": [{"rect": [5, 8, 2, 1]}, {"rect": [9, 8, 2, 1]}, {"rect": [6, 9, 4, 1]}, {"rect": [5, 10, 2, 1]}, {"rect": [9, 10, 2, 1]}], "z": 0},
  "f": {"union": [{"rect": [5, 11, 2, 1]}, {"rect": [9, 11, 2, 1]}], "z": 0}
}}

{"type": "sprite", "name": "walk_4", "size": [16, 12], "palette": "walker", "regions": {
  "h": {"union": [{"rect": [5, 0, 6, 1]}, {"rect": [4, 1, 8, 1]}], "z": 0},
  "s": {"rect": [4, 2, 8, 2], "z": 0},
  "b": {"union": [{"rect": [5, 4, 6, 1]}, {"rect": [4, 5, 8, 2]}, {"rect": [5, 7, 6, 1]}], "z": 0},
  "p": {"union": [{"rect": [5, 8, 2, 1]}, {"rect": [9, 8, 2, 1]}, {"rect": [5, 9, 2, 1]}, {"rect": [9, 9, 2, 1]}, {"rect": [5, 10, 2, 1]}, {"rect": [9, 10, 2, 1]}], "z": 0},
  "f": {"union": [{"rect": [5, 11, 2, 1]}, {"rect": [9, 11, 2, 1]}], "z": 0}
}}

{"type": "animation", "name": "walk_cycle", "frames": ["walk_1", "walk_2", "walk_3", "walk_4"], "duration": 150, "loop": true}
```

---

## Scene: Tiled Composition

**Prompt:**
> Create a 32x32 scene using 8x8 tiles. Make sky and grass tiles, then compose them into a simple landscape with sky on top and grass on bottom.

**Output:**
```jsonl
{"type":"palette","name":"nature","colors":{"sky":"#87CEEB","cloud":"#FFFFFF","grass":"#228B22","grass_dark":"#1E5F1E"}}
{"type":"sprite","name":"sky_tile","size":[8,8],"palette":"nature","regions":{"sky":{"rect":[0,0,8,8],"z":0},"cloud":{"union":[{"rect":[2,1,2,1]},{"rect":[1,2,4,1]},{"rect":[2,3,2,1]}],"z":1}}}
{"type":"sprite","name":"grass_tile","size":[8,8],"palette":"nature","regions":{"grass":{"rect":[0,0,8,8],"z":0},"grass_dark":{"points":[[1,0],[5,0],[3,1],[7,1],[0,2],[4,3],[1,4],[6,4],[3,6],[5,6],[0,7],[7,7]],"z":1}}}
{"type":"composition","name":"landscape","size":[32,32],"cell_size":[8,8],"sprites":{"S":"sky_tile","G":"grass_tile"},"layers":[{"map":["SSSS","SSSS","GGGG","GGGG"]}]}
```

---

## Variant: Color Swap

**Prompt:**
> Create a hero sprite, then make a variant with red hair for an "enemy" version

**Output:**
```jsonl
{"type":"sprite","name":"hero","size":[4,5],"palette":{"_":"#00000000","skin":"#FFCC99","hair":"#8B4513","shirt":"#4169E1"},"regions":{"hair":{"union":[{"rect":[1,0,2,1]},{"points":[[0,1],[3,1]]}],"z":0},"skin":{"union":[{"rect":[1,1,2,1]},{"rect":[1,2,2,1]}],"z":1},"shirt":{"union":[{"rect":[0,3,4,1]},{"points":[[0,4],[3,4]]}],"z":0}}}
{"type":"variant","name":"enemy","base":"hero","palette":{"hair":"#FF0000","shirt":"#8B0000"}}
```

---

## Tips for Writing Prompts

1. **Be specific about size**: "16x16", "32x32", "8x8"
2. **Name colors explicitly**: "silver blade", "golden hilt", "brown handle"
3. **Reference real games**: "like Zelda items", "SNES-era style"
4. **Request semantic tokens**: "use descriptive token names like skin and hair"
5. **Ask for palettes separately**: "first create a palette, then the sprite"
6. **Request regions format**: "use rect and union shapes for regions"

## Formatting Generated Output

Use `pxl fmt` to ensure consistent formatting:

```bash
# Format a generated file
pxl fmt generated.pxl

# Check if formatting is needed
pxl fmt --check generated.pxl
```
