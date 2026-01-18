# CSS Blend Mode Demos

Blend modes in composition context. Each demo shows a colored base layer with an overlay layer using the specified blend mode.

## Demos

| File | Description |
|------|-------------|
| `normal.jsonl` | Standard alpha compositing (source over destination) |
| `multiply.jsonl` | Darkens underlying colors: result = base × blend |
| `screen.jsonl` | Lightens underlying colors: result = 1 - (1 - base) × (1 - blend) |
| `overlay.jsonl` | Combines multiply/screen based on base brightness |
| `darken.jsonl` | Keeps darker color: result = min(base, blend) |
| `lighten.jsonl` | Keeps lighter color: result = max(base, blend) |
| `add.jsonl` | Additive blending: result = min(1, base + blend) |
| `subtract.jsonl` | Subtractive blending: result = max(0, base - blend) |
| `difference.jsonl` | Color difference: result = abs(base - blend) |

## Usage

Blend modes are specified in composition layers:

```json
{
  "type": "composition",
  "layers": [
    {"map": ["background"]},
    {"map": ["overlay"], "blend": "multiply"}
  ]
}
```

## Supported Blend Modes

- `normal` - Default alpha compositing
- `multiply` - Darkening effect, useful for shadows
- `screen` - Lightening effect, useful for highlights
- `overlay` - Contrast enhancement
- `darken` - Takes minimum of each channel
- `lighten` - Takes maximum of each channel
- `add` / `additive` - Brightening effect, good for lights/glows
- `subtract` / `subtractive` - Darkening by removal
- `difference` - Creates inverted/negative effects
