# Build System Integration

Integrate Pixelsrc into your build pipeline to automatically render sprites during development and deployment.

## Command-Line Interface

The `pxl` CLI is designed for build system integration:

```bash
# Render a sprite
pxl render sprite.pxl -o output.png

# Validate without rendering (fast)
pxl validate sprite.pxl

# Strict mode for CI (warnings become errors)
pxl render sprite.pxl --strict -o output.png
```

## Exit Codes

Use exit codes for conditional build logic:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (or warning in strict mode) |
| 2 | Invalid arguments |

## Make / Justfile

Basic Makefile integration:

```makefile
SPRITES := $(wildcard assets/*.pxl)
PNGS := $(SPRITES:.pxl=.png)

all: $(PNGS)

%.png: %.pxl
	pxl render $< -o $@

clean:
	rm -f $(PNGS)
```

With [Just](https://just.systems), the project includes a ready-to-use justfile:

```bash
# Render an example
just render coin

# Run all checks
just check

# List available commands
just --list
```

## npm Scripts

Add to `package.json`:

```json
{
  "scripts": {
    "sprites:build": "pxl render assets/sprites/*.pxl -o dist/sprites/",
    "sprites:watch": "pxl build assets/sprites/*.pxl -o dist/sprites/ --watch",
    "sprites:validate": "pxl validate assets/sprites/*.pxl --strict"
  }
}
```

## Watch Mode

The CLI supports watch mode for development:

```bash
pxl build assets/sprites/ -o dist/ --watch
```

Watch mode:
- Monitors source files for changes
- Re-renders only changed files
- Shows errors inline without stopping
- Recovers automatically when errors are fixed

## CI/CD Integration

### GitHub Actions

```yaml
name: Build Sprites

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Pixelsrc
        run: cargo install pixelsrc

      - name: Validate sprites
        run: pxl validate assets/*.pxl --strict

      - name: Render sprites
        run: pxl render assets/*.pxl -o dist/

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sprites
          path: dist/
```

### GitLab CI

```yaml
build-sprites:
  image: rust:latest
  script:
    - cargo install pixelsrc
    - pxl validate assets/*.pxl --strict
    - pxl render assets/*.pxl -o dist/
  artifacts:
    paths:
      - dist/
```

## Configuration File

Create `pxl.toml` in your project root for persistent settings:

```toml
[render]
# Default output format
format = "png"

# Default scale factor
scale = 4

# Strict mode by default
strict = true

[build]
# Watch directories
watch = ["assets/sprites"]

# Output directory
output = "dist/sprites"

# Exclude patterns
exclude = ["*_draft.pxl"]
```

## Project Structure

Recommended layout for projects with sprites:

```
project/
├── assets/
│   └── sprites/
│       ├── characters/
│       │   ├── hero.pxl
│       │   └── enemy.pxl
│       ├── items/
│       │   ├── sword.pxl
│       │   └── potion.pxl
│       └── palettes/
│           └── shared.pxl
├── dist/
│   └── sprites/
│       └── ... (generated PNGs)
├── pxl.toml
└── package.json
```

## Shared Palettes

Use the `--include` flag to share palettes across sprites:

```bash
pxl render sprite.pxl --include palettes/shared.pxl -o output.png
```

Or reference palettes by path in your source files using the `includes` field.

## Batch Processing

Render multiple sprites with glob patterns:

```bash
# Render all sprites in a directory
pxl render assets/sprites/**/*.pxl -o dist/

# Render with specific output names
pxl render character.pxl enemy.pxl -o dist/{name}.png
```

The `{name}` placeholder uses the sprite name for the output filename.

## Format Conversion

Convert between output formats:

```bash
# PNG (default)
pxl render sprite.pxl -o output.png

# GIF animation
pxl render animation.pxl -o output.gif

# Spritesheet
pxl render sprites.pxl -o sheet.png --spritesheet

# Terminal preview
pxl render sprite.pxl --terminal
```

## Error Handling in Scripts

Example bash script with proper error handling:

```bash
#!/bin/bash
set -e

echo "Validating sprites..."
if ! pxl validate assets/*.pxl --strict; then
    echo "Validation failed!"
    exit 1
fi

echo "Rendering sprites..."
pxl render assets/*.pxl -o dist/

echo "Done! Rendered $(ls dist/*.png | wc -l) sprites."
```

## Performance Tips

1. **Use validate for quick checks** - Faster than full rendering
2. **Batch similar sprites** - Reduces CLI startup overhead
3. **Cache in CI** - Cache Cargo builds to speed up installation
4. **Watch mode for development** - Avoids manual re-running
5. **Incremental builds** - Only re-render changed files

## Related

- [CLI Reference](../cli/overview.md) - Complete command documentation
- [Configuration](../reference/config.md) - pxl.toml reference
- [Exit Codes](../reference/exit-codes.md) - All exit code meanings
