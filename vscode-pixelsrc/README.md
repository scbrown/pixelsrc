# Pixelsrc for VS Code

Syntax highlighting, LSP integration, and live preview for [Pixelsrc](https://github.com/scbrown/pixelsrc) `.pxl` pixel art files.

## Features

- **Syntax Highlighting** — TextMate grammar for `.pxl` files with semantic colorization of types, shapes, modifiers, roles, colors, and CSS functions
- **LSP Integration** — Real-time diagnostics, hover information, completions, document symbols, go-to-definition, and color picker via the `pxl lsp` server
- **Live Preview** — Side panel that renders sprites as you type with configurable scale and background
- **Color Decorators** — Inline color swatches next to hex color values in palettes
- **Export** — Export `.pxl` files to PNG directly from VS Code

## Requirements

- [pixelsrc](https://github.com/scbrown/pixelsrc) (`pxl`) binary on your PATH
- VS Code 1.75+

## Installation

### From VSIX (local)

```bash
cd vscode-pixelsrc
npm install
npm run compile
npx vsce package
code --install-extension pixelsrc-0.1.0.vsix
```

### From Marketplace

Search for "Pixelsrc" in the VS Code Extensions panel.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `pixelsrc.lsp.enabled` | `true` | Enable the language server |
| `pixelsrc.lsp.path` | `"pxl"` | Path to the `pxl` binary |
| `pixelsrc.preview.enabled` | `true` | Enable live preview panel |
| `pixelsrc.preview.scale` | `8` | Pixel scale factor for preview |
| `pixelsrc.preview.background` | `"checkerboard"` | Preview background: checkerboard, dark, light, transparent |
| `pixelsrc.colorDecorators.enabled` | `true` | Show inline color swatches |

## Commands

| Command | Description |
|---------|-------------|
| `Pixelsrc: Open Sprite Preview` | Open preview in current column |
| `Pixelsrc: Open Sprite Preview to the Side` | Open preview in side column |
| `Pixelsrc: Export as PNG` | Save current `.pxl` file as PNG |

## Development

```bash
cd vscode-pixelsrc
npm install
npm run watch     # Compile in watch mode
# Press F5 in VS Code to launch Extension Development Host
```

## License

MIT
