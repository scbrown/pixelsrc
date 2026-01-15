# Build System

Project-level build configuration for pixelsrc.

**Personas:** Animator, Motion Designer, Game Developer

**Related:**
- [Personas](../personas.md) - Target workflows section
- [Advanced Transforms](./advanced-transforms.md) - Atlas export

---

## Overview

| Component | Purpose |
|-----------|---------|
| `pxl.toml` | Project configuration |
| `pxl init` | Generate starter project |
| `pxl build` | Build all assets from config |
| `justfile` | Standard build recipes |

---

## Directory Convention

```
my-project/
├── src/
│   └── pxl/                    # All pixelsrc sources
│       ├── palettes/           # Shared color definitions
│       │   ├── characters.pxl
│       │   └── environment.pxl
│       ├── sprites/            # Static sprites
│       │   ├── player/
│       │   ├── enemies/
│       │   └── items/
│       ├── animations/         # Animation definitions
│       │   └── player.pxl
│       ├── transforms/         # Reusable transform definitions
│       │   └── effects.pxl
│       └── ui/                 # UI elements (nine-slice, etc.)
│           ├── buttons.pxl
│           └── panels.pxl
├── build/                      # Generated output (gitignored)
│   ├── atlases/
│   │   ├── characters.png
│   │   ├── characters.json
│   │   └── ...
│   ├── animations/
│   │   └── *.gif              # Preview GIFs
│   └── manifest.json          # Build manifest
├── pxl.toml                   # Project configuration
└── justfile                   # Build commands
```

**Conventions:**
- All source under `src/pxl/`
- Output to `build/` (gitignored)
- Config at project root
- Organize by asset type, then by game entity

---

## `pxl.toml` Specification

### Minimal Config

```toml
[project]
name = "my-game"
```

Uses defaults for everything else.

### Full Config

```toml
[project]
name = "my-game"
version = "1.0.0"
src = "src/pxl"           # Default: src/pxl
out = "build"             # Default: build

# Default settings for all outputs
[defaults]
scale = 1
padding = 1

# Atlas definitions
[atlases.characters]
sources = [
    "sprites/player/**",
    "sprites/enemies/**"
]
max_size = [1024, 1024]
padding = 2               # Override default
power_of_two = true

[atlases.environment]
sources = ["sprites/environment/**"]
max_size = [2048, 2048]

[atlases.ui]
sources = ["ui/**"]
max_size = [512, 512]
nine_slice = true         # Preserve nine-slice metadata

# Animation output settings
[animations]
sources = ["animations/**"]
preview = true            # Generate preview GIFs
preview_scale = 2
sheet_layout = "horizontal"

# Export format configurations
[export.godot]
enabled = true
atlas_format = "godot"
resource_path = "res://assets/sprites"
animation_player = true   # Generate AnimationPlayer resources

[export.unity]
enabled = false
atlas_format = "unity"
# resources_path = "Assets/Sprites"

[export.generic]
enabled = true
atlas_format = "json"     # Generic JSON + PNG

# Validation settings
[validate]
strict = true
unused_palettes = "warn"  # error | warn | ignore
missing_refs = "error"

# Watch mode settings
[watch]
debounce_ms = 100
clear_screen = true
```

### Config Resolution

1. CLI flags override config
2. Config overrides defaults
3. Sensible defaults for everything

```bash
# Uses pxl.toml
pxl build

# Override output dir
pxl build --out dist/

# Override specific atlas
pxl build --atlas characters --max-size 512x512
```

---

## `pxl init` Command

Generate starter project structure.

### Usage

```bash
# Interactive mode
pxl init

# With options
pxl init --name my-game --preset game

# In existing directory
pxl init .
```

### Presets

```bash
pxl init --preset <preset>
```

| Preset | Description | Creates |
|--------|-------------|---------|
| `minimal` | Single sprite, basic render | `src/pxl/sprites/`, minimal `pxl.toml` |
| `artist` | Static art workflow | Palettes, sprites, variants structure |
| `animator` | Animation workflow | Sprites, animations, justfile with GIF recipes |
| `game` | Full game asset pipeline | Full structure, atlases, multi-export |

