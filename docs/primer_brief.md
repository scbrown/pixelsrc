# Pixelsrc Quick Reference

Text-based JSONL format for pixel art. Generate text, render to PNG with `pxl render`.

## Object Types

**Palette** - Define colors (with CSS variables):
```json
{"type": "palette", "name": "x", "colors": {"--main": "#FFCC99", "{_}": "transparent", "{skin}": "var(--main)", "{shadow}": "color-mix(in oklch, var(--main) 70%, black)"}}
```

**Sprite** - Pixel grid:
```json
{"type": "sprite", "name": "x", "palette": "x", "grid": ["{_}{skin}{skin}{_}", "..."]}
```

**Animation** - CSS Keyframes (recommended):
```json
{"type": "animation", "name": "x", "keyframes": {"0%": {"sprite": "s", "opacity": 1.0}, "50%": {"sprite": "s", "transform": "scale(1.2)"}}, "duration": "1s", "timing_function": "ease-in-out"}
```

**Animation** - Frame sequence (legacy):
```json
{"type": "animation", "name": "x", "frames": ["sprite1", "sprite2"], "duration": 100}
```

## Token Syntax

- Tokens: `{name}` - multi-character, semantic
- Transparent: `{_}` mapped to `#00000000`
- Grid row: `"{a}{a}{b}{b}"` = 4 pixels

## Example (8x8 coin with CSS)

```jsonl
{"type": "palette", "name": "coin", "colors": {"--gold": "#FFD700", "{_}": "transparent", "{gold}": "var(--gold)", "{shine}": "color-mix(in oklch, var(--gold) 60%, white)", "{shadow}": "color-mix(in oklch, var(--gold) 70%, black)"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": ["{_}{_}{gold}{gold}{gold}{gold}{_}{_}", "{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}", "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}", "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}", "{gold}{gold}{gold}{gold}{gold}{gold}{shadow}{gold}", "{gold}{gold}{gold}{gold}{gold}{shadow}{shadow}{gold}", "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}", "{_}{_}{shadow}{shadow}{shadow}{shadow}{_}{_}"]}
{"type": "animation", "name": "coin_bounce", "keyframes": {"0%": {"sprite": "coin"}, "50%": {"sprite": "coin", "transform": "translate(0, -4)"}}, "duration": "400ms", "timing_function": "ease-out", "loop": true}
```

## Rules

1. **Palette before sprite** - Define colors first
2. **Semantic tokens** - `{skin}`, `{hair}`, not `{a}`, `{b}`
3. **Consistent rows** - All rows same token count
4. **Small sprites** - 8x8, 16x16, 32x32 typical
5. **Use `{_}` for transparency**
6. **Use CSS variables** - `--name` for base colors, `var(--name)` for tokens
7. **Use color-mix()** - Auto-generate shadows: `color-mix(in oklch, color 70%, black)`
8. **Use CSS keyframes** - Percentage-based timing with `timing_function`

## Common Errors

- **Row mismatch**: All rows need same token count
- **Undefined token**: Every `{token}` must be in palette
- **Invalid color**: Use `#RGB`, `#RRGGBB`, `#RRGGBBAA`, or CSS colors (`red`, `rgb()`, `hsl()`)
- **Forward reference**: Palette must come before sprite

## Commands

```bash
pxl render file.jsonl              # Render to PNG
pxl render file.jsonl --scale 4    # 4x upscale
pxl render file.jsonl --gif        # Animated GIF
pxl validate file.jsonl            # Check for errors
```
