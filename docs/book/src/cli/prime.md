# prime

Print Pixelsrc format guide for AI context injection.

## Usage

```
pxl prime [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--brief` | Print condensed version (~2000 tokens) |
| `--section <SECTION>` | Print specific section: `format`, `examples`, `tips`, `full` |

## Description

The `prime` command outputs documentation about the Pixelsrc format suitable for injecting into AI prompts. This helps AI models generate valid Pixelsrc content by providing format specifications, examples, and best practices.

The output is designed to fit within typical context windows while providing enough information for accurate generation.

## Examples

### Full context

```bash
# Print complete format guide
pxl prime
```

### Brief version

```bash
# Condensed version (~2000 tokens)
pxl prime --brief
```

### Specific sections

```bash
# Just the format specification
pxl prime --section format

# Just examples
pxl prime --section examples

# Just tips and best practices
pxl prime --section tips
```

### Integration with AI tools

```bash
# Inject into prompt file
pxl prime --brief > context.txt

# Pipe to clipboard (macOS)
pxl prime --brief | pbcopy

# Use in a prompt template
echo "$(pxl prime --brief)

Create a 16x16 warrior sprite:" | ai-tool
```

## Output Sections

### Format Section

Documents the Pixelsrc syntax:
- Object types (palette, sprite, animation)
- Grid syntax and tokens
- Color definitions
- Special tokens (`_` for transparency)

### Examples Section

Provides complete, working examples:
- Simple sprite with inline palette
- Animation with multiple frames
- Composition combining sprites

### Tips Section

Best practices for AI generation:
- Token naming conventions
- Grid consistency rules
- Common mistakes to avoid
- Validation recommendations

## Token Estimates

| Version | Approximate Tokens |
|---------|-------------------|
| Full | ~5000 |
| Brief | ~2000 |
| Format only | ~1500 |
| Examples only | ~1000 |
| Tips only | ~500 |

## Use Cases

- **AI prompt engineering**: Provide context for sprite generation
- **Documentation**: Generate format reference docs
- **Training**: Create training data documentation
- **Integration**: Build AI-powered sprite tools

## See Also

- [prompts](prompts.md) - Template prompts for sprite generation
- [suggest](suggest.md) - Fix AI-generated content issues
- [validate](validate.md) - Validate generated content
