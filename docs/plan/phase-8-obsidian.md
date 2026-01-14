# Phase 8: Obsidian Plugin

**Goal:** Render pixelsrc code blocks in Obsidian markdown notes

**Status:** Complete

**Depends on:** Phase 6 complete (WASM)

---

## Scope

Phase 8 creates an Obsidian plugin that:
- Detects ```pixelsrc or ```pxl code blocks
- Renders sprites inline as images
- Supports live preview in edit mode
- Provides "Copy as PNG" context action
- Configurable default scale and appearance

**Not in scope:** In-Obsidian sprite editing, palette management UI, animation playback

---

## Task Dependency Diagram

```
                          PHASE 8 TASK FLOW
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
│  │   8.1 Plugin Scaffold                                    │   │
│  │   - Clone obsidian-sample-plugin                         │   │
│  │   - Configure manifest.json                              │   │
│  │   - Setup build with esbuild                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (Core)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   8.2 WASM Integration                                   │   │
│  │   - Bundle @pixelsrc/wasm                                │   │
│  │   - Initialize on plugin load                            │   │
│  │   - Handle WASM loading in Electron                      │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 3 (Rendering)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   8.3 Code Block   │  │   8.4 Live Preview │                 │
│  │   Processor        │  │   Widget           │                 │
│  │   - Read mode      │  │   - Edit mode      │                 │
│  │   - Parse + render │  │   - CodeMirror ext │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 4 (Polish)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   8.5 Settings     │  │   8.6 Publish      │                 │
│  │   - Scale option   │  │   - Community      │                 │
│  │   - Background     │  │   - Documentation  │                 │
│  │   - Copy action    │  │   - Release        │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 8.1              (1 task - setup)                      │
│  Wave 2: 8.2              (1 task - WASM)                       │
│  Wave 3: 8.3 + 8.4        (2 tasks in parallel)                 │
│  Wave 4: 8.5 + 8.6        (2 tasks in parallel)                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 8.1: Plugin Scaffold

**Wave:** 1

Create Obsidian plugin project structure.

**Deliverables:**

1. Create `obsidian-pixelsrc/` directory from template:
   ```bash
   git clone https://github.com/obsidianmd/obsidian-sample-plugin obsidian-pixelsrc
   cd obsidian-pixelsrc
   rm -rf .git
   npm install
   ```

2. Update `manifest.json`:
   ```json
   {
     "id": "pixelsrc",
     "name": "PixelSrc",
     "version": "0.1.0",
     "minAppVersion": "1.0.0",
     "description": "Render pixelsrc pixel art code blocks in your notes",
     "author": "Your Name",
     "authorUrl": "https://github.com/yourusername",
     "fundingUrl": "",
     "isDesktopOnly": false
   }
   ```

3. Update `package.json`:
   ```json
   {
     "name": "obsidian-pixelsrc",
     "version": "0.1.0",
     "description": "Obsidian plugin for rendering pixelsrc pixel art",
     "main": "main.js",
     "scripts": {
       "dev": "node esbuild.config.mjs",
       "build": "tsc -noEmit -skipLibCheck && node esbuild.config.mjs production",
       "version": "node version-bump.mjs && git add manifest.json versions.json"
     },
     "keywords": ["obsidian", "pixelsrc", "pixel-art"],
     "author": "",
     "license": "MIT",
     "devDependencies": {
       "@types/node": "^20.0.0",
       "@typescript-eslint/eslint-plugin": "^6.0.0",
       "@typescript-eslint/parser": "^6.0.0",
       "builtin-modules": "^3.3.0",
       "esbuild": "^0.19.0",
       "obsidian": "latest",
       "tslib": "^2.6.0",
       "typescript": "^5.0.0"
     },
     "dependencies": {
       "@pixelsrc/wasm": "^0.1.0"
     }
   }
   ```

4. Create project structure:
   ```
   obsidian-pixelsrc/
   ├── manifest.json
   ├── package.json
   ├── tsconfig.json
   ├── esbuild.config.mjs
   ├── src/
   │   ├── main.ts
   │   ├── settings.ts
   │   ├── codeblock.ts
   │   ├── livepreview.ts
   │   └── renderer.ts
   └── styles.css
   ```

**Verification:**
```bash
cd obsidian-pixelsrc
npm install
npm run build
# Verify main.js is generated
```

**Dependencies:** Phase 6 complete

---

### Task 8.2: WASM Integration

**Wave:** 2

Bundle and initialize @pixelsrc/wasm for Obsidian.

**Deliverables:**

1. Update `esbuild.config.mjs` to handle WASM:
   ```javascript
   import esbuild from "esbuild";
   import process from "process";
   import builtins from "builtin-modules";
   import { copyFileSync } from "fs";

   const prod = process.argv[2] === "production";

   const context = await esbuild.context({
     entryPoints: ["src/main.ts"],
     bundle: true,
     external: [
       "obsidian",
       "electron",
       "@codemirror/autocomplete",
       "@codemirror/collab",
       "@codemirror/commands",
       "@codemirror/language",
       "@codemirror/lint",
       "@codemirror/search",
       "@codemirror/state",
       "@codemirror/view",
       "@lezer/common",
       "@lezer/highlight",
       "@lezer/lr",
       ...builtins,
     ],
     format: "cjs",
     target: "es2018",
     logLevel: "info",
     sourcemap: prod ? false : "inline",
     treeShaking: true,
     outfile: "main.js",
     loader: {
       ".wasm": "binary",
     },
   });

   if (prod) {
     await context.rebuild();
     process.exit(0);
   } else {
     await context.watch();
   }
   ```

2. Create `src/renderer.ts`:
   ```typescript
   import init, { render_to_png, render_to_rgba, validate } from '@pixelsrc/wasm';
   // Import WASM binary directly for bundling
   import wasmBinary from '@pixelsrc/wasm/pixelsrc_bg.wasm';

   let initialized = false;

   export async function initWasm(): Promise<void> {
     if (initialized) return;

     try {
       // Initialize with bundled WASM binary
       await init(wasmBinary);
       initialized = true;
       console.log('PixelSrc WASM initialized');
     } catch (error) {
       console.error('Failed to initialize PixelSrc WASM:', error);
       throw error;
     }
   }

   export function renderToPng(jsonl: string): Uint8Array {
     if (!initialized) {
       throw new Error('WASM not initialized');
     }
     return render_to_png(jsonl);
   }

   export function renderToRgba(jsonl: string): { width: number; height: number; pixels: Uint8Array } {
     if (!initialized) {
       throw new Error('WASM not initialized');
     }
     const result = render_to_rgba(jsonl);
     return {
       width: result.width,
       height: result.height,
       pixels: result.pixels,
     };
   }

   export function validateJsonl(jsonl: string): string[] {
     if (!initialized) {
       throw new Error('WASM not initialized');
     }
     return validate(jsonl);
   }
   ```

3. Update `src/main.ts`:
   ```typescript
   import { Plugin } from 'obsidian';
   import { initWasm } from './renderer';
   import { PixelsrcSettingTab, DEFAULT_SETTINGS, PixelsrcSettings } from './settings';
   import { registerCodeBlockProcessor } from './codeblock';
   import { registerLivePreviewExtension } from './livepreview';

   export default class PixelsrcPlugin extends Plugin {
     settings: PixelsrcSettings;

     async onload() {
       console.log('Loading PixelSrc plugin');

       // Load settings
       await this.loadSettings();

       // Initialize WASM
       try {
         await initWasm();
       } catch (error) {
         console.error('PixelSrc: Failed to initialize WASM', error);
         return;
       }

       // Register code block processor for reading mode
       registerCodeBlockProcessor(this);

       // Register live preview extension for edit mode
       registerLivePreviewExtension(this);

       // Add settings tab
       this.addSettingTab(new PixelsrcSettingTab(this.app, this));
     }

     onunload() {
       console.log('Unloading PixelSrc plugin');
     }

     async loadSettings() {
       this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
     }

     async saveSettings() {
       await this.saveData(this.settings);
     }
   }
   ```

**Verification:**
```bash
npm run build
# Copy to Obsidian vault .obsidian/plugins/pixelsrc/
# Enable plugin in Obsidian settings
# Check console for "PixelSrc WASM initialized"
```

**Dependencies:** Task 8.1

---

### Task 8.3: Code Block Processor

**Wave:** 3 (parallel with 8.4)

Render pixelsrc blocks in reading mode.

**Deliverables:**

1. Create `src/codeblock.ts`:
   ```typescript
   import { MarkdownPostProcessorContext } from 'obsidian';
   import type PixelsrcPlugin from './main';
   import { renderToPng, validateJsonl } from './renderer';

   export function registerCodeBlockProcessor(plugin: PixelsrcPlugin) {
     // Register for both 'pixelsrc' and 'pxl' languages
     plugin.registerMarkdownCodeBlockProcessor('pixelsrc', (source, el, ctx) => {
       processCodeBlock(source, el, ctx, plugin);
     });

     plugin.registerMarkdownCodeBlockProcessor('pxl', (source, el, ctx) => {
       processCodeBlock(source, el, ctx, plugin);
     });
   }

   function processCodeBlock(
     source: string,
     el: HTMLElement,
     ctx: MarkdownPostProcessorContext,
     plugin: PixelsrcPlugin
   ) {
     const container = el.createDiv({ cls: 'pixelsrc-container' });

     // Validate first
     const errors = validateJsonl(source);
     const hasErrors = errors.some(e => e.startsWith('Error:'));

     if (hasErrors) {
       // Show error state
       const errorDiv = container.createDiv({ cls: 'pixelsrc-error' });
       errorDiv.createEl('strong', { text: 'PixelSrc Error' });
       const errorList = errorDiv.createEl('ul');
       for (const error of errors) {
         errorList.createEl('li', { text: error });
       }
       return;
     }

     try {
       // Render to PNG
       const pngBytes = renderToPng(source);

       // Create image element
       const blob = new Blob([pngBytes], { type: 'image/png' });
       const url = URL.createObjectURL(blob);

       const img = container.createEl('img', {
         cls: 'pixelsrc-image',
         attr: {
           src: url,
           alt: 'PixelSrc sprite',
         },
       });

       // Apply scale from settings
       const scale = plugin.settings.defaultScale;
       img.style.imageRendering = 'pixelated';

       // Optional: Show warnings
       const warnings = errors.filter(e => e.startsWith('Warning:'));
       if (warnings.length > 0 && plugin.settings.showWarnings) {
         const warnDiv = container.createDiv({ cls: 'pixelsrc-warnings' });
         for (const warn of warnings) {
           warnDiv.createEl('small', { text: warn });
         }
       }

       // Context menu for copy
       container.addEventListener('contextmenu', (e) => {
         e.preventDefault();
         showContextMenu(e, pngBytes, plugin);
       });

       // Cleanup blob URL when element is removed
       const observer = new MutationObserver((mutations) => {
         for (const mutation of mutations) {
           for (const node of mutation.removedNodes) {
             if (node === container || node.contains?.(container)) {
               URL.revokeObjectURL(url);
               observer.disconnect();
               return;
             }
           }
         }
       });
       observer.observe(el.parentElement!, { childList: true, subtree: true });

     } catch (error) {
       console.error('PixelSrc render error:', error);
       container.createDiv({
         cls: 'pixelsrc-error',
         text: `Render failed: ${error}`,
       });
     }
   }

   function showContextMenu(e: MouseEvent, pngBytes: Uint8Array, plugin: PixelsrcPlugin) {
     const menu = new (plugin.app as any).Menu();

     menu.addItem((item: any) => {
       item
         .setTitle('Copy as PNG')
         .setIcon('copy')
         .onClick(async () => {
           try {
             const blob = new Blob([pngBytes], { type: 'image/png' });
             await navigator.clipboard.write([
               new ClipboardItem({ 'image/png': blob })
             ]);
           } catch (err) {
             console.error('Failed to copy:', err);
           }
         });
     });

     menu.showAtMouseEvent(e);
   }
   ```

2. Add styles in `styles.css`:
   ```css
   .pixelsrc-container {
     padding: 1rem;
     background: var(--background-secondary);
     border-radius: 4px;
     display: flex;
     flex-direction: column;
     align-items: center;
     gap: 0.5rem;
   }

   .pixelsrc-image {
     image-rendering: pixelated;
     image-rendering: crisp-edges;
     max-width: 100%;
     cursor: context-menu;
   }

   .pixelsrc-error {
     color: var(--text-error);
     padding: 0.5rem;
     border: 1px solid var(--text-error);
     border-radius: 4px;
     font-size: 0.9em;
   }

   .pixelsrc-warnings {
     color: var(--text-warning);
     font-size: 0.8em;
   }

   /* Checkered background for transparency */
   .pixelsrc-container.show-transparency {
     background-image:
       linear-gradient(45deg, var(--background-modifier-border) 25%, transparent 25%),
       linear-gradient(-45deg, var(--background-modifier-border) 25%, transparent 25%),
       linear-gradient(45deg, transparent 75%, var(--background-modifier-border) 75%),
       linear-gradient(-45deg, transparent 75%, var(--background-modifier-border) 75%);
     background-size: 16px 16px;
     background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
   }
   ```

**Verification:**
```bash
npm run build
# In Obsidian, create note with:
# ```pixelsrc
# {"type":"sprite","name":"test","palette":{"{x}":"#FF0000"},"grid":["{x}"]}
# ```
# Verify image renders in reading mode
# Right-click and verify "Copy as PNG" works
```

**Dependencies:** Task 8.2

---

### Task 8.4: Live Preview Widget

**Wave:** 3 (parallel with 8.3)

Render pixelsrc inline while editing.

**Deliverables:**

1. Create `src/livepreview.ts`:
   ```typescript
   import { EditorView, WidgetType, Decoration, DecorationSet, ViewPlugin, ViewUpdate } from '@codemirror/view';
   import { syntaxTree } from '@codemirror/language';
   import type PixelsrcPlugin from './main';
   import { renderToPng } from './renderer';

   class PixelsrcWidget extends WidgetType {
     constructor(private source: string, private scale: number) {
       super();
     }

     toDOM() {
       const container = document.createElement('div');
       container.className = 'pixelsrc-widget';

       try {
         const pngBytes = renderToPng(this.source);
         const blob = new Blob([pngBytes], { type: 'image/png' });
         const url = URL.createObjectURL(blob);

         const img = document.createElement('img');
         img.src = url;
         img.className = 'pixelsrc-image';
         img.style.imageRendering = 'pixelated';

         container.appendChild(img);

         // Cleanup on remove
         const observer = new MutationObserver(() => {
           if (!document.body.contains(container)) {
             URL.revokeObjectURL(url);
             observer.disconnect();
           }
         });
         observer.observe(document.body, { childList: true, subtree: true });

       } catch (error) {
         container.className = 'pixelsrc-widget pixelsrc-error';
         container.textContent = 'Render error';
       }

       return container;
     }

     eq(other: PixelsrcWidget) {
       return this.source === other.source;
     }
   }

   function findCodeBlocks(view: EditorView): { from: number; to: number; source: string }[] {
     const blocks: { from: number; to: number; source: string }[] = [];
     const doc = view.state.doc;

     // Simple regex-based approach for finding code blocks
     const text = doc.toString();
     const regex = /```(?:pixelsrc|pxl)\n([\s\S]*?)```/g;

     let match;
     while ((match = regex.exec(text)) !== null) {
       const from = match.index;
       const to = from + match[0].length;
       const source = match[1];
       blocks.push({ from, to, source });
     }

     return blocks;
   }

   function buildDecorations(view: EditorView, scale: number): DecorationSet {
     const decorations: any[] = [];
     const blocks = findCodeBlocks(view);

     for (const block of blocks) {
       // Add widget after the code block
       const widget = Decoration.widget({
         widget: new PixelsrcWidget(block.source, scale),
         side: 1,
       });
       decorations.push(widget.range(block.to));
     }

     return Decoration.set(decorations);
   }

   export function registerLivePreviewExtension(plugin: PixelsrcPlugin) {
     const livePreviewPlugin = ViewPlugin.fromClass(
       class {
         decorations: DecorationSet;

         constructor(view: EditorView) {
           this.decorations = buildDecorations(view, plugin.settings.defaultScale);
         }

         update(update: ViewUpdate) {
           if (update.docChanged || update.viewportChanged) {
             this.decorations = buildDecorations(update.view, plugin.settings.defaultScale);
           }
         }
       },
       {
         decorations: (v) => v.decorations,
       }
     );

     plugin.registerEditorExtension(livePreviewPlugin);
   }
   ```

2. Add widget styles to `styles.css`:
   ```css
   .pixelsrc-widget {
     display: block;
     padding: 0.5rem;
     margin: 0.5rem 0;
     background: var(--background-secondary);
     border-radius: 4px;
     text-align: center;
   }

   .pixelsrc-widget .pixelsrc-image {
     max-width: 100%;
     image-rendering: pixelated;
   }

   .pixelsrc-widget.pixelsrc-error {
     color: var(--text-error);
     font-size: 0.9em;
   }
   ```

**Verification:**
```bash
npm run build
# In Obsidian, create note with pixelsrc code block
# Switch to Live Preview mode (not Source mode)
# Verify image appears below code block
# Edit the code and verify preview updates
```

**Dependencies:** Task 8.2

---

### Task 8.5: Settings Tab

**Wave:** 4 (parallel with 8.6)

Add plugin settings UI.

**Deliverables:**

1. Create `src/settings.ts`:
   ```typescript
   import { App, PluginSettingTab, Setting } from 'obsidian';
   import type PixelsrcPlugin from './main';

   export interface PixelsrcSettings {
     defaultScale: number;
     showWarnings: boolean;
     showTransparency: boolean;
     enableLivePreview: boolean;
   }

   export const DEFAULT_SETTINGS: PixelsrcSettings = {
     defaultScale: 4,
     showWarnings: false,
     showTransparency: true,
     enableLivePreview: true,
   };

   export class PixelsrcSettingTab extends PluginSettingTab {
     plugin: PixelsrcPlugin;

     constructor(app: App, plugin: PixelsrcPlugin) {
       super(app, plugin);
       this.plugin = plugin;
     }

     display(): void {
       const { containerEl } = this;
       containerEl.empty();

       containerEl.createEl('h2', { text: 'PixelSrc Settings' });

       new Setting(containerEl)
         .setName('Default Scale')
         .setDesc('Scale factor for rendered sprites (1-16)')
         .addSlider((slider) =>
           slider
             .setLimits(1, 16, 1)
             .setValue(this.plugin.settings.defaultScale)
             .setDynamicTooltip()
             .onChange(async (value) => {
               this.plugin.settings.defaultScale = value;
               await this.plugin.saveSettings();
             })
         );

       new Setting(containerEl)
         .setName('Show Warnings')
         .setDesc('Display rendering warnings below sprites')
         .addToggle((toggle) =>
           toggle
             .setValue(this.plugin.settings.showWarnings)
             .onChange(async (value) => {
               this.plugin.settings.showWarnings = value;
               await this.plugin.saveSettings();
             })
         );

       new Setting(containerEl)
         .setName('Transparency Background')
         .setDesc('Show checkered background for transparent pixels')
         .addToggle((toggle) =>
           toggle
             .setValue(this.plugin.settings.showTransparency)
             .onChange(async (value) => {
               this.plugin.settings.showTransparency = value;
               await this.plugin.saveSettings();
             })
         );

       new Setting(containerEl)
         .setName('Live Preview')
         .setDesc('Show sprite preview while editing (requires restart)')
         .addToggle((toggle) =>
           toggle
             .setValue(this.plugin.settings.enableLivePreview)
             .onChange(async (value) => {
               this.plugin.settings.enableLivePreview = value;
               await this.plugin.saveSettings();
             })
         );

       containerEl.createEl('h3', { text: 'Usage' });
       containerEl.createEl('p', {
         text: 'Create a code block with language "pixelsrc" or "pxl":',
       });
       containerEl.createEl('pre', {
         text: '```pixelsrc\n{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}\n```',
       });

       containerEl.createEl('h3', { text: 'Links' });
       const linkContainer = containerEl.createDiv();
       linkContainer.createEl('a', {
         text: 'PixelSrc Documentation',
         href: 'https://github.com/user/pixelsrc',
       });
       linkContainer.createEl('br');
       linkContainer.createEl('a', {
         text: 'Online Editor',
         href: 'https://pixelsrc.dev',
       });
     }
   }
   ```

**Verification:**
```bash
npm run build
# In Obsidian, go to Settings → Community Plugins → PixelSrc
# Verify settings appear and can be changed
# Change scale, verify sprites re-render
```

**Dependencies:** Tasks 8.3, 8.4

---

### Task 8.6: Publish to Community Plugins

**Wave:** 4 (parallel with 8.5)

Prepare and submit plugin for Obsidian Community Plugins.

**Deliverables:**

1. Create `README.md`:
   ```markdown
   # PixelSrc for Obsidian

   Render [pixelsrc](https://github.com/user/pixelsrc) pixel art sprites directly in your Obsidian notes.

   ## Features

   - Render pixelsrc code blocks as images
   - Live preview while editing
   - Copy sprites as PNG images
   - Configurable scale and appearance
   - Support for both `pixelsrc` and `pxl` code block languages

   ## Usage

   Create a code block with language `pixelsrc` or `pxl`:

   ~~~markdown
   ```pixelsrc
   {"type":"sprite","name":"heart","palette":{"{_}":"#00000000","{r}":"#FF0000"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}
   ```
   ~~~

   The sprite will render as an image in both reading mode and live preview.

   ## Settings

   - **Default Scale**: Scale factor for rendered sprites (1-16x)
   - **Show Warnings**: Display rendering warnings below sprites
   - **Transparency Background**: Show checkered background for transparent pixels
   - **Live Preview**: Show sprite preview while editing

   ## Installation

   ### From Community Plugins (Recommended)

   1. Open Settings → Community Plugins
   2. Disable Safe Mode
   3. Click Browse and search for "PixelSrc"
   4. Install and enable the plugin

   ### Manual Installation

   1. Download the latest release from GitHub
   2. Extract to `.obsidian/plugins/pixelsrc/`
   3. Reload Obsidian and enable the plugin

   ## Links

   - [PixelSrc Documentation](https://github.com/user/pixelsrc)
   - [Online Editor](https://pixelsrc.dev)
   - [Report Issues](https://github.com/user/obsidian-pixelsrc/issues)

   ## License

   MIT
   ```

2. Create `versions.json`:
   ```json
   {
     "0.1.0": "1.0.0"
   }
   ```

3. Create `.github/workflows/release.yml`:
   ```yaml
   name: Release Obsidian Plugin

   on:
     push:
       tags:
         - '*'

   jobs:
     build:
       runs-on: ubuntu-latest

       steps:
         - uses: actions/checkout@v4

         - name: Setup Node
           uses: actions/setup-node@v4
           with:
             node-version: '20'

         - name: Install dependencies
           run: npm ci

         - name: Build
           run: npm run build

         - name: Create Release
           uses: softprops/action-gh-release@v1
           with:
             files: |
               main.js
               manifest.json
               styles.css
   ```

4. Submit to Obsidian Community Plugins:
   - Fork https://github.com/obsidianmd/obsidian-releases
   - Add entry to `community-plugins.json`
   - Submit pull request

**Verification:**
```bash
# Build release
npm run build

# Verify files exist
ls -la main.js manifest.json styles.css

# Test in fresh vault before submission
```

**Dependencies:** Tasks 8.3, 8.4, 8.5

---

## Example Usage

In an Obsidian note:

~~~markdown
# My Pixel Art Collection

## Heart

```pixelsrc
{"type":"sprite","name":"heart","palette":{"{_}":"#00000000","{r}":"#FF0000","{p}":"#FF6B6B"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{p}{r}{r}{p}{r}{r}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}
```

## Game Character

```pxl
{"type":"palette","name":"skin","colors":{"{_}":"#00000000","{s}":"#FFD5B8","{h}":"#8B4513","{e}":"#000000"}}
{"type":"sprite","name":"face","palette":"skin","size":[8,8],"grid":["{_}{h}{h}{h}{h}{h}{h}{_}","{h}{s}{s}{s}{s}{s}{s}{h}","{h}{s}{e}{s}{s}{e}{s}{h}","{h}{s}{s}{s}{s}{s}{s}{h}","{h}{s}{s}{_}{_}{s}{s}{h}","{h}{s}{s}{s}{s}{s}{s}{h}","{_}{h}{s}{s}{s}{s}{h}{_}","{_}{_}{h}{h}{h}{h}{_}{_}"]}
```
~~~

---

## Verification Summary

```bash
# 1. Build succeeds
cd obsidian-pixelsrc
npm run build

# 2. Plugin loads in Obsidian
# Copy main.js, manifest.json, styles.css to vault
# Enable plugin, check console for errors

# 3. Reading mode works
# Create note with pixelsrc code block
# Switch to reading mode, verify image renders

# 4. Live preview works
# Switch to Live Preview mode
# Verify inline preview appears

# 5. Settings work
# Open plugin settings
# Change scale, verify sprites update

# 6. Copy works
# Right-click rendered sprite
# Click "Copy as PNG"
# Paste in image editor
```

---

## Future Considerations

Features considered but not included in Phase 8:

| Feature | Rationale for Deferral |
|---------|------------------------|
| In-editor sprite editing | Significant complexity; use online editor |
| Palette UI | Visual picker would be separate plugin |
| Animation playback | Needs Phase 3 complete |
| Sprite catalog sidebar | Nice-to-have, not essential |
| Hover preview for code | Could add later as enhancement |
