# new

Create a new asset from template.

## Usage

```
pxl new [OPTIONS] <ASSET_TYPE> <NAME>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<ASSET_TYPE>` | Asset type: `sprite`, `animation`, `palette` |
| `<NAME>` | Asset name |

## Options

| Option | Description |
|--------|-------------|
| `--palette <PALETTE>` | Palette to use (for sprites and animations) |

## Description

The `new` command creates new Pixelsrc assets from templates. It generates properly structured files with placeholder content that you can customize.

## Examples

### Create a sprite

```bash
# Create a new sprite with default palette
pxl new sprite hero

# Create with a specific built-in palette
pxl new sprite player --palette @pico8

# Create with a custom palette name
pxl new sprite enemy --palette monster_colors
```

### Create an animation

```bash
# Create a new animation
pxl new animation walk

# With palette
pxl new animation run --palette @gameboy
```

### Create a palette

```bash
# Create a new palette definition
pxl new palette fantasy_colors
```

## Generated Content

### Sprite Template

```
palette:
  name: hero_palette
  colors:
    outline: #000000
    fill: #4a90d9
    highlight: #7cb5eb
    _: transparent

sprite:
  name: hero
  palette: hero_palette
  grid:
    _ _ outline outline outline outline _ _
    _ outline fill fill fill fill outline _
    outline fill highlight fill fill fill fill outline
    outline fill fill fill fill fill fill outline
    outline fill fill fill fill fill fill outline
    outline fill fill fill fill fill fill outline
    _ outline fill fill fill fill outline _
    _ _ outline outline outline outline _ _
```

### Animation Template

```
animation:
  name: walk
  frames:
    - sprite: walk_1
      duration: 100
    - sprite: walk_2
      duration: 100
    - sprite: walk_3
      duration: 100
    - sprite: walk_4
      duration: 100
```

### Palette Template

```
palette:
  name: fantasy_colors
  colors:
    black: #000000
    dark: #3a3a3a
    mid: #7a7a7a
    light: #bababa
    white: #ffffff
    _: transparent
```

## Output Location

By default, new assets are created in the current directory:
- `hero.pxl` for sprites
- `walk.pxl` for animations
- `fantasy_colors.pxl` for palettes

In a project with `pxl.toml`, assets are created in the configured source directory.

## See Also

- [init](init.md) - Initialize a new project
- [palettes](palettes.md) - List built-in palettes
- [render](render.md) - Render created sprites
