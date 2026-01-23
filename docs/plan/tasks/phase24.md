# Phase 24: Format v2 - Structured Regions

**Goal:** Replace grid-based format with structured regions, add semantic metadata

**Status:** Planning

**Spec:** `docs/plan/format2.md`

---

## Task Dependency Diagram

```
                              PHASE 24 TASK FLOW
    ═══════════════════════════════════════════════════════════════════════════

    WAVE 1 (Foundation) - No dependencies
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                                                                         │
    │   ┌────────────────┐                                                    │
    │   │   24.1         │                                                    │
    │   │   JSON5 Parser │                                                    │
    │   │   [CRITICAL]   │                                                    │
    │   └───────┬────────┘                                                    │
    │           │                                                             │
    └───────────┼─────────────────────────────────────────────────────────────┘
                │
                ▼
    WAVE 2 (Structured Core) - Depends on 24.1
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                                                                         │
    │   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
    │   │   24.2         │  │   24.3         │  │   24.4         │           │
    │   │   RegionDef    │  │   Shapes       │  │   Path Parser  │           │
    │   │   Struct       │  │   Rasterize    │  │   (SVG-lite)   │           │
    │   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
    │           │                   │                   │                     │
    │           └───────────────────┼───────────────────┘                     │
    │                               │                                         │
    │                               ▼                                         │
    │                       ┌────────────────┐                                │
    │                       │   24.5         │                                │
    │                       │   Compounds    │                                │
    │                       │   (union/sub)  │                                │
    │                       └───────┬────────┘                                │
    │                               │                                         │
    │                               ▼                                         │
    │   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
    │   │   24.6         │  │   24.7         │  │   24.8         │           │
    │   │   Fill Op      │  │   Modifiers    │  │   Structured   │           │
    │   │   (flood fill) │  │   (symmetric)  │  │   Renderer     │           │
    │   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
    │           │                   │                   │                     │
    └───────────┼───────────────────┼───────────────────┼─────────────────────┘
                │                   │                   │
                └───────────────────┼───────────────────┘
                                    │
                                    ▼
    WAVE 3 (Semantic) - Depends on Wave 2
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                                                                         │
    │   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
    │   │   24.9         │  │   24.10        │  │   24.11        │           │
    │   │   Roles in     │  │   Relation-    │  │   Constraint   │           │
    │   │   Palette      │  │   ships        │  │   Validation   │           │
    │   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
    │           │                   │                   │                     │
    │           └───────────────────┼───────────────────┘                     │
    │                               │                                         │
    └───────────────────────────────┼─────────────────────────────────────────┘
                                    │
                ┌───────────────────┼───────────────────┐
                │                   │                   │
                ▼                   ▼                   ▼
    WAVE 4 (Import) ────────────────────────  WAVE 5 (Grid Removal)
    ┌───────────────────────────────────┐     ┌───────────────────────────────┐
    │                                   │     │                               │
    │  ┌──────────┐  ┌──────────┐      │     │  ┌──────────┐  ┌──────────┐  │
    │  │  24.12   │  │  24.13   │      │     │  │  24.17   │  │  24.18   │  │
    │  │  Shape   │  │  Symmetry│      │     │  │  Remove  │  │  Remove  │  │
    │  │  Detect  │  │  Detect  │      │     │  │  Grid    │  │  Grid    │  │
    │  └────┬─────┘  └────┬─────┘      │     │  │  Code    │  │  CLI     │  │
    │       │             │            │     │  └────┬─────┘  └────┬─────┘  │
    │       └──────┬──────┘            │     │       │             │        │
    │              │                   │     │       └──────┬──────┘        │
    │              ▼                   │     │              │               │
    │       ┌──────────┐               │     │              ▼               │
    │       │  24.14   │               │     │       ┌──────────┐           │
    │       │  Role    │               │     │       │  24.19   │           │
    │       │  Infer   │               │     │       │  Delete  │           │
    │       └────┬─────┘               │     │       │  Files   │           │
    │            │                     │     │       └────┬─────┘           │
    │            ▼                     │     │            │                 │
    │  ┌──────────┐  ┌──────────┐      │     │            ▼                 │
    │  │  24.15   │  │  24.16   │      │     │  ┌──────────┐  ┌──────────┐  │
    │  │  Relation│  │  Import  │      │     │  │  24.20   │  │  24.21   │  │
    │  │  Infer   │  │  CLI     │      │     │  │  Update  │  │  Update  │  │
    │  └──────────┘  └──────────┘      │     │  │  Examples│  │  Tests   │  │
    │                                   │     │  └──────────┘  └──────────┘  │
    └───────────────────────────────────┘     └───────────────────────────────┘
                │                                         │
                └───────────────────┬─────────────────────┘
                                    │
                                    ▼
    WAVE 6 (Config & Telemetry) - Depends on Waves 4, 5
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                                                                         │
    │   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
    │   │   24.22        │  │   24.23        │  │   24.24        │           │
    │   │   Config       │  │   Error        │  │   Strict       │           │
    │   │   pxl.toml     │  │   Telemetry    │  │   Mode         │           │
    │   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
    │           │                   │                   │                     │
    └───────────┼───────────────────┼───────────────────┼─────────────────────┘
                │                   │                   │
                └───────────────────┼───────────────────┘
                                    │
                                    ▼
    WAVE 7 (Polish) - Depends on Wave 6
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                                                                         │
    │   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐           │
    │   │   24.25        │  │   24.26        │  │   24.27        │           │
    │   │   State        │  │   LSP          │  │   mdbook       │           │
    │   │   Rules        │  │   Updates      │  │   Rewrite      │           │
    │   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘           │
    │           │                   │                   │                     │
    │           └───────────────────┼───────────────────┘                     │
    │                               │                                         │
    │                               ▼                                         │
    │                       ┌────────────────┐                                │
    │                       │   24.28        │                                │
    │                       │   Final Docs   │                                │
    │                       │   & Spec       │                                │
    │                       └────────────────┘                                │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY
    ┌─────────────────────────────────────────────────────────────────────────┐
    │  Wave 1:  24.1                           (1 task, CRITICAL PATH)        │
    │  Wave 2:  24.2 ∥ 24.3 ∥ 24.4 → 24.5 → 24.6 ∥ 24.7 ∥ 24.8              │
    │  Wave 3:  24.9 ∥ 24.10 ∥ 24.11          (3 tasks in parallel)          │
    │  Wave 4:  24.12 ∥ 24.13 → 24.14 → 24.15 ∥ 24.16                        │
    │  Wave 5:  24.17 ∥ 24.18 → 24.19 → 24.20 ∥ 24.21                        │
    │  Wave 6:  24.22 ∥ 24.23 ∥ 24.24         (3 tasks in parallel)          │
    │  Wave 7:  24.25 ∥ 24.26 ∥ 24.27 → 24.28                                │
    │                                                                         │
    │  Legend: ∥ = parallel, → = sequential dependency                        │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## Wave 1: Foundation

### 24.1 JSON5 Parser [CRITICAL]

**Parallel:** No (blocking)
**Files:** `Cargo.toml`, `src/parser.rs`

Add JSON5 support for comments, trailing commas, unquoted keys.

**Deliverables:**
- Add `json5` crate dependency
- Update `parse_stream()` to use JSON5
- Support comments (`// ...`)
- Support trailing commas
- Support unquoted keys
- All existing tests pass

