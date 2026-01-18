# Pixelsrc Quick Reference

Text-based JSONL format for pixel art. Generate text, render to PNG with `pxl render`.

## Object Types

**Palette** - Define colors:
```json
{"type": "palette", "name": "x", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99"}}
```

**Sprite** - Pixel grid:
```json
{"type": "sprite", "name": "x", "palette": "x", "grid": ["{_}{skin}{skin}{_}", "..."]}
```

**Animation** - Frame sequence:
```json
{"type": "animation", "name": "x", "frames": ["sprite1", "sprite2"], "duration": 100}
```

## Token Syntax

- Tokens: `{name}` - multi-character, semantic
- Transparent: `{_}` mapped to `#00000000`
- Grid row: `"{a}{a}{b}{b}"` = 4 pixels

## Example (8x8 coin)

```jsonl
{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{shine}": "#FFFACD", "{shadow}": "#B8860B"}}
{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": ["{_}{_}{gold}{gold}{gold}{gold}{_}{_}", "{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}", "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}", "{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}", "{gold}{gold}{gold}{gold}{gold}{gold}{shadow}{gold}", "{gold}{gold}{gold}{gold}{gold}{shadow}{shadow}{gold}", "{_}{gold}{gold}{gold}{gold}{gold}{gold}{_}", "{_}{_}{shadow}{shadow}{shadow}{shadow}{_}{_}"]}
```

## Rules

1. **Palette before sprite** - Define colors first
2. **Semantic tokens** - `{skin}`, `{hair}`, not `{a}`, `{b}`
3. **Consistent rows** - All rows same token count
4. **Small sprites** - 8x8, 16x16, 32x32 typical
5. **Use `{_}` for transparency**

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
