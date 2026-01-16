# Obsidian Plugin

The Pixelsrc plugin for Obsidian lets you render pixel art sprites directly in your notes.

## Features

- **Inline rendering** - View sprites as images in your notes
- **Live preview** - See sprites render as you type
- **Copy to clipboard** - Right-click any sprite to copy as PNG
- **Configurable display** - Adjust scale, transparency background, and more

## Installation

### From Community Plugins (Recommended)

1. Open **Settings > Community Plugins**
2. Disable **Safe Mode** if enabled
3. Click **Browse** and search for "PixelSrc"
4. Click **Install**, then **Enable**

### Manual Installation

1. Download the latest release from [GitHub Releases](https://github.com/pixelsrc/obsidian-pixelsrc/releases)
2. Extract files to your vault's `.obsidian/plugins/pixelsrc/` directory
3. Reload Obsidian
4. Enable the plugin in **Settings > Community Plugins**

## Usage

Create a code block with language `pixelsrc` or `pxl`:

~~~markdown
```pixelsrc
{"type":"sprite","name":"heart","palette":{"{_}":"#00000000","{r}":"#FF0000"},"grid":["{_}{r}{r}{_}{r}{r}{_}","{r}{r}{r}{r}{r}{r}{r}","{_}{r}{r}{r}{r}{r}{_}","{_}{_}{r}{r}{r}{_}{_}","{_}{_}{_}{r}{_}{_}{_}"]}
```
~~~

The sprite renders in both Reading mode and Live Preview.

## Multi-Line Format

For better readability, use the multi-line format:

~~~markdown
```pxl
{"type": "palette", "name": "skin", "colors": {
  "{_}": "#00000000",
  "{s}": "#FFD5B8",
  "{h}": "#8B4513",
  "{e}": "#000000"
}}

{"type": "sprite", "name": "face", "palette": "skin", "grid": [
  "{_}{h}{h}{h}{h}{h}{h}{_}",
  "{h}{s}{s}{s}{s}{s}{s}{h}",
  "{h}{s}{e}{s}{s}{e}{s}{h}",
  "{h}{s}{s}{s}{s}{s}{s}{h}",
  "{h}{s}{s}{_}{_}{s}{s}{h}",
  "{h}{s}{s}{s}{s}{s}{s}{h}",
  "{_}{h}{s}{s}{s}{s}{h}{_}",
  "{_}{_}{h}{h}{h}{h}{_}{_}"
]}
```
~~~

## Settings

Access settings via **Settings > Community Plugins > PixelSrc**:

| Setting | Description | Default |
|---------|-------------|---------|
| Default Scale | Scale factor for rendered sprites (1-16x) | 4 |
| Show Warnings | Display rendering warnings below sprites | Off |
| Transparency Background | Show checkered background for transparent pixels | On |
| Live Preview | Show sprite preview while editing | On |

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Re-render current block | `Ctrl/Cmd + Shift + R` |
| Toggle Live Preview | `Ctrl/Cmd + Shift + P` |

## Tips for Note-Taking

### Game Design Documentation

Document game assets with their specifications:

~~~markdown
## Player Character

Scale: 16x16 pixels
Animation: 4-frame walk cycle

```pxl
{"type":"sprite","name":"player","palette":{"{_}":"#0000","{body}":"#4169E1","{skin}":"#FFCC99"},"grid":["{_}{skin}{skin}{_}","{body}{body}{body}{body}","{_}{body}{body}{_}","{skin}{_}{_}{skin}"]}
```

States: idle, walk, jump, attack
~~~

### Mood Boards

Create visual reference collections:

~~~markdown
# Color Palette Exploration

## Warm Sunset
```pxl
{"type":"sprite","name":"sunset","palette":{"{a}":"#FF6B35","{b}":"#F7C59F","{c}":"#EFEFEF"},"grid":["{a}{a}{a}","{b}{b}{b}","{c}{c}{c}"]}
```

## Cool Ocean
```pxl
{"type":"sprite","name":"ocean","palette":{"{a}":"#1A535C","{b}":"#4ECDC4","{c}":"#F7FFF7"},"grid":["{a}{a}{a}","{b}{b}{b}","{c}{c}{c}"]}
```
~~~

### Tutorials and Guides

Embed sprites inline with explanations:

~~~markdown
# Creating a Simple Tree

Start with the trunk:
```pxl
{"type":"sprite","name":"trunk","palette":{"{_}":"#0000","{t}":"#8B4513"},"grid":["{_}{t}{t}{_}","{_}{t}{t}{_}","{_}{t}{t}{_}"]}
```

Then add the foliage...
~~~

## Troubleshooting

### Sprite Not Rendering

1. Check that the code block language is `pixelsrc` or `pxl`
2. Verify JSON syntax (look for missing quotes or commas)
3. Ensure the plugin is enabled
4. Try toggling Live Preview off and on

### Blurry Sprites

Increase the scale factor in settings. The default 4x works well for most sprites, but very small sprites (4x4 or 8x8) may benefit from 8x or higher.

### Performance Issues

For notes with many sprites:
- Consider using PNG images for finalized sprites
- Disable Live Preview for large documents
- Split sprite collections across multiple notes

## Building from Source

```bash
git clone https://github.com/pixelsrc/obsidian-pixelsrc
cd obsidian-pixelsrc
npm install
npm run build
```

Copy `main.js`, `manifest.json`, and `styles.css` to your vault's plugin directory.

## Related

- [WASM Module](wasm.md) - The rendering engine used by this plugin
- [Format Specification](../format/overview.md) - Complete JSONL format reference
