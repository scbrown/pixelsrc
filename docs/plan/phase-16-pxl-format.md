# Phase 16: .pxl Format with Multi-line JSON and Auto-formatter

**Goal:** Improve file format readability with multi-line JSON support and auto-formatting

**Status:** Complete

**Depends on:** Phase 0 (Core CLI exists)

---

## Scope

Phase 16 adds:
- `.pxl` file extension support (alongside `.jsonl`)
- Multi-line JSON object parsing (concatenated JSON)
- `pxl fmt` command for auto-formatting
- Visual grid alignment in formatted output

**Not in scope:** JSON5/JSONC support, comments, custom syntax

---

## Motivation

The current JSONL format requires each JSON object on a single line, making sprite grids unreadable:

```json
{"type": "sprite", "name": "bracket_l", "grid": ["{_}{_}{_}{cg}{c}{c}", "{_}{_}{cg}{c}{c}{_}", "{_}{_}{cg}{c}{cg}{_}"]}
```

With multi-line support, grids become visual:

```json
{"type": "sprite", "name": "bracket_l", "grid": [
  "{_}{_}{_}{cg}{c}{c}",
  "{_}{_}{cg}{c}{c}{_}",
  "{_}{_}{cg}{c}{cg}{_}",
  "{_}{cg}{c}{cg}{_}{_}",
  "{_}{_}{cg}{c}{cg}{_}",
  "{_}{_}{_}{cg}{c}{c}"
]}
```

This improves:
- **Readability** - Grids look like actual pixel art
- **Editability** - Easier to modify specific rows
- **Debugging** - Visual alignment errors become obvious
- **AI generation** - Models can reason about spatial relationships

---

## Task Dependency Diagram

```
                              PHASE 16 TASK FLOW
    ═══════════════════════════════════════════════════════════════════

    PREREQUISITE
    ┌─────────────────────────────────────────────────────────────────┐
    │                      Phase 0 Complete                           │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 1 (Foundation)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   16.1 StreamDeserializer Parser                         │   │
    │  │   - Replace line-by-line parsing                         │   │
    │  │   - Support multi-line JSON objects                      │   │
    │  │   - Maintain line number tracking                        │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 2 (Parallel - Extensions & CLI)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌────────────────────────────┐  ┌──────────────────────────┐  │
    │  │   16.2                     │  │   16.3                   │  │
    │  │  Extension Support         │  │  Fmt CLI Structure       │  │
    │  │  (.pxl + .jsonl)           │  │  (--check, in-place)     │  │
    │  └────────────────────────────┘  └──────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘
              │                              │
              └──────────────┬───────────────┘
                             │
                             ▼
    WAVE 3 (Formatter Logic)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌──────────────────────────────────────────────────────────┐   │
    │  │   16.4 Formatter Implementation                          │   │
    │  │   - Grid array formatting (one row per line)             │   │
    │  │   - Composition layer formatting                         │   │
    │  │   - Palettes as single-line                              │   │
    │  └──────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────┘
              │
              ▼
    WAVE 4 (Integration & Docs)
    ┌─────────────────────────────────────────────────────────────────┐
    │  ┌────────────────────────────┐  ┌──────────────────────────┐  │
    │  │   16.5                     │  │   16.6                   │  │
    │  │  Include Path Support      │  │  Documentation &         │  │
    │  │  (@include:.pxl)           │  │  Migration Guide         │  │
    │  └────────────────────────────┘  └──────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────┘

    ═══════════════════════════════════════════════════════════════════

    PARALLELIZATION SUMMARY:
    ┌─────────────────────────────────────────────────────────────────┐
    │  Wave 1: 16.1                        (1 task, critical)        │
    │  Wave 2: 16.2 + 16.3                 (2 tasks in parallel)     │
    │  Wave 3: 16.4                        (1 task, needs 16.2+16.3) │
    │  Wave 4: 16.5 + 16.6                 (2 tasks in parallel)     │
    └─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 16.1: StreamDeserializer Parser

**Wave:** 1 (critical path)

Replace line-by-line parsing with streaming JSON parser.

**Deliverables:**
- Update `src/parser.rs`:
  ```rust
  use serde_json::StreamDeserializer;

  pub fn parse_stream<R: Read>(reader: R) -> ParseResult {
      let mut result = ParseResult::default();
      let deserializer = serde_json::Deserializer::from_reader(reader);
      let iterator = deserializer.into_iter::<TtpObject>();

      for item in iterator {
          match item {
              Ok(obj) => result.objects.push(obj),
              Err(e) => result.warnings.push(Warning {
                  message: e.to_string(),
                  line: e.line(),
                  column: e.column(),
              }),
          }
      }
      result
  }

  // Keep backward compatibility
  pub fn parse_file(path: &Path) -> ParseResult {
      let file = File::open(path)?;
      parse_stream(BufReader::new(file))
  }
  ```

- Key behaviors:
  - Handle multi-line JSON objects
  - Multiple objects separated by whitespace
  - Proper line/column number tracking for errors
  - Continue parsing after recoverable errors

**Verification:**
```bash
cargo test parser

