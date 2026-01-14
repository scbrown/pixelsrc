import { MarkdownPostProcessorContext, Menu } from 'obsidian';
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
  _ctx: MarkdownPostProcessorContext,
  plugin: PixelsrcPlugin
) {
  const container = el.createDiv({ cls: 'pixelsrc-container' });

  // Add transparency background class if enabled
  if (plugin.settings.showTransparency) {
    container.addClass('show-transparency');
  }

  // Check if WASM is ready
  if (!plugin.isWasmReady()) {
    const error = plugin.getWasmError();
    container.createDiv({
      cls: 'pixelsrc-error',
      text: error
        ? `WASM error: ${error.message}`
        : 'WASM not initialized yet. Please wait...',
    });
    return;
  }

  // Validate first
  let validationMessages: string[] = [];
  try {
    validationMessages = validateJsonl(source);
  } catch (error) {
    container.createDiv({
      cls: 'pixelsrc-error',
      text: `Validation failed: ${error instanceof Error ? error.message : String(error)}`,
    });
    return;
  }

  const errors = validationMessages.filter((m) => m.startsWith('Error:'));
  const warnings = validationMessages.filter((m) => m.startsWith('Warning:'));

  if (errors.length > 0) {
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

    // Create image element - copy buffer to ensure correct type for Blob
    const blob = new Blob([new Uint8Array(pngBytes)], { type: 'image/png' });
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

    // Scale the image once loaded
    img.onload = () => {
      img.style.width = `${img.naturalWidth * scale}px`;
      img.style.height = `${img.naturalHeight * scale}px`;
    };

    // Show warnings if enabled
    if (warnings.length > 0 && plugin.settings.showWarnings) {
      const warnDiv = container.createDiv({ cls: 'pixelsrc-warnings' });
      for (const warn of warnings) {
        warnDiv.createEl('small', { text: warn });
        warnDiv.createEl('br');
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
        const removedNodes = Array.from(mutation.removedNodes);
        for (const node of removedNodes) {
          if (node === container || (node as Element).contains?.(container)) {
            URL.revokeObjectURL(url);
            observer.disconnect();
            return;
          }
        }
      }
    });

    // Observe parent for removal
    if (el.parentElement) {
      observer.observe(el.parentElement, { childList: true, subtree: true });
    }
  } catch (error) {
    console.error('PixelSrc render error:', error);
    container.createDiv({
      cls: 'pixelsrc-error',
      text: `Render failed: ${error instanceof Error ? error.message : String(error)}`,
    });
  }
}

function showContextMenu(
  e: MouseEvent,
  pngBytes: Uint8Array,
  plugin: PixelsrcPlugin
) {
  const menu = new Menu();

  menu.addItem((item) => {
    item
      .setTitle('Copy as PNG')
      .setIcon('copy')
      .onClick(async () => {
        try {
          const blob = new Blob([new Uint8Array(pngBytes)], { type: 'image/png' });
          await navigator.clipboard.write([
            new ClipboardItem({ 'image/png': blob }),
          ]);
        } catch (err) {
          console.error('Failed to copy:', err);
        }
      });
  });

  menu.showAtMouseEvent(e);
}
