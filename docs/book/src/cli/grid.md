# grid

Display grid with row/column coordinates for easy reference.

## Usage

```
pxl grid [OPTIONS] <INPUT>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input file containing palette and sprite definitions |

## Options

| Option | Description |
|--------|-------------|
| `--sprite <SPRITE>` | Sprite name (if file contains multiple sprites) |
| `--full` | Show full token names instead of abbreviations |

## Description

The `grid` command displays a sprite's pixel grid with row and column coordinates. This is useful for:
- Identifying specific pixel positions
- Communicating about edits ("change row 3, column 5")
- Understanding sprite structure

By default, tokens are abbreviated for compact display. Use `--full` to see complete token names.

## Examples

### Basic grid display

```bash
# Show grid with coordinates
pxl grid sprite.pxl
```

### Specific sprite

```bash
# Show grid for a named sprite
pxl grid sprites.pxl --sprite hero
```

### Full token names

```bash
# Show complete token names
pxl grid sprite.pxl --full
```

## Sample Output

Default (abbreviated):

```
     0   1   2   3   4   5   6   7
   ┌───┬───┬───┬───┬───┬───┬───┬───┐
 0 │ _ │ _ │ b │ b │ b │ b │ _ │ _ │
   ├───┼───┼───┼───┼───┼───┼───┼───┤
 1 │ _ │ b │ s │ s │ s │ s │ b │ _ │
   ├───┼───┼───┼───┼───┼───┼───┼───┤
 2 │ b │ s │ e │ s │ s │ e │ s │ b │
   ├───┼───┼───┼───┼───┼───┼───┼───┤
 3 │ b │ s │ s │ s │ s │ s │ s │ b │
   └───┴───┴───┴───┴───┴───┴───┴───┘
```

With `--full`:

```
       0       1       2       3
   ┌───────┬───────┬───────┬───────┐
 0 │ trans │ trans │ black │ black │
   ├───────┼───────┼───────┼───────┤
 1 │ trans │ black │ skin  │ skin  │
   └───────┴───────┴───────┴───────┘
```

## Use Cases

- **Editing guidance**: Identify exact coordinates for pixel changes
- **Documentation**: Reference specific sprite positions
- **Debugging**: Verify sprite structure is correct
- **Teaching**: Explain pixel placement to others

## See Also

- [show](show.md) - Colored visual preview
- [inline](inline.md) - Expand grid spacing
- [explain](explain.md) - Detailed sprite explanation
