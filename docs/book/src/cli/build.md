# build

Build all assets according to `pxl.toml` configuration.

## Synopsis

```bash
pxl build [OPTIONS]
```

## Description

The `build` command processes all `.pxl` source files and generates output assets (PNG sprites, GIF animations, sprite atlases, and game engine exports). It reads configuration from `pxl.toml` to determine source locations, output directories, and export settings.

The build system supports:

- **Incremental builds**: Only rebuilds files that have changed
- **Parallel builds**: Uses multiple workers for faster processing
- **Progress reporting**: Shows build status in console or JSON format
- **Watch mode**: Monitors files and rebuilds on changes
- **Game engine exports**: Generates assets for Godot, Unity, and libGDX

## Options

| Option | Description |
|--------|-------------|
| `-o, --out <DIR>` | Override output directory (default: from `pxl.toml` or `build/`) |
| `--src <DIR>` | Override source directory (default: from `pxl.toml` or `src/pxl/`) |
| `-w, --watch` | Watch for changes and rebuild automatically |
| `--dry-run` | Show what would be built without building |
| `-v, --verbose` | Show detailed output including config path and file processing |

## Examples

<!-- DEMOS cli/build#basic -->
**Basic Build Example**

Build all assets according to `pxl.toml` configuration.

<div class="demo-source">

```toml
# pxl.toml
[project]
name = "my-game"
src = "src/pxl"
out = "build"

[defaults]
scale = 2
```

</div>

<div class="demo-container" data-demo="basic">
</div>

```bash
pxl build
# Building 5 targets...
#   sprite:player ... ok (150ms)
#   sprite:enemy ... ok (120ms)
# Build succeeded: 5 built in 270ms
```
<!-- /DEMOS -->

### Basic Build

Build all assets using defaults from `pxl.toml`:

```bash
pxl build
```

### Custom Directories

Override source and output directories:

```bash
pxl build --src assets/sprites --out dist/images
```

### Watch Mode

<!-- DEMOS cli/build#watch -->
**Watch Mode**

Automatically rebuild when source files change—ideal for development workflows.

<div class="demo-source">

```bash
pxl build --watch
# Watching src/pxl for changes...
# [12:34:56] sprite:player rebuilt (45ms)
# [12:35:01] sprite:enemy rebuilt (38ms)
# Press Ctrl+C to stop
```

</div>

<div class="demo-container" data-demo="watch">
</div>
<!-- /DEMOS -->

Automatically rebuild when source files change:

```bash
pxl build --watch
```

Watch mode:
- Monitors source files for changes
- Re-renders only changed files (incremental)
- Shows errors inline without stopping
- Recovers automatically when errors are fixed
- Press `Ctrl+C` to stop

### Dry Run

Preview what would be built without writing files:

```bash
pxl build --dry-run
```

Output shows source/output directories and file counts:

```
Dry run - would build:
  Source: src/pxl
  Output: build
  Files: 24
  Sprites: 18
```

### Verbose Output

Show detailed build information:

```bash
pxl build --verbose
```

Displays config file path, individual file processing, and timing information.

## Build Pipeline

The build system processes files through a multi-stage pipeline:

1. **Discovery**: Scan source directories for `.pxl` files using glob patterns
2. **Planning**: Create a build plan with targets and dependencies
3. **Incremental Check**: Skip targets that haven't changed since last build
4. **Parallel Execution**: Process independent targets concurrently
5. **Export Generation**: Create game engine-specific output formats

### Incremental Builds

The build system tracks file hashes and timestamps in a manifest file. On subsequent builds, it skips targets where:
- Source files haven't changed
- Output files still exist
- No dependencies have been rebuilt

Use `--force` flag (when available) to bypass incremental checking.

### Parallel Execution

By default, the build uses multiple worker threads to process independent targets simultaneously. Targets with dependencies are scheduled to build after their dependencies complete.

## Configuration

The build command reads settings from `pxl.toml`. Key sections:

```toml
[project]
name = "my-game"
src = "src/pxl"    # Source directory
out = "build"      # Output directory

[defaults]
scale = 2          # Default scale factor
padding = 1        # Default atlas padding

[atlases.characters]
sources = ["sprites/player/**", "sprites/enemies/**"]
max_size = [2048, 2048]

[animations]
sources = ["animations/**"]
preview = true     # Generate GIF previews
sheet_layout = "horizontal"

[export.godot]
enabled = true
resource_path = "res://assets/sprites"

[export.unity]
enabled = true
pixels_per_unit = 16

[watch]
debounce_ms = 100
clear_screen = true
```

See [Configuration Reference](../reference/config.md) for complete options.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Build succeeded |
| 1 | Build failed (errors in source files or I/O) |
| 2 | Invalid arguments or missing config |

## Progress Output

### Console Output

Standard build progress shows target status:

```
Building 5 targets...
  sprite:player ... ok (150ms)
  sprite:enemy ... ok (120ms)
  atlas:characters ... ok (45ms)
  export:godot:characters ... ok (12ms)
  export:unity:characters ... ok (8ms)
Build succeeded: 5 built in 335ms
```

Skipped targets (incremental builds) show:

```
  sprite:player ... skipped (up to date)
```

### Verbose Mode

With `--verbose`, shows additional details:

```
Using config: /project/pxl.toml
Building 5 targets (4 workers)...
  [1/5] sprite:player src/pxl/player.pxl -> build/player.png
  ...
```

## Related

- [Configuration](../reference/config.md) - Full `pxl.toml` reference
- [Build System Integration](../integrations/build-system.md) - CI/CD and Makefile integration
- [render](render.md) - Render individual files
- [validate](validate.md) - Validate without rendering
