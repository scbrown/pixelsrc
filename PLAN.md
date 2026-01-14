# TTP (Text To Pixel) - Implementation Plan

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Name | TTP (Text To Pixel) | Clear, memorable |
| Syntax | JSONL (JSON Lines) | Streaming-friendly, each line self-describing |
| Tokens | Multi-char (`{skin}`) | Readability, expressability |
| Coordinates | Implicit (grid position) | Avoids GenAI weakness |
| Transparency | `{_}` | Short, minimal, common "empty" convention |
| CLI command | `pxl` | Short and punchy |
| Language | Rust | Single binary, WASM target, fast streaming |
| Image processing | Shell out to ImageMagick | Don't reinvent the wheel |
| File extension | Any (`.ttp` or `.jsonl`) | CLI accepts any file, extension is convention |
| File structure | Unified | Palette, sprite, animation in one file or streamed |

---

## Phased Implementation

### Phase 0: MVP (Core)
**Goal:** Parse .ttp files and render sprites to PNG

**Deliverables:**
- [ ] `VISION.md` - Project vision and tenets
- [ ] `spec/format.md` - JSONL format specification
- [ ] `src/parser.rs` - Parse and validate .ttp JSONL streams
- [ ] `src/renderer.rs` - Generate PNG via ImageMagick
- [ ] `src/cli.rs` - Basic CLI: `pxl render sprite.ttp -o sprite.png`
- [ ] `examples/` - Sample .ttp files for testing
- [ ] `tests/` - Unit and integration tests

**Example .ttp format (JSONL - each line is a JSON object):**
```jsonl
{"type": "palette", "name": "hero_colors", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99", "{hair}": "#8B4513"}}
{"type": "sprite", "name": "hero_idle", "size": [16, 16], "palette": "hero_colors", "grid": ["{_}{_}{hair}{hair}{_}{_}", "{_}{hair}{hair}{hair}{hair}{_}", "{_}{skin}{skin}{skin}{skin}{_}"]}
```

**Or all-in-one sprite with inline palette:**
```jsonl
{"type": "sprite", "name": "hero_idle", "size": [16, 16], "palette": {"{_}": "#00000000", "{skin}": "#FFCC99"}, "grid": ["{_}{skin}{skin}{_}"]}
```

**Tech stack:**
- Rust (stable)
- serde_json (parsing)
- clap (CLI framework)
- ImageMagick (via std::process::Command)
- cargo test (testing)

---

### Phase 1: Palette Library
**Goal:** Built-in and shareable palettes

**Deliverables:**
- [ ] Built-in palettes bundled in binary (gameboy, nes, pico-8, etc.)
- [ ] CLI: `pxl palettes list` - show available palettes
- [ ] CLI: `pxl palettes show <name>` - preview a palette
- [ ] Reference built-in by name: `"palette": "@gameboy"`
- [ ] Include external file: `"palette": "@include:./my-palette.jsonl"`

**Example with built-in palette:**
```jsonl
{"type": "sprite", "name": "hero", "palette": "@gameboy", "grid": ["{lightest}{dark}{lightest}"]}
```

**Built-in palette definition (compiled into binary):**
```jsonl
{"type": "palette", "name": "gameboy", "colors": {"{lightest}": "#9BBC0F", "{light}": "#8BAC0F", "{dark}": "#306230", "{darkest}": "#0F380F"}}
```

---

### Phase 2: Animation
**Goal:** Multi-frame sprites and animation export

**Deliverables:**
- [ ] Animation type in JSONL stream
- [ ] CLI: `pxl render input.jsonl -o spritesheet.png --spritesheet`
- [ ] CLI: `pxl render input.jsonl -o animation.gif --gif`
- [ ] Spritesheet generation (via ImageMagick montage)
- [ ] GIF generation (via ImageMagick convert)

**Example - animation defined inline with sprites:**
```jsonl
{"type": "palette", "name": "hero_colors", "colors": {"{_}": "#00000000", "{skin}": "#FFCC99"}}
{"type": "sprite", "name": "hero_walk_1", "palette": "hero_colors", "grid": ["{skin}{_}{skin}"]}
{"type": "sprite", "name": "hero_walk_2", "palette": "hero_colors", "grid": ["{_}{skin}{_}"]}
{"type": "sprite", "name": "hero_walk_3", "palette": "hero_colors", "grid": ["{skin}{_}{skin}"]}
{"type": "animation", "name": "hero_walk", "frames": ["hero_walk_1", "hero_walk_2", "hero_walk_3"], "duration": 100, "loop": true}
```

---

### Phase 3: Game Engine Integration
**Goal:** Export to Unity, Godot, Tiled formats

**Deliverables:**
- [ ] Unity spritesheet + .meta export
- [ ] Godot .tres resource export
- [ ] Tiled .tsx tileset export
- [ ] CSS sprite export

---

### Phase 4: Ecosystem
**Goal:** Developer experience tooling

**Deliverables:**
- [ ] VS Code extension (syntax highlighting, preview)
- [ ] Web-based previewer
- [ ] PNG → TTP import tool (reverse engineering)
- [ ] GenAI prompt templates

---

## Project Structure

```
ttp/
├── VISION.md              # Project vision and tenets
├── PLAN.md                # This file
├── README.md              # Usage documentation
├── Cargo.toml             # Rust package config
├── spec/
│   └── format.md          # Formal JSONL specification
├── src/
│   ├── main.rs            # Entry point
│   ├── cli.rs             # Clap-based CLI
│   ├── parser.rs          # JSONL parsing + validation
│   ├── renderer.rs        # ImageMagick integration
│   ├── models.rs          # Serde structs
│   └── palettes/          # Built-in palettes (embedded)
│       ├── mod.rs
│       ├── gameboy.jsonl
│       ├── nes.jsonl
│       └── pico8.jsonl
├── examples/
│   ├── simple_heart.jsonl
│   └── hero_walk.jsonl
└── tests/
    ├── parser_tests.rs
    ├── renderer_tests.rs
    └── fixtures/
```

---

## Verification Plan

1. **Parser tests:** Valid/invalid .ttp files parse correctly
2. **Renderer tests:** Output PNGs match expected fixtures
3. **CLI tests:** Commands work end-to-end
4. **GenAI test:** Prompt Claude to generate .ttp files, measure validity rate
5. **Manual test:** Render example sprites, visually verify
