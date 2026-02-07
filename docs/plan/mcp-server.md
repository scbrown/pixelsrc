# MCP Server for Pixelsrc

**Goal:** Expose pixelsrc capabilities as MCP (Model Context Protocol) tools so Claude and other AI models can render, validate, explain, and manipulate pixel art directly. Add MCP Resources for context injection and MCP Prompts for guided workflows.

**Status:** Planning

**Depends on:** Phase 15 (AI Tools), Phase 20 (Build System), Phase 21 (mdbook docs)

**Related beads:** TTP-q8jr (planning), label:`mcp-server` (10 implementation beads)

---

## Motivation

Pixelsrc is AI-first by design — the format is optimized for LLM generation. But today, AI models interact with pixelsrc only through CLI text output. An MCP server lets AI models:

1. **Render sprites directly** — generate `.pxl` source and immediately see the rendered PNG, enabling visual feedback loops
2. **Validate in-context** — check `.pxl` source for errors without leaving the conversation
3. **Browse palettes** — discover and inspect built-in palettes as structured data
4. **Get format context** — pull in the format spec and examples as Resources, not tool calls
5. **Use guided workflows** — Prompts that inject the right context for sprite creation, review, etc.

Every persona benefits. Sketchers get render-in-conversation. Game Devs get validate-in-CI-agent. Motion Designers get transform library browsing.

---

## Current State: What Exists

### LspAgentClient (src/lsp_agent_client.rs)
Synchronous API designed for AI agent integration:
- `verify_content()` → structured validation with diagnostics
- `get_completions()` → context-aware token completions
- `get_grid_position()` → coordinate info at cursor position
- `resolve_colors()` → CSS variable and color-mix() resolution
- `analyze_timing()` → timing function parsing and visualization

### CLI Commands (src/cli/mod.rs)
Full command set that maps directly to MCP tools:
- `render` — source → PNG with scale, antialias, atlas, nine-slice, GIF, spritesheet
- `validate` — lint/check `.pxl` files, JSON output
- `suggest` — fix suggestions (typo detection, row completion)
- `explain` — human-readable sprite description
- `diff` — semantic sprite comparison
- `import` — PNG → `.pxl` conversion
- `fmt` — format `.pxl` source
- `prime` — format guide text (brief/full/sections)
- `palettes` — list, show, export built-in palettes
- `analyze` — corpus metrics extraction
- `prompts` — GenAI prompt templates
- `agent-verify` — structured JSON verification for agents
- `build` — build all assets per pxl.toml
- `init` / `new` — project/asset scaffolding

### LSP Server Pattern (src/lsp/server.rs)
- `tower-lsp` over stdio with tokio async runtime
- Feature-gated: `#[cfg(feature = "lsp")]`
- Hidden CLI command: `pxl lsp`
- Same pattern we'll follow for MCP

### WASM (src/wasm.rs)
- `render_to_png(jsonl: &str) -> Vec<u8>` — proves source-string-in, PNG-out works
- Stateless, no filesystem dependency

---

## Design: Three-Level MCP Server

### Level 1: MCP Foundation + Tools

The core: `pxl mcp` subcommand that starts an MCP server over stdio, exposing pixelsrc CLI commands as MCP tools.

### Level 2: MCP Resources

Context that clients pull into the LLM conversation without tool calls: format spec, palette catalog, examples.

### Level 3: MCP Prompts

User-triggered workflows that inject the right context for common tasks: create sprite, create animation, review source.

---

## Level 1: MCP Tools

### Architecture

