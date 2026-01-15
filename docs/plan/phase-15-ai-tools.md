# Phase 15: AI Assistance Tools

**Goal:** Build CLI tools that help AI systems generate better pixelsrc content - context injection, validation, suggestions, and debugging

**Status:** Planning

**Depends on:** Phase 0 (Core CLI exists)

---

## Scope

Phase 15 adds:
- `pxl prime` - Print format docs and best practices for AI context
- `pxl validate` - Check for common mistakes before rendering
- `pxl suggest` - Suggest completions for partial input
- `pxl diff` - Compare sprites semantically
- `pxl explain` - Human-readable sprite explanation

**Not in scope:** AI model integration, automatic fixing, interactive modes

---

## Motivation

Pixelsrc is designed for GenAI, but the format alone isn't enough. AI needs:

1. **Context** - What is pixelsrc? How does it work? What are best practices?
2. **Validation** - Did I make mistakes? What's wrong with my output?
3. **Suggestions** - Given partial input, what should come next?
4. **Debugging** - Why doesn't this render correctly?

These tools live alongside the renderer but serve a different purpose: **helping AI succeed**.

---

## Command Overview

| Command | Purpose | Primary User | Priority |
|---------|---------|--------------|----------|
| `pxl prime` | Print format docs, best practices, examples | AI (session start) | P0 |
| `pxl validate` | Check for common mistakes | AI (before render) | P1 |
| `pxl suggest` | Suggest completions for partial input | AI (during generation) | P2 |
| `pxl diff` | Compare sprites semantically | AI/Human (debugging) | P2 |
| `pxl explain` | Explain what a sprite does | Human (learning) | P3 |

---

## Task Dependency Diagram

```
                              PHASE 15 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                      Phase 0 Complete                           │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Foundation - Critical Path)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   15.1 `pxl prime` Implementation (P0)                   │   │
    │  │   - Primer content embedding                             │   │
    │  │   - --brief, --section flags                             │   │
    │  │   - CLI integration                                      │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - Core Validation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌────────────────────────────┐  ┌──────────────────────────┐  │
    │  │   15.2                     │  │   15.3                   │  │
    │  │  Validation Checks         │  │  Typo Suggestions        │  │
    │  │  (JSON, tokens, rows)      │  │  (Levenshtein)           │  │
    │  └────────────────────────────┘  └──────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘
              │                              │
              └──────────────┬───────────────┘
                             │
                             ▼
    WAVE 3 (After Validation Core)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   15.4 `pxl validate` CLI Integration                    │   │
    │  │   - --stdin, --strict, --json flags                      │   │
    │  │   - Exit codes                                           │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 4 (Parallel - Suggestions & Diff)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌────────────────────────────┐  ┌──────────────────────────┐  │
    │  │   15.5                     │  │   15.6                   │  │
    │  │  `pxl suggest`             │  │  `pxl diff`              │  │
    │  │  (missing tokens, rows)    │  │  (semantic comparison)   │  │
    │  └────────────────────────────┘  └──────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘
              │                              │
              └──────────────┬───────────────┘
                             │
                             ▼
    WAVE 5 (Polish)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   15.7 `pxl explain` Implementation                      │   │
    │  │   - Sprite description generation                        │   │
    │  │   - Human-friendly output                                │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 15.1                        (1 task, critical)        │
    │  Wave 2: 15.2 + 15.3                 (2 tasks in parallel)     │
    │  Wave 3: 15.4                        (1 task, needs 15.2+15.3) │
    │  Wave 4: 15.5 + 15.6                 (2 tasks in parallel)     │
    │  Wave 5: 15.7                        (1 task, lowest priority) │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 15.1: `pxl prime` Implementation

**Wave:** 1 (critical path - P0)

Print format documentation for AI context injection.

**Deliverables:**
- New file `src/prime.rs`:
  ```rust
  pub const PRIMER_FULL: &str = include_str!("../docs/primer.md");
  pub const PRIMER_BRIEF: &str = include_str!("../docs/primer_brief.md");

  pub enum PrimerSection {
      Format,
      Examples,
      Tips,
      Full,
  }

  pub fn get_primer(section: PrimerSection, brief: bool) -> &'static str
  ```

- New file `docs/primer.md` (embedded at compile time):
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
  [Full coin example...]

  ## Best Practices

  ### DO
  - Use semantic token names: `{skin}`, `{outline}`, `{shadow}`
  - Keep sprites small (8x8, 16x16, 32x32)
  - Define palette before sprite
  - Use `{_}` for transparency

  ### DON'T
  - Use coordinates - positions are implicit
  - Use single-char tokens
  - Generate huge sprites
  - Forget row consistency
  ```

