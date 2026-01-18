# CLI Overview

The `pxl` command-line interface provides tools for working with Pixelsrc files. Commands fall into several categories:

## Core Workflow

| Command | Description |
|---------|-------------|
| [render](render.md) | Render sprites to PNG, GIF, or atlas formats |
| [import](import.md) | Convert PNG images to Pixelsrc format |
| [validate](validate.md) | Check files for errors and common mistakes |
| [fmt](fmt.md) | Format files for consistent style |
| [build](build.md) | Build all assets according to `pxl.toml` |

## Authoring Tools

| Command | Description |
|---------|-------------|
| [new](new.md) | Create new assets from templates |
| [init](init.md) | Initialize a new Pixelsrc project |
| [sketch](sketch.md) | Create sprites from simple text grids |

## Inspection & Debugging

| Command | Description |
|---------|-------------|
| [show](show.md) | Display sprites with colored terminal output |
| [grid](grid.md) | Display grid with row/column coordinates |
| [inline](inline.md) | Expand grid with column-aligned spacing |
| [explain](explain.md) | Explain objects in human-readable format |
| [diff](diff.md) | Compare sprites semantically |
| [analyze](analyze.md) | Extract corpus metrics from files |

## AI Integration

| Command | Description |
|---------|-------------|
| [prime](prime.md) | Print format guide for AI context injection |
| [prompts](prompts.md) | Show GenAI prompt templates |
| [suggest](suggest.md) | Suggest fixes for common issues |
| [alias](alias.md) | Extract repeated patterns into aliases |

## Reference Data

| Command | Description |
|---------|-------------|
| [palettes](palettes.md) | List and inspect built-in palettes |

## Global Behavior

### File Formats

The CLI supports two file formats:
- `.pxl` - Human-readable Pixelsrc format
- `.jsonl` - JSON Lines format (legacy, still supported)

Both formats can contain palettes, sprites, animations, compositions, and other objects.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid input, missing file, etc.) |
| 2 | Validation failed (with `--strict` mode) |

See [Exit Codes](../reference/exit-codes.md) for the complete list.

### Common Options

Many commands share common options:

- `--json` - Output as JSON for scripting
- `--strict` - Treat warnings as errors
- `--stdin` - Read input from stdin

## Quick Examples

```bash
# Render a sprite to PNG
pxl render sprite.pxl -o output.png

# Validate all files in a directory
pxl validate *.pxl

# Format files in place
pxl fmt *.pxl

# Preview a sprite in terminal
pxl show sprite.pxl

# Build a project
pxl build

# Get AI context for sprite generation
pxl prime --brief
```
