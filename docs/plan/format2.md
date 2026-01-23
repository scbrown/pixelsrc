# Pixelsrc Format v2 Specification

**Status:** Draft
**Version:** 0.2.0

---

## Overview

Format v2 replaces the grid-based format with a **structured region** approach. Sprites are defined as semantic shapes, not pixel grids.

This is a breaking change from v1. The old grid format is removed entirely.

### Design Goals

| Goal | Description |
|------|-------------|
| **Context efficient** | Scales with semantic complexity, not pixel count |
| **Edit friendly** | Change a number, not rewrite pixel rows |
| **Semantically rich** | Roles and relationships are explicit |
| **AI optimized** | Describe intent, compiler resolves pixels |
| **JSON5** | Comments, trailing commas, unquoted keys |

### What Changes from v1

| Aspect | v1 | v2 |
|--------|----|----|
| Sprite definition | `grid: ["{a}{b}{c}", ...]` | `regions: { token: shape }` |
| Token syntax | `{token}` with braces | `token` without braces |
| File format | Strict JSON | JSON5 |
| Semantic metadata | None | Roles, relationships |
| Size | Scales with pixels | Scales with complexity |

### What's Removed

- `grid` field
- `tokens` field (aliases)
- `{braces}` around token names
- RLE compression
- Greedy token parsing
- `.jsonl` legacy format

---

## MVP Scope

### Included in MVP

| Category | Features |
|----------|----------|
| **Format** | JSON5 parsing (comments, trailing commas, unquoted keys) |
| **Sprites** | `regions` field with all shape primitives |
| **Shapes** | `points`, `line`, `rect`, `stroke`, `ellipse`, `circle`, `polygon`, `path`, `fill` |
| **Compounds** | `union`, `subtract`, `intersect`, `except` |
| **Modifiers** | `symmetric`, `z`, `round`, `thickness`, `x`/`y` range |
| **Transforms** | `repeat`, `spacing`, `offset-alternate`, `transform` (basic), `jitter`, `seed` |
| **Auto-gen** | `auto-outline`, `auto-shadow`, `background` shorthand |
| **Constraints** | `within` (validation), `adjacent-to` (validation) |
| **Palettes** | `colors`, `roles`, `relationships` |
| **State Rules** | `StateRules` type with MVP CSS selector subset |
| **Selectors** | `[token=X]`, `[token*=X]`, `[role=X]`, `.state` |
| **CLI** | `render`, `import --analyze`, `validate --strict`, `fmt`, `show --roles` |
| **Config** | `pxl.toml` with confidence, telemetry, strict mode settings |
| **Telemetry** | `--collect-errors` flag, `.pxl-errors.jsonl` log |
| **LSP** | Completions, hover, diagnostics for structured format |

### Deferred to Post-MVP

| Feature | Reason |
|---------|--------|
| Semantic rotation algorithm | Complex; basic rotation available |
| Equipment/layering system | Game-specific, needs more design |
| Procedural variation | Lower priority |
| Hit region export | Game-specific metadata |
| Animation region targeting | Per-token keyframes need more design |
| Complex CSS selectors | `:not()`, combinators, etc. |
| Runtime metadata export | Focus on compile-time first |
| Procedural shapes | `noise`, etc. |
| Shape libraries | `use: "primitives/..."` |
| Expressions in shapes | `rect: [0, 0, "var(--x)", 2]` |

### MVP Success Criteria

1. JSON5 parsing works everywhere
2. Structured sprites render correctly
3. All shape primitives rasterize
4. Compound operations work
5. Modifiers work (symmetric, fill, constraints)
6. Roles and relationships validate
7. Import outputs structured format
8. `--analyze` produces roles/relationships
9. Grid code completely removed
10. Error telemetry captures failures
11. Strict mode provides debug help
12. All examples converted
13. mdbook documentation complete

---

## Part 1: Sprites

### Basic Structure

