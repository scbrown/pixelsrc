# Configuration (pxl.toml)

Project configuration file for Pixelsrc builds and exports.

## Overview

Create a `pxl.toml` file in your project root to configure the build system. The only required field is `project.name`.

```toml
[project]
name = "my-game"
```

## Full Example

```toml
[project]
name = "my-game"
version = "1.0.0"
src = "assets/pxl"
out = "dist"

[defaults]
scale = 2
padding = 4

[atlases.characters]
sources = ["sprites/player/**", "sprites/enemies/**"]
max_size = [2048, 2048]
padding = 2
power_of_two = true

[atlases.ui]
sources = ["sprites/ui/**"]
max_size = [1024, 1024]
nine_slice = true

[animations]
sources = ["anims/**"]
preview = true
preview_scale = 4
sheet_layout = "vertical"

[export.generic]
enabled = true
atlas_format = "json"

[export.godot]
enabled = true
resource_path = "res://sprites"
animation_player = true
sprite_frames = true

[export.unity]
enabled = true
pixels_per_unit = 32
filter_mode = "point"

[export.libgdx]
enabled = true

[validate]
strict = true
unused_palettes = "warn"
missing_refs = "error"

[watch]
debounce_ms = 200
clear_screen = false
```

## Sections

### [project]

Project metadata and directory configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | *required* | Project name |
| `version` | string | `"0.1.0"` | Project version |
| `src` | path | `"src/pxl"` | Source directory for `.pxl` files |
| `out` | path | `"build"` | Output directory for rendered assets |

```toml
[project]
name = "my-game"
version = "1.0.0"
src = "assets/sprites"
out = "dist/sprites"
```

### [defaults]

Default settings applied to all outputs unless overridden.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `scale` | integer | `1` | Default scale factor for rendering |
| `padding` | integer | `1` | Default padding between sprites in atlases |

```toml
[defaults]
scale = 2
padding = 4
```

### [atlases.\<name\>]

Define named atlas configurations. Multiple atlases can be defined.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sources` | array | *required* | Glob patterns for sprite sources |
| `max_size` | [w, h] | `[1024, 1024]` | Maximum atlas dimensions |
| `padding` | integer | from defaults | Padding between sprites |
| `power_of_two` | boolean | `false` | Constrain to power-of-two dimensions |
| `nine_slice` | boolean | `false` | Preserve nine-slice metadata |

```toml
[atlases.characters]
sources = ["sprites/player/**", "sprites/enemies/**"]
max_size = [2048, 2048]
padding = 2
power_of_two = true

[atlases.ui]
sources = ["sprites/ui/**"]
max_size = [1024, 1024]
nine_slice = true
```

### [animations]

Animation output configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sources` | array | `["animations/**"]` | Glob patterns for animation files |
| `preview` | boolean | `false` | Generate preview GIFs |
| `preview_scale` | integer | `1` | Scale factor for previews |
| `sheet_layout` | string | `"horizontal"` | Layout: `horizontal`, `vertical`, or `grid` |

```toml
[animations]
sources = ["anims/**", "characters/*/walk.pxl"]
preview = true
preview_scale = 4
sheet_layout = "vertical"
```

### [export.generic]

Generic JSON export configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable generic JSON export |
| `atlas_format` | string | `"json"` | Output format identifier |

```toml
[export.generic]
enabled = true
atlas_format = "json"
```

### [export.godot]

Godot game engine export configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable Godot export |
| `atlas_format` | string | `"godot"` | Output format identifier |
| `resource_path` | string | `"res://assets/sprites"` | Godot resource path prefix |
| `animation_player` | boolean | `true` | Generate AnimationPlayer resources |
| `sprite_frames` | boolean | `true` | Generate SpriteFrames resources |

```toml
[export.godot]
enabled = true
resource_path = "res://sprites"
animation_player = true
sprite_frames = true
```

**Generated files:**
- `.tres` resource files for atlases
- `AnimationPlayer` resources for animations
- `SpriteFrames` resources for animated sprites

### [export.unity]

Unity game engine export configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable Unity export |
| `atlas_format` | string | `"unity"` | Output format identifier |
| `pixels_per_unit` | integer | `16` | Pixels per Unity unit |
| `filter_mode` | string | `"point"` | Texture filter: `point` or `bilinear` |

```toml
[export.unity]
enabled = true
pixels_per_unit = 32
filter_mode = "point"
```

**Generated files:**
- `.asset` meta files for sprites
- Sprite slicing data for atlases
- Animation clips for animated sprites

### [export.libgdx]

libGDX game framework export configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable libGDX export |
| `atlas_format` | string | `"libgdx"` | Output format identifier |

```toml
[export.libgdx]
enabled = true
```

**Generated files:**
- `.atlas` files in libGDX TexturePacker format
- Region definitions for sprite lookup

### [validate]

Validation settings for the build process.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `strict` | boolean | `false` | Treat warnings as errors |
| `unused_palettes` | level | `"warn"` | How to handle unused palettes |
| `missing_refs` | level | `"error"` | How to handle missing references |

Validation levels: `error`, `warn`, `ignore`

```toml
[validate]
strict = true
unused_palettes = "error"
missing_refs = "warn"
```

### [watch]

Watch mode configuration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `debounce_ms` | integer | `100` | Debounce delay in milliseconds |
| `clear_screen` | boolean | `true` | Clear terminal between rebuilds |

```toml
[watch]
debounce_ms = 200
clear_screen = false
```

## Minimal Configuration

The simplest valid configuration:

```toml
[project]
name = "my-project"
```

This uses all default values:
- Source: `src/pxl/`
- Output: `build/`
- Scale: 1
- No exports enabled (except generic JSON)

## Environment Variables

Some settings can be overridden via environment variables or command-line flags:

| Setting | CLI Flag | Environment Variable |
|---------|----------|---------------------|
| Source directory | `--src` | - |
| Output directory | `--out` or `-o` | - |
| Verbose output | `--verbose` or `-v` | - |

## Validation

The configuration is validated on load. Common validation errors:

| Error | Cause |
|-------|-------|
| `project.name must be non-empty` | Missing or empty project name |
| `defaults.scale must be positive` | Scale set to 0 |
| `atlases.\<name\>.sources must contain at least one glob pattern` | Empty sources array |
| `atlases.\<name\>.max_size dimensions must be positive` | Zero dimension in max_size |
| `export.unity.pixels_per_unit must be positive` | Zero pixels_per_unit with Unity enabled |

## Related

- [build command](../cli/build.md) - Build system usage
- [Build System Integration](../integrations/build-system.md) - CI/CD integration
- [validate command](../cli/validate.md) - Validation options
