# Backlog

Ideas and features deferred for future consideration.

---

## Edge Constraints for Tiling

**Related to:** Phase 16 (Composition Tiling)

When tiling sprites to create larger images, adjacent tiles need to connect seamlessly. Edge constraints would formalize this.

### Concept

```jsonl
{"type": "sprite", "name": "sky_left", "size": [32, 32],
  "edges": {
    "right": ["sky_gradient", "cloud_wisps"],
    "bottom": ["treeline"]
  },
  "grid": [...]
}

{"type": "sprite", "name": "sky_right", "size": [32, 32],
  "edges": {
    "left": ["sky_gradient", "cloud_wisps"],
    "bottom": ["treeline"]
  },
  "grid": [...]
}
```

### Semantics

- `edges` is optional metadata (not rendered, but used for validation/guidance)
- Edge names are semantic tags that should match between adjacent tiles
- AI uses edge constraints to understand what should connect
- Renderer could validate that adjacent tiles have compatible edges

### Open Questions

- Should edges be enforced or advisory?
- How granular? Per-pixel edge colors? Semantic zones? Single tag?
- Should edges be defined on sprites or on the composition?
- How to handle corners (where 4 tiles meet)?

### Alternative: Overlap Regions

Instead of edge constraints, tiles could have explicit overlap:
```jsonl
{"type": "composition", "cell_size": [32, 32], "overlap": [4, 4], ...}
```

Tiles would be 36x36 but only 32x32 would be visible, with 4px shared border for blending.

---

## AI Generation Assistance (Non-Rendering Features)

**Related to:** GenAI accessibility goals

Ideas for helping AI generate better pixelsrc content that don't affect the renderer.

### Guide/Sketch Workflow

AI generates a low-res "guide" first, then uses it as context for detailed tiles:

```jsonl
{"type": "guide", "name": "scene_plan", "size": [8, 8], "grid": [
  "{sky}{sky}{sky}{sky}{sky}{sky}{sky}{sky}",
  "{sky}{sky}{tree}{sky}{sky}{tree}{sky}{sky}",
  "{tree}{tree}{tree}{grass}{grass}{tree}{tree}{tree}",
  ...
]}
```

**Open question:** Is this a format feature or just a workflow pattern? The AI can reference any sprite as a "guide" without special syntax.

### Generation Hints

Metadata that helps AI but isn't rendered:

```jsonl
{"type": "sprite", "name": "hero",
  "hints": {
    "style": "16-bit RPG",
    "symmetry": "vertical",
    "reference": "classic JRPG protagonist"
  },
  "grid": [...]
}
```

**Open question:** Is this useful, or does it just add noise? The AI already has the prompt context.

### Validation Mode for AI

A `pxl validate` command that checks for common AI mistakes:
- Inconsistent row lengths
- Undefined palette tokens
- Missing transparency where expected
- Aspect ratio anomalies

This could be integrated into generation workflows to catch errors before rendering.

**Note:** See Phase 17 (AI Assistance Tools) for full implementation plan.

---

## Metadata / Frontmatter

**Concept:** Optional metadata that travels with pixelsrc files but isn't part of the rendering spec.

Like how Markdown has frontmatter (YAML between `---` markers) that's separate from the Markdown spec itself, pixelsrc could have a metadata convention that tools recognize but renderers ignore.

### Potential Approaches

**Option A: First-line comment convention**
```jsonl
// {"meta": {"author": "claude", "intent": "16x16 RPG hero", "version": "1.0"}}
{"type": "palette", ...}
```
Comments aren't valid JSON, so renderer skips. Tools parse specially.

**Option B: Meta object type**
```jsonl
{"type": "meta", "author": "claude", "intent": "16x16 RPG hero", "created": "2025-01-14"}
{"type": "palette", ...}
```
Renderer ignores `type: "meta"`. Clean JSON, no special parsing.

**Option C: Separate .meta file**
```
hero.jsonl      # The sprite
hero.jsonl.meta # Metadata JSON
```
Complete separation. More files to manage.

### Use Cases

- **AI context**: Intent, style references, generation parameters
- **Human documentation**: Author, description, license
- **Tooling**: VS Code extension showing hints, version tracking
- **Multi-session work**: AI can read metadata to understand prior context

### Open Questions

- Should metadata be standardized or freeform?
- Which approach balances cleanliness with practicality?
- Does this belong in the spec or stay purely conventional?

**Current leaning:** Option B (`type: "meta"`) - cleanest, valid JSON, easy to implement.

---

## Compositions as Animation Frames

