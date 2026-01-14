// WASM Integration for Obsidian PixelSrc Plugin
// Wraps @stiwi/pixelsrc-wasm for rendering pixel art from JSONL

// Static import of WASM binary - esbuild bundles this as Uint8Array with binary loader
import wasmBinary from '@stiwi/pixelsrc-wasm/pkg/pixelsrc_bg.wasm';

// WASM module will be imported dynamically after init
import * as wasm from '@stiwi/pixelsrc-wasm';

let initialized = false;
let initError: Error | null = null;

/**
 * Initialize the WASM module for rendering.
 * Must be called before any render functions.
 * Safe to call multiple times - will only initialize once.
 */
export async function initWasm(): Promise<void> {
  if (initialized) return;
  if (initError) throw initError;

  try {
    // Initialize with binary data using initSync for synchronous initialization
    // This avoids issues with fetch() in Electron/Obsidian environments
    // The wasmBinary is bundled as a Uint8Array by esbuild's binary loader
    wasm.initSync(wasmBinary);

    initialized = true;
    console.log('PixelSrc WASM initialized');
  } catch (error) {
    initError = error instanceof Error ? error : new Error(String(error));
    console.error('Failed to initialize PixelSrc WASM:', initError);
    throw initError;
  }
}

/**
 * Check if WASM is initialized and ready to use.
 */
export function isInitialized(): boolean {
  return initialized;
}

/**
 * Get the initialization error if WASM failed to load.
 */
export function getInitError(): Error | null {
  return initError;
}

/**
 * Render JSONL sprite definition to PNG bytes.
 * @param jsonl - JSONL string containing sprite definition
 * @returns PNG image data as Uint8Array
 * @throws Error if WASM not initialized or rendering fails
 */
export function renderToPng(jsonl: string): Uint8Array {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  const result = wasm.render_to_png(jsonl);
  return new Uint8Array(result);
}

/**
 * Result of rendering to RGBA pixels.
 */
export interface RenderResult {
  width: number;
  height: number;
  pixels: Uint8Array;
  warnings: string[];
}

/**
 * Render JSONL sprite definition to RGBA pixels.
 * @param jsonl - JSONL string containing sprite definition
 * @returns Object with width, height, pixels (RGBA), and warnings
 * @throws Error if WASM not initialized
 */
export function renderToRgba(jsonl: string): RenderResult {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  const result = wasm.render_to_rgba(jsonl);
  return {
    width: result.width,
    height: result.height,
    pixels: new Uint8Array(result.pixels),
    warnings: Array.from(result.warnings),
  };
}

/**
 * List all sprite names in a JSONL string.
 * @param jsonl - JSONL string containing sprite definitions
 * @returns Array of sprite names
 * @throws Error if WASM not initialized
 */
export function listSprites(jsonl: string): string[] {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  return Array.from(wasm.list_sprites(jsonl));
}

/**
 * Validate JSONL and return any errors or warnings.
 * @param jsonl - JSONL string to validate
 * @returns Array of validation messages (empty if valid)
 * @throws Error if WASM not initialized
 */
export function validateJsonl(jsonl: string): string[] {
  if (!initialized) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  return Array.from(wasm.validate(jsonl));
}
