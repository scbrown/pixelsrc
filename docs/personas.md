# Pixelsrc User Personas

Understanding who uses pixelsrc helps us design features at the right complexity level.

## Core Principle: AI-Assisted at Every Level

All personas benefit from AI assistance. Pixelsrc is designed to be AI-friendly across the entire complexity spectrum:

| Persona | AI Assistance |
|---------|---------------|
| **Sketcher** | AI generates complete sprites from descriptions |
| **Pixel Artist** | AI suggests palette harmonies, creates variants, composes scenes |
| **Animator** | AI generates walk cycles, suggests frame timing, creates mirrored versions |
| **Motion Designer** | AI writes keyframe expressions, suggests easing, designs transform sequences |
| **Game Developer** | AI generates atlas configs, hitbox suggestions, tag structures |

The format's design choices (token-based grids, JSON structure, semantic naming) make it readable and writable by both humans and AI at every complexity level. AI isn't just for beginners—it's a creative partner for experts too.

## File Extension

**Use `.pxl`** for all pixelsrc files.

The `.jsonl` extension is deprecated. While still supported for backwards compatibility, new projects should use `.pxl` exclusively. The `.pxl` format supports both single-line JSONL and multi-line JSON for readability.

---

## Persona Overview

| Persona | Goal | Complexity | Key Features |
|---------|------|------------|--------------|
| **Sketcher** | Quick sprites, prototypes | Minimal | Basic sprites, inline palettes |
| **Pixel Artist** | Polished static art | Low-Medium | Named palettes, variants, compositions |
| **Animator** | Simple animations | Medium | Animations, spritesheets, basic transforms |
| **Motion Designer** | Complex animated effects | High | Keyframes, expressions, user-defined transforms |
| **Game Developer** | Integration-ready assets | Medium-High | Frame tags, nine-slice, metadata, export formats |

---

## Sketcher

**Who:** Hobbyists, people learning pixel art, quick prototypers, AI-assisted generation.

**Goal:** Get pixels on screen fast. Minimal ceremony.

**Workflow:**
1. Write a simple sprite with inline palette
2. Render to PNG
3. Done

**Cares about:**
- Simplicity
- Fast iteration
- Forgiving errors (lenient mode)

**Doesn't care about:**
- Reusability
- Advanced features
- Optimization

**Example workflow:**
```json
{"type": "sprite", "name": "star", "palette": {"{_}": "#0000", "{y}": "#FFD700"}, "grid": [
  "{_}{y}{_}",
  "{y}{y}{y}",
  "{_}{y}{_}"
]}
```
```bash
pxl render star.pxl -o star.png
```

**Features used:**
- `sprite` with inline `palette`
- `pxl render`
- Lenient mode (default)

---

## Pixel Artist

**Who:** Artists creating polished static pixel art, icon designers, game artists making assets.

**Goal:** Create beautiful, organized, maintainable pixel art.

**Workflow:**
1. Define shared palettes
2. Create base sprites
3. Create variants with palette swaps
4. Compose scenes from sprites
5. Export at various scales

**Cares about:**
- Color consistency (shared palettes)
- Variations without duplication
- Composition tools
- Visual verification (`pxl show`)
- Clean, organized source files

**Doesn't care about:**
- Animation
- Complex transforms
- Programmatic generation

**Example workflow:**
```json
{"type": "palette", "name": "forest", "colors": {"{_}": "#0000", "{trunk}": "#4A3728", "{leaf}": "#228B22", "{leaf_light}": "#32CD32"}}
{"type": "sprite", "name": "tree", "palette": "forest", "grid": ["..."]}
{"type": "variant", "name": "tree_autumn", "source": "tree", "palette": {"{leaf}": "#FF8C00", "{leaf_light}": "#FFD700"}}
{"type": "composition", "name": "forest_scene", "size": [64, 32], "layers": [
  {"sprite": "tree", "position": [0, 0]},
  {"sprite": "tree_autumn", "position": [24, 0]},
  {"sprite": "tree", "position": [48, 0]}
]}
```