**Related to:** Phase 3 (Animation), Phase 2 (Composition)

### The Problem

When creating animations with sprites that need precise positioning (like logo assembly or complex multi-element scenes), you have to manually construct each frame's full grid. Compositions make positioning easy with character maps and `cell_size`, but currently animations can only reference sprites, not compositions.

### Example: Current Workaround

To create a logo assembly animation, each frame must be a full 32x32 sprite with manually positioned pixels:
```jsonl
{"type": "sprite", "name": "f01", "size": [32, 32], "grid": ["{p}{p}{_}...long grid..."]}
{"type": "sprite", "name": "f02", "size": [32, 32], "grid": ["{P}{P}{_}...another long grid..."]}
...
{"type": "animation", "frames": ["f01", "f02", ...]}
```

### Proposed Solution

Allow animations to reference compositions as frames:
```jsonl
{"type": "sprite", "name": "blk_p", "grid": ["{p}{p}", "{p}{p}"]}
{"type": "composition", "name": "f01", "size": [32, 32], "cell_size": [2, 2],
  "sprites": {"p": "blk_p", ...},
  "layers": [{"map": ["p..............c", "...", "k..............g"]}]}
{"type": "animation", "frames": ["f01", "f02", ...]}  // compositions work here
```

### Benefits

1. **Cleaner authoring** - Use readable character maps instead of long token strings
2. **Reusable elements** - Define color blocks once, position them in multiple frames
3. **AI-friendly** - Easier to reason about character grid alignment
4. **Smaller files** - DRY principle reduces repetition

### Implementation Notes

- Animation frame lookup should check both sprites and compositions
- Compositions would be rendered to images at animation time
- No change to animation format, just expanded resolution behavior

---

## `pxl show` - Grid Display with Coordinates

**Related to:** Phase 15 (AI Assistance Tools)

### The Problem

AI struggles with pixel art alignment because there's no visual feedback during generation. When editing raw JSONL grids, it's hard to verify alignment, symmetry, and proportions without rendering to PNG.

### Concept

Simple grid display with row/column coordinates:

```bash
pxl show sprite.jsonl
pxl show sprite.jsonl --sprite bracket_l
```

Output:
```
bracket_l (6x10):

     0 1 2 3 4 5
   ┌─────────────
 0 │ _ _ _ g c c
 1 │ _ _ g c c _
 2 │ _ _ g c g _
 3 │ _ _ g c g _
 4 │ _ g c g _ _
 5 │ _ g c g _ _
 6 │ _ _ g c g _
 7 │ _ _ g c g _
 8 │ _ _ g c c _
 9 │ _ _ _ g c c

Tokens: _ g c
Palette: favicon
```

### Key Features

1. **Coordinate display** - Row/column numbers for precise reference
2. **Token simplification** - Show short token names (strip `{}` braces)
3. **Multiple sprites** - Show all sprites or filter with `--sprite`
4. **Composition support** - Show composed result with `--composition`

### Implementation Priority

**P0** - Immediate value for AI workflows. Simple to implement.

---

## `pxl check` - Symmetry and Alignment Analysis

**Related to:** Phase 15 (AI Assistance Tools)

### The Problem

Common pixel art mistakes:
- Asymmetric sprites that should be symmetric
- Off-center content
- Inconsistent proportions between related sprites

These are hard to spot in raw JSONL but obvious visually.

### Concept

Analyze sprites for symmetry and alignment:

```bash
pxl check sprite.jsonl
pxl check sprite.jsonl --sprite bracket_l
```

Output:
```
bracket_l (6x10):

     0 1 2 3 4 5
   ┌─────────────
 0 │ _ _ _ g c c
 1 │ _ _ g c c _
 2 │ _ _ g c g _
 3 │ _ _ g c g _
 4 │ _ g c g _ _    ← middle point
 5 │ _ g c g _ _    ← middle point
 6 │ _ _ g c g _
 7 │ _ _ g c g _
 8 │ _ _ g c c _
 9 │ _ _ _ g c c
         ↑
     center (col 3)

Symmetry:
  ✓ Horizontal: 100% (rows 0-4 mirror rows 5-9)
  ✗ Vertical: 0% (intentionally asymmetric - curly brace)

Alignment:
  Content bounds: cols 1-5, rows 0-9
  Center of mass: col 3.2, row 4.5
  ⚠ Horizontal: off-center right by 0.7px

Suggestions:
  - Consider adding 1 column of padding on left
```

### Analysis Features

