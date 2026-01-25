# Example Gallery

Example prompts and their expected outputs for generating pixel art with AI.

## Simple Item: Heart

**Prompt:**

> Create a 7x6 pixel art heart icon with a red base color and pink highlight

**Output:**

```json5
{
  type: "palette",
  name: "heart",
  colors: {
    _: "transparent",
    r: "#FF0000",
    p: "#FF6B6B",
  },
}

{
  type: "sprite",
  name: "heart",
  size: [7, 6],
  palette: "heart",
  regions: {
    r: {
      union: [
        { rect: [1, 0, 2, 1] },     // Left lobe top
        { rect: [4, 0, 2, 1] },     // Right lobe top
        { rect: [0, 1, 7, 2] },     // Wide middle
        { rect: [1, 3, 5, 1] },     // Narrowing
        { rect: [2, 4, 3, 1] },     // Near tip
        { rect: [3, 5, 1, 1] },     // Tip
      ],
      z: 0,
    },
    p: { points: [[1, 1], [4, 1]], z: 1 },
  },
}
```

## Collectible: Animated Coin

**Prompt:**

> Create an 8x8 animated spinning coin with 4 frames showing rotation

**Output:**

```json5
{
  type: "palette",
  name: "coin",
  colors: {
    _: "transparent",
    g: "#FFD700",
    y: "#FFEC8B",
    o: "#B8860B",
    d: "#8B6914",
  },
}

// Frame 1: Full face
{
  type: "sprite",
  name: "coin_1",
  size: [8, 8],
  palette: "coin",
  regions: {
    g: {
      union: [
        { rect: [2, 0, 4, 1] },
        { rect: [1, 1, 6, 1] },
        { rect: [0, 2, 8, 4] },
        { rect: [1, 6, 6, 1] },
      ],
      z: 0,
    },
    y: {
      union: [
        { rect: [2, 1, 4, 1] },
        { rect: [1, 2, 2, 4] },
      ],
      z: 1,
    },
    o: { rect: [2, 6, 4, 1], z: 1 },
    d: { rect: [2, 7, 4, 1], z: 2 },
  },
}

// Frame 2: Slight rotation
{
  type: "sprite",
  name: "coin_2",
  size: [8, 8],
  palette: "coin",
  regions: {
    g: {
      union: [
        { rect: [3, 0, 2, 1] },
        { rect: [2, 1, 4, 1] },
        { rect: [1, 2, 6, 4] },
        { rect: [2, 6, 4, 1] },
      ],
      z: 0,
    },
    y: { rect: [2, 2, 2, 4], z: 1 },
    o: { rect: [3, 6, 2, 1], z: 1 },
    d: { rect: [3, 7, 2, 1], z: 2 },
  },
}

// Frame 3: Edge on
{
  type: "sprite",
  name: "coin_3",
  size: [8, 8],
  palette: "coin",
  regions: {
    g: { rect: [3, 0, 2, 6], z: 0 },
    y: { rect: [3, 2, 1, 3], z: 1 },
    o: { rect: [3, 6, 2, 1], z: 1 },
    d: { rect: [3, 7, 2, 1], z: 2 },
  },
}

// Frame 4: Returning
{
  type: "sprite",
  name: "coin_4",
  size: [8, 8],
  palette: "coin",
  regions: {
    g: {
      union: [
        { rect: [3, 0, 2, 1] },
        { rect: [2, 1, 4, 1] },
        { rect: [1, 2, 6, 4] },
        { rect: [2, 6, 4, 1] },
      ],
      z: 0,
    },
    y: { rect: [4, 2, 2, 4], z: 1 },
    o: { rect: [3, 6, 2, 1], z: 1 },
    d: { rect: [3, 7, 2, 1], z: 2 },
  },
}

{
  type: "animation",
  name: "coin_spin",
  frames: ["coin_1", "coin_2", "coin_3", "coin_4"],
  duration: 150,
  loop: true,
}
```

