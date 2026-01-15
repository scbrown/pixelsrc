import init from '@stiwi/pixelsrc-wasm';
import LZString from 'lz-string';
import { Preview } from './preview';
import { Gallery } from './gallery';
import { Export } from './export';

// DOM Elements
let editor: HTMLTextAreaElement;
let renderBtn: HTMLButtonElement;
let previewContainer: HTMLDivElement;
let previewError: HTMLDivElement;
let previewStatus: HTMLDivElement;
let previewEmpty: HTMLDivElement;
let zoomIndicator: HTMLDivElement;
let zoomValue: HTMLSpanElement;
let loadFirstExampleBtn: HTMLButtonElement;
let galleryContainer: HTMLDivElement;

// State
let wasmReady = false;
let preview: Preview | null = null;
let gallery: Gallery | null = null;

async function initApp(): Promise<void> {
  const overlay = document.getElementById('loading-overlay');

  // Get DOM elements
  editor = document.getElementById('editor') as HTMLTextAreaElement;
  renderBtn = document.getElementById('render-btn') as HTMLButtonElement;
  previewContainer = document.getElementById('preview-canvas') as HTMLDivElement;
  previewError = document.getElementById('preview-error') as HTMLDivElement;
  previewStatus = document.getElementById('preview-status') as HTMLDivElement;
  previewEmpty = document.getElementById('preview-empty') as HTMLDivElement;
  zoomIndicator = document.getElementById('zoom-indicator') as HTMLDivElement;
  zoomValue = zoomIndicator.querySelector('.zoom-value') as HTMLSpanElement;
  loadFirstExampleBtn = document.getElementById('load-first-example') as HTMLButtonElement;
  galleryContainer = document.getElementById('gallery') as HTMLDivElement;

  // Initialize WASM
  try {
    await init();
    wasmReady = true;
    console.log('WASM module initialized');
  } catch (err) {
    // Show user-friendly error in overlay
    const loadingText = overlay?.querySelector('.loading-text');
    const loadingContent = overlay?.querySelector('.loading-content');
    const loadingSpinner = overlay?.querySelector('.loading-spinner');

    if (loadingText && loadingContent) {
      loadingText.textContent = 'Failed to load the editor';
      loadingText.classList.add('error');

      // Hide spinner
      if (loadingSpinner) {
        (loadingSpinner as HTMLElement).style.display = 'none';
      }

      // Add helpful retry/info
      const helpText = document.createElement('p');
      helpText.className = 'loading-help';
      helpText.innerHTML = `
        <button onclick="location.reload()">Try again</button>
        <br><small>If this persists, try a different browser.</small>
      `;
      loadingContent.appendChild(helpText);
    }

    console.error('WASM init failed:', err);
    renderBtn.disabled = true;
    return;
  }

  // Initialize preview component
  preview = new Preview({
    container: previewContainer,
    debounceMs: 100,
  });

  // Initialize export component
  const exportContainer = document.getElementById('export-controls') as HTMLDivElement;
  new Export({
    container: exportContainer,
    getJsonl: () => editor.value.trim(),
    onError: showError,
  });

  // Set up event listeners
  renderBtn.addEventListener('click', handleRender);
  editor.addEventListener('keydown', handleEditorKeydown);
  editor.addEventListener('input', handleEditorInput);
  loadFirstExampleBtn.addEventListener('click', loadFirstExample);

  // Load from URL hash if present
  loadFromHash();

  // Initialize gallery
  gallery = new Gallery({
    container: galleryContainer,
    onSelect: (jsonl: string) => {
      editor.value = jsonl;
      handleRender();
    },
  });
  await gallery.loadExamples();

  // If editor has content (from URL hash), render it
  // Otherwise, show the empty state for onboarding
  if (editor.value.trim()) {
    handleRender();
  } else {
    showEmptyState();
  }

  // Hide loading overlay with animation
  if (overlay) {
    // Update aria-live region to announce ready state
    const loadingText = overlay.querySelector('.loading-text');
    if (loadingText) {
      loadingText.textContent = 'Ready';
    }
  }
  overlay?.classList.add('hidden');
  setTimeout(() => overlay?.remove(), 300);
}

function handleEditorKeydown(e: KeyboardEvent): void {
  // Ctrl/Cmd + Enter to render
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault();
    handleRender();
  }
}

