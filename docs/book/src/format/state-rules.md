# State Rules

State rules define visual states for sprites without creating separate sprite definitions. Apply filters, animations, and style changes based on selectors.

## Overview

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

## Basic Structure

```json5
{
  type: "state_rules",
  name: "string (required)",
  rules: {
    "selector": { effect },
    "selector": { effect }
  }
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | Must be `"state_rules"` |
| `name` | Yes | Unique identifier for this rule set |
| `rules` | Yes | Map of CSS selectors to state effects |

## Selectors

State rules use a CSS-like selector syntax to target tokens.

### Token Selectors

| Selector | Example | Meaning |
|----------|---------|---------|
| `[token=name]` | `[token=eye]` | Exact token match |
| `[token*=str]` | `[token*=skin]` | Token contains substring |
| `[token]` | `[token]` | Any token (wildcard) |

### Role Selectors

| Selector | Example | Meaning |
|----------|---------|---------|
| `[role=type]` | `[role=boundary]` | Match by role |

### State Classes

| Selector | Example | Meaning |
|----------|---------|---------|
| `.state` | `.damaged` | Match when state class is active |

### Combined Selectors

Combine selectors for specific targeting:

```json5
// Skin tokens when damaged
".damaged [token=skin]": { filter: "brightness(1.5)" }

// All boundary roles when selected
".selected [role=boundary]": { filter: "brightness(1.2)" }

// Any token with "shadow" in the name
"[token*=shadow]": { opacity: 0.8 }
```

## State Effects

### filter

Apply CSS filter effects.

```json5
".damaged [token]": {
  filter: "brightness(2)"
}

".frozen [token=skin]": {
  filter: "hue-rotate(180deg) saturate(0.5)"
}

"[role=boundary]": {
  filter: "drop-shadow(0 0 1px black)"
}
```

**Supported filters:**
- `brightness(n)` - Adjust brightness (1 = normal)
- `contrast(n)` - Adjust contrast
- `saturate(n)` - Adjust saturation
- `hue-rotate(deg)` - Shift hue
- `invert(n)` - Invert colors
- `grayscale(n)` - Convert to grayscale
- `sepia(n)` - Apply sepia tone
- `drop-shadow(x y blur color)` - Add shadow

### animation

Apply CSS animation.

```json5
".damaged [token]": {
  animation: "flash 0.1s 3"
}

".floating [token]": {
  animation: "bob 1s ease-in-out infinite"
}
```

Animation format: `name duration [timing] [iterations]`

### opacity

Adjust transparency.

```json5
".fading [token]": {
  opacity: 0.5
}

".invisible [token]": {
  opacity: 0
}
```

## Examples

### Combat States

```json5
{
  type: "state_rules",
  name: "combat",
  rules: {
    // Flash white when hit
    ".damaged [token]": {
      filter: "brightness(2)",
      animation: "flash 0.1s 3"
    },

    // Green tint when poisoned
    ".poisoned [token=skin]": {
      filter: "hue-rotate(80deg)"
    },
    ".poisoned [token*=skin]": {
      filter: "hue-rotate(80deg)"
    },

    // Blue tint when frozen
    ".frozen [token]": {
      filter: "hue-rotate(180deg) saturate(0.5)"
    },

    // Red tint when burning
    ".burning [token]": {
      filter: "sepia(1) saturate(3) hue-rotate(-30deg)"
    }
  }
}
```

### UI States

```json5
{
  type: "state_rules",
  name: "ui",
  rules: {
    // Highlight on hover
    ".hover [role=boundary]": {
      filter: "brightness(1.3)"
    },

    // Dim when disabled
    ".disabled [token]": {
      filter: "grayscale(1)",
      opacity: 0.5
    },

    // Glow when selected
    ".selected [role=boundary]": {
      filter: "drop-shadow(0 0 2px gold)"
    },

    // Pulse when active
    ".active [token]": {
      animation: "pulse 0.5s ease-in-out infinite"
    }
  }
}
```

### Visual Effects

```json5
{
  type: "state_rules",
  name: "effects",
  rules: {
    // Outline glow for all boundary tokens
    "[role=boundary]": {
      filter: "drop-shadow(0 0 1px black)"
    },

    // Highlight anchors (eyes, buttons)
    "[role=anchor]": {
      filter: "contrast(1.1)"
    },

    // Subtle shadow enhancement
    "[role=shadow]": {
      opacity: 0.9
    }
  }
}
```

## Usage

### Applying State Rules

State rules are applied at render time. The runtime determines which state classes are active and applies matching rules.

```bash
# Render with state class
pxl render sprite.pxl --state damaged -o damaged.png

# Multiple states
pxl render sprite.pxl --state "damaged poisoned" -o hurt.png
```

### Pre-rendering States

For game engines, you can pre-render all states:

```bash
pxl render sprite.pxl --all-states -o sprites/
```

This creates separate output files for each state combination.

### Referencing State Rules

Associate state rules with a sprite in the same file:

```json5
{
  type: "state_rules",
  name: "hero_states",
  rules: { /* ... */ }
}

{
  type: "sprite",
  name: "hero",
  palette: "hero",
  regions: { /* ... */ },
  "state-rules": "hero_states"
}
```

## Limitations

### MVP Selector Support

The current implementation supports a subset of CSS selectors:

**Supported:**
- `[token=name]` - Exact match
- `[token*=str]` - Contains
- `[role=type]` - Role match
- `.state` - State class

**Not supported (deferred):**
- `:not()` pseudo-class
- Descendant/child combinators
- `^=`, `$=`, `|=` attribute operators
- Multiple classes (`.a.b`)

### Filter Limitations

Filters are applied per-token. Complex multi-token effects may require composition or custom rendering.
