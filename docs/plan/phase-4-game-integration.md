# Phase 4: Game Engine Integration

**Goal:** Export to Unity, Godot, Tiled, and CSS formats

**Status:** Planning

**Depends on:** Phase 3 complete

---

## Scope

Phase 4 adds export formats for popular game engines:
- Unity spritesheet + .meta file
- Godot .tres resource
- Tiled .tsx tileset
- CSS sprites

**Not in scope:** Editor plugins, real-time preview

---

## Tasks

### Task 4.1: Unity Export
**Parallelizable:** Yes

Export Unity-compatible spritesheet with metadata.

**Deliverables:**
- Generate spritesheet PNG
- Generate `.meta` file with slice data
- Correct pivot points and borders

**Acceptance Criteria:**
- Unity imports spritesheet correctly
- Individual sprites accessible

---

### Task 4.2: Godot Export
**Parallelizable:** Yes

Export Godot resource files.

**Deliverables:**
- Generate `.tres` SpriteFrames resource
- Animation data included
- Correct frame references

**Acceptance Criteria:**
- Godot loads resource correctly
- AnimatedSprite works

---

### Task 4.3: Tiled Export
**Parallelizable:** Yes

Export Tiled tileset format.

**Deliverables:**
- Generate `.tsx` tileset file
- PNG tileset image
- Correct tile dimensions

**Acceptance Criteria:**
- Tiled loads tileset correctly
- Can paint with tiles

---

### Task 4.4: CSS Export
**Parallelizable:** Yes

Export CSS sprite format.

**Deliverables:**
- Generate spritesheet PNG
- Generate CSS file with classes
- Background-position for each sprite

**Acceptance Criteria:**
- CSS classes display correct sprites
- Works in browsers

---

### Task 4.5: Export CLI
**Parallelizable:** Yes (after 4.1-4.4)

Add export CLI options.

**Deliverables:**
- `pxl export unity input.jsonl -o output/`
- `pxl export godot input.jsonl -o output/`
- `pxl export tiled input.jsonl -o output/`
- `pxl export css input.jsonl -o output/`

**Acceptance Criteria:**
- All export commands work
- Output files in correct locations

---

## Dependency Graph

```
Phase 3 complete
     │
     ├── 4.1 (Unity) ────┐
     ├── 4.2 (Godot) ────┼── 4.5 (CLI)
     ├── 4.3 (Tiled) ────┤
     └── 4.4 (CSS) ──────┘
```

---

## Verification

1. Import into Unity, verify sprites work
2. Import into Godot, verify animation works
3. Load in Tiled, verify tileset works
4. Use CSS in browser, verify sprites display