```json5
{
  type: "sprite",
  name: "hero",
  size: [16, 16],
  palette: "hero_palette",
  regions: {
    outline: { stroke: [0, 0, 16, 16] },
    skin: { fill: "inside(outline)", y: [4, 12] },
    eye: { points: [[5, 6]], symmetric: "x" }
  }
}
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `type` | `"sprite"` | Object type |
| `name` | string | Unique identifier |
| `size` | `[w, h]` | Canvas dimensions in pixels |
| `regions` | object | Token → shape definition map |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `palette` | string | Reference to palette by name (if omitted, all tokens render white with warning) |
| `background` | string | Token to fill empty pixels (default: `_`) |
| `origin` | `[x, y]` | Anchor point for transforms |
| `metadata` | object | Custom data passthrough |

### Region Resolution Order

Regions are processed in two passes:

1. **Shape resolution** (definition order): Each region's pixels are computed. Pixel-affecting operations (`fill: "inside(X)"`, `except: [X]`, `auto-outline: X`) require X to be defined earlier.

2. **Validation** (after all resolved): Constraints (`within`, `adjacent-to`) are checked. These can reference any region.

This enables streaming while catching errors:
```json5
regions: {
  outline: { stroke: [0, 0, 16, 16] },     // defined first
  skin: { fill: "inside(outline)" },        // OK: outline exists
  eye: { rect: [5, 6, 2, 2] },
  pupil: { points: [[6, 7]], within: "eye" } // OK: within is validation-only
}
```

**Error** (forward reference in pixel-affecting operation):
```json5
regions: {
  skin: { fill: "inside(outline)" },  // ERROR: outline not yet defined
  outline: { stroke: [0, 0, 16, 16] }
}
```

---

## Part 2: Shape Primitives

### Points

Individual pixels at specific coordinates.

```json5
eye: { points: [[2, 3], [5, 3]] }
```

### Line

Bresenham line between points.

```json5
mouth: { line: [[3, 6], [5, 6]] }

// Multiple segments
crack: { line: [[0, 0], [2, 3], [4, 2]] }
```

Optional: `thickness` (default: 1)

### Rectangle

Filled rectangle.

```json5
body: { rect: [2, 4, 12, 8] }  // [x, y, width, height]
```

Optional: `round` (corner radius)

### Stroke

Rectangle outline (unfilled).

```json5
outline: { stroke: [0, 0, 16, 16] }
```

Optional: `thickness`, `round`

### Ellipse

Filled ellipse.

```json5
head: { ellipse: [8, 4, 6, 4] }  // [cx, cy, rx, ry]
```

### Circle

Shorthand for equal-radius ellipse.

```json5
dot: { circle: [4, 4, 2] }  // [cx, cy, r]
```

### Polygon

Filled polygon from vertices.

```json5
hair: {
  polygon: [[4, 0], [12, 0], [14, 4], [2, 4]]
}
```

### Path

SVG-lite path syntax.

```json5
complex: {
  path: "M2,0 L6,0 L8,2 L8,6 L6,8 L2,8 L0,6 L0,2 Z"
}
```

**Supported commands:**
- `M x,y` - Move to
- `L x,y` - Line to
- `H x` - Horizontal line to
- `V y` - Vertical line to
- `Z` - Close path

Curves (C, Q, A) are not supported—they don't make sense for pixel art.

### Fill

Flood fill inside a boundary.

```json5
skin: { fill: "inside(outline)" }
```

Optional: `seed: [x, y]` (starting point, auto-detected if omitted)

---

## Part 3: Compound Operations

### Union

Combine multiple shapes.

```json5
hair: {
  union: [
    { rect: [2, 0, 12, 2] },
    { rect: [0, 2, 16, 2] }
  ]
}
```

### Subtract

Remove shapes from a base.

```json5
face: {
  base: { rect: [2, 4, 12, 8] },
  subtract: [
    { points: [[5, 6], [10, 6]] }  // eye holes
  ]
}
```

Or using token references:

```json5
skin: {
  fill: "inside(outline)",
  except: ["eye", "mouth"]
}
```

### Intersect

Keep only overlapping area.

```json5
visor: {
  intersect: [
    { rect: [2, 4, 12, 4] },
    { fill: "inside(helmet)" }
  ]
}
```

---

## Part 4: Modifiers

### Symmetry

Auto-mirror across axis.

```json5
eye: {
  points: [[4, 6]],
  symmetric: "x"  // mirrors to [11, 6] for 16-wide sprite
}
```

**Values:**
- `"x"` - Horizontal mirror (around canvas center)
- `"y"` - Vertical mirror (around canvas center)
- `"xy"` - Both axes (4-way symmetry)
- `8` - Mirror around specific x-coordinate

### Range Constraints

Limit region to specific rows or columns.

```json5
hair: {
  fill: "inside(outline)",
  y: [0, 4]  // rows 0-4 inclusive
}

