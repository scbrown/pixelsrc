import { render_to_png } from '@stiwi/pixelsrc-wasm';

export interface GalleryExample {
  name: string;
  jsonl: string;
}

export interface GalleryOptions {
  container: HTMLElement;
  onSelect: (jsonl: string) => void;
}

const EXAMPLE_FILES = [
  'heart.jsonl',
  'hero.jsonl',
  'coin.jsonl',
  'tree.jsonl',
  'sword.jsonl',
];

export class Gallery {
  private container: HTMLElement;
  private onSelect: (jsonl: string) => void;
  private examples: GalleryExample[] = [];

  constructor(options: GalleryOptions) {
    this.container = options.container;
    this.onSelect = options.onSelect;
  }

  async loadExamples(): Promise<void> {
    this.examples = [];

    // Show loading state
    this.showLoading();

    // Load all examples in parallel for better performance
    const results = await Promise.allSettled(
      EXAMPLE_FILES.map(async (filename) => {
        const response = await fetch(`/examples/${filename}`);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        const jsonl = await response.text();
        const name = filename.replace('.jsonl', '');
        return { name, jsonl: jsonl.trim() };
      })
    );

    // Collect successful results
    for (const result of results) {
      if (result.status === 'fulfilled') {
        this.examples.push(result.value);
      } else {
        console.warn('Failed to load example:', result.reason);
      }
    }

    this.render();
  }

  private showLoading(): void {
    this.container.innerHTML = '';
    const loadingEl = document.createElement('div');
    loadingEl.className = 'gallery-loading';
    loadingEl.textContent = 'Loading examples...';
    this.container.appendChild(loadingEl);
  }

  private render(): void {
    this.container.innerHTML = '';

    if (this.examples.length === 0) {
      const emptyEl = document.createElement('div');
      emptyEl.className = 'gallery-empty';
      emptyEl.textContent = 'No examples available';
      this.container.appendChild(emptyEl);
      return;
    }

    for (const example of this.examples) {
      const item = document.createElement('button');
      item.className = 'gallery-item';
      item.title = `Load ${example.name} example`;
      item.type = 'button';

      // Create thumbnail container
      const thumbContainer = document.createElement('div');
      thumbContainer.className = 'gallery-thumb';

      // Render thumbnail
      try {
        const pngBytes = render_to_png(example.jsonl);
        const blob = new Blob([pngBytes.slice()], { type: 'image/png' });
        const url = URL.createObjectURL(blob);

        const img = document.createElement('img');
        img.src = url;
        img.alt = `${example.name} sprite preview`;
        thumbContainer.appendChild(img);
      } catch (err) {
        // Fallback to first letter if render fails
        const fallback = document.createElement('span');
        fallback.textContent = example.name.charAt(0).toUpperCase();
        fallback.className = 'gallery-fallback';
        fallback.setAttribute('aria-label', example.name);
        thumbContainer.appendChild(fallback);
      }

      // Add label
      const label = document.createElement('span');
      label.className = 'gallery-label';
      label.textContent = example.name;

      item.appendChild(thumbContainer);
      item.appendChild(label);

      // Click handler
      item.addEventListener('click', () => {
        this.onSelect(example.jsonl);
      });

      this.container.appendChild(item);
    }
  }

  getExamples(): GalleryExample[] {
    return [...this.examples];
  }

  getExampleByName(name: string): GalleryExample | undefined {
    return this.examples.find(e => e.name === name);
  }
}
