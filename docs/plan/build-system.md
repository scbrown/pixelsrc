---
phase: 20
title: Build System
---

# Phase 20: Build System

Project-level build configuration for pixelsrc.

**Personas:** Animator, Motion Designer, Game Developer

**Status:** Complete (2026-01-25)

> Core build system implemented:
> - `pxl build` with --watch, --dry-run, --force, incremental builds
> - `pxl init` with presets (minimal, artist, animator, game)
> - `pxl new` for scaffolding sprite/animation/palette
> - Config system (pxl.toml schema, loader, defaults)
> - Build pipeline (discovery, incremental, parallel, manifest, progress)
> - Export formats: Godot, Unity, libGDX, JSON

**Depends on:** Phase 18 (Transforms - for atlas generation with transforms applied)

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

[export.strips]
enabled = true            # Export animations as individual strip files
naming = "{name}_strip"   # e.g., "idle" -> "idle_strip.png"
layout = "horizontal"     # horizontal | vertical | grid

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
| `mod` | Character modding (RoA, FNF) | Animation strips, naming conventions, scale |
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
| `strips` | Individual PNGs | Mods (Rivals), simple engines |
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

## Task Dependency Diagram

```
                           BUILD SYSTEM TASK FLOW
═══════════════════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Phase 18 Complete                                 │
│                      (Transforms for atlas generation)                      │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 1 (Foundation - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────┐  ┌────────────────────────────────┐    │
│  │         BST-1                  │  │         BST-2                  │    │
│  │    Config Schema               │  │    Config Parser               │    │
│  │    (pxl.toml spec)             │  │    (src/config.rs)             │    │
│  │    - TOML structure            │  │    - Parse pxl.toml            │    │
│  │    - Validation rules          │  │    - Defaults handling         │    │
│  │                                │  │    - CLI override merge        │    │
│  └────────────────────────────────┘  └────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                    │
            └─────────────────┬──────────────────┘
                              ▼
WAVE 2 (Core Commands)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            BST-3                                    │    │
│  │               Build Pipeline Core                                   │    │
│  │               (pxl build basic flow)                                │    │
│  │               - Discover .pxl files                                 │    │
│  │               - Parse and validate                                  │    │
│  │               - Render to build/                                    │    │
│  │               Needs: BST-1, BST-2                                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            BST-4                                    │    │
│  │               Init Command (Minimal)                                │    │
│  │               (pxl init --preset minimal)                           │    │
│  │               - Directory scaffolding                               │    │
│  │               - Minimal pxl.toml                                    │    │
│  │               - Example sprite                                      │    │
│  │               Needs: BST-1                                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                     │
            ▼                                     │
WAVE 3 (Atlas & Export - Parallel)                │
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │    BST-5      │  │    BST-6      │  │    BST-7      │  │    BST-8      │ │
│  │ Atlas Packer  │  │ JSON Export   │  │ Watch Mode    │  │ Build         │ │
│  │  - Bin pack   │  │  - Atlas JSON │  │  - File watch │  │ Manifest      │ │
│  │  - Padding    │  │  - Sprite     │  │  - Debounce   │  │  - Stats      │ │
│  │  - Max size   │  │    coords     │  │  - Rebuild    │  │  - Checksums  │ │
│  │  Needs: BST-3 │  │  Needs: BST-5 │  │  Needs: BST-3 │  │  Needs: BST-3 │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │               │               │               │
            └───────────────┴───────────────┴───────────────┘
                              │
                              ▼
WAVE 4 (Presets & Templates - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────────────┐ ┌───────────────────────┐ ┌───────────────────┐  │
│  │       BST-9           │ │       BST-10          │ │     BST-11        │  │
│  │  Init Presets         │ │  Justfile Templates   │ │  New Scaffolding  │  │
│  │  (artist, animator,   │ │  - Standard recipes   │ │  - pxl new sprite │  │
│  │   game presets)       │ │  - Build/watch/clean  │ │  - pxl new anim   │  │
│  │  Needs: BST-4, BST-8  │ │  Needs: BST-8         │ │  Needs: BST-4     │  │
│  └───────────────────────┘ └───────────────────────┘ └───────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
            │                       │                       │
            └───────────────────────┴───────────────────────┘
                              │
                              ▼
WAVE 5 (Engine Exports - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────┐     │
│  │      BST-12        │  │      BST-13        │  │      BST-14        │     │
│  │   Godot Export     │  │   Unity Export     │  │   libGDX Export    │     │
│  │   - .tres files    │  │   - .asset files   │  │   - .atlas format  │     │
│  │   - AnimPlayer     │  │   - .meta files    │  │   - TextureRegion  │     │
│  │   - SpriteFrames   │  │   - PPU settings   │  │                    │     │
│  │   Needs: BST-6     │  │   Needs: BST-6     │  │   Needs: BST-6     │     │
│  └────────────────────┘  └────────────────────┘  └────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘
            │                       │                       │
            └───────────────────────┴───────────────────────┘
                              │
                              ▼
WAVE 6 (Polish - Parallel)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │    BST-15     │  │    BST-16     │  │    BST-17     │  │    BST-18     │ │
│  │ Incremental   │  │ Parallel      │  │ Progress      │  │ Watch Error   │ │
│  │ Builds        │  │ Builds        │  │ Reporting     │  │ Recovery      │ │
│  │  - Caching    │  │  - Rayon      │  │  - Live stats │  │  - Continue   │ │
│  │  - Hash check │  │  - Thread     │  │  - ETA        │  │    on error   │ │
│  │  Needs: BST-8 │  │    pool       │  │  Needs: BST-3 │  │  Needs: BST-7 │ │
│  │               │  │  Needs: BST-3 │  │               │  │               │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │                                               │
            └───────────────────────────────────────────────┘
                              │
                              ▼
WAVE 7 (Testing & Documentation)
┌─────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            BST-19                                   │    │
│  │                    Build System Test Suite                          │    │
│  │                    (unit + integration tests)                       │    │
│  │                    Needs: BST-12, BST-13, BST-14, BST-15-18         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                            BST-20                                   │    │
│  │                    Documentation Update                             │    │
│  │                    - prime output                                   │    │
│  │                    - format spec                                    │    │
│  │                    - demo.sh examples                               │    │
│  │                    Needs: BST-19                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Wave 1: BST-1 + BST-2                          (2 tasks in parallel)       │
│  Wave 2: BST-3 + BST-4                          (2 tasks, BST-4 after BST-1)│
│  Wave 3: BST-5 + BST-7 + BST-8 (after BST-3)    (3 tasks in parallel)       │
│          BST-6 (after BST-5)                                                │
│  Wave 4: BST-9 + BST-10 + BST-11                (3 tasks in parallel)       │
│  Wave 5: BST-12 + BST-13 + BST-14               (3 tasks in parallel)       │
│  Wave 6: BST-15 + BST-16 + BST-17 + BST-18      (4 tasks in parallel)       │
│  Wave 7: BST-19 → BST-20                        (sequential)                │
└─────────────────────────────────────────────────────────────────────────────┘

CRITICAL PATH: BST-1 → BST-3 → BST-5 → BST-6 → BST-12 → BST-19 → BST-20

BEADS CREATION ORDER:
  1. BST-1, BST-2 (no deps)
  2. BST-3 (dep: BST-1, BST-2), BST-4 (dep: BST-1)
  3. BST-5, BST-7, BST-8 (dep: BST-3)
  4. BST-6 (dep: BST-5)
  5. BST-9 (dep: BST-4, BST-8), BST-10 (dep: BST-8), BST-11 (dep: BST-4)
  6. BST-12, BST-13, BST-14 (dep: BST-6)
  7. BST-15 (dep: BST-8), BST-16 (dep: BST-3), BST-17 (dep: BST-3), BST-18 (dep: BST-7)
  8. BST-19 (dep: BST-12-18)
  9. BST-20 (dep: BST-19)
```