```
┌─────────────────────────────────────────────────┐
│  MCP Host (Claude Code / Claude Desktop / etc.)  │
│  ┌─────────────────────────────────────────────┐ │
│  │  MCP Client  ←→  JSON-RPC over stdio        │ │
│  └──────────────────┬──────────────────────────┘ │
└─────────────────────┼────────────────────────────┘
                      │ stdin/stdout
┌─────────────────────┼────────────────────────────┐
│  pxl mcp (subprocess)                             │
│  ┌──────────────────▼──────────────────────────┐ │
│  │  rmcp transport layer (stdio)                │ │
│  └──────────────────┬──────────────────────────┘ │
│  ┌──────────────────▼──────────────────────────┐ │
│  │  PixelsrcMcpServer (ServerHandler)           │ │
│  │  ┌────────────────────────────────────────┐  │ │
│  │  │ render         validate    suggest     │  │ │
│  │  │ explain        diff        import      │  │ │
│  │  │ format         prime       palettes    │  │ │
│  │  │ analyze        scaffold    show        │  │ │
│  │  └────────────────────────────────────────┘  │ │
│  │  pixelsrc library (parser, renderer, etc.)   │ │
│  └──────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────┘
```

### Binary Strategy

- `pxl mcp` subcommand, feature-gated behind `mcp` feature flag
- Mirrors `pxl lsp` pattern: hidden command, starts server on stdio
- Reuses all library code directly (no subprocess spawning)

### Dependency

```toml
[dependencies]
rmcp = { version = "0.14", features = ["server", "transport-io", "macros"], optional = true }
schemars = { version = "1.0", optional = true }

[features]
mcp = ["rmcp", "schemars", "tokio"]
```

`rmcp` is the official Rust MCP SDK with `#[tool]` proc macros. `schemars` generates JSON Schema for tool input types.

### Tool Input Model

All tools accept **both** a `source` string and a `path` file:

```json
{
  "source": "...",   // .pxl content as string (preferred)
  "path": "/abs/path/to/file.pxl"  // OR file path to read
}
```

If both provided, `source` wins. This makes tools work in conversation (source) and in project context (path).

### Tool Definitions

#### `pixelsrc_render`

Render `.pxl` source to PNG. Returns base64-encoded image.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | * | — | `.pxl` content to render |
| `path` | string | * | — | File path to read and render |
| `sprite` | string | no | — | Sprite name (if multiple) |
| `composition` | string | no | — | Composition name |
| `scale` | integer | no | 1 | Scale factor (1-16) |
| `antialias` | string | no | — | AA algorithm (scale2x, hq2x, xbr2x, etc.) |

*One of `source` or `path` required.

**Returns:** `image` content (base64 PNG) + `text` content (render summary: name, dimensions, colors).

#### `pixelsrc_validate`

Validate `.pxl` source for errors and warnings.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | * | — | `.pxl` content to validate |
| `path` | string | * | — | File path to validate |
| `strict` | boolean | no | false | Treat warnings as errors |

**Returns:** `text` content with structured JSON diagnostics (errors, warnings, line numbers, suggestions).

#### `pixelsrc_suggest`

Suggest fixes for `.pxl` source (typo detection, row completion, missing tokens).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | * | — | `.pxl` content to analyze |
| `path` | string | * | — | File path to analyze |

**Returns:** `text` content with JSON suggestions (type, location, fix, confidence).

#### `pixelsrc_explain`

Describe sprites and objects in human-readable format with resolved colors.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | * | — | `.pxl` content to explain |
| `path` | string | * | — | File path to explain |
| `name` | string | no | — | Specific object name |

**Returns:** `text` content with semantic description (dimensions, colors, regions, palette).

#### `pixelsrc_diff`

