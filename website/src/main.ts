import init, { render_to_png, validate } from '@pixelsrc/wasm';
import LZString from 'lz-string';
import { createEditor, Editor } from './editor';
import { Export } from './export';
import { Gallery } from './gallery';

// DOM Elements
let editorContainer: HTMLDivElement;
let editor: Editor;
let renderBtn: HTMLButtonElement;
let previewCanvas: HTMLDivElement;
let previewError: HTMLDivElement;
let galleryContainer: HTMLDivElement;
let gallery: Gallery;

// State
let wasmReady = false;

async function initApp(): Promise<void> {
  // Get DOM elements
  editorContainer = document.getElementById('editor-container') as HTMLDivElement;
  renderBtn = document.getElementById('render-btn') as HTMLButtonElement;
  previewCanvas = document.getElementById('preview-canvas') as HTMLDivElement;
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

  // Initialize Gallery component (handles loading external example files)
  gallery = new Gallery({
    container: galleryContainer,
    onSelect: (jsonl: string) => {
      editor.setValue(jsonl);
      handleRender();
    },
  });
  await gallery.loadExamples();

  // Determine initial content
  let initialContent = '';
  const hash = window.location.hash.slice(1);
  if (hash) {
    try {
      const decompressed = LZString.decompressFromEncodedURIComponent(hash);
      if (decompressed) {
        initialContent = decompressed;
      }
    } catch (err) {
      console.warn('Failed to decompress hash:', err);
    }
  }
  if (!initialContent) {
    // Use first example from gallery if available
    const examples = gallery.getExamples();
    if (examples.length > 0) {
      initialContent = examples[0].jsonl;
    }
  }

  // Initialize CodeMirror editor
  editor = createEditor(editorContainer, initialContent);

  // Initialize export component
  const exportContainer = document.getElementById('export-controls') as HTMLDivElement;
  new Export({
    container: exportContainer,
    getJsonl: () => editor.value.trim(),
    onError: showError,
  });

  // Set up event listeners
  renderBtn.addEventListener('click', handleRender);
}

function handleRender(): void {
  if (!wasmReady) {
    showError('WASM module not ready');
    return;
  }

  const jsonl = editor.getValue().trim();
  if (!jsonl) {
    showError('Please enter some JSONL content');
    return;
  }

  // Validate first
  try {
    const errors = validate(jsonl);
    if (errors.length > 0) {
      showError(errors.join('\n'));
      return;
    }
  } catch (err) {
    showError(`Validation error: ${err}`);
    return;
  }

  // Render
  try {
    const pngBytes = render_to_png(jsonl);
    displayImage(pngBytes);
    hideError();
    updateHash(jsonl);
  } catch (err) {
    showError(`Render error: ${err}`);
  }
}

function displayImage(pngBytes: Uint8Array): void {
  const blob = new Blob([pngBytes.slice()], { type: 'image/png' });
  const url = URL.createObjectURL(blob);

  // Clear previous content
  previewCanvas.innerHTML = '';

  const img = document.createElement('img');
  img.src = url;
  img.alt = 'Rendered sprite';
  img.onload = () => URL.revokeObjectURL(url);

  previewCanvas.appendChild(img);
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

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', initApp);