**Verification:**
```bash
echo '{type: "palette", name: "test", colors: {_: "#0000",}}' | pxl validate -
# Should pass (unquoted keys, trailing comma)
```

---

## Wave 2: Structured Core

### 24.2 RegionDef Struct

**Parallel:** Yes (with 24.3, 24.4)
**Depends:** 24.1
**Files:** `src/models.rs`

Define the `RegionDef` struct with all shape variants.

**Deliverables:**
```rust
pub struct RegionDef {
    // Shapes (exactly one)
    pub points: Option<Vec<[i32; 2]>>,
    pub line: Option<Vec<[i32; 2]>>,
    pub rect: Option<[i32; 4]>,
    pub stroke: Option<[i32; 4]>,
    pub ellipse: Option<[i32; 4]>,
    pub circle: Option<[i32; 3]>,
    pub polygon: Option<Vec<[i32; 2]>>,
    pub path: Option<String>,
    pub fill: Option<String>,

    // Compound
    pub union: Option<Vec<RegionDef>>,
    pub base: Option<Box<RegionDef>>,
    pub subtract: Option<Vec<RegionDef>>,
    pub intersect: Option<Vec<RegionDef>>,

    // Constraints
    pub within: Option<String>,
    pub except: Option<Vec<String>>,
    pub x: Option<[i32; 2]>,
    pub y: Option<[i32; 2]>,
    pub adjacent_to: Option<String>,

    // Modifiers
    pub symmetric: Option<Symmetric>,
    pub z: Option<i32>,
    pub round: Option<i32>,
    pub thickness: Option<i32>,

    // ... rest
}
```

---

### 24.3 Shape Rasterization