Compare two sprites semantically.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source_a` | string | yes | — | First `.pxl` source |
| `source_b` | string | yes | — | Second `.pxl` source |
| `sprite` | string | no | — | Specific sprite to compare |

**Returns:** `text` content with JSON diff (added/removed/changed tokens, palette changes, size changes).

#### `pixelsrc_import`

Convert PNG image to `.pxl` format.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `image` | string | * | — | Base64-encoded PNG |
| `path` | string | * | — | Path to PNG file |
| `max_colors` | integer | no | 16 | Max palette colors (2-256) |
| `name` | string | no | — | Sprite name |
| `analyze` | boolean | no | false | Enable role/relationship inference |

**Returns:** `text` content with generated `.pxl` source.

#### `pixelsrc_format`

Format `.pxl` source for readability.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | yes | — | `.pxl` content to format |

**Returns:** `text` content with formatted `.pxl` source.

#### `pixelsrc_prime`

Return the pixelsrc format guide for context injection.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `brief` | boolean | no | false | Condensed version (~2000 tokens) |
| `section` | string | no | "full" | Section: format, examples, tips, full |

**Returns:** `text` content with format documentation.

#### `pixelsrc_palettes`

List and inspect built-in palettes.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `action` | string | no | "list" | Action: list, show |
| `name` | string | no | — | Palette name (for show) |

**Returns:** `text` content with palette data (names, colors, token mappings).

#### `pixelsrc_analyze`

Extract corpus metrics from `.pxl` source.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | * | — | `.pxl` content to analyze |
| `path` | string | * | — | File or directory to analyze |

**Returns:** `text` content with JSON metrics (sprite counts, token stats, co-occurrence).

#### `pixelsrc_scaffold`

Generate skeleton `.pxl` structures from parameters.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `asset_type` | string | yes | — | Type: sprite, animation, palette, composition |
| `name` | string | yes | — | Asset name |
| `palette` | string | no | — | Palette to use |
| `size` | array | no | [16,16] | Sprite dimensions [w, h] |

**Returns:** `text` content with generated `.pxl` skeleton source.

---

## Level 2: MCP Resources

Resources provide context that clients can inject into conversations without tool calls. The AI (or user) selects which resources to include.

### Resource Definitions

#### Static Resources

| URI | Description | MIME Type |
|-----|-------------|-----------|
| `pixelsrc://format-spec` | Complete `.pxl` format reference | text/markdown |
| `pixelsrc://format-brief` | Condensed format guide (~2000 tokens) | text/markdown |
| `pixelsrc://palettes` | JSON catalog of all built-in palette names and color counts | application/json |

#### Resource Templates (Dynamic)

| URI Template | Description | MIME Type |
|--------------|-------------|-----------|
| `pixelsrc://palette/{name}` | Full palette definition with all colors | application/json |
| `pixelsrc://example/{name}` | Example `.pxl` file (coin, hero, walk_cycle, etc.) | text/plain |
| `pixelsrc://template/{type}` | GenAI prompt template (character, item, tileset, animation) | text/markdown |

### Why Resources Matter

Without Resources, the AI must call `pixelsrc_prime` as a tool before generating any `.pxl` code. With Resources, the client pre-loads the format spec into context — the AI already knows the syntax before the user asks for anything. This is especially valuable for Claude Desktop where Resources appear in the sidebar.

---

## Level 3: MCP Prompts

Prompts are user-facing workflow templates that appear as selectable options in the client UI.

### Prompt Definitions

#### `create_sprite`

Guided sprite creation workflow.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `description` | string | yes | What the sprite should look like |
| `size` | string | no | Dimensions (e.g., "16x16", "32x32") |
| `palette` | string | no | Built-in palette name or "custom" |
| `style` | string | no | Art style hint (retro, modern, minimal) |

**Returns messages:**
1. System: Format spec + palette data (injected as resource)
2. User: "Create a {size} pixel art sprite of {description} using the {palette} palette"

#### `create_animation`

Guided animation creation workflow.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `description` | string | yes | What the animation should show |
| `frames` | integer | no | Number of frames |
| `fps` | integer | no | Frame rate |

**Returns messages:**
1. System: Format spec + animation examples
2. User: "Create a {frames}-frame animation at {fps}fps showing {description}"

#### `review_pxl`

Review existing `.pxl` source for improvements.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `source` | string | yes | The `.pxl` content to review |

**Returns messages:**
1. System: Format spec + validation rules + common mistakes
2. User: "Review this pixelsrc file and suggest improvements: {source}"

#### `pixel_art_guide`

Style guide and palette recommendations for a genre.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `genre` | string | yes | Game genre (RPG, platformer, puzzle, etc.) |

