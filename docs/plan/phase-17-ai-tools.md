# Phase 17: AI Assistance Tools

**Goal:** Build CLI tools that help AI systems generate better pixelsrc content - context injection, validation, suggestions, and debugging.

**Status:** Planning

**Depends on:** Phase 0 (Core CLI exists)

---

## Motivation

Pixelsrc is designed for GenAI, but the format alone isn't enough. AI needs:

1. **Context** - What is pixelsrc? How does it work? What are best practices?
2. **Validation** - Did I make mistakes? What's wrong with my output?
3. **Suggestions** - Given partial input, what should come next?
4. **Debugging** - Why doesn't this render correctly?

These tools live alongside the renderer but serve a different purpose: **helping AI succeed**.

---

## Commands Overview

| Command | Purpose | Primary User |
|---------|---------|--------------|
| `pxl prime` | Print format docs, best practices, examples | AI (session start) |
| `pxl validate` | Check for common mistakes | AI (before render) |
| `pxl suggest` | Suggest completions for partial input | AI (during generation) |
| `pxl diff` | Compare sprites semantically | AI/Human (debugging) |
| `pxl explain` | Explain what a sprite/composition does | Human (learning) |

---

## `pxl prime` - Context Injection for AI

### Purpose

Print everything an AI needs to generate good pixelsrc content. Run at session start to "prime" the AI with format knowledge.

### Usage

```bash
pxl prime                    # Full primer (format + best practices + examples)
pxl prime --brief            # Condensed version for limited context
pxl prime --section format   # Just the format spec
pxl prime --section examples # Just examples
pxl prime --section tips     # Just best practices
```

### Output Structure

```markdown
# Pixelsrc Primer

## What is Pixelsrc?
A GenAI-native pixel art format. Text-based JSONL that you generate,
`pxl render` converts to PNG/GIF.

## Format Quick Reference

### Object Types
- `palette` - Define named colors: {"type": "palette", "name": "x", "colors": {...}}
- `sprite` - Pixel grid: {"type": "sprite", "name": "x", "size": [w,h], "grid": [...]}
- `composition` - Layer sprites: {"type": "composition", ...}
- `animation` - Frame sequences: {"type": "animation", ...}

### Token Syntax
- Tokens are `{name}` - multi-character, semantic
- `{_}` is transparent (special)
- Define in palette: `"{skin}": "#FFCC99"`
- Use in grid: `"{skin}{skin}{hair}{hair}"`

### Example Sprite
```jsonl
{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{shine}": "#FFFACD"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": [
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}",
  "{_}{gold}{shine}{gold}{gold}{gold}{gold}{_}",
  "{gold}{shine}{gold}{gold}{gold}{gold}{gold}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{gold}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{gold}{gold}",
  "{gold}{gold}{gold}{gold}{gold}{gold}{gold}{gold}",
  "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}",
  "{_}{_}{gold}{gold}{gold}{gold}{_}{_}"
]}
```

## Best Practices

### DO
- Use semantic token names: `{skin}`, `{outline}`, `{shadow}`
- Keep sprites small (8x8, 16x16, 32x32) - easier to generate accurately
- Define palette before sprite that uses it
- Use `{_}` for transparency around sprites

### DON'T
- Use coordinates - positions are implicit in grid
- Use single-char tokens - `{s}` is harder to read than `{skin}`
- Generate huge sprites - use composition/tiling for large images
- Forget row consistency - all rows should have same token count

## Common Patterns

### Character with outline
Outer ring of `{outline}`, inner details, `{_}` for background.

### Shading triad
Use three related tokens: `{main}`, `{light}`, `{shadow}` for depth.

### Symmetric sprites
Many sprites are vertically symmetric - generate left half, mirror right.

## Rendering

```bash
pxl render input.jsonl                    # Render all sprites
pxl render input.jsonl --scale 4          # 4x upscale
pxl render input.jsonl --sprite hero      # Render specific sprite
pxl render input.jsonl --gif              # Export animation as GIF
```
```

### Implementation Notes

