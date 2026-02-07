# Import System, Sprite References & Library Sharing

**Goal:** Enable cross-file references via a module system with project-qualified namespaces, explicit imports, and eventually a library/package system for sharing pixelsrc components across projects.

**Status:** Planning

**Depends on:** Phase 20 (Build System), Phase 16 (pxl.toml)

**Related beads:** TTP-5odz (planning), label:`import-system` (11 implementation beads)

---

## Motivation

Every `.pxl` file is currently an island. A sprite in `player.pxl` cannot reference a palette defined in `shared/colors.pxl` by name. The only cross-file mechanism is `@include:path` which resolves exactly one palette from an external file — no sprites, no compositions, no transforms, no named references.

**Real pain points (in priority order):**

1. **Palette sharing** — Game projects define palettes once and reuse across dozens of sprite files. Currently requires `@include:` with a file path in every sprite.
2. **Cross-file sprite references** — Compositions and animations want to reference sprites from other files. Currently impossible without inlining everything in one file.
3. **Transform reuse** — Motion designers define transform libraries but can't share them across files.
4. **Variant/source reuse** — Define a base sprite in one file, create variants in another. Currently requires the `source` field to be in the same file.
5. **Project organization** — Large projects want to split assets into logical directories without losing cross-boundary references.
6. **Library sharing** — Share palettes and sprite templates across projects (e.g., a "retro palettes" package).

---

## Current State: What Exists

### `@include:` (src/include.rs)
- Syntax: `"palette": "@include:path/to/file.pxl"`
- Resolves relative to including file's directory
- Returns the **first palette** found in the target file
- Has circular include detection
- Auto-detects `.pxl` and `.jsonl` extensions
- **Limitation:** Palette-only. No sprites, no transforms, no named selection.

### SpriteRegistry (src/registry/sprite.rs)
- In-memory HashMap of sprites by name
- Supports `resolve()` for transform chains (source references, mirroring, etc.)
- **Scoped to a single parse** — never shared across files

### PaletteRegistry (src/registry/palette.rs)
- In-memory HashMap of palettes by name
- Supports `resolve_strict()` and `resolve_lenient()`
- **Scoped to a single parse** — never shared across files

### Build Pipeline (src/build/pipeline.rs)
- Discovers all `.pxl` files under `src/pxl/`
- Processes each file independently with its own registries (lines 374-397)
- For atlas targets: parses each source file with fresh registries
- **No cross-file registry merging**

### pxl.toml (src/config/)
- Has `project`, `atlases`, `animations`, `exports`, `validate`, `watch` sections
- **No `imports` or `dependencies` section**

---

## Design: Module System with Project-Qualified Namespaces

### Core Concept: File Paths Are Namespaces

The directory structure under `src/pxl/` defines a module hierarchy. Every item (palette, sprite, transform, animation) has a **canonical name** of the form:

```
project_name/path/to/file:item_name
```

Examples:
```
my-rpg/characters/hero/base:idle
my-rpg/palettes/shared:gameboy
lospec-palettes/retro:gameboy          ← external dependency
```

The project name comes from `[project] name` in `pxl.toml`. This prevents collisions when consuming external libraries.

### Reference Scoping Levels

Resolution from narrowest to broadest:

| Context | Syntax | Example | When to Use |
|---|---|---|---|
| Same file | bare name | `"idle"` | Always works for items in the same file |
| Same directory | `file:item` | `"base:idle"` | Sibling files see each other |
| Same project | `path/file:item` | `"characters/hero/base:idle"` | Root-relative within project |
| External dep | `project/path/file:item` | `"lospec-palettes/retro:gameboy"` | Cross-project references |

### Within Your Own Project — Project Prefix Implied

You never write your own project name when referencing items in the same project:

```json5
// compositions/battle.pxl in project "my-rpg"
{"type": "import", "from": "characters/hero/base", "as": "hero"}
{"type": "import", "from": "characters/goblin/base", "as": "goblin"}
{"type": "import", "from": "environment/tiles/grass"}
{"type": "composition", "name": "battle", "sprites": {
  "player": "hero:idle",
  "enemy": "goblin:idle",
  "ground": "grass:base"
}}
```

### External Dependencies — Project Name Required

```json5
// Using a palette from an installed library
{"type": "import", "from": "lospec-palettes/retro", "palettes": ["gameboy"]}
{"type": "sprite", "palette": "gameboy", ...}
```

