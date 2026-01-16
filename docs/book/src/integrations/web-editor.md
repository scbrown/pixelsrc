# Web Editor

The Pixelsrc web editor provides a browser-based environment for creating and previewing pixel art without installing anything.

**Try it:** [pixelsrc.dev](https://pixelsrc.dev)

## Features

- **Live preview** - See sprites render as you type
- **Example gallery** - Start from pre-built examples
- **Export options** - Download PNG or copy to clipboard
- **Shareable URLs** - Share your creations via URL hash
- **Keyboard shortcuts** - Fast editing workflow

## Interface Overview

The editor has three main panels:

| Panel | Purpose |
|-------|---------|
| Editor | Write and edit Pixelsrc JSONL |
| Preview | Real-time rendered output |
| Gallery | Example sprites to load and modify |

## Getting Started

1. Open the [web editor](https://pixelsrc.dev)
2. Click an example from the Gallery to load it
3. Modify the JSONL in the Editor panel
4. Watch the Preview update in real-time

Or paste your own Pixelsrc JSONL directly into the editor.

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Render immediately | `Ctrl/Cmd + Enter` |
| Select all | `Ctrl/Cmd + A` |

The editor auto-renders as you type with a slight debounce delay. Use `Ctrl/Cmd + Enter` to force an immediate render.

## Export Options

### Download PNG

1. Select a scale factor (1x, 2x, 4x, or 8x)
2. Click **Download PNG**
3. File saves as `pixelsrc-{scale}x.png`

### Copy to Clipboard

1. Select a scale factor
2. Click **Copy**
3. Paste directly into other applications

The clipboard copy produces a PNG image, compatible with design tools, chat apps, and document editors.

## Sharing Sprites

The editor stores your JSONL in the URL hash. To share:

1. Create or load a sprite
2. Copy the browser URL
3. Share the link

Recipients see your exact sprite when they open the link.

**Example URL structure:**
```
https://pixelsrc.dev/#eJyLVkrOz0nVUcpMUbJRKs9ILUpVslKKBQAkDgWV
```

The hash contains compressed JSONL, keeping URLs reasonably short even for complex sprites.

## Error Handling

When your JSONL contains errors, the editor shows helpful messages:

| Error | Meaning | Fix |
|-------|---------|-----|
| Invalid JSON syntax | Missing quotes, commas, or brackets | Check JSON formatting |
| Palette error | Undefined color token in grid | Define all tokens in palette |
| Grid error | Inconsistent row lengths | Ensure all rows have same token count |
| Missing "type" field | Object lacks required type | Add `"type": "sprite"` or similar |

## Offline Usage

The editor works offline after initial load. It uses:
- Service worker caching for assets
- WASM module stored in browser
- No server-side rendering required

Bookmark the editor for quick access even without internet.

## Mobile Support

The web editor works on mobile devices with some considerations:

- **Touch editing** - Standard text input in editor panel
- **Pinch zoom** - Zoom preview with two fingers
- **Responsive layout** - Panels stack vertically on narrow screens

For the best mobile experience, use landscape orientation.

## Embedding

Embed the editor in your own site using an iframe:

```html
<iframe
  src="https://pixelsrc.dev/#YOUR_COMPRESSED_HASH"
  width="100%"
  height="600"
  style="border: 1px solid #ccc;"
></iframe>
```

To embed a specific sprite, include the compressed JSONL hash in the URL.

## Local Development

Run the editor locally:

```bash
cd website
npm install
npm run dev
```

This starts a Vite development server with hot module replacement.

### Building for Production

```bash
npm run build
```

Output goes to `website/dist/` ready for static hosting.

## Technical Details

The web editor is built with:

- **TypeScript** - Type-safe codebase
- **Vite** - Fast development and bundling
- **WASM** - `@pixelsrc/wasm` for rendering
- **LZ-String** - URL hash compression

No framework dependencies. The editor uses vanilla DOM manipulation for minimal bundle size.

## Related

- [WASM Module](wasm.md) - The rendering library
- [Format Specification](../format/overview.md) - JSONL format reference
- [AI Generation](../ai-generation/system-prompts.md) - Generate sprites with AI