**Features used:**
- Named `palette`
- `variant` with palette overrides
- `composition`
- `pxl show` for terminal preview
- Scale factor on export

**Would benefit from:**
- Color ramps (auto-generate shades)
- Hue-shifted shadows
- Dithering patterns
- Basic transforms (mirror, rotate)

---

## Animator

**Who:** Game developers, artists creating animated sprites, people making GIFs.

**Goal:** Create smooth, looping animations with minimal frame duplication.

**Workflow:**
1. Create key frame sprites
2. Define animation sequences
3. Use basic transforms (mirror, pingpong) to reduce work
4. Export as spritesheet or GIF

**Cares about:**
- Reducing repetitive work
- Smooth loops
- Spritesheet export
- Frame timing

**Doesn't care about:**
- Mathematical expressions
- Complex procedural animation
- Physics simulation

**Example workflow:**
```json
{"type": "sprite", "name": "walk_1", "palette": "player", "grid": ["..."]}
{"type": "sprite", "name": "walk_2", "palette": "player", "grid": ["..."]}
{"type": "sprite", "name": "walk_3", "palette": "player", "grid": ["..."]}
{"type": "animation", "name": "walk_right", "frames": ["walk_1", "walk_2", "walk_3", "walk_2"], "fps": 8}
{"type": "animation", "name": "walk_left", "source": "walk_right", "transform": ["mirror-h"]}
```

**Features used:**
- `animation` with frames
- Basic transforms: `mirror-h`, `pingpong`
- Animation `source` references
- Spritesheet export

**Would benefit from:**
- Frame tags (idle, walk, attack ranges)
- Hold frames
- Palette cycling (for effects)
- Onion skinning preview

---

## Motion Designer

**Who:** Advanced animators, VFX artists, procedural animation enthusiasts, tool builders.

**Goal:** Create complex, expressive animations with minimal manual frame work.

**Workflow:**
1. Define reusable transform patterns
2. Use keyframes and easing for smooth motion
3. Compose multiple effects
4. Use expressions for precise control
5. Generate many variations programmatically

**Cares about:**
- Expressiveness
- Reusability
- Mathematical precision
- Composability
- Procedural generation

**Doesn't care about:**
- Simplicity (willing to learn)
- Beginner-friendliness

**Example workflow:**
```json
{"type": "transform", "name": "bounce", "frames": 8, "keyframes": [
  {"frame": 0, "shift-y": 0},
  {"frame": 4, "shift-y": -6},
  {"frame": 8, "shift-y": 0}
], "easing": "ease-out"}

{"type": "transform", "name": "spiral-in", "params": ["radius", "decay", "spin"], "frames": 16, "keyframes": {
  "shift-x": {"expr": "${radius} * pow(${decay}, frame) * cos(frame * ${spin})"},
  "shift-y": {"expr": "${radius} * pow(${decay}, frame) * sin(frame * ${spin})"}
}}

{"type": "animation", "name": "coin_collect", "source": "coin", "transform": [
  {"op": "spiral-in", "radius": 12, "decay": 0.9, "spin": 0.4},
  "fade-out"
]}
```

**Features used:**
- User-defined transforms
- Keyframe animation
- Easing functions
- Mathematical expressions
- Parameterized transforms
- Transform composition

**Would benefit from:**
- Squash & stretch
- Secondary motion / follow-through
- Physics expressions (gravity, springs)
- Particle system definitions
- Complex blend modes

---

## Game Developer

**Who:** Indie devs, game studios, anyone integrating pixelsrc into a game pipeline.

**Goal:** Create game-ready assets with proper metadata for engine integration.

**Workflow:**
1. Create sprites and animations
2. Add game-specific metadata (hitboxes, anchor points)
3. Tag animation states
4. Export in engine-compatible format
5. Use nine-slice for UI elements

