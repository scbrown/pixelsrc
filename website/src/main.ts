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
let galleryContainer: HTMLDivElement;

// State
let wasmReady = false;
let preview: Preview | null = null;
let gallery: Gallery | null = null;

async function initApp(): Promise<void> {
  // Get DOM elements
  editor = document.getElementById('editor') as HTMLTextAreaElement;
  renderBtn = document.getElementById('render-btn') as HTMLButtonElement;
  previewContainer = document.getElementById('preview-canvas') as HTMLDivElement;
  previewError = document.getElementById('preview-error') as HTMLDivElement;
  galleryContainer = document.getElementById('gallery') as HTMLDivElement;

  // Initialize WASM
  try {
    await init();
    wasmReady = true;
    console.log('WASM module initialized');
  } catch (err) {
    showError(`Failed to initialize WASM: ${err}`);
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

  // Set default content if editor is empty
  if (!editor.value.trim() && gallery) {
    const examples = gallery.getExamples();
    if (examples.length > 0) {
      editor.value = examples[0].jsonl;
    }
  }

  // Initial render
  handleRender();
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
    preview.render(jsonl);
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
    return;
  }

  // Use preview component for rendering (immediate, no debounce)
  const result = preview.renderImmediate(jsonl);

  if (result.success) {
    hideError();
    updateHash(jsonl);

    // Show warnings if any
    if (result.warnings.length > 0) {
      console.warn('Render warnings:', result.warnings);
    }
  } else if (result.error) {
    showError(result.error);
  }
}

function showError(message: string): void {
  previewError.textContent = message;
  previewError.classList.remove('hidden');
}

function hideError(): void {
  previewError.classList.add('hidden');
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
