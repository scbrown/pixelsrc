# @stiwi/pixelsrc-wasm

WebAssembly build of [pixelsrc](https://github.com/scbrown/pixelsrc) - a GenAI-native pixel art format and renderer.

## Installation

```bash
npm install @stiwi/pixelsrc-wasm
```

## Usage (Browser)

```javascript
import init, { render_to_png, render_to_rgba, list_sprites } from '@stiwi/pixelsrc-wasm';

// Initialize WASM module (required before first use)
await init();

// Define a sprite in pixelsrc JSONL format
const jsonl = `{"type":"sprite","name":"heart","size":[7,5],"palette":{"_":"#00000000","r":"#FF0000"},"regions":{"r":{"union":[{"rect":[1,0,2,1]},{"rect":[4,0,2,1]},{"rect":[0,1,7,1]},{"rect":[1,2,5,1]},{"rect":[2,3,3,1]},{"points":[[3,4]]}],"z":0}}}`;

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
import init, { render_to_png } from '@stiwi/pixelsrc-wasm';

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
{"type":"palette","name":"mono","colors":{"_":"#00000000","on":"#FFFFFF","off":"#000000"}}
{"type":"sprite","name":"checker","size":[4,2],"palette":"mono","regions":{"on":{"points":[[0,0],[2,0],[1,1],[3,1]],"z":0},"off":{"points":[[1,0],[3,0],[0,1],[2,1]],"z":0}}}
```

See the [full documentation](https://github.com/scbrown/pixelsrc) for more details.

## License

MIT