---

## Tasks

### Task BST-1: Config Schema Definition

**Wave:** 1 (parallel with BST-2)

Define the `pxl.toml` configuration schema and validation rules.

**Deliverables:**
- Document schema in `docs/spec/pxl-toml.md`:
  ```toml
  [project]
  name = "string"        # required
  version = "string"     # optional, default "0.1.0"
  src = "path"          # optional, default "src/pxl"
  out = "path"          # optional, default "build"

  [defaults]
  scale = 1
  padding = 1

  [atlases.<name>]
  sources = ["glob patterns"]
  max_size = [1024, 1024]
  padding = 2
  power_of_two = true

  [animations]
  sources = ["glob patterns"]
  preview = true
  preview_scale = 2

  [export.<format>]
  enabled = true
  # format-specific options

  [validate]
  strict = true
  unused_palettes = "warn"   # error | warn | ignore
  missing_refs = "error"

  [watch]
  debounce_ms = 100
  clear_screen = true
  ```

- Create `src/config/schema.rs` with validation types

**Verification:**
```bash
cargo build
cargo test config::schema
```

**Dependencies:** Phase 18 complete

---

### Task BST-2: Config Parser

**Wave:** 1 (parallel with BST-1)

Implement `pxl.toml` parsing with defaults and CLI override merging.

