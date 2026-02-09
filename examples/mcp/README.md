# Pixelsrc MCP Server Configuration

Configuration examples for using pixelsrc as an MCP (Model Context Protocol) server
with AI coding assistants.

## Prerequisites

Build pixelsrc with MCP support:

```bash
cargo build --release --features mcp
```

Ensure `pxl` is on your PATH, or use the full path in the config files below.

## Claude Code

Copy `mcp.json` to your project root as `.mcp.json`:

```bash
cp examples/mcp/mcp.json /path/to/your/project/.mcp.json
```

Claude Code will detect the server automatically when working in that project.

## Claude Desktop

Merge the contents of `claude_desktop_config.json` into your Claude Desktop config:

- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux:** `~/.config/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

## Available Tools

The MCP server exposes 11 tools:

| Tool | Description |
|------|-------------|
| `pixelsrc_render` | Render `.pxl` source to PNG (base64) |
| `pixelsrc_validate` | Validate `.pxl` source, return structured diagnostics |
| `pixelsrc_suggest` | Detect typos and suggest fixes |
| `pixelsrc_explain` | Describe `.pxl` objects in structured JSON |
| `pixelsrc_diff` | Compare two `.pxl` sources semantically |
| `pixelsrc_import` | Convert PNG image to `.pxl` source |
| `pixelsrc_format` | Format `.pxl` source for readability |
| `pixelsrc_prime` | Return the `.pxl` format guide |
| `pixelsrc_palettes` | List or show built-in palettes |
| `pixelsrc_analyze` | Extract corpus metrics from `.pxl` files |
| `pixelsrc_scaffold` | Generate skeleton `.pxl` structures |

## Resources

Static resources available via `resources/read`:

- `pixelsrc://format-spec` — Full format reference
- `pixelsrc://format-brief` — Condensed format guide (~2000 tokens)
- `pixelsrc://palettes` — Built-in palette catalog

Dynamic resource templates:

- `pixelsrc://palette/{name}` — Individual palette definition
- `pixelsrc://example/{name}` — Example `.pxl` files (coin, heart, hero, walk_cycle)
- `pixelsrc://template/{type}` — Prompt templates (character, item, tileset, animation)

## Prompts

Pre-built prompt templates:

- `create_sprite` — Generate a new pixel art sprite
- `create_animation` — Generate a sprite animation sequence
- `review_pxl` — Review `.pxl` source for issues
- `pixel_art_guide` — Get pixel art tips for a specific genre
