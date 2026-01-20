---
phase: 23b
title: Demo Tests Part 2
---

# Phase 23b: Demo Tests Part 2

Additional demo tests covering features not included in the initial Phase 23 scope.

**Status:** Not Started

**Depends on:** Phase 23 (Demo Tests) foundation (DT-1, DT-2)

**Related:**
- [demo-tests.md](demo-tests.md) - Original demo tests plan

---

## Overview

Phase 23 focused on core sprite, animation, composition, and CSS features. This document covers the remaining functionality that needs demo test coverage:

| Category | Features | Priority |
|----------|----------|----------|
| Palette Cycling | 4 | P2 - Core feature |
| Imports | 4 | P2 - Core workflow |
| Exports (additional) | 4 | P2 - Completes export coverage |
| Build System | 5 | P3 - Advanced feature |
| CLI Commands | 18 | P3 - User-facing |

---

## Task Dependency Diagram

```
                       DEMO TESTS PART 2 TASK FLOW
===============================================================================

PREREQUISITE
+-----------------------------------------------------------------------------+
|                     Phase 23 DT-1 + DT-2 Complete                           |
|              (Test harness and fixture structure exist)                      |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 1 (Core Features - Parallel)
+-----------------------------------------------------------------------------+
|  +--------------------------------+  +--------------------------------+     |
|  |          DT-20                 |  |          DT-21                 |     |
|  |    Palette Cycling Demos       |  |    Import Demos                |     |
|  |    - single_cycle              |  |    - png_to_jsonl              |     |
|  |    - multiple_cycles           |  |    - palette_detection         |     |
|  |    - cycle_timing              |  |    - multi_frame_import        |     |
|  |    - ping_pong                 |  |    - transparency_handling     |     |
|  +--------------------------------+  +--------------------------------+     |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 2 (Export Completion)
+-----------------------------------------------------------------------------+
|  +-----------------------------------------------------------------------+  |
|  |                              DT-22                                    |  |
|  |                    Additional Export Demos                            |  |
|  |    - png_static (basic PNG, complements DT-6)                        |  |
|  |    - atlas_aseprite (Aseprite .json format)                          |  |
|  |    - recolor (palette swap export)                                   |  |
|  |    Needs: DT-20 (for palette swap testing)                           |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 3 (Build System)
+-----------------------------------------------------------------------------+
|  +-----------------------------------------------------------------------+  |
|  |                              DT-23                                    |  |
|  |                    Build System Demos                                 |  |
|  |    - basic_config (pxl.toml)                                         |  |
|  |    - multi_target                                                    |  |
|  |    - incremental                                                     |  |
|  |    - watch_mode                                                      |  |
|  |    - build_variants                                                  |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 4 (CLI Commands - Parallel groups)
+-----------------------------------------------------------------------------+
|  +----------------------+  +----------------------+  +--------------------+ |
|  |        DT-24         |  |        DT-25         |  |        DT-26       | |
|  |   Core CLI Demos     |  |   Format CLI Demos   |  |   Analysis CLI     | |
|  |   - render           |  |   - fmt              |  |   Demos            | |
|  |   - validate         |  |   - inline           |  |   - show           | |
|  |   - import           |  |   - alias            |  |   - explain        | |
|  |                      |  |   - grid             |  |   - diff           | |
|  |                      |  |                      |  |   - suggest        | |
|  |                      |  |                      |  |   - analyze        | |
|  +----------------------+  +----------------------+  +--------------------+ |
|                                                                             |
|  +----------------------+  +----------------------+                         |
|  |        DT-27         |  |        DT-28         |                         |
|  |   Project CLI Demos  |  |   Info CLI Demos     |                         |
|  |   - build            |  |   - prime            |                         |
|  |   - new              |  |   - prompts          |                         |
|  |   - init             |  |   - palettes         |                         |
|  +----------------------+  +----------------------+                         |
+-----------------------------------------------------------------------------+
            |
            v
WAVE 5 (Coverage Update)
+-----------------------------------------------------------------------------+
|  +-----------------------------------------------------------------------+  |
|  |                              DT-29                                    |  |
|  |                    Update Coverage Script                             |  |
|  |    - Add all new feature categories to FEATURE_REGISTRY              |  |
|  |    - Update coverage thresholds                                      |  |
|  |    - Add to CI checks                                                |  |
|  +-----------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------+

===============================================================================

PARALLELIZATION SUMMARY:
+-----------------------------------------------------------------------------+
|  Wave 1: DT-20 + DT-21                          (2 tasks in parallel)       |
|  Wave 2: DT-22                                  (sequential)                |
|  Wave 3: DT-23                                  (sequential)                |
|  Wave 4: DT-24 + DT-25 + DT-26 + DT-27 + DT-28  (5 tasks in parallel)       |
|  Wave 5: DT-29                                  (sequential)                |
+-----------------------------------------------------------------------------+

CRITICAL PATH: DT-20 → DT-22 → DT-29
```