`lospec-palettes` is the dependency's project name from their `pxl.toml`.

### Import Declarations

The `import` object type pulls items from other files into local scope:

```json5
// Import specific items
{"type": "import", "from": "characters/hero/base", "sprites": ["idle", "walk", "run"]}
// Now "idle" resolves to my-rpg/characters/hero/base:idle

// Import with a namespace alias
{"type": "import", "from": "characters/hero/base", "as": "hero"}
// Now "hero:idle" works instead of "characters/hero/base:idle"

// Import everything from a file (all item types)
{"type": "import", "from": "characters/hero/base"}
// All items available by short name (warn on collision)

// Import a whole directory
{"type": "import", "from": "characters/hero/"}
// All files in dir importable: "base:idle", "attack:slash"

// Import from external dependency
{"type": "import", "from": "lospec-palettes/retro", "palettes": ["gameboy"]}

// Relative import (works without pxl.toml)
{"type": "import", "from": "./palettes/brand"}
{"type": "import", "from": "../shared/colors"}
```

### Path Resolution Rules

| Path form | Resolves relative to | Works without project? |
|---|---|---|
| `./path` or `../path` | Current file's directory | Yes |
| `path` (no prefix) | Project root (`src/pxl/`) | No — requires `pxl.toml` |
| `depname/path` | Dependency's project root | No — requires `[dependencies]` |

**Distinguishing local paths from dependency names:** The resolver checks `[dependencies]` in `pxl.toml`. If the first path segment matches a declared dependency name, it resolves from that dependency. Otherwise, it resolves from `src/pxl/`. Warn if a local directory name shadows a dependency name.

### Default Item Resolution

When referencing a path without `:item`:

1. Look for an item with the same name as the filename (e.g., file `idle.pxl` → item `idle`)
2. If the file has exactly one item of the required type, use that
3. Otherwise: error ("ambiguous reference, specify item with `path:name`")

This avoids the ugly redundancy of `sprites/idle:idle` for single-sprite files.

### All Object Types Are Importable

The import system covers **all** object types, not just palettes:
- Palettes: `"palettes": ["gameboy", "nes"]`
- Sprites: `"sprites": ["idle", "walk"]`
- Transforms: `"transforms": ["spiral-in", "bounce"]`
- Animations: `"animations": ["walk_cycle"]`

Unfiltered import (`{"type":"import","from":"path"}`) imports everything.

---

## Persona Impact Analysis

### Sketcher (Minimal complexity)

**Impact:** Zero. Single-file, inline-palette workflow is completely unaffected. No project context, no imports, no multi-file anything. The import system is invisible.

**Risk:** Does project auto-detection in `pxl render` slow them down? No — without `pxl.toml` in parent directories, nothing changes.

### Pixel Artist (Low-Medium complexity)

**Impact:** Clear improvement over `@include:`. They share palettes across files.

Before:
```json5
// icons/home.pxl
{"type": "sprite", "palette": "@include:../palettes/brand.pxl", ...}
```

After (with project):
```json5
{"type": "import", "from": "palettes/brand"}
{"type": "sprite", "palette": "brand", ...}
```

After (without project — relative import):
```json5
{"type": "import", "from": "../palettes/brand"}
{"type": "sprite", "palette": "brand", ...}
```

Works with or without `pxl.toml`. `@include:` continues to work for backward compatibility.

### Animator (Medium complexity)

**Impact:** Critical enabler. Animation files reference sprites from other files:

```json5
// animations/player.pxl
{"type": "import", "from": "sprites/player/idle"}
{"type": "import", "from": "sprites/player/run"}
{"type": "animation", "name": "walk", "frames": ["idle_1", "run_1", "run_2"], "fps": 8}
```

The default-item resolution prevents the clunky `sprites/player/idle:idle_1` pattern.

### Motion Designer (High complexity)

**Impact:** Critical enabler for transform libraries. Currently transforms defined in one file can't be used from another.

```json5
// demos/coin_collect.pxl
{"type": "import", "from": "transforms/motion", "transforms": ["spiral-in", "bounce-in"]}
{"type": "animation", "name": "coin_collect", "source": "coin", "transform": [
  {"op": "spiral-in", "radius": 12, "decay": 0.9, "spin": 0.5}
]}
```

This is only possible because the import system covers transforms, not just palettes/sprites.

### Game Developer (Medium-High complexity)

**Impact:** Largest win. Complex directory hierarchies with cross-cutting references resolved cleanly via qualified paths + import aliases:

