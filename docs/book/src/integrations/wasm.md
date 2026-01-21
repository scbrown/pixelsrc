# WASM Module

The `@stiwi/pixelsrc-wasm` package provides a WebAssembly build of the Pixelsrc renderer for use in browsers and Node.js.

## Installation

```bash
npm install @stiwi/pixelsrc-wasm
```

Or with other package managers:

```bash
yarn add @stiwi/pixelsrc-wasm
pnpm add @stiwi/pixelsrc-wasm
```

## Browser Usage

```javascript
import init, { render_to_png, render_to_rgba, list_sprites } from '@stiwi/pixelsrc-wasm';

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
```

## Canvas Rendering

For direct canvas rendering, use `render_to_rgba`:

```javascript
const result = render_to_rgba(jsonl);
const imageData = new ImageData(
  new Uint8ClampedArray(result.pixels),
  result.width,
  result.height
);
ctx.putImageData(imageData, 0, 0);
```

## Node.js Usage

```javascript
import { readFileSync, writeFileSync } from 'fs';
import init, { render_to_png } from '@stiwi/pixelsrc-wasm';

await init();

const jsonl = readFileSync('sprite.pxl', 'utf8');
const pngBytes = render_to_png(jsonl);
writeFileSync('sprite.png', pngBytes);
```

## API Reference

### `init(): Promise<void>`

Initialize the WASM module. Must be called once before using any other function.

### `render_to_png(jsonl: string, spriteName?: string): Uint8Array`

Render JSONL input to PNG bytes.

| Parameter | Type | Description |
|-----------|------|-------------|
| `jsonl` | string | Pixelsrc JSONL content |
| `spriteName` | string? | Optional: render specific sprite |

**Returns:** PNG image as `Uint8Array`

### `render_to_rgba(jsonl: string, spriteName?: string): RenderResult`

Render to raw RGBA pixel data for canvas manipulation.

**Returns:**

| Property | Type | Description |
|----------|------|-------------|
| `width` | number | Image width in pixels |
| `height` | number | Image height in pixels |
| `pixels` | Uint8Array | Raw RGBA pixel data |
| `warnings` | string[] | Any rendering warnings |

### `list_sprites(jsonl: string): string[]`

Get list of sprite and composition names defined in the input.

### `validate(jsonl: string): string[]`

Validate JSONL without rendering. Returns array of error/warning messages. Empty array means valid input.

## Build Targets

The WASM module supports multiple bundler targets:

| Target | Use Case | Build Command |
|--------|----------|---------------|
| `web` | Browser ES modules | `npm run build` |
| `nodejs` | Node.js | `npm run build:nodejs` |
| `bundler` | Webpack/Rollup | `npm run build:bundler` |

## Error Handling

```javascript
try {
  const pngBytes = render_to_png(jsonl);
  // Success
} catch (err) {
  // Handle parsing or rendering errors
  console.error('Render failed:', err.message);
}
```

Common errors:
- Invalid JSON syntax
- Undefined palette tokens
- Missing required fields

## Performance Tips

1. **Initialize once**: Call `init()` once at app startup, not per render
2. **Reuse results**: Cache PNG blobs when displaying the same sprite multiple times
3. **Use RGBA for animations**: `render_to_rgba` is faster for frequent canvas updates
4. **Batch validation**: Use `validate()` for quick syntax checks before rendering

## Framework Integration

### React

```jsx
import { useEffect, useState, useRef } from 'react';
import init, { render_to_png } from '@stiwi/pixelsrc-wasm';

function PixelSprite({ jsonl }) {
  const [imgUrl, setImgUrl] = useState(null);
  const initRef = useRef(false);

  useEffect(() => {
    async function render() {
      if (!initRef.current) {
        await init();
        initRef.current = true;
      }
      const png = render_to_png(jsonl);
      const blob = new Blob([png], { type: 'image/png' });
      setImgUrl(URL.createObjectURL(blob));
    }
    render();
  }, [jsonl]);

  return imgUrl ? <img src={imgUrl} alt="Pixel sprite" /> : null;
}
```

### Vue

```vue
<template>
  <img v-if="imgUrl" :src="imgUrl" alt="Pixel sprite" />
</template>

<script setup>
import { ref, watch, onMounted } from 'vue';
import init, { render_to_png } from '@stiwi/pixelsrc-wasm';

const props = defineProps(['jsonl']);
const imgUrl = ref(null);
let wasmReady = false;

onMounted(async () => {
  await init();
  wasmReady = true;
  render();
});

watch(() => props.jsonl, render);

function render() {
  if (!wasmReady) return;
  const png = render_to_png(props.jsonl);
  const blob = new Blob([png], { type: 'image/png' });
  imgUrl.value = URL.createObjectURL(blob);
}
</script>
```