## Character: Hero Sprite

**Prompt:**

> Create a 16x16 pixel art character with brown hair, skin tone face, blue shirt, and dark pants. Include an outline for definition.

**Output:**

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#2C1810",
    skin: "#FFCC99",
    hair: "#8B4513",
    eye: "#000000",
    shirt: "#4169E1",
    pants: "#1E3A5F",
    shoes: "#000000",
  },
}

{
  type: "sprite",
  name: "hero_idle",
  size: [16, 16],
  palette: "hero",
  regions: {
    // Outline drawn first (lowest z)
    outline: {
      union: [
        { stroke: [5, 1, 6, 10] },   // Head/body outline
        { stroke: [6, 11, 4, 4] },   // Legs outline
      ],
      z: 0,
    },
    // Hair
    hair: { rect: [6, 2, 4, 3], z: 1 },
    // Face
    skin: {
      union: [
        { rect: [6, 5, 4, 3] },      // Face
        { points: [[4, 10], [11, 10]] }, // Hands
      ],
      z: 1,
    },
    // Eyes
    eye: { points: [[7, 6], [9, 6]], z: 2 },
    // Shirt
    shirt: { rect: [5, 9, 6, 3], z: 1 },
    // Pants
    pants: { rect: [6, 12, 4, 2], z: 1 },
    // Shoes
    shoes: {
      union: [
        { rect: [5, 14, 2, 2] },
        { rect: [9, 14, 2, 2] },
      ],
      z: 1,
    },
  },
}
```

## Weapon: Sword

**Prompt:**

> Create a 16x16 pixel art sword with a silver blade, golden crossguard, and brown leather handle

**Output:**

```json5
{
  type: "sprite",
  name: "sword",
  size: [16, 16],
  palette: {
    _: "transparent",
    blade: "#C0C0C0",
    edge: "#E8E8E8",
    guard: "#FFD700",
    handle: "#8B4513",
    pommel: "#DAA520",
  },
  regions: {
    // Blade body (diagonal via line)
    blade: {
      union: [
        { line: [[14, 0], [5, 9]] },
        { line: [[13, 1], [6, 8]] },
      ],
      z: 0,
    },
    // Blade edge highlight
    edge: { line: [[14, 1], [6, 9]], z: 1 },
    // Crossguard
    guard: { rect: [3, 10, 6, 1], z: 2 },
    // Handle
    handle: { rect: [5, 11, 2, 4], z: 0 },
    // Pommel
    pommel: { rect: [5, 15, 2, 1], z: 1 },
  },
}
```

## Tile: Grass with Variation

**Prompt:**

> Create a 16x16 grass tile for a top-down RPG with varied green shades

**Output:**

```json5
{
  type: "sprite",
  name: "grass",
  size: [16, 16],
  palette: {
    g1: "#228B22",
    g2: "#32CD32",
    g3: "#3CB371",
    d: "#1E5F1E",
  },
  regions: {
    // Base grass (fills entire tile)
    g1: { rect: [0, 0, 16, 16], z: 0 },
    // Light grass highlights scattered
    g2: {
      points: [
        [1, 0], [6, 1], [11, 0], [14, 2],
        [2, 4], [8, 3], [13, 5], [0, 7],
        [5, 8], [10, 7], [15, 9], [3, 11],
        [7, 12], [12, 10], [1, 14], [9, 15],
      ],
      z: 1,
    },
    // Medium grass accents
    g3: {
      points: [
        [4, 0], [9, 2], [2, 5], [14, 4],
        [0, 9], [6, 10], [11, 8], [3, 13],
        [8, 14], [13, 12], [5, 15], [15, 15],
      ],
      z: 1,
    },
    // Dark grass shadows
    d: {
      points: [
        [8, 4], [2, 8], [12, 6], [6, 13],
      ],
      z: 1,
    },
  },
}
```

## Scene: Tiled Composition

**Prompt:**

> Create a 32x32 scene using 8x8 tiles. Make sky and grass tiles, then compose them into a simple landscape.

**Output:**

```json5
{
  type: "palette",
  name: "nature",
  colors: {
    sky: "#87CEEB",
    cloud: "#FFFFFF",
    grass: "#228B22",
    grass_dark: "#1E5F1E",
  },
}

