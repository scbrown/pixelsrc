import init, { render_to_png, validate } from '@pixelsrc/wasm';
import LZString from 'lz-string';
import { createEditor, Editor } from './editor';

// DOM Elements
let editorContainer: HTMLDivElement;
let editor: Editor;
let renderBtn: HTMLButtonElement;
let previewCanvas: HTMLDivElement;
let previewError: HTMLDivElement;
let gallery: HTMLDivElement;

// State
let wasmReady = false;

// Example sprites for the gallery
const EXAMPLE_SPRITES = [
  {
    name: 'heart',
    jsonl: `{"type":"sprite","name":"heart","palette":{"{_}":"#00000000","{r}":"#FF0000"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}`,
  },
  {
    name: 'star',
    jsonl: `{"type":"sprite","name":"star","palette":{"{_}":"#00000000","{y}":"#FFD700"},"grid":["{_}{_}{y}{_}{_}","{_}{y}{y}{y}{_}","{y}{y}{y}{y}{y}","{_}{y}{y}{y}{_}","{y}{_}{y}{_}{y}"]}`,
  },
  {
    name: 'smiley',
    jsonl: `{"type":"sprite","name":"smiley","palette":{"{_}":"#00000000","{y}":"#FFFF00","{b}":"#000000"},"grid":["{_}{y}{y}{y}{y}{y}{_}","{y}{y}{b}{y}{b}{y}{y}","{y}{y}{y}{y}{y}{y}{y}","{y}{b}{y}{y}{y}{b}{y}","{y}{y}{b}{b}{b}{y}{y}","{_}{y}{y}{y}{y}{y}{_}"]}`,
  },
];

async function initApp(): Promise<void> {
  // Get DOM elements
  editorContainer = document.getElementById('editor-container') as HTMLDivElement;
  renderBtn = document.getElementById('render-btn') as HTMLButtonElement;
  previewCanvas = document.getElementById('preview-canvas') as HTMLDivElement;
  previewError = document.getElementById('preview-error') as HTMLDivElement;
  gallery = document.getElementById('gallery') as HTMLDivElement;

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
    initialContent = EXAMPLE_SPRITES[0].jsonl;
  }

  // Initialize CodeMirror editor
  editor = createEditor(editorContainer, initialContent);

  // Set up event listeners
  renderBtn.addEventListener('click', handleRender);

  // Initialize gallery
  initGallery();
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

function initGallery(): void {
  gallery.innerHTML = '';

  for (const example of EXAMPLE_SPRITES) {
    const item = document.createElement('div');
    item.className = 'gallery-item';
    item.title = example.name;

    // Render thumbnail
    try {
      const pngBytes = render_to_png(example.jsonl);
      const blob = new Blob([pngBytes.slice()], { type: 'image/png' });
      const url = URL.createObjectURL(blob);

      const img = document.createElement('img');
      img.src = url;
      img.alt = example.name;

      item.appendChild(img);
    } catch (err) {
      item.textContent = example.name;
    }

    // Click to load
    item.addEventListener('click', () => {
      editor.setValue(example.jsonl);
      handleRender();
    });

    gallery.appendChild(item);
  }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', initApp);
