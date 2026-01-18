# alias

Extract repeated patterns into single-letter aliases (outputs JSON).

## Usage

```
pxl alias [OPTIONS] <INPUT>
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

The `alias` command analyzes a sprite's grid and identifies repeated patterns that could be replaced with single-letter aliases. This is useful for:
- Reducing file size
- Improving readability of complex sprites
- Identifying reusable patterns

The output is JSON, making it easy to integrate with other tools or scripts.

## Examples

### Basic usage

```bash
# Analyze a sprite for aliasing opportunities
pxl alias sprite.pxl

# Analyze a specific sprite
pxl alias sprites.pxl --sprite hero
```

### Processing output

```bash
# Pretty-print the JSON output
pxl alias sprite.pxl | jq .

# Extract just the suggested aliases
pxl alias sprite.pxl | jq '.aliases'

# Count potential savings
pxl alias sprite.pxl | jq '.savings_percent'
```

## Sample Output

```json
{
  "sprite": "hero",
  "original_tokens": 256,
  "unique_tokens": 8,
  "aliases": [
    {
      "pattern": "black",
      "alias": "b",
      "occurrences": 45
    },
    {
      "pattern": "skin",
      "alias": "s",
      "occurrences": 38
    },
    {
      "pattern": "outline",
      "alias": "o",
      "occurrences": 24
    }
  ],
  "savings_bytes": 312,
  "savings_percent": 28.5
}
```

## How Aliases Work

In Pixelsrc format, you can define aliases in the palette:

```
palette:
  name: hero_colors
  colors:
    black: #000000
    skin: #E0A070
  aliases:
    b: black
    s: skin
```

Then use the short form in grids:

```
grid:
  _ _ b b b b _ _
  _ b s s s s b _
  b s s s s s s b
```

The `alias` command helps identify which aliases would be most beneficial.

## Use Cases

- **Optimization**: Reduce file size for large sprites
- **Refactoring**: Identify patterns before cleaning up sprites
- **Analysis**: Understand token distribution in sprites
- **AI integration**: Pre-process AI output for consistency

## See Also

- [fmt](fmt.md) - Format files after applying aliases
- [inline](inline.md) - View expanded grid
- [explain](explain.md) - Detailed sprite analysis