**Deliverables:**
- New file `src/config.rs`:
  ```rust
  pub struct ProjectConfig {
      pub project: ProjectInfo,
      pub defaults: DefaultSettings,
      pub atlases: HashMap<String, AtlasConfig>,
      pub animations: AnimationConfig,
      pub exports: HashMap<String, ExportConfig>,
      pub validate: ValidateConfig,
      pub watch: WatchConfig,
  }

  /// Load config from pxl.toml with fallback to defaults
  pub fn load_config(path: Option<&Path>) -> Result<ProjectConfig, ConfigError>

  /// Merge CLI overrides into config
  pub fn merge_cli_overrides(config: &mut ProjectConfig, cli: &CliArgs)

  /// Find pxl.toml by walking up from cwd
  pub fn find_config() -> Option<PathBuf>
  ```

- Update `src/lib.rs` to add `pub mod config;`

**Verification:**
```bash
cargo build
cargo test config
# Test: missing config uses defaults
# Test: CLI overrides take precedence
# Test: find_config walks up directories
```

**Dependencies:** Phase 18 complete

---

### Task BST-3: Build Pipeline Core

**Wave:** 2 (after BST-1, BST-2)

Implement the basic `pxl build` command flow.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Build all assets according to pxl.toml
  Build {
      /// Override output directory
      #[arg(long)]
      out: Option<PathBuf>,
      /// Specific atlas to build
      #[arg(long)]
      atlas: Option<String>,
      /// Dry run (show what would be built)
      #[arg(long)]
      dry_run: bool,
      /// Verbose output
      #[arg(short, long)]
      verbose: bool,
  }
  ```

- New file `src/build.rs`:
  ```rust
  /// Execute full build pipeline
  pub fn build(config: &ProjectConfig, options: &BuildOptions) -> Result<BuildResult, BuildError>

  /// Build pipeline stages
  pub fn discover_sources(src_dir: &Path) -> Vec<PathBuf>
  pub fn parse_sources(paths: &[PathBuf]) -> Result<Registry, ParseError>
  pub fn render_sprites(registry: &Registry, config: &ProjectConfig) -> Result<Vec<RenderedSprite>, RenderError>
  ```

**Verification:**
```bash
cd examples/project && pxl build
# Should discover .pxl files, parse, render to build/
pxl build --dry-run
# Should show what would be built without writing
```

**Dependencies:** Tasks BST-1, BST-2

---

### Task BST-4: Init Command (Minimal)

**Wave:** 2 (after BST-1)

Implement `pxl init --preset minimal` for basic project scaffolding.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Initialize a new pixelsrc project
  Init {
      /// Project directory (default: current)
      path: Option<PathBuf>,
      /// Project name
      #[arg(long)]
      name: Option<String>,
      /// Preset template
      #[arg(long, default_value = "minimal")]
      preset: String,
  }
  ```

