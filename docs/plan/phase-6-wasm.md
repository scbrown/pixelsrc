# Phase 6: WASM Foundation

**Goal:** Compile pixelsrc renderer to WebAssembly with JavaScript bindings

**Status:** Complete

**Depends on:** Phase 2 complete

---

## Scope

Phase 6 creates a WASM module that enables:
- Browser-based rendering (website, extensions)
- Obsidian plugin (Electron)
- Node.js usage (npm package)
- Serverless functions (Cloudflare Workers, Vercel Edge)
- Any JavaScript environment

**Not in scope:** Higher-level integrations (those are Phase 7-9)

---

## Task Dependency Diagram

```
                          PHASE 6 TASK FLOW
═══════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────┐
│                      Phase 2 Complete                           │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 1 (Foundation)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   6.1 WASM Build Setup                                   │   │
│  │   - Add wasm-bindgen, wasm-pack                          │   │
│  │   - Configure Cargo.toml for wasm32 target               │   │
│  │   - Create wasm feature flag                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (API)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   6.2 WASM API Module                                    │   │
│  │   - src/wasm.rs with #[wasm_bindgen] exports             │   │
│  │   - render_to_png(jsonl: &str) -> Vec<u8>                │   │
│  │   - render_to_rgba(jsonl: &str) -> RenderResult          │   │
│  │   - Error handling via JsValue                           │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 3 (npm Package)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   6.3 npm Package                                        │   │
│  │   - wasm/ directory with package.json                    │   │
│  │   - TypeScript type definitions                          │   │
│  │   - Build script (wasm-pack build)                       │   │
│  │   - README with usage examples                           │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 4 (Testing & CI)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   6.4 WASM Tests   │  │   6.5 CI Pipeline  │                 │
│  │   - wasm-pack test │  │   - GitHub Actions │                 │
│  │   - Node.js tests  │  │   - npm publish    │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 6.1              (1 task - foundation)                 │
│  Wave 2: 6.2              (1 task - needs 6.1)                  │
│  Wave 3: 6.3              (1 task - needs 6.2)                  │
│  Wave 4: 6.4 + 6.5        (2 tasks in parallel, need 6.3)       │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 6.1: WASM Build Setup

**Wave:** 1

Configure Rust project for WASM compilation.

**Deliverables:**

1. Update `Cargo.toml`:
   ```toml
   [package]
   name = "pixelsrc"
   version = "0.1.0"
   edition = "2021"
   description = "PixelSrc - GenAI-native pixel art format and renderer"

   [lib]
   crate-type = ["cdylib", "rlib"]

   [features]
   default = []
   wasm = ["wasm-bindgen", "console_error_panic_hook"]

   [dependencies]
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   image = "0.24"
   clap = { version = "4.0", features = ["derive"] }
   wasm-bindgen = { version = "0.2", optional = true }
   console_error_panic_hook = { version = "0.1", optional = true }

   [target.'cfg(target_arch = "wasm32")'.dependencies]
   getrandom = { version = "0.2", features = ["js"] }

   [dev-dependencies]
   tempfile = "3"
   wasm-bindgen-test = "0.3"
   ```

2. Create `wasm/` directory structure:
   ```
   wasm/
   ├── package.json
   └── README.md
   ```

3. Create basic `wasm/package.json`:
   ```json
   {
     "name": "@pixelsrc/wasm",
     "version": "0.1.0",
     "description": "WebAssembly build of pixelsrc pixel art renderer",
     "scripts": {
       "build": "cd .. && wasm-pack build --target web --features wasm --out-dir wasm/pkg"
     }
   }
   ```

**Verification:**
```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Verify Cargo.toml is valid
cargo check --features wasm