## Browser Compatibility

The WASM module requires a modern browser with WebAssembly support.

### Supported Browsers

| Browser | Minimum Version | Notes |
|---------|-----------------|-------|
| Chrome | 90+ | Full support |
| Firefox | 88+ | Full support |
| Safari | 14+ | Full support |
| Edge | 90+ | Full support (Chromium-based) |

### Required Features

The WASM module relies on the following browser APIs:

- **WebAssembly** - Core requirement for running the compiled Rust code
- **ES6 Modules** - Dynamic import for loading the WASM module
- **Blob API** - For creating image URLs from PNG bytes
- **URL.createObjectURL** - For displaying rendered images
- **Promises** - For async initialization

### Feature Detection

The documentation demos include built-in feature detection:

```javascript
// Check if WASM is supported
if (typeof WebAssembly === 'object' &&
    typeof WebAssembly.instantiate === 'function') {
  // WASM is supported
}

// Using the pixelsrcDemo API (in documentation pages)
const support = pixelsrcDemo.getBrowserSupport();
if (support.isCompatible) {
  // All required features available
} else {
  console.log('Missing:', support);
}
```

### Fallback Behavior

When running in an unsupported browser:

- Demo containers display a helpful error message
- The page remains functional (text content, navigation)
- Static images are used where possible as fallbacks

### Cross-Browser CSS

The documentation uses cross-browser compatible CSS for pixel art rendering:

```css
/* Pixel-perfect image scaling */
img {
  image-rendering: -moz-crisp-edges;    /* Firefox */
  image-rendering: -webkit-optimize-contrast; /* Safari */
  image-rendering: pixelated;           /* Chrome, Edge */
  image-rendering: crisp-edges;         /* Standard */
}
```

### Testing Checklist

When deploying WASM-based features, verify:

- [ ] Chrome (latest and 2 versions back)
- [ ] Firefox (latest and 2 versions back)
- [ ] Safari (latest)
- [ ] Edge (latest)
- [ ] Mobile Safari (iOS 14+)
- [ ] Chrome for Android (latest)

### Known Issues

- **Safari 13 and earlier**: No WebAssembly SIMD support (module loads but may be slower)
- **IE11**: Not supported (no WebAssembly)
- **Some corporate proxies**: May block `.wasm` file downloads

## Chronicle Integration

[Chronicle](https://git.lan/stiwi/chronicle) is a narrative engine for games and graphic novels that uses pixelsrc as its asset rendering layer.

### How Chronicle Uses Pixelsrc

Chronicle embeds pixelsrc WASM internally to:
- Pre-render character sprites (all emotions/variants)
- Pre-render location backgrounds
- Generate animation frames for portraits
- Compose scenes (background + positioned characters)
- Render comic panels

Game clients interact with Chronicle, not pixelsrc directly:

```
Game Client → Chronicle (WASM) → Pixelsrc (embedded)
                  │
                  └─► Pre-rendered PNGs
```

### Asset Flow

1. Game generates pixelsrc JSONL (via AI or authoring)
2. Chronicle receives: `createCharacter(def, pixelsrc_jsonl)`
3. Chronicle stores the JSONL
4. Chronicle calls pixelsrc to render → caches PNG
5. Later: Chronicle returns cached PNG on request

### Why Embed vs. Direct Use

Chronicle embeds pixelsrc rather than exposing it to clients because:
- **Pre-rendering**: Assets rendered once at creation, not on demand
- **Caching**: Chronicle manages the render cache
- **Simplicity**: Clients only deal with PNGs, not JSONL
- **Consistency**: All rendering goes through one path

### Generating Assets for Chronicle

When generating assets for use with Chronicle, follow the standard pixelsrc format:

```jsonl
{"type": "palette", "name": "hero_colors", "{skin}": "#FFDAB9", "{hair}": "#8B4513", "{armor}": "#4682B4"}
{"type": "sprite", "name": "hero_neutral", "palette": "hero_colors", "grid": [...]}
{"type": "sprite", "name": "hero_happy", "palette": "hero_colors", "grid": [...]}
{"type": "animation", "name": "hero_talk", "frames": ["hero_talk_1", "hero_talk_2"], "duration": 100}
```

Chronicle will parse this JSONL and render each sprite/animation to its cache.

## Related

- [Web Editor](web-editor.md) - Browser-based editor using WASM
- [Obsidian Plugin](obsidian.md) - Uses WASM for rendering