- New file `src/init.rs`:
  ```rust
  pub fn init_project(path: &Path, name: &str, preset: &str) -> Result<(), InitError>
  ```

- Generated structure for `--preset minimal`:
  ```
  <name>/
  ├── src/pxl/
  │   ├── palettes/main.pxl
  │   └── sprites/example.pxl
  ├── build/.gitkeep
  ├── .gitignore
  └── pxl.toml
  ```

**Verification:**
```bash
pxl init my-project
cd my-project && pxl build
# Should create valid project that builds
```

**Dependencies:** Task BST-1

---

### Task BST-5: Atlas Packer

**Wave:** 3 (after BST-3)

Implement atlas packing algorithm for combining sprites into texture atlases.

**Deliverables:**
- New file `src/atlas.rs`:
  ```rust
  pub struct Atlas {
      pub name: String,
      pub image: RgbaImage,
      pub sprites: Vec<AtlasSprite>,
  }

  pub struct AtlasSprite {
      pub name: String,
      pub rect: Rect,    // x, y, w, h in atlas
      pub source: PathBuf,
  }

  /// Pack sprites into atlas using bin packing
  pub fn pack_atlas(
      sprites: &[RenderedSprite],
      config: &AtlasConfig,
  ) -> Result<Atlas, AtlasError>

  /// Bin packing algorithm (shelf or maxrects)
  fn bin_pack(sizes: &[(u32, u32)], max_size: (u32, u32)) -> Result<Vec<Rect>, PackError>
  ```

**Verification:**
```bash
cargo test atlas
pxl build --atlas characters
# Should create atlases/characters.png with packed sprites
```

**Dependencies:** Task BST-3

---

### Task BST-6: JSON Export

**Wave:** 3 (after BST-5)

Implement generic JSON export format for atlases.

**Deliverables:**
- Update `src/atlas.rs`:
  ```rust
  /// Export atlas as JSON + PNG
  pub fn export_json(atlas: &Atlas, out_dir: &Path) -> Result<(), ExportError>
  ```

- Output format `atlases/<name>.json`:
  ```json
  {
    "name": "characters",
    "image": "characters.png",
    "size": [1024, 512],
    "sprites": [
      {
        "name": "player_idle",
        "x": 0, "y": 0, "w": 16, "h": 16,
        "source": "sprites/player/idle.pxl"
      }
    ]
  }
  ```

**Verification:**
```bash
pxl build
cat build/atlases/main.json | jq '.sprites | length'
# Should show sprite count
```

**Dependencies:** Task BST-5

---

### Task BST-7: Watch Mode

**Wave:** 3 (after BST-3, parallel with BST-5, BST-8)

