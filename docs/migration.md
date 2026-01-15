# Migrating from .jsonl to .pxl

This guide covers migrating existing pixelsrc files to the new `.pxl` format.

---

## Quick Migration

```bash
# 1. Rename files (optional - both extensions work)
for f in *.jsonl; do mv "$f" "${f%.jsonl}.pxl"; done

# 2. Format for readability
pxl fmt *.pxl
```

That's it. Your files are migrated.

---

## What Changes

### File Extension

| Before | After |
|--------|-------|
| `hero.jsonl` | `hero.pxl` |
| `sprites/*.jsonl` | `sprites/*.pxl` |

The extension change is **optional**—both `.jsonl` and `.pxl` work identically. The `.pxl` extension signals that the file may contain multi-line JSON.

### Content Format

| Before (single-line) | After (multi-line) |
|---------------------|-------------------|
| Compact, hard to read | Visual, easy to edit |
| One object per line | Objects span multiple lines |
| Git diffs show entire lines | Git diffs show specific changes |

**Before:**
```jsonl
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "colors", "grid": ["{_}{_}{hair}{hair}{hair}{hair}{_}{_}", "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}", "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}", "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}", "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}", "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}", "{_}{_}{skin}{_}{_}{skin}{_}{_}", "{_}{_}{skin}{_}{_}{skin}{_}{_}"]}
```

**After:**
```json
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "colors", "grid": [
  "{_}{_}{hair}{hair}{hair}{hair}{_}{_}",
  "{_}{hair}{hair}{hair}{hair}{hair}{hair}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{skin}{skin}{skin}{skin}{skin}{skin}{_}",
  "{_}{_}{shirt}{shirt}{shirt}{shirt}{_}{_}",
  "{_}{shirt}{shirt}{shirt}{shirt}{shirt}{shirt}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}",
  "{_}{_}{skin}{_}{_}{skin}{_}{_}"
]}
```

---

## Backward Compatibility

The migration is fully backward compatible:

| Scenario | Works? |
|----------|--------|
| Existing `.jsonl` files | ✓ No changes required |
| Single-line JSON in `.pxl` files | ✓ Parses correctly |
| Multi-line JSON in `.jsonl` files | ✓ Parses correctly |
| Mixed single/multi-line in one file | ✓ Parses correctly |
| `@include:*.jsonl` paths | ✓ Works |
| `@include:*.pxl` paths | ✓ Works |

**No breaking changes.** The parser handles all variations.

---

## Using `pxl fmt`

The formatter converts files to the canonical multi-line format.

### Basic Usage

```bash
# Format a single file in-place
pxl fmt sprites.pxl

# Format multiple files
pxl fmt *.pxl

# Preview without changing (check mode)
pxl fmt sprites.pxl --check

# Output to stdout
pxl fmt sprites.pxl --stdout
```

### CI Integration

Add format checking to your CI pipeline:

```yaml
# GitHub Actions example
- name: Check formatting
  run: pxl fmt --check *.pxl
```

Exit code 1 means files need formatting.

### Formatting Rules

The formatter applies these conventions:

- **Sprites**: Grid arrays expanded (one row per line)
- **Compositions**: Layer maps expanded
- **Palettes**: Single line (compact)
- **Animations**: Single line (compact)
- **Blank line**: Between each object

---

## Updating Includes

If you use `@include:` paths, you can update them to `.pxl`:

**Before:**
```json
{"type": "sprite", "palette": "@include:shared/colors.jsonl", ...}
```

**After:**
```json
{"type": "sprite", "palette": "@include:shared/colors.pxl", ...}
```

Or use extension-less paths (auto-detected):
```json
{"type": "sprite", "palette": "@include:shared/colors", ...}
```

---

## Benefits of Migration

1. **Readability** - Sprite grids look like actual pixel art
2. **Editability** - Modify specific rows without parsing the whole line
3. **Debugging** - Visual alignment errors are obvious
4. **Git diffs** - See exactly which pixels changed
5. **AI generation** - LLMs can reason about spatial relationships in grid rows

---

## Example Workflow

```bash
# Start with existing project
ls sprites/
# coin.jsonl  hero.jsonl  items.jsonl

# Rename to .pxl
for f in sprites/*.jsonl; do mv "$f" "${f%.jsonl}.pxl"; done

# Format for readability
pxl fmt sprites/*.pxl

# Verify output is unchanged
pxl render sprites/hero.pxl -o /tmp/after.png
# Compare with original - should be identical

# Commit the migration
git add sprites/
git commit -m "Migrate sprites to .pxl format"
```

---

## Troubleshooting

### Format check fails in CI

```bash
pxl fmt --check sprites.pxl
# Error: File needs formatting
```

**Fix:** Run `pxl fmt sprites.pxl` locally and commit the changes.

### Include paths not found

```
Error: File not found: shared/colors.jsonl
```

**Fix:** Either:
1. Rename the included file to `.pxl`
2. Update the include path: `@include:shared/colors.pxl`
3. Use extension-less path: `@include:shared/colors`

### Multi-line format causes issues with other tools

Some tools expect strict JSONL (one object per line). If needed:
- Keep `.jsonl` extension for those files
- Or use `pxl fmt --stdout | jq -c` to compact back to single-line
