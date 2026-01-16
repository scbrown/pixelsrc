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
            // Use path configured in head.hbs, or fall back to default
            const wasmPath = window.pixelsrcWasmPath || '/book/wasm-assets/pixelsrc_wasm.js';
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

    // Convert Uint8Array PNG bytes to data URL
    function pngToDataUrl(pngBytes) {
        const blob = new Blob([pngBytes], { type: 'image/png' });
        return URL.createObjectURL(blob);
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
         * @param {Object} options - Render options
         * @param {string} options.spriteName - Optional sprite name to render
         * @param {number} options.scale - CSS scale factor for display (default: 4)
         */
        render: async function(jsonl, containerId, options = {}) {
            const container = document.getElementById(containerId);
            if (!container) {
                console.error('Container not found:', containerId);
                return;
            }

            if (!wasmReady) {
                container.innerHTML = '<p class="error">WASM module not loaded. Try refreshing the page.</p>';
                return;
            }

            const scale = options.scale || 4;
            const spriteName = options.spriteName;

            try {
                // Call WASM render_to_png function
                const pngBytes = wasmModule.render_to_png(jsonl, spriteName);
                const dataUrl = pngToDataUrl(pngBytes);

                // Create image with CSS scaling for crisp pixel art
                const img = document.createElement('img');
                img.src = dataUrl;
                img.alt = 'Rendered sprite';
                img.style.imageRendering = 'pixelated';
                img.style.transform = `scale(${scale})`;
                img.style.transformOrigin = 'top left';

                container.innerHTML = '';
                container.appendChild(img);
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
        },

        /**
         * Render from a textarea element to a preview container
         * Convenience function for embedded "try it" demos
         * @param {string} textareaId - ID of the textarea containing JSONL
         * @param {string} previewId - ID of the preview container
         * @param {Object} options - Render options (same as render())
         */
        renderFromTextarea: async function(textareaId, previewId, options = {}) {
            const textarea = document.getElementById(textareaId);
            if (!textarea) {
                console.error('Textarea not found:', textareaId);
                return;
            }
            const jsonl = textarea.value;
            await this.render(jsonl, previewId, options);
        },

        /**
         * Initialize all demo containers on the page
         * Finds elements with data-pixelsrc-demo attribute and sets up handlers
         */
        initDemos: function() {
            document.querySelectorAll('[data-pixelsrc-demo]').forEach(demo => {
                const textarea = demo.querySelector('textarea');
                const button = demo.querySelector('button');
                const preview = demo.querySelector('.preview');

                if (textarea && button && preview) {
                    button.addEventListener('click', () => {
                        this.render(textarea.value, preview.id);
                    });
                }
            });
        }
    };

    // Initialize on page load
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initWasm);
    } else {
        initWasm();
    }
})();
