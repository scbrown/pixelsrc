# Phase 5: CLI Extras

**Goal:** Additional CLI features for import, prompts, and alternative output

**Status:** Complete

**Depends on:** Phase 0 complete

---

## Scope

Phase 5 adds CLI-focused features that don't require WASM:
- PNG import (reverse-engineer images to pixelsrc format)
- GenAI prompt templates for sprite generation
- Emoji art output for quick terminal preview

**Not in scope:** VS Code extension (→ Phase 10), Web previewer (→ Phase 7)

---

## Task Dependency Diagram

```
                          PHASE 5 TASK FLOW
═══════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────┐
│                      Phase 0 Complete                           │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
ALL TASKS PARALLELIZABLE
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   5.1        │  │   5.2        │  │   5.3        │          │
│  │  PNG Import  │  │  GenAI       │  │  Emoji       │          │
│  │              │  │  Prompts     │  │  Output      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  All tasks (5.1, 5.2, 5.3) can run in parallel                  │
│  No dependencies between tasks                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 5.1: PNG Import

**Parallelizable:** Yes

Convert PNG images to pixelsrc format.

**Deliverables:**

1. Add `import` subcommand to CLI:
   ```rust
   /// Import a PNG image to pixelsrc JSONL format
   Import {
       /// Input PNG file
       #[arg(required = true)]
       input: PathBuf,

       /// Output JSONL file (default: stdout)
       #[arg(short, long)]
       output: Option<PathBuf>,

       /// Maximum colors in palette (default: 16)
       #[arg(long, default_value = "16")]
       max_colors: u8,

       /// Sprite name (default: derived from filename)
       #[arg(long)]
       name: Option<String>,
   }
   ```

2. Create `src/import.rs`:
   ```rust
   use image::RgbaImage;
   use std::collections::HashMap;

   pub struct ImportResult {
       pub palette: HashMap<String, String>,
       pub grid: Vec<String>,
       pub width: u32,
       pub height: u32,
   }

   pub fn import_png(image: &RgbaImage, max_colors: u8) -> ImportResult {
       // 1. Extract unique colors
       // 2. Quantize if > max_colors
       // 3. Generate token names ({c1}, {c2}, etc.)
       // 4. Build grid strings
       // 5. Return result
   }
   ```

3. Color quantization using median cut or similar algorithm

**Acceptance Criteria:**
- Simple images convert correctly
- Round-trip: import → render matches original (for ≤max_colors images)
- Color quantization works for complex images

**Verification:**
```bash
# Import simple image
pxl import tests/fixtures/heart.png -o heart.jsonl

# Verify round-trip
pxl render heart.jsonl -o heart_roundtrip.png
# Compare images

# Test quantization
pxl import complex_photo.png --max-colors 8 -o photo.jsonl
```

**Dependencies:** Phase 0 complete

---

### Task 5.2: GenAI Prompt Templates

**Parallelizable:** Yes

Templates and guides for GenAI sprite generation.

**Deliverables:**

1. Create `docs/prompts/` directory:
   ```
   docs/prompts/
   ├── system-prompt.md      # System prompt for Claude/GPT
   ├── sprite-examples.md    # Example prompts with outputs
   ├── best-practices.md     # Tips for reliable generation
   └── templates/
       ├── character.txt     # Character sprite template
       ├── item.txt          # Item/object template
       ├── tileset.txt       # Tileset template
       └── animation.txt     # Animation template
   ```

2. Add CLI command to show prompts:
   ```rust
   /// Show GenAI prompt templates
   Prompts {
       /// Template name (list available if not specified)
       #[arg()]
       template: Option<String>,
   }
   ```

3. System prompt example (`docs/prompts/system-prompt.md`):
   ```markdown
   # PixelSrc System Prompt

   You are a pixel art generator that outputs sprites in pixelsrc JSONL format.

   ## Format Rules

   1. Each line is a valid JSON object with a "type" field
   2. Sprites use token-based grids: `{token}` format
   3. Common tokens: `{_}` for transparency, descriptive names for colors
   4. Palette colors use hex: `#RRGGBB` or `#RRGGBBAA`

   ## Example Output

   ```jsonl
   {"type":"sprite","name":"red_heart","palette":{"{_}":"#00000000","{r}":"#FF0000","{p}":"#FF6B6B"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{p}{r}{r}{p}{r}{r}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}
   ```

   ## Best Practices

   - Use semantic token names: `{skin}`, `{hair}`, `{outline}`
   - Keep sprites small: 8x8, 16x16, 32x32
   - Use transparency `{_}` for background
   - Limit palette to 4-16 colors
   ```

**Acceptance Criteria:**
- Templates produce valid pixelsrc output when used with Claude/GPT
- Guide is clear and helpful
- CLI command shows templates

**Verification:**
```bash
# List templates
pxl prompts

