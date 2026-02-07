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
- Aligned region definitions
- Normalized whitespace
- Ordered object fields

By default, files are modified in place. Use `--check` to verify formatting without making changes, or `--stdout` to preview the result.

## Examples

<!-- DEMOS cli/fmt#before_after -->
**Formatting Example**

The formatter standardizes spacing and alignment for consistent, readable files.

<div class="demo-source">

```jsonl
# Before formatting (compact):
{"type":"sprite","name":"icon","size":[2,2],"palette":{"x":"#ff0000"},"regions":{"x":{"rect":[0,0,2,2],"z":0}}}

# After pxl fmt:
{"type": "sprite", "name": "icon", "size": [2, 2], "palette": {"x": "#ff0000"}, "regions": {"x": {"rect": [0, 0, 2, 2], "z": 0}}}
```

</div>

<div class="demo-container" data-demo="before_after">
</div>

```bash
pxl fmt sprite.pxl
```
<!-- /DEMOS -->

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

<!-- DEMOS cli/fmt#check -->
**Check Mode for CI**

Verify formatting without modifying files—useful for CI pipelines.

<div class="demo-source">

```bash
# Check if files are formatted
pxl fmt --check sprite.pxl

# Exit code 0: already formatted
# Exit code 1: needs formatting
```

</div>

<div class="demo-container" data-demo="check">
</div>
<!-- /DEMOS -->

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

### Region alignment

Shape arrays and union blocks are consistently formatted:

```json5
regions: {
  body: { rect: [0, 0, 8, 8], z: 0 },
  detail: {
    union: [
      { rect: [1, 1, 2, 2] },
      { rect: [5, 1, 2, 2] },
    ],
    z: 1,
  },
}
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
