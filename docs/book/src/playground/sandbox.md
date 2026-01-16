# Interactive Sandbox

The sandbox lets you write and test Pixelsrc sprites directly in your browser. Edit the JSONL below and click **Render** to see your sprite.

<div class="pixelsrc-sandbox" id="sandbox">
  <div class="sandbox-editor">
    <div class="editor-toolbar">
      <button onclick="sandboxClear()" title="Clear editor">Clear</button>
      <button onclick="sandboxFormat()" title="Format JSON">Format</button>
      <button onclick="sandboxValidate()" title="Validate syntax">Validate</button>
    </div>
    <textarea id="sandbox-input" spellcheck="false">{"type":"palette","id":"colors","colors":{"{_}":"#0000","{b}":"#3b4994","{w}":"#ffffff","{k}":"#000000"}}
{"type":"sprite","name":"star","palette":"colors","grid":["{_}{_}{b}{_}{_}","{_}{b}{b}{b}{_}","{b}{b}{w}{b}{b}","{_}{b}{b}{b}{_}","{_}{_}{b}{_}{_}"]}</textarea>
  </div>
  <div class="sandbox-output">
    <div class="output-toolbar">
      <button onclick="sandboxRender()" class="primary" title="Render sprite">Render</button>
      <button onclick="sandboxDownloadPng()" title="Download as PNG">PNG</button>
      <button onclick="sandboxDownloadGif()" title="Download as animated GIF" disabled id="gif-btn">GIF</button>
    </div>
    <div id="sandbox-preview" class="preview"></div>
    <div id="sandbox-info" class="info"></div>
    <div id="sandbox-errors" class="errors"></div>
  </div>
</div>

<style>
.pixelsrc-sandbox {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin: 1rem 0;
  min-height: 400px;
}

@media (max-width: 768px) {
  .pixelsrc-sandbox {
    grid-template-columns: 1fr;
  }
}