# Test: Single-line JSONL (backward compatible)
echo '{"type":"palette","name":"t","colors":{}}' | ./target/release/pxl render -

# Test: Multi-line JSON
cat <<'EOF' | ./target/release/pxl render -
{
  "type": "palette",
  "name": "test",
  "colors": {"{a}": "#FF0000"}
}
{
  "type": "sprite",
  "name": "test",
  "size": [2, 2],
  "palette": "test",
  "grid": [
    "{a}{a}",
    "{a}{a}"
  ]
}
EOF
# Should render successfully
```

**Test Fixture:** `tests/fixtures/valid/multiline.pxl`
```json
{
  "type": "palette",
  "name": "test",
  "colors": {
    "{_}": "#00000000",
    "{a}": "#FF0000"
  }
}
{
  "type": "sprite",
  "name": "test",
  "size": [4, 2],
  "palette": "test",
  "grid": [
    "{_}{a}{a}{_}",
    "{a}{a}{a}{a}"
  ]
}
```

**Dependencies:** Phase 0 complete

---

### Task 16.2: Extension Support

**Wave:** 2 (parallel with 16.3)

Support both `.pxl` and `.jsonl` file extensions.

**Deliverables:**
- Update file detection in CLI:
  ```rust
  fn is_pixelsrc_file(path: &Path) -> bool {
      matches!(
          path.extension().and_then(|e| e.to_str()),
          Some("pxl") | Some("jsonl")
      )
  }
  ```

- Update glob patterns:
  ```rust
  // When scanning directories
  fn find_pixelsrc_files(dir: &Path) -> Vec<PathBuf> {
      glob(&format!("{}/**/*.pxl", dir.display()))
          .chain(glob(&format!("{}/**/*.jsonl", dir.display())))
          .flatten()
          .collect()
  }
  ```

- Default output naming:
  - Input `foo.pxl` → Output `foo.png`
  - Input `foo.jsonl` → Output `foo.png`

**Verification:**
```bash
# Create test files
cp examples/coin.jsonl /tmp/test.pxl

# Both extensions work
./target/release/pxl render /tmp/test.pxl -o /tmp/
./target/release/pxl render examples/coin.jsonl -o /tmp/

# Both produce output
ls /tmp/test.png /tmp/coin.png
```

**Dependencies:** Task 16.1

---

### Task 16.3: Fmt CLI Structure

**Wave:** 2 (parallel with 16.2)

Add `pxl fmt` command structure.

**Deliverables:**
- Update `src/cli.rs`:
  ```rust
  /// Format pixelsrc files for readability
  Fmt {
      /// Input file(s) to format
      #[arg(required = true)]
      files: Vec<PathBuf>,

      /// Check formatting without writing (exit 1 if changes needed)
      #[arg(long)]
      check: bool,

      /// Write to stdout instead of in-place
      #[arg(long)]
      stdout: bool,
  }
  ```

- Exit codes:
  - 0: Files formatted (or already formatted with --check)
  - 1: Files need formatting (with --check)

- Basic handler:
  ```rust
  fn handle_fmt(args: FmtArgs) -> Result<()> {
      for file in &args.files {
          let content = fs::read_to_string(file)?;
          let formatted = format_pixelsrc(&content)?;

          if args.check {
              if content != formatted {
                  eprintln!("{}: needs formatting", file.display());
                  return Err(ExitCode(1));
              }
          } else if args.stdout {
              print!("{}", formatted);
          } else {
              fs::write(file, formatted)?;
              eprintln!("{}: formatted", file.display());
          }
      }
      Ok(())
  }
  ```

**Verification:**
```bash
./target/release/pxl fmt --help
# Should show: files, --check, --stdout options

