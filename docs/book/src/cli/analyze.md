# analyze

Analyze Pixelsrc files and extract corpus metrics.

## Usage

```
pxl analyze [OPTIONS] [FILES]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[FILES]...` | Files to analyze |

## Options

| Option | Description |
|--------|-------------|
| `--dir <DIR>` | Directory to scan for `.jsonl`/`.pxl` files |
| `-r, --recursive` | Include subdirectories when scanning a directory |
| `--format <FORMAT>` | Output format: `text` or `json` (default: `text`) |
| `-o, --output <OUTPUT>` | Write output to file instead of stdout |

## Description

The `analyze` command extracts statistics and metrics from Pixelsrc files:
- Total file, sprite, and palette counts
- Dimension distributions
- Color usage patterns
- Token frequency analysis
- Animation statistics

This is useful for understanding a corpus of sprites, training data analysis, or project auditing.

## Examples

### Analyze single file

```bash
# Analyze one file
pxl analyze sprite.pxl
```

### Analyze directory

```bash
# Analyze all files in a directory
pxl analyze --dir assets/sprites

# Include subdirectories
pxl analyze --dir assets --recursive
```

### JSON output

```bash
# Get machine-readable output
pxl analyze --dir sprites --format json

# Save to file
pxl analyze --dir sprites --format json -o metrics.json
```

### Analyze multiple files

```bash
# Analyze specific files
pxl analyze player.pxl enemy.pxl items.pxl

# Use glob patterns
pxl analyze *.pxl
```

## Sample Output

Text format:

```
Corpus Analysis
===============

Files:        24
Sprites:      156
Palettes:     12
Animations:   28

Dimensions:
  8x8:        42 (26.9%)
  16x16:      89 (57.1%)
  32x32:      18 (11.5%)
  Other:       7 (4.5%)

Colors:
  Average per sprite: 6.2
  Most common:
    black     (156 sprites)
    white     (142 sprites)
    skin      (98 sprites)

Tokens:
  Unique:     47
  Most used:  _ (transparent), black, white
```

JSON format:

```json
{
  "files": 24,
  "sprites": 156,
  "palettes": 12,
  "animations": 28,
  "dimensions": {
    "8x8": 42,
    "16x16": 89,
    "32x32": 18,
    "other": 7
  },
  "colors": {
    "average_per_sprite": 6.2,
    "most_common": ["black", "white", "skin"]
  }
}
```

## Use Cases

- **Corpus analysis**: Understand patterns in sprite collections
- **Training data audit**: Verify AI training data quality
- **Project metrics**: Track sprite counts and dimensions
- **Documentation**: Generate statistics for project READMEs

## See Also

- [explain](explain.md) - Detailed explanation of single file
- [validate](validate.md) - Check files for errors
- [diff](diff.md) - Compare two files
