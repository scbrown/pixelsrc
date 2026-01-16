# The AI Enthusiast

You want to **generate pixel art using AI**. Claude, GPT, or other LLMs—you're harnessing AI to create game assets at scale.

## Why Pixelsrc for AI?

Pixelsrc was designed with AI generation in mind:

- **Text-based**: LLMs excel at generating structured text
- **Semantic tokens**: `{skin}` is more reliable than hex codes
- **JSONL streaming**: Generate line-by-line, validate incrementally
- **Lenient parsing**: Small AI mistakes don't break everything
- **Deterministic output**: Same input = same rendered image

## Your Workflow

1. Write or refine a system prompt
2. Give the AI examples and constraints
3. Generate sprites iteratively
4. Validate and render output

## Getting Started

### Use the Built-in Prompts

Pixelsrc includes optimized system prompts:

```bash
pxl prompts show sprite
```

This displays a prompt tuned for sprite generation. Copy it into your AI conversation.

### List Available Prompts

```bash
pxl prompts list
```

Available prompts:
- `sprite` - Single sprite generation
- `animation` - Animation sequences
- `character` - Full character with variants
- `tileset` - Tileable environment pieces

## Prompting Strategies

### Be Specific About Size

```
Create a 16x16 sprite of a treasure chest.
```

AI tends to make sprites too large without constraints.

### Provide Palette Constraints

```
Use only these colors:
- {_}: transparent
- {outline}: #1a1a2e (dark outline)
- {wood}: #8b4513 (wood brown)
- {metal}: #c0c0c0 (metal gray)
- {gold}: #ffd700 (gold accents)
```

Limiting the palette improves consistency and reduces errors.

### Show Examples

Include a working example in your prompt:

```
Here's an example of the format:

{"type": "palette", "name": "chest", "colors": {"{_}": "#0000", "{wood}": "#8b4513"}}
{"type": "sprite", "name": "chest_closed", "palette": "chest", "grid": [
  "{_}{wood}{wood}{_}",
  "{wood}{wood}{wood}{wood}",
  "{wood}{wood}{wood}{wood}",
  "{_}{wood}{wood}{_}"
]}

Now create a 12x10 sprite of a key using similar style.
```

### Request Semantic Naming

```
Use descriptive token names like {handle}, {blade}, {shine} rather than single letters.
```

This makes AI output more maintainable.

## Iteration Workflow

### Generate → Validate → Refine

1. **Generate**: Ask AI for a sprite
2. **Save**: Copy output to `sprite.pxl`
3. **Validate**: `pxl validate sprite.pxl`
4. **Preview**: `pxl show sprite.pxl`
5. **Refine**: Ask AI to fix issues or adjust

### Common AI Fixes

If the AI makes mistakes, you can ask for corrections:

```
The grid rows have inconsistent lengths. Each row should have exactly 16 tokens.
Please regenerate with consistent row widths.
```

```
The sprite is using {green} but the palette doesn't define that token.
Add {green} to the palette or use an existing color.
```

### Lenient Mode for Prototyping

During iteration, lenient mode (the default) is your friend:

```bash
pxl show sprite.pxl  # Shows sprite even with minor errors
```

Missing tokens render as magenta, helping you spot issues visually.

## Batch Generation

### Character Variants

Ask AI to generate multiple variants:

```
Generate a base knight sprite, then create variants:
1. knight_idle - Standing pose
2. knight_walk_1, knight_walk_2, knight_walk_3, knight_walk_4 - Walk cycle
3. knight_attack_1, knight_attack_2, knight_attack_3 - Attack sequence

Use the same palette for all sprites.
```

### Asset Packs

Generate cohesive sets:

```
Create a dungeon tileset with these 16x16 tiles:
1. floor - Stone floor
2. wall_top - Top of wall
3. wall_front - Front-facing wall
4. door_closed - Closed wooden door
5. door_open - Open door
6. torch - Wall torch with flame

Use a consistent dark fantasy palette.
```

## System Prompt Template

Here's a template for reliable AI generation:

```
You are a pixel art generator that outputs Pixelsrc format (JSONL).

Rules:
1. Output valid JSON, one object per line
2. Define palettes before sprites that use them
3. Use semantic token names wrapped in braces: {token_name}
4. Grid rows must have equal width (same number of tokens)
5. Token names can contain letters, numbers, underscores
6. {_} is the conventional transparent color

Format example:
{"type": "palette", "name": "example", "colors": {"{_}": "#00000000", "{main}": "#FF0000"}}
{"type": "sprite", "name": "example", "palette": "example", "grid": ["{_}{main}{_}", "{main}{main}{main}", "{_}{main}{_}"]}

When I request a sprite, respond with ONLY valid Pixelsrc JSONL. No explanations or markdown code blocks.
```

## Validation in AI Pipelines

### Automated Validation

```python
import subprocess
import json

def validate_pixelsrc(content: str) -> tuple[bool, str]:
    """Validate Pixelsrc content, return (success, error_message)."""
    with open("temp.pxl", "w") as f:
        f.write(content)

    result = subprocess.run(
        ["pxl", "validate", "temp.pxl", "--strict", "--json"],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        return True, ""
    return False, result.stderr
```

### Retry Loop

```python
def generate_sprite(prompt: str, max_retries: int = 3) -> str:
    for attempt in range(max_retries):
        response = ai_generate(prompt)

        valid, error = validate_pixelsrc(response)
        if valid:
            return response

        # Ask AI to fix the error
        prompt = f"The previous output had this error: {error}\nPlease fix and regenerate."

    raise Exception("Failed to generate valid sprite")
```

## Suggestions and Fixes

### Get AI-Friendly Suggestions

```bash
pxl suggest sprite.pxl
```

This outputs suggestions the AI can understand and act on.

### Prime Your Conversation

```bash
pxl prime sprite.pxl
```

Generates context about the file that helps AI understand existing work.

## Best Practices

### Start Simple

Begin with small sprites (8x8, 12x12) before attempting larger ones.

### Provide Visual Reference

If possible, describe what you want in detail:

```
A 16x16 potion bottle:
- Glass bottle shape (rounded bottom, narrow neck)
- Purple liquid filling 2/3 of the bottle
- Cork stopper at top
- Small highlight on glass
- Dark outline around entire sprite
```

### Iterate on Style

Once you get a sprite you like, ask AI to generate more in the same style:

```
Great! Now create a health potion, mana potion, and speed potion
using the same visual style and palette.
```

### Save Your Prompts

Keep successful prompts in a `prompts/` directory for reuse.

## Next Steps

- Explore [System Prompts](../ai-generation/system-prompts.md) for optimized templates
- See [Best Practices](../ai-generation/best-practices.md) for advanced techniques
- Browse the [Example Gallery](../ai-generation/examples.md) for inspiration
