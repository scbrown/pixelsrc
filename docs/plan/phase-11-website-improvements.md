# Phase 11: Website Improvements

**Goal:** Fix broken deployment and transform the site from ugly/non-functional to polished and delightful

**Status:** Planning

**Depends on:** Phase 7 complete (Website exists)

---

## Current Problems

### Critical (Site Broken)
1. **Wrong base path**: `vite.config.ts` still references old project name in base path
   - Assets (JS, CSS, WASM) fail to load due to 404s
   - Site appears empty/broken

### Major UX Issues
2. **No loading states**: WASM initialization takes time, no feedback to user
3. **No rendering feedback**: User doesn't know if render is in progress or failed
4. **Gallery doesn't show previews**: Just names, no visual thumbnails

### Visual/Polish Issues
5. **Bland design**: Generic dark theme, no personality
6. **No branding**: Just text header, no logo or visual identity
7. **Poor visual hierarchy**: Everything looks the same weight
8. **No micro-interactions**: Buttons feel dead, no hover/active states worth mentioning
9. **No empty states**: Blank canvas is just... blank

### Accessibility Issues
10. **No keyboard navigation**: Can't use site without a mouse
11. **Missing focus indicators**: Can't tell what's focused
12. **Color-only feedback**: Errors/success rely on red/green alone
13. **No screen reader support**: No ARIA labels, no live regions
14. **Animations ignore motion preferences**: No `prefers-reduced-motion` support
15. **Form labels missing**: Textarea has no label for screen readers

---

## Task Dependency Diagram

```
                        DEPENDENCY GRAPH
═══════════════════════════════════════════════════════════════════

                         ┌─────────┐
                         │  11.1   │  Fix Base Path (CRITICAL)
                         │ Deploy  │
                         └────┬────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
         ┌─────────┐    ┌─────────┐    ┌─────────┐
         │  11.2   │    │  11.3   │    │  11.4   │
         │ App     │    │ Render  │    │ Theme   │───────┐
         │ Loading │    │ Loading │    │ Dracula │       │
         └─────────┘    └─────────┘    └────┬────┘       │
                                            │            │
                              ┌─────────────┤            │
                              │             │            │
                              ▼             ▼            ▼
                         ┌─────────┐   ┌─────────┐  ┌─────────┐
                         │  11.5   │   │  11.6   │  │  11.7   │
                         │ Header  │   │ Gallery │  │ Preview │
                         │ Brand   │   │ Upgrade │  │ Polish  │
                         └─────────┘   └────┬────┘  └────┬────┘
                                            │            │
                              ┌─────────────┴────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │   11.8 + 11.9     │
                    │ Animations &      │
                    │ Accessibility     │
                    └───────────────────┘

═══════════════════════════════════════════════════════════════════


                      PARALLELIZATION WAVES
═══════════════════════════════════════════════════════════════════

  WAVE 1 ──────────────────────────────────────────────────────────
  │  11.1 Fix Base Path & Deployment                              │
  └────────────────────────────────────────────────────────────────

  WAVE 2 (after 11.1) ─────────────────────────────────────────────
  │  11.2 App Loading  ║  11.3 Render Loading  ║  11.4 Theme      │
  │                    ║                       ║                  │
  │  (parallel - no interdependencies)                            │
  └────────────────────────────────────────────────────────────────

  WAVE 3 (after 11.4) ─────────────────────────────────────────────
  │  11.5 Header*      ║  11.6 Gallery   ║  11.7 Preview          │
  │                    ║                 ║                        │
  │  (all need theme CSS variables from 11.4)                     │
  │  * 11.5 can start with 11.4 if CSS var names coordinated      │
  └────────────────────────────────────────────────────────────────

  WAVE 4 (after all above) ────────────────────────────────────────
  │  11.8 Animations   ║  11.9 Accessibility                      │
  │                    ║                                          │
  │  (final polish - needs complete UI to work with)              │
  └────────────────────────────────────────────────────────────────

═══════════════════════════════════════════════════════════════════


                      DEPENDENCY MATRIX
═══════════════════════════════════════════════════════════════════

  Task  │ Depends On        │ Blocks          │ Can Parallel With
  ──────┼───────────────────┼─────────────────┼───────────────────
  11.1  │ (none)            │ ALL             │ (none - do first)
  11.2  │ 11.1              │ 11.8, 11.9      │ 11.3, 11.4, 11.5
  11.3  │ 11.1              │ 11.8, 11.9      │ 11.2, 11.4, 11.5
  11.4  │ 11.1              │ 11.5, 11.6, 11.7│ 11.2, 11.3
  11.5  │ 11.1, 11.4*       │ 11.8, 11.9      │ 11.4* (if coordinated)
  11.6  │ 11.1, 11.4        │ 11.8, 11.9      │ 11.5, 11.7
  11.7  │ 11.1, 11.4        │ 11.8, 11.9      │ 11.5, 11.6
  11.8  │ 11.1-11.7         │ (none)          │ 11.9
  11.9  │ 11.1-11.7         │ (none)          │ 11.8

  * 11.5 uses CSS variables from 11.4. Can parallel if variable names agreed upfront.

═══════════════════════════════════════════════════════════════════


                    CRITICAL PATH (longest)
═══════════════════════════════════════════════════════════════════

  11.1 → 11.4 → 11.6 → 11.8
         (or)   (or)
         11.4 → 11.7 → 11.9

  Minimum sequential tasks: 4

═══════════════════════════════════════════════════════════════════
```