**Returns messages:**
1. System: Palette catalog + genre-specific tips
2. User: "Recommend palettes and art style guidelines for a {genre} game"

---

## Persona Impact Analysis

### Sketcher
**High value.** Render-in-conversation is transformative. Generate `.pxl` source and immediately see the sprite without leaving the chat. Format spec as a Resource means better first-attempt quality.

### Pixel Artist
**Medium value.** Palette browsing via Resources helps discover colors. Validate + suggest tools catch mistakes. Import tool converts reference PNGs.

### Animator
**Medium-High value.** Render animations (GIF/spritesheet) in conversation. Explain tool helps understand complex animation setups. Prompts guide frame creation.

### Motion Designer
**Medium value.** Analyze and explain tools help debug transform chains. Less direct value since motion design is more about code than conversation.

### Game Developer
**High value.** Validate in CI agents, analyze corpus metrics, scaffold new assets programmatically. Full build integration via MCP lets game dev AI agents manage the asset pipeline.

---

## Parsing Edge Cases

### 1. Large Source Strings
Tool calls may include entire project files. The parser already streams line-by-line (`parse_stream`), so memory isn't a concern. But MCP has no defined max message size — document a practical limit (e.g., 1MB source string).

### 2. Binary Data (Render Output, Import Input)
MCP supports `image` content type with base64 encoding. Render returns base64 PNG. Import accepts base64 PNG. This adds ~33% size overhead but is standard.

### 3. File Path Security
When `path` parameter is used, validate that the path is within the project root or an allowed directory. Don't blindly read arbitrary filesystem paths — this is an MCP server, not a shell.

### 4. Concurrent Tool Calls
MCP clients may call multiple tools concurrently. The server is stateless per-call (no shared mutable state between tools), so this is safe. `rmcp` with tokio handles concurrency.

### 5. Missing Project Context
Tools that accept `path` may need `pxl.toml` context for cross-file references (import system). Without it, fall back to single-file mode. Don't error — degrade gracefully.

---

## Implementation Plan

### Level 1: MCP Foundation + Tools

#### MCP-1: Server Scaffold + `pxl mcp` Command (`TTP-u3rxu`)

New `src/mcp/` module with `PixelsrcMcpServer` struct. Add `mcp` feature flag to Cargo.toml. Wire `pxl mcp` subcommand to start the server over stdio. Implement `initialize` handshake with server info and capabilities.

#### MCP-2: Render Tool (`TTP-jt6mg`)

Implement `pixelsrc_render` — parse `.pxl` source (or read file), render to PNG, return as base64 `image` content. Handle scale, sprite selection, antialias options.

#### MCP-3: Validate + Suggest Tools (`TTP-3zlvf`)

Implement `pixelsrc_validate` and `pixelsrc_suggest` — reuse existing validator and suggestion engine. Return structured JSON diagnostics.

#### MCP-4: Explain + Diff Tools (`TTP-0ima7`)

Implement `pixelsrc_explain` and `pixelsrc_diff` — reuse existing explain and diff logic. Return human-readable and JSON descriptions.

#### MCP-5: Import Tool (`TTP-xw4y6`)

Implement `pixelsrc_import` — accept base64 PNG or file path, convert to `.pxl` source. Handle max_colors, name, analyze options.

#### MCP-6: Format + Prime + Palettes + Analyze + Scaffold Tools (`TTP-pybpe`)

Implement remaining tools: `pixelsrc_format`, `pixelsrc_prime`, `pixelsrc_palettes`, `pixelsrc_analyze`, `pixelsrc_scaffold`. These are thin wrappers around existing CLI logic.

#### MCP-7: Integration Tests + Claude Code Config (`TTP-lm83s`)

End-to-end tests: start server, send JSON-RPC messages, verify responses. Document Claude Code configuration (`claude_desktop_config.json` or `.mcp.json`). Test with actual Claude Code session.

### Level 2: MCP Resources

#### MCP-8: Static Resources (`TTP-osg1v`)