### Generated Files

**`pxl init --preset game`:**

```
my-game/
├── src/
│   └── pxl/
│       ├── palettes/
│       │   └── main.pxl
│       ├── sprites/
│       │   └── example.pxl
│       ├── animations/
│       │   └── .gitkeep
│       └── ui/
│           └── .gitkeep
├── build/
│   └── .gitkeep
├── .gitignore
├── pxl.toml
├── justfile
└── README.md
```

**Generated `pxl.toml`:**

```toml
[project]
name = "my-game"
version = "0.1.0"

[atlases.main]
sources = ["sprites/**"]
max_size = [1024, 1024]

[animations]
sources = ["animations/**"]
preview = true

[export.generic]
enabled = true
```

**Generated `justfile`:**

```just
# Pixelsrc build commands

default: build

# Build all assets
build: validate
    pxl build

# Validate all source files
validate:
    pxl validate src/pxl/ --strict

# Watch for changes and rebuild
watch:
    pxl build --watch

# Generate preview GIFs for all animations
preview:
    pxl build --animations-only --preview

# Clean build directory
clean:
    rm -rf build/*

# Show project stats
stats:
    pxl analyze src/pxl/

# Format all source files
fmt:
    pxl fmt src/pxl/

# Check formatting without changes
fmt-check:
    pxl fmt src/pxl/ --check
```

**Generated `.gitignore`:**

```gitignore
# Pixelsrc build output
build/

# OS files
.DS_Store
Thumbs.db
```

**Generated example `src/pxl/palettes/main.pxl`:**

```pxl
{
  "type": "palette",
  "name": "main",
  "colors": {
    "{_}": "#00000000",
    "{black}": "#000000",
    "{white}": "#FFFFFF",
    "{primary}": "#4A90D9",
    "{secondary}": "#D94A4A"
  }
}
```

**Generated example `src/pxl/sprites/example.pxl`:**

```pxl
{
  "type": "sprite",
  "name": "example",
  "palette": "main",
  "grid": [
    "{_}{primary}{primary}{_}",
    "{primary}{white}{white}{primary}",
    "{primary}{white}{white}{primary}",
    "{_}{primary}{primary}{_}"
  ]
}
```

---

## `pxl build` Command

Build all assets according to `pxl.toml`.

### Usage

```bash
# Build everything
pxl build

# Watch mode
pxl build --watch

# Specific atlas only
pxl build --atlas characters

# Specific export format only
pxl build --export godot

# Dry run (show what would be built)
pxl build --dry-run

# Verbose output
pxl build --verbose
```

### Build Process

1. **Discover** - Find all `.pxl` files under `src`
2. **Parse** - Load and validate all sources
3. **Resolve** - Resolve references (palettes, sprites, transforms)
4. **Transform** - Apply transforms
5. **Render** - Generate images
6. **Pack** - Create atlases
7. **Export** - Write format-specific outputs
8. **Manifest** - Write build manifest

### Build Manifest

`build/manifest.json`:

```json
{
  "version": "1.0.0",
  "built_at": "2024-01-15T10:30:00Z",
  "config": "pxl.toml",
  "assets": {
    "sprites": 47,
    "animations": 12,
    "atlases": 3
  },
  "atlases": {
    "characters": {
      "file": "atlases/characters.png",
      "size": [1024, 512],
      "sprites": 24
    },
    "environment": {
      "file": "atlases/environment.png",
      "size": [2048, 1024],
      "sprites": 18
    }
  },
  "exports": {
    "godot": {
      "files": ["godot/sprites.tres", "godot/animations.tres"]
    },
    "generic": {
      "files": ["atlases/*.json"]
    }
  },
  "warnings": [],
  "duration_ms": 342
}
```

### Watch Mode

```bash
pxl build --watch
```