---

## Tasks

### Task 11.1: Fix Base Path & Deployment

**Wave:** 1 (Critical - must be first)

Fix the broken deployment so the site actually works.

**Problem:**
- `vite.config.ts` has stale base path referencing old project name
- Site is deployed to `scbrown.github.io/pixelsrc/`
- All assets 404, site is completely broken

**Deliverables:**

1. Update `website/vite.config.ts` to use correct base path:
   ```typescript
   export default defineConfig({
     // GitHub Pages base path - matches repo deployment
     base: process.env.VITE_BASE_PATH || '/pixelsrc/',
     // ... rest of config
   });
   ```

2. Search for and remove any remaining old project name references in the website directory

3. Verify GitHub Actions workflow uses correct base path

4. Test locally:
   ```bash
   cd website
   npm run build
   npm run preview
   ```

5. Verify WASM module loads correctly after fix

**Verification:**
```bash
# Build and test locally
cd website && npm run build && npm run preview
# Open http://localhost:4173/pixelsrc/
# Verify: Console shows "WASM module initialized"
# Verify: Gallery loads with example sprites
# Verify: Typing in editor shows preview
```

**Dependencies:** None

---

### Task 11.2: App Loading State

**Wave:** 2 (parallel with 11.3)

Add a loading state while WASM initializes.

**Problem:**
- WASM initialization takes 1-3 seconds
- During this time, the page appears broken
- User has no idea anything is happening

**Deliverables:**

1. Add loading overlay to `index.html`:
   ```html
   <div id="loading-overlay">
     <div class="loading-content">
       <div class="loading-logo">
         <span class="pixel-icon">▣</span>
         <span class="loading-title">Pixelsrc</span>
       </div>
       <div class="loading-spinner"></div>
       <p class="loading-text">Initializing...</p>
     </div>
   </div>
   ```

2. Add loading styles to `style.css` (Dracula themed):
   ```css
   #loading-overlay {
     position: fixed;
     inset: 0;
     background: var(--bg-primary);  /* Dracula Background #282a36 */
     display: flex;
     align-items: center;
     justify-content: center;
     z-index: 1000;
     transition: opacity 0.3s ease-out;
   }

   #loading-overlay.hidden {
     opacity: 0;
     pointer-events: none;
   }

   .loading-spinner {
     width: 40px;
     height: 40px;
     border: 3px solid var(--border);  /* Dracula Selection #44475a */
     border-top-color: var(--accent-purple);  /* Dracula Purple #bd93f9 */
     border-radius: 50%;
     animation: spin 1s linear infinite;
   }

   .loading-title {
     color: var(--accent-purple);  /* Dracula Purple */
   }

   @keyframes spin {
     to { transform: rotate(360deg); }
   }
   ```

