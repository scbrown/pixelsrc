# Example Gallery

A curated collection of Pixelsrc sprites to learn from and remix. Click any example to load it into the [Sandbox](sandbox.md).

<style>
.gallery-filters {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-bottom: 1rem;
}

.gallery-filters button {
  background: #6272a4;
  color: #f8f8f2;
  border: none;
  padding: 0.4rem 0.8rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}

.gallery-filters button:hover,
.gallery-filters button.active {
  background: #50fa7b;
  color: #282a36;
}

.gallery-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
  margin: 1rem 0;
}

.gallery-item {
  background: var(--code-bg, #44475a);
  border-radius: 4px;
  padding: 0.75rem;
  cursor: pointer;
  transition: transform 0.1s, box-shadow 0.1s;
}

.gallery-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}

.gallery-item .preview {
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(#808080 0% 25%, #c0c0c0 0% 50%) 50% / 16px 16px;
  border-radius: 4px;
  margin-bottom: 0.5rem;
}

.gallery-item .preview img {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  max-height: 64px;
}

.gallery-item .name {
  font-weight: bold;
  color: #f8f8f2;
  font-size: 0.9rem;
}

.gallery-item .category {
  font-size: 0.75rem;
  color: #8be9fd;
  text-transform: uppercase;
}

.gallery-section {
  margin: 2rem 0;
}

.gallery-section h2 {
  border-bottom: 1px solid #6272a4;
  padding-bottom: 0.5rem;
}
</style>

<div class="gallery-filters">
  <button class="active" onclick="filterGallery('all')">All</button>
  <button onclick="filterGallery('characters')">Characters</button>
  <button onclick="filterGallery('items')">Items</button>
  <button onclick="filterGallery('ui')">UI</button>
  <button onclick="filterGallery('nature')">Nature</button>
  <button onclick="filterGallery('effects')">Effects</button>
</div>

## Characters

<div class="gallery-grid" data-category="characters">

<div class="gallery-item" onclick="loadGalleryExample('hero')">
  <div class="preview" id="preview-hero"></div>
  <div class="name">Hero</div>
  <div class="category">Characters</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('slime')">
  <div class="preview" id="preview-slime"></div>
  <div class="name">Slime</div>
  <div class="category">Characters</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('ghost')">
  <div class="preview" id="preview-ghost"></div>
  <div class="name">Ghost</div>
  <div class="category">Characters</div>
</div>

</div>

## Items

<div class="gallery-grid" data-category="items">

<div class="gallery-item" onclick="loadGalleryExample('sword')">
  <div class="preview" id="preview-sword"></div>
  <div class="name">Sword</div>
  <div class="category">Items</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('potion')">
  <div class="preview" id="preview-potion"></div>
  <div class="name">Health Potion</div>
  <div class="category">Items</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('key')">
  <div class="preview" id="preview-key"></div>
  <div class="name">Key</div>
  <div class="category">Items</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('coin')">
  <div class="preview" id="preview-coin"></div>
  <div class="name">Coin</div>
  <div class="category">Items</div>
</div>

</div>

## UI Elements

<div class="gallery-grid" data-category="ui">

<div class="gallery-item" onclick="loadGalleryExample('heart')">
  <div class="preview" id="preview-heart"></div>
  <div class="name">Heart</div>
  <div class="category">UI</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('star')">
  <div class="preview" id="preview-star"></div>
  <div class="name">Star</div>
  <div class="category">UI</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('arrow')">
  <div class="preview" id="preview-arrow"></div>
  <div class="name">Arrow</div>
  <div class="category">UI</div>
</div>

</div>

## Nature

<div class="gallery-grid" data-category="nature">

<div class="gallery-item" onclick="loadGalleryExample('tree')">
  <div class="preview" id="preview-tree"></div>
  <div class="name">Tree</div>
  <div class="category">Nature</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('flower')">
  <div class="preview" id="preview-flower"></div>
  <div class="name">Flower</div>
  <div class="category">Nature</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('mushroom')">
  <div class="preview" id="preview-mushroom"></div>
  <div class="name">Mushroom</div>
  <div class="category">Nature</div>
</div>

</div>

## Effects

<div class="gallery-grid" data-category="effects">

<div class="gallery-item" onclick="loadGalleryExample('explosion')">
  <div class="preview" id="preview-explosion"></div>
  <div class="name">Explosion</div>
  <div class="category">Effects</div>
</div>

<div class="gallery-item" onclick="loadGalleryExample('sparkle')">
  <div class="preview" id="preview-sparkle"></div>
  <div class="name">Sparkle</div>
  <div class="category">Effects</div>
</div>

</div>

<script>
// Example sprite data
const galleryExamples = {
  hero: `{"type":"palette","id":"hero-pal","colors":{"{_}":"#0000","{s}":"#f5deb3","{h}":"#8b4513","{b}":"#4169e1","{e}":"#000000","{w}":"#ffffff"}}
{"type":"sprite","name":"hero","palette":"hero-pal","grid":["{_}{_}{h}{h}{h}{_}{_}","{_}{h}{h}{h}{h}{h}{_}","{_}{s}{e}{s}{e}{s}{_}","{_}{s}{s}{s}{s}{s}{_}","{_}{_}{s}{s}{s}{_}{_}","{_}{b}{b}{b}{b}{b}{_}","{_}{b}{_}{b}{_}{b}{_}","{_}{s}{_}{_}{_}{s}{_}"]}`,

  slime: `{"type":"palette","id":"slime-pal","colors":{"{_}":"#0000","{g}":"#32cd32","{d}":"#228b22","{w}":"#ffffff","{k}":"#000000"}}
{"type":"sprite","name":"slime","palette":"slime-pal","grid":["{_}{_}{g}{g}{g}{_}{_}","{_}{g}{g}{g}{g}{g}{_}","{g}{w}{k}{g}{w}{k}{g}","{g}{g}{g}{g}{g}{g}{g}","{d}{g}{g}{g}{g}{g}{d}","{_}{d}{d}{d}{d}{d}{_}"]}`,

  ghost: `{"type":"palette","id":"ghost-pal","colors":{"{_}":"#0000","{w}":"#f0f0f0","{g}":"#d0d0d0","{b}":"#000000"}}
{"type":"sprite","name":"ghost","palette":"ghost-pal","grid":["{_}{_}{w}{w}{w}{_}{_}","{_}{w}{w}{w}{w}{w}{_}","{w}{b}{w}{w}{b}{w}{w}","{w}{w}{w}{w}{w}{w}{w}","{w}{w}{w}{w}{w}{w}{w}","{w}{_}{w}{_}{w}{_}{w}"]}`,

  sword: `{"type":"palette","id":"sword-pal","colors":{"{_}":"#0000","{s}":"#c0c0c0","{h}":"#ffd700","{g}":"#808080","{b}":"#8b4513"}}
{"type":"sprite","name":"sword","palette":"sword-pal","grid":["{_}{_}{_}{_}{s}{_}","{_}{_}{_}{s}{g}{_}","{_}{_}{s}{g}{_}{_}","{_}{s}{g}{_}{_}{_}","{h}{g}{_}{_}{_}{_}","{h}{h}{_}{_}{_}{_}","{b}{h}{_}{_}{_}{_}","{b}{_}{_}{_}{_}{_}"]}`,

  potion: `{"type":"palette","id":"potion-pal","colors":{"{_}":"#0000","{g}":"#808080","{r}":"#ff0000","{p}":"#ff6666","{k}":"#404040"}}
{"type":"sprite","name":"potion","palette":"potion-pal","grid":["{_}{g}{g}{g}{_}","{_}{k}{k}{k}{_}","{_}{g}{g}{g}{_}","{g}{r}{r}{r}{g}","{g}{r}{p}{r}{g}","{g}{r}{r}{r}{g}","{_}{g}{g}{g}{_}"]}`,

  key: `{"type":"palette","id":"key-pal","colors":{"{_}":"#0000","{y}":"#ffd700","{d}":"#daa520"}}
{"type":"sprite","name":"key","palette":"key-pal","grid":["{_}{y}{y}{y}{_}{_}{_}{_}","{y}{d}{d}{d}{y}{_}{_}{_}","{y}{d}{_}{d}{y}{_}{_}{_}","{_}{y}{y}{y}{y}{y}{y}{y}","{_}{_}{_}{_}{y}{_}{y}{_}","{_}{_}{_}{_}{y}{y}{y}{_}"]}`,

  coin: `{"type":"palette","id":"coin-pal","colors":{"{_}":"#0000","{y}":"#ffd700","{o}":"#ffa500","{w}":"#ffff00"}}
{"type":"sprite","name":"coin","palette":"coin-pal","grid":["{_}{y}{y}{y}{_}","{y}{w}{y}{o}{y}","{y}{w}{y}{o}{y}","{y}{y}{y}{o}{y}","{_}{y}{o}{y}{_}"]}`,

  heart: `{"type":"palette","id":"heart-pal","colors":{"{_}":"#0000","{r}":"#ff0000","{p}":"#ff6666","{d}":"#cc0000"}}
{"type":"sprite","name":"heart","palette":"heart-pal","grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{p}{r}{r}{p}{r}{r}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}`,

  star: `{"type":"palette","id":"star-pal","colors":{"{_}":"#0000","{y}":"#ffd700","{w}":"#ffff00"}}
{"type":"sprite","name":"star","palette":"star-pal","grid":["{_}{_}{y}{_}{_}","{_}{y}{w}{y}{_}","{y}{y}{y}{y}{y}","{_}{y}{w}{y}{_}","{_}{y}{_}{y}{_}"]}`,

  arrow: `{"type":"palette","id":"arrow-pal","colors":{"{_}":"#0000","{w}":"#ffffff","{g}":"#808080"}}
{"type":"sprite","name":"arrow","palette":"arrow-pal","grid":["{_}{_}{w}{_}{_}","{_}{w}{w}{w}{_}","{w}{_}{w}{_}{w}","{_}{_}{w}{_}{_}","{_}{_}{w}{_}{_}"]}`,

  tree: `{"type":"palette","id":"tree-pal","colors":{"{_}":"#0000","{g}":"#228b22","{l}":"#32cd32","{b}":"#8b4513","{d}":"#654321"}}
{"type":"sprite","name":"tree","palette":"tree-pal","grid":["{_}{_}{g}{g}{g}{_}{_}","{_}{g}{l}{g}{l}{g}{_}","{g}{l}{g}{l}{g}{l}{g}","{g}{g}{l}{g}{l}{g}{g}","{_}{g}{g}{g}{g}{g}{_}","{_}{_}{b}{b}{b}{_}{_}","{_}{_}{b}{d}{b}{_}{_}","{_}{_}{b}{b}{b}{_}{_}"]}`,

  flower: `{"type":"palette","id":"flower-pal","colors":{"{_}":"#0000","{p}":"#ff69b4","{y}":"#ffff00","{g}":"#228b22","{l}":"#90ee90"}}
{"type":"sprite","name":"flower","palette":"flower-pal","grid":["{_}{p}{_}{p}{_}","{p}{p}{y}{p}{p}","{_}{p}{y}{p}{_}","{_}{_}{g}{_}{_}","{_}{l}{g}{l}{_}","{_}{_}{g}{_}{_}"]}`,

  mushroom: `{"type":"palette","id":"mush-pal","colors":{"{_}":"#0000","{r}":"#ff0000","{w}":"#ffffff","{t}":"#f5deb3","{b}":"#8b4513"}}
{"type":"sprite","name":"mushroom","palette":"mush-pal","grid":["{_}{_}{r}{r}{r}{_}{_}","{_}{r}{w}{r}{w}{r}{_}","{r}{r}{r}{r}{r}{r}{r}","{_}{_}{t}{t}{t}{_}{_}","{_}{_}{t}{t}{t}{_}{_}","{_}{b}{t}{t}{t}{b}{_}"]}`,

  explosion: `{"type":"palette","id":"exp-pal","colors":{"{_}":"#0000","{r}":"#ff0000","{o}":"#ff8800","{y}":"#ffff00","{w}":"#ffffff"}}
{"type":"sprite","name":"explosion","palette":"exp-pal","grid":["{_}{r}{_}{y}{_}{r}{_}","{o}{_}{y}{w}{y}{_}{o}","{_}{y}{o}{w}{o}{y}{_}","{r}{w}{w}{w}{w}{w}{r}","{_}{y}{o}{w}{o}{y}{_}","{o}{_}{y}{w}{y}{_}{o}","{_}{r}{_}{y}{_}{r}{_}"]}`,

  sparkle: `{"type":"palette","id":"sparkle-pal","colors":{"{_}":"#0000","{w}":"#ffffff","{y}":"#ffffaa","{b}":"#aaaaff"}}
{"type":"sprite","name":"sparkle","palette":"sparkle-pal","grid":["{_}{_}{w}{_}{_}","{_}{b}{y}{b}{_}","{w}{y}{w}{y}{w}","{_}{b}{y}{b}{_}","{_}{_}{w}{_}{_}"]}`
};

function loadGalleryExample(name) {
  const jsonl = galleryExamples[name];
  if (!jsonl) return;

  // Navigate to sandbox with the example
  const sandboxUrl = 'sandbox.html';
  // Store in sessionStorage so sandbox can pick it up
  sessionStorage.setItem('pixelsrc-gallery-load', jsonl);
  window.location.href = sandboxUrl;
}

function filterGallery(category) {
  // Update button states
  document.querySelectorAll('.gallery-filters button').forEach(btn => {
    btn.classList.remove('active');
    if (btn.textContent.toLowerCase() === category || (category === 'all' && btn.textContent === 'All')) {
      btn.classList.add('active');
    }
  });

  // Show/hide items
  document.querySelectorAll('.gallery-grid').forEach(grid => {
    const gridCategory = grid.dataset.category;
    if (category === 'all' || gridCategory === category) {
      grid.style.display = 'grid';
      grid.previousElementSibling.style.display = 'block'; // h2
    } else {
      grid.style.display = 'none';
      grid.previousElementSibling.style.display = 'none'; // h2
    }
  });
}

// Render preview thumbnails
function renderPreviews() {
  if (!window.pixelsrcDemo || !window.pixelsrcDemo.isReady()) {
    // Retry in a moment
    setTimeout(renderPreviews, 500);
    return;
  }

  Object.keys(galleryExamples).forEach(name => {
    const previewId = 'preview-' + name;
    const container = document.getElementById(previewId);
    if (container) {
      window.pixelsrcDemo.render(galleryExamples[name], previewId, { scale: 4 });
    }
  });
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
  setTimeout(renderPreviews, 500);
});
</script>

## Using Gallery Examples

1. **Browse**: Scroll through categories or use filters
2. **Preview**: Hover over an example to see it
3. **Load**: Click to open in the Sandbox
4. **Remix**: Modify colors, shapes, or add animations

## Submitting Examples

Have a cool sprite to share? Examples in this gallery should be:

- **Small**: 16x16 or smaller works best
- **Clear**: Easy to understand the subject
- **Educational**: Demonstrates a technique or pattern
- **Original**: Your own creation

See [Contributing](../appendix/contributing.md) for submission guidelines.

## Related Resources

- [Interactive Sandbox](sandbox.md) - Create your own sprites
- [Format Specification](../format/overview.md) - Learn the syntax
- [Persona Guides](../personas/sketcher.md) - Workflow tutorials