- New file `docs/primer_brief.md` (~2000 tokens max)

- Update `src/cli.rs`:
  ```rust
  /// Print pixelsrc format guide for AI context
  Prime {
      /// Print condensed version
      #[arg(long)]
      brief: bool,

      /// Print specific section: format, examples, tips
      #[arg(long)]
      section: Option<PrimerSection>,
  }
  ```

**Verification:**
```bash
cargo build
./target/release/pxl prime --help
# Should show: --brief, --section options

./target/release/pxl prime | head -20
# Should print full primer

./target/release/pxl prime --brief | wc -c
# Should be < 8000 characters (~2000 tokens)

./target/release/pxl prime --section format
# Should print only format section
```

**Dependencies:** Phase 0 complete

---

### Task 15.2: Validation Checks

**Wave:** 2 (parallel with 15.3)

Implement core validation logic.

**Deliverables:**
- New file `src/validate.rs`:
  ```rust
  #[derive(Debug, Clone)]
  pub enum Severity {
      Error,
      Warning,
  }

  #[derive(Debug, Clone)]
  pub struct ValidationIssue {
      pub line: usize,
      pub severity: Severity,
      pub issue_type: IssueType,
      pub message: String,
      pub suggestion: Option<String>,
  }

  #[derive(Debug, Clone)]
  pub enum IssueType {
      JsonSyntax,
      MissingType,
      UnknownType,
      UndefinedToken,
      RowLengthMismatch,
      MissingPalette,
      InvalidColor,
      SizeMismatch,
      EmptyGrid,
      DuplicateName,
  }

  pub struct Validator {
      issues: Vec<ValidationIssue>,
      palettes: HashMap<String, HashSet<String>>,  // palette name -> defined tokens
      sprite_names: HashSet<String>,
  }

  impl Validator {
      pub fn new() -> Self
      pub fn validate_line(&mut self, line: usize, content: &str)
      pub fn validate_file(&mut self, path: &Path) -> Vec<ValidationIssue>
      pub fn has_errors(&self) -> bool
      pub fn has_warnings(&self) -> bool
  }
  ```

- Validation checks:

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

**Verification:**
```bash
cargo test validate::checks
# Test: Valid file → no issues
# Test: Invalid JSON → Error
# Test: Undefined token → Warning with line number
# Test: Row mismatch → Warning with expected/actual counts
```

**Test Fixture:** `tests/fixtures/invalid/validate_errors.jsonl`
```jsonl
{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "name": "test", "size": [4, 2], "palette": "test", "grid": ["{a}{a}{a}{a}", "{a}{b}{a}"]}
```
Expected: Warning on line 2 for undefined `{b}` and row length mismatch.

**Dependencies:** Task 15.1

---

### Task 15.3: Typo Suggestions

**Wave:** 2 (parallel with 15.2)

Add "did you mean?" suggestions using Levenshtein distance.

**Deliverables:**
- Add to `src/validate.rs`:
  ```rust
  pub fn suggest_token(unknown: &str, known: &[&str]) -> Option<String> {
      // Find closest match using Levenshtein distance
      // Only suggest if distance <= 2
  }

  fn levenshtein_distance(a: &str, b: &str) -> usize
  ```

- Integrate with undefined token check:
  ```
  Line 5: WARNING - Undefined token {hiar} in sprite "hero" (did you mean {hair}?)
  ```

**Verification:**
```bash
cargo test validate::suggest
# Test: {hiar} suggests {hair}
# Test: {skni} suggests {skin}
# Test: {xyz123} suggests nothing (distance > 2)

./target/release/pxl validate tests/fixtures/invalid/typo.jsonl
# Should show "did you mean?" suggestion
```

**Test Fixture:** `tests/fixtures/invalid/validate_typo.jsonl`
```jsonl
{"type": "palette", "name": "char", "colors": {"{skin}": "#FFCC99", "{hair}": "#8B4513"}}
{"type": "sprite", "name": "test", "size": [2, 1], "palette": "char", "grid": ["{skni}{hiar}"]}
```

