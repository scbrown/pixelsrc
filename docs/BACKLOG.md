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