# Build WASM module (may fail until 6.2 adds wasm.rs)
wasm-pack build --target web --features wasm --out-dir wasm/pkg
```

**Dependencies:** Phase 2 complete

---

### Task 6.2: WASM API Module

**Wave:** 2 (after 6.1)

Create WASM-specific rendering API.

**Deliverables:**

1. Create `src/wasm.rs`:
   ```rust
   //! WebAssembly bindings for pixelsrc

   use wasm_bindgen::prelude::*;
   use crate::models::{TtpObject, PaletteRef};
   use crate::parser;
   use crate::registry::PaletteRegistry;
   use crate::renderer;
   use crate::composition;
   use image::codecs::png::PngEncoder;
   use image::{ImageEncoder, RgbaImage};
   use std::io::Cursor;

   #[wasm_bindgen(start)]
   pub fn init() {
       #[cfg(feature = "wasm")]
       console_error_panic_hook::set_once();
   }

   /// Result of rendering to RGBA pixels
   #[wasm_bindgen]
   pub struct RenderResult {
       width: u32,
       height: u32,
       pixels: Vec<u8>,
       warnings: Vec<String>,
   }

   #[wasm_bindgen]
   impl RenderResult {
       #[wasm_bindgen(getter)]
       pub fn width(&self) -> u32 {
           self.width
       }

       #[wasm_bindgen(getter)]
       pub fn height(&self) -> u32 {
           self.height
       }

       #[wasm_bindgen(getter)]
       pub fn pixels(&self) -> Vec<u8> {
           self.pixels.clone()
       }

       #[wasm_bindgen(getter)]
       pub fn warnings(&self) -> Vec<String> {
           self.warnings.clone()
       }
   }

   /// Render JSONL input to PNG bytes
   ///
   /// Returns PNG-encoded image data as a byte array.
   /// If sprite_name is provided, renders only that sprite.
   /// Otherwise renders the first sprite found.
   #[wasm_bindgen]
   pub fn render_to_png(jsonl: &str, sprite_name: Option<String>) -> Result<Vec<u8>, JsValue> {
       let result = render_internal(jsonl, sprite_name)?;

       // Encode to PNG
       let mut png_bytes: Vec<u8> = Vec::new();
       let encoder = PngEncoder::new(&mut png_bytes);
       encoder
           .write_image(
               &result.pixels,
               result.width,
               result.height,
               image::ExtendedColorType::Rgba8,
           )
           .map_err(|e| JsValue::from_str(&format!("PNG encoding failed: {}", e)))?;

       Ok(png_bytes)
   }

   /// Render JSONL to raw RGBA pixels
   ///
   /// Returns a RenderResult with width, height, and raw RGBA pixel data.
   /// Useful for drawing directly to a canvas element.
   #[wasm_bindgen]
   pub fn render_to_rgba(jsonl: &str, sprite_name: Option<String>) -> Result<RenderResult, JsValue> {
       render_internal(jsonl, sprite_name)
   }

   fn render_internal(jsonl: &str, sprite_name: Option<String>) -> Result<RenderResult, JsValue> {
       // Parse JSONL
       let parse_result = parser::parse_stream(Cursor::new(jsonl));

       if !parse_result.errors.is_empty() {
           return Err(JsValue::from_str(&format!(
               "Parse errors: {}",
               parse_result.errors.join("; ")
           )));
       }

       // Build palette registry
       let mut registry = PaletteRegistry::new();
       let mut sprites = Vec::new();
       let mut compositions = Vec::new();
       let mut all_warnings = Vec::new();

       for obj in &parse_result.objects {
           match obj {
               TtpObject::Palette(p) => {
                   registry.register(p.clone());
               }
               TtpObject::Sprite(s) => {
                   sprites.push(s.clone());
               }
               TtpObject::Composition(c) => {
                   compositions.push(c.clone());
               }
               TtpObject::Animation(_) => {
                   // Animations not yet supported in WASM
               }
           }
       }

       // Collect parse warnings
       for w in &parse_result.warnings {
           all_warnings.push(w.message.clone());
       }

       // Find what to render
       let image: RgbaImage = if let Some(name) = &sprite_name {
           // Try to find composition first, then sprite
           if let Some(comp) = compositions.iter().find(|c| &c.name == name) {
               let sprite_map: std::collections::HashMap<_, _> = sprites
                   .iter()
                   .map(|s| (s.name.clone(), s.clone()))
                   .collect();
               let (img, warnings) = composition::render_composition(comp, &sprite_map, &registry);
               for w in warnings {
                   all_warnings.push(w.message);
               }
               img
           } else if let Some(sprite) = sprites.iter().find(|s| &s.name == name) {
               let palette = registry.resolve(&sprite.palette);
               let (img, warnings) = renderer::render_sprite(sprite, &palette);
               for w in warnings {
                   all_warnings.push(w.message);
               }
               img
           } else {
               return Err(JsValue::from_str(&format!(
                   "Sprite or composition '{}' not found",
                   name
               )));
           }
       } else {
           // Render first composition or sprite
           if let Some(comp) = compositions.first() {
               let sprite_map: std::collections::HashMap<_, _> = sprites
                   .iter()
                   .map(|s| (s.name.clone(), s.clone()))
                   .collect();
               let (img, warnings) = composition::render_composition(comp, &sprite_map, &registry);
               for w in warnings {
                   all_warnings.push(w.message);
               }
               img
           } else if let Some(sprite) = sprites.first() {
               let palette = registry.resolve(&sprite.palette);
               let (img, warnings) = renderer::render_sprite(sprite, &palette);
               for w in warnings {
                   all_warnings.push(w.message);
               }
               img
           } else {
               return Err(JsValue::from_str("No sprites or compositions found"));
           }
       };

       let (width, height) = image.dimensions();
       let pixels = image.into_raw();

       Ok(RenderResult {
           width,
           height,
           pixels,
           warnings: all_warnings,
       })
   }

   /// Get list of sprite names in the JSONL input
   #[wasm_bindgen]
   pub fn list_sprites(jsonl: &str) -> Result<Vec<String>, JsValue> {
       let parse_result = parser::parse_stream(Cursor::new(jsonl));

       let names: Vec<String> = parse_result
           .objects
           .iter()
           .filter_map(|obj| match obj {
               TtpObject::Sprite(s) => Some(s.name.clone()),
               TtpObject::Composition(c) => Some(c.name.clone()),
               _ => None,
           })
           .collect();

       Ok(names)
   }

   /// Validate JSONL input without rendering
   #[wasm_bindgen]
   pub fn validate(jsonl: &str) -> Result<Vec<String>, JsValue> {
       let parse_result = parser::parse_stream(Cursor::new(jsonl));

       let mut messages = Vec::new();

       for e in parse_result.errors {
           messages.push(format!("Error: {}", e));
       }
       for w in parse_result.warnings {
           messages.push(format!("Warning: {}", w.message));
       }

       Ok(messages)
   }
   ```

2. Update `src/lib.rs` to include wasm module:
   ```rust
   #[cfg(feature = "wasm")]
   pub mod wasm;
   ```

**Verification:**
```bash
# Build WASM
wasm-pack build --target web --features wasm --out-dir wasm/pkg

