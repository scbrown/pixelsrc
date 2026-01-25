# Best Practices

Tips and techniques for getting reliable, high-quality pixel art from LLMs.

## Prompt Structure

### Be Explicit About Format

LLMs work best with clear, specific instructions. Always include:

1. **Dimensions**: "16x16 pixels" not just "small"
2. **Color count**: "using 4-8 colors"
3. **Style reference**: "SNES-era" or "Game Boy style"
4. **Output format**: "in pixelsrc JSONL format"

### Good Prompt Example

> Create a 16x16 pixel art treasure chest in pixelsrc format. Use a warm brown wood color, gold trim, and a dark shadow. Include a keyhole detail. Use semantic token names like {wood}, {gold}, {shadow}. Keep the palette under 8 colors.

### Bad Prompt Example

> Make me a chest

## Token Naming

### Use Semantic Names

Semantic token names produce better, more editable output:

| Good | Bad |
|------|-----|
| `{skin}` | `{c1}` |
| `{outline}` | `{black}` |
| `{highlight}` | `{light}` |
| `{shadow}` | `{dark}` |
| `{hair_dark}` | `{brown2}` |

### Consistent Naming Patterns

For related colors, use suffixes:

```
{skin}        - base color
{skin_light}  - highlight
{skin_dark}   - shadow
```

For body parts:

```
{hair}
{hair_highlight}
{eye}
{eye_pupil}
```

## Color Palettes

### Limit Colors

Different color counts suit different styles:

- **4 colors**: Game Boy, minimalist
- **8 colors**: NES, clean retro
- **16 colors**: SNES, detailed retro
- **32+ colors**: Modern pixel art

Ask for specific counts: "use exactly 6 colors"

### Request Harmony

> "Use a complementary color scheme with blue and orange"

> "Use an analogous palette of warm earth tones"

> "Match the NES palette limitations"

### Include Alpha

Always request transparency handling:

> "Use {_} with #00000000 for transparent pixels"

## Multi-Sprite Workflows

### Shared Palettes

When generating multiple related sprites, define the palette first:

> "First, create a palette called 'hero_colors' with skin, hair, shirt, pants, and outline colors. Then create 4 sprite frames using that palette."

### Animation Frames

For animations, be specific about frame differences:

> "Create a 4-frame walk cycle:
> - Frame 1: left foot forward
> - Frame 2: feet together
> - Frame 3: right foot forward
> - Frame 4: feet together
> Keep the body consistent across frames, only animate the legs."

### Tileset Consistency

For tilesets, establish rules:

> "Create a 4-tile grass tileset that tiles seamlessly. Use the same 3 shades of green across all tiles. Ensure edges match when tiles are placed adjacent."

### Large Images with Tiling

For images larger than 32x32, use composition with `cell_size`:

> "Create a 64x64 landscape scene. First generate four 32x32 tiles: sky_left, sky_right, ground_left, ground_right. Then compose them with cell_size [32, 32]."

Example composition:

```jsonl
{"type": "composition", "name": "landscape", "size": [64, 64], "cell_size": [32, 32],
  "sprites": {"A": "sky_left", "B": "sky_right", "C": "ground_left", "D": "ground_right"},
  "layers": [{"map": ["AB", "CD"]}]}
```

### Color Variants

Use variants to recolor existing sprites without regenerating:

> "Create a hero sprite, then create a variant called 'hero_fire' with red hair instead of brown"

```jsonl
{"type": "variant", "name": "hero_fire", "base": "hero", "palette": {"{hair}": "#FF4400"}}
```

## Iteration Strategies

### Silhouette First

Before adding detail, nail the outline. A good silhouette is recognizable when filled solid.

**The Mushroom Principle**: Most organic forms can be built from 3 basic shapes:
- **Circle** - head/cranium
- **Rectangle** - torso/mid-section
- **Triangle/Polygon** - tapers (jaw, legs)

**Workflow:**
1. Define outline with 2-3 basic shapes
2. Adjust proportions until shape reads correctly
3. Add asymmetry for 3/4 view (shift shapes, angle triangles)
4. Only then add internal detail, shading, features

> If you can't make the silhouette work with 3 shapes, the problem is proportions, not detail.

### Start Simple, Add Detail

1. Generate basic shape
2. Add colors/shading
3. Refine details
4. Create variations

### Ask for Variations

> "Create 3 color variations of this sprite:
> 1. Original colors
> 2. Night version (darker, bluer)
> 3. Autumn version (warmer oranges)"

### Refine Incrementally

> "The sword looks good, but make the blade 2 pixels longer and add a shine highlight on the edge"

## Quality Checklist

Before using generated sprites, verify:

- [ ] Valid JSON (parseable)
- [ ] Correct dimensions match request
- [ ] All rows have same token count
- [ ] Transparency token defined
- [ ] Colors are valid hex format
- [ ] Token names are semantic
- [ ] Renders correctly with `pxl render`
- [ ] No magenta (error color) in output

```bash
# Quick validation
pxl render generated.pxl --strict

# Format for consistency
pxl fmt generated.pxl

# If it renders without errors, you're good
```

## Model-Specific Tips

### Claude

- Excellent at following complex format rules
- Include the full system prompt for best results
- Can generate and explain in same response
- Handles multi-step workflows well

### GPT-4

- Strong at visual concepts
- May need reminding about JSON syntax
- Use "System" message for format instructions
- Good at creative variations

### Local Models

- Keep prompts simpler
- Provide more examples
- May need multiple attempts
- Consider fine-tuning for pixelsrc format

## Known Rendering Limitations

Some shape combinations can cause rendering artifacts. Keep these in mind:

| Issue | Trigger | Workaround |
|-------|---------|------------|
| Stripe artifacts | 5+ shapes with same color | Keep to 4 shapes max per color |
| Polygon fill gaps | 6+ vertices in polygon | Break into simpler shapes, use `union` |
| Overlap artifacts | Multiple regions overlapping at same z | Use different z-levels |

**Safe patterns:**
- 3-4 simple shapes (circle, rect, polygon with 3-5 vertices)
- `union` of simple shapes

**Risky patterns:**
- 5+ overlapping regions with same color
- Single polygon with 6+ vertices

When in doubt, keep it simple. If you hit artifacts, reduce complexity.
