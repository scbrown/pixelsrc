# Backlog

Ideas and features deferred for future consideration.

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
