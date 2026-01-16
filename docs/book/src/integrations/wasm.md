# WASM Module

The `@pixelsrc/wasm` package provides a WebAssembly build of the Pixelsrc renderer for use in browsers and Node.js.

## Installation

```bash
npm install @pixelsrc/wasm
```

Or with other package managers:

```bash
yarn add @pixelsrc/wasm
pnpm add @pixelsrc/wasm
```

## Browser Usage

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
import init, { render_to_png } from '@pixelsrc/wasm';

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
import init, { render_to_png } from '@pixelsrc/wasm';

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
import init, { render_to_png } from '@pixelsrc/wasm';

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

## Related

- [Web Editor](web-editor.md) - Browser-based editor using WASM
- [Obsidian Plugin](obsidian.md) - Uses WASM for rendering
