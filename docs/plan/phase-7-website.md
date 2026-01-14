# Phase 7: Interactive Website

**Goal:** Browser-based pixelsrc editor and previewer

**Status:** Complete

**Depends on:** Phase 6 complete (WASM)

---

## Scope

Phase 7 creates a web application for:
- Live editing pixelsrc JSONL with syntax highlighting
- Real-time preview as you type
- PNG download and clipboard copy
- Shareable URLs (encode pixelsrc in URL hash)
- Example gallery for learning
- Mobile-friendly responsive design

**Not in scope:** User accounts, cloud storage, collaboration, palette editor UI

---

## Task Dependency Diagram

```
                          PHASE 7 TASK FLOW
═══════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────┐
│                      Phase 6 Complete                           │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 1 (Setup)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   7.1 Project Setup                                      │   │
│  │   - Vite + TypeScript scaffold                           │   │
│  │   - @pixelsrc/wasm integration                           │   │
│  │   - Basic HTML/CSS structure                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (Core Components - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   7.2 Editor       │  │   7.3 Preview      │                 │
│  │   - CodeMirror 6   │  │   - Canvas render  │                 │
│  │   - JSON syntax    │  │   - Auto-scale     │                 │
│  │   - Error markers  │  │   - Grid overlay   │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 3 (Features)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   7.4 Export       │  │   7.5 URL Sharing  │                 │
│  │   - Download PNG   │  │   - lz-string      │                 │
│  │   - Copy to clip   │  │   - Hash routing   │                 │
│  │   - Scale options  │  │   - Short URLs     │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 4 (Polish)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   7.6 Gallery      │  │   7.7 Deploy       │                 │
│  │   - Example sprites│  │   - GitHub Pages   │                 │
│  │   - Load on click  │  │   - Custom domain  │                 │
│  │   - Categories     │  │   - CI/CD          │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 7.1              (1 task - setup)                      │
│  Wave 2: 7.2 + 7.3        (2 tasks in parallel)                 │
│  Wave 3: 7.4 + 7.5        (2 tasks in parallel)                 │
│  Wave 4: 7.6 + 7.7        (2 tasks in parallel)                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 7.1: Project Setup

**Wave:** 1

Scaffold web application with Vite and TypeScript.

**Deliverables:**

1. Create `website/` directory with Vite project:
   ```bash
   npm create vite@latest website -- --template vanilla-ts
   cd website
   npm install
   npm install @pixelsrc/wasm
   npm install lz-string
   ```

2. Project structure:
   ```
   website/
   ├── index.html
   ├── package.json
   ├── tsconfig.json
   ├── vite.config.ts
   ├── src/
   │   ├── main.ts
   │   ├── editor.ts
   │   ├── preview.ts
   │   ├── export.ts
   │   ├── sharing.ts
   │   ├── gallery.ts
   │   └── style.css
   └── public/
       └── examples/
           ├── heart.jsonl
           ├── hero.jsonl
           └── ...
   ```

3. Basic `index.html`:
   ```html
   <!DOCTYPE html>
   <html lang="en">
   <head>
     <meta charset="UTF-8">
     <meta name="viewport" content="width=device-width, initial-scale=1.0">
     <title>PixelSrc - Pixel Art Editor</title>
     <meta name="description" content="Browser-based pixel art editor using the pixelsrc format">
     <link rel="stylesheet" href="/src/style.css">
   </head>
   <body>
     <div id="app">
       <header>
         <h1>PixelSrc</h1>
         <nav>
           <button id="btn-download">Download PNG</button>
           <button id="btn-copy">Copy to Clipboard</button>
           <button id="btn-share">Share URL</button>
         </nav>
       </header>
       <main>
         <section id="editor-panel">
           <div id="editor"></div>
         </section>
         <section id="preview-panel">
           <canvas id="preview"></canvas>
         </section>
       </main>
       <aside id="gallery-panel">
         <h2>Examples</h2>
         <div id="gallery"></div>
       </aside>
     </div>
     <script type="module" src="/src/main.ts"></script>
   </body>
   </html>
   ```

4. Basic `src/main.ts`:
   ```typescript
   import init from '@pixelsrc/wasm';
   import { setupEditor } from './editor';
   import { setupPreview } from './preview';
   import { setupExport } from './export';
   import { setupSharing, loadFromHash } from './sharing';
   import { setupGallery } from './gallery';
   import './style.css';

   async function main() {
     // Initialize WASM
     await init();

     // Setup components
     const editor = setupEditor(document.getElementById('editor')!);
     const preview = setupPreview(document.getElementById('preview') as HTMLCanvasElement);

     setupExport(preview);
     setupSharing(editor);
     setupGallery(editor);

     // Connect editor to preview
     editor.onChange((content) => {
       preview.render(content);
     });

     // Load from URL hash if present
     const initial = loadFromHash() || getDefaultExample();
     editor.setValue(initial);
   }

   function getDefaultExample(): string {
     return `{"type":"sprite","name":"heart","palette":{"{_}":"#00000000","{r}":"#FF0000","{p}":"#FF6B6B"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{p}{r}{r}{p}{r}{r}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}`;
   }

   main();
   ```

**Verification:**
```bash
cd website
npm run dev
# Open http://localhost:5173 and verify basic structure loads
```

**Dependencies:** Phase 6 complete

---

### Task 7.2: Editor Component

**Wave:** 2 (parallel with 7.3)

Implement code editor with syntax highlighting.

**Deliverables:**

1. Install CodeMirror:
   ```bash
   npm install @codemirror/state @codemirror/view @codemirror/lang-json @codemirror/theme-one-dark
   ```

2. Create `src/editor.ts`:
   ```typescript
   import { EditorState } from '@codemirror/state';
   import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
   import { json } from '@codemirror/lang-json';
   import { oneDark } from '@codemirror/theme-one-dark';
   import { defaultKeymap } from '@codemirror/commands';

   export interface Editor {
     getValue(): string;
     setValue(content: string): void;
     onChange(callback: (content: string) => void): void;
   }

   export function setupEditor(container: HTMLElement): Editor {
     let changeCallback: ((content: string) => void) | null = null;

     const updateListener = EditorView.updateListener.of((update) => {
       if (update.docChanged && changeCallback) {
         changeCallback(update.state.doc.toString());
       }
     });

     const state = EditorState.create({
       doc: '',
       extensions: [
         lineNumbers(),
         highlightActiveLine(),
         json(),
         oneDark,
         keymap.of(defaultKeymap),
         updateListener,
         EditorView.lineWrapping,
       ],
     });

     const view = new EditorView({
       state,
       parent: container,
     });

     return {
       getValue() {
         return view.state.doc.toString();
       },

       setValue(content: string) {
         view.dispatch({
           changes: {
             from: 0,
             to: view.state.doc.length,
             insert: content,
           },
         });
       },

       onChange(callback) {
         changeCallback = callback;
       },
     };
   }
   ```

**Verification:**
```bash
npm run dev
# Verify editor loads with syntax highlighting
# Type JSON and verify it highlights correctly
```

**Dependencies:** Task 7.1

---

### Task 7.3: Preview Component

**Wave:** 2 (parallel with 7.2)

Implement canvas-based preview with auto-scaling.

**Deliverables:**

1. Create `src/preview.ts`:
   ```typescript
   import { render_to_rgba, validate } from '@pixelsrc/wasm';

   export interface Preview {
     render(jsonl: string): void;
     getImageData(): ImageData | null;
     getCanvas(): HTMLCanvasElement;
   }

   export function setupPreview(canvas: HTMLCanvasElement): Preview {
     const ctx = canvas.getContext('2d')!;
     let lastImageData: ImageData | null = null;
     let renderTimeout: number | null = null;

     // Debounce rendering for performance
     function debouncedRender(jsonl: string) {
       if (renderTimeout) {
         clearTimeout(renderTimeout);
       }
       renderTimeout = window.setTimeout(() => {
         doRender(jsonl);
       }, 100);
     }

     function doRender(jsonl: string) {
       try {
         // Clear previous
         ctx.clearRect(0, 0, canvas.width, canvas.height);

         if (!jsonl.trim()) {
           lastImageData = null;
           return;
         }

         const result = render_to_rgba(jsonl);
         const { width, height, pixels, warnings } = result;

         // Log warnings
         if (warnings.length > 0) {
           console.warn('Render warnings:', warnings);
         }

         // Create ImageData
         const imageData = new ImageData(
           new Uint8ClampedArray(pixels),
           width,
           height
         );
         lastImageData = imageData;

         // Scale to fit canvas while maintaining aspect ratio
         const scale = calculateScale(width, height, canvas.clientWidth, canvas.clientHeight);
         const scaledWidth = width * scale;
         const scaledHeight = height * scale;

         // Set canvas size
         canvas.width = scaledWidth;
         canvas.height = scaledHeight;

         // Draw scaled (nearest-neighbor for pixel art)
         ctx.imageSmoothingEnabled = false;

         // Create temporary canvas for the original size
         const tempCanvas = document.createElement('canvas');
         tempCanvas.width = width;
         tempCanvas.height = height;
         tempCanvas.getContext('2d')!.putImageData(imageData, 0, 0);

         // Draw scaled
         ctx.drawImage(tempCanvas, 0, 0, scaledWidth, scaledHeight);

       } catch (error) {
         console.error('Render error:', error);
         // Show error state
         ctx.fillStyle = '#FF00FF';
         ctx.fillRect(0, 0, 50, 50);
         ctx.fillStyle = '#FFFFFF';
         ctx.font = '12px monospace';
         ctx.fillText('Error', 5, 30);
       }
     }

     function calculateScale(srcW: number, srcH: number, maxW: number, maxH: number): number {
       // Integer scaling for pixel art
       const scaleX = Math.floor(maxW / srcW) || 1;
       const scaleY = Math.floor(maxH / srcH) || 1;
       return Math.min(scaleX, scaleY, 16); // Cap at 16x
     }

     return {
       render(jsonl: string) {
         debouncedRender(jsonl);
       },

       getImageData() {
         return lastImageData;
       },

       getCanvas() {
         return canvas;
       },
     };
   }
   ```

2. Add CSS for preview panel in `src/style.css`:
   ```css
   #preview-panel {
     display: flex;
     align-items: center;
     justify-content: center;
     background: #1a1a1a;
     background-image:
       linear-gradient(45deg, #222 25%, transparent 25%),
       linear-gradient(-45deg, #222 25%, transparent 25%),
       linear-gradient(45deg, transparent 75%, #222 75%),
       linear-gradient(-45deg, transparent 75%, #222 75%);
     background-size: 20px 20px;
     background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
   }

   #preview {
     image-rendering: pixelated;
     image-rendering: crisp-edges;
   }
   ```

**Verification:**
```bash
npm run dev
# Type valid pixelsrc JSONL
# Verify preview updates in real-time
# Verify pixel art is crisp (not blurred)
```

**Dependencies:** Task 7.1

---

### Task 7.4: Export Features

**Wave:** 3 (parallel with 7.5)

Implement PNG download and clipboard copy.

**Deliverables:**

1. Create `src/export.ts`:
   ```typescript
   import { render_to_png } from '@pixelsrc/wasm';
   import type { Preview } from './preview';

   let currentJsonl: string = '';

   export function setCurrentJsonl(jsonl: string) {
     currentJsonl = jsonl;
   }

   export function setupExport(preview: Preview) {
     const downloadBtn = document.getElementById('btn-download')!;
     const copyBtn = document.getElementById('btn-copy')!;

     downloadBtn.addEventListener('click', () => downloadPng());
     copyBtn.addEventListener('click', () => copyToClipboard(preview));
   }

   async function downloadPng(scale: number = 4) {
     if (!currentJsonl.trim()) {
       alert('No sprite to download');
       return;
     }

     try {
       const pngBytes = render_to_png(currentJsonl);
       const blob = new Blob([pngBytes], { type: 'image/png' });
       const url = URL.createObjectURL(blob);

       const a = document.createElement('a');
       a.href = url;
       a.download = 'pixelsrc-sprite.png';
       a.click();

       URL.revokeObjectURL(url);
     } catch (error) {
       console.error('Download failed:', error);
       alert('Failed to generate PNG');
     }
   }

   async function copyToClipboard(preview: Preview) {
     const imageData = preview.getImageData();
     if (!imageData) {
       alert('No image to copy');
       return;
     }

     try {
       // Create a canvas with the image
       const canvas = document.createElement('canvas');
       canvas.width = imageData.width;
       canvas.height = imageData.height;
       canvas.getContext('2d')!.putImageData(imageData, 0, 0);

       // Convert to blob and copy
       const blob = await new Promise<Blob>((resolve, reject) => {
         canvas.toBlob((b) => {
           if (b) resolve(b);
           else reject(new Error('Failed to create blob'));
         }, 'image/png');
       });

       await navigator.clipboard.write([
         new ClipboardItem({ 'image/png': blob })
       ]);

       // Visual feedback
       const copyBtn = document.getElementById('btn-copy')!;
       const originalText = copyBtn.textContent;
       copyBtn.textContent = 'Copied!';
       setTimeout(() => {
         copyBtn.textContent = originalText;
       }, 1500);

     } catch (error) {
       console.error('Copy failed:', error);
       alert('Failed to copy to clipboard');
     }
   }
   ```

2. Add scale selector UI (optional enhancement):
   ```html
   <select id="scale-select">
     <option value="1">1x</option>
     <option value="2">2x</option>
     <option value="4" selected>4x</option>
     <option value="8">8x</option>
   </select>
   ```

**Verification:**
```bash
npm run dev
# Click Download PNG - verify file downloads
# Click Copy to Clipboard - verify image copies (paste into image editor)
```

**Dependencies:** Tasks 7.2, 7.3

---

### Task 7.5: URL Sharing

**Wave:** 3 (parallel with 7.4)

Implement shareable URLs using hash encoding.

**Deliverables:**

1. Install lz-string:
   ```bash
   npm install lz-string
   npm install -D @types/lz-string
   ```

2. Create `src/sharing.ts`:
   ```typescript
   import LZString from 'lz-string';
   import type { Editor } from './editor';

   export function setupSharing(editor: Editor) {
     const shareBtn = document.getElementById('btn-share')!;

     shareBtn.addEventListener('click', () => {
       const jsonl = editor.getValue();
       if (!jsonl.trim()) {
         alert('Nothing to share');
         return;
       }

       const url = createShareUrl(jsonl);
       copyUrlToClipboard(url);
     });

     // Listen for hash changes
     window.addEventListener('hashchange', () => {
       const content = loadFromHash();
       if (content) {
         editor.setValue(content);
       }
     });
   }

   export function createShareUrl(jsonl: string): string {
     const compressed = LZString.compressToEncodedURIComponent(jsonl);
     const url = new URL(window.location.href);
     url.hash = compressed;
     return url.toString();
   }

   export function loadFromHash(): string | null {
     const hash = window.location.hash.slice(1); // Remove #
     if (!hash) return null;

     try {
       const decompressed = LZString.decompressFromEncodedURIComponent(hash);
       return decompressed;
     } catch (error) {
       console.error('Failed to decompress hash:', error);
       return null;
     }
   }

   async function copyUrlToClipboard(url: string) {
     try {
       await navigator.clipboard.writeText(url);

       const shareBtn = document.getElementById('btn-share')!;
       const originalText = shareBtn.textContent;
       shareBtn.textContent = 'URL Copied!';
       setTimeout(() => {
         shareBtn.textContent = originalText;
       }, 1500);

     } catch (error) {
       // Fallback for older browsers
       prompt('Copy this URL:', url);
     }
   }
   ```

3. Update `src/main.ts` to save to hash on change:
   ```typescript
   // In main(), after editor setup:
   editor.onChange((content) => {
     preview.render(content);
     setCurrentJsonl(content);
     // Update URL hash (debounced)
     updateHashDebounced(content);
   });

   let hashTimeout: number | null = null;
   function updateHashDebounced(content: string) {
     if (hashTimeout) clearTimeout(hashTimeout);
     hashTimeout = window.setTimeout(() => {
       const compressed = LZString.compressToEncodedURIComponent(content);
       history.replaceState(null, '', '#' + compressed);
     }, 1000);
   }
   ```

**Verification:**
```bash
npm run dev
# Edit sprite, click Share URL
# Paste URL in new tab - verify same sprite loads
# Edit sprite, verify URL hash updates automatically
```

**Dependencies:** Tasks 7.2, 7.3

---

### Task 7.6: Example Gallery

**Wave:** 4 (parallel with 7.7)

Implement gallery of example sprites.

**Deliverables:**

1. Create example files in `public/examples/`:
   - `heart.jsonl`
   - `hero.jsonl`
   - `coin.jsonl`
   - `tree.jsonl`
   - `sword.jsonl`

2. Create `src/gallery.ts`:
   ```typescript
   import type { Editor } from './editor';

   interface Example {
     name: string;
     file: string;
     preview?: string; // Optional pre-rendered preview
   }

   const EXAMPLES: Example[] = [
     { name: 'Heart', file: 'heart.jsonl' },
     { name: 'Hero', file: 'hero.jsonl' },
     { name: 'Coin', file: 'coin.jsonl' },
     { name: 'Tree', file: 'tree.jsonl' },
     { name: 'Sword', file: 'sword.jsonl' },
   ];

   export function setupGallery(editor: Editor) {
     const container = document.getElementById('gallery')!;

     for (const example of EXAMPLES) {
       const button = document.createElement('button');
       button.className = 'gallery-item';
       button.textContent = example.name;
       button.addEventListener('click', () => loadExample(example, editor));
       container.appendChild(button);
     }
   }

   async function loadExample(example: Example, editor: Editor) {
     try {
       const response = await fetch(`/examples/${example.file}`);
       if (!response.ok) throw new Error(`HTTP ${response.status}`);
       const content = await response.text();
       editor.setValue(content);
     } catch (error) {
       console.error('Failed to load example:', error);
       alert(`Failed to load ${example.name}`);
     }
   }
   ```

3. Add gallery styles:
   ```css
   #gallery-panel {
     padding: 1rem;
     background: #2a2a2a;
     border-left: 1px solid #333;
   }

   .gallery-item {
     display: block;
     width: 100%;
     padding: 0.5rem 1rem;
     margin-bottom: 0.5rem;
     background: #3a3a3a;
     border: none;
     border-radius: 4px;
     color: #fff;
     cursor: pointer;
     text-align: left;
   }

   .gallery-item:hover {
     background: #4a4a4a;
   }
   ```

**Verification:**
```bash
npm run dev
# Click example buttons
# Verify examples load into editor and preview
```

**Dependencies:** Tasks 7.2, 7.3

---

### Task 7.7: Deployment

**Wave:** 4 (parallel with 7.6)

Deploy to GitHub Pages with CI/CD.

**Deliverables:**

1. Update `vite.config.ts` for GitHub Pages:
   ```typescript
   import { defineConfig } from 'vite';

   export default defineConfig({
     base: '/pixelsrc/', // Adjust for repo name
     build: {
       outDir: 'dist',
     },
   });
   ```

2. Create `.github/workflows/website.yml`:
   ```yaml
   name: Deploy Website

   on:
     push:
       branches: [main]
       paths:
         - 'website/**'
     workflow_dispatch:

   permissions:
     contents: read
     pages: write
     id-token: write

   concurrency:
     group: "pages"
     cancel-in-progress: false

   jobs:
     build:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4

         - name: Setup Node
           uses: actions/setup-node@v4
           with:
             node-version: '20'
             cache: 'npm'
             cache-dependency-path: website/package-lock.json

         - name: Install dependencies
           run: |
             cd website
             npm ci

         - name: Build
           run: |
             cd website
             npm run build

         - name: Upload artifact
           uses: actions/upload-pages-artifact@v3
           with:
             path: website/dist

     deploy:
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       runs-on: ubuntu-latest
       needs: build
       steps:
         - name: Deploy to GitHub Pages
           id: deployment
           uses: actions/deploy-pages@v4
   ```

3. Add `website/.gitignore`:
   ```
   node_modules/
   dist/
   ```

**Verification:**
```bash
# Local build test
cd website
npm run build
npm run preview

# Push to main and verify GitHub Pages deployment
```

**Dependencies:** Tasks 7.1-7.6

---

## Responsive Design

The website should work on mobile devices:

```css
/* Mobile-first responsive layout */
@media (max-width: 768px) {
  main {
    flex-direction: column;
  }

  #editor-panel,
  #preview-panel {
    width: 100%;
    height: 50vh;
  }

  #gallery-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: auto;
    max-height: 30vh;
    overflow-y: auto;
    border-left: none;
    border-top: 1px solid #333;
  }

  .gallery-item {
    display: inline-block;
    width: auto;
    margin-right: 0.5rem;
  }
}
```

---

## Verification Summary

```bash
# 1. Development server works
cd website && npm run dev

# 2. Build succeeds
npm run build

# 3. Features work:
# - Type JSONL, preview updates
# - Download PNG works
# - Copy to clipboard works
# - Share URL works
# - Examples load correctly
# - Mobile responsive

# 4. Deployment works
# Push to main, verify GitHub Pages updates
```

---

## Future Considerations

Features considered but not included in Phase 7:

| Feature | Rationale for Deferral |
|---------|------------------------|
| User accounts | Adds significant complexity; URL sharing is sufficient |
| Palette editor UI | Visual palette picker could be Phase 11 |
| Animation preview | Needs Phase 3 complete first |
| PWA / offline mode | Nice-to-have, not essential |
| Drag-and-drop file loading | Can add later as enhancement |
