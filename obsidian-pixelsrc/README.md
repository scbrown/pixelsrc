# PixelSrc for Obsidian

Render [pixelsrc](https://github.com/pixelsrc/pixelsrc) pixel art sprites directly in your Obsidian notes.

## Features

- **Render pixelsrc code blocks as images** - View your pixel art inline in your notes
- **Live preview while editing** - See sprites render as you type in Live Preview mode
- **Copy sprites as PNG** - Right-click any sprite to copy it as a PNG image
- **Configurable display** - Adjust scale, transparency background, and more
- **Multiple language aliases** - Use either `pixelsrc` or `pxl` as the code block language

## Usage

Create a code block with language `pixelsrc` or `pxl`:

~~~markdown
```pixelsrc
{"type":"sprite","name":"heart","size":[7,5],"palette":{"_":"#00000000","r":"#FF0000"},"regions":{"r":{"union":[{"rect":[1,0,2,1]},{"rect":[4,0,2,1]},{"rect":[0,1,7,1]},{"rect":[1,2,5,1]},{"rect":[2,3,3,1]},{"points":[[3,4]]}],"z":0}}}
```
~~~

The sprite will render as an image in both reading mode and Live Preview.

### Multi-line Example

For more complex sprites, you can use multiple JSONL lines:

~~~markdown
```pxl
{"type":"palette","name":"skin","colors":{"_":"#00000000","s":"#FFD5B8","h":"#8B4513","e":"#000000"}}
{"type":"sprite","name":"face","size":[8,8],"palette":"skin","regions":{"h":{"union":[{"rect":[1,0,6,1]},{"points":[[0,1],[7,1],[0,2],[7,2],[0,3],[7,3],[0,4],[7,4],[0,5],[7,5]]},{"points":[[1,6],[6,6]]},{"rect":[2,7,4,1]}],"z":0},"s":{"union":[{"rect":[1,1,6,5]},{"rect":[2,6,4,1]}],"z":1},"e":{"points":[[2,2],[5,2]],"z":2}}}
```
~~~

## Settings

Access settings via **Settings > Community Plugins > PixelSrc**:

| Setting | Description | Default |
|---------|-------------|---------|
| **Default Scale** | Scale factor for rendered sprites (1-16x) | 4 |
| **Show Warnings** | Display rendering warnings below sprites | Off |
| **Transparency Background** | Show checkered background for transparent pixels | On |
| **Live Preview** | Show sprite preview while editing (requires restart) | On |

## Installation

### From Community Plugins (Recommended)

1. Open **Settings > Community Plugins**
2. Disable **Safe Mode** if enabled
3. Click **Browse** and search for "PixelSrc"
4. Click **Install**, then **Enable**

### Manual Installation

1. Download the latest release from [GitHub Releases](https://github.com/pixelsrc/obsidian-pixelsrc/releases)
2. Extract `main.js`, `manifest.json`, and `styles.css` to your vault's `.obsidian/plugins/pixelsrc/` directory
3. Reload Obsidian
4. Enable the plugin in **Settings > Community Plugins**

### From Source

```bash
git clone https://github.com/pixelsrc/obsidian-pixelsrc
cd obsidian-pixelsrc
npm install
npm run build
```

Copy `main.js`, `manifest.json`, and `styles.css` to your vault's plugin directory.

## PixelSrc Format

PixelSrc uses a JSONL (JSON Lines) format to define pixel art:

```json
{"type":"sprite","name":"dot","size":[1,1],"palette":{"x":"#FF0000"},"regions":{"x":{"rect":[0,0,1,1],"z":0}}}
```

Key concepts:
- **Sprites** define pixel regions using shapes (rect, points, union)
- **Palettes** map tokens to colors (can be inline or referenced)
- **Regions** map token names to shape definitions with z-order
- **Transparency** uses `#00000000` or similar RGBA with alpha=0

For complete documentation on the pixelsrc format, see the [PixelSrc Specification](https://github.com/pixelsrc/pixelsrc).

## Links

- [PixelSrc Documentation](https://github.com/pixelsrc/pixelsrc)
- [Report Issues](https://github.com/pixelsrc/obsidian-pixelsrc/issues)

## License

MIT