**Parallel:** Yes (with 24.2, 24.4)
**Depends:** 24.1
**Files:** `src/shapes.rs` (new)

Implement pixel rasterization for all primitives.

**Deliverables:**
- `rasterize_points(points) -> HashSet<(i32, i32)>`
- `rasterize_line(points) -> HashSet<(i32, i32)>` (Bresenham)
- `rasterize_rect(x, y, w, h) -> HashSet<(i32, i32)>`
- `rasterize_stroke(x, y, w, h, thickness) -> HashSet<(i32, i32)>`
- `rasterize_ellipse(cx, cy, rx, ry) -> HashSet<(i32, i32)>` (Midpoint)
- `rasterize_polygon(vertices) -> HashSet<(i32, i32)>` (Scanline fill)

**Verification:**
```rust
#[test]
fn test_rasterize_rect() {
    let pixels = rasterize_rect(0, 0, 3, 2);
    assert_eq!(pixels.len(), 6);
    assert!(pixels.contains(&(0, 0)));
    assert!(pixels.contains(&(2, 1)));
}
```

---

### 24.4 Path Parser (SVG-lite)

**Parallel:** Yes (with 24.2, 24.3)
**Depends:** 24.1
**Files:** `src/path.rs` (new)

Parse SVG-lite path syntax.

**Deliverables:**
- Parse `M`, `L`, `H`, `V`, `Z` commands
- Convert to polygon vertices
- Handle relative vs absolute coords

**Verification:**
```rust
let vertices = parse_path("M0,0 L5,0 L5,5 L0,5 Z");
assert_eq!(vertices, vec![[0,0], [5,0], [5,5], [0,5]]);
```

---

### 24.5 Compound Operations

**Parallel:** No
**Depends:** 24.2, 24.3, 24.4
**Files:** `src/shapes.rs`

Implement union, subtract, intersect.

**Deliverables:**
- `union(regions) -> HashSet<(i32, i32)>`
- `subtract(base, removals) -> HashSet<(i32, i32)>`
- `intersect(regions) -> HashSet<(i32, i32)>`

---

### 24.6 Fill Operation

**Parallel:** Yes (with 24.7, 24.8)
**Depends:** 24.5
**Files:** `src/shapes.rs`

Implement flood fill inside boundaries.

**Deliverables:**
- `flood_fill(boundary, seed) -> HashSet<(i32, i32)>`
- Auto-detect seed point if not provided
- Handle `except` token references

---

### 24.7 Modifiers

**Parallel:** Yes (with 24.6, 24.8)
**Depends:** 24.5
**Files:** `src/modifiers.rs` (new)

Implement region modifiers.

**Deliverables:**
- `apply_symmetric(pixels, axis, width, height)`
- `apply_range(pixels, x_range, y_range)`
- `apply_repeat(pixels, count, spacing)`
- `apply_jitter(pixels, jitter, seed)`

---

### 24.8 Structured Renderer

**Parallel:** Yes (with 24.6, 24.7)
**Depends:** 24.5
**Files:** `src/structured.rs` (new), `src/renderer.rs`

Render structured sprites to pixel buffer.

**Deliverables:**
- Parse `regions` from sprite
- Rasterize each region in z-order
- Apply modifiers
- Output pixel buffer

**Verification:**
```bash
cat > /tmp/test.pxl << 'EOF'
{type: "palette", name: "p", colors: {_: "#0000", o: "#000", f: "#F00"}}
{type: "sprite", name: "s", size: [8, 8], palette: "p", regions: {
  o: { stroke: [0, 0, 8, 8] },
  f: { fill: "inside(o)" }
}}
EOF
pxl render /tmp/test.pxl -o /tmp/test.png
# Should produce 8x8 red square with black outline
```

---

## Wave 3: Semantic Metadata

### 24.9 Roles in Palette

**Parallel:** Yes (with 24.10, 24.11)
**Depends:** Wave 2
**Files:** `src/models.rs`, `src/parser.rs`

Add `roles` field to Palette.

**Deliverables:**
- Add `roles: Option<HashMap<String, Role>>` to Palette
- Parse role values: `boundary`, `anchor`, `fill`, `shadow`, `highlight`
- Expose in LSP

---

### 24.10 Relationships

**Parallel:** Yes (with 24.9, 24.11)
**Depends:** Wave 2
**Files:** `src/models.rs`, `src/parser.rs`

Add `relationships` field to Palette.

**Deliverables:**
- Add `relationships: Option<HashMap<String, Relationship>>` to Palette
- Parse relationship types: `derives-from`, `contained-within`, `adjacent-to`, `paired-with`
- Validate no circular dependencies