# Check exports exist
ls wasm/pkg/pixelsrc.js
ls wasm/pkg/pixelsrc_bg.wasm

# Check generated types
cat wasm/pkg/pixelsrc.d.ts
```

**Dependencies:** Task 6.1

---

### Task 6.3: npm Package

**Wave:** 3 (after 6.2)

Create publishable npm package with TypeScript types.

**Deliverables:**

1. Update `wasm/package.json`:
   ```json
   {
     "name": "@pixelsrc/wasm",
     "version": "0.1.0",
     "description": "WebAssembly build of pixelsrc pixel art renderer",
     "type": "module",
     "main": "./pkg/pixelsrc.js",
     "types": "./pkg/pixelsrc.d.ts",
     "exports": {
       ".": {
         "import": "./pkg/pixelsrc.js",
         "types": "./pkg/pixelsrc.d.ts"
       }
     },
     "files": [
       "pkg/pixelsrc_bg.wasm",
       "pkg/pixelsrc.js",
       "pkg/pixelsrc.d.ts",
       "README.md"
     ],
     "scripts": {
       "build": "cd .. && wasm-pack build --target web --features wasm --out-dir wasm/pkg",
       "test": "node --experimental-vm-modules test.mjs"
     },
     "keywords": [
       "pixel-art",
       "wasm",
       "webassembly",
       "renderer",
       "sprite",
       "pixelsrc",
       "genai"
     ],
     "author": "",
     "license": "MIT",
     "repository": {
       "type": "git",
       "url": "https://github.com/user/pixelsrc"
     },
     "bugs": {
       "url": "https://github.com/user/pixelsrc/issues"
     },
     "homepage": "https://github.com/user/pixelsrc#readme"
   }
   ```

2. Create `wasm/README.md`:
   ```markdown
   # @pixelsrc/wasm

   WebAssembly build of [pixelsrc](https://github.com/user/pixelsrc) - a GenAI-native pixel art format and renderer.

   ## Installation

   ```bash
   npm install @pixelsrc/wasm
   ```

   ## Usage (Browser)

   ```javascript
   import init, { render_to_png, render_to_rgba, list_sprites } from '@pixelsrc/wasm';

   // Initialize WASM module (required before first use)
   await init();

   // Define a sprite in pixelsrc JSONL format
   const jsonl = `{"type":"sprite","name":"heart","palette":{"{_}":"#00000000","{r}":"#FF0000"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}`;

   // Render to PNG bytes
   const pngBytes = render_to_png(jsonl);

   // Display in an <img> element
   const blob = new Blob([pngBytes], { type: 'image/png' });
   const url = URL.createObjectURL(blob);
   document.getElementById('preview').src = url;

   // Or render to RGBA for canvas
   const result = render_to_rgba(jsonl);
   const imageData = new ImageData(
     new Uint8ClampedArray(result.pixels),
     result.width,
     result.height
   );
   ctx.putImageData(imageData, 0, 0);

   // List available sprites
   const sprites = list_sprites(jsonl);
   console.log('Sprites:', sprites);
   ```

   ## Usage (Node.js)

   ```javascript
   import { readFileSync, writeFileSync } from 'fs';
   import init, { render_to_png } from '@pixelsrc/wasm';

   await init();

   const jsonl = readFileSync('sprite.jsonl', 'utf8');
   const pngBytes = render_to_png(jsonl);
   writeFileSync('sprite.png', pngBytes);
   ```

   ## API

   ### `init(): Promise<void>`
   Initialize the WASM module. Must be called before any other function.

   ### `render_to_png(jsonl: string, spriteName?: string): Uint8Array`
   Render JSONL input to PNG bytes. Optionally specify which sprite to render.

   ### `render_to_rgba(jsonl: string, spriteName?: string): RenderResult`
   Render to raw RGBA pixels. Returns an object with:
   - `width: number` - Image width in pixels
   - `height: number` - Image height in pixels
   - `pixels: Uint8Array` - Raw RGBA pixel data
   - `warnings: string[]` - Any rendering warnings

   ### `list_sprites(jsonl: string): string[]`
   Get list of sprite and composition names in the input.

   ### `validate(jsonl: string): string[]`
   Validate JSONL without rendering. Returns array of error/warning messages.

   ## pixelsrc Format

   pixelsrc uses JSONL (JSON Lines) format. Each line is a self-contained JSON object:

   ```jsonl
   {"type":"palette","name":"mono","colors":{"{_}":"#00000000","{on}":"#FFFFFF","{off}":"#000000"}}
   {"type":"sprite","name":"checker","palette":"mono","grid":["{on}{off}{on}{off}","{off}{on}{off}{on}"]}
   ```

   See the [full documentation](https://github.com/user/pixelsrc) for more details.

   ## License

   MIT
   ```

3. Create `wasm/test.mjs`:
   ```javascript
   import { readFileSync } from 'fs';
   import { fileURLToPath } from 'url';
   import { dirname, join } from 'path';

   const __dirname = dirname(fileURLToPath(import.meta.url));

   // Dynamic import for WASM
   const { default: init, render_to_png, render_to_rgba, list_sprites, validate } = await import('./pkg/pixelsrc.js');

   await init();

   console.log('Running @pixelsrc/wasm tests...\n');

   // Test 1: Minimal sprite
   const minimalSprite = '{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}';

   const png = render_to_png(minimalSprite);
   console.assert(png.length > 0, 'PNG should have content');
   console.assert(png[0] === 0x89 && png[1] === 0x50 && png[2] === 0x4E && png[3] === 0x47,
     'PNG should start with magic bytes');
   console.log('✓ render_to_png produces valid PNG');

   // Test 2: RGBA output
   const rgba = render_to_rgba(minimalSprite);
   console.assert(rgba.width === 1, `Width should be 1, got ${rgba.width}`);
   console.assert(rgba.height === 1, `Height should be 1, got ${rgba.height}`);
   console.assert(rgba.pixels.length === 4, `Should have 4 bytes (RGBA), got ${rgba.pixels.length}`);
   console.assert(rgba.pixels[0] === 255, 'Red channel should be 255');
   console.assert(rgba.pixels[1] === 0, 'Green channel should be 0');
   console.assert(rgba.pixels[2] === 0, 'Blue channel should be 0');
   console.assert(rgba.pixels[3] === 255, 'Alpha channel should be 255');
   console.log('✓ render_to_rgba returns correct dimensions and pixels');

   // Test 3: List sprites
   const multiSprite = `{"type":"sprite","name":"one","palette":{"{x}":"#FF0000"},"grid":["{x}"]}
   {"type":"sprite","name":"two","palette":{"{x}":"#00FF00"},"grid":["{x}"]}`;
   const names = list_sprites(multiSprite);
   console.assert(names.length === 2, `Should have 2 sprites, got ${names.length}`);
   console.assert(names.includes('one'), 'Should include "one"');
   console.assert(names.includes('two'), 'Should include "two"');
   console.log('✓ list_sprites returns sprite names');

   // Test 4: Validate
   const invalid = '{"type":"sprite","name":"bad"';
   const messages = validate(invalid);
   console.assert(messages.length > 0, 'Should have validation messages');
   console.log('✓ validate catches errors');

   // Test 5: Warnings
   const withWarning = '{"type":"sprite","name":"warn","palette":{"{x}":"#FF0000"},"size":[2,1],"grid":["{x}"]}';
   const warnResult = render_to_rgba(withWarning);
   console.assert(warnResult.warnings.length > 0, 'Should have warnings for short row');
   console.log('✓ Warnings are captured');

   console.log('\n✅ All tests passed!');
   ```

**Verification:**
```bash
cd wasm
npm run build
npm test
npm pack --dry-run  # Verify package contents
```

**Dependencies:** Task 6.2

---

### Task 6.4: WASM Tests

**Wave:** 4 (parallel with 6.5)

Add comprehensive WASM tests.

**Deliverables:**

1. Create `tests/wasm_tests.rs`:
   ```rust
   #![cfg(target_arch = "wasm32")]

   use wasm_bindgen_test::*;

   wasm_bindgen_test_configure!(run_in_browser);

   #[wasm_bindgen_test]
   fn test_render_minimal_sprite_to_png() {
       let jsonl = r#"{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}"#;
       let result = pixelsrc::wasm::render_to_png(jsonl, None);
       assert!(result.is_ok());
       let bytes = result.unwrap();
       assert!(!bytes.is_empty());
       // PNG magic bytes
       assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
   }

   #[wasm_bindgen_test]
   fn test_render_minimal_sprite_to_rgba() {
       let jsonl = r#"{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}"#;
       let result = pixelsrc::wasm::render_to_rgba(jsonl, None);
       assert!(result.is_ok());
       let render_result = result.unwrap();
       assert_eq!(render_result.width(), 1);
       assert_eq!(render_result.height(), 1);
       let pixels = render_result.pixels();
       assert_eq!(pixels.len(), 4);
       // Red pixel: RGBA(255, 0, 0, 255)
       assert_eq!(pixels[0], 255);
       assert_eq!(pixels[1], 0);
       assert_eq!(pixels[2], 0);
       assert_eq!(pixels[3], 255);
   }

   #[wasm_bindgen_test]
   fn test_render_named_sprite() {
       let jsonl = r#"{"type":"sprite","name":"red","palette":{"{r}":"#FF0000"},"grid":["{r}"]}
   {"type":"sprite","name":"blue","palette":{"{b}":"#0000FF"},"grid":["{b}"]}"#;

       let result = pixelsrc::wasm::render_to_rgba(jsonl, Some("blue".to_string()));
       assert!(result.is_ok());
       let render_result = result.unwrap();
       let pixels = render_result.pixels();
       // Blue pixel: RGBA(0, 0, 255, 255)
       assert_eq!(pixels[0], 0);
       assert_eq!(pixels[1], 0);
       assert_eq!(pixels[2], 255);
   }

   #[wasm_bindgen_test]
   fn test_list_sprites() {
       let jsonl = r#"{"type":"palette","name":"colors","colors":{"{x}":"#FF0000"}}
   {"type":"sprite","name":"one","palette":"colors","grid":["{x}"]}
   {"type":"sprite","name":"two","palette":"colors","grid":["{x}"]}"#;

       let result = pixelsrc::wasm::list_sprites(jsonl);
       assert!(result.is_ok());
       let names = result.unwrap();
       assert_eq!(names.len(), 2);
       assert!(names.contains(&"one".to_string()));
       assert!(names.contains(&"two".to_string()));
   }

   #[wasm_bindgen_test]
   fn test_validate_valid_input() {
       let jsonl = r#"{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}"#;
       let result = pixelsrc::wasm::validate(jsonl);
       assert!(result.is_ok());
       let messages = result.unwrap();
       assert!(messages.is_empty(), "Valid input should have no messages");
   }

   #[wasm_bindgen_test]
   fn test_validate_invalid_input() {
       let jsonl = r#"{"type":"sprite","name":"bad""#;
       let result = pixelsrc::wasm::validate(jsonl);
       assert!(result.is_ok());
       let messages = result.unwrap();
       assert!(!messages.is_empty(), "Invalid input should have error messages");
   }

   #[wasm_bindgen_test]
   fn test_render_with_transparency() {
       let jsonl = r#"{"type":"sprite","name":"trans","palette":{"{_}":"#00000000","{x}":"#FF0000"},"grid":["{_}{x}","{x}{_}"]}"#;
       let result = pixelsrc::wasm::render_to_rgba(jsonl, None);
       assert!(result.is_ok());
       let render_result = result.unwrap();
       assert_eq!(render_result.width(), 2);
       assert_eq!(render_result.height(), 2);
   }
   ```

2. Update `Cargo.toml` dev-dependencies:
   ```toml
   [dev-dependencies]
   tempfile = "3"
   wasm-bindgen-test = "0.3"
   ```

**Verification:**
```bash
# Run Rust WASM tests
wasm-pack test --node --features wasm

# Run browser tests (requires Chrome/Firefox)
wasm-pack test --chrome --features wasm
```

**Dependencies:** Task 6.3

---

### Task 6.5: CI Pipeline

**Wave:** 4 (parallel with 6.4)

GitHub Actions for WASM build and npm publish.

**Deliverables:**

1. Create `.github/workflows/wasm.yml`:
   ```yaml
   name: WASM Build

   on:
     push:
       branches: [main]
       paths:
         - 'src/**'
         - 'Cargo.toml'
         - 'wasm/**'
     pull_request:
       branches: [main]

   jobs:
     build:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4

         - name: Install Rust
           uses: dtolnay/rust-action@stable
           with:
             targets: wasm32-unknown-unknown

         - name: Install wasm-pack
           run: cargo install wasm-pack

         - name: Build WASM
           run: wasm-pack build --target web --features wasm --out-dir wasm/pkg

         - name: Run WASM tests (Node)
           run: wasm-pack test --node --features wasm

         - name: Run npm tests
           run: |
             cd wasm
             npm test

         - name: Upload artifact
           uses: actions/upload-artifact@v4
           with:
             name: wasm-pkg
             path: wasm/pkg/

     publish:
       needs: build
       runs-on: ubuntu-latest
       if: github.ref == 'refs/heads/main' && github.event_name == 'push'
       steps:
         - uses: actions/checkout@v4

         - uses: actions/download-artifact@v4
           with:
             name: wasm-pkg
             path: wasm/pkg/

         - uses: actions/setup-node@v4
           with:
             node-version: '20'
             registry-url: 'https://registry.npmjs.org'

         - name: Publish to npm
           run: |
             cd wasm
             npm publish --access public || echo "Version already published"
           env:
             NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
   ```

**Verification:**
```bash
# Verify workflow syntax
# Push to branch and check GitHub Actions tab

# Or test locally with act (optional)
act -j build
```

**Dependencies:** Task 6.3

---

## Example Usage After Phase 6

### Browser
```html
<script type="module">
  import init, { render_to_png } from '@pixelsrc/wasm';

  await init();

  const jsonl = '{"type":"sprite",...}';
  const png = render_to_png(jsonl);

  const blob = new Blob([png], { type: 'image/png' });
  document.getElementById('img').src = URL.createObjectURL(blob);
</script>
```

### Node.js
```javascript
import init, { render_to_png } from '@pixelsrc/wasm';
import { writeFileSync } from 'fs';

await init();
const png = render_to_png('{"type":"sprite",...}');
writeFileSync('output.png', png);
```

### Cloudflare Worker
```javascript
import init, { render_to_png } from '@pixelsrc/wasm';

export default {
  async fetch(request) {
    await init();
    const jsonl = await request.text();
    const png = render_to_png(jsonl);
    return new Response(png, {
      headers: { 'Content-Type': 'image/png' }
    });
  }
};
```

---

## Verification Summary

```bash
# 1. WASM builds successfully
wasm-pack build --target web --features wasm --out-dir wasm/pkg

# 2. All tests pass
cargo test
wasm-pack test --node --features wasm
cd wasm && npm test

# 3. Package can be published
cd wasm && npm pack --dry-run

# 4. Manual browser test
# Create simple HTML page that imports and uses the module
```

---

## Future Considerations

Features considered but not included in Phase 6:

| Feature | Rationale for Deferral |
|---------|------------------------|
| Streaming parse | JSONL is already line-based; full parse is fast enough |
| Animation GIF | Phase 3 not complete; add when ready |
| Worker thread support | Adds complexity; optimize later if needed |
| SIMD optimization | Not widely supported yet; premature optimization |