1. **Horizontal symmetry** - Top/bottom mirror check
2. **Vertical symmetry** - Left/right mirror check
3. **Center of mass** - Where the "weight" of non-transparent pixels sits
4. **Bounding box** - Actual content area vs declared size
5. **Comparison mode** - Check two sprites match proportions

### Comparison Mode

```bash
pxl check sprite.jsonl --compare bracket_l bracket_r
```

Output:
```
Comparing bracket_l vs bracket_r:

  bracket_l        bracket_r
  _ _ _ g c c      k k g _ _ _
  _ _ g c c _      _ k k g _ _
  ...              ...

  ✓ Same dimensions (6x10)
  ✓ Horizontally mirrored (98% match)
  ⚠ bracket_r middle point at row 4-5, bracket_l at row 4-5 (aligned)
```

### Implementation Priority

**P1** - High value for catching mistakes before rendering.

---

## `pxl edit` - Region-Based Sprite Editing (Future)

**Related to:** Phase 15 (AI Assistance Tools), Phase 12 (Tiling)

**Note:** This is deferred. For most use cases, the **composition-as-jig** workflow is preferable - each element becomes its own sprite with guaranteed alignment.

### Concept

Region-based editing with discrete, verifiable commands:

```bash
# Define named regions
pxl edit sprite.jsonl --define-region "p" 4 3 3 5

# Edit a specific region
pxl edit sprite.jsonl --region p --set "ppp|p_p|ppp|p__|p__"

# Or use a region file for batch edits
pxl edit sprite.jsonl --apply regions.txt
```

### Region File Format

```
# Region definitions
region bracket_l 0 0 3 8
region p 4 3 3 5
region x 8 3 3 5

# Content (rows separated by |)
set bracket_l "_cc|c__|c__|c__|_c_|c__|c__|_cc"
set p "ppp|p_p|ppp|p__|p__"
set x "x_x|_x_|_x_|x_x|___"
```

### Key Design Principles

1. **Batch/declarative over interactive** - AI can't track cursor state
2. **Region isolation** - Changes to one region can't affect others
3. **Atomic operations** - Each command is verifiable before next

### Open Questions

- Is this valuable enough vs. just using composition-as-jig?
- Should regions be sprite-level metadata or external files?
- What's the MVP feature set?

---

## Multi-File Compositions

**Related to:** Phase 12 (Tiling)

### The Problem

For complex compositions using the "jig" workflow, having all sprites in one file creates large context. Editing one letter means loading the entire banner.

### Basic Approach (Near-term)

CLI accepts multiple files, builds shared namespace:

```bash
pxl render bracket_l.jsonl letter_p.jsonl letter_x.jsonl banner.jsonl
```

Each file exposes named objects. Compositions reference by name only - they don't know/care what file a sprite came from. Files expected to be in the same directory.

### Project Mode (Future)

For larger projects with nested directories:

```
my-game/
  pxl.toml              # project manifest
  palettes/
    dracula.jsonl
  sprites/
    characters/
      hero.jsonl
    ui/
      buttons.jsonl
  scenes/
    title.jsonl
```

With manifest:
```toml
[project]
name = "my-game"

[sources]
include = ["**/*.jsonl"]

[output]
dir = "dist"
```

CLI works like a compiler:
```bash
pxl build                    # uses pxl.toml
pxl build --composition title
```

### Open Questions

- Name collision handling (error? last-wins?)
- Order dependency (palette before sprite that uses it)
- Does this stray too far from "GenAI-native" simplicity?

---

## Game Engine Integration

Export to Unity, Godot, Tiled, and CSS formats.

**Depends on:** Phase 3 (Animation) complete

### Unity Export

Export Unity-compatible spritesheet with metadata.

- Generate spritesheet PNG
- Generate `.meta` file with slice data
- Correct pivot points and borders

### Godot Export

Export Godot resource files.

- Generate `.tres` SpriteFrames resource
- Animation data included
- Correct frame references

### Tiled Export

Export Tiled tileset format.

- Generate `.tsx` tileset file
- PNG tileset image
- Correct tile dimensions

### CSS Export

Export CSS sprite format.

- Generate spritesheet PNG
- Generate CSS file with classes
- Background-position for each sprite

### Export CLI

Add export CLI options:

```
pxl export unity input.jsonl -o output/
pxl export godot input.jsonl -o output/
pxl export tiled input.jsonl -o output/
pxl export css input.jsonl -o output/
```

### Dependency Graph

```
Phase 3 complete
     │
     ├── Unity Export ────┐
     ├── Godot Export ────┼── Export CLI
     ├── Tiled Export ────┤
     └── CSS Export ──────┘
```
