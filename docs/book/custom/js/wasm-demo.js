/**
 * Pixelsrc WASM Demo Integration
 *
 * This script provides the pixelsrcDemo API for interactive examples
 * embedded throughout the documentation.
 *
 * Cross-Browser Compatibility:
 * - Chrome 90+, Firefox 88+, Safari 14+, Edge 90+
 * - Requires: WebAssembly, ES6 modules, Blob API, URL.createObjectURL
 * - Graceful fallback for unsupported browsers
 */

(function() {
    'use strict';

    // Feature detection for browser compatibility
    var browserSupport = {
        wasm: typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function',
        dynamicImport: (function() {
            try {
                // Check if dynamic import is supported
                return typeof import === 'function' || true; // Modern browsers
            } catch (e) {
                return false;
            }
        })(),
        blob: typeof Blob !== 'undefined',
        objectURL: typeof URL !== 'undefined' && typeof URL.createObjectURL === 'function',
        promises: typeof Promise !== 'undefined'
    };

    // Check overall compatibility
    var isCompatible = browserSupport.wasm && browserSupport.blob &&
                       browserSupport.objectURL && browserSupport.promises;

    // WASM module state
    var wasmModule = null;
    var wasmReady = false;
    var initError = null;

    // Initialize WASM module
    function initWasm() {
        if (!isCompatible) {
            initError = 'Browser not supported. Requires WebAssembly and ES6 features.';
            console.warn('Pixelsrc demos disabled:', initError);
            return Promise.resolve();
        }

        // Load the WASM module from the wasm-assets directory
        // Use path configured in head.hbs, or fall back to default
        var wasmPath = window.pixelsrcWasmPath || '/book/wasm-assets/pixelsrc_wasm.js';

        return import(wasmPath)
            .then(function(module) {
                return module.default().then(function() {
                    wasmModule = module;
                    wasmReady = true;
                    console.log('Pixelsrc WASM module loaded successfully');
                });
            })
            .catch(function(error) {
                initError = error.message;
                console.warn('WASM module not available:', error.message);
                // Demos will show fallback message
            });
    }

    // Convert Uint8Array PNG bytes to data URL
    function pngToDataUrl(pngBytes) {
        var blob = new Blob([pngBytes], { type: 'image/png' });
        return URL.createObjectURL(blob);
    }

    // Generate user-friendly browser support message
    function getBrowserSupportMessage() {
        if (!browserSupport.wasm) {
            return 'WebAssembly is not supported in your browser. Please use a modern browser like Chrome, Firefox, Safari, or Edge.';
        }
        if (!browserSupport.promises) {
            return 'Your browser does not support JavaScript Promises. Please update your browser.';
        }
        if (initError) {
            return 'WASM module failed to load: ' + initError;
        }
        return 'WASM module not loaded. Try refreshing the page.';
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
         * Check browser compatibility
         * @returns {Object} Browser support status
         */
        getBrowserSupport: function() {
            return {
                isCompatible: isCompatible,
                wasm: browserSupport.wasm,
                blob: browserSupport.blob,
                objectURL: browserSupport.objectURL,
                promises: browserSupport.promises,
                error: initError
            };
        },

        /**
         * Render JSONL content to a container
         * @param {string} jsonl - Pixelsrc JSONL content
         * @param {string} containerId - ID of the container element
         * @param {Object} options - Render options
         * @param {string} options.spriteName - Optional sprite name to render
         * @param {number} options.scale - CSS scale factor for display (default: 4)
         */
        render: function(jsonl, containerId, options) {
            options = options || {};
            var container = document.getElementById(containerId);
            if (!container) {
                console.error('Container not found:', containerId);
                return Promise.resolve();
            }

            if (!wasmReady) {
                container.innerHTML = '<p class="error">' + getBrowserSupportMessage() + '</p>';
                return Promise.resolve();
            }

            var scale = options.scale || 4;
            var spriteName = options.spriteName;

            return new Promise(function(resolve) {
                try {
                    // Call WASM render_to_png function
                    var pngBytes = wasmModule.render_to_png(jsonl, spriteName);
                    var dataUrl = pngToDataUrl(pngBytes);

                    // Create image with CSS scaling for crisp pixel art
                    var img = document.createElement('img');
                    img.src = dataUrl;
                    img.alt = 'Rendered sprite';
                    img.style.imageRendering = 'pixelated';
                    img.style.transform = 'scale(' + scale + ')';
                    img.style.transformOrigin = 'top left';

                    container.innerHTML = '';
                    container.appendChild(img);
                } catch (error) {
                    container.innerHTML = '<p class="error">Render error: ' + error.message + '</p>';
                }
                resolve();
            });
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
        renderFromTextarea: function(textareaId, previewId, options) {
            options = options || {};
            var textarea = document.getElementById(textareaId);
            if (!textarea) {
                console.error('Textarea not found:', textareaId);
                return Promise.resolve();
            }
            var jsonl = textarea.value;
            return this.render(jsonl, previewId, options);
        },

        /**
         * Initialize all demo containers on the page
         * Finds elements with data-pixelsrc-demo attribute and sets up handlers
         */
        initDemos: function() {
            var self = this;
            var demos = document.querySelectorAll('[data-pixelsrc-demo]');
            for (var i = 0; i < demos.length; i++) {
                (function(demo) {
                    var textarea = demo.querySelector('textarea');
                    var button = demo.querySelector('button');
                    var preview = demo.querySelector('.preview');

                    if (textarea && button && preview) {
                        button.addEventListener('click', function() {
                            self.render(textarea.value, preview.id);
                        });
                    }
                })(demos[i]);
            }
        },

        /**
         * Initialize auto-render demo containers (from generate-demos.sh)
         * Finds .demo-container[data-demo] elements, extracts JSONL from
         * preceding .demo-source code block, and renders sprites.
         */
        initDemoContainers: function() {
            var self = this;
            var containers = document.querySelectorAll('.demo-container[data-demo]');

            for (var i = 0; i < containers.length; i++) {
                (function(container) {
                    var demoId = container.getAttribute('data-demo');

                    // Find preceding .demo-source sibling
                    var source = container.previousElementSibling;
                    while (source && !source.classList.contains('demo-source')) {
                        source = source.previousElementSibling;
                    }

                    if (!source) {
                        container.innerHTML = '<p class="demo-error">Demo source not found</p>';
                        return;
                    }

                    // Extract JSONL from code block
                    var codeBlock = source.querySelector('pre code');
                    if (!codeBlock) {
                        container.innerHTML = '<p class="demo-error">Code block not found</p>';
                        return;
                    }

                    var jsonl = codeBlock.textContent;
                    if (!jsonl || !jsonl.trim()) {
                        container.innerHTML = '<p class="demo-error">Empty JSONL content</p>';
                        return;
                    }

                    // Generate unique container ID
                    var containerId = 'demo-render-' + demoId + '-' + i;
                    container.id = containerId;

                    // Create loading indicator
                    container.innerHTML = '<p class="demo-loading">Loading demo...</p>';

                    // Wait for WASM to be ready, then render
                    var attempts = 0;
                    var maxAttempts = 50; // 5 seconds max

                    function tryRender() {
                        if (self.isReady()) {
                            self.renderDemo(jsonl, containerId);
                        } else if (attempts < maxAttempts) {
                            attempts++;
                            setTimeout(tryRender, 100);
                        } else {
                            container.innerHTML = '<p class="demo-error">WASM module not available</p>';
                        }
                    }

                    tryRender();
                })(containers[i]);
            }
        },

        /**
         * Render a demo with multiple sprites/animations
         * Renders the first sprite found in the JSONL content
         * @param {string} jsonl - Pixelsrc JSONL content
         * @param {string} containerId - ID of the container element
         */
        renderDemo: function(jsonl, containerId) {
            var container = document.getElementById(containerId);
            if (!container) {
                console.error('Demo container not found:', containerId);
                return;
            }

            if (!wasmReady) {
                container.innerHTML = '<p class="demo-error">' + getBrowserSupportMessage() + '</p>';
                return;
            }

            try {
                // Get list of sprites from the JSONL
                var sprites = this.listSprites(jsonl);

                if (!sprites || sprites.length === 0) {
                    container.innerHTML = '<p class="demo-error">No sprites found in demo</p>';
                    return;
                }

                // Create container for rendered sprites
                container.innerHTML = '';
                container.className = 'demo-container demo-rendered';

                // Render each sprite (or first few if many)
                var maxSprites = Math.min(sprites.length, 4);
                for (var i = 0; i < maxSprites; i++) {
                    var spriteName = sprites[i];
                    var spriteDiv = document.createElement('div');
                    spriteDiv.className = 'demo-sprite';

                    try {
                        var pngBytes = wasmModule.render_to_png(jsonl, spriteName);
                        var blob = new Blob([pngBytes], { type: 'image/png' });
                        var dataUrl = URL.createObjectURL(blob);

                        var img = document.createElement('img');
                        img.src = dataUrl;
                        img.alt = spriteName;
                        img.title = spriteName;
                        img.style.imageRendering = 'pixelated';

                        var label = document.createElement('span');
                        label.className = 'demo-sprite-label';
                        label.textContent = spriteName;

                        spriteDiv.appendChild(img);
                        spriteDiv.appendChild(label);
                    } catch (error) {
                        spriteDiv.innerHTML = '<span class="demo-error">' + spriteName + ': ' + error.message + '</span>';
                    }

                    container.appendChild(spriteDiv);
                }

                // If there are more sprites, show count
                if (sprites.length > maxSprites) {
                    var moreLabel = document.createElement('span');
                    moreLabel.className = 'demo-more';
                    moreLabel.textContent = '+ ' + (sprites.length - maxSprites) + ' more';
                    container.appendChild(moreLabel);
                }
            } catch (error) {
                container.innerHTML = '<p class="demo-error">Render error: ' + error.message + '</p>';
            }
        }
    };

    // Initialize on page load
    function init() {
        initWasm();
        // Initialize demo containers after DOM is ready
        // Each container will wait for WASM to be ready before rendering
        window.pixelsrcDemo.initDemoContainers();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
