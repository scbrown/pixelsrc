# Troubleshooting

Common issues when generating Pixelsrc sprites with AI and how to fix them.

## Format Issues

### Invalid JSON

**Problem:** Output has syntax errors and won't parse.

**Symptoms:**
```
Error: expected ',' or '}' at line 1, column 42
```

**Solutions:**

1. Ask the AI to regenerate with explicit validation:
   > "Output valid JSON, one complete object per line. Verify each line is valid JSON before outputting."

2. Check for common mistakes:
   - Missing commas between key-value pairs
   - Trailing commas after last element
   - Unquoted string values
   - Single quotes instead of double quotes

3. Use `pxl fmt` to attempt auto-repair:
   ```bash
   pxl fmt generated.pxl
   ```

### Wrong Token Format

**Problem:** Using `[skin]` or `skin` instead of `{skin}`.

**Solutions:**

1. Include example in prompt:
   > "Use curly brace token format: {skin}, {hair}, {_}"

2. Show the exact format needed:
   > "Tokens must be wrapped in curly braces. Example: `{outline}{skin}{skin}{outline}`"

### Inconsistent Grid Width

**Problem:** Rows have different numbers of tokens.

**Symptoms:**
```
Warning: Row 3 has 15 tokens but expected 16
```

**Solutions:**

1. Ask for explicit size validation:
   > "Ensure each row has exactly 16 tokens"

2. Use strict mode to catch issues:
   ```bash
   pxl render output.pxl --strict
   ```

3. Request the AI double-check:
   > "After generating, verify that every grid row has exactly the same number of tokens"

## Visual Issues

### Missing Transparency

**Problem:** Background is solid instead of transparent.

**Solutions:**

1. Explicitly request transparency:
   > "Use {_} mapped to #00000000 for all background pixels"

2. Verify the hex code has alpha:
   ```json
   "{_}": "#00000000"  // Correct: 8 digits with 00 alpha
   "{_}": "#000000"    // Wrong: 6 digits, fully opaque black
   ```

### Magenta Pixels in Output

**Problem:** Rendered image has bright magenta pixels.

**Cause:** Magenta (#FF00FF) is the error color for undefined tokens.

**Solutions:**

1. Check for typos in token names:
   ```json
   // In palette
   "{skin}": "#FFCC99"

   // In grid - typo!
   "{skn}{skn}{skn}"
   ```

2. Ensure all tokens used in grid are defined in palette

3. Use `pxl explain` to see which tokens are missing:
   ```bash
   pxl explain generated.pxl
   ```

### Colors Look Wrong

**Problem:** Colors don't match expectations.

**Solutions:**

1. Verify hex format:
   - `#RGB` - 3 digits, expanded to 6
   - `#RGBA` - 4 digits, expanded to 8
   - `#RRGGBB` - 6 digits, standard
   - `#RRGGBBAA` - 8 digits, with alpha

2. Check for RGB vs BGR confusion (rare)

3. Request specific hex values:
   > "Use #FF0000 for red, not #F00"

## Generation Quality Issues

### Sprite Looks Stretched/Wrong Proportions

**Problem:** 16x16 sprite has 15 or 17 rows.

**Solutions:**

1. Be explicit about both dimensions:
   > "Create a 16 pixels wide by 16 pixels tall sprite with exactly 16 rows and 16 tokens per row"

2. Verify after generation:
   ```bash
   pxl render generated.pxl --strict
   ```

### Animation Frames Don't Match

**Problem:** Frames have different sizes or character shifts position.

**Solutions:**

1. Define constraints clearly:
   > "All 4 frames must be exactly 16x16. Keep the character centered. Only animate the legs - head and torso stay in the same position."

2. Request shared palette first:
   > "First create a palette called 'walk_colors'. Then create 4 frames all using that palette with identical dimensions."

### Details Too Small/Large

**Problem:** Requested 16x16 but details are too fine or too coarse.

**Solutions:**

1. Reference pixel art style:
   > "NES-style with chunky 2-3 pixel wide lines"
   > "Detailed SNES-style with 1-pixel outlines"

2. Specify detail level:
   > "Simple silhouette with minimal detail"
   > "Detailed with visible features like eyes and clothing seams"

## Workflow Issues

### Inconsistent Palettes Across Sprites

**Problem:** Related sprites have different colors.

**Solutions:**

1. Define palette first:
   > "Create a palette named 'character_colors' with {skin}: #FFCC99, {hair}: #8B4513, etc. Then create all frames using this palette by name."

2. Use variants for color swaps instead of regenerating:
   ```json
   {"type": "variant", "name": "hero_red", "base": "hero", "palette": {"{hair}": "#FF0000"}}
   ```

### Tiles Don't Seamlessly Connect

**Problem:** Edges don't match when tiles are placed adjacent.

**Solutions:**

1. Request seamless design:
   > "The left edge must match the right edge. The top edge must match the bottom edge. Test by mentally placing 4 copies in a 2x2 grid."

2. Generate multiple tiles with shared edge constraints:
   > "Create grass_1 and grass_2 that can be placed adjacent in any order"

### Large Image Generation Fails

**Problem:** AI produces inconsistent or truncated output for large sprites.

**Solutions:**

1. Use composition with smaller tiles:
   > "Create four 16x16 tiles, then compose them into a 32x32 image"

2. Generate in stages:
   ```
   Step 1: Create the palette
   Step 2: Create top-left 16x16 quadrant
   Step 3: Create top-right 16x16 quadrant
   ...
   Final: Compose into 32x32
   ```

## Verification Commands

```bash
# Basic render test
pxl render sprite.pxl -o test.png

# Strict validation (fails on any issue)
pxl render sprite.pxl --strict

# Format check
pxl fmt --check sprite.pxl

# Explain structure
pxl explain sprite.pxl

# Validate without rendering
pxl validate sprite.pxl
```

## Recovery Strategies

### When All Else Fails

1. **Start fresh**: Ask for a complete regeneration with all constraints restated
2. **Simplify**: Request a smaller sprite first, then scale up
3. **Manual fix**: Edit the JSONL directly - it's just text
4. **Use lenient mode**: Let Pixelsrc auto-fix minor issues (default behavior)

### Common Fix Patterns

```bash
# Auto-format fixes many issues
pxl fmt broken.pxl -o fixed.pxl

# Lenient render shows warnings but produces output
pxl render broken.pxl -o output.png

# Strict mode for CI/validation only
pxl render sprite.pxl --strict
```