.sandbox-editor,
.sandbox-output {
  display: flex;
  flex-direction: column;
  background: var(--code-bg, #44475a);
  border-radius: 4px;
  padding: 0.5rem;
}

.editor-toolbar,
.output-toolbar {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.editor-toolbar button,
.output-toolbar button {
  background: #6272a4;
  color: #f8f8f2;
  border: none;
  padding: 0.4rem 0.8rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}

.editor-toolbar button:hover,
.output-toolbar button:hover {
  background: #7282b4;
}

.output-toolbar button.primary {
  background: #50fa7b;
  color: #282a36;
  font-weight: bold;
}

.output-toolbar button.primary:hover {
  background: #5af78e;
}

.output-toolbar button:disabled {
  background: #44475a;
  color: #6272a4;
  cursor: not-allowed;
}

#sandbox-input {
  flex: 1;
  width: 100%;
  min-height: 300px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 0.9rem;
  background: var(--bg-color, #282a36);
  color: var(--fg-color, #f8f8f2);
  border: 1px solid #6272a4;
  border-radius: 4px;
  padding: 0.75rem;
  resize: vertical;
  line-height: 1.4;
}

#sandbox-preview {
  flex: 1;
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(#808080 0% 25%, #c0c0c0 0% 50%) 50% / 16px 16px;
  border-radius: 4px;
  overflow: hidden;
}

#sandbox-preview img {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}

#sandbox-info {
  margin-top: 0.5rem;
  font-size: 0.85rem;
  color: #8be9fd;
}

#sandbox-errors {
  margin-top: 0.5rem;
  font-size: 0.85rem;
  color: #ff5555;
  font-family: monospace;
  white-space: pre-wrap;
}

.sandbox-success {
  color: #50fa7b !important;
}
</style>

<script>
// Sandbox state
let lastRenderedPng = null;

function sandboxRender() {
  const input = document.getElementById('sandbox-input');
  const preview = document.getElementById('sandbox-preview');
  const info = document.getElementById('sandbox-info');
  const errors = document.getElementById('sandbox-errors');
  const gifBtn = document.getElementById('gif-btn');

  errors.textContent = '';
  info.textContent = '';
  info.className = 'info';

  if (!window.pixelsrcDemo || !window.pixelsrcDemo.isReady()) {
    errors.textContent = 'WASM module not loaded. Please refresh the page.';
    return;
  }

  const jsonl = input.value;

  // Validate first
  const validation = window.pixelsrcDemo.validate(jsonl);
  if (validation.error) {
    errors.textContent = validation.error;
    return;
  }
  if (validation.errors && validation.errors.length > 0) {
    errors.textContent = validation.errors.join('\n');
    return;
  }

  // List sprites
  const sprites = window.pixelsrcDemo.listSprites(jsonl);
  if (sprites.length === 0) {
    errors.textContent = 'No sprites found in input';
    return;
  }

  // Check if any sprite has animation
  let hasAnimation = false;
  try {
    const lines = jsonl.split('\n').filter(l => l.trim());
    for (const line of lines) {
      const obj = JSON.parse(line);
      if (obj.type === 'animation') {
        hasAnimation = true;
        break;
      }
    }
  } catch (e) {}

  // Enable/disable GIF button
  gifBtn.disabled = !hasAnimation;

  // Render
  window.pixelsrcDemo.render(jsonl, 'sandbox-preview', { scale: 8 });

  info.textContent = `Rendered: ${sprites.join(', ')}`;
  info.className = 'info sandbox-success';
}

function sandboxClear() {
  document.getElementById('sandbox-input').value = '';
  document.getElementById('sandbox-preview').innerHTML = '';
  document.getElementById('sandbox-info').textContent = '';
  document.getElementById('sandbox-errors').textContent = '';
}

function sandboxFormat() {
  const input = document.getElementById('sandbox-input');
  const errors = document.getElementById('sandbox-errors');
  errors.textContent = '';

  try {
    const lines = input.value.split('\n').filter(l => l.trim());
    const formatted = lines.map(line => {
      const obj = JSON.parse(line);
      return JSON.stringify(obj);
    }).join('\n');
    input.value = formatted;
  } catch (e) {
    errors.textContent = 'Format error: ' + e.message;
  }
}

function sandboxValidate() {
  const input = document.getElementById('sandbox-input');
  const info = document.getElementById('sandbox-info');
  const errors = document.getElementById('sandbox-errors');

  errors.textContent = '';
  info.textContent = '';

  if (!window.pixelsrcDemo || !window.pixelsrcDemo.isReady()) {
    errors.textContent = 'WASM module not loaded. Please refresh the page.';
    return;
  }

  const validation = window.pixelsrcDemo.validate(input.value);

  if (validation.error) {
    errors.textContent = validation.error;
    return;
  }

  if (validation.errors && validation.errors.length > 0) {
    errors.textContent = validation.errors.join('\n');
    return;
  }

  if (validation.warnings && validation.warnings.length > 0) {
    info.textContent = 'Warnings:\n' + validation.warnings.join('\n');
    return;
  }

  info.textContent = 'Valid!';
  info.className = 'info sandbox-success';
}

function sandboxDownloadPng() {
  const preview = document.getElementById('sandbox-preview');
  const img = preview.querySelector('img');

  if (!img) {
    document.getElementById('sandbox-errors').textContent = 'Render a sprite first';
    return;
  }

  const link = document.createElement('a');
  link.download = 'sprite.png';
  link.href = img.src;
  link.click();
}

function sandboxDownloadGif() {
  // GIF export requires animation support in WASM
  document.getElementById('sandbox-errors').textContent = 'GIF export requires an animation. Define animation frames first.';
}

// Load example into sandbox
function loadExample(jsonl) {
  document.getElementById('sandbox-input').value = jsonl;
  sandboxRender();
}

// Auto-render on page load if WASM is ready
document.addEventListener('DOMContentLoaded', function() {
  // Check if we have a gallery example to load
  const galleryData = sessionStorage.getItem('pixelsrc-gallery-load');
  if (galleryData) {
    document.getElementById('sandbox-input').value = galleryData;
    sessionStorage.removeItem('pixelsrc-gallery-load');
  }

  // Give WASM a moment to initialize
  setTimeout(function() {
    if (window.pixelsrcDemo && window.pixelsrcDemo.isReady()) {
      sandboxRender();
    }
  }, 500);
});
</script>

## Quick Reference

### Palette Definition

```json
{"type":"palette","id":"my-colors","colors":{"{_}":"#0000","{r}":"#ff0000","{g}":"#00ff00","{b}":"#0000ff"}}
```

- `{_}` is conventionally used for transparent
- Colors can be `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA`
- Tokens are 1-3 characters wrapped in `{}`

### Sprite Definition

```json
{"type":"sprite","name":"my-sprite","palette":"my-colors","grid":["row1","row2"]}
```

- `palette` references a palette by ID
- `grid` is an array of strings using palette tokens
- Each row must have the same width (in tokens)

### Animation Definition

```json
{"type":"animation","sprite":"my-sprite","frames":[{"duration":100},{"duration":100}]}
```

- `sprite` references the animated sprite
- `frames` array defines timing per frame
- `duration` is in milliseconds

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | Render sprite |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> | Format JSON |

<script>
document.getElementById('sandbox-input').addEventListener('keydown', function(e) {
  if (e.ctrlKey && e.key === 'Enter') {
    e.preventDefault();
    sandboxRender();
  }
  if (e.ctrlKey && e.shiftKey && e.key === 'F') {
    e.preventDefault();
    sandboxFormat();
  }
});
</script>

## Tips

1. **Start simple**: Begin with a small palette and sprite, then expand
2. **Use transparency**: The `{_}` token with `#0000` creates transparent pixels
3. **Check validation**: Click Validate to catch syntax errors before rendering
4. **Scale matters**: Sprites render at 8x scale by default for visibility
5. **Token length**: Keep tokens short (1-3 chars) for readable grid patterns

## Next Steps

- Browse the [Example Gallery](gallery.md) for inspiration
- Learn about [Animations](../format/animation.md) to bring sprites to life
- Explore [Compositions](../format/composition.md) for complex scenes