{
  type: "sprite",
  name: "sky_tile",
  size: [8, 8],
  palette: "nature",
  regions: {
    sky: { rect: [0, 0, 8, 8], z: 0 },
    cloud: {
      union: [
        { rect: [2, 1, 2, 1] },
        { rect: [1, 2, 4, 2] },
      ],
      z: 1,
    },
  },
}

{
  type: "sprite",
  name: "grass_tile",
  size: [8, 8],
  palette: "nature",
  regions: {
    grass: { rect: [0, 0, 8, 8], z: 0 },
    grass_dark: {
      points: [
        [1, 0], [5, 1], [0, 2], [3, 3],
        [6, 4], [1, 5], [4, 6], [7, 7],
      ],
      z: 1,
    },
  },
}

{
  type: "composition",
  name: "landscape",
  size: [32, 32],
  layers: [
    // Top rows: sky
    { sprite: "sky_tile", x: 0, y: 0 },
    { sprite: "sky_tile", x: 8, y: 0 },
    { sprite: "sky_tile", x: 16, y: 0 },
    { sprite: "sky_tile", x: 24, y: 0 },
    { sprite: "sky_tile", x: 0, y: 8 },
    { sprite: "sky_tile", x: 8, y: 8 },
    { sprite: "sky_tile", x: 16, y: 8 },
    { sprite: "sky_tile", x: 24, y: 8 },
    // Bottom rows: grass
    { sprite: "grass_tile", x: 0, y: 16 },
    { sprite: "grass_tile", x: 8, y: 16 },
    { sprite: "grass_tile", x: 16, y: 16 },
    { sprite: "grass_tile", x: 24, y: 16 },
    { sprite: "grass_tile", x: 0, y: 24 },
    { sprite: "grass_tile", x: 8, y: 24 },
    { sprite: "grass_tile", x: 16, y: 24 },
    { sprite: "grass_tile", x: 24, y: 24 },
  ],
}
```

## Variant: Color Swap

**Prompt:**

> Create a hero sprite, then make a variant with red hair for an "enemy" version

**Output:**

```json5
{
  type: "sprite",
  name: "hero",
  size: [8, 8],
  palette: {
    _: "transparent",
    skin: "#FFCC99",
    hair: "#8B4513",
    shirt: "#4169E1",
  },
  regions: {
    hair: { rect: [2, 0, 4, 2], z: 0 },
    skin: { rect: [2, 2, 4, 2], z: 0 },
    shirt: {
      union: [
        { rect: [1, 4, 6, 3] },
        { rect: [0, 4, 1, 2] },   // Left arm
        { rect: [7, 4, 1, 2] },   // Right arm
      ],
      z: 0,
    },
  },
}

{
  type: "variant",
  name: "enemy",
  base: "hero",
  palette: {
    hair: "#FF0000",
    shirt: "#8B0000",
  },
}
```

## Tips for Writing Prompts

1. **Be specific about size**: "16x16", "32x32", "8x8"
2. **Name colors explicitly**: "silver blade", "golden hilt", "brown handle"
3. **Reference real games**: "like Zelda items", "SNES-era style"
4. **Request semantic tokens**: "use descriptive token names like skin and hair"
5. **Ask for palettes separately**: "first create a palette, then the sprite"
6. **Request structured regions**: "use rect, circle, and points shapes"

## Formatting Output

Use `pxl fmt` to clean up generated output:

```bash
# Format a generated file
pxl fmt generated.pxl

# Check if formatting is needed
pxl fmt --check generated.pxl
```