**Dependencies:** Task 15.1

---

### Task 15.4: `pxl validate` CLI Integration

**Wave:** 3 (after 15.2, 15.3)

Complete CLI integration for validate command.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Validate pixelsrc files for common mistakes
  Validate {
      /// Files to validate
      files: Vec<PathBuf>,

      /// Read from stdin
      #[arg(long)]
      stdin: bool,

      /// Treat warnings as errors
      #[arg(long)]
      strict: bool,

      /// Output as JSON
      #[arg(long)]
      json: bool,
  }
  ```

- Exit codes:
  - 0: No errors (warnings OK unless --strict)
  - 1: Has errors (or warnings with --strict)

- Text output format:
  ```
  Validating input.jsonl...

  Line 5: WARNING - Undefined token {hiar} in sprite "hero" (did you mean {hair}?)
  Line 5: WARNING - Row 3 has 15 tokens, expected 16 (row 1 has 16)
  Line 8: ERROR - Invalid color "#GGG" in palette "enemy"

  Found 1 error, 2 warnings.
  Hint: Run with --strict to treat warnings as errors.
  ```

- JSON output format (`--json`):
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

**Verification:**
```bash
# Valid file
./target/release/pxl validate examples/coin.jsonl
echo $?  # Should be 0

# File with warnings
./target/release/pxl validate tests/fixtures/invalid/validate_errors.jsonl
echo $?  # Should be 0 (warnings only)

# Strict mode
./target/release/pxl validate tests/fixtures/invalid/validate_errors.jsonl --strict
echo $?  # Should be 1

# JSON output
./target/release/pxl validate tests/fixtures/invalid/validate_errors.jsonl --json | jq '.valid'
# Should return false

# Stdin support
cat examples/coin.jsonl | ./target/release/pxl validate --stdin
```

**Dependencies:** Tasks 15.2, 15.3

---

### Task 15.5: `pxl suggest` Implementation

**Wave:** 4 (parallel with 15.6)

Suggest completions for partial input.

**Deliverables:**
- New file `src/suggest.rs`:
  ```rust
  pub struct Suggester {
      palettes: HashMap<String, HashSet<String>>,
      sprites: Vec<String>,
  }

  impl Suggester {
      pub fn from_file(path: &Path) -> Self

      /// Find tokens used in grids but not defined in palettes
      pub fn missing_tokens(&self) -> Vec<MissingToken>

      /// Suggest completion for incomplete row
      pub fn complete_row(&self, sprite_name: &str, row_index: usize) -> Option<String>

      /// Suggest next logical content based on file structure
      pub fn suggest_structure(&self) -> Vec<String>
  }

  #[derive(Debug)]
  pub struct MissingToken {
      pub token: String,
      pub sprite: String,
      pub lines: Vec<usize>,
  }
  ```

- CLI integration:
  ```rust
  /// Suggest completions for partial input
  Suggest {
      /// Input file
      file: PathBuf,

      /// Show missing token definitions
      #[arg(long)]
      missing_tokens: bool,

      /// Suggest completion for specific row
      #[arg(long)]
      complete_row: Option<usize>,
  }
  ```

- Missing tokens output:
  ```
  Tokens used but not defined in any palette:
    {armor} - used in sprite "knight" (lines 12-28)
    {glow}  - used in sprite "magic" (lines 30-45)

  Suggested palette addition:
  {"type": "palette", "name": "missing", "colors": {"{armor}": "#808080", "{glow}": "#FFFF00"}}
  ```

- Row completion output:
  ```
  Row 5 has 12 tokens, sprite width is 16.
  Based on row 4 pattern, suggested completion:
    Current: "{_}{_}{skin}{skin}{skin}{skin}{hair}{hair}{hair}{hair}{skin}{skin}"
    Add: "{skin}{skin}{_}{_}"
  ```

**Verification:**
```bash
cargo test suggest
# Test: Missing tokens detected correctly
# Test: Row completion uses pattern matching

./target/release/pxl suggest partial.jsonl --missing-tokens
# Should list undefined tokens with suggested palette