Implement `resources/list` and `resources/read` for format-spec, format-brief, and palettes catalog.

#### MCP-9: Resource Templates (`TTP-48nff`)

Implement `resources/templates/list` and dynamic `resources/read` for palette/{name}, example/{name}, template/{type}.

### Level 3: MCP Prompts

#### MCP-10: Prompt Templates (`TTP-sk5f3`)

Implement `prompts/list` and `prompts/get` for create_sprite, create_animation, review_pxl, pixel_art_guide.

---

## Task Dependency Diagram

```
                     MCP SERVER TASK FLOW
═════════════════════════════════════════════════════════════════

LEVEL 1 (Foundation + Tools)
┌───────────────────────────────────────────────────────────────┐
│  MCP-1 ──→ MCP-2 ──→ MCP-3 ──→ MCP-4                         │
│  Scaffold   Render    Validate   Explain                      │
│             Tool      +Suggest   +Diff                        │
│                                                               │
│             MCP-5 (independent after MCP-1)                   │
│             Import                                            │
│                                                               │
│             MCP-6 (independent after MCP-1)                   │
│             Format+Prime+Palettes+Analyze+Scaffold            │
│                                                               │
│  MCP-7 (after all tools)                                      │
│  Integration Tests + Claude Code Config                       │
└───────────────────────────────────────────────────────────────┘

LEVEL 2 (Resources)  ← after MCP-1
┌───────────────────────────────────────────────────────────────┐
│  MCP-8 ──→ MCP-9                                              │
│  Static     Resource                                          │
│  Resources  Templates                                         │
└───────────────────────────────────────────────────────────────┘

LEVEL 3 (Prompts)  ← after MCP-1
┌───────────────────────────────────────────────────────────────┐
│  MCP-10                                                       │
│  Prompt Templates                                             │
└───────────────────────────────────────────────────────────────┘

CRITICAL PATH: MCP-1 → MCP-2 → MCP-7 (foundation + render + tests)
HIGHEST VALUE: MCP-2 (render) — visual feedback loop for AI generation
```

---

## Success Criteria

### Level 1 (Foundation + Tools)
1. `pxl mcp` starts an MCP server that Claude Code can connect to
2. AI can generate `.pxl` source and see the rendered PNG in conversation
3. AI can validate `.pxl` source and get structured error diagnostics
4. AI can get fix suggestions for broken `.pxl` files
5. AI can explain sprites in natural language
6. AI can compare two sprites semantically
7. AI can convert PNG to `.pxl` source
8. AI can format `.pxl` source
9. AI can browse built-in palettes
10. All tools work with both source strings and file paths
11. Integration tests pass with real JSON-RPC messages

### Level 2 (Resources)
12. Format spec available as a Resource (no tool call needed)
13. Individual palettes browsable via URI template
14. Example files available as Resources
15. Claude Desktop shows Resources in sidebar

### Level 3 (Prompts)
16. "Create sprite" prompt injects format spec + palette context
17. "Create animation" prompt injects animation examples
18. "Review pxl" prompt includes validation rules
19. Prompts appear as selectable options in Claude Desktop

---

## Open Questions

1. **Streaming HTTP transport:** Should Level 1 include HTTP/SSE transport for remote use cases (e.g., web-based AI agents)? Recommendation: No — stdio first, HTTP as a future follow-up.

2. **Project-aware mode:** When `pxl mcp` starts in a directory with `pxl.toml`, should it auto-load the project registry for cross-file references? Recommendation: Yes, opportunistically, but don't require it.

3. **Watch mode integration:** Should Resource subscriptions trigger on file changes (for projects)? Recommendation: Defer to Level 2 — implement basic Resources first, subscriptions later.

4. **Tool naming convention:** `pixelsrc_render` vs `render` vs `pixelsrc/render`? MCP tool names allow `[A-Za-z0-9_\-.]`. Recommendation: Use `pixelsrc_` prefix to avoid conflicts with other MCP servers.
