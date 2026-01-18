# inline

Expand grid with column-aligned spacing for readability.

## Usage

```
pxl inline [OPTIONS] <INPUT>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input file containing sprite definitions |

## Options

| Option | Description |
|--------|-------------|
| `--sprite <SPRITE>` | Sprite name (if file contains multiple) |

## Description

The `inline` command outputs a sprite's grid with tokens aligned in columns. This makes it easier to read and edit sprites in text editors, especially those with variable-width token names.

Unlike the compact format used in `.pxl` files, the inline output uses consistent column widths so that rows line up vertically.

## Examples

### Basic usage

```bash
# Show inline-expanded grid
pxl inline sprite.pxl

# For a specific sprite
pxl inline sprites.pxl --sprite hero
```

## Sample Output

Original (compact):

```
grid:
  _ _ black black _ _
  _ black skin skin black _
  black skin eye skin eye skin black
  black skin skin nose skin skin black
```

Inline (expanded):

```
grid:
  _     _     black black _     _
  _     black skin  skin  black _
  black skin  eye   skin  eye   skin  black
  black skin  skin  nose  skin  skin  black
```

## Use Cases

- **Editing**: Easier to identify column positions in text editor
- **Comparison**: Better visual alignment for manual diff
- **Teaching**: Clearer structure for learning sprite format
- **Documentation**: More readable examples in docs

## Comparison with Other Commands

| Command | Purpose |
|---------|---------|
| `inline` | Aligned text output for editing |
| `grid` | Coordinates for position reference |
| `show` | Colored visual preview |
| `fmt` | Format for storage (compact) |

## See Also

- [grid](grid.md) - Show grid with coordinates
- [show](show.md) - Colored visual preview
- [fmt](fmt.md) - Format files for storage