./target/release/pxl suggest partial.jsonl --complete-row 5
# Should suggest row completion
```

**Test Fixture:** `tests/fixtures/valid/suggest_partial.jsonl`
```jsonl
{"type": "palette", "name": "test", "colors": {"{_}": "#00000000", "{a}": "#FF0000"}}
{"type": "sprite", "name": "test", "size": [4, 3], "palette": "test", "grid": ["{_}{a}{a}{_}", "{a}{a}{a}{a}", "{_}{a}"]}
```

**Dependencies:** Task 15.4

---

### Task 15.6: `pxl diff` Implementation

**Wave:** 4 (parallel with 15.5)

Compare sprites semantically.

**Deliverables:**
- New file `src/diff.rs`:
  ```rust
  #[derive(Debug)]
  pub struct SpriteDiff {
      pub dimension_change: Option<DimensionChange>,
      pub palette_changes: Vec<PaletteChange>,
      pub grid_changes: Vec<GridChange>,
      pub summary: String,
  }

  #[derive(Debug)]
  pub struct DimensionChange {
      pub old: (u32, u32),
      pub new: (u32, u32),
  }

  #[derive(Debug)]
  pub enum PaletteChange {
      Added { token: String, color: String },
      Removed { token: String },
      Changed { token: String, old_color: String, new_color: String },
  }

  #[derive(Debug)]
  pub struct GridChange {
      pub row: usize,
      pub description: String,
  }

  pub fn diff_sprites(a: &Sprite, b: &Sprite) -> SpriteDiff
  pub fn diff_files(a: &Path, b: &Path) -> Vec<(String, SpriteDiff)>
  ```

- CLI integration:
  ```rust
  /// Compare sprites semantically
  Diff {
      /// First file
      file_a: PathBuf,

      /// Second file
      file_b: PathBuf,

      /// Compare specific sprites within same file
      #[arg(long)]
      sprite: Option<(String, String)>,
  }
  ```

- Output format:
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

**Verification:**
```bash
cargo test diff
# Test: Identical sprites → no changes
# Test: Color change detected
# Test: Added token detected

./target/release/pxl diff a.jsonl b.jsonl
# Should show semantic diff
```

**Test Fixtures:**
- `tests/fixtures/valid/diff_a.jsonl`
- `tests/fixtures/valid/diff_b.jsonl`

**Dependencies:** Task 15.4

---

### Task 15.7: `pxl explain` Implementation

**Wave:** 5 (lowest priority)

Generate human-readable explanations of sprites.

**Deliverables:**
- New file `src/explain.rs`:
  ```rust
  pub struct Explanation {
      pub summary: String,
      pub palette_description: String,
      pub sprite_description: String,
      pub structural_notes: Vec<String>,
  }

  pub fn explain_file(path: &Path) -> Explanation
  pub fn explain_sprite(sprite: &Sprite, palette: &Palette) -> String
  ```

- CLI integration:
  ```rust
  /// Explain what a sprite/file does in plain English
  Explain {
      /// Input file
      file: PathBuf,

      /// Explain specific sprite
      #[arg(long)]
      sprite: Option<String>,
  }
  ```

- Output format:
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

**Verification:**
```bash
cargo test explain
# Test: Basic sprite produces description
# Test: Palette colors described with human-readable names

./target/release/pxl explain examples/coin.jsonl
# Should produce readable explanation
```

**Dependencies:** Tasks 15.5, 15.6

---

## AI Workflow Integration

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

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. Prime command works
./target/release/pxl prime --brief | head -20
./target/release/pxl prime --section format

# 3. Validate catches errors
./target/release/pxl validate tests/fixtures/invalid/validate_errors.jsonl
./target/release/pxl validate examples/coin.jsonl && echo "Valid!"

# 4. Validate JSON output
./target/release/pxl validate examples/coin.jsonl --json | jq '.'

# 5. Suggest works
./target/release/pxl suggest tests/fixtures/valid/suggest_partial.jsonl --missing-tokens

# 6. Diff works
./target/release/pxl diff tests/fixtures/valid/diff_a.jsonl tests/fixtures/valid/diff_b.jsonl

# 7. Explain works
./target/release/pxl explain examples/coin.jsonl
```

---

## Success Criteria

1. `pxl prime` outputs actionable guidance that improves AI generation quality
2. `pxl validate` catches 90%+ of common AI mistakes before rendering
3. Tools integrate smoothly into AI workflows (stdin support, JSON output)
4. Human users also find tools useful for learning and debugging