---

## Tasks

### Task DT-20: Palette Cycling Demos

**Wave:** 1 (parallel with DT-21)
**Priority:** P2

Create demo tests for palette cycling/color animation features.

**Deliverables:**
- Create `tests/demos/palette_cycling/`:
  - `single_cycle.rs` - Single color cycling through values
  - `multiple_cycles.rs` - Multiple independent cycle groups
  - `cycle_timing.rs` - Cycle speed and duration control
  - `ping_pong.rs` - Reverse direction cycling
- Create `examples/demos/palette_cycling/` fixtures
- Add @demo annotations:
  - `@demo format/palette#cycling`
  - `@demo format/palette#timing`
  - `@demo format/palette#ping_pong`

**Verification:**
```bash
cargo test demos::palette_cycling
```

**Dependencies:** DT-1, DT-2

---

### Task DT-21: Import Demos

**Wave:** 1 (parallel with DT-20)
**Priority:** P2

Create demo tests for PNG import functionality.

**Deliverables:**
- Create `tests/demos/imports/`:
  - `png_to_jsonl.rs` - Basic PNG to JSONL conversion
  - `palette_detection.rs` - Auto-detect palette from image
  - `multi_frame_import.rs` - Import spritesheet as animation
  - `transparency_handling.rs` - Preserve/detect transparency
- Create `examples/demos/imports/` fixtures (small test PNGs)
- Add @demo annotations:
  - `@demo cli/import#basic`
  - `@demo cli/import#palette`
  - `@demo cli/import#animation`
  - `@demo cli/import#transparency`

**Verification:**
```bash
cargo test demos::imports
```

**Dependencies:** DT-1, DT-2

---

### Task DT-22: Additional Export Demos

**Wave:** 2
**Priority:** P2

Complete export demo coverage with remaining formats.

**Deliverables:**
- Add to `tests/demos/exports/`:
  - `atlas_aseprite.rs` - Aseprite JSON atlas format
  - `recolor_export.rs` - Export with palette swap applied
- Create corresponding fixtures in `examples/demos/exports/`
- Add @demo annotations:
  - `@demo export/atlas#aseprite`
  - `@demo export/recolor`

**Verification:**
```bash
cargo test demos::exports::atlas_aseprite
cargo test demos::exports::recolor
```

**Dependencies:** DT-20 (palette swap uses cycling infrastructure)

---

### Task DT-23: Build System Demos

**Wave:** 3
**Priority:** P3

Create demo tests for the pxl.toml build system.

**Deliverables:**
- Create `tests/demos/build/`:
  - `basic_config.rs` - Minimal pxl.toml project
  - `multi_target.rs` - Multiple output targets
  - `incremental.rs` - Only rebuild changed files
  - `watch_mode.rs` - File watching (may need mocking)
  - `build_variants.rs` - Debug/release/custom variants
- Create `examples/demos/build/` with sample project structures
- Add @demo annotations:
  - `@demo build/config`
  - `@demo build/targets`
  - `@demo build/incremental`
  - `@demo build/watch`
  - `@demo build/variants`

**Verification:**
```bash
cargo test demos::build
```

**Dependencies:** DT-1, DT-2

---

### Task DT-24: Core CLI Demos

**Wave:** 4 (parallel)
**Priority:** P3

Demo tests for core CLI commands.

**Deliverables:**
- Create `tests/demos/cli/`:
  - `render.rs` - Basic render command usage
  - `validate.rs` - Validation command and error reporting
  - `import_cmd.rs` - Import command (distinct from import logic)
- Add @demo annotations:
  - `@demo cli/render`
  - `@demo cli/validate`
  - `@demo cli/import`

**Verification:**
```bash
cargo test demos::cli::core
```

**Dependencies:** DT-1, DT-2

---

### Task DT-25: Format CLI Demos

**Wave:** 4 (parallel)
**Priority:** P3

Demo tests for formatting/transformation CLI commands.

**Deliverables:**
- Create `tests/demos/cli/`:
  - `fmt.rs` - Format/prettify JSONL
  - `inline.rs` - Inline palette references
  - `alias.rs` - Create color aliases
  - `grid.rs` - Grid manipulation