# Show specific template
pxl prompts character

# Test with Claude (manual)
# Copy system prompt, request sprite, verify output
```

**Dependencies:** Phase 0 complete

---

### Task 5.3: Emoji Art Output

**Parallelizable:** Yes (after Phase 0)

Text-based output using emoji for quick terminal preview.

**Deliverables:**

1. Add `--emoji` flag to render command:
   ```rust
   Render {
       // ... existing args

       /// Output as emoji art to stdout instead of PNG
       #[arg(long)]
       emoji: bool,
   }
   ```

2. Create `src/emoji.rs`:
   ```rust
   use image::Rgba;

   /// Map RGBA color to closest emoji
   pub fn color_to_emoji(color: Rgba<u8>) -> &'static str {
       let [r, g, b, a] = color.0;

       // Transparent
       if a < 128 {
           return "⬜";  // or "  " for space
       }

       // Map to emoji based on hue/saturation/lightness
       let (h, s, l) = rgb_to_hsl(r, g, b);

       // Black/white/gray
       if s < 0.1 {
           return match l {
               _ if l < 0.2 => "⬛",
               _ if l < 0.4 => "🔳",
               _ if l < 0.6 => "◻️",
               _ if l < 0.8 => "⬜",
               _ => "🔲",
           };
       }

       // Colors by hue
       match h {
           _ if h < 30.0 => "🟥",   // Red
           _ if h < 60.0 => "🟧",   // Orange
           _ if h < 90.0 => "🟨",   // Yellow
           _ if h < 150.0 => "🟩",  // Green
           _ if h < 210.0 => "🟦",  // Cyan/Blue
           _ if h < 270.0 => "🟪",  // Blue/Purple
           _ if h < 330.0 => "🟪",  // Purple/Pink
           _ => "🟥",               // Red (wrap around)
       }
   }

   pub fn render_emoji(image: &RgbaImage) -> String {
       let mut output = String::new();
       for y in 0..image.height() {
           for x in 0..image.width() {
               let pixel = image.get_pixel(x, y);
               output.push_str(color_to_emoji(*pixel));
           }
           output.push('\n');
       }
       output
   }
   ```

3. Extended emoji palette (optional):
   ```rust
   // More specific colors
   "🤍" // White
   "🖤" // Black
   "❤️" // Red
   "🧡" // Orange
   "💛" // Yellow
   "💚" // Green
   "💙" // Blue
   "💜" // Purple
   "🤎" // Brown
   "🩷" // Pink
   ```

**Acceptance Criteria:**
- Output is visually recognizable
- Works in terminals that support emoji
- Transparent pixels render as empty/white

**Verification:**
```bash
# Render as emoji
pxl render examples/heart.jsonl --emoji

# Should output something like:
# ⬜🟥🟥⬜🟥🟥⬜
# 🟥🟥🟥🟥🟥🟥🟥
# ...

# Test in different terminals
# - macOS Terminal
# - iTerm2
# - VS Code terminal
# - Windows Terminal
```

**Dependencies:** Phase 0 complete

---

## Verification Summary

```bash
# 1. PNG import works
pxl import test.png -o test.jsonl
pxl render test.jsonl -o roundtrip.png

# 2. Prompts available
pxl prompts
pxl prompts character

# 3. Emoji output works
pxl render examples/heart.jsonl --emoji

# 4. All tests pass
cargo test
```

---

## Future Considerations

Features that could extend Phase 5:

| Feature | Description |
|---------|-------------|
| SVG import | Convert vector art to pixel grid |
| ASCII art output | Alternative to emoji using ASCII chars |
| Color palette extraction | `pxl colors image.png` to suggest palettes |
| Batch import | Import directory of PNGs |
