# diff

Compare sprites semantically between two files.

## Usage

```
pxl diff [OPTIONS] <FILE_A> <FILE_B>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<FILE_A>` | First file to compare |
| `<FILE_B>` | Second file to compare |

## Options

| Option | Description |
|--------|-------------|
| `--sprite <SPRITE>` | Compare only a specific sprite by name |
| `--json` | Output as JSON |

## Description

The `diff` command performs a semantic comparison between Pixelsrc files, showing:
- Added, removed, or modified sprites
- Palette color changes
- Dimension changes
- Pixel-level differences within sprites

Unlike text diff, this command understands the structure of Pixelsrc files and provides meaningful comparisons.

## Examples

### Compare two files

```bash
# Compare all objects in two files
pxl diff before.pxl after.pxl
```

### Compare specific sprite

```bash
# Compare only the hero sprite between versions
pxl diff v1/character.pxl v2/character.pxl --sprite hero
```

### JSON output

```bash
# Get structured diff output
pxl diff before.pxl after.pxl --json

# Check if files are identical (empty diff)
pxl diff a.pxl b.pxl --json | jq '.changes | length'
```

## Sample Output

```
Comparing before.pxl → after.pxl

Sprite 'hero':
  Dimensions: unchanged (16x16)
  Colors: 1 changed
    - shirt: #4169E1 → #228B22 (blue → green)
  Pixels: 32 modified (12.5%)

Sprite 'enemy':
  Status: added in after.pxl
  Dimensions: 16x16
  Colors: 4

Palette 'colors':
  Colors: 1 added
    + accent: #FFD700
```

## Use Cases

- **Version control**: Understand what changed between commits
- **Review**: Check sprite modifications before merging
- **Debugging**: Find unexpected changes
- **Testing**: Verify that transformations produce expected results

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Comparison completed (even if differences found) |
| 1 | Error (file not found, invalid format, etc.) |

## See Also

- [validate](validate.md) - Check files for errors
- [explain](explain.md) - Detailed explanation of single file
- [analyze](analyze.md) - Corpus-level metrics
