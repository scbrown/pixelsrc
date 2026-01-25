# prompts

Show GenAI prompt templates for sprite generation.

## Usage

```
pxl prompts [TEMPLATE]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[TEMPLATE]` | Template name to show. If omitted, lists all available templates |

## Available Templates

| Template | Description |
|----------|-------------|
| `character` | Character sprite generation prompt |
| `item` | Item and object sprite generation |
| `tileset` | Tileset and terrain generation |
| `animation` | Animation sequence generation |

## Description

The `prompts` command provides ready-to-use prompt templates for generating Pixelsrc content with AI models. Each template includes:
- Clear instructions for the AI
- Format specifications
- Example output structure
- Common parameters to customize

## Examples

### List available templates

```bash
# Show all available templates
pxl prompts
```

### Get specific template

```bash
# Get character sprite template
pxl prompts character

# Get animation template
pxl prompts animation
```

### Use with AI tools

```bash
# Copy template to clipboard (macOS)
pxl prompts character | pbcopy

# Pipe to AI tool
pxl prompts character | ai-chat --model gpt-4

# Save template for editing
pxl prompts tileset > tileset-prompt.txt
```

### Preview AI output quickly

```bash
# After generating sprite, preview in terminal
pxl show sprite.pxl -s character

# View with coordinates for debugging
pxl grid sprite.pxl -s character

# Get structural breakdown
pxl explain sprite.pxl -s character
```

## Sample Template Output

```
# Character Sprite Generation

Generate a pixel art character sprite in Pixelsrc format.

## Requirements
- Size: 16x16 pixels
- Style: Retro game aesthetic
- Include: Front-facing idle pose

## Output Format
Create a palette with 4-8 colors, then a sprite referencing those colors.

Example structure:
```
palette:
  name: character_colors
  colors:
    outline: #000000
    skin: #E0A070
    ...

sprite:
  name: character
  palette: character_colors
  grid:
    _ _ _ _ ...
```

## Guidelines
- Use `_` for transparent pixels
- Keep outlines consistent (usually 1px black)
- Center the character in the grid
...
```

## Use Cases

- **Quick start**: Get working prompts without writing from scratch
- **Consistency**: Ensure AI outputs match expected format
- **Training**: Understand effective prompt structure
- **Integration**: Build into AI-powered workflows

## See Also

- [prime](prime.md) - Format specification for context injection
- [validate](validate.md) - Validate AI-generated output
- [suggest](suggest.md) - Fix common AI generation errors