3. Update `main.ts` to hide overlay after init:
   ```typescript
   async function initApp(): Promise<void> {
     const overlay = document.getElementById('loading-overlay');

     try {
       await init();
       wasmReady = true;
       // ... setup components

       // Hide loading overlay with animation
       overlay?.classList.add('hidden');
       setTimeout(() => overlay?.remove(), 300);
     } catch (err) {
       // Show user-friendly error in overlay
       const loadingText = overlay?.querySelector('.loading-text');
       const loadingContent = overlay?.querySelector('.loading-content');
       if (loadingText && loadingContent) {
         loadingText.textContent = 'Failed to load the editor';
         loadingText.classList.add('error');

         // Add helpful retry/info
         const helpText = document.createElement('p');
         helpText.className = 'loading-help';
         helpText.innerHTML = `
           <button onclick="location.reload()">Try again</button>
           <br><small>If this persists, try a different browser.</small>
         `;
         loadingContent.appendChild(helpText);

         console.error('WASM init failed:', err);
       }
     }
   }
   ```

**Verification:**
- Hard refresh page, see loading spinner
- Spinner disappears smoothly when ready
- If WASM fails, error message shows in overlay

**Dependencies:** Task 11.1

---

### Task 11.3: Render Loading State

**Wave:** 2 (parallel with 11.2)

Add loading/progress indication during rendering.

**Problem:**
- When render button is clicked, nothing visually happens
- Large sprites can take noticeable time
- Errors aren't visually distinct

**Deliverables:**

1. Add render state indicator to preview panel:
   ```html
   <div id="preview-status" class="preview-status hidden">
     <span class="status-icon"></span>
     <span class="status-text"></span>
   </div>
   ```

2. Add status styles (Dracula themed):
   ```css
   .preview-status {
     position: absolute;
     bottom: 1rem;
     left: 50%;
     transform: translateX(-50%);
     padding: 0.5rem 1rem;
     background: var(--bg-secondary);  /* Dracula darker bg */
     border: 1px solid var(--border);
     border-radius: 4px;
     font-size: 0.875rem;
     display: flex;
     align-items: center;
     gap: 0.5rem;
   }

   .preview-status.rendering .status-icon::before {
     content: "⟳";
     animation: spin 1s linear infinite;
     display: inline-block;
     color: var(--accent-purple);  /* Dracula Purple */
   }

   .preview-status.success .status-icon::before {
     content: "✓";
     color: var(--accent-green);  /* Dracula Green #50fa7b */
   }

   .preview-status.error .status-icon::before {
     content: "✗";
     color: var(--accent-red);  /* Dracula Red #ff5555 */
   }
   ```

3. Update render handling with **user-friendly error messages**:
   ```typescript
   function handleRender(): void {
     const status = document.getElementById('preview-status');

     // Show rendering state
     showStatus('rendering', 'Rendering...');

     // ... do render

     if (result.success) {
       showStatus('success', `${result.width}×${result.height}`);
       hideStatusAfter(2000);
     } else {
       // Convert technical errors to user-friendly messages
       showStatus('error', friendlyError(result.error));
     }
   }

   function friendlyError(error: string): string {
     // Map common errors to helpful messages
     if (error.includes('JSON')) {
       return 'Invalid JSON syntax. Check for missing quotes or commas.';
     }
     if (error.includes('palette')) {
       return 'Palette error. Make sure all colors in the grid are defined.';
     }
     if (error.includes('grid')) {
       return 'Grid error. Check that all rows have the same length.';
     }
     if (error.includes('type')) {
       return 'Missing or invalid "type" field. Try: {"type": "sprite", ...}';
     }
     // Fallback: show original but truncated
     return error.length > 60 ? error.slice(0, 60) + '...' : error;
   }
   ```

**Verification:**
- Click Render, see "Rendering..." briefly
- Success shows dimensions
- Invalid input shows error clearly

**Dependencies:** Task 11.1

---

### Task 11.4: Theme & Color System Overhaul (Dracula)

**Wave:** 3 (parallel with 11.5)

Replace the bland dark theme with the Dracula color scheme.

**Current problem:**
- Generic dark blue/gray palette
- No visual interest or brand identity
- Flat, lifeless appearance

