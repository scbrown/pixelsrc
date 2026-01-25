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
{
  type: "sprite",
  name: "heart",
  size: [7, 5],
  palette: { _: "transparent", r: "#FF0000" },
  regions: {
    r: {
      union: [
        { rect: [1, 0, 2, 1] },
        { rect: [4, 0, 2, 1] },
        { rect: [0, 1, 7, 1] },
        { rect: [1, 2, 5, 1] },
        { rect: [2, 3, 3, 1] },
        { rect: [3, 4, 1, 1] },
      ],
      z: 0,
    },
  },
}
```
~~~

The sprite renders in both Reading mode and Live Preview.

## Multi-Line Format

For better readability, use the multi-line format:

~~~markdown
```pxl
{
  type: "palette",
  name: "skin",
  colors: {
    _: "transparent",
    s: "#FFD5B8",
    h: "#8B4513",
    e: "#000000",
  },
}

{
  type: "sprite",
  name: "face",
  size: [8, 8],
  palette: "skin",
  regions: {
    h: {
      union: [
        { rect: [1, 0, 6, 1] },
        { points: [[0, 1], [7, 1], [0, 6], [7, 6]] },
        { rect: [2, 7, 4, 1] },
      ],
      z: 0,
    },
    s: {
      union: [
        { rect: [1, 1, 6, 5] },
        { rect: [2, 6, 4, 1] },
      ],
      z: 1,
    },
    e: { points: [[2, 2], [5, 2]], z: 2 },
  },
}
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
{
  type: "sprite",
  name: "player",
  size: [4, 4],
  palette: { _: "transparent", body: "#4169E1", skin: "#FFCC99" },
  regions: {
    skin: { rect: [1, 0, 2, 1], z: 0 },
    body: { rect: [0, 1, 4, 2], z: 0 },
  },
}
```

States: idle, walk, jump, attack
~~~

### Mood Boards

Create visual reference collections:

~~~markdown
# Color Palette Exploration

## Warm Sunset
```pxl
{
  type: "sprite",
  name: "sunset",
  size: [3, 3],
  palette: { a: "#FF6B35", b: "#F7C59F", c: "#EFEFEF" },
  regions: {
    a: { rect: [0, 0, 3, 1], z: 0 },
    b: { rect: [0, 1, 3, 1], z: 0 },
    c: { rect: [0, 2, 3, 1], z: 0 },
  },
}
```

## Cool Ocean
```pxl
{
  type: "sprite",
  name: "ocean",
  size: [3, 3],
  palette: { a: "#1A535C", b: "#4ECDC4", c: "#F7FFF7" },
  regions: {
    a: { rect: [0, 0, 3, 1], z: 0 },
    b: { rect: [0, 1, 3, 1], z: 0 },
    c: { rect: [0, 2, 3, 1], z: 0 },
  },
}
```
~~~

### Tutorials and Guides

Embed sprites inline with explanations:

~~~markdown
# Creating a Simple Tree

Start with the trunk:
```pxl
{
  type: "sprite",
  name: "trunk",
  size: [4, 3],
  palette: { _: "transparent", t: "#8B4513" },
  regions: {
    t: { rect: [1, 0, 2, 3], z: 0 },
  },
}
```

Then add the foliage...
~~~

## Troubleshooting

### Sprite Not Rendering

1. Check that the code block language is `pixelsrc` or `pxl`
2. Verify JSON5 syntax (look for missing quotes or commas)
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
- [Format Specification](../format/overview.md) - Complete format reference
