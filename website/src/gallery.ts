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

    for (const filename of EXAMPLE_FILES) {
      try {
        const response = await fetch(`/examples/${filename}`);
        if (response.ok) {
          const jsonl = await response.text();
          const name = filename.replace('.jsonl', '');
          this.examples.push({ name, jsonl: jsonl.trim() });
        }
      } catch (err) {
        console.warn(`Failed to load example: ${filename}`, err);
      }
    }

    this.render();
  }

  private render(): void {
    this.container.innerHTML = '';

    for (const example of this.examples) {
      const item = document.createElement('button');
      item.className = 'gallery-item';
      item.title = example.name;
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
        img.alt = example.name;
        thumbContainer.appendChild(img);
      } catch (err) {
        // Fallback to text if render fails
        const fallback = document.createElement('span');
        fallback.textContent = example.name.charAt(0).toUpperCase();
        fallback.className = 'gallery-fallback';
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
