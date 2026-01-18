# explain

Explain sprites and other objects in human-readable format.

## Usage

```
pxl explain [OPTIONS] <INPUT>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input file containing Pixelsrc objects |

## Options

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Name of specific object to explain (sprite, palette, etc.) |
| `--json` | Output as JSON |

## Description

The `explain` command provides detailed, human-readable descriptions of Pixelsrc objects including:
- Dimensions and size information
- Color usage statistics
- Animation frame counts
- Composition structure
- And more

This is useful for understanding complex sprites or debugging issues.

## Examples

### Explain all objects

```bash
# Explain everything in a file
pxl explain character.pxl
```

### Explain specific object

```bash
# Explain just the hero sprite
pxl explain character.pxl --name hero

# Explain a palette
pxl explain character.pxl --name colors
```

### JSON output

```bash
# Get structured output
pxl explain character.pxl --json

# Filter with jq
pxl explain character.pxl --json | jq '.sprites[0].dimensions'
```

## Sample Output

```
Sprite: hero
  Dimensions: 16x16 pixels
  Colors used: 5
    - skin (tan): 42 pixels
    - outline (black): 28 pixels
    - hair (brown): 18 pixels
    - shirt (blue): 32 pixels
    - transparent: 136 pixels

Palette: colors
  Colors: 8 defined
    skin    #E0A070
    outline #000000
    hair    #8B4513
    shirt   #4169E1
    ...

Animation: walk
  Frames: 4
  Duration: 400ms total (100ms per frame)
  Sprites: hero_walk_1, hero_walk_2, hero_walk_3, hero_walk_4
```

## Use Cases

- **Debugging**: Understand why a sprite looks wrong
- **Documentation**: Generate sprite documentation automatically
- **Analysis**: Count colors, measure dimensions, audit animations
- **Learning**: Understand how existing sprites are structured

## See Also

- [analyze](analyze.md) - Extract metrics from multiple files
- [show](show.md) - Visual preview in terminal
- [diff](diff.md) - Compare two sprites