"left-arm": {
  fill: "inside(outline)",
  x: [0, 8]  // columns 0-8 inclusive
}
```

### Containment

Validate that region stays within another.

```json5
pupil: {
  points: [[4, 6]],
  within: "eye"  // compiler validates this
}
```

Note: `within` is a validation constraint (checked after all regions resolved). It's distinct from `fill: "inside(X)"` which is a pixel-affecting operation.

### Adjacency

Ensure region touches another.

```json5
shadow: {
  fill: "inside(outline)",
  "adjacent-to": "skin",
  y: [10, 14]
}
```

### Z-Order

Explicit render order (default: definition order).

```json5
detail: {
  points: [[8, 8]],
  z: 100  // renders on top
}
```

---

## Part 5: Transform Modifiers

### Repeat

Tile a shape.

```json5
bricks: {
  rect: [0, 0, 4, 2],
  repeat: [8, 16],
  spacing: [1, 1],
  "offset-alternate": true
}
```

### Geometric Transform

Apply rotation, translation, scale.

```json5
sword: {
  line: [[0, 0], [0, 8]],
  transform: "rotate(45deg) translate(12, 4)"
}
```

**Supported transforms:**
- `translate(x, y)`
- `rotate(angle)` - supports `deg` or `rad` (basic rotation, may be lossy for pixel art)
- `scale(x, y)` or `scale(n)`
- `flip-x`, `flip-y`

Note: Basic rotation is available but may produce artifacts. Semantic-aware rotation (using roles to preserve anchors/boundaries) is deferred to post-MVP.

### Jitter

Controlled randomness.

```json5
grass: {
  points: [[0, 15], [4, 15], [8, 15], [12, 15]],
  jitter: { y: [-2, 0] },
  seed: 42
}
```

---

## Part 6: Auto-Generation

### Auto-Outline

Generate outline around a region.

```json5
outline: {
  "auto-outline": "body",
  thickness: 1
}
```

### Auto-Shadow

Generate drop shadow.

```json5
shadow: {
  "auto-shadow": "body",
  offset: [1, 1]
}
```

### Background

Shorthand to fill all unoccupied pixels.

```json5
_: "background"
```

---

## Part 7: Palettes

### Basic Structure

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    eye: "#4169E1"
  }
}
```

Token `_` is conventionally used for transparency.

### CSS Variables

```json5
{
  type: "palette",
  name: "hero",
  colors: {
    "--base-skin": "#FFD5B4",
    skin: "var(--base-skin)",
    "skin-shadow": "color-mix(in oklch, var(--base-skin), black 30%)"
  }
}
```

### Roles

Classify tokens by visual/functional purpose.

```json5
{
  type: "palette",
  name: "hero",
  colors: { /* ... */ },
  roles: {
    outline: "boundary",
    eye: "anchor",
    skin: "fill"
  }
}
```

| Role | Meaning | Transform Behavior |
|------|---------|-------------------|
| `boundary` | Edge-defining (outlines) | High priority, preserve connectivity |
| `anchor` | Critical details (eyes) | Must survive transforms (min 1px) |
| `fill` | Interior mass (skin, clothes) | Can shrink, low priority |
| `shadow` | Depth indicators | Derives from parent |
| `highlight` | Light indicators | Derives from parent |

### Relationships

Explicit semantic connections.

```json5
{
  type: "palette",
  name: "hero",
  colors: { /* ... */ },
  relationships: {
    "skin-shadow": {
      "derives-from": "skin",
      transform: "darken(30%)"
    },
    pupil: {
      "contained-within": "eye"
    },
    "left-eye": {
      "paired-with": "right-eye"
    }
  }
}
```

