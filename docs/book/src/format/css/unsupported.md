# Unsupported CSS Features

Pixelsrc deliberately excludes certain CSS features to maintain simplicity, GenAI reliability, and focus on pixel art use cases. This page documents what's **not** supported and why.

---

## Design Philosophy

Pixelsrc follows a **GenAI-first** design philosophy: every feature should be reliably generatable by LLMs. CSS features are excluded when they:

1. **Add complexity without pixel art benefit** - Features designed for responsive web layouts
2. **Require browser/runtime context** - Features that depend on DOM state or inheritance
3. **Have poor GenAI reliability** - Syntax that LLMs frequently generate incorrectly
4. **Exceed scope** - Full CSS engine features beyond color/transform/timing

---

## Colors

### Not Supported

| Feature | Syntax | Why Excluded |
|---------|--------|--------------|
| `lab()` | `lab(50% 40 59)` | Rarely used; `oklch()` covers perceptual needs better |
| `lch()` | `lch(50% 59 40)` | Superseded by `oklch()` with better perceptual uniformity |
| `color()` | `color(display-p3 1 0.5 0)` | Wide-gamut displays irrelevant for pixel art |
| `currentColor` | `currentColor` | No CSS cascade; colors are explicit per-palette |
| System colors | `Canvas`, `CanvasText` | No browser context; pixel art needs explicit colors |
| Relative color syntax | `hsl(from red h s 50%)` | Complex syntax; use `color-mix()` instead |

### Rationale

**Perceptual color spaces**: Pixelsrc supports `oklch()` as the modern perceptual color space. Older `lab()` and `lch()` are excluded because:
- `oklch()` has better perceptual uniformity than `lch()`
- Supporting multiple perceptual spaces adds complexity without benefit
- GenAI is more reliable generating `oklch()` due to its simpler mental model

**Context-dependent colors**: `currentColor` and system colors require a CSS cascade or browser context that doesn't exist in Pixelsrc. All colors must be explicitly defined in palettes.

**Wide-gamut colors**: The `color()` function for display-p3 and other wide-gamut spaces targets modern displays with extended color ranges. Pixel art is rendered to standard sRGB and doesn't benefit from this.

**Relative color syntax**: While powerful, this CSS5 feature has complex syntax that GenAI struggles to generate correctly. The same effect can be achieved more reliably with `color-mix()`.

---

## Variables

### Not Supported

| Feature | Syntax | Why Excluded |
|---------|--------|--------------|
| `:root` scope | `:root { --color: red }` | No CSS cascade; palette is the scope |
| `@property` | `@property --color {...}` | Typed custom properties require CSS engine |
| Variables in grid tokens | `{var(--name)}` | Tokens are literal names, not CSS values |
| Variables in sprite/palette names | `"name": "var(--x)"` | Names are identifiers, not expressions |
| `calc()` | `calc(100% - 10px)` | Math expressions require CSS engine |

### Rationale

**No CSS cascade**: Pixelsrc palettes are flat maps, not cascading stylesheets. There's no `:root` or inheritance - each palette defines its own scope. Variables are resolved within their palette only.

**Tokens vs. values**: Grid tokens (`{skin}`, `{hair}`) are literal names that map to palette entries. They're not CSS values and can't contain expressions. This keeps parsing simple and reliable.

**No runtime math**: `calc()` and other CSS math functions require a layout engine to resolve. Pixelsrc resolves colors at parse time, not runtime.

**Why `var()` works for colors**: Inside palette `colors`, `var()` references other entries:

```json
{
  "colors": {
    "--base": "#FF0000",
    "{main}": "var(--base)"
  }
}
```

This is palette-scoped variable resolution, not full CSS custom properties.

---

## Timing Functions

### Not Supported

| Feature | Syntax | Why Excluded |
|---------|--------|--------------|
| `linear()` with stops | `linear(0, 0.25 75%, 1)` | Complex piecewise timing rarely needed |
| `spring()` | `spring(1 100 10 0)` | Proposed CSS; not standardized |

### Supported

| Feature | Syntax | Notes |
|---------|--------|-------|
| `linear` | `linear` | Constant speed |
| `ease` | `ease` | Smooth acceleration/deceleration |
| `ease-in` | `ease-in` | Slow start |
| `ease-out` | `ease-out` | Slow end |
| `ease-in-out` | `ease-in-out` | Slow start and end |
| `cubic-bezier()` | `cubic-bezier(0.4, 0, 0.2, 1)` | Custom easing curves |
| `steps()` | `steps(4, end)` | Discrete frame steps |

### Rationale

**Named timing functions**: The standard set (`ease`, `ease-in`, `ease-out`, `ease-in-out`, `linear`) covers 99% of animation needs. GenAI reliably generates these.

**`cubic-bezier()` for custom curves**: When named functions aren't enough, `cubic-bezier()` provides full control. This is the standard approach.

**`steps()` for pixel art**: Frame-by-frame animation is core to pixel art. `steps(n)` provides discrete, frame-accurate timing.

**`linear()` with stops**: The CSS `linear()` function with multiple stops creates piecewise linear timing. This is rarely needed for pixel art and adds complexity. Use multiple keyframes instead.

**`spring()`**: This is a proposed CSS feature for spring physics. It's not standardized and has poor GenAI support. Physics-based animation is out of scope for Pixelsrc.

---

## Transforms

### Not Supported

