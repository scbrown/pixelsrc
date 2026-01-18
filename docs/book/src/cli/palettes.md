# palettes

List and inspect built-in palettes.

## Usage

```
pxl palettes <COMMAND>
```

## Subcommands

| Command | Description |
|---------|-------------|
| `list` | List all available built-in palettes |
| `show` | Show details of a specific palette |

## pxl palettes list

List all available built-in palettes.

```bash
pxl palettes list
```

Output:

```
Built-in palettes:
  @gameboy
  @nes
  @pico8
  @grayscale
  @1bit
  @dracula
```

## pxl palettes show

Show details of a specific palette.

```bash
pxl palettes show <NAME>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<NAME>` | Name of the palette to show (e.g., `gameboy`, `pico8`) |

### Examples

```bash
# Show Game Boy palette
pxl palettes show gameboy

# Show PICO-8 palette
pxl palettes show pico8
```

### Sample Output

```
Palette: gameboy
  Colors: 4
    darkest   #0f380f
    dark      #306230
    light     #8bac0f
    lightest  #9bbc0f

  Usage in sprites:
    palette: @gameboy
```

## Built-in Palettes

### @gameboy

The classic Game Boy 4-color green palette.

| Token | Color | Hex |
|-------|-------|-----|
| darkest | Dark green | #0f380f |
| dark | Medium green | #306230 |
| light | Light green | #8bac0f |
| lightest | Pale green | #9bbc0f |

### @nes

NES-inspired color palette with 54 colors.

### @pico8

The PICO-8 fantasy console 16-color palette.

### @grayscale

8-level grayscale from black to white.

### @1bit

Simple 2-color black and white palette.

### @dracula

The popular Dracula dark theme colors.

## Using Built-in Palettes

Reference built-in palettes with the `@` prefix:

```
sprite:
  name: player
  palette: @gameboy
  grid:
    _ _ darkest darkest _ _
    _ darkest light light darkest _
    darkest light lightest lightest light darkest
```

## See Also

- [Format: Palette](../format/palette.md) - Creating custom palettes
- [Reference: Palettes](../reference/palettes.md) - Complete palette reference
- [new](new.md) - Create sprites with built-in palettes
