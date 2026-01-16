# `pxl.toml` Configuration Specification

Project-level configuration for pixelsrc build system.

## Overview

The `pxl.toml` file defines how pixelsrc sources are built, packed into atlases,
and exported to various game engine formats.

## Schema

### `[project]` Section (Required)

Core project metadata.

```toml
[project]
name = "my-game"           # Required: Project name
version = "1.0.0"          # Optional: Default "0.1.0"
src = "src/pxl"            # Optional: Source directory, default "src/pxl"
out = "build"              # Optional: Output directory, default "build"
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | **Yes** | - | Project identifier |
| `version` | string | No | `"0.1.0"` | Semantic version |
| `src` | path | No | `"src/pxl"` | Source directory for `.pxl` files |
| `out` | path | No | `"build"` | Build output directory |

---

### `[defaults]` Section (Optional)

Default settings applied to all outputs unless overridden.

```toml
[defaults]
scale = 1                  # Default scale factor
padding = 1                # Default padding between sprites
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `scale` | integer | `1` | Default sprite scale factor |
| `padding` | integer | `1` | Default padding in pixels |

---

### `[atlases.<name>]` Sections (Optional)

Define texture atlases for sprite packing.

```toml
[atlases.characters]
sources = [
    "sprites/player/**",
    "sprites/enemies/**"
]
max_size = [1024, 1024]
padding = 2
power_of_two = true

[atlases.environment]
sources = ["sprites/environment/**"]
max_size = [2048, 2048]

[atlases.ui]
sources = ["ui/**"]
max_size = [512, 512]
nine_slice = true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sources` | array[glob] | **Required** | Glob patterns for sprite sources |
| `max_size` | [u32, u32] | `[1024, 1024]` | Maximum atlas dimensions |
| `padding` | integer | from `[defaults]` | Padding between sprites |
| `power_of_two` | boolean | `false` | Constrain to power-of-two dimensions |
| `nine_slice` | boolean | `false` | Preserve nine-slice metadata |

---

### `[animations]` Section (Optional)

Animation output configuration.

```toml
[animations]
sources = ["animations/**"]
preview = true
preview_scale = 2
sheet_layout = "horizontal"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sources` | array[glob] | `["animations/**"]` | Glob patterns for animation files |
| `preview` | boolean | `false` | Generate preview GIFs |
| `preview_scale` | integer | `1` | Scale factor for previews |
| `sheet_layout` | string | `"horizontal"` | Layout: `"horizontal"`, `"vertical"`, `"grid"` |

---

### `[export.<format>]` Sections (Optional)

Format-specific export configurations.

#### Generic JSON Export

```toml
[export.generic]
enabled = true
atlas_format = "json"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable this export |
| `atlas_format` | string | `"json"` | Output format |

#### Godot Export

```toml
[export.godot]
enabled = true
atlas_format = "godot"
resource_path = "res://assets/sprites"
animation_player = true
sprite_frames = true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable Godot export |
| `atlas_format` | string | `"godot"` | Output format |
| `resource_path` | string | `"res://assets/sprites"` | Godot resource path |
| `animation_player` | boolean | `true` | Generate AnimationPlayer resources |
| `sprite_frames` | boolean | `true` | Generate SpriteFrames resources |

#### Unity Export

```toml
[export.unity]
enabled = false
atlas_format = "unity"
pixels_per_unit = 16
filter_mode = "point"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable Unity export |
| `atlas_format` | string | `"unity"` | Output format |
| `pixels_per_unit` | integer | `16` | PPU setting |
| `filter_mode` | string | `"point"` | Filter: `"point"`, `"bilinear"` |

#### libGDX Export

```toml
[export.libgdx]
enabled = false
atlas_format = "libgdx"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable libGDX export |
| `atlas_format` | string | `"libgdx"` | Output format |

---

### `[validate]` Section (Optional)

Validation settings for the build process.

```toml
[validate]
strict = true
unused_palettes = "warn"
missing_refs = "error"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `strict` | boolean | `false` | Treat warnings as errors |
| `unused_palettes` | string | `"warn"` | `"error"`, `"warn"`, `"ignore"` |
| `missing_refs` | string | `"error"` | `"error"`, `"warn"`, `"ignore"` |

---

### `[watch]` Section (Optional)

Watch mode configuration.

```toml
[watch]
debounce_ms = 100
clear_screen = true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `debounce_ms` | integer | `100` | Debounce delay in milliseconds |
| `clear_screen` | boolean | `true` | Clear terminal between rebuilds |

---

## Validation Rules

### Required Fields
- `project.name` must be a non-empty string

### Type Constraints
- `max_size` dimensions must be positive integers
- `scale` must be a positive integer
- `padding` must be a non-negative integer
- `debounce_ms` must be a non-negative integer
- `pixels_per_unit` must be a positive integer

### Enum Values
- `sheet_layout`: `"horizontal"` | `"vertical"` | `"grid"`
- `filter_mode`: `"point"` | `"bilinear"`
- `unused_palettes`, `missing_refs`: `"error"` | `"warn"` | `"ignore"`

### Path Resolution
- All paths are relative to the `pxl.toml` location
- Glob patterns use standard glob syntax (`*`, `**`, `?`)
- Source paths must exist during build

---

## Config Resolution Order

1. **CLI flags** override config values
2. **Config file** overrides defaults
3. **Built-in defaults** for unspecified values

```bash
# Uses pxl.toml
pxl build

# Override output dir
pxl build --out dist/

# Override specific atlas
pxl build --atlas characters --max-size 512x512
```

---

## Minimal Example

```toml
[project]
name = "my-game"
```

Uses defaults for everything else. Equivalent to:

```toml
[project]
name = "my-game"
version = "0.1.0"
src = "src/pxl"
out = "build"

[defaults]
scale = 1
padding = 1
```

---

## Full Example

```toml
[project]
name = "pixel-adventure"
version = "1.0.0"
src = "src/pxl"
out = "build"

[defaults]
scale = 1
padding = 1

[atlases.characters]
sources = [
    "sprites/player/**",
    "sprites/enemies/**"
]
max_size = [1024, 1024]
padding = 2
power_of_two = true

[atlases.environment]
sources = ["sprites/environment/**"]
max_size = [2048, 2048]

[atlases.ui]
sources = ["ui/**"]
max_size = [512, 512]
nine_slice = true

[animations]
sources = ["animations/**"]
preview = true
preview_scale = 2
sheet_layout = "horizontal"

[export.godot]
enabled = true
resource_path = "res://assets/sprites"
animation_player = true
sprite_frames = true

[export.unity]
enabled = false

[export.generic]
enabled = true

[validate]
strict = true
unused_palettes = "warn"
missing_refs = "error"

[watch]
debounce_ms = 100
clear_screen = true
```

---

## Error Messages

### Missing Required Fields

```
Error: pxl.toml missing required field 'project.name'
```

### Invalid Types

```
Error: pxl.toml: 'atlases.characters.max_size' must be [width, height], got "1024"
```

### Invalid Enum Values

```
Error: pxl.toml: 'validate.unused_palettes' must be "error", "warn", or "ignore", got "skip"
```

### Path Not Found

```
Warning: pxl.toml: 'atlases.characters.sources' pattern 'sprites/player/**' matches no files
```