| Relationship | Meaning |
|--------------|---------|
| `derives-from` | Color derived from another token |
| `contained-within` | Spatially inside another region |
| `adjacent-to` | Must touch specified region |
| `paired-with` | Symmetric relationship |

---

## Part 8: State Rules

Define visual states without separate sprites.

```json5
{
  type: "state_rules",
  name: "combat",
  rules: {
    ".damaged [token]": {
      filter: "brightness(2)",
      animation: "flash 0.1s 3"
    },
    ".poisoned [token=skin]": {
      filter: "hue-rotate(80deg)"
    },
    "[role=boundary]": {
      filter: "drop-shadow(0 0 1px black)"
    }
  }
}
```

### CSS Selector Subset (MVP)

| Selector | Example | Meaning |
|----------|---------|---------|
| `[token=name]` | `[token=eye]` | Exact token match |
| `[token*=str]` | `[token*=skin]` | Token contains substring |
| `[role=type]` | `[role=boundary]` | Role-based selection |
| `.state` | `.damaged` | State class |

**Not supported in MVP:** combinators, pseudo-classes, `^=`/`$=`/`|=` operators

---

## Part 9: Runtime vs Compile-Time

### Compile-Time (pxl render)

| Feature | Resolution |
|---------|------------|
| Region rasterization | Shapes → pixels |
| Fill operations | Flood fill executed |
| Constraint validation | `within`, `adjacent-to` checked |
| Role inference | Auto-detect from shape type |
| State pre-rendering | `.damaged` → separate PNG |

### Runtime (exported metadata)

Secondary priority for MVP:

| Feature | Export Format |
|---------|---------------|
| Region bounds | `{ eye: { x: 5, y: 6, w: 6, h: 2 } }` |
| Role classifications | `{ boundary: ["outline"], anchor: ["eye"] }` |
| Relationship graph | `{ "skin-shadow": { "derives-from": "skin" } }` |

---

## Part 10: JSON5 Schema

### Sprite

```typescript
interface Sprite {
  type: "sprite";
  name: string;
  size: [number, number];
  regions: Record<string, RegionDef>;

  palette?: string;
  background?: string;
  origin?: [number, number];
  metadata?: Record<string, unknown>;
}
```

### RegionDef

```typescript
interface RegionDef {
  // Shape (exactly one, or compound)
  points?: [number, number][];
  line?: [number, number][];
  rect?: [number, number, number, number];
  stroke?: [number, number, number, number];
  ellipse?: [number, number, number, number];
  circle?: [number, number, number];
  polygon?: [number, number][];
  path?: string;
  fill?: string;  // "inside(token)" - pixel-affecting, requires forward def

  // Compound
  union?: RegionDef[];
  base?: RegionDef;
  subtract?: RegionDef[];
  intersect?: RegionDef[];

  // Pixel-affecting modifiers (require forward definition)
  except?: string[];        // subtract these tokens' pixels
  "auto-outline"?: string;  // generate outline around token
  "auto-shadow"?: string;   // generate shadow from token
  offset?: [number, number]; // for auto-shadow

  // Validation constraints (checked after all regions resolved)
  within?: string;          // must be inside token's bounds
  "adjacent-to"?: string;   // must touch token

  // Range constraints
  x?: [number, number];
  y?: [number, number];

  // Modifiers
  symmetric?: "x" | "y" | "xy" | number;
  z?: number;
  round?: number;
  thickness?: number;

  // Transforms (basic transforms available; semantic-aware transforms deferred)
  repeat?: [number, number];
  spacing?: [number, number];
  "offset-alternate"?: boolean;
  transform?: string;  // rotate/translate/scale - basic (may be lossy)
  jitter?: { x?: [number, number]; y?: [number, number] };
  seed?: number;
}
```

Note: Roles and relationships are defined in Palette, not RegionDef. A token has the same role everywhere it's used.

### Palette

