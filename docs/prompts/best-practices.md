# Best Practices for GenAI Sprite Generation

Tips and techniques for getting reliable, high-quality pixel art from LLMs.

---

## Prompt Structure

### Be Explicit About Format

LLMs work best with clear, specific instructions. Always include:

1. **Dimensions**: "16x16 pixels" not just "small"
2. **Color count**: "using 4-8 colors"
3. **Style reference**: "SNES-era" or "Game Boy style"
4. **Output format**: "in pixelsrc JSONL format"

### Good Prompt Example

> Create a 16x16 pixel art treasure chest in pixelsrc JSONL format. Use a warm brown wood color, gold trim, and a dark shadow. Include a keyhole detail. Use semantic token names like {wood}, {gold}, {shadow}. Keep the palette under 8 colors.

### Bad Prompt Example

> Make me a chest

---

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

---

## Color Palettes

### Limit Colors

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

---

## Common Issues and Fixes

### Issue: Inconsistent Grid Width

**Problem**: Rows have different numbers of tokens

**Fix**: Ask for explicit size validation:
> "Ensure each row has exactly 16 tokens"

Or use `--strict` mode to catch issues:
```bash
pxl render output.jsonl --strict
```

### Issue: Missing Transparency

**Problem**: Background is solid instead of transparent

**Fix**: Explicitly request transparency:
> "Use {_} mapped to #00000000 for all background pixels"

### Issue: Invalid JSON

**Problem**: Output has syntax errors

**Fix**: Request validation:
> "Output valid JSON, one complete object per line"

Or ask for the sprite to be re-generated.

### Issue: Wrong Token Format

**Problem**: Using `[skin]` instead of `{skin}`

**Fix**: Include example in prompt:
> "Use curly brace token format: {skin}, {hair}, {_}"

---

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

---

## Iteration Strategies

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

---

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
pxl render generated.jsonl --strict

# If it renders without errors, you're good
```

---

## Model-Specific Tips

### Claude

- Excellent at following complex format rules
- Include the full system prompt for best results
- Can generate and explain in same response

### GPT-4

- Strong at visual concepts
- May need reminding about JSON syntax
- Use "System" message for format instructions

### Local Models

- Keep prompts simpler
- Provide more examples
- May need multiple attempts
- Consider fine-tuning for pixelsrc format
