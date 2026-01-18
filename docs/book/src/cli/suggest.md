# suggest

Suggest fixes for Pixelsrc files (missing tokens, row completion).

## Usage

```
pxl suggest [OPTIONS] [FILES]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[FILES]...` | Files to analyze (omit if using `--stdin`) |

## Options

| Option | Description |
|--------|-------------|
| `--stdin` | Read input from stdin |
| `--json` | Output as JSON |
| `--only <ONLY>` | Only show a specific type of suggestion (`token`, `row`) |

## Description

The `suggest` command analyzes Pixelsrc files and provides actionable suggestions for:
- **Token suggestions**: Likely intended tokens for typos or undefined references
- **Row completion**: Missing pixels to complete partial rows

This is particularly useful when working with AI-generated content that may have minor issues.

## Examples

### Get all suggestions

```bash
# Analyze a file and show suggestions
pxl suggest sprite.pxl
```

### Filter by suggestion type

```bash
# Only show token suggestions (typos, undefined refs)
pxl suggest sprite.pxl --only token

# Only show row completion suggestions
pxl suggest sprite.pxl --only row
```

### JSON output

```bash
# Get structured suggestions
pxl suggest sprite.pxl --json

# Apply suggestions programmatically
pxl suggest sprite.pxl --json | jq '.suggestions[] | .fix'
```

### Stdin input

```bash
# Suggest fixes for piped content
cat sprite.pxl | pxl suggest --stdin

# Chain with AI generation
generate_sprite | pxl suggest --stdin
```

## Sample Output

```
Analyzing sprite.pxl...

Token suggestions:
  Line 5: 'blck' → did you mean 'black'?
  Line 8: 'wht' → did you mean 'white'?

Row completion:
  Line 12: row has 7 tokens, expected 8
    Suggested: add 1x '_' (transparent) at end

  Line 15: row has 6 tokens, expected 8
    Suggested: add 2x 'black' at end (matches surrounding rows)
```

## Use Cases

- **AI output cleanup**: Fix common AI generation errors
- **Typo detection**: Find and fix token typos
- **Incomplete sprites**: Complete partially-defined rows
- **Learning**: Understand common mistakes and fixes

## See Also

- [validate](validate.md) - Check files for errors
- [fmt](fmt.md) - Format files for consistency
- [prime](prime.md) - Get AI context to prevent issues
