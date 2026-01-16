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

# Full project build
pxl build
```

## Exit Codes

Use exit codes for conditional build logic:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (or warning in strict mode) |
| 2 | Invalid arguments |

## Incremental Builds

The build system tracks file changes and only rebuilds what's necessary:

```bash
# First build - processes all files
pxl build
# Build complete - Files: 24 | Sprites: 18

# Second build (no changes) - skips everything
pxl build
# Build complete - Files: 24 | Sprites: 0 (all skipped)

# After modifying player.pxl - only rebuilds affected files
pxl build
# Build complete - Files: 24 | Sprites: 1
```

The manifest file (`.pxl-manifest.json`) tracks:
- Source file hashes
- Output file paths
- Build timestamps
- Dependency relationships

### Forcing Full Rebuild

To bypass incremental checking:

```bash
# Delete the manifest
rm .pxl-manifest.json && pxl build

# Or use clean build in your tooling
```

## Parallel Builds

The build system automatically parallelizes independent targets:

```
Building 12 targets (4 workers)...
  sprite:player ... ok (150ms)
  sprite:enemy ... ok (120ms)
  sprite:coin ... ok (80ms)
  atlas:characters ... ok (45ms)  # waits for sprites
  export:godot ... ok (12ms)      # waits for atlas
```

Dependencies are respected: atlas builds wait for their source sprites, exports wait for atlases.

## Progress Reporting

### Console Output

Standard progress shows build status:

```
Building...
Build complete - Files: 24 | Sprites: 18
```

Verbose mode (`-v`) shows per-target details:

```bash
pxl build -v
# Using config: /project/pxl.toml
# Building 5 targets...
#   sprite:player ... ok (150ms)
#   sprite:enemy ... ok (120ms)
#   ...
```

### JSON Output (CI Integration)

For programmatic parsing, the build system can output JSON progress events:

```json
{"event":"build_started","total_targets":5}
{"event":"target_completed","target_id":"sprite:player","status":"success","duration_ms":150}
{"event":"target_completed","target_id":"sprite:enemy","status":"success","duration_ms":120}
{"event":"build_completed","success":true,"duration_ms":335,"succeeded":5,"skipped":0,"failed":0}
```

## Game Engine Exports

Configure exports in `pxl.toml` to generate engine-specific files alongside your sprites.

### Godot Export

```toml
[export.godot]
enabled = true
resource_path = "res://assets/sprites"
animation_player = true
sprite_frames = true
```

Generates:
- `.tres` resource files for atlases
- `AnimationPlayer` resources for animations
- `SpriteFrames` resources for animated sprites

### Unity Export

```toml
[export.unity]
enabled = true
pixels_per_unit = 16
filter_mode = "point"  # or "bilinear"
```

Generates:
- `.asset` meta files for sprites
- Sprite slicing data for atlases
- Animation clips for animated sprites

### libGDX Export

```toml
[export.libgdx]
enabled = true
```

Generates:
- `.atlas` files in libGDX TexturePacker format
- Region definitions for sprite lookup

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
    "sprites:build": "pxl build",
    "sprites:watch": "pxl build --watch",
    "sprites:validate": "pxl validate assets/sprites/*.pxl --strict"
  }
}
```

## Watch Mode

The CLI supports watch mode for development:

```bash
pxl build --watch
```

Watch mode:
- Monitors source files for changes
- Re-renders only changed files (incremental)
- Shows errors inline without stopping
- Recovers automatically when errors are fixed
- Configurable debounce delay

Configure watch behavior in `pxl.toml`:

```toml
[watch]
debounce_ms = 100    # Wait before rebuilding
clear_screen = true  # Clear terminal between builds
```

### Error Recovery

Watch mode continues running after errors. When you fix the error, the build automatically recovers:

```
Watching for changes (Ctrl+C to stop)...
[15:23:01] player.pxl changed
  ERROR: Invalid color 'gren' at line 5
[15:23:15] player.pxl changed
  ok - player.png
```

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

      - name: Build sprites
        run: pxl build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: sprites
          path: build/
```

### GitLab CI

```yaml
build-sprites:
  image: rust:latest
  script:
    - cargo install pixelsrc
    - pxl validate assets/*.pxl --strict
    - pxl build
  artifacts:
    paths:
      - build/
```

### Caching

Cache the Cargo build to speed up CI:

```yaml
# GitHub Actions
- uses: actions/cache@v4
  with:
    path: ~/.cargo
    key: ${{ runner.os }}-cargo-pixelsrc

# GitLab CI
cache:
  paths:
    - ~/.cargo
```

## Configuration File

Create `pxl.toml` in your project root for persistent settings:

```toml
[project]
name = "my-game"
src = "assets/sprites"
out = "dist/sprites"

[defaults]
scale = 4
padding = 1

[atlases.characters]
sources = ["player/**", "enemies/**"]
max_size = [2048, 2048]
power_of_two = true

[animations]
sources = ["animations/**"]
preview = true
sheet_layout = "horizontal"

[export.godot]
enabled = true
resource_path = "res://sprites"

[validate]
strict = true
unused_palettes = "warn"
missing_refs = "error"

[watch]
debounce_ms = 100
clear_screen = true
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
├── build/
│   ├── sprites/
│   │   └── ... (generated PNGs)
│   ├── godot/
│   │   └── ... (generated .tres files)
│   └── .pxl-manifest.json
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

echo "Building sprites..."
pxl build

echo "Done! Rendered $(ls build/*.png 2>/dev/null | wc -l) sprites."
```

## Performance Tips

1. **Use incremental builds** - Let the manifest track changes
2. **Validate before building** - Faster than full renders
3. **Batch similar sprites** - Reduces CLI startup overhead
4. **Cache in CI** - Cache Cargo builds and manifest files
5. **Watch mode for development** - Avoids manual re-running
6. **Parallel builds** - Enabled by default for multi-core systems

## Troubleshooting

### Build is slow

- Check if incremental builds are working (look for "skipped" in output)
- Ensure manifest file isn't being deleted between builds
- Use `--verbose` to identify slow targets

### Watch mode missing changes

- Check `debounce_ms` setting (increase if changes are batched)
- Verify source directory path in `pxl.toml`
- Some editors use atomic saves that may need different handling

### Exports not generating

- Verify export is enabled in `pxl.toml` (e.g., `[export.godot] enabled = true`)
- Check that atlas sources are correctly specified
- Run with `--verbose` to see export target status

## Related

- [CLI Reference](../cli/overview.md) - Complete command documentation
- [build command](../cli/build.md) - Full build command reference
- [Configuration](../reference/config.md) - pxl.toml reference
- [Exit Codes](../reference/exit-codes.md) - All exit code meanings
