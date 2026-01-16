/**
 * Pixelsrc WASM Demo Integration
 *
 * This script provides the pixelsrcDemo API for interactive examples
 * embedded throughout the documentation.
 */

(function() {
    'use strict';

    // WASM module state
    let wasmModule = null;
    let wasmReady = false;

    // Initialize WASM module
    async function initWasm() {
        try {
            // Load the WASM module from the wasm-assets directory
            const wasmPath = '/book/wasm-assets/pixelsrc_wasm.js';
            const module = await import(wasmPath);
            await module.default();
            wasmModule = module;
            wasmReady = true;
            console.log('Pixelsrc WASM module loaded successfully');
        } catch (error) {
            console.warn('WASM module not available:', error.message);
            // Demos will show fallback message
        }
    }

    // Public API
    window.pixelsrcDemo = {
        /**
         * Check if WASM is ready
         * @returns {boolean}
         */
        isReady: function() {
            return wasmReady;
        },

        /**
         * Render JSONL content to a container
         * @param {string} jsonl - Pixelsrc JSONL content
         * @param {string} containerId - ID of the container element
         * @param {number} scale - Scale factor (default: 4)
         */
        render: async function(jsonl, containerId, scale = 4) {
            const container = document.getElementById(containerId);
            if (!container) {
                console.error('Container not found:', containerId);
                return;
            }

            if (!wasmReady) {
                container.innerHTML = '<p class="error">WASM module not loaded. Try refreshing the page.</p>';
                return;
            }

            try {
                // Call WASM render function
                const result = wasmModule.render_to_data_url(jsonl, scale);
                if (result.error) {
                    container.innerHTML = `<p class="error">${result.error}</p>`;
                } else {
                    container.innerHTML = `<img src="${result.data_url}" alt="Rendered sprite">`;
                }
            } catch (error) {
                container.innerHTML = `<p class="error">Render error: ${error.message}</p>`;
            }
        },

        /**
         * Validate JSONL content
         * @param {string} jsonl - Pixelsrc JSONL content
         * @returns {Object} Validation result with warnings and errors
         */
        validate: function(jsonl) {
            if (!wasmReady) {
                return { error: 'WASM module not loaded' };
            }

            try {
                return wasmModule.validate(jsonl);
            } catch (error) {
                return { error: error.message };
            }
        },

        /**
         * List sprites in JSONL content
         * @param {string} jsonl - Pixelsrc JSONL content
         * @returns {string[]} Array of sprite names
         */
        listSprites: function(jsonl) {
            if (!wasmReady) {
                return [];
            }

            try {
                return wasmModule.list_sprites(jsonl);
            } catch (error) {
                console.error('List sprites error:', error);
                return [];
            }
        }
    };

    // Initialize on page load
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initWasm);
    } else {
        initWasm();
    }
})();