```json5
// compositions/battle.pxl
{"type": "import", "from": "characters/hero/base", "as": "hero"}
{"type": "import", "from": "characters/goblin/base", "as": "goblin"}
{"type": "composition", "name": "battle", "sprites": {
  "player": "hero:idle",
  "enemy": "goblin:idle"
}}
```

Project-qualified canonical names (`my-rpg/characters/hero/base:idle`) prevent collisions with external deps.

---

## Parsing Edge Cases & Resolution Rules

### 1. Names Cannot Contain `:` or `/`

These characters are reserved for path syntax. Enforce during parsing/validation.

**Verified:** Zero existing `.pxl` or `.jsonl` files in the codebase use `:` or `/` in names.

### 2. Local Scope Always Wins

If a file defines sprite `"idle"` and a sibling file `idle.pxl` also exists, bare `"idle"` always means the local definition. To reference the sibling: `"idle:sprite_name"`.

### 3. File Takes Priority Over Same-Named Directory

If both `characters/hero.pxl` and `characters/hero/` exist, `characters/hero:idle` resolves from the **file**. To reach into the directory: `characters/hero/base:idle`.

### 4. No Extension in Paths

Write `characters/hero/base:idle`, not `characters/hero/base.pxl:idle`. Auto-resolve `.pxl` then `.jsonl`, matching existing `@include:` behavior.

### 5. Type-Aware Resolution

A file can have both palette `"forest"` and sprite `"forest"`. No ambiguity — resolution is context-aware. A `"palette"` field searches palettes only. A `"frames"` entry searches sprites only. This matches how PaletteRegistry and SpriteRegistry already work as separate registries.

### 6. Import Alias Priority

Import aliases take priority over sibling scope:

```json5
{"type": "import", "from": "characters/hero/base", "as": "hero"}
// "hero:idle" → import alias, NOT sibling file hero.pxl
```

Import aliases **cannot** shadow local names (error if alias matches a name defined in the current file).

### 7. Import Collision: First Wins

```json5
{"type":"import","from":"a"}  // defines sprite "x"
{"type":"import","from":"b"}  // also defines sprite "x"
```

First import wins. Warn in lenient mode, error in strict mode.

### 8. Circular Imports

Detected via visited-set (same mechanism as existing `@include:` circular detection). Error with clear message showing the cycle.

### 9. Diamond Imports

A imports B and C. Both B and C import D. D is parsed once (visited set prevents re-parsing). No duplication.

### 10. Standalone Files (No pxl.toml)

Without a project:
- Local scope: works
- `@include:`: works (backward compat)
- Relative imports (`./`, `../`): work
- Root-relative paths: **error** ("requires pxl.toml project")
- External deps: **error** ("requires [dependencies] in pxl.toml")

Clean subset — standalone files have reduced but functional cross-file support.

### 11. Distinguishing Local Paths from Dependency Names

`characters/hero` (local) and `lospec-palettes/retro` (dependency) look identical syntactically.

**Rule:** Check `[dependencies]` in `pxl.toml`. If the first path segment is a declared dependency name, resolve from that dependency. Otherwise, resolve from `src/pxl/`. Warn if a local directory shadows a dependency name.

---

## Resolution Precedence (Complete)

When resolving a name, check in this order:

1. **Local** — Item defined in the same file
2. **Import alias** — Matching an `"as"` alias from an import declaration
3. **Explicit import** — Item pulled in by a selective import (`"sprites": ["name"]`)
4. **Wildcard import** — Item from an unfiltered import
5. **Sibling scope** — Item from a file in the same directory
6. **Project namespace** — Item from any file in `src/pxl/` (only for globally unique names)

Collisions at the same level: warn in lenient mode, error in strict mode.

---

## Backward Compatibility

- **`@include:` syntax** — Continues to work exactly as before. Not deprecated.
- **Standalone files** — No behavior change unless `import` objects are present.
- **Build system** — Existing `pxl build` behavior unchanged for projects without cross-file refs.
- **Name restriction** — New rule that names cannot contain `:` or `/`. No existing files are affected (verified).

---

## Implementation Plan

### Level 1: Project Namespace + Qualified References

#### IMP-1: Project-Wide Registry Loading (`TTP-w0sf`)

New `src/build/project_registry.rs` — parse all project files into shared registries with qualified names.

#### IMP-2: Build Pipeline Two-Pass Architecture (`TTP-c1gr`)

