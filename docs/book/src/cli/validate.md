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
- Undefined tokens in regions
- Invalid shape coordinates
- Other structural issues

By default, the command distinguishes between errors (which cause a non-zero exit) and warnings (informational only). Use `--strict` to treat all issues as errors.

## Examples

<!-- DEMOS cli/validate#valid -->
**Valid File Example**

A properly structured file passes validation without errors.

<div class="demo-source">

```jsonl
{"type": "palette", "name": "valid", "colors": {"_": "#00000000", "x": "#ff0000"}}
{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "valid", "regions": {"x": {"rect": [0, 0, 1, 1], "z": 0}}}
```

</div>

<div class="demo-container" data-demo="valid">
</div>

```bash
pxl validate dot.pxl
# ✓ dot.pxl: valid
```
<!-- /DEMOS -->

<!-- DEMOS cli/validate#error -->
**Validation Error Example**

Files with errors show detailed diagnostic messages.

<div class="demo-source">

```jsonl
{"type": "sprite", "name": "broken", "size": [3, 1], "palette": "missing", "regions": {"x": {"rect": [0, 0, 1, 1], "z": 0}, "y": {"rect": [1, 0, 1, 1], "z": 0}, "z": {"rect": [2, 0, 1, 1], "z": 0}}}
```

</div>

<div class="demo-container" data-demo="error">
</div>

```bash
pxl validate broken.pxl
# error: sprite 'broken' references undefined palette 'missing'
# error: undefined tokens in regions: x, y, z
```
<!-- /DEMOS -->

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
Error: undefined token 'xyz' in regions
```

The regions reference a token that isn't defined in the palette.

### Invalid shape coordinates

```
Error: rect extends beyond sprite bounds at [10, 0, 5, 5]
```

Shape coordinates must fall within the sprite's defined size.

### Missing palette

```
Error: sprite 'hero' references undefined palette 'colors'
```

The sprite uses a palette that doesn't exist in the file or includes.

## See Also

- [fmt](fmt.md) - Format files for consistent style
- [suggest](suggest.md) - Get fix suggestions for errors
- [explain](explain.md) - Get detailed explanations of objects