Add file watching for automatic rebuilds.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  Build {
      // ... existing args
      /// Watch for changes and rebuild
      #[arg(long)]
      watch: bool,
  }
  ```

- New file `src/watch.rs`:
  ```rust
  pub fn watch_and_rebuild(
      config: &ProjectConfig,
      options: &BuildOptions,
  ) -> Result<(), WatchError>
  ```

- Features:
  - Watch `src/pxl/` for changes
  - Debounce rapid changes
  - Clear screen between builds (configurable)
  - Show build time and stats

**Verification:**
```bash
pxl build --watch
# In another terminal: touch src/pxl/sprites/test.pxl
# Should see rebuild output
```

**Crate dependencies:** `notify` for file watching

**Dependencies:** Task BST-3

---

### Task BST-8: Build Manifest

**Wave:** 3 (after BST-3, parallel with BST-5, BST-7)

Generate build manifest with stats and metadata.

**Deliverables:**
- Update `src/build.rs`:
  ```rust
  pub struct BuildManifest {
      pub version: String,
      pub built_at: DateTime<Utc>,
      pub config: String,
      pub assets: AssetCounts,
      pub atlases: HashMap<String, AtlasInfo>,
      pub exports: HashMap<String, Vec<String>>,
      pub warnings: Vec<String>,
      pub duration_ms: u64,
  }

  pub fn write_manifest(manifest: &BuildManifest, out_dir: &Path) -> Result<(), IoError>
  ```

- Output `build/manifest.json`

**Verification:**
```bash
pxl build
cat build/manifest.json | jq '.duration_ms'
```

**Dependencies:** Task BST-3

---

### Task BST-9: Init Presets

**Wave:** 4 (after BST-4, BST-8)

Implement additional `pxl init` presets.

**Deliverables:**
- Update `src/init.rs` with presets:
  | Preset | Description |
  |--------|-------------|
  | `artist` | Static art workflow (palettes, sprites, variants) |
  | `animator` | Animation workflow (sprites, animations, GIF recipes) |
  | `game` | Full pipeline (atlases, multi-export, justfile) |

- Each preset generates appropriate:
  - Directory structure
  - `pxl.toml` configuration
  - Example files
  - README.md

**Verification:**
```bash
pxl init --preset artist art-project
pxl init --preset animator anim-project
pxl init --preset game game-project
# Each should create valid, buildable project
```

**Dependencies:** Tasks BST-4, BST-8

---

### Task BST-10: Justfile Templates

**Wave:** 4 (after BST-8, parallel with BST-9, BST-11)

Generate standard justfile recipes for projects.

**Deliverables:**
- Update `src/init.rs` to generate `justfile`:
  ```just
  default: build

  build: validate
      pxl build

  validate:
      pxl validate src/pxl/ --strict

  watch:
      pxl build --watch

  preview:
      pxl build --animations-only --preview

  clean:
      rm -rf build/*

  stats:
      pxl analyze src/pxl/

  fmt:
      pxl fmt src/pxl/

  fmt-check:
      pxl fmt src/pxl/ --check

  ci: fmt-check validate build
      @echo "CI build complete"
  ```

**Verification:**
```bash
pxl init --preset game test-project
cd test-project && just build
```

**Dependencies:** Task BST-8

---

### Task BST-11: New Scaffolding

**Wave:** 4 (after BST-4, parallel with BST-9, BST-10)

Add `pxl new sprite` and `pxl new animation` scaffolding commands.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Create new asset from template
  New {
      /// Asset type (sprite, animation, palette)
      asset_type: String,
      /// Asset name
      name: String,
      /// Palette to use
      #[arg(long)]
      palette: Option<String>,
  }
  ```

- New file `src/scaffold.rs`:
  ```rust
  pub fn new_sprite(name: &str, palette: Option<&str>) -> Result<PathBuf, ScaffoldError>
  pub fn new_animation(name: &str, palette: Option<&str>) -> Result<PathBuf, ScaffoldError>
  pub fn new_palette(name: &str) -> Result<PathBuf, ScaffoldError>
  ```

**Verification:**
```bash
pxl new sprite hero --palette main
# Creates src/pxl/sprites/hero.pxl with template
pxl new animation walk --palette main
# Creates src/pxl/animations/walk.pxl with template
```

**Dependencies:** Task BST-4

---

### Task BST-12: Godot Export

**Wave:** 5 (after BST-6)

Implement Godot 4.x export format.

**Deliverables:**
- New file `src/export/godot.rs`:
  ```rust
  pub fn export_godot(
      atlas: &Atlas,
      animations: &[Animation],
      config: &GodotExportConfig,
      out_dir: &Path,
  ) -> Result<(), ExportError>
  ```

- Output files:
  - `godot/sprites.tres` - SpriteFrames resource
  - `godot/animations.tres` - AnimationPlayer resources

- Config options:
  ```toml
  [export.godot]
  enabled = true
  resource_path = "res://assets/sprites"
  animation_player = true
  sprite_frames = true
  ```

**Verification:**
```bash
pxl build --export godot
# Open in Godot, verify sprites load correctly
```

**Dependencies:** Task BST-6

---

### Task BST-13: Unity Export

**Wave:** 5 (after BST-6, parallel with BST-12, BST-14)

Implement Unity export format.

**Deliverables:**
- New file `src/export/unity.rs`:
  ```rust
  pub fn export_unity(
      atlas: &Atlas,
      config: &UnityExportConfig,
      out_dir: &Path,
  ) -> Result<(), ExportError>
  ```

- Output files:
  - `unity/<name>.asset` - Texture settings
  - `unity/<name>.asset.meta` - Unity metadata

- Config options:
  ```toml
  [export.unity]
  enabled = true
  pixels_per_unit = 16
  filter_mode = "point"
  ```

**Verification:**
```bash
pxl build --export unity
# Import to Unity, verify sprites slice correctly
```

**Dependencies:** Task BST-6

---

### Task BST-14: libGDX Export

**Wave:** 5 (after BST-6, parallel with BST-12, BST-13)

Implement libGDX TextureAtlas format.

**Deliverables:**
- New file `src/export/libgdx.rs`:
  ```rust
  pub fn export_libgdx(
      atlas: &Atlas,
      config: &LibGdxExportConfig,
      out_dir: &Path,
  ) -> Result<(), ExportError>
  ```

- Output `libgdx/<name>.atlas` (text format):
  ```
  characters.png
  size: 1024,512
  format: RGBA8888
  filter: Nearest,Nearest
  repeat: none
  player_idle
    xy: 0, 0
    size: 16, 16
    orig: 16, 16
    offset: 0, 0
    index: -1
  ```

**Verification:**
```bash
pxl build --export libgdx
# Load in libGDX, verify TextureAtlas works
```

**Dependencies:** Task BST-6

---

### Task BST-15: Incremental Builds

**Wave:** 6 (after BST-8)

Add caching for unchanged assets to speed up rebuilds.

**Deliverables:**
- New file `src/cache.rs`:
  ```rust
  pub struct BuildCache {
      entries: HashMap<PathBuf, CacheEntry>,
  }

  pub struct CacheEntry {
      hash: String,
      output: PathBuf,
      timestamp: SystemTime,
  }

  /// Check if source needs rebuild
  pub fn needs_rebuild(source: &Path, cache: &BuildCache) -> bool

  /// Update cache after successful build
  pub fn update_cache(source: &Path, output: &Path, cache: &mut BuildCache)
  ```

- Store cache in `build/.cache.json`

**Verification:**
```bash
pxl build  # Full build
pxl build  # Should be much faster (cached)
touch src/pxl/sprites/one.pxl
pxl build  # Should only rebuild changed sprite
```

**Dependencies:** Task BST-8

---

### Task BST-16: Parallel Builds

**Wave:** 6 (after BST-3, parallel with BST-15, BST-17, BST-18)

Add multi-threaded rendering and packing.

**Deliverables:**
- Update `src/build.rs`:
  ```rust
  /// Render sprites in parallel
  pub fn render_sprites_parallel(
      sprites: &[SpriteRef],
      thread_count: usize,
  ) -> Result<Vec<RenderedSprite>, RenderError>
  ```

- Use `rayon` for parallel iteration
- Configurable thread pool size

**Verification:**
```bash
time pxl build --jobs 1  # Single-threaded
time pxl build --jobs 8  # Parallel
# Parallel should be significantly faster for large projects
```

**Crate dependencies:** `rayon`

**Dependencies:** Task BST-3

---

### Task BST-17: Progress Reporting

**Wave:** 6 (after BST-3, parallel with BST-15, BST-16, BST-18)

Add live progress display during builds.

**Deliverables:**
- New file `src/progress.rs`:
  ```rust
  pub struct BuildProgress {
      total: usize,
      completed: usize,
      current: String,
      start_time: Instant,
  }

  pub fn create_progress_bar(total: usize) -> ProgressBar
  pub fn update_progress(bar: &ProgressBar, msg: &str)
  ```

- Features:
  - Show current file being processed
  - Progress bar with percentage
  - Elapsed time
  - ETA for completion

**Verification:**
```bash
pxl build
# Should show progress: [=====>    ] 50% Rendering player_walk.pxl (12s / ~24s)
```

**Crate dependencies:** `indicatif`

**Dependencies:** Task BST-3

---

### Task BST-18: Watch Error Recovery

**Wave:** 6 (after BST-7, parallel with BST-15, BST-16, BST-17)

Improve watch mode error handling.

**Deliverables:**
- Update `src/watch.rs`:
  - Continue watching after parse/render errors
  - Display errors clearly
  - Recover when errors are fixed
  - Don't crash on invalid files

- Error display:
  ```
  [10:30:22] Error in sprites/broken.pxl:
            Line 5: Invalid color format "#GGG"
  [10:30:22] Watching for changes...
  [10:30:45] Fixed: sprites/broken.pxl
  [10:30:45] Building... done (124ms)
  ```

**Verification:**
```bash
pxl build --watch
# Add syntax error to file - should show error, keep watching
# Fix error - should rebuild successfully
```

**Dependencies:** Task BST-7

---

### Task BST-19: Build System Test Suite

**Wave:** 7 (after BST-12 through BST-18)

Comprehensive tests for all build system functionality.

**Deliverables:**
- `tests/config_tests.rs`:
  - Config parsing
  - Default handling
  - CLI override merging

- `tests/build_tests.rs`:
  - Full build pipeline
  - Atlas packing
  - Export formats

- `tests/cli_integration.rs` additions:
  - `pxl build` command tests
  - `pxl init` command tests
  - `pxl new` command tests

- Test fixtures:
  - `tests/fixtures/projects/minimal/`
  - `tests/fixtures/projects/game/`

**Verification:**
```bash
cargo test config
cargo test build
cargo test atlas
cargo test export
cargo test --test cli_integration build
cargo test --test cli_integration init
```

**Dependencies:** Tasks BST-12 through BST-18

---

### Task BST-20: Documentation Update

**Wave:** 7 (after BST-19)

Update all documentation for build system feature.

**Deliverables:**
- Update `src/prime.rs`:
  - Add `pxl build`, `pxl init`, `pxl new` to command list
  - Add project workflow examples

- Update `docs/spec/format.md`:
  - Add `pxl.toml` specification reference

- Create `docs/guides/project-setup.md`:
  - Getting started guide
  - Preset explanations
  - CI/CD integration

- Update `demo.sh`:
  ```bash
  # Project initialization
  pxl init --preset game demo-game
  cd demo-game

  # Build all assets
  pxl build

  # Watch mode
  pxl build --watch &

  # Export for specific engine
  pxl build --export godot
  ```

**Verification:**
```bash
pxl prime | grep build
pxl prime | grep init
./demo.sh  # Should run without errors
```

**Dependencies:** Task BST-19

---

## Verification Summary

```bash
# 1. All existing tests pass
cargo test

# 2. Config parsing works
cargo test config

# 3. Build pipeline works
pxl init --preset game test-project
cd test-project && pxl build

# 4. Atlas generation works
pxl build --atlas main
cat build/atlases/main.json | jq '.sprites | length'

# 5. Watch mode works
pxl build --watch &
touch src/pxl/sprites/test.pxl
# Should see rebuild

# 6. Export formats work
pxl build --export godot
pxl build --export unity
pxl build --export libgdx

# 7. Init presets work
pxl init --preset minimal min-proj
pxl init --preset artist art-proj
pxl init --preset animator anim-proj
pxl init --preset game game-proj

# 8. Documentation updated
pxl prime | grep build
```

---

## Success Criteria

1. `pxl.toml` configuration works with sensible defaults
2. `pxl build` discovers, parses, renders, and packs assets
3. Atlas packing produces efficient texture atlases
4. Watch mode rebuilds on file changes
5. All engine export formats produce valid output
6. `pxl init` creates buildable projects for all presets
7. Incremental builds skip unchanged assets
8. Parallel builds utilize multiple cores
9. Progress reporting shows build status
10. All tests pass
11. Documentation reflects new capabilities

---

## Crate Dependencies

| Crate | Purpose |
|-------|---------|
| `toml` | Config file parsing |
| `notify` | File system watching |
| `rayon` | Parallel iteration |
| `indicatif` | Progress bars |

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
