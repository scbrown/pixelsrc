# The Sprite Artist

You create **polished, production-ready sprites**. Every pixel matters. You want semantic organization, reusable palettes, and clean composition.

## Your Workflow

1. Design a thoughtful palette with semantic tokens
2. Build sprites using meaningful color names
3. Create variants for different states or colors
4. Compose complex scenes from layered sprites
5. Export at the right resolution

## Semantic Palettes

The foundation of good sprite art is a well-designed palette:

```json
{"type": "palette", "name": "character", "colors": {
  "{_}": "#00000000",
  "{outline}": "#1a1a2e",
  "{skin}": "#ffd5c8",
  "{skin_shadow}": "#e8b4a0",
  "{hair}": "#4a2c2a",
  "{hair_highlight}": "#6b4444",
  "{eye}": "#2d4059",
  "{cloth}": "#16537e",
  "{cloth_shadow}": "#0d3a5c"
}}
```

Why semantic tokens matter:
- **Readability**: `{skin_shadow}` is clearer than `#e8b4a0`
- **Maintainability**: Change a color once, update everywhere
- **Reusability**: Share palettes across multiple sprites
- **AI-friendly**: LLMs can reason about `{outline}` reliably

## Building Sprites

With your palette ready, create sprites:

```json
{"type": "sprite", "name": "hero_idle", "palette": "character", "grid": [
  "{_}{_}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair_highlight}{hair}{hair}{hair}{_}",
  "{_}{outline}{skin}{skin}{skin}{outline}{_}",
  "{_}{skin}{eye}{skin}{eye}{skin}{_}",
  "{_}{skin}{skin}{skin_shadow}{skin}{skin}{_}",
  "{_}{_}{cloth}{cloth}{cloth}{_}{_}",
  "{_}{cloth}{cloth_shadow}{cloth}{cloth_shadow}{cloth}{_}",
  "{_}{_}{skin}{_}{skin}{_}{_}"
]}
```

## Variants

Create variations without duplicating entire sprites:

```json
{"type": "variant", "name": "hero_damaged", "base": "hero_idle", "palette": {
  "{skin}": "#ffb8a8",
  "{skin_shadow}": "#d89080"
}}
```

The variant inherits everything from `hero_idle` but overrides the skin colors to show damage.

Common variant use cases:
- Damage/heal states
- Different color schemes (red team vs blue team)
- Seasonal variations
- Lighting conditions (day/night)

## Compositions

Layer sprites to create complex scenes:

```json
{"type": "composition", "name": "scene", "size": [64, 64], "layers": [
  {"sprite": "background_grass", "x": 0, "y": 0},
  {"sprite": "tree", "x": 8, "y": 24},
  {"sprite": "hero_idle", "x": 28, "y": 48},
  {"sprite": "ui_healthbar", "x": 2, "y": 2}
]}
```

Layers render bottom-to-top, so the first layer is the background.

## Organizing Your Files

For larger projects, split files by purpose:

```
assets/
├── palettes/
│   ├── characters.pxl
│   └── environment.pxl
├── sprites/
│   ├── hero.pxl
│   ├── enemies.pxl
│   └── items.pxl
└── scenes/
    └── level1.pxl
```

Use includes to reference palettes:

```json
{"type": "include", "path": "../palettes/characters.pxl"}
{"type": "sprite", "name": "hero", "palette": "character", "grid": [...]}
```

## Export Tips

### High-Quality PNG

```bash
pxl render hero.pxl -o hero.png --scale 1
```

Use `--scale 1` for pixel-perfect output, then scale in your game engine.

### Preview Before Export

```bash
pxl show hero.pxl --name hero_idle
```

### Validation

Before committing:

```bash
pxl validate hero.pxl --strict
```

Strict mode catches warnings that lenient mode ignores.

## Example: Complete Character

```json
{"type": "palette", "name": "knight", "colors": {
  "{_}": "#00000000",
  "{outline}": "#1a1a2e",
  "{armor}": "#7f8c8d",
  "{armor_light}": "#95a5a6",
  "{armor_dark}": "#5d6d7e",
  "{visor}": "#2c3e50",
  "{plume}": "#c0392b",
  "{plume_dark}": "#922b21"
}}
{"type": "sprite", "name": "knight_idle", "palette": "knight", "grid": [
  "{_}{_}{plume}{plume}{plume}{_}{_}{_}",
  "{_}{plume_dark}{plume}{plume}{plume}{plume}{_}{_}",
  "{_}{outline}{armor}{armor}{armor}{outline}{_}{_}",
  "{outline}{armor_light}{armor}{visor}{armor}{armor_light}{outline}{_}",
  "{outline}{armor}{armor_dark}{armor_dark}{armor_dark}{armor}{outline}{_}",
  "{_}{outline}{armor}{armor}{armor}{outline}{_}{_}",
  "{_}{armor}{armor_dark}{armor}{armor_dark}{armor}{_}{_}",
  "{_}{outline}{armor}{_}{armor}{outline}{_}{_}"
]}
{"type": "variant", "name": "knight_gold", "base": "knight_idle", "palette": {
  "{armor}": "#f39c12",
  "{armor_light}": "#f7dc6f",
  "{armor_dark}": "#d4ac0d"
}}
```

### Try It

Edit the knight's palette to create your own color scheme:

<div class="pixelsrc-demo" data-pixelsrc-demo>
  <textarea id="sprite-artist-demo">{"type": "palette", "name": "knight", "colors": {"{_}": "#00000000", "{outline}": "#1a1a2e", "{armor}": "#7f8c8d", "{armor_light}": "#95a5a6", "{armor_dark}": "#5d6d7e", "{visor}": "#2c3e50", "{plume}": "#c0392b", "{plume_dark}": "#922b21"}}
{"type": "sprite", "name": "knight_idle", "palette": "knight", "grid": ["{_}{_}{plume}{plume}{plume}{_}{_}{_}", "{_}{plume_dark}{plume}{plume}{plume}{plume}{_}{_}", "{_}{outline}{armor}{armor}{armor}{outline}{_}{_}", "{outline}{armor_light}{armor}{visor}{armor}{armor_light}{outline}{_}", "{outline}{armor}{armor_dark}{armor_dark}{armor_dark}{armor}{outline}{_}", "{_}{outline}{armor}{armor}{armor}{outline}{_}{_}", "{_}{armor}{armor_dark}{armor}{armor_dark}{armor}{_}{_}", "{_}{outline}{armor}{_}{armor}{outline}{_}{_}"]}</textarea>
  <button onclick="pixelsrcDemo.renderFromTextarea('sprite-artist-demo', 'sprite-artist-demo-preview')">Try it</button>
  <div class="preview" id="sprite-artist-demo-preview"></div>
</div>

Try changing `{armor}` to `#FFD700` (gold) and `{plume}` to `#4169E1` (blue) for a royal knight.

This gives you a silver knight, a gold variant, and a reusable palette for additional knight sprites.