**Solution:** Use the [Dracula](https://draculatheme.com/) color scheme - a popular, well-tested dark theme with excellent readability and a vibrant palette.

**Deliverables:**

1. Define CSS custom properties using Dracula palette:
   ```css
   :root {
     /* Dracula Background Colors */
     --bg-primary: #282a36;      /* Background */
     --bg-secondary: #21222c;    /* Darker background */
     --bg-panel: #44475a;        /* Current Line / Selection */
     --bg-elevated: #6272a4;     /* Comment (for elevated surfaces) */

     /* Dracula Accent Colors */
     --accent-primary: #ff79c6;  /* Pink */
     --accent-secondary: #8be9fd; /* Cyan */
     --accent-tertiary: #ffb86c;  /* Orange */
     --accent-green: #50fa7b;     /* Green */
     --accent-purple: #bd93f9;    /* Purple */
     --accent-red: #ff5555;       /* Red */
     --accent-yellow: #f1fa8c;    /* Yellow */

     /* Text - Dracula Foreground */
     --text-primary: #f8f8f2;     /* Foreground */
     --text-secondary: #6272a4;   /* Comment */
     --text-muted: #44475a;       /* Selection */

     /* Semantic (using Dracula colors) */
     --success: #50fa7b;          /* Green */
     --warning: #ffb86c;          /* Orange */
     --error: #ff5555;            /* Red */

     /* Borders */
     --border: #44475a;           /* Selection */
     --border-active: var(--accent-purple);

     /* Shadows */
     --shadow-sm: 0 2px 4px rgba(0, 0, 0, 0.3);
     --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
     --shadow-glow: 0 0 20px rgba(189, 147, 249, 0.3); /* Purple glow */
   }
   ```

2. Add subtle depth with Dracula colors:
   ```css
   body {
     background: var(--bg-primary);
   }

   .panel {
     background: var(--bg-secondary);
     border: 1px solid var(--border);
     box-shadow: var(--shadow-md);
   }

   .panel-header {
     background: var(--bg-panel);
   }
   ```

3. Use Dracula accent colors for interactive elements:
   ```css
   #render-btn {
     background: var(--accent-purple);
     color: var(--bg-primary);
     font-weight: 600;
   }

   #render-btn:hover {
     background: var(--accent-pink);
     transform: translateY(-2px);
     box-shadow: var(--shadow-glow);
   }

   .export-btn:hover {
     border-color: var(--accent-cyan);
     color: var(--accent-cyan);
   }
   ```

4. Apply Dracula to editor/code areas:
   ```css
   #editor {
     background: var(--bg-secondary);
     color: var(--text-primary);
     caret-color: var(--accent-pink);
   }

   #editor::selection {
     background: var(--bg-panel);
   }
   ```

**Verification:**
- Page uses consistent Dracula palette
- Colors match the official Dracula spec
- Excellent contrast and readability
- Accents (purple, pink, cyan) make interactive elements pop

**Dependencies:** Task 11.1

---

### Task 11.5: Header & Branding

**Wave:** 3 (parallel with 11.4)

Transform the plain text header into proper branding.

**Current problem:**
- Just "Pixelsrc" text and tagline
- No visual identity
- Doesn't feel like a product

**Deliverables:**

1. Create pixel-art inspired logo/wordmark using CSS:
   ```html
   <header id="header">
     <div class="brand">
       <div class="logo">
         <span class="logo-pixel p1"></span>
         <span class="logo-pixel p2"></span>
         <span class="logo-pixel p3"></span>
         <span class="logo-pixel p4"></span>
       </div>
       <div class="brand-text">
         <h1>Pixel<span class="accent">src</span></h1>
         <p class="tagline">GenAI-native pixel art format</p>
       </div>
     </div>
     <nav class="header-nav">
       <a href="https://github.com/scbrown/pixelsrc" class="nav-link" target="_blank">
         GitHub
       </a>
       <a href="#" class="nav-link" id="docs-link">
         Docs
       </a>
     </nav>
   </header>
   ```

2. Add logo animation/styling (Dracula themed):
   ```css
   .logo {
     display: grid;
     grid-template-columns: repeat(2, 12px);
     gap: 2px;
   }

   .logo-pixel {
     width: 12px;
     height: 12px;
     border-radius: 2px;
   }

   /* Using Dracula accent colors for the pixel logo */
   .logo-pixel.p1 { background: var(--accent-purple); }  /* #bd93f9 */
   .logo-pixel.p2 { background: var(--accent-cyan); }    /* #8be9fd */
   .logo-pixel.p3 { background: var(--accent-pink); }    /* #ff79c6 */
   .logo-pixel.p4 { background: var(--accent-green); }   /* #50fa7b */

   .brand h1 .accent {
     color: var(--accent-cyan);  /* Dracula Cyan */
   }
   ```

3. Add subtle header animation on load:
   ```css
   .logo-pixel {
     animation: pixelFadeIn 0.5s ease-out backwards;
   }
   .logo-pixel.p1 { animation-delay: 0.1s; }
   .logo-pixel.p2 { animation-delay: 0.2s; }
   .logo-pixel.p3 { animation-delay: 0.3s; }
   .logo-pixel.p4 { animation-delay: 0.4s; }

   @keyframes pixelFadeIn {
     from {
       opacity: 0;
       transform: scale(0);
     }
   }
   ```

**Verification:**
- Header looks professional and branded
- Logo animation plays on page load
- Navigation links work

**Dependencies:** Task 11.1, Task 11.4 (uses theme CSS variables)

**Can parallel with:** 11.4 if CSS variable names are coordinated

---

### Task 11.6: Gallery Upgrade

**Wave:** 4 (parallel with 11.7)

Transform gallery from boring text buttons to visual thumbnails.

**Current problem:**
- Gallery items are just text buttons
- No preview of what you'll get
- Not visually appealing

**Deliverables:**

1. Update gallery component to render thumbnails:
   ```typescript
   // In gallery.ts
   async function createGalleryItem(example: Example): Promise<HTMLElement> {
     const item = document.createElement('button');
     item.className = 'gallery-item';

     // Create thumbnail container
     const thumb = document.createElement('div');
     thumb.className = 'gallery-thumb';

     // Try to render thumbnail
     try {
       const response = await fetch(`${BASE_PATH}examples/${example.file}`);
       const jsonl = await response.text();
       const pngBytes = render_to_png(jsonl);
       const blob = new Blob([pngBytes], { type: 'image/png' });
       const img = document.createElement('img');
       img.src = URL.createObjectURL(blob);
       img.alt = example.name;
       thumb.appendChild(img);
     } catch {
       // Fallback to first letter
       const fallback = document.createElement('span');
       fallback.className = 'gallery-fallback';
       fallback.textContent = example.name[0];
       thumb.appendChild(fallback);
     }

     // Label
     const label = document.createElement('span');
     label.className = 'gallery-label';
     label.textContent = example.name;

     item.appendChild(thumb);
     item.appendChild(label);
     return item;
   }
   ```

2. Add hover effects and better grid (Dracula themed):
   ```css
   .gallery-grid {
     display: grid;
     grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
     gap: 1rem;
   }

   .gallery-item {
     aspect-ratio: 1;
     display: flex;
     flex-direction: column;
     align-items: center;
     justify-content: center;
     background: var(--bg-panel);  /* Dracula Selection #44475a */
     border: 2px solid transparent;
     border-radius: 8px;
     padding: 0.75rem;
     cursor: pointer;
     transition: all 0.2s ease;
   }

   .gallery-item:hover {
     border-color: var(--accent-purple);  /* Dracula Purple */
     transform: translateY(-4px);
     box-shadow: var(--shadow-glow);
   }

   .gallery-thumb img {
     max-width: 48px;
     max-height: 48px;
     image-rendering: pixelated;
   }
   ```

3. Add loading state for gallery:
   ```css
   .gallery-item.loading .gallery-thumb::after {
     content: "";
     width: 24px;
     height: 24px;
     border: 2px solid var(--border);
     border-top-color: var(--accent-purple);  /* Dracula Purple */
     border-radius: 50%;
     animation: spin 1s linear infinite;
   }
   ```

**Verification:**
- Gallery shows visual thumbnails
- Hover effects feel good
- Clicking loads example into editor

**Dependencies:** Tasks 11.1, 11.4

---

### Task 11.7: Preview Panel Polish

**Wave:** 4 (parallel with 11.6)

Improve the preview panel with better empty states and controls.

**Current problem:**
- Empty preview is just a checkered box
- No indication of what to do
- No zoom/view controls

**Deliverables:**

1. Add helpful empty state with onboarding:
   ```html
   <div id="preview-empty" class="preview-empty">
     <div class="empty-icon" aria-hidden="true">🎨</div>
     <p class="empty-text">Your pixel art will appear here</p>
     <p class="empty-hint">
       Click an example below to get started,<br>
       or paste Pixelsrc JSONL in the editor
     </p>
     <button class="empty-cta" id="load-first-example">
       Try the Heart example
     </button>
   </div>
   ```

2. Style empty state (Dracula themed):
   ```css
   .preview-empty {
     display: flex;
     flex-direction: column;
     align-items: center;
     justify-content: center;
     height: 100%;
     color: var(--text-secondary);  /* Dracula Comment #6272a4 */
     text-align: center;
   }

   .empty-icon {
     font-size: 3rem;
     margin-bottom: 1rem;
     opacity: 0.5;
   }

   .empty-text {
     font-size: 1rem;
     margin-bottom: 0.5rem;
     color: var(--text-primary);  /* Dracula Foreground */
   }

   .empty-hint {
     font-size: 0.875rem;
     color: var(--text-secondary);  /* Dracula Comment */
   }
   ```

3. Add zoom indicator:
   ```html
   <div id="zoom-indicator" class="zoom-indicator">
     <span class="zoom-value">4x</span>
   </div>
   ```

4. Show dimensions when rendered:
   ```css
   .zoom-indicator {
     position: absolute;
     bottom: 0.5rem;
     right: 0.5rem;
     padding: 0.25rem 0.5rem;
     background: rgba(0, 0, 0, 0.7);
     border-radius: 4px;
     font-size: 0.75rem;
     font-family: monospace;
     color: var(--text-secondary);
   }
   ```

**Verification:**
- Empty state shows helpful message
- Zoom/scale indicator shows current scale
- Preview feels polished

**Dependencies:** Tasks 11.1, 11.4

---

### Task 11.8: Animations & Micro-interactions

**Wave:** 5 (parallel with 11.9)

Add delightful micro-interactions throughout, with reduced motion support.

**Deliverables:**

1. **Respect `prefers-reduced-motion`** (CRITICAL):
   ```css
   @media (prefers-reduced-motion: reduce) {
     *,
     *::before,
     *::after {
       animation-duration: 0.01ms !important;
       animation-iteration-count: 1 !important;
       transition-duration: 0.01ms !important;
     }

     .loading-spinner {
       animation: none;
       /* Show static indicator instead */
       border-style: dotted;
     }
   }
   ```

2. Button interactions:
   ```css
   button {
     transition: all 0.15s ease;
   }

   button:active {
     transform: scale(0.95);
   }

   /* Ripple effect on click */
   button.ripple {
     position: relative;
     overflow: hidden;
   }

   button.ripple::after {
     content: "";
     position: absolute;
     inset: 0;
     background: radial-gradient(circle, rgba(255,255,255,0.3) 0%, transparent 70%);
     transform: scale(0);
     opacity: 0;
   }

   button.ripple:active::after {
     transform: scale(2);
     opacity: 1;
     transition: transform 0.3s, opacity 0.3s;
   }
   ```

2. Success feedback animations:
   ```css
   .copy-success {
     animation: successPop 0.3s ease;
   }

   @keyframes successPop {
     0% { transform: scale(1); }
     50% { transform: scale(1.1); }
     100% { transform: scale(1); }
   }
   ```

3. **Cross-platform keyboard shortcut hints**:
   ```html
   <span class="kbd-hint">
     <kbd class="kbd-mac">⌘</kbd>
     <kbd class="kbd-win">Ctrl</kbd>
     + <kbd>Enter</kbd> to render
   </span>
   ```

   ```css
   /* Show correct shortcut based on platform */
   .kbd-mac { display: none; }
   .kbd-win { display: inline-block; }

   @supports (-webkit-touch-callout: none) {
     /* iOS/macOS */
     .kbd-mac { display: inline-block; }
     .kbd-win { display: none; }
   }
   ```

   ```typescript
   // Better: detect at runtime
   const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
   document.querySelectorAll('.kbd-mac').forEach(el => {
     (el as HTMLElement).style.display = isMac ? 'inline-block' : 'none';
   });
   document.querySelectorAll('.kbd-win').forEach(el => {
     (el as HTMLElement).style.display = isMac ? 'none' : 'inline-block';
   });
   ```

   ```css
   kbd {
     display: inline-block;
     padding: 0.125rem 0.375rem;
     font-size: 0.75rem;
     font-family: inherit;
     background: var(--bg-panel);  /* Dracula Selection */
     border: 1px solid var(--border);
     border-radius: 3px;
     box-shadow: 0 1px 0 var(--border);
     color: var(--text-primary);
   }

   .kbd-hint {
     font-size: 0.75rem;
     color: var(--text-secondary);  /* Dracula Comment */
   }
   ```

4. Panel focus states (Dracula themed):
   ```css
   .panel:focus-within {
     border-color: var(--accent-cyan);  /* Dracula Cyan #8be9fd */
     box-shadow: 0 0 0 2px rgba(139, 233, 253, 0.2);
   }
   ```

5. Smooth transitions between states:
   ```css
   .preview-canvas canvas {
     transition: opacity 0.2s ease;
   }

   .preview-canvas.rendering canvas {
     opacity: 0.5;
   }
   ```

**Verification:**
- All buttons have satisfying click feedback
- Copy/download show success animation
- Focus states are clear and helpful
- Keyboard hints are visible but unobtrusive

**Dependencies:** Tasks 11.1-11.7

---

### Task 11.9: Accessibility (WCAG 2.1 AA)

**Wave:** 5 (parallel with 11.8)

Ensure the site meets WCAG 2.1 AA accessibility standards.

**Current accessibility failures:**
- Color-only status indication
- Missing form labels
- No skip link
- No ARIA live regions for dynamic updates
- Untested color contrast
- No visible focus indicators

**Deliverables:**

1. **Skip link for keyboard/screen reader users**:
   ```html
   <a href="#editor" class="skip-link">Skip to editor</a>
   ```

   ```css
   .skip-link {
     position: absolute;
     top: -40px;
     left: 0;
     padding: 0.5rem 1rem;
     background: var(--accent-purple);
     color: var(--bg-primary);
     z-index: 1001;
     transition: top 0.2s;
   }

   .skip-link:focus {
     top: 0;
   }
   ```

2. **Proper form labels** (not just placeholders):
   ```html
   <label for="editor" class="visually-hidden">JSONL Editor</label>
   <textarea id="editor" ...></textarea>
   ```

   ```css
   .visually-hidden {
     position: absolute;
     width: 1px;
     height: 1px;
     padding: 0;
     margin: -1px;
     overflow: hidden;
     clip: rect(0, 0, 0, 0);
     white-space: nowrap;
     border: 0;
   }
   ```

3. **ARIA live regions for status updates**:
   ```html
   <div id="preview-status" class="preview-status" role="status" aria-live="polite">
     <span class="status-icon" aria-hidden="true"></span>
     <span class="status-text"></span>
   </div>
   ```

   ```typescript
   // Announce to screen readers
   function showStatus(type: string, message: string): void {
     const status = document.getElementById('preview-status');
     status.className = `preview-status ${type}`;
     status.querySelector('.status-text').textContent = message;
     // aria-live="polite" will announce this automatically
   }
   ```

4. **Don't rely on color alone** - add text/icons:
   ```css
   /* Error state: red color + "Error:" text prefix */
   .preview-status.error .status-text::before {
     content: "Error: ";
   }

   /* Success: green + text description */
   .preview-status.success .status-text::before {
     content: "Rendered: ";
   }
   ```

5. **Visible focus indicators** (not just outlines):
   ```css
   /* High-visibility focus ring */
   :focus-visible {
     outline: 2px solid var(--accent-cyan);
     outline-offset: 2px;
   }

   /* Remove default outline when using focus-visible */
   :focus:not(:focus-visible) {
     outline: none;
   }

   /* Button focus state */
   button:focus-visible {
     outline: 2px solid var(--accent-cyan);
     outline-offset: 2px;
     box-shadow: 0 0 0 4px rgba(139, 233, 253, 0.3);
   }

   /* Gallery item focus */
   .gallery-item:focus-visible {
     border-color: var(--accent-cyan);
     outline: none;
     box-shadow: 0 0 0 3px rgba(139, 233, 253, 0.4);
   }
   ```

6. **Color contrast verification** (Dracula palette):
   ```
   Required: 4.5:1 for normal text, 3:1 for large text/UI components

   ✓ #f8f8f2 on #282a36 = 11.4:1 (passes AAA)
   ✓ #f8f8f2 on #44475a = 7.1:1 (passes AAA)
   ⚠ #6272a4 on #282a36 = 3.5:1 (passes AA for large text only)
   ✓ #bd93f9 on #282a36 = 5.3:1 (passes AA)
   ✓ #50fa7b on #282a36 = 8.5:1 (passes AAA)
   ✓ #ff5555 on #282a36 = 4.9:1 (passes AA)

   Fix: Use #f8f8f2 for important text, reserve #6272a4 for large/decorative text only
   ```

7. **Decorative elements marked as such**:
   ```html
   <!-- Logo pixels are decorative -->
   <div class="logo" aria-hidden="true">
     <span class="logo-pixel p1"></span>
     ...
   </div>

   <!-- But brand text is not -->
   <div class="brand-text">
     <h1>Pixel<span class="accent">src</span></h1>
     ...
   </div>
   ```

8. **Keyboard navigation for gallery**:
   ```typescript
   // Arrow key navigation within gallery
   galleryContainer.addEventListener('keydown', (e) => {
     const items = Array.from(galleryContainer.querySelectorAll('.gallery-item'));
     const current = document.activeElement as HTMLElement;
     const index = items.indexOf(current);

     if (index === -1) return;

     let next: HTMLElement | null = null;
     if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
       next = items[index + 1] || items[0];
     } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
       next = items[index - 1] || items[items.length - 1];
     }

     if (next) {
       e.preventDefault();
       next.focus();
     }
   });
   ```

9. **Accessible button labels**:
   ```html
   <!-- Export buttons need clear labels -->
   <button class="export-btn export-download-btn" aria-label="Download PNG">
     Download
   </button>
   <button class="export-btn export-copy-btn" aria-label="Copy image to clipboard">
     Copy
   </button>

   <!-- Render button -->
   <button id="render-btn" aria-describedby="render-hint">
     Render
   </button>
   <span id="render-hint" class="visually-hidden">
     Press to render the JSONL content. Keyboard shortcut: Command or Control plus Enter.
   </span>
   ```

**Verification:**
```bash
# Automated testing
npx axe-cli https://scbrown.github.io/pixelsrc/

# Manual testing checklist:
# [ ] Navigate entire page using only Tab key
# [ ] All interactive elements have visible focus state
# [ ] Screen reader announces status changes
# [ ] Skip link works and is visible on focus
# [ ] Gallery navigable with arrow keys
# [ ] Error messages include text, not just color
# [ ] All images have alt text or are marked decorative
```

**Dependencies:** Tasks 11.1-11.7

---

## Verification Summary

```bash
# After all tasks complete:

# 1. Site loads and works
open https://scbrown.github.io/pixelsrc/
# Verify: Page loads without errors
# Verify: WASM initializes (console log)
# Verify: Gallery shows thumbnails

# 2. Loading states work
# Hard refresh - see loading spinner
# Type invalid JSONL - see error state
# Type valid JSONL - see success state

# 3. Visual polish
# Verify: Colors are cohesive and appealing
# Verify: Logo animation plays
# Verify: Buttons have hover/active states
# Verify: Empty state shows helpful text

# 4. Functionality
# Verify: Editor works
# Verify: Preview updates
# Verify: Export (download/copy) works
# Verify: URL sharing works

# 5. Accessibility
# [ ] Tab through entire page - all elements reachable
# [ ] Skip link visible on Tab, jumps to editor
# [ ] Focus indicators visible on all interactive elements
# [ ] Gallery navigable with arrow keys
# [ ] Screen reader announces status changes (test with VoiceOver/NVDA)
# [ ] Error messages include text explanation, not just color
# [ ] Reduced motion: disable animations in system prefs, verify no motion
# [ ] Run axe DevTools - no critical/serious violations

# 6. Cross-platform
# [ ] Windows: Ctrl+Enter shortcut shown and works
# [ ] Mac: Cmd+Enter shortcut shown and works
# [ ] Linux: Ctrl+Enter shortcut shown and works
```

---

## Success Criteria

1. **Site works**: No 404s, WASM loads, rendering functions
2. **Loading feedback**: User always knows what's happening
3. **Visual appeal**: Site feels polished, not generic
4. **Delightful interactions**: Micro-interactions make it feel alive
5. **Professional branding**: Looks like a real product
6. **Accessible**: Passes WCAG 2.1 AA, keyboard navigable, screen reader friendly
7. **User-friendly errors**: All error messages are actionable, not technical dumps
8. **Cross-platform**: Works on Mac, Windows, Linux; keyboard shortcuts adapt

---

## Future Considerations

Not in scope for Phase 11, but potential future work:

| Feature | Notes |
|---------|-------|
| Light mode toggle | Some users prefer light themes |
| Animation preview | Once Phase 3 (animation) is complete |
| Fullscreen preview | For presentations/sharing |
| Sound effects | Retro beeps on actions (opt-in) |
| Onboarding tour | First-time user walkthrough |
