# sketch

Create sprite from simple text grid (space-separated characters).

## Usage

```
pxl sketch [OPTIONS] [FILE]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[FILE]` | Input file (omit to read from stdin) |

## Options

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Sprite name (default: "sketch") |
| `-p, --palette <PALETTE>` | Reference a named palette instead of inline placeholder colors |
| `-o, --output <OUTPUT>` | Output file (default: stdout) |

## Description

The `sketch` command creates Pixelsrc sprites from a simple text grid format. Each line is a row, with characters separated by spaces. Use `_` for transparent pixels.

This provides a quick way to prototype sprites without writing full Pixelsrc syntax.

## Input Format

```
_ _ b b _ _
_ b c c b _
b c c c c b
b c c c c b
_ b c c b _
_ _ b b _ _
```

Each character becomes a color token. The command generates a placeholder palette with colors for each unique character found.

## Examples

### From file

```bash
# Create sprite from text file
pxl sketch circle.txt -o circle.pxl

# With custom name
pxl sketch input.txt --name player -o player.pxl
```

### From stdin

```bash
# Pipe text directly
echo "_ b _
b c b
_ b _" | pxl sketch -n dot

# Here-doc for multi-line
pxl sketch -n heart << 'EOF'
_ r _ r _
r r r r r
r r r r r
_ r r r _
_ _ r _ _
EOF
```

### With palette reference

```bash
# Use an existing palette
pxl sketch sprite.txt --palette @pico8 -o sprite.pxl

# Reference a custom palette
pxl sketch sprite.txt --palette my_colors -o sprite.pxl
```

## Sample Output

Input:

```
_ _ b b _ _
_ b s s b _
b s e s e b
b s s n s b
_ b s s b _
_ _ b b _ _
```

Generated output:

```
palette:
  name: sketch_palette
  colors:
    b: #000000
    s: #E0A070
    e: #FFFFFF
    n: #8B4513
    _: transparent

sprite:
  name: sketch
  palette: sketch_palette
  grid:
    _ _ b b _ _
    _ b s s b _
    b s e s e b
    b s s n s b
    _ b s s b _
    _ _ b b _ _
```

## Placeholder Colors

When no palette is specified, the command generates placeholder colors:
- Single letters get assigned distinct colors
- `_` is always transparent
- Colors are designed to be visually distinct

You should replace these placeholder colors with your actual palette colors after generation.

## Workflow

1. **Sketch**: Create rough sprite with single-letter tokens
2. **Generate**: Run `pxl sketch` to create valid Pixelsrc
3. **Refine**: Update palette colors and token names
4. **Preview**: Use `pxl show` to see the result
5. **Render**: Use `pxl render` to export

## Use Cases

- **Quick prototyping**: Sketch ideas rapidly in text
- **ASCII art conversion**: Convert existing ASCII art to sprites
- **Teaching**: Learn Pixelsrc format with simple examples
- **Scripting**: Generate sprites programmatically

## See Also

- [new](new.md) - Create sprites from templates
- [import](import.md) - Import from PNG images
- [show](show.md) - Preview generated sprites
