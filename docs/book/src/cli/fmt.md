# fmt

Format Pixelsrc files for consistent, readable style.

## Usage

```
pxl fmt [OPTIONS] <FILES>...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<FILES>...` | Input file(s) to format |

## Options

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without writing (exit 1 if changes needed) |
| `--stdout` | Write to stdout instead of in-place |

## Description

The `fmt` command standardizes the formatting of Pixelsrc files:
- Consistent indentation
- Aligned grid columns
- Normalized whitespace
- Ordered object fields

By default, files are modified in place. Use `--check` to verify formatting without making changes, or `--stdout` to preview the result.

## Examples

### Format files in place

```bash
# Format a single file
pxl fmt sprite.pxl

# Format multiple files
pxl fmt *.pxl

# Format all files in a directory tree
pxl fmt assets/**/*.pxl
```

### Check mode (CI integration)

```bash
# Check if files are formatted (exit 1 if not)
pxl fmt --check sprite.pxl

# In a CI pipeline
pxl fmt --check *.pxl || echo "Run 'pxl fmt' to fix formatting"
```

### Preview changes

```bash
# See formatted output without modifying file
pxl fmt --stdout sprite.pxl

# Diff against current file
pxl fmt --stdout sprite.pxl | diff sprite.pxl -
```

## Formatting Rules

The formatter applies these conventions:

### Grid alignment

Tokens in grids are space-separated and columns are aligned:

```
grid:
  r r r _ r r r
  r R R R R R r
  r R _ _ _ R r
  r r r r r r r
```

### Whitespace

- Single space between tokens
- No trailing whitespace
- Single newline at end of file

### Field ordering

Object fields are ordered consistently:
1. Type/kind fields first
2. Name/identifier
3. Content fields
4. Metadata fields last

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (files formatted or already formatted) |
| 1 | Files need formatting (with `--check`) |

## See Also

- [validate](validate.md) - Check files for errors
- [inline](inline.md) - Expand grid spacing for readability
