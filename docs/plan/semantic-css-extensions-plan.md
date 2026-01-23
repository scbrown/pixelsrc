# Semantic CSS Extensions for Pixelsrc

> **STATUS: SUPERSEDED**
>
> This document has been merged into the unified Format v2 specification.
> See: [`format2.md`](./format2.md)
>
> Key concepts from this document that made it into Format v2 MVP:
> - Semantic roles (`boundary`, `anchor`, `fill`, `shadow`, `highlight`)
> - Relationships (`derives-from`, `contained-within`, `adjacent-to`, `paired-with`)
> - State rules with CSS selector subset
> - Role-based CSS selectors
>
> Deferred to post-MVP:
> - Animation regions (per-token keyframes)
> - Semantic rotation algorithm
> - Equipment/layering system
> - Procedural variation
> - Hit regions / interaction zones

---

## Original Overview (Historical)

This proposal extends pixelsrc's CSS variable system to leverage semantic token information for game-specific features: animation, state management, transforms (including rotation), equipment layering, procedural variation, and hit detection.

The core insight: pixelsrc's semantic tokens (`{skin}`, `{eye}`, `{outline}`) contain *meaning* that no other pixel art format captures. CSS selectors can target this meaning, enabling behaviors that would otherwise require manual sprite work.

---

## What Was Included in Format v2 MVP

### Semantic Roles (Feature 1)

Included as `Palette.roles`:

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

### Semantic Relationships (Feature 2)

Included as `Palette.relationships`:

```json5
{
  type: "palette",
  name: "hero",
  colors: { /* ... */ },
  relationships: {
    "skin-shadow": { "derives-from": "skin" },
    pupil: { "contained-within": "eye" }
  }
}
```

### State Modifiers (Feature 4)

Included as `StateRules` object type with MVP CSS selector subset:

```json5
{
  type: "state_rules",
  name: "combat",
  rules: {
    ".damaged [token]": { filter: "brightness(2)" },
    ".poisoned [token=skin]": { filter: "hue-rotate(80deg)" },
    "[role=boundary]": { filter: "drop-shadow(0 0 1px black)" }
  }
}
```

**MVP selector subset:**
- `[token=name]` - exact match
- `[token*=str]` - contains
- `[role=type]` - role-based
- `.state` - state class

---

## What Was Deferred to Post-MVP

### Animation Regions (Feature 3)
Per-token keyframe animations for blinking eyes, breathing, etc.

### Semantic Rotation (Feature 5)
Weight-based rotation algorithm using roles to preserve anchors and boundaries.

### Equipment System (Feature 6)
Slot-based equipment layering anchored to semantic regions.

### Procedural Variation (Feature 7)
Variant generation with randomized token colors.

### Hit Regions (Feature 8)
Interaction metadata export for collision, damage, etc.

---

## See Also

- [Format v2 Specification](./format2.md) - Unified spec
- [Phase 24 Tasks](./tasks/phase24.md) - Implementation plan
