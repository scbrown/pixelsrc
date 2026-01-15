# Persona Integration

Express user personas throughout the project to help AI assistants and users find relevant features.

**Related:** [Personas](../personas.md) - Persona definitions

---

## Overview

| Area | Implementation |
|------|----------------|
| `pxl prime` | Include personas inline in output |
| Documentation | Frontmatter tags on sections |
| Format spec | Persona + version annotations per attribute |

---

## `pxl prime` Updates

Include personas inline in the primer output. The AI reading the prime determines which persona applies based on conversation context.

### Add to `docs/primer.md`

```markdown
## User Personas

Determine the user's intent to focus on relevant features:

**Sketcher** (quick prototypes, learning)
→ Use: inline palette, single sprite, `pxl render`
→ Keep it simple, don't over-engineer

**Pixel Artist** (polished static art)
→ Use: named palettes, variants, compositions
→ Focus on color consistency and organization

**Animator** (sprite animations)
→ Use: animation type, basic transforms (mirror, pingpong)
→ Minimize frame duplication

**Motion Designer** (complex procedural animation)
→ Use: keyframes, expressions, user-defined transforms
→ Maximize expressiveness and reusability

**Game Developer** (engine integration)
→ Use: frame tags, metadata, nine-slice, atlas export
→ Focus on integration and export formats
```

### Behavior

- No flags needed - personas are always included
- AI matches user's request to a persona based on context
- Guides feature selection without requiring user to self-identify

---

## Documentation Frontmatter Tags

Add persona tags to documentation sections using HTML comments. These are invisible to readers but parseable by tools and AI.

### Syntax

```markdown
<!-- @personas: animator, motion-designer -->
<!-- @complexity: basic | intermediate | advanced -->
## Section Title

Content...
```

### Apply to

| File | What to Tag |
|------|-------------|
| `docs/spec/format.md` | Each attribute, type, and feature |
| `docs/primer.md` | Major sections |
| `docs/primer_brief.md` | Major sections |
| Plan docs | Already have persona columns in tables |

### Example: Format Spec

```markdown
### `palette` type
<!-- @personas: all -->
<!-- @complexity: basic -->

Defines reusable color mappings...

### `transform` attribute
<!-- @personas: pixel-artist, animator, motion-designer, game-dev -->
<!-- @complexity: intermediate -->

Applies transform operations to sprites or animations...

### `nine_slice` attribute
<!-- @personas: game-dev -->
<!-- @complexity: intermediate -->

Defines scalable sprite regions for UI elements...

### `keyframes`
<!-- @personas: motion-designer -->
<!-- @complexity: advanced -->

Define animation key moments for interpolation...
```

### Example: Primer

```markdown
<!-- @personas: all -->
## Format Quick Reference

...

<!-- @personas: animator, motion-designer -->
## Animation

...

<!-- @personas: motion-designer -->
## Advanced: Keyframes & Expressions

...
```

---

## Complexity Levels

Map to personas:

| Complexity | Personas |
|------------|----------|
| `basic` | Sketcher, all |
| `intermediate` | Pixel Artist, Animator, Game Dev |
| `advanced` | Motion Designer |

---

## Benefits

1. **AI context**: Models can filter/prioritize based on detected persona
2. **Documentation**: Users can mentally filter what's relevant
3. **Tooling**: Future tools could filter by persona (optional enhancement)
4. **Maintenance**: Clear ownership of which personas each feature serves
5. **Discoverability**: Features tagged for your persona = features you should know about

---

## Tasks

1. Update `docs/primer.md` with personas section
2. Update `docs/primer_brief.md` with condensed personas
3. Add frontmatter tags to `docs/spec/format.md`
4. Add frontmatter tags to primer sections
5. Update `pxl prime` tests to verify personas included

---

## Future Enhancements

- `pxl help --persona <name>` - Filter help output (optional, not required)
- Persona detection in `pxl suggest` - Tailor suggestions to detected usage pattern
- Website documentation filtering - Show/hide sections by persona toggle