| Feature | Syntax | Why Excluded |
|---------|--------|--------------|
| `skew()` | `skew(10deg)` | Distorts pixel grid; produces blurry output |
| `matrix()` | `matrix(1, 0, 0, 1, 0, 0)` | Low-level; use individual transforms |
| 3D transforms | `rotateX()`, `translateZ()` | 2D only; pixel art is flat |
| `transform-origin` | `transform-origin: top left` | Adds complexity; anchor points handled differently |

### Supported

| Feature | Syntax | Notes |
|---------|--------|-------|
| `translate()` | `translate(10, 5)` | Pixel-level movement |
| `rotate()` | `rotate(90deg)` | Best at 90-degree increments |
| `scale()` | `scale(2)` | Integer scales for crisp pixels |
| `flip()` | `flip(x)` / `flip(y)` | Pixel-perfect mirroring |

### Rationale

**2D only**: Pixel art is inherently 2D. 3D transforms (`rotateX`, `rotateY`, `rotateZ`, `translateZ`, `perspective`) add complexity without benefit.

**Pixel-perfect transforms**: Pixelsrc prioritizes crisp pixel output:
- `rotate()` works best at 90-degree increments
- `scale()` works best with integer factors
- `translate()` uses integer pixel offsets

**No skew**: `skew()` transforms distort the pixel grid, producing anti-aliased edges that blur the art. If you need skewed sprites, create them in your source art.

**No matrix**: `matrix()` is the low-level representation of 2D transforms. It's harder for GenAI to generate correctly and harder for humans to read. Use individual transform functions instead.

**Anchor points**: CSS `transform-origin` sets the pivot point for transforms. In Pixelsrc, anchor points are handled by the `anchor` parameter on squash/stretch operations, or implied by the transform order.

---

## Blend Modes

### Not Supported

| Feature | Why Excluded |
|---------|--------------|
| `hue` | Requires HSL color space conversion per-pixel |
| `saturation` | Requires HSL color space conversion per-pixel |
| `color` | Requires HSL color space conversion per-pixel |
| `luminosity` | Requires HSL color space conversion per-pixel |
| `plus-lighter` | Non-standard; limited support |
| `plus-darker` | Non-standard; limited support |

### Supported

| Mode | Use Case |
|------|----------|
| `normal` | Standard alpha compositing |
| `multiply` | Shadows, color tinting |
| `screen` | Glows, highlights |
| `overlay` | Contrast enhancement |
| `add` | Additive glow effects |
| `subtract` | Special effects |
| `difference` | Masks, inversions |
| `darken` | Shadow overlays |
| `lighten` | Highlight overlays |

### Rationale

**HSL-based blend modes**: `hue`, `saturation`, `color`, and `luminosity` blend modes require converting pixels to HSL, manipulating components, and converting back. This is expensive and rarely used in pixel art. The supported modes cover common pixel art compositing needs.

**Non-standard modes**: `plus-lighter` and `plus-darker` are not widely supported and have inconsistent implementations. Use `add` or `multiply` instead.

---

## Animation

### Not Supported

| Feature | Syntax | Why Excluded |
|---------|--------|--------------|
| Multiple animations | `animation: a 1s, b 2s` | One animation per sprite; use compositions |
| `animation-delay` | `animation-delay: 500ms` | Use keyframe percentages instead |
| `animation-direction` | `animation-direction: reverse` | Use explicit keyframe ordering |
| `animation-fill-mode` | `animation-fill-mode: forwards` | Animations loop or stop; no fill state |
| `animation-play-state` | `animation-play-state: paused` | Runtime control not applicable |

### Supported

| Feature | Description |
|---------|-------------|
| Keyframes | `0%`, `50%`, `100%` (or `from`, `to`) |
| Duration | `"duration": "500ms"` or `"duration": 500` |
| Timing function | `"timing_function": "ease-in-out"` |
| Loop | `"loop": true` or `"loop": false` |
| Frame arrays | `"frames": ["walk_1", "walk_2"]` |
| Palette cycling | Color rotation animations |
| Frame tags | Named sub-sequences |

### Rationale

**One animation at a time**: Pixelsrc animations target single sprites. For complex scenes with multiple animated elements, use compositions with multiple layers.

**Keyframe-based delays**: Instead of `animation-delay`, incorporate timing into keyframe percentages:

```json
{
  "keyframes": {
    "0%": {"sprite": "idle"},
    "50%": {"sprite": "idle"},
    "60%": {"sprite": "blink"},
    "100%": {"sprite": "idle"}
  }
}
```

The first 50% acts as a delay before the blink.

**Explicit direction**: Instead of `animation-direction: reverse`, define keyframes in the desired order or use palette cycling with `"direction": "reverse"`.

**No runtime state**: `animation-fill-mode` and `animation-play-state` are runtime concepts. Pixelsrc renders to static files (PNG, GIF) or spritesheet. Playback control is the game engine's responsibility.

---

## Summary

Pixelsrc intentionally limits CSS feature scope to maintain:

1. **Reliability** - Features GenAI can generate correctly on first attempt
2. **Simplicity** - Flat palettes, explicit colors, 2D transforms
3. **Focus** - Pixel art optimization, not web layout

For features not supported, there's usually a simpler alternative:

| Instead of... | Use... |
|---------------|--------|
| `lab()`, `lch()` | `oklch()` |
| Relative color syntax | `color-mix()` |
| `calc()` | Pre-computed values |
| `skew()` | Source art |
| 3D transforms | 2D transforms |
| `animation-delay` | Keyframe percentages |
| Multiple animations | Compositions |

If you need a feature not supported, consider whether the complexity is worth it for pixel art, or if a simpler approach achieves the same result.
