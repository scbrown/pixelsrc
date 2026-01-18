# init

Initialize a new Pixelsrc project.

## Usage

```
pxl init [OPTIONS] [PATH]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[PATH]` | Project directory (default: current directory) |

## Options

| Option | Description |
|--------|-------------|
| `--name <NAME>` | Project name (default: directory name) |
| `--preset <PRESET>` | Preset template: `minimal`, `artist`, `animator`, `game` (default: `minimal`) |

## Description

The `init` command creates a new Pixelsrc project with:
- A `pxl.toml` configuration file
- Directory structure based on the chosen preset
- Sample assets to get started

## Examples

### Initialize in current directory

```bash
# Minimal setup in current directory
pxl init

# With a custom project name
pxl init --name "my-game-sprites"
```

### Initialize in new directory

```bash
# Create and initialize a new directory
pxl init my-project

# With preset
pxl init my-game --preset game
```

### Presets

```bash
# Minimal: just config and one sample sprite
pxl init --preset minimal

# Artist: includes palette examples and common templates
pxl init --preset artist

# Animator: includes animation examples and frame templates
pxl init --preset animator

# Game: full setup with atlases, exports, and CI integration
pxl init --preset game
```

## Presets

### minimal

Basic setup for getting started:

```
project/
├── pxl.toml
└── src/
    └── sample.pxl
```

### artist

For sprite artists and designers:

```
project/
├── pxl.toml
├── src/
│   ├── characters/
│   ├── items/
│   └── palettes/
│       └── custom.pxl
└── build/
```

### animator

For animation-focused projects:

```
project/
├── pxl.toml
├── src/
│   ├── sprites/
│   ├── animations/
│   │   └── sample_walk.pxl
│   └── palettes/
└── build/
```

### game

Full game asset pipeline:

```
project/
├── pxl.toml
├── src/
│   ├── characters/
│   ├── enemies/
│   ├── items/
│   ├── tiles/
│   ├── ui/
│   └── palettes/
├── build/
└── .github/
    └── workflows/
        └── build.yml
```

## Generated pxl.toml

```toml
[project]
name = "my-project"
src = "src"
out = "build"

[defaults]
scale = 1
format = "png"

# Game preset includes:
[export.godot]
enabled = false
resource_path = "res://assets/sprites"

[export.unity]
enabled = false
pixels_per_unit = 16
```

## See Also

- [build](build.md) - Build project assets
- [new](new.md) - Create individual assets
- [Configuration](../reference/config.md) - Full config reference
