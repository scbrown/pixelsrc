# The Tool Builder

You build **pipelines and automation**. You want reliable validation, reproducible builds, and seamless CI/CD integration.

## Your Workflow

1. Set up a project with `pxl init`
2. Configure validation rules
3. Integrate with build systems
4. Add CI/CD checks

## Project Setup

Initialize a Pixelsrc project:

```bash
pxl init
```

This creates `pxl.toml` with default configuration:

```toml
[project]
name = "my-sprites"
version = "1.0.0"

[build]
input = "src/**/*.pxl"
output = "dist"

[validation]
strict = true
```

## Validation Pipeline

### Basic Validation

Check all files for errors:

```bash
pxl validate src/
```

### Strict Mode

Catch warnings in CI:

```bash
pxl validate src/ --strict
```

Strict mode fails on:
- Missing palette definitions
- Undefined tokens
- Row length mismatches
- Unused palettes or sprites

### Validation Output

```bash
$ pxl validate src/ --strict
src/hero.pxl:15: warning: undefined token {typo} in sprite "hero_idle"
src/items.pxl:8: error: palette "missing" not found
Validation failed: 1 error, 1 warning
```

## Build System

### Single File Build

```bash
pxl render hero.pxl -o dist/hero.png
```

### Batch Building

Build all sprites in a directory:

```bash
pxl build src/ -o dist/
```

This renders all sprites and animations to the output directory.

### Build Configuration

Configure in `pxl.toml`:

```toml
[build]
input = "src/**/*.pxl"
output = "dist"
scale = 1

[build.png]
enabled = true

[build.gif]
enabled = true
scale = 2

[build.spritesheet]
enabled = true
format = "json"
```

### Watch Mode

Auto-rebuild on changes:

```bash
pxl build src/ -o dist/ --watch
```

## Formatting

Keep files consistent:

```bash
pxl fmt src/
```

Format configuration in `pxl.toml`:

```toml
[format]
indent = 2
sort_keys = true
compact_small_grids = true
```

### Format Check (CI)

```bash
pxl fmt src/ --check
```

Exits with non-zero status if files need formatting.

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Validate Sprites

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install pxl
        run: cargo install pixelsrc

      - name: Check formatting
        run: pxl fmt src/ --check

      - name: Validate sprites
        run: pxl validate src/ --strict

      - name: Build assets
        run: pxl build src/ -o dist/

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sprites
          path: dist/
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
pxl fmt src/ --check || exit 1
pxl validate src/ --strict || exit 1
```

## Analysis Tools

### Diff Sprites

Compare two versions of a file:

```bash
pxl diff old/hero.pxl new/hero.pxl
```

### Analyze Usage

Find unused palettes and sprites:

```bash
pxl analyze src/
```

Output:
```
Unused palettes:
  - old_palette (src/palettes.pxl:15)

Unused sprites:
  - hero_v1 (src/hero.pxl:30)
  - test_sprite (src/test.pxl:5)

Statistics:
  - 12 palettes defined
  - 48 sprites defined
  - 8 animations defined
```

## Import/Export Workflows

### Import from PNG

Convert existing pixel art:

```bash
pxl import hero.png -o hero.pxl
```

Options:
- `--palette-name`: Name for generated palette
- `--sprite-name`: Name for generated sprite
- `--max-colors`: Limit palette size

### Batch Import

```bash
for f in assets/*.png; do
  pxl import "$f" -o "src/$(basename "$f" .png).pxl"
done
```

## Makefile Integration

```makefile
SOURCES := $(wildcard src/**/*.pxl)
OUTPUTS := $(patsubst src/%.pxl,dist/%.png,$(SOURCES))

.PHONY: all validate format clean

all: validate $(OUTPUTS)

validate:
	pxl validate src/ --strict

format:
	pxl fmt src/

dist/%.png: src/%.pxl
	@mkdir -p $(dir $@)
	pxl render $< -o $@

clean:
	rm -rf dist/
```

## Best Practices

### Directory Structure

```
project/
├── pxl.toml
├── src/
│   ├── palettes/
│   │   └── common.pxl
│   ├── sprites/
│   │   ├── characters/
│   │   ├── items/
│   │   └── ui/
│   └── animations/
├── dist/           # Generated output
└── .github/
    └── workflows/
        └── sprites.yml
```

### Versioning

Track `.pxl` source files in git. Add `dist/` to `.gitignore` if building in CI.

### Documentation

Use the `explain` command to generate documentation:

```bash
pxl explain src/hero.pxl > docs/hero.md
```

This creates readable documentation of all objects in the file.
