# validate

Validate Pixelsrc files for errors and common mistakes.

## Usage

```
pxl validate [OPTIONS] [FILES]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[FILES]...` | Files to validate (omit if using `--stdin`) |

## Options

| Option | Description |
|--------|-------------|
| `--stdin` | Read input from stdin |
| `--strict` | Treat warnings as errors |
| `--json` | Output as JSON |

## Description

The `validate` command checks Pixelsrc files for:
- Syntax errors
- Missing palette references
- Invalid color values
- Undefined tokens in grids
- Row length mismatches
- Other structural issues

By default, the command distinguishes between errors (which cause a non-zero exit) and warnings (informational only). Use `--strict` to treat all issues as errors.

## Examples

### Basic validation

```bash
# Validate a single file
pxl validate sprite.pxl

# Validate multiple files
pxl validate *.pxl

# Validate with glob pattern
pxl validate assets/**/*.pxl
```

### Strict mode

```bash
# Fail on any warnings (useful in CI)
pxl validate --strict sprite.pxl
```

### JSON output

```bash
# Get machine-readable output
pxl validate --json sprite.pxl

# Pipe to jq for filtering
pxl validate --json sprite.pxl | jq '.errors'
```

### Stdin input

```bash
# Validate piped content
cat sprite.pxl | pxl validate --stdin

# Validate generated content
pxl sketch -n test < input.txt | pxl validate --stdin
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All files valid (no errors) |
| 1 | Validation errors found |
| 2 | Validation warnings found (only with `--strict`) |

## Common Errors

### Undefined token

```
Error: undefined token 'xyz' at row 3, column 2
```

The grid references a token that isn't defined in the palette.

### Row length mismatch

```
Error: row 3 has 5 columns, expected 8
```

All rows in a sprite grid should have the same width.

### Missing palette

```
Error: sprite 'hero' references undefined palette 'colors'
```

The sprite uses a palette that doesn't exist in the file or includes.

## See Also

- [fmt](fmt.md) - Format files for consistent style
- [suggest](suggest.md) - Get fix suggestions for errors
- [explain](explain.md) - Get detailed explanations of objects