```typescript
interface Palette {
  type: "palette";
  name: string;
  colors: Record<string, string>;
  roles?: Record<string, Role>;
  relationships?: Record<string, Relationship>;
}

interface Relationship {
  "derives-from"?: string;
  "contained-within"?: string;
  "adjacent-to"?: string;
  "paired-with"?: string;
  transform?: string;
}
```

### StateRules

```typescript
interface StateRules {
  type: "state_rules";
  name: string;
  rules: Record<string, StateEffect>;
}

interface StateEffect {
  filter?: string;
  animation?: string;
  opacity?: number;
}
```

---

## Part 11: CLI Impact

### Removed Commands

| Command | Reason |
|---------|--------|
| `pxl compact` | No grid format to compact |
| `pxl expand` | No grid format to expand |
| `pxl inline` | Grid-specific |
| `pxl alias` | Grid-specific |
| `pxl sketch` | Grid-specific |

### Modified Commands

| Command | Changes |
|---------|---------|
| `pxl render` | Only structured format |
| `pxl import` | Outputs structured format, `--analyze` for semantic detection |
| `pxl fmt` | JSON5 formatting |
| `pxl validate` | Validate constraints, roles, relationships |
| `pxl show` | Display regions, role annotations |
| `pxl analyze` | Report semantic metadata |

### New Flags

```bash
# Import with analysis
pxl import sprite.png                    # basic structured output
pxl import sprite.png --analyze          # with role/relationship inference
pxl import sprite.png --hints "o=black"  # color → token name hints

# Validation
pxl validate sprite.pxl                  # lenient (default)
pxl validate sprite.pxl --strict         # error on warnings

# Display
pxl show sprite.pxl --roles              # annotate with roles
pxl show sprite.pxl --regions            # show region boundaries
```

---

## Part 12: LSP Impact

### Completions

| Context | Completions |
|---------|-------------|
| `role:` | `boundary`, `anchor`, `fill`, `shadow`, `highlight` |
| Inside `regions:` | Shape primitives, modifiers |
| Inside `relationships:` | Relationship types |
| Token references | Tokens from current sprite/palette |

### Hover

| Target | Information |
|--------|-------------|
| Token name | Color preview from palette |
| Role value | Description, transform behavior |
| Shape primitive | Rendered pixel preview |
| Relationship | Related tokens, meaning |

### Diagnostics

| Diagnostic | Severity |
|------------|----------|
| Unknown token reference | Error |
| Constraint violation (`within`) | Warning |
| Circular relationship | Error |
| Region outside canvas | Warning |
| Overlapping without z-order | Info |

### Code Actions

| Trigger | Action |
|---------|--------|
| Detected symmetry | Suggest `symmetric` modifier |
| Similar colors | Suggest `derives-from` |
| Small isolated region | Suggest `anchor` role |
| Edge region | Suggest `boundary` role |

---

## Part 13: PNG → PXL Conversion

### Basic Import

```bash
pxl import sprite.png -o sprite.pxl
```

Produces:
- Connected regions as shapes
- Generic token names: `c1`, `c2`, `c3`, ...
- Shape detection: rect, ellipse, line where possible
- Fallback to polygon for irregular shapes

### Analyzed Import

```bash
pxl import sprite.png --analyze -o sprite.pxl
```

Additional detection:

**Color analysis:**
| Detection | Method | Token Name |
|-----------|--------|------------|
| Transparency | Alpha = 0 | `_` |
| Black outline | Darkest on edges | `outline` |
| Background | Most frequent | `bg` |

**Role inference:**
| Heuristic | Role |
|-----------|------|
| 1px on edges | `boundary` |
| Small isolated (< 4px) | `anchor` |
| Large interior | `fill` |
| Darker than neighbor | `shadow` |
| Lighter than neighbor | `highlight` |

**Relationship inference:**
| Heuristic | Relationship |
|-----------|--------------|
| Colors differ by lightness | `derives-from` |
| Region inside another | `contained-within` |
| Regions share boundary | `adjacent-to` |
| Symmetric regions | `paired-with` |

**Shape detection:**
| Detection | Algorithm | Output |
|-----------|-----------|--------|
| Rectangles | Bounding box + fill | `rect` |
| Ellipses | Roundness heuristic | `ellipse` |
| Lines | Bresenham matching | `line` |
| Outlines | Edge detection | `stroke` |
| Symmetry | Axis comparison | `symmetric` modifier |