- Content sourced from `docs/prompts/system-prompt.md`, `docs/spec/format.md`, `docs/prompts/best-practices.md`
- `--brief` version fits in ~2000 tokens
- Consider embedding this in the binary (compile-time include) vs. reading from docs

---

## `pxl validate` - Pre-Render Checking

### Purpose

Check pixelsrc content for common AI mistakes before rendering. Catches errors that would produce magenta pixels or warnings.

### Usage

```bash
pxl validate input.jsonl              # Validate file
pxl validate --stdin                  # Validate from stdin (for piping)
pxl validate input.jsonl --strict     # Fail on warnings too
pxl validate input.jsonl --json       # Output as JSON for tooling
```

### Checks Performed

| Check | Severity | Description |
|-------|----------|-------------|
| JSON syntax | Error | Invalid JSON on any line |
| Missing type | Error | Line missing `"type"` field |
| Unknown type | Warning | Type not in (palette, sprite, animation, composition, variant) |
| Undefined token | Warning | Token in grid not defined in palette |
| Row length mismatch | Warning | Rows have different token counts |
| Missing palette | Warning | Sprite references undefined palette |
| Invalid color | Error | Color not valid hex (#RGB, #RRGGBB, etc.) |
| Size mismatch | Warning | Grid dimensions don't match declared size |
| Empty grid | Warning | Sprite has no grid rows |
| Duplicate names | Warning | Multiple objects with same name |

### Output Example

```
Validating input.jsonl...

Line 5: WARNING - Undefined token {hiar} in sprite "hero" (did you mean {hair}?)
Line 5: WARNING - Row 3 has 15 tokens, expected 16 (row 1 has 16)
Line 8: ERROR - Invalid color "#GGG" in palette "enemy"

Found 1 error, 2 warnings.
Hint: Run with --strict to treat warnings as errors.
```

### JSON Output (for tooling)

```json
{
  "valid": false,
  "errors": [
    {"line": 8, "type": "invalid_color", "message": "Invalid color \"#GGG\"", "context": "palette \"enemy\""}
  ],
  "warnings": [
    {"line": 5, "type": "undefined_token", "message": "Undefined token {hiar}", "suggestion": "{hair}"},
    {"line": 5, "type": "row_length", "message": "Row 3 has 15 tokens, expected 16"}
  ]
}
```

---

## `pxl suggest` - Completion Assistance

### Purpose

Given partial pixelsrc content, suggest what should come next. Helps AI continue generation or fix incomplete output.

### Usage

```bash
pxl suggest input.jsonl                    # Suggest next likely content
pxl suggest input.jsonl --complete-row 5   # Suggest completion for row 5
pxl suggest input.jsonl --missing-tokens   # List tokens used but not defined
```

### Capabilities

**1. Missing Token Detection**
```bash
$ pxl suggest partial.jsonl --missing-tokens
Tokens used but not defined in any palette:
  {armor} - used in sprite "knight" (lines 12-28)
  {glow}  - used in sprite "magic" (lines 30-45)

Suggested palette addition:
{"type": "palette", "name": "missing", "colors": {"{armor}": "#808080", "{glow}": "#FFFF00"}}
```

**2. Row Completion**
```bash
$ pxl suggest partial.jsonl --complete-row 5
Row 5 has 12 tokens, sprite width is 16.
Based on row 4 pattern, suggested completion:
  Current: "{_}{_}{skin}{skin}{skin}{skin}{hair}{hair}{hair}{hair}{skin}{skin}"
  Add: "{skin}{skin}{_}{_}"
```

**3. Structure Suggestions**
```bash
$ pxl suggest partial.jsonl
File has 2 palettes, 3 sprites, 0 compositions.
Suggestions:
  - Sprite "hero_walk_1" and "hero_walk_2" exist - consider animation?
  - Sprites share palette "hero" - consider composition to arrange them?
```

---

## `pxl diff` - Semantic Comparison

### Purpose

Compare two sprites or files and describe differences semantically. Helps debug "why doesn't this look right?"

### Usage

```bash
pxl diff a.jsonl b.jsonl                    # Compare files
pxl diff file.jsonl --sprite hero v2_hero   # Compare two sprites in same file
pxl diff a.jsonl b.jsonl --visual           # Show visual diff (terminal)
```

### Output Example

```
Comparing sprite "hero" (a.jsonl) vs "hero" (b.jsonl):

Dimensions: Same (16x16)
Palette:
  - a.jsonl: 8 colors
  - b.jsonl: 9 colors (+{highlight})

Token changes:
  - {skin} color: #FFCC99 → #FFD4AA (lighter)
  - {hair} color: #8B4513 → #654321 (darker)
  - Added: {highlight} = #FFFFFF

Grid changes:
  - Row 2: Added {highlight} tokens at positions 4,5
  - Row 8-10: {shadow} replaced with {skin} (removed shading)

Summary: b.jsonl adds highlight effect, lightens skin, removes leg shading.
```

---

## `pxl explain` - Human-Readable Explanation

### Purpose

Explain what a pixelsrc file does in plain English. Useful for learning and documentation.

### Usage

```bash
pxl explain input.jsonl                    # Explain entire file
pxl explain input.jsonl --sprite hero      # Explain specific sprite
```

### Output Example

```
File: hero.jsonl

This file defines a 16x16 pixel art character.

Palette "hero" (6 colors):
  - {_} transparent background
  - {skin} peach skin tone (#FFCC99)
  - {hair} brown hair (#8B4513)
  - {eye} dark eyes (#000000)
  - {outfit} blue clothing (#4169E1)
  - {outline} black outline (#000000)

Sprite "hero" (16x16):
  A humanoid character facing forward. Has:
  - Black outline around entire figure
  - Brown hair on top
  - Two dark eyes
  - Peach skin for face and hands
  - Blue outfit covering torso and legs
  - Transparent background

The sprite is roughly symmetric vertically.
```

---

## Implementation Priority

| Command | Priority | Complexity | Value |
|---------|----------|------------|-------|
| `pxl prime` | **P0** | Low | Critical for AI onboarding |
| `pxl validate` | **P1** | Medium | Catches errors before render |
| `pxl suggest` | P2 | High | Nice-to-have for incomplete work |
| `pxl diff` | P2 | Medium | Debugging aid |
| `pxl explain` | P3 | Medium | Learning aid, less critical |

---

## Tasks

### Task 17.1: `pxl prime` Implementation

- Create `src/prime.rs` module
- Embed primer content (from docs or compile-time)
- Add `--brief`, `--section` flags
- Integrate with CLI

### Task 17.2: `pxl validate` Implementation

- Create `src/validate.rs` module
- Implement all checks from table above
- Add typo suggestions (Levenshtein distance)
- JSON output mode

### Task 17.3: `pxl suggest` Implementation

- Create `src/suggest.rs` module
- Missing token detection
- Row completion suggestions
- Structure suggestions

### Task 17.4: `pxl diff` Implementation

- Create `src/diff.rs` module
- Palette comparison
- Grid comparison (token-level)
- Semantic summary generation

### Task 17.5: `pxl explain` Implementation

- Create `src/explain.rs` module
- Sprite description generation
- Composition description
- Human-friendly output formatting

---

## Integration with AI Workflows

### Session Start
```bash
# AI receives context at session start
pxl prime --brief
```

### Generation Loop
```bash
# AI generates content
echo '<jsonl>' > sprite.jsonl

# AI validates before declaring done
pxl validate sprite.jsonl
# If errors, AI fixes and re-validates

# Finally render
pxl render sprite.jsonl
```

### Debugging
```bash
# Something looks wrong
pxl diff expected.jsonl actual.jsonl
# AI sees semantic differences, can fix
```

---

## Success Criteria

1. `pxl prime` outputs actionable guidance that improves AI generation quality
2. `pxl validate` catches 90%+ of common AI mistakes before rendering
3. Tools integrate smoothly into AI workflows (stdin support, JSON output)
4. Human users also find tools useful for learning and debugging