function handleEditorInput(): void {
  // Debounced live preview as user types
  if (!wasmReady || !preview) return;

  const jsonl = editor.value.trim();
  if (jsonl) {
    hideEmptyState();
    preview.render(jsonl);
    // Update zoom indicator after debounced render completes
    setTimeout(() => {
      const result = preview?.getRenderResult();
      if (result && result.width > 0) {
        const scale = preview?.getScale() || 1;
        updateZoomIndicator(result.width, result.height, scale);
      }
    }, 150); // Slightly longer than debounce time
  } else {
    showEmptyState();
  }
}

function handleRender(): void {
  if (!wasmReady || !preview) {
    showError('WASM module not ready');
    return;
  }

  const jsonl = editor.value.trim();
  if (!jsonl) {
    showError('Please enter some JSONL content');
    showEmptyState();
    return;
  }

  // Show rendering state
  showStatus('rendering', 'Rendering...');

  // Use preview component for rendering (immediate, no debounce)
  const result = preview.renderImmediate(jsonl);

  if (result.success) {
    hideError();
    hideEmptyState();
    updateZoomIndicator(result.width, result.height, result.scale);
    updateHash(jsonl);

    // Show success with dimensions
    showStatus('success', `${result.width}×${result.height}`);
    hideStatusAfter(2000);

    // Show warnings if any
    if (result.warnings.length > 0) {
      console.warn('Render warnings:', result.warnings);
    }
  } else if (result.error) {
    // Show user-friendly error in status
    showStatus('error', friendlyError(result.error));
    showError(result.error);
    showEmptyState();
  }
}

function loadFirstExample(): void {
  if (!gallery) return;

  const examples = gallery.getExamples();
  if (examples.length > 0) {
    editor.value = examples[0].jsonl;
    handleRender();
  }
}

function showEmptyState(): void {
  if (previewEmpty) {
    previewEmpty.classList.remove('hidden');
  }
  if (zoomIndicator) {
    zoomIndicator.classList.add('hidden');
  }
}

function hideEmptyState(): void {
  if (previewEmpty) {
    previewEmpty.classList.add('hidden');
  }
}

function updateZoomIndicator(width: number, height: number, scale: number): void {
  if (!zoomIndicator || !zoomValue) return;

  zoomValue.textContent = `${width}×${height} @ ${scale}x`;
  zoomIndicator.classList.remove('hidden');
}

function showError(message: string): void {
  previewError.textContent = message;
  previewError.classList.remove('hidden');
}

function hideError(): void {
  previewError.classList.add('hidden');
}

// Status indicator functions
let statusTimer: ReturnType<typeof setTimeout> | null = null;

function showStatus(type: 'rendering' | 'success' | 'error', message: string): void {
  if (statusTimer) {
    clearTimeout(statusTimer);
    statusTimer = null;
  }

  previewStatus.className = `preview-status ${type}`;
  previewStatus.setAttribute('aria-busy', type === 'rendering' ? 'true' : 'false');
  const textEl = previewStatus.querySelector('.status-text');
  if (textEl) {
    // Prefix message for screen readers based on type
    const srPrefix = type === 'rendering' ? 'Rendering: ' : type === 'success' ? 'Success: ' : 'Error: ';
    textEl.textContent = message;
    // Set aria-label with full context for screen readers
    previewStatus.setAttribute('aria-label', srPrefix + message);
  }
}

function hideStatus(): void {
  previewStatus.classList.add('hidden');
}

function hideStatusAfter(ms: number): void {
  if (statusTimer) {
    clearTimeout(statusTimer);
  }
  statusTimer = setTimeout(() => {
    hideStatus();
    statusTimer = null;
  }, ms);
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

function updateHash(jsonl: string): void {
  const compressed = LZString.compressToEncodedURIComponent(jsonl);
  window.history.replaceState(null, '', `#${compressed}`);
}

function loadFromHash(): void {
  const hash = window.location.hash.slice(1);
  if (hash) {
    try {
      const decompressed = LZString.decompressFromEncodedURIComponent(hash);
      if (decompressed) {
        editor.value = decompressed;
      }
    } catch (err) {
      console.warn('Failed to decompress hash:', err);
    }
  }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', initApp);
