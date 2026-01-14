/**
 * Export module for PNG download and clipboard copy functionality.
 *
 * Features:
 * - Download PNG with optional scale factor
 * - Copy to clipboard as PNG image
 * - Visual feedback on copy success
 */

import { render_to_png, render_to_rgba } from '@stiwi/pixelsrc-wasm';

export type ScaleFactor = 1 | 2 | 4 | 8;

export interface ExportOptions {
  /** Container for export controls */
  container: HTMLElement;
  /** Callback to get current JSONL content */
  getJsonl: () => string;
  /** Callback when export error occurs */
  onError?: (message: string) => void;
}

/**
 * Export component for PNG download and clipboard operations.
 */
export class Export {
  private container: HTMLElement;
  private getJsonl: () => string;
  private onError: (message: string) => void;

  private scaleSelect: HTMLSelectElement | null = null;
  private downloadBtn: HTMLButtonElement | null = null;
  private copyBtn: HTMLButtonElement | null = null;
  private copyFeedback: HTMLSpanElement | null = null;
  private feedbackTimeout: ReturnType<typeof setTimeout> | null = null;

  constructor(options: ExportOptions) {
    this.container = options.container;
    this.getJsonl = options.getJsonl;
    this.onError = options.onError ?? console.error;

    this.render();
    this.attachEventListeners();
  }

  /**
   * Render the export controls into the container.
   */
  private render(): void {
    this.container.innerHTML = `
      <div class="export-controls">
        <label class="export-scale-label">
          Scale:
          <select class="export-scale-select">
            <option value="1">1x</option>
            <option value="2">2x</option>
            <option value="4" selected>4x</option>
            <option value="8">8x</option>
          </select>
        </label>
        <button class="export-btn export-download-btn" title="Download PNG">
          Download PNG
        </button>
        <button class="export-btn export-copy-btn" title="Copy to Clipboard">
          Copy
        </button>
        <span class="export-copy-feedback hidden">Copied!</span>
      </div>
    `;

    this.scaleSelect = this.container.querySelector('.export-scale-select');
    this.downloadBtn = this.container.querySelector('.export-download-btn');
    this.copyBtn = this.container.querySelector('.export-copy-btn');
    this.copyFeedback = this.container.querySelector('.export-copy-feedback');
  }

  /**
   * Attach event listeners to buttons.
   */
  private attachEventListeners(): void {
    this.downloadBtn?.addEventListener('click', () => this.handleDownload());
    this.copyBtn?.addEventListener('click', () => this.handleCopy());
  }

  /**
   * Get the current scale factor from the selector.
   */
  getScale(): ScaleFactor {
    const value = parseInt(this.scaleSelect?.value ?? '4', 10);
    return (value === 1 || value === 2 || value === 4 || value === 8) ? value : 4;
  }

  /**
   * Set the scale factor.
   */
  setScale(scale: ScaleFactor): void {
    if (this.scaleSelect) {
      this.scaleSelect.value = String(scale);
    }
  }

  /**
   * Handle PNG download.
   */
  private async handleDownload(): Promise<void> {
    const jsonl = this.getJsonl();
    if (!jsonl.trim()) {
      this.onError('No content to export');
      return;
    }

    try {
      const scale = this.getScale();
      const pngBytes = this.renderScaledPng(jsonl, scale);

      if (pngBytes.length === 0) {
        this.onError('No sprites to export');
        return;
      }

      // Create blob and download
      const blob = new Blob([new Uint8Array(pngBytes)], { type: 'image/png' });
      const url = URL.createObjectURL(blob);

      const link = document.createElement('a');
      link.href = url;
      link.download = `pixelsrc-${scale}x.png`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      URL.revokeObjectURL(url);
    } catch (err) {
      this.onError(`Download failed: ${err}`);
    }
  }

  /**
   * Handle copy to clipboard.
   */
  private async handleCopy(): Promise<void> {
    const jsonl = this.getJsonl();
    if (!jsonl.trim()) {
      this.onError('No content to copy');
      return;
    }

    try {
      const scale = this.getScale();
      const pngBytes = this.renderScaledPng(jsonl, scale);

      if (pngBytes.length === 0) {
        this.onError('No sprites to copy');
        return;
      }

      // Create blob for clipboard
      const blob = new Blob([new Uint8Array(pngBytes)], { type: 'image/png' });

      await navigator.clipboard.write([
        new ClipboardItem({
          'image/png': blob,
        }),
      ]);

      this.showCopyFeedback();
    } catch (err) {
      this.onError(`Copy failed: ${err}`);
    }
  }

  /**
   * Render JSONL to PNG with scaling.
   */
  private renderScaledPng(jsonl: string, scale: ScaleFactor): Uint8Array {
    if (scale === 1) {
      // Use direct PNG rendering for 1x scale
      const bytes = render_to_png(jsonl);
      return new Uint8Array(bytes);
    }

    // For other scales, render to RGBA and scale up
    const result = render_to_rgba(jsonl);
    if (result.width === 0 || result.height === 0) {
      return new Uint8Array(0);
    }

    // Create canvas at original size
    const srcCanvas = document.createElement('canvas');
    srcCanvas.width = result.width;
    srcCanvas.height = result.height;
    const srcCtx = srcCanvas.getContext('2d')!;

    // Draw RGBA data
    const imageData = new ImageData(
      new Uint8ClampedArray(result.pixels),
      result.width,
      result.height
    );
    srcCtx.putImageData(imageData, 0, 0);

    // Create scaled canvas
    const scaledCanvas = document.createElement('canvas');
    scaledCanvas.width = result.width * scale;
    scaledCanvas.height = result.height * scale;
    const scaledCtx = scaledCanvas.getContext('2d')!;

    // Disable smoothing for crisp pixels
    scaledCtx.imageSmoothingEnabled = false;

    // Draw scaled
    scaledCtx.drawImage(
      srcCanvas,
      0, 0, result.width, result.height,
      0, 0, scaledCanvas.width, scaledCanvas.height
    );

    // Convert to PNG blob synchronously using toDataURL
    const dataUrl = scaledCanvas.toDataURL('image/png');
    const base64 = dataUrl.split(',')[1];
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }

    return bytes;
  }

  /**
   * Show "Copied!" feedback briefly.
   */
  private showCopyFeedback(): void {
    if (!this.copyFeedback) return;

    // Clear any existing timeout
    if (this.feedbackTimeout) {
      clearTimeout(this.feedbackTimeout);
    }

    // Show feedback
    this.copyFeedback.classList.remove('hidden');
    this.copyFeedback.classList.add('visible');

    // Hide after 2 seconds
    this.feedbackTimeout = setTimeout(() => {
      this.copyFeedback?.classList.add('hidden');
      this.copyFeedback?.classList.remove('visible');
    }, 2000);
  }

  /**
   * Destroy the export component.
   */
  destroy(): void {
    if (this.feedbackTimeout) {
      clearTimeout(this.feedbackTimeout);
    }
    this.container.innerHTML = '';
  }
}