# Check mode (placeholder impl)
./target/release/pxl fmt examples/coin.jsonl --check
```

**Dependencies:** Task 16.1

---

### Task 16.4: Formatter Implementation

**Wave:** 3 (after 16.2, 16.3)

Implement the actual formatting logic.

**Deliverables:**
- New file `src/fmt.rs`:
  ```rust
  pub fn format_pixelsrc(content: &str) -> Result<String> {
      let objects = parse_to_objects(content)?;
      let mut output = String::new();

      for (i, obj) in objects.iter().enumerate() {
          if i > 0 {
              output.push('\n');  // Blank line between objects
          }
          output.push_str(&format_object(obj));
          output.push('\n');
      }
      Ok(output)
  }

  fn format_object(obj: &TtpObject) -> String {
      match obj {
          TtpObject::Palette(p) => format_palette(p),
          TtpObject::Sprite(s) => format_sprite(s),
          TtpObject::Composition(c) => format_composition(c),
          TtpObject::Animation(a) => format_animation(a),
          TtpObject::Variant(v) => format_variant(v),
      }
  }
  ```

- Sprite formatting (visual grids):
  ```rust
  fn format_sprite(sprite: &Sprite) -> String {
      // Output format:
      // {"type": "sprite", "name": "x", "size": [w, h], "palette": "p", "grid": [
      //   "{_}{_}{a}{a}{_}{_}",
      //   "{_}{a}{a}{a}{a}{_}",
      //   "{a}{a}{a}{a}{a}{a}"
      // ]}

      let mut s = String::new();
      s.push_str(&format!(
          r#"{{"type": "sprite", "name": "{}", "size": [{}, {}]"#,
          sprite.name, sprite.size[0], sprite.size[1]
      ));

      if let Some(ref palette) = sprite.palette {
          s.push_str(&format!(r#", "palette": "{}""#, palette));
      }

      s.push_str(r#", "grid": ["#);
      s.push('\n');

      for (i, row) in sprite.grid.iter().enumerate() {
          s.push_str("  ");
          s.push_str(&format!(r#""{}""#, row));
          if i < sprite.grid.len() - 1 {
              s.push(',');
          }
          s.push('\n');
      }

      s.push_str("]}");
      s
  }
  ```

- Composition formatting (visual layers):
  ```rust
  fn format_composition(comp: &Composition) -> String {
      // Similar to sprites - format layer maps visually
  }
  ```

- Palette formatting (single line):
  ```rust
  fn format_palette(palette: &Palette) -> String {
      // Palettes stay on single line for compactness
      serde_json::to_string(palette).unwrap()
  }
  ```

- Animation/Variant formatting (single line):
  ```rust
  fn format_animation(anim: &Animation) -> String {
      serde_json::to_string(anim).unwrap()
  }
  ```

**Verification:**
```bash
cargo test fmt

# Format a file
./target/release/pxl fmt examples/coin.jsonl --stdout
# Should show formatted output with visual grids

# In-place formatting
cp examples/coin.jsonl /tmp/test.jsonl
./target/release/pxl fmt /tmp/test.jsonl
cat /tmp/test.jsonl  # Should be formatted

# Check mode
./target/release/pxl fmt /tmp/test.jsonl --check
echo $?  # Should be 0 (already formatted)

# Round-trip test
./target/release/pxl render /tmp/test.jsonl -o /tmp/before.png
./target/release/pxl fmt /tmp/test.jsonl
./target/release/pxl render /tmp/test.jsonl -o /tmp/after.png
diff /tmp/before.png /tmp/after.png  # Should be identical
```

**Test Fixtures:**
- `tests/fixtures/valid/fmt_input.jsonl` (unformatted)
- `tests/fixtures/valid/fmt_expected.pxl` (expected formatted output)

**Dependencies:** Tasks 16.2, 16.3

---

### Task 16.5: Include Path Support

**Wave:** 4 (parallel with 16.6)

Update `@include:` to support both extensions.

**Deliverables:**
- Update `src/include.rs`:
  ```rust
  pub fn resolve_include(path: &str, base_dir: &Path) -> Result<PathBuf> {
      let resolved = base_dir.join(path);

      // Try exact path first
      if resolved.exists() {
          return Ok(resolved);
      }

      // Try alternate extensions
      let alternates = [
          resolved.with_extension("pxl"),
          resolved.with_extension("jsonl"),
      ];

      for alt in &alternates {
          if alt.exists() {
              return Ok(alt.clone());
          }
      }

      Err(Error::FileNotFound(resolved))
  }
  ```

- Include syntax supports both:
  ```jsonl
  {"type": "sprite", "palette": "@include:shared/colors.pxl", ...}
  {"type": "sprite", "palette": "@include:shared/colors.jsonl", ...}
  {"type": "sprite", "palette": "@include:shared/colors", ...}  // Auto-detect
  ```

**Verification:**
```bash
# Create test structure
mkdir -p /tmp/pxltest/shared
echo '{"type":"palette","name":"shared","colors":{"{a}":"#FF0000"}}' > /tmp/pxltest/shared/colors.pxl

# Test include with .pxl
cat > /tmp/pxltest/main.pxl <<'EOF'
{"type": "sprite", "name": "test", "palette": "@include:shared/colors.pxl", "size": [2, 1], "grid": ["{a}{a}"]}
EOF

./target/release/pxl render /tmp/pxltest/main.pxl -o /tmp/
ls /tmp/test.png  # Should exist
```

**Dependencies:** Task 16.4

---

### Task 16.6: Documentation & Migration Guide

**Wave:** 4 (parallel with 16.5)

Update documentation for new format.

**Deliverables:**
- Update `docs/spec/format.md`:
  - Document `.pxl` extension
  - Document multi-line JSON support
  - Add formatting guidelines

- Update `README.md`:
  - Change examples to use `.pxl` extension
  - Add `pxl fmt` to command list

- Create `docs/migration.md`:
  ```markdown
  # Migrating from .jsonl to .pxl

  ## Quick Migration

  1. Rename files: `mv *.jsonl *.pxl`
  2. Format: `pxl fmt *.pxl`

  ## What Changes

  - File extension: `.jsonl` → `.pxl` (both work)
  - Content format: Single-line → Multi-line (both work)
  - Visual grids: Grids can span multiple lines

  ## Backward Compatibility

  - Both extensions are supported
  - Single-line and multi-line JSON both parse correctly
  - No changes required to existing files
  ```

- Update AI primer (`docs/primer.md`):
  - Show multi-line format in examples
  - Recommend visual grid formatting

**Verification:**
```bash
# Check docs exist
ls docs/spec/format.md docs/migration.md

# Verify examples in README use correct format
grep -c 'grid.*\[' README.md

# Verify primer shows multi-line format
grep -A5 '"grid"' docs/primer.md
```

**Dependencies:** Task 16.4

---

## Formatting Rules

### Sprites
```json
{"type": "sprite", "name": "hero", "size": [16, 16], "palette": "colors", "grid": [
  "{_}{_}{_}{_}{o}{o}{o}{o}{o}{o}{_}{_}{_}{_}{_}{_}",
  "{_}{_}{_}{o}{skin}{skin}{skin}{skin}{skin}{skin}{o}{_}{_}{_}{_}{_}",
  ...
]}
```

### Compositions
```json
{"type": "composition", "name": "scene", "size": [64, 64], "sprites": {"H": "hero", "T": "tree"}, "layers": [
  {"name": "background", "fill": "grass"},
  {"name": "objects", "map": [
    "T......T",
    "........",
    "...H....",
    "........"
  ]}
]}
```

### Palettes (single line)
```json
{"type": "palette", "name": "colors", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{o}": "#000000"}}
```

### Animations (single line)
```json
{"type": "animation", "name": "walk", "frames": ["walk_1", "walk_2", "walk_3"], "duration": 100}
```

---

## Backward Compatibility

| Scenario | Behavior |
|----------|----------|
| `.jsonl` files | Continue to work unchanged |
| Single-line JSON | Parses correctly |
| Multi-line JSON | Parses correctly |
| Mixed file (some single, some multi) | Parses correctly |
| `@include:*.jsonl` | Works |
| `@include:*.pxl` | Works |
| Unformatted `.pxl` | Parses correctly |

---

## Verification Summary

```bash
# 1. All previous tests pass
cargo test

# 2. Multi-line parsing works
./target/release/pxl render tests/fixtures/valid/multiline.pxl

# 3. Both extensions work
./target/release/pxl render examples/coin.jsonl
cp examples/coin.jsonl /tmp/coin.pxl
./target/release/pxl render /tmp/coin.pxl

# 4. Formatter produces valid output
./target/release/pxl fmt examples/coin.jsonl --stdout > /tmp/formatted.pxl
./target/release/pxl render /tmp/formatted.pxl

# 5. Check mode works
./target/release/pxl fmt /tmp/formatted.pxl --check
echo $?  # Should be 0

# 6. Round-trip is lossless
./target/release/pxl render examples/coin.jsonl -o /tmp/before.png
./target/release/pxl fmt examples/coin.jsonl --stdout > /tmp/coin.pxl
./target/release/pxl render /tmp/coin.pxl -o /tmp/after.png
diff /tmp/before.png /tmp/after.png  # Should be identical

# 7. Include paths work with both extensions
# (See Task 16.5 verification)
```

---

## Success Criteria

1. Multi-line JSON objects parse correctly
2. Both `.pxl` and `.jsonl` extensions work everywhere
3. `pxl fmt` produces readable, visual output
4. Round-trip formatting doesn't change render output
5. Existing `.jsonl` files work without modification
6. Documentation updated with new format examples