- Add @demo annotations:
  - `@demo cli/fmt`
  - `@demo cli/inline`
  - `@demo cli/alias`
  - `@demo cli/grid`

**Verification:**
```bash
cargo test demos::cli::format
```

**Dependencies:** DT-1, DT-2

---

### Task DT-26: Analysis CLI Demos

**Wave:** 4 (parallel)
**Priority:** P3

Demo tests for analysis/inspection CLI commands.

**Deliverables:**
- Create `tests/demos/cli/`:
  - `show.rs` - Display sprite info
  - `explain.rs` - Explain validation errors
  - `diff.rs` - Compare sprites/files
  - `suggest.rs` - Suggest improvements
  - `analyze.rs` - Analyze sprite metrics
- Add @demo annotations:
  - `@demo cli/show`
  - `@demo cli/explain`
  - `@demo cli/diff`
  - `@demo cli/suggest`
  - `@demo cli/analyze`

**Verification:**
```bash
cargo test demos::cli::analysis
```

**Dependencies:** DT-1, DT-2

---

### Task DT-27: Project CLI Demos

**Wave:** 4 (parallel)
**Priority:** P3

Demo tests for project management CLI commands.

**Deliverables:**
- Create `tests/demos/cli/`:
  - `build_cmd.rs` - Build project from pxl.toml
  - `new.rs` - Create new sprite/project
  - `init.rs` - Initialize project in directory
- Add @demo annotations:
  - `@demo cli/build`
  - `@demo cli/new`
  - `@demo cli/init`

**Verification:**
```bash
cargo test demos::cli::project
```

**Dependencies:** DT-23 (build system)

---

### Task DT-28: Info CLI Demos

**Wave:** 4 (parallel)
**Priority:** P3

Demo tests for informational CLI commands.

**Deliverables:**
- Create `tests/demos/cli/`:
  - `prime.rs` - Prime output for AI context
  - `prompts.rs` - Show available prompts
  - `palettes.rs` - List/show built-in palettes
- Add @demo annotations:
  - `@demo cli/prime`
  - `@demo cli/prompts`
  - `@demo cli/palettes`

**Verification:**
```bash
cargo test demos::cli::info
```

**Dependencies:** DT-1, DT-2

---

### Task DT-29: Update Coverage Script

**Wave:** 5
**Priority:** P2

Update demo-coverage.sh to track all new feature categories.

**Deliverables:**
- Update `scripts/demo-coverage.sh` FEATURE_REGISTRY:
  - Add palette-cycling category (4 features)
  - Add imports category (4 features)
  - Add build-system category (5 features)
  - Add cli-core category (3 features)
  - Add cli-format category (4 features)
  - Add cli-analysis category (5 features)
  - Add cli-project category (3 features)
  - Add cli-info category (3 features)
  - Add missing items to existing categories:
    - transforms: recolor
    - animation: frame-metadata
    - composition: multi-sprite
    - exports: png-static, png-scaled, gif-animated, atlas-aseprite
- Update threshold recommendations
- Create `docs/plan/demo-coverage.md` tracking doc

**Verification:**
```bash
./scripts/demo-coverage.sh --verbose
# Should show all new categories
```

**Dependencies:** DT-20 through DT-28

---

## Feature Coverage Additions

### Palette Cycling
- [ ] Single color cycle
- [ ] Multiple cycle groups
- [ ] Cycle timing
- [ ] Ping-pong mode

### Imports
- [ ] PNG to JSONL conversion
- [ ] Palette detection
- [ ] Multi-frame import
- [ ] Transparent color handling

### Exports (additional)
- [ ] Atlas (Aseprite)
- [ ] Recolor/palette swap export

### Build System
- [ ] Basic pxl.toml configuration
- [ ] Multi-target builds
- [ ] Incremental rebuilds
- [ ] Watch mode
- [ ] Build variants

### CLI Commands
- [ ] render
- [ ] import
- [ ] validate
- [ ] fmt
- [ ] show
- [ ] explain
- [ ] diff
- [ ] suggest
- [ ] inline
- [ ] alias
- [ ] grid
- [ ] build
- [ ] new
- [ ] init
- [ ] analyze
- [ ] prime
- [ ] prompts
- [ ] palettes

---

## Notes

- CLI demos may require subprocess testing or output capture
- Build system demos need temporary directory fixtures
- Watch mode testing may need mocking or short timeouts
- Some CLI commands may not exist yet - verify before implementing
