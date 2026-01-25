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

```json5
{
  type: "palette",
  name: "character",
  colors: {
    _: "transparent",
    outline: "#1A1A2E",
    skin: "#FFD5C8",
    skin_shadow: "#E8B4A0",
    hair: "#4A2C2A",
    hair_highlight: "#6B4444",
    eye: "#2D4059",
    cloth: "#16537E",
    cloth_shadow: "#0D3A5C",
  },
}
```

Why semantic tokens matter:
- **Readability**: `skin_shadow` is clearer than `#E8B4A0`
- **Maintainability**: Change a color once, update everywhere
- **Reusability**: Share palettes across multiple sprites
- **AI-friendly**: LLMs can reason about `outline` reliably

## Building Sprites

With your palette ready, create sprites using structured regions:

```json5
{
  type: "sprite",
  name: "hero_idle",
  size: [7, 8],
  palette: "character",
  regions: {
    // Hair on top
    hair: { rect: [2, 0, 3, 2], z: 0 },
    hair_highlight: { points: [[2, 1]], z: 1 },
    // Face
    outline: {
      union: [
        { points: [[1, 2], [5, 2]] },
        { points: [[1, 7], [4, 7]] },
      ],
      z: 0,
    },
    skin: { rect: [2, 2, 3, 3], z: 1 },
    skin_shadow: { points: [[3, 4]], z: 2 },
    eye: { points: [[2, 3], [4, 3]], z: 2 },
    // Body
    cloth: { rect: [2, 5, 3, 2], z: 0 },
    cloth_shadow: { points: [[2, 6], [4, 6]], z: 1 },
  },
}
```

## Variants

Create variations without duplicating entire sprites:

```json5
{
  type: "variant",
  name: "hero_damaged",
  base: "hero_idle",
  palette: {
    skin: "#FFB8A8",
    skin_shadow: "#D89080",
  },
}
```

The variant inherits everything from `hero_idle` but overrides the skin colors to show damage.

Common variant use cases:
- Damage/heal states
- Different color schemes (red team vs blue team)
- Seasonal variations
- Lighting conditions (day/night)

## Compositions

Layer sprites to create complex scenes:

```json5
{
  type: "composition",
  name: "scene",
  size: [64, 64],
  layers: [
    { sprite: "background_grass", x: 0, y: 0 },
    { sprite: "tree", x: 8, y: 24 },
    { sprite: "hero_idle", x: 28, y: 48 },
    { sprite: "ui_healthbar", x: 2, y: 2 },
  ],
}
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

```json5
{ type: "include", path: "../palettes/characters.pxl" }

{
  type: "sprite",
  name: "hero",
  size: [16, 16],
  palette: "character",
  regions: { ... },
}
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

```json5
{
  type: "palette",
  name: "knight",
  colors: {
    _: "transparent",
    outline: "#1A1A2E",
    armor: "#7F8C8D",
    armor_light: "#95A5A6",
    armor_dark: "#5D6D7E",
    visor: "#2C3E50",
    plume: "#C0392B",
    plume_dark: "#922B21",
  },
}

{
  type: "sprite",
  name: "knight_idle",
  size: [8, 8],
  palette: "knight",
  regions: {
    // Plume on helmet
    plume: {
      union: [
        { rect: [2, 0, 3, 1] },
        { rect: [1, 1, 5, 1] },
      ],
      z: 0,
    },
    plume_dark: { points: [[1, 1]], z: 1 },
    // Helmet
    outline: {
      union: [
        { points: [[1, 2], [5, 2]] },
        { points: [[0, 3], [6, 3]] },
        { points: [[0, 4], [6, 4]] },
        { points: [[1, 5], [5, 5]] },
        { points: [[1, 7], [4, 7]] },
      ],
      z: 0,
    },
    armor_light: { points: [[1, 3], [5, 3]], z: 1 },
    armor: {
      union: [
        { rect: [2, 2, 3, 1] },
        { rect: [2, 3, 3, 1] },
        { rect: [2, 5, 3, 1] },
        { rect: [1, 6, 5, 1] },
      ],
      z: 1,
    },
    armor_dark: {
      union: [
        { rect: [2, 4, 3, 1] },
        { points: [[2, 6], [4, 6]] },
      ],
      z: 2,
    },
    visor: { points: [[3, 3]], z: 2 },
  },
}

{
  type: "variant",
  name: "knight_gold",
  base: "knight_idle",
  palette: {
    armor: "#F39C12",
    armor_light: "#F7DC6F",
    armor_dark: "#D4AC0D",
  },
}
```

Try changing `armor` to `#FFD700` (gold) and `plume` to `#4169E1` (blue) for a royal knight.

This gives you a silver knight, a gold variant, and a reusable palette for additional knight sprites.
