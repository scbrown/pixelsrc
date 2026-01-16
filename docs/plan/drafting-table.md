# Plan: The Drafting Table & Drafting Machine

**Status:** Planning
**Related Phases:** [Phase 12 (Tiling)](./phase-12-tiling.md), [Phase 14 (Analysis)](./phase-14-analyze.md), [Phase 17 (Grid Display)](./colored-grid-display.md)

## Vision

Transform the `pxl` toolchain from a passive compiler into an active **Drafting Table** for GenAI agents.

While `pixelsrc` provides a readable *format*, the **Drafting Machine** provides the *spatial reasoning engine* that LLMs lack. It bridges the gap between the "Implicit over explicit" tenet (format) and the explicit, coordinate-based instructions LLMs often prefer during iterative editing.

### Alignment with Core Tenets

1.  **Expressability**: The Drafting Machine handles the mechanical drudgery (grid alignment, string counting), allowing the Agent to use verbose, semantic token names (`{brick_red}`, `{shadow_deep}`) without worrying about line length limits or typos.
2.  **GenAI-First**: LLMs suffer from "context drift" and poor spatial indexing in raw text. The Drafting Machine provides stable, coordinate-based editing tools (`pxl draw`) and "scaffolding" that acts as an external spatial memory.
3.  **Source, Not Export**: This tooling reinforces that the `.pxl` file is the living source of truth, editable by both human and machine tools simultaneously.

---

## 1. The Drafting Machine (CLI Features)

These features formalize the "Region-Based Sprite Editing" concept from the [BACKLOG](../BACKLOG.md). They provide a safe API for agents to modify sprites without rewriting entire files.

### A. `pxl scaffold` (The Blueprints)
Generates valid structures, leveraging **Phase 12 (Composition Tiling)** for large scale generation.

*   **Usage**: `pxl scaffold sprite --name "hero" --size 16x16 --palette "medieval"`
    *   *Result*: A 16x16 grid of `{_}` tokens.
*   **Usage**: `pxl scaffold composition --name "level" --size 128x128 --tile-size 16x16`
    *   *Result*: A valid composition object with empty tile sprites and a character map pre-filled with placeholders. This directly implements the "Tiling" strategy for large-image generation.

### B. `pxl draw` (The Stylus)
Precise, coordinate-based manipulation. This moves spatial logic from the LLM's "brain" to the Rust binary.

*   **Usage**: `pxl draw "hero" --set 5,10="{eye}"`
*   **Usage**: `pxl draw "hero" --rect 0,0,16,4="{sky}"` (Fill region)
*   **Usage**: `pxl draw "hero" --line 0,0,16,16="{rope}"` (Bresenham's algorithm)
*   **Agent Benefit**: Eliminates off-by-one errors and row misalignment. The agent commands "Draw a door at 10,10" rather than trying to calculate which character index in a JSON string corresponds to (10,10).

### C. `pxl mask` (The Stencil)
Read-only queries to recover state, crucial for "stateless" agents.

*   **Usage**: `pxl mask "hero" --query "{skin}"`
    *   *Output*: List of `(x,y)` coordinates.
*   **Usage**: `pxl mask "hero" --sample 5,5`
    *   *Output*: `{skin}`
*   **Agent Benefit**: "Where did I put the eyes?" Allows the agent to "see" the current state of specific features without parsing the full grid visually.

---

## 2. The Interactive Guide (LSP Features)

The LSP acts as the "Smart Guide" for the drafting table, leveraging **Phase 14 (Corpus Analysis)** to provide intelligent suggestions.

### A. Smart Brush (Context-Aware Completion)
Intelligent token completion based on **Phase 14** data.

*   **Co-occurrence**: If the agent is using `{brick_red}`, the LSP suggests `{brick_highlight}` and `{brick_shadow}` based on `pxl analyze` co-occurrence stats from the project.
*   **Neighbor Awareness**: If the pixel to the left is `{grass}`, the Smart Brush suggests `{grass}` (continuation) or `{dirt}` (common neighbor), rather than `{sky}`.

### B. Inlay Hints (The Ruler)
Implements the coordinate system from **Phase 17 (`pxl grid`)** directly in the editor.

*   **Visual**: `"{a} {b} |col:2| {c} ..."`
*   **Agent Benefit**: When an agent reads the file, these virtual "ghost text" markers provide anchors, preventing the "drift" where an agent loses track of column position in long rows.

### C. Active Alignment (The T-Square)
Leverages `pxl fmt` logic to actively maintain grid structure.

*   **Feature**: `textDocument/formatting`
*   **Logic**: Automatically pads tokens to aligned columns (e.g., `{_}   {skin}`).
*   **Agent Benefit**: Agents can generate "messy" but valid arrays. The LSP snap-aligns them, making the grid readable for the *next* generation pass.

---

## 3. Workflow: GenAI + Drafting Machine

How an agent creates a complex **128x128 Castle Scene** using the full Drafting Table suite:

1.  **Plan & Scaffold (Phase 12)**:
    *   Agent: "I need a 128x128 castle scene."
    *   Tool: `pxl scaffold composition --size 128x128 --tile-size 32x32`
    *   Result: A 4x4 composition of empty 32x32 sprites (`tile_0_0` to `tile_3_3`).

2.  **Broad Strokes (Drafting Machine)**:
    *   Agent: "Fill the bottom tiles with grass."
    *   Tool: `pxl draw "tile_0_3" --rect 0,0,32,32="{grass}" --fill` (Repeats for bottom row).
    *   Tool: `pxl draw "tile_1_2" --rect 10,10,12,22="{stone}"` (Draws castle base).

3.  **Detailing (LSP / Smart Brush)**:
    *   Agent focuses on `tile_1_2` (Castle Gate).
    *   Agent uses **Smart Brush** (Completion) to find the `{stone_highlight}` token suggested by the `{stone}` context.
    *   Agent types the gate detail row by row.

4.  **Verification (Phase 17)**:
    *   Agent: "Show me the castle gate."
    *   Tool: `pxl show --sprite tile_1_2` (Terminal preview).
    *   Tool: `pxl check --sprite tile_1_2` (Checks symmetry of the gate).

5.  **Refinement**:
    *   Agent: "The gate is off-center."
    *   Tool: `pxl transform "tile_1_2" --shift 1,0` (Phase 18 Transform).