---

### 24.11 Constraint Validation

**Parallel:** Yes (with 24.9, 24.10)
**Depends:** Wave 2
**Files:** `src/validate.rs`

Validate semantic constraints.

**Deliverables:**
- Validate `within` constraints
- Validate `adjacent-to` constraints
- Validate relationship references exist
- Detect circular relationships
- Warn on uncertain conditions

---

## Wave 4: Import Rewrite

### 24.12 Shape Detection

**Parallel:** Yes (with 24.13)
**Depends:** Wave 3
**Files:** `src/analyze.rs` (new)

Detect shapes from pixel regions.

**Deliverables:**
- `detect_rect(pixels) -> Option<[i32; 4]>` (bounding box check)
- `detect_ellipse(pixels) -> Option<[i32; 4]>` (roundness heuristic)
- `detect_line(pixels) -> Option<Vec<[i32; 2]>>` (Bresenham match)
- `detect_stroke(pixels) -> Option<[i32; 4]>` (hollow rect check)
- Confidence scoring
- Fall back to polygon

---

### 24.13 Symmetry Detection

**Parallel:** Yes (with 24.12)
**Depends:** Wave 3
**Files:** `src/analyze.rs`

Detect symmetric regions.

**Deliverables:**
- `detect_symmetry(pixels, width, height) -> Option<Symmetric>`
- Column-wise comparison for x-symmetry
- Row-wise comparison for y-symmetry

---

### 24.14 Role Inference

**Parallel:** No
**Depends:** 24.12, 24.13
**Files:** `src/analyze.rs`

Infer roles from region properties.

**Deliverables:**
- 1px on edges → `boundary`
- Small isolated (< 4px) → `anchor`
- Large interior → `fill`
- Darker than neighbor → `shadow`
- Lighter than neighbor → `highlight`
- Confidence scoring
- Warn when uncertain

---

### 24.15 Relationship Inference

**Parallel:** Yes (with 24.16)
**Depends:** 24.14
**Files:** `src/analyze.rs`

Infer relationships between regions.

**Deliverables:**
- Colors differ by lightness → `derives-from`
- Region inside another → `contained-within`
- Regions share boundary → `adjacent-to`
- Symmetric regions → `paired-with`

---

### 24.16 Import CLI

**Parallel:** Yes (with 24.15)
**Depends:** 24.14
**Files:** `src/import.rs`, `src/cli.rs`

Update import to output structured format.

**Deliverables:**
- Output structured regions instead of grid
- `--analyze` flag for role/relationship inference
- `--confidence` flag override
- `--hints` flag for token naming

**Verification:**
```bash
pxl import sprite.png --analyze -o sprite.pxl
cat sprite.pxl  # Should show regions, roles, relationships
```

---

## Wave 5: Grid Removal

### 24.17 Remove Grid Code

**Parallel:** Yes (with 24.18)
**Depends:** Wave 3
**Files:** `src/models.rs`, `src/parser.rs`, `src/renderer.rs`

Remove grid-related code from core.

**Deliverables:**
- Remove `grid` field from Sprite
- Remove `tokens` field from Sprite
- Remove grid parsing logic
- Remove grid rendering path
- Update Sprite to require `regions`

---

### 24.18 Remove Grid CLI

**Parallel:** Yes (with 24.17)
**Depends:** Wave 3
**Files:** `src/cli.rs`

Remove grid-specific CLI commands.

**Deliverables:**
- Remove `pxl alias` command
- Remove `pxl inline` command
- Remove `pxl sketch` command
- Remove `pxl grid` command
- Update help text

---

### 24.19 Delete Grid Files

**Parallel:** No
**Depends:** 24.17, 24.18
**Files:** Multiple

Delete grid-specific source files.

**Deliverables:**
- DELETE `src/alias.rs`
- DELETE `src/tokenizer.rs` (if exists)
- Remove grid-related modules from `src/lib.rs`

---

### 24.20 Update Examples

**Parallel:** Yes (with 24.21)
**Depends:** 24.19
**Files:** `examples/*.pxl`, `examples/*.jsonl`

Convert all examples to structured format.

**Deliverables:**
- Convert `coin.pxl` to structured
- Convert `hero.pxl` to structured
- Convert all other examples
- Rename `.jsonl` files to `.pxl`
- Delete v1 examples

---

### 24.21 Update Tests

**Parallel:** Yes (with 24.20)
**Depends:** 24.19
**Files:** `tests/**/*`

Update test fixtures and test code.

**Deliverables:**
- Convert test fixtures to structured
- Delete grid-specific tests
- Add structured format tests
- Update integration tests

