import { Notice, Plugin } from 'obsidian';
import { PixelsrcSettingTab, DEFAULT_SETTINGS, PixelsrcSettings } from './settings';
import { registerCodeBlockProcessor } from './codeblock';
import { registerLivePreviewExtension } from './livepreview';
import { initWasm, isInitialized, getInitError } from './renderer';

export default class PixelsrcPlugin extends Plugin {
  settings!: PixelsrcSettings;
  wasmReady = false;

  async onload() {
    console.log('Loading PixelSrc plugin');

    // Load settings
    await this.loadSettings();

    // Initialize WASM module
    // This is async but we don't want to block plugin load
    this.initializeWasm();

    // Register code block processor for reading mode
    registerCodeBlockProcessor(this);

    // Register live preview extension for edit mode
    registerLivePreviewExtension(this);

    // Add settings tab
    this.addSettingTab(new PixelsrcSettingTab(this.app, this));
  }

  /**
   * Initialize WASM module with error handling.
   * Runs asynchronously to avoid blocking plugin load.
   */
  private async initializeWasm(): Promise<void> {
    try {
      await initWasm();
      this.wasmReady = true;
      console.log('PixelSrc WASM initialized');
    } catch (error) {
      console.error('PixelSrc WASM initialization failed:', error);
      // Show a notice to the user that rendering won't work
      new Notice(
        'PixelSrc: Failed to initialize WASM renderer. ' +
        'Pixel art rendering will not work. ' +
        'Check the console for details.',
        10000
      );
    }
  }

  /**
   * Check if WASM is ready for rendering.
   */
  isWasmReady(): boolean {
    return this.wasmReady && isInitialized();
  }

  /**
   * Get WASM initialization error if any.
   */
  getWasmError(): Error | null {
    return getInitError();
  }

  onunload() {
    console.log('Unloading PixelSrc plugin');
  }

  async loadSettings() {
    this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
  }

  async saveSettings() {
    await this.saveData(this.settings);
  }
}