**Cares about:**
- Engine integration
- Metadata (hitboxes, origins, tags)
- Efficient exports (atlases, spritesheets)
- Scalable UI (nine-slice)
- Automation / CI pipeline

**Doesn't care about:**
- Terminal preview
- Artistic experimentation
- Maximum expressiveness

**Example workflow:**
```json
{"type": "sprite", "name": "button", "palette": "ui", "nine_slice": {"left": 4, "right": 4, "top": 4, "bottom": 4}, "grid": ["..."]}

{"type": "animation", "name": "player", "frames": ["idle_1", "idle_2", "run_1", "run_2", "run_3", "jump"], "fps": 10, "tags": {
  "idle": {"start": 0, "end": 1, "loop": true},
  "run": {"start": 2, "end": 4, "loop": true},
  "jump": {"start": 5, "end": 5, "loop": false}
}, "metadata": {
  "origin": [8, 16],
  "hitbox": {"x": 2, "y": 0, "w": 12, "h": 16}
}}
```

**Features used:**
- `nine_slice` for UI
- Animation `tags`
- `metadata` for engine integration
- Spritesheet / atlas export
- `pxl` in CI pipelines

**Would benefit from:**
- Multiple hitbox types (hurt, hit, trigger)
- Per-frame metadata
- Export format options (JSON atlas, Aseprite, etc.)
- Parallax layer hints
- Batch processing

---

## Feature Complexity Matrix

Shows which features each persona typically uses:

| Feature | Sketcher | Pixel Artist | Animator | Motion Designer | Game Dev |
|---------|:--------:|:------------:|:--------:|:---------------:|:--------:|
| Inline palette | ✓ | | | | |
| Named palette | | ✓ | ✓ | ✓ | ✓ |
| Variant | | ✓ | ✓ | ✓ | ✓ |
| Composition | | ✓ | | ✓ | ✓ |
| Animation | | | ✓ | ✓ | ✓ |
| Basic transforms | | ✓ | ✓ | ✓ | ✓ |
| Keyframes | | | | ✓ | |
| Expressions | | | | ✓ | |
| User-defined transforms | | | | ✓ | |
| Nine-slice | | | | | ✓ |
| Frame tags | | | | | ✓ |
| Metadata | | | | | ✓ |
| Color ramps | | ✓ | | ✓ | |
| Palette cycling | | | ✓ | ✓ | ✓ |
| Dithering | | ✓ | | | |
| Blend modes | | ✓ | | ✓ | |

---

## Design Principles by Persona

### For Sketcher
- Zero configuration required
- Inline everything
- Lenient by default
- AI-friendly format

### For Pixel Artist
- Encourage organization (named palettes)
- Make variations easy (variants)
- Visual tools (`pxl show`)
- Don't require animation knowledge

### For Animator
- Minimize frame duplication
- Smart defaults (pingpong, mirror)
- Frame-level control when needed
- Good spritesheet export

### For Motion Designer
- Maximum expressiveness
- Composability
- Reusable abstractions
- Don't hide complexity

### For Game Developer
- Metadata support
- Export flexibility
- Pipeline-friendly (stdin/stdout, JSON output)
- Engine-agnostic but practical

---

## Target Workflows

Realistic workflows for each persona, from one-shot to full CI/CD.

### Sketcher Workflow: One-Shot Generation

Single command, single file, immediate result.

```bash
# AI generates sprite, render immediately
echo '{"type":"sprite","name":"star","palette":{"{_}":"#0000","{y}":"#FFD700"},"grid":["{_}{y}{_}","{y}{y}{y}","{_}{y}{_}"]}' | pxl render --stdin -o star.png
```

**Characteristics:**
- No project structure
- Inline everything
- Ephemeral (regenerate rather than edit)
- AI does most of the work

---

