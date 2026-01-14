import { MarkdownPostProcessorContext } from 'obsidian';
import type PixelsrcPlugin from './main';

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
  _plugin: PixelsrcPlugin
) {
  const container = el.createDiv({ cls: 'pixelsrc-container' });

  // Placeholder for Task 8.2 (WASM integration)
  // For now, just show the source as a placeholder
  container.createDiv({
    cls: 'pixelsrc-placeholder',
    text: `[PixelSrc sprite placeholder - WASM not yet integrated]\n${source.substring(0, 50)}...`,
  });
}