Restructure build: pass 1 parses all files into ProjectRegistry, pass 2 resolves and renders using shared registries.

#### IMP-3: Project Context for `pxl render` (`TTP-dt5b`)

Auto-detect `pxl.toml`, load ProjectRegistry for single-file render. Add `--no-project` flag.

### Level 2: Explicit Import System

#### IMP-4: Import Object Type (`TTP-olst`)

New `Import` model + parser support for `{"type":"import",...}`.

#### IMP-5: Import Resolution (`TTP-bj1o`)

Resolve import declarations during parsing. Handle relative/root-relative/external paths.

#### IMP-6: Extend @include (#name syntax) (`TTP-hgvw`)

Backward-compatible: `@include:path#palette_name` for specific palette selection.

### Level 3: Tooling

#### IMP-7: LSP Cross-File Support (`TTP-3orc`)

ProjectRegistry for completions, go-to-definition, hover info across files.

#### IMP-8: Validate & Suggest (`TTP-vnvl`)

Cross-file reference checking, unused import detection, `@include:` → import migration suggestions.

### Level 4: Library System (Future)

#### IMP-9: pxl.toml [dependencies] (`TTP-e0m7`)

Path and git-based dependency declarations.

#### IMP-10: Dependency Fetching (`pxl install`) (`TTP-10qq`)

Fetch, cache in `.pxl/deps/`, resolve external references.

#### IMP-11: Namespaced External References (`TTP-bwz7`)

`"depname/path:item"` resolution through installed dependencies.

---

## Task Dependency Diagram

```
                     IMPORT SYSTEM TASK FLOW
═════════════════════════════════════════════════════════════════

LEVEL 1 (Project Namespace)
┌───────────────────────────────────────────────────────────────┐
│  IMP-1 ──→ IMP-2 ──→ IMP-3                                   │
│  Registry    Two-Pass   pxl render                            │
│  Loading     Build      project ctx                           │
└───────────────────────────────────────────────────────────────┘

LEVEL 2 (Explicit Imports)  ← can parallel with Level 1
┌───────────────────────────────────────────────────────────────┐
│  IMP-4 ──→ IMP-5        IMP-6 (independent)                  │
│  Import     Import       @include                             │
│  Model      Resolution   #name ext                            │
└───────────────────────────────────────────────────────────────┘

LEVEL 3 (Tooling)  ← after Levels 1+2
┌───────────────────────────────────────────────────────────────┐
│  IMP-7 (LSP)  ∥  IMP-8 (Validate/Suggest)                    │
└───────────────────────────────────────────────────────────────┘

LEVEL 4 (Libraries)  ← future, after Level 2
┌───────────────────────────────────────────────────────────────┐
│  IMP-9 ──→ IMP-10 ──→ IMP-11                                 │
│  Config     Fetch       Namespaced                            │
│  Schema     & Cache     References                            │
└───────────────────────────────────────────────────────────────┘

CRITICAL PATH: IMP-1 → IMP-2 → IMP-3 (enables project namespace)
HIGHEST VALUE: Level 1 alone enables cross-file refs for all personas
```

---

## Success Criteria

### Level 1 (Project Namespace)
1. Sprite in `sprites/hero.pxl` references palette from `palettes/gameboy.pxl` by name
2. Composition references sprites from other files via qualified paths
3. Animation references sprites from other directories
4. `pxl build` resolves all cross-file references
5. `pxl render` detects project context automatically
6. All existing tests pass (backward compatible)

### Level 2 (Explicit Imports)
7. `import` objects parsed and resolved
8. Selective imports filter by type and name
9. `"as"` aliases create namespace shortcuts
10. Directory imports work
11. Relative imports (`./`, `../`) work without project
12. Circular import detection with clear errors

### Level 3 (Tooling)
13. LSP provides cross-file completions
14. Go-to-definition navigates across files
15. `pxl validate` checks cross-file references
16. `pxl suggest` recommends import improvements

---

## Open Questions

1. **Watch mode interaction:** When a shared palette file changes, should watch mode rebuild all dependent files? Requires dependency tracking — deferred to implementation.

2. **Re-exports:** If A imports from B, and C imports from A, should C see B's items? Recommendation: No — keep it simple, direct imports only.

3. **Wildcard file imports:** `{"type":"import","from":"characters/hero/*.pxl"}` — glob patterns in imports? Recommendation: Not initially. Use directory imports instead.