- Watches `src/pxl/` for changes
- Debounces rapid changes
- Incremental rebuild (only changed assets)
- Clear screen between builds (configurable)
- Shows build time and stats

```
[10:30:15] Watching src/pxl/ for changes...
[10:30:22] Changed: sprites/player/idle.pxl
[10:30:22] Building... done (124ms)
           Sprites: 47 | Animations: 12 | Atlases: 3
[10:31:05] Changed: palettes/main.pxl
[10:31:05] Building... done (342ms) [full rebuild - palette changed]
           Sprites: 47 | Animations: 12 | Atlases: 3
```

---

## Standard `justfile` Recipes

Recommended recipe names for consistency across projects:

| Recipe | Purpose |
|--------|---------|
| `default` | Alias for `build` |
| `build` | Build all assets |
| `validate` | Validate sources |
| `watch` | Watch mode |
| `preview` | Generate animation previews |
| `clean` | Remove build directory |
| `stats` | Show project statistics |
| `fmt` | Format source files |
| `fmt-check` | Check formatting |

### Extended Recipes

```just
# Full CI build
ci: fmt-check validate build
    @echo "CI build complete"

# Build specific atlas
atlas name:
    pxl build --atlas {{name}}

# Build for specific export target
export target:
    pxl build --export {{target}}

# Open build directory
open:
    open build/

# Create new sprite (interactive)
new-sprite name:
    pxl new sprite {{name}}

# Create new animation (interactive)
new-animation name:
    pxl new animation {{name}}
```

---

## Multi-Format Export

Single source, multiple engine outputs.

### Supported Formats

| Format | Output | Use Case |
|--------|--------|----------|
| `generic` | JSON + PNG | Engine-agnostic, custom loaders |
| `godot` | `.tres` resources | Godot 4.x |
| `unity` | `.asset` + meta | Unity |
| `libgdx` | `.atlas` text format | libGDX |
| `aseprite` | `.aseprite` | Editing in Aseprite |
| `pyxel` | Pyxel Edit format | Editing in Pyxel Edit |

### Format-Specific Config

```toml
[export.godot]
enabled = true
resource_path = "res://assets/sprites"
animation_player = true
sprite_frames = true

[export.unity]
enabled = true
pixels_per_unit = 16
filter_mode = "point"
```

### Output Structure

```
build/
├── atlases/              # Generic PNG + JSON (always generated)
│   ├── characters.png
│   ├── characters.json
│   └── ...
├── godot/                # Godot-specific exports
│   ├── sprites.tres
│   └── animations.tres
├── unity/                # Unity-specific exports
│   ├── characters.asset
│   └── characters.asset.meta
└── manifest.json
```

---

## Tasks

### Phase 1: Core Build System
1. Define `pxl.toml` schema
2. Implement config parsing
3. Implement `pxl build` basic flow
4. Implement `pxl init --preset minimal`

### Phase 2: Atlas & Export
5. Implement atlas packing algorithm
6. Implement generic JSON export
7. Add `--watch` mode
8. Generate build manifest

### Phase 3: Presets & Templates
9. Implement `pxl init` presets (artist, animator, game)
10. Generate `justfile` templates
11. Add `pxl new sprite/animation` scaffolding

### Phase 4: Engine Exports
12. Implement Godot export format
13. Implement Unity export format
14. Implement libGDX export format

### Phase 5: Polish
15. Incremental builds (cache unchanged assets)
16. Parallel builds
17. Build progress reporting
18. Error recovery in watch mode

---

## Open Questions

1. **Config inheritance:** Should `pxl.toml` support extending a base config?
   ```toml
   extends = "../shared/pxl-base.toml"
   ```

2. **Monorepo support:** Multiple `pxl.toml` files in subdirectories?

3. **Lock file:** `pxl.lock` to track exact versions/hashes for reproducible builds?

4. **Remote palettes:** Import palettes from URL?
   ```toml
   [palettes]
   lospec = "https://lospec.com/palette-list/pico-8.toml"
   ```

5. **Plugin system:** Custom export formats via plugins?
