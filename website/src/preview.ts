/**
 * Preview component for rendering pixelsrc JSONL to canvas.
 *
 * Features:
 * - Canvas-based rendering with auto-scaling
 * - Integer scaling for crisp pixel art (nearest-neighbor)
 * - Checkered background for transparency
 * - Debounced rendering (100ms)
 */

import { render_to_rgba, validate, RenderResult } from '@stiwi/pixelsrc-wasm';

export interface PreviewOptions {
  /** Container element to render into */
  container: HTMLElement;
  /** Debounce delay in ms (default: 100) */
  debounceMs?: number;
  /** Minimum scale factor (default: 1) */
  minScale?: number;
  /** Maximum scale factor (default: 32) */
  maxScale?: number;
}

export interface RenderOutput {
  /** Whether rendering was successful */
  success: boolean;
  /** Error message if rendering failed */
  error?: string;
  /** Any warnings from rendering */
  warnings: string[];
  /** Width of rendered sprite in pixels */
  width: number;
  /** Height of rendered sprite in pixels */
  height: number;
  /** Scale factor used */
  scale: number;
}

/**
 * Canvas-based preview component for pixelsrc JSONL.
 */
export class Preview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private checkerCanvas: HTMLCanvasElement;
  private debounceMs: number;
  private minScale: number;
  private maxScale: number;
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private lastRenderResult: RenderResult | null = null;
  private currentScale: number = 1;

  constructor(options: PreviewOptions) {
    this.container = options.container;
    this.debounceMs = options.debounceMs ?? 100;
    this.minScale = options.minScale ?? 1;
    this.maxScale = options.maxScale ?? 32;

    // Create main canvas
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'preview-canvas';
    this.ctx = this.canvas.getContext('2d')!;

    // Disable image smoothing for crisp pixels
    this.ctx.imageSmoothingEnabled = false;

    // Create checker pattern for transparency
    this.checkerCanvas = this.createCheckerPattern();

    // Add to container
    this.container.innerHTML = '';
    this.container.appendChild(this.canvas);

    // Apply CSS for pixel art rendering
    this.canvas.style.imageRendering = 'pixelated';
    this.canvas.style.imageRendering = 'crisp-edges';
  }

  /**
   * Create a checkered pattern canvas for transparency background.
   */
  private createCheckerPattern(): HTMLCanvasElement {
    const size = 8; // Size of each checker square
    const canvas = document.createElement('canvas');
    canvas.width = size * 2;
    canvas.height = size * 2;
    const ctx = canvas.getContext('2d')!;

    // Light gray squares
    ctx.fillStyle = '#e0e0e0';
    ctx.fillRect(0, 0, size * 2, size * 2);

    // Dark gray squares
    ctx.fillStyle = '#c0c0c0';
    ctx.fillRect(0, 0, size, size);
    ctx.fillRect(size, size, size, size);

    return canvas;
  }

  /**
   * Calculate optimal integer scale factor for the container size.
   */
  private calculateScale(width: number, height: number): number {
    if (width === 0 || height === 0) return 1;

    const containerWidth = this.container.clientWidth - 32; // Account for padding
    const containerHeight = this.container.clientHeight - 32;

    const scaleX = Math.floor(containerWidth / width);
    const scaleY = Math.floor(containerHeight / height);

    let scale = Math.min(scaleX, scaleY);
    scale = Math.max(this.minScale, Math.min(this.maxScale, scale));

    return scale;
  }

  /**
   * Draw the checkered background pattern.
   */
  private drawCheckerBackground(width: number, height: number): void {
    const pattern = this.ctx.createPattern(this.checkerCanvas, 'repeat');
    if (pattern) {
      this.ctx.fillStyle = pattern;
      this.ctx.fillRect(0, 0, width, height);
    }
  }

  /**
   * Render JSONL to the canvas with debouncing.
   */
  render(jsonl: string): void {
    // Clear any pending debounced render
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    // Schedule new render
    this.debounceTimer = setTimeout(() => {
      this.renderImmediate(jsonl);
    }, this.debounceMs);
  }

  /**
   * Render JSONL to the canvas immediately (no debouncing).
   */
  renderImmediate(jsonl: string): RenderOutput {
    const output: RenderOutput = {
      success: false,
      warnings: [],
      width: 0,
      height: 0,
      scale: 1,
    };

    // Validate first
    try {
      const errors = validate(jsonl);
      if (errors.length > 0) {
        output.error = errors.join('\n');
        return output;
      }
    } catch (err) {
      output.error = `Validation error: ${err}`;
      return output;
    }

    // Render to RGBA
    try {
      const result = render_to_rgba(jsonl);
      this.lastRenderResult = result;

      output.warnings = result.warnings;
      output.width = result.width;
      output.height = result.height;

      if (result.width === 0 || result.height === 0) {
        output.error = 'No sprites found in input';
        return output;
      }

      // Calculate scale
      const scale = this.calculateScale(result.width, result.height);
      this.currentScale = scale;
      output.scale = scale;

      const canvasWidth = result.width * scale;
      const canvasHeight = result.height * scale;

      // Resize canvas
      this.canvas.width = canvasWidth;
      this.canvas.height = canvasHeight;

      // Re-disable image smoothing after resize
      this.ctx.imageSmoothingEnabled = false;

      // Draw checker background
      this.drawCheckerBackground(canvasWidth, canvasHeight);

      // Create ImageData from RGBA pixels
      const imageData = new ImageData(
        new Uint8ClampedArray(result.pixels),
        result.width,
        result.height
      );

      // Create temporary canvas for unscaled image
      const tempCanvas = document.createElement('canvas');
      tempCanvas.width = result.width;
      tempCanvas.height = result.height;
      const tempCtx = tempCanvas.getContext('2d')!;
      tempCtx.putImageData(imageData, 0, 0);

      // Draw scaled image
      this.ctx.drawImage(
        tempCanvas,
        0,
        0,
        result.width,
        result.height,
        0,
        0,
        canvasWidth,
        canvasHeight
      );

      output.success = true;
    } catch (err) {
      output.error = `Render error: ${err}`;
    }

    return output;
  }

  /**
   * Get the ImageData of the current rendered sprite (unscaled).
   */
  getImageData(): ImageData | null {
    if (!this.lastRenderResult || this.lastRenderResult.width === 0) {
      return null;
    }

    return new ImageData(
      new Uint8ClampedArray(this.lastRenderResult.pixels),
      this.lastRenderResult.width,
      this.lastRenderResult.height
    );
  }

  /**
   * Get the canvas element.
   */
  getCanvas(): HTMLCanvasElement {
    return this.canvas;
  }

  /**
   * Get the current scale factor.
   */
  getScale(): number {
    return this.currentScale;
  }

  /**
   * Get the last render result.
   */
  getRenderResult(): RenderResult | null {
    return this.lastRenderResult;
  }

  /**
   * Clear the preview.
   */
  clear(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
    this.lastRenderResult = null;
    this.canvas.width = 0;
    this.canvas.height = 0;
  }

  /**
   * Destroy the preview component.
   */
  destroy(): void {
    this.clear();
    this.container.removeChild(this.canvas);
  }
}