### Pixel Artist Workflow: Organized Asset Collection

Multiple related files with shared resources.

```
my-icons/
├── palettes/
│   └── brand.pxl        # Shared brand colors
├── icons/
│   ├── home.pxl         # References brand palette
│   ├── settings.pxl
│   └── user.pxl
├── variants/
│   └── dark-mode.pxl    # Palette overrides for dark theme
└── render.sh              # Batch render script
```

**render.sh:**
```bash
#!/bin/bash
for icon in icons/*.pxl; do
  name=$(basename "$icon" .pxl)
  pxl render palettes/brand.pxl "$icon" -o "out/${name}.png" --scale 2
  pxl render palettes/brand.pxl variants/dark-mode.pxl "$icon" -o "out/${name}_dark.png" --scale 2
done
```

**Characteristics:**
- Logical file organization
- Shared palettes across files
- Batch processing
- Multiple output variants

---

### Animator Workflow: Sprite Sheet Production

Animation-focused with frame management.

```
player/
├── src/
│   └── pxl/
│       ├── palettes.pxl
│       ├── sprites/
│       │   ├── idle_1.pxl
│       │   ├── idle_2.pxl
│       │   ├── run_1.pxl
│       │   ├── run_2.pxl
│       │   ├── run_3.pxl
│       │   └── run_4.pxl
│       └── animations.pxl
├── build/
├── pxl.toml
└── justfile
```

**src/pxl/animations.pxl:**
```json
{"type": "animation", "name": "idle", "frames": ["idle_1", "idle_2"], "fps": 4}
{"type": "animation", "name": "run_right", "frames": ["run_1", "run_2", "run_3", "run_4"], "fps": 10}
{"type": "animation", "name": "run_left", "source": "run_right", "transform": ["mirror-h"]}
```

**pxl.toml:**
```toml
[project]
name = "player"

[animations]
sources = ["animations.pxl"]
preview = true
```

**justfile:**
```just
default: build

build:
    pxl build

watch:
    pxl build --watch

clean:
    rm -rf build/
```

**Characteristics:**
- Separate sprite files for editability
- Animations reference sprites by name
- Transforms reduce duplication (mirror for left/right)
- `just` for simple commands, `pxl build` handles dependencies

---

### Motion Designer Workflow: Effect Library

Reusable transforms and procedural animations.

```
effects-library/
├── src/
│   └── pxl/
│       ├── transforms/       # Reusable transform definitions
│       │   ├── easing.pxl
│       │   ├── motion.pxl
│       │   └── particles.pxl
│       ├── sprites/
│       │   └── *.pxl
│       └── demos/
│           ├── coin_collect.pxl
│           ├── enemy_death.pxl
│           └── powerup.pxl
├── build/
├── pxl.toml
└── justfile
```

**src/pxl/transforms/motion.pxl:**
```json
{"type": "transform", "name": "spiral-in", "params": ["radius", "decay", "spin"], "frames": 16, "keyframes": {
  "shift-x": {"expr": "${radius} * pow(${decay}, frame) * cos(frame * ${spin})"},
  "shift-y": {"expr": "${radius} * pow(${decay}, frame) * sin(frame * ${spin})"}
}}
{"type": "transform", "name": "shake", "params": ["intensity"], "cycle": [
  ["shift:${intensity},0"], ["shift:-${intensity},0"], ["shift:0,${intensity}"], ["shift:0,-${intensity}"]
]}
{"type": "transform", "name": "bounce-in", "frames": 12, "keyframes": [
  {"frame": 0, "scale": 0, "shift-y": -20},
  {"frame": 8, "scale": 1.2, "shift-y": 0},
  {"frame": 12, "scale": 1, "shift-y": 0}
], "easing": "bounce"}
```

**src/pxl/demos/coin_collect.pxl:**
```json
{"type": "animation", "name": "coin_collect", "source": "coin", "transform": [
  {"op": "spiral-in", "radius": 12, "decay": 0.9, "spin": 0.5},
  "fade-out"
]}
```

