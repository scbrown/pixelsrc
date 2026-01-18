# Demo Fixtures

JSONL demo files organized by feature category. Each demo serves dual purposes:

1. **Integration tests** - Verified by `cargo test --test demos`
2. **Documentation** - Embedded into mdbook pages

## Structure

```
demos/
├── sprites/       - Sprite definitions and metadata
├── animation/     - Frame sequences, timing, tags
├── composition/   - Layer stacking, blending
├── exports/       - PNG, GIF, spritesheet, atlas output
└── css/           - CSS-style syntax features
    ├── colors/    - Color formats (hex, rgb, hsl, oklch)
    ├── variables/ - Custom properties and var()
    ├── timing/    - Easing functions
    ├── transforms/- Transform functions
    ├── blend/     - Blend modes
    └── keyframes/ - Animation keyframes
```

## Naming Convention

- `basic.jsonl` - Minimal valid example
- `<feature>.jsonl` - Focused demo of specific capability