### Hints

Guide token naming:

```bash
pxl import sprite.png --hints "outline=#000,skin=#FFD5B4"
```

---

## Part 14: Documentation Impact

### Files to Rewrite

| File | Changes |
|------|---------|
| `docs/spec/format.md` | Complete rewrite for v2 |
| `docs/primer.md` | All new examples |
| `README.md` | Update feature list |

### Files to Remove

Grid-specific documentation and examples.

### New Documentation

| File | Content |
|------|---------|
| `docs/tutorials/structured.md` | Creating sprites with regions |
| `docs/tutorials/semantic.md` | Roles and relationships |
| `docs/migration.md` | v1 → v2 migration guide |

---

## Part 15: Codebase Impact

### Files to Modify Significantly

| File | Changes |
|------|---------|
| `src/models.rs` | Remove `grid`, add `regions`, `RegionDef` |
| `src/parser.rs` | JSON5 parsing, structured format |
| `src/renderer.rs` | Shape rasterization, region rendering |
| `src/import.rs` | Output structured format, shape detection |
| `src/fmt.rs` | JSON5 formatting |
| `src/validate.rs` | Constraint validation |
| `src/lsp.rs` | New completions, hover, diagnostics |
| `src/cli.rs` | Remove grid commands, update flags |

### Files to Remove/Gut

| File | Reason |
|------|--------|
| `src/alias.rs` | Grid-specific |
| `src/tokenizer.rs` | Grid parsing |
| Grid-related tests | No longer applicable |

### New Files

| File | Purpose |
|------|---------|
| `src/structured.rs` | Region → pixel rasterization |
| `src/shapes.rs` | Shape primitives (rect, ellipse, line, etc.) |
| `src/analyze.rs` | Import analysis heuristics |

---

## Part 16: Error Telemetry

### Local Error Collection

Opt-in error gathering for debugging:

```bash
# Enable via CLI flag
pxl render sprite.pxl --collect-errors

# Or via config
# pxl.toml
[telemetry]
collect_errors = true
error_log = ".pxl-errors.jsonl"
```

### Error Log Format

```json5
{
  timestamp: "2024-01-15T10:30:00Z",
  command: "render",
  file: "sprite.pxl",
  error_type: "constraint_violation",
  message: "Region 'pupil' not inside 'eye'",
  context: { line: 45, column: 12 },
  suggestion: "Check that eye region is defined before pupil"
}
```

### Strict Mode Behavior

When `--strict` is enabled:

1. Warnings become errors
2. Provide actionable debug output:
   ```
   ERROR: Region 'pupil' extends outside 'eye' bounds

   Details:
     pupil pixels: [[4,6], [5,6]]
     eye bounds: x=[5,6], y=[5,6]
     out-of-bounds: [[4,6]]

   Suggestions:
     - Move pupil to x=5 or x=6
     - Expand eye region to include x=4
     - Use --allow-overflow to proceed anyway
   ```

3. Offer flags to proceed:
   - `--allow-overflow`: Ignore bounds violations
   - `--allow-orphans`: Ignore unresolved references
   - `--allow-cycles`: Ignore circular relationships

---

## Part 17: Configuration

### pxl.toml Additions

```toml
[format]
version = 2  # Require v2 structured format

[import]
confidence_threshold = 0.7  # Shape detection confidence (0.0-1.0)
role_inference = true       # Auto-infer roles
relationship_inference = true  # Auto-infer relationships

[validation]
strict = false  # Warnings as errors
lenient_bounds = true  # Allow slight out-of-bounds

[telemetry]
collect_errors = false
error_log = ".pxl-errors.jsonl"

[output]
json5 = true  # Output JSON5 (vs strict JSON)
```

### CLI Overrides

```bash
pxl import sprite.png --confidence 0.9
pxl validate sprite.pxl --strict
pxl render sprite.pxl --collect-errors
```

---

## Part 18: mdbook Updates

### Pages to Rewrite

