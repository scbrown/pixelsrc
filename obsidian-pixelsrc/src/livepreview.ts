import {
  ViewPlugin,
  ViewUpdate,
  WidgetType,
  Decoration,
  DecorationSet,
  EditorView,
} from '@codemirror/view';
import type PixelsrcPlugin from './main';
import { renderToPng } from './renderer';

/**
 * Widget that displays a rendered pixelsrc sprite image.
 */
class PixelsrcWidget extends WidgetType {
  private source: string;
  private plugin: PixelsrcPlugin;

  constructor(source: string, plugin: PixelsrcPlugin) {
    super();
    this.source = source;
    this.plugin = plugin;
  }

  toDOM(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'pixelsrc-widget';

    if (this.plugin.settings.showTransparency) {
      container.classList.add('show-transparency');
    }

    if (!this.plugin.isWasmReady()) {
      container.classList.add('pixelsrc-error');
      const error = this.plugin.getWasmError();
      container.textContent = error
        ? `WASM error: ${error.message}`
        : 'WASM not ready';
      return container;
    }

    try {
      const pngData = renderToPng(this.source);
      // Create a copy of the buffer to ensure correct type for Blob
      const blob = new Blob([new Uint8Array(pngData)], { type: 'image/png' });
      const url = URL.createObjectURL(blob);

      const img = document.createElement('img');
      img.className = 'pixelsrc-image';
      img.src = url;

      // Apply scale from settings
      const scale = this.plugin.settings.defaultScale;
      img.style.imageRendering = 'pixelated';

      // Clean up blob URL when image loads or errors
      img.onload = () => {
        // Scale the image dimensions
        img.style.width = `${img.naturalWidth * scale}px`;
        img.style.height = `${img.naturalHeight * scale}px`;
      };
      img.onerror = () => URL.revokeObjectURL(url);

      container.appendChild(img);
    } catch (error) {
      container.classList.add('pixelsrc-error');
      container.textContent =
        error instanceof Error ? error.message : String(error);
    }

    return container;
  }

  eq(other: PixelsrcWidget): boolean {
    return this.source === other.source;
  }
}

/**
 * Find pixelsrc code blocks in the document and create decorations.
 */
function findCodeBlocks(view: EditorView, plugin: PixelsrcPlugin): DecorationSet {
  const decorations: Array<{ from: number; widget: WidgetType }> = [];
  const doc = view.state.doc;

  // Use regex to find fenced code blocks with pixelsrc or pxl language
  const text = doc.toString();
  const codeBlockRegex = /```(?:pixelsrc|pxl)\s*\n([\s\S]*?)```/g;

  let match;
  while ((match = codeBlockRegex.exec(text)) !== null) {
    const source = match[1].trim();
    const endPos = match.index + match[0].length;

    // Find the line position after the code block
    const line = doc.lineAt(endPos);
    const pos = line.to;

    decorations.push({
      from: pos,
      widget: new PixelsrcWidget(source, plugin),
    });
  }

  // Sort by position and create decoration set
  decorations.sort((a, b) => a.from - b.from);
  return Decoration.set(
    decorations.map((d) => Decoration.widget({ widget: d.widget, side: 1 }).range(d.from))
  );
}

/**
 * Create the live preview ViewPlugin.
 */
function createLivePreviewPlugin(plugin: PixelsrcPlugin) {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = findCodeBlocks(view, plugin);
      }

      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = findCodeBlocks(update.view, plugin);
        }
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );
}

/**
 * Register the live preview extension with the plugin.
 */
export function registerLivePreviewExtension(plugin: PixelsrcPlugin) {
  if (!plugin.settings.enableLivePreview) {
    console.log('PixelSrc: Live preview disabled in settings');
    return;
  }

  plugin.registerEditorExtension(createLivePreviewPlugin(plugin));
  console.log('PixelSrc: Live preview extension registered');
}
