// Type declarations for @stiwi/pixelsrc-wasm

declare module '@stiwi/pixelsrc-wasm' {
  /** Result from render_to_rgba */
  export class RenderResult {
    readonly width: number;
    readonly height: number;
    readonly pixels: Uint8Array;
    readonly warnings: string[];
    free(): void;
  }

  /** Initialize the WASM module asynchronously */
  export default function init(wasmBinary?: BufferSource | URL | Response): Promise<void>;

  /** Initialize the WASM module synchronously with binary data */
  export function initSync(wasmBinary: BufferSource | WebAssembly.Module): void;

  /** Render JSONL to PNG bytes */
  export function render_to_png(jsonl: string): Uint8Array;

  /** Render JSONL to RGBA pixels */
  export function render_to_rgba(jsonl: string): RenderResult;

  /** List sprite names in JSONL */
  export function list_sprites(jsonl: string): string[];

  /** Validate JSONL and return error messages */
  export function validate(jsonl: string): string[];
}

declare module '@stiwi/pixelsrc-wasm/pkg/pixelsrc_bg.wasm' {
  const wasmBinary: BufferSource;
  export default wasmBinary;
}