| Page | Changes |
|------|---------|
| `format/overview.md` | Complete rewrite for structured format |
| `format/sprite.md` | Regions instead of grid |
| `format/palette.md` | Add roles, relationships |
| `getting-started/quick-start.md` | New examples |
| `getting-started/concepts.md` | Structured format concepts |
| `ai-generation/*.md` | All new examples |

### Pages to Remove

| Page | Reason |
|------|--------|
| `cli/alias.md` | Grid-specific |
| `cli/inline.md` | Grid-specific |
| `cli/sketch.md` | Grid-specific |
| `cli/grid.md` | Grid-specific |
| `demos/cli_alias.md` | Grid-specific |
| `demos/cli_inline.md` | Grid-specific |
| `demos/cli_grid.md` | Grid-specific |

### New Pages

| Page | Content |
|------|---------|
| `format/regions.md` | Shape primitives, modifiers |
| `format/semantic.md` | Roles, relationships |
| `format/state-rules.md` | State system |
| `tutorials/structured.md` | Creating structured sprites |
| `migration/v1-to-v2.md` | Migration guide |

### SUMMARY.md Update

```markdown
# Format
- [Overview](./format/overview.md)
- [Sprite](./format/sprite.md)
- [Regions](./format/regions.md)        # NEW
- [Palette](./format/palette.md)
- [Semantic Metadata](./format/semantic.md)  # NEW
- [State Rules](./format/state-rules.md)     # NEW
- [Animation](./format/animation.md)
- [Composition](./format/composition.md)
# Removed: alias, inline, sketch, grid demos
```

---

## Part 19: Integration Tests

### Test Categories

| Category | Location | Action |
|----------|----------|--------|
| Parser tests | `tests/parser/` | Rewrite for structured |
| Render tests | `tests/render/` | Update fixtures |
| CLI tests | `tests/cli/` | Remove grid commands |
| Import tests | `tests/import/` | Structured output |
| LSP tests | `tests/lsp/` | New completions |

### Fixture Migration

All `.pxl` and `.jsonl` fixtures need conversion:

```bash
# Location
tests/fixtures/valid/*.pxl
tests/fixtures/valid/*.jsonl
tests/fixtures/invalid/*.pxl
examples/*.pxl
examples/*.jsonl
```

### New Test Cases

| Test | Purpose |
|------|---------|
| `structured_basic.pxl` | Simple structured sprite |
| `structured_compound.pxl` | Union, subtract, intersect |
| `structured_modifiers.pxl` | Symmetric, constraints |
| `roles_basic.pxl` | Role definitions |
| `relationships.pxl` | Relationship validation |
| `state_rules.pxl` | State system |
| `import_shapes.png` | Shape detection |
| `import_symmetry.png` | Symmetry detection |

---

## Part 20: Grid Removal Checklist

### Code Files

| File | Action |
|------|--------|
| `src/models.rs` | Remove `grid` field, `tokens` field |
| `src/parser.rs` | Remove grid parsing |
| `src/renderer.rs` | Remove grid rendering path |
| `src/alias.rs` | DELETE entire file |
| `src/tokenizer.rs` | DELETE entire file |
| `src/cli.rs` | Remove `alias`, `inline`, `sketch`, `grid` commands |
| `src/fmt.rs` | Remove grid formatting |
| `src/lsp.rs` | Remove grid completions |
| `src/import.rs` | Output structured only |
| `src/validate.rs` | Remove grid validation |

### Documentation

| File | Action |
|------|--------|
| `docs/spec/format.md` | Rewrite |
| `docs/primer.md` | Rewrite |
| `README.md` | Update examples |
| `docs/book/src/cli/alias.md` | DELETE |
| `docs/book/src/cli/inline.md` | DELETE |
| `docs/book/src/cli/sketch.md` | DELETE |
| `docs/book/src/cli/grid.md` | DELETE |
| `docs/book/src/demos/cli_alias.md` | DELETE |
| `docs/book/src/demos/cli_inline.md` | DELETE |
| `docs/book/src/demos/cli_grid.md` | DELETE |

### Examples