---

## Wave 6: Config & Telemetry

### 24.22 Config pxl.toml

**Parallel:** Yes (with 24.23, 24.24)
**Depends:** Waves 4, 5
**Files:** `src/config.rs`

Add new config options.

**Deliverables:**
- `[format] version = 2`
- `[import] confidence_threshold`
- `[import] role_inference`
- `[validation] strict`
- `[telemetry] collect_errors`
- `[telemetry] error_log`

---

### 24.23 Error Telemetry

**Parallel:** Yes (with 24.22, 24.24)
**Depends:** Waves 4, 5
**Files:** `src/telemetry.rs` (new), `src/cli.rs`

Implement local error collection.

**Deliverables:**
- `--collect-errors` flag
- Write errors to `.pxl-errors.jsonl`
- Include timestamp, command, file, error type, context, suggestion
- Respect config setting

---

### 24.24 Strict Mode

**Parallel:** Yes (with 24.22, 24.23)
**Depends:** Waves 4, 5
**Files:** `src/validate.rs`, `src/cli.rs`

Enhance strict mode with debug help.

**Deliverables:**
- `--strict` flag
- Warnings become errors
- Actionable debug output with details
- Suggestions to proceed
- `--allow-overflow`, `--allow-orphans`, `--allow-cycles` flags

---

## Wave 7: Polish

### 24.25 State Rules

**Parallel:** Yes (with 24.26, 24.27)
**Depends:** Wave 6
**Files:** `src/models.rs`, `src/state.rs` (new)

Implement state rules system.

**Deliverables:**
- StateRules object type
- CSS selector parsing (MVP subset)
- `[token=name]`, `[role=type]`, `.state` selectors
- Filter application

---

### 24.26 LSP Updates

**Parallel:** Yes (with 24.25, 24.27)
**Depends:** Wave 6
**Files:** `src/lsp.rs`

Update LSP for structured format.

**Deliverables:**
- Completions for regions, shapes, modifiers
- Completions for roles, relationships
- Hover for token colors, shapes
- Diagnostics for constraints
- Remove grid-specific features

---

### 24.27 mdbook Rewrite

**Parallel:** Yes (with 24.25, 24.26)
**Depends:** Wave 6
**Files:** `docs/book/src/**/*.md`

Rewrite documentation for v2.

**Deliverables:**
- Rewrite format pages
- Delete grid CLI pages (alias, inline, sketch, grid)
- Delete grid demo pages
- Add new pages: regions, semantic, state-rules
- Update SUMMARY.md
- Update all examples

---

### 24.28 Final Docs & Spec

**Parallel:** No
**Depends:** 24.25, 24.26, 24.27
**Files:** `docs/spec/format.md`, `docs/primer.md`, `README.md`

Finalize all documentation.

**Deliverables:**
- Rewrite `docs/spec/format.md` for v2
- Rewrite `docs/primer.md`
- Update `README.md` examples
- Create `docs/migration.md`
- Final review pass

---

## Summary

| Wave | Tasks | Parallel Tasks | Dependencies |
|------|-------|----------------|--------------|
| 1 | 24.1 | 1 | None |
| 2 | 24.2-24.8 | 24.2∥24.3∥24.4, 24.6∥24.7∥24.8 | 24.1 |
| 3 | 24.9-24.11 | 24.9∥24.10∥24.11 | Wave 2 |
| 4 | 24.12-24.16 | 24.12∥24.13, 24.15∥24.16 | Wave 3 |
| 5 | 24.17-24.21 | 24.17∥24.18, 24.20∥24.21 | Wave 3 |
| 6 | 24.22-24.24 | 24.22∥24.23∥24.24 | Waves 4,5 |
| 7 | 24.25-24.28 | 24.25∥24.26∥24.27 | Wave 6 |

**Total Tasks:** 28
**Critical Path:** 24.1 → 24.2 → 24.5 → 24.8 → 24.11 → 24.17 → 24.19 → 24.22 → 24.28

---

## Success Criteria

1. JSON5 parsing works (comments, trailing commas, unquoted keys)
2. Structured sprites render correctly
3. All shape primitives rasterize correctly
4. Compound operations work (union, subtract, intersect)
5. Modifiers work (symmetric, fill, constraints)
6. Roles and relationships parse and validate
7. Import outputs structured format with shape detection
8. `--analyze` produces roles and relationships
9. Grid code completely removed
10. All examples converted
11. All tests pass
12. mdbook documentation complete
13. Error telemetry works
14. Strict mode provides helpful debug output