**Characteristics:**
- Library of reusable transforms under `transforms/`
- Parameterized for flexibility
- Demos compose library pieces
- Standard `src/pxl/` convention

---

### Game Developer Workflow: Full CI/CD Pipeline

Production-grade asset pipeline with multiple output formats.

```
game-assets/
├── src/
│   └── pxl/
│       ├── palettes/
│       │   ├── characters.pxl
│       │   ├── environment.pxl
│       │   └── ui.pxl
│       ├── sprites/
│       │   ├── player/
│       │   │   ├── idle.pxl
│       │   │   ├── run.pxl
│       │   │   └── attack.pxl
│       │   ├── enemies/
│       │   │   └── *.pxl
│       │   └── items/
│       │       └── *.pxl
│       ├── ui/
│       │   ├── buttons.pxl      # nine-slice buttons
│       │   ├── panels.pxl
│       │   └── hud.pxl
│       └── animations/
│           ├── player.pxl       # animation defs + tags
│           └── effects.pxl
├── build/
│   ├── atlases/
│   ├── animations/
│   └── manifest.json
├── .github/
│   └── workflows/
│       └── assets.yml           # CI/CD pipeline (yaml required by GitHub)
├── pxl.toml
└── justfile
```

**pxl.toml:**
```toml
[project]
name = "game-assets"
version = "1.0.0"

[atlases.characters]
sources = ["sprites/player/**", "sprites/enemies/**"]
max_size = [1024, 1024]
padding = 1

[atlases.ui]
sources = ["ui/**"]
max_size = [512, 512]

[animations]
sources = ["animations/**"]
preview = true

[export.godot]
enabled = true
resource_path = "res://assets/sprites"

[export.generic]
enabled = true
```

**justfile:**
```just
default: build

# Build all assets
build: validate
    pxl build

# Validate all sources
validate:
    pxl validate src/pxl/ --strict

# Watch for changes
watch:
    pxl build --watch

# Clean build output
clean:
    rm -rf build/

# CI build (used by GitHub Actions)
ci: validate build
    @echo "CI build complete"
```

**.github/workflows/assets.yml:** *(GitHub requires yaml)*
```yaml
name: Build Assets

on:
  push:
    paths: ['src/pxl/**', 'pxl.toml']
  pull_request:
    paths: ['src/pxl/**']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install tools
        run: |
          cargo install pixelsrc
          cargo install just

      - name: Build
        run: just ci

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: game-assets
          path: build/

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        run: |
          echo "## Asset Build Report" >> $GITHUB_STEP_SUMMARY
          echo "- Assets: $(cat build/manifest.json | jq '.assets | length')" >> $GITHUB_STEP_SUMMARY
          echo "- Atlases: $(ls build/atlases/*.png | wc -l)" >> $GITHUB_STEP_SUMMARY
```

**Characteristics:**
- Hierarchical `src/pxl/` organization
- `pxl.toml` for configuration
- `justfile` for build commands
- Multiple export formats (Godot, Unity, generic)
- Validation in CI
- Asset manifest for engine loading
- PR build reports

---

## Progressive Disclosure

Features should be discoverable in layers:

**Layer 1 (Sketcher):** `sprite`, inline `palette`, `pxl render`

**Layer 2 (Pixel Artist):** Named `palette`, `variant`, `composition`, `pxl show`

**Layer 3 (Animator):** `animation`, `transform` array, `pingpong`, `mirror`

**Layer 4 (Motion Designer):** `type: transform`, keyframes, expressions, composition

**Layer 5 (Game Developer):** `tags`, `metadata`, `nine_slice`, export options

Each layer builds on the previous. Users can stop at any layer and be productive.

---

See [Persona Integration Plan](./plan/persona-integration.md) for implementation details.