| File | Action |
|------|--------|
| `examples/*.pxl` | Convert to structured |
| `examples/*.jsonl` | Convert to structured, rename to `.pxl` |

### Tests

| Pattern | Action |
|---------|--------|
| `tests/**/grid*.rs` | DELETE |
| `tests/**/alias*.rs` | DELETE |
| `tests/fixtures/**/*.jsonl` | Convert or delete |

---

## Part 21: Implementation Phases

See `docs/plan/tasks/phase24.md` for detailed task breakdown with dependencies.

**Summary:**
- Wave 1: JSON5 Foundation
- Wave 2: Structured Core (shapes, compounds, modifiers)
- Wave 3: Semantic Metadata (roles, relationships)
- Wave 4: Import Rewrite (shape detection, analysis)
- Wave 5: Grid Removal (code, docs, tests)
- Wave 6: Error Telemetry & Config
- Wave 7: State Rules & Polish

---

## Part 22: Decisions

### Resolved

1. **Shape detection confidence**: Configurable via `pxl.toml`. Default: conservative (70%). Fall back to polygon when below threshold. `--confidence` flag overrides.

2. **Role inference warnings**: Warn when heuristics are uncertain (< 80% confidence). In `--strict` mode, warnings become errors with debug help.

3. **Token naming in import**: Short names (`c1`, `c2`, `c3`). Semantic metadata (roles, relationships) provides meaning.

4. **Strict mode behavior**: Warnings become errors. Provide actionable debug output with suggestions to proceed.

5. **JSON5**: Enabled everywhere (all `.pxl` files, stdin, config).

### Deferred to Post-MVP

1. Semantic rotation algorithms
2. Equipment/layering system
3. Procedural variant generation
4. Hit region export
5. Animation region targeting
6. Complex CSS selectors

---

## Appendix A: Complete Example

```json5
// hero.pxl
{
  type: "palette",
  name: "hero",
  colors: {
    _: "transparent",
    outline: "#000000",
    skin: "#FFD5B4",
    "skin-shadow": "#D4A574",
    hair: "#8B4513",
    eye: "#4169E1",
    pupil: "#000000",
    shirt: "#E74C3C",
  },
  roles: {
    outline: "boundary",
    eye: "anchor",
    pupil: "anchor",
    skin: "fill",
    hair: "fill",
    shirt: "fill",
    "skin-shadow": "shadow",
  },
  relationships: {
    "skin-shadow": { "derives-from": "skin" },
    pupil: { "contained-within": "eye" },
  },
}

{
  type: "sprite",
  name: "hero",
  size: [16, 24],
  palette: "hero",
  regions: {
    // Background
    _: "background",

    // Head outline
    "head-outline": {
      stroke: [4, 0, 8, 10],
      round: 2,
    },

    // Hair (top of head)
    hair: {
      fill: "inside(head-outline)",
      y: [0, 4],
    },

    // Face
    skin: {
      fill: "inside(head-outline)",
      y: [4, 10],
      except: ["eye", "pupil"],
    },

    // Eyes (symmetric)
    eye: {
      rect: [5, 5, 2, 2],
      symmetric: "x",
    },

    // Pupils
    pupil: {
      points: [[6, 6]],
      symmetric: "x",
      within: "eye",
    },

    // Body outline
    "body-outline": {
      stroke: [3, 10, 10, 14],
    },

    // Shirt
    shirt: {
      fill: "inside(body-outline)",
    },

    // Shadow on skin
    "skin-shadow": {
      fill: "inside(head-outline)",
      y: [8, 10],
      "adjacent-to": "skin",
    },
  },
}
```

---

## Appendix B: Size Comparison

| Sprite | v1 Grid | v2 Structured |
|--------|---------|---------------|
| 8×8 simple | 400 chars | 200 chars |
| 16×16 character | 2,000 chars | 400 chars |
| 32×32 detailed | 8,000 chars | 600 chars |
| 64×64 complex | 35,000 chars | 800 chars |

Structured format scales with semantic complexity, not pixel count.

---

## Changelog

- v0.2.0 - Removed grid format, simplified token syntax (no braces)
- v0.1.0 - Initial draft (unified Phase 25 + structured + semantic specs)
