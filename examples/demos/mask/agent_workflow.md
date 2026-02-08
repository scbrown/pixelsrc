# Agent Workflow: Mask-Driven Sprite Inspection

This example demonstrates how a stateless AI agent uses `pxl mask` to inspect
and verify sprite state without parsing visual output.

## The Problem

AI agents can generate `.pxl` files but cannot visually verify them.
After drawing operations, an agent needs to confirm:
- Tokens landed where expected
- No unintended overlaps occurred
- Symmetry and alignment are correct

`pxl mask` provides structured, machine-readable answers to these questions.

## Workflow: Inspect a Hero Sprite

### Step 1: Discover file contents

```bash
$ pxl mask examples/hero.pxl --list --json
```

Output tells the agent what sprites, animations, and compositions exist,
including size, palette, format (grid vs regions), and region count.

### Step 2: Get a token frequency overview

```bash
$ pxl mask examples/hero.pxl --sprite hero_idle --count --json
```

The agent now knows every token in the sprite and how many pixels each
occupies. This is the "census" step -- a quick sanity check.

### Step 3: Locate specific features

```bash
# Where are the eyes?
$ pxl mask examples/hero.pxl --sprite hero_idle --query "{eye}" --json
```

Returns exact coordinates: `{"token":"{eye}","count":2,"coords":[[6,6],[9,6]]}`.
The agent can now verify symmetry (both eyes at y=6, x-distance = 3).

### Step 4: Check bounding boxes

```bash
# How big is the skin area?
$ pxl mask examples/hero.pxl --sprite hero_idle --bounds "{skin}" --json
```

Returns `{"token":"{skin}","bounds":[4,5,8,6],"pixel_count":...}`.
The agent knows the face occupies a specific rectangular region.

### Step 5: Point-sample a coordinate

```bash
# What's at the center of the face?
$ pxl mask examples/hero.pxl --sprite hero_idle --sample 7,6 --json
```

Returns the exact token at that pixel. Useful for spot-checking after edits.

### Step 6: Inspect neighborhood context

```bash
# What surrounds the left eye?
$ pxl mask examples/hero.pxl --sprite hero_idle --neighbors 6,6 --json
```

Returns the token at (6,6) plus its up/down/left/right neighbors.
Useful for verifying transitions (e.g., eye is bordered by skin, not outline).

### Step 7: Extract a region for detailed inspection

```bash
# Dump the face region
$ pxl mask examples/hero.pxl --sprite hero_idle --region 5,5,6,4 --json
```

Returns a 2D grid of tokens for just that sub-region. The agent can
analyze the full token layout without processing the entire sprite.

## Key Design Principles

1. **Stateless**: Every mask command reads the current file state. No session
   or cache to manage. After any `pxl draw` edit, just re-run the mask query.

2. **JSON output**: All operations support `--json` for machine parsing.
   Text output is human-friendly but JSON is the agent interface.

3. **Token normalization**: Agents can use `{eye}` or `eye` interchangeably.
   The CLI strips braces automatically.

4. **Zero writes**: `pxl mask` never modifies the file. It is purely read-only.

## Full Inspect-Edit-Verify Cycle

```bash
# 1. Inspect current state
pxl mask hero.pxl --sprite hero_idle --count --json
# → Agent sees token distribution

# 2. Make an edit (add a hat)
pxl draw hero.pxl --sprite hero_idle --rect 4,0,8,2="{hat}"

# 3. Verify the edit took effect
pxl mask hero.pxl --sprite hero_idle --query "{hat}" --json
# → Agent confirms hat pixels exist at expected coordinates

# 4. Check nothing was broken
pxl mask hero.pxl --sprite hero_idle --bounds "{eye}" --json
# → Agent confirms eyes are still in place

# 5. Final census
pxl mask hero.pxl --sprite hero_idle --count --json
# → Agent confirms new token distribution looks right
```

This cycle can repeat indefinitely. Each step is independent and verifiable.
