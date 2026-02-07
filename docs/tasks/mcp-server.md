# MCP Server — Task Breakdown

**Plan:** [docs/plan/mcp-server.md](../plan/mcp-server.md)
**Epic:** TTP-q8jr

---

## Level 1: MCP Foundation + Tools

### MCP-1: Server Scaffold + `pxl mcp` Command (`TTP-u3rxu`)

**Goal:** Minimal MCP server that starts, handshakes, and responds to `tools/list`.

**Work:**
- Add `rmcp` (official Rust MCP SDK) and `schemars` to Cargo.toml behind `mcp` feature flag
- Create `src/mcp/mod.rs` with `PixelsrcMcpServer` struct implementing `rmcp::ServerHandler`
- Implement `get_info()` returning server name, version, protocol version, capabilities (tools)
- Add `Mcp` variant to `Commands` enum in `src/cli/mod.rs`, feature-gated with `#[cfg(feature = "mcp")]`
- Wire `pxl mcp` to start the server: `PixelsrcMcpServer::new().serve(stdio()).await`
- Add `src/cli/mcp.rs` with `run_mcp()` entrypoint
- Register one placeholder tool (e.g., `pixelsrc_prime`) to verify the full stack works

**Acceptance criteria:**
- `pxl mcp` starts and completes JSON-RPC `initialize` handshake
- `tools/list` returns at least one tool definition with valid JSON Schema
- `tools/call` on the placeholder tool returns a text response
- `cargo build --features mcp` succeeds; `cargo build` (no mcp feature) still succeeds
- Unit test for server info and tool list

**Files:** `Cargo.toml`, `src/mcp/mod.rs`, `src/mcp/server.rs`, `src/cli/mod.rs`, `src/cli/mcp.rs`, `src/main.rs`

---

### MCP-2: Render Tool (`TTP-jt6mg`)

**Goal:** AI generates `.pxl` source and sees the rendered PNG in conversation.

**Work:**
- Implement `pixelsrc_render` tool with `#[tool]` macro
- Input struct: `source: Option<String>`, `path: Option<String>`, `sprite: Option<String>`, `composition: Option<String>`, `scale: Option<u8>`, `antialias: Option<String>`
- Parse source string (or read file) using `parse_stream()`
- Build palette/sprite registries, resolve, render to PNG bytes
- Encode PNG as base64, return as `Content::image(base64, "image/png")` + `Content::text(summary)`
- Handle errors gracefully: parse errors, missing sprite, render failures → `isError: true` with diagnostic text
- Reuse logic from `src/wasm.rs` `render_to_png()` and `src/cli/render.rs`

**Acceptance criteria:**
- Tool call with `source` containing valid `.pxl` returns a base64 PNG image
- Tool call with `path` pointing to a `.pxl` file returns rendered image
- `sprite` parameter selects specific sprite from multi-sprite source
- `scale` parameter produces upscaled output
- Parse errors return `isError: true` with error details
- Missing sprite name returns clear error
- Integration test: send `tools/call` JSON-RPC, verify image content in response

**Files:** `src/mcp/tools/render.rs`, `src/mcp/tools/mod.rs`

---

### MCP-3: Validate + Suggest Tools (`TTP-3zlvf`)

**Goal:** AI can lint `.pxl` source and get fix suggestions in structured format.

**Work:**
- Implement `pixelsrc_validate` tool
  - Input: `source`/`path`, `strict: Option<bool>`
  - Run validator, collect errors/warnings with line numbers
  - Return JSON diagnostics as text content
- Implement `pixelsrc_suggest` tool
  - Input: `source`/`path`
  - Run suggestion engine (typo detection via Levenshtein, row completion, missing tokens)
  - Return JSON suggestions as text content
- Reuse `src/cli/validate.rs` logic (run_validate, run_agent_verify)
- Reuse `src/cli/explain.rs` logic (run_suggest)

**Acceptance criteria:**
- Validate returns structured errors with line numbers for invalid source
- Validate returns clean result for valid source
- Strict mode escalates warnings to errors
- Suggest returns typo corrections with confidence scores
- Suggest returns row completion hints
- Both tools work with source strings and file paths
- Unit tests for valid, invalid, and warning cases

**Files:** `src/mcp/tools/validate.rs`, `src/mcp/tools/suggest.rs`

---

### MCP-4: Explain + Diff Tools (`TTP-0ima7`)

**Goal:** AI can describe sprites semantically and compare two versions.

**Work:**
- Implement `pixelsrc_explain` tool
  - Input: `source`/`path`, `name: Option<String>`
  - Parse source, resolve palettes, generate human-readable description
  - Include: dimensions, color count, region breakdown, palette details
  - Reuse `src/cli/explain.rs` logic (run_explain)
- Implement `pixelsrc_diff` tool
  - Input: `source_a: String`, `source_b: String`, `sprite: Option<String>`
  - Parse both, compare semantically
  - Return: added/removed/changed tokens, palette changes, size changes
  - Reuse `src/cli/explain.rs` logic (run_diff)

**Acceptance criteria:**
- Explain returns human-readable description of sprite structure
- Explain with `name` parameter targets specific sprite
- Diff identifies added, removed, and changed pixels between two sprites
- Diff detects palette changes
- Both tools return JSON-structured output
- Unit tests for single sprite, multi-sprite, and diff cases

**Files:** `src/mcp/tools/explain.rs`, `src/mcp/tools/diff.rs`

---

### MCP-5: Import Tool (`TTP-xw4y6`)

**Goal:** AI can convert PNG images to `.pxl` source within conversation.

**Work:**
- Implement `pixelsrc_import` tool
  - Input: `image: Option<String>` (base64 PNG), `path: Option<String>`, `max_colors: Option<usize>`, `name: Option<String>`, `analyze: Option<bool>`
  - Decode base64 PNG (or read file), run color quantization, generate `.pxl` source
  - Reuse `src/cli/import.rs` logic (run_import) — may need to refactor to accept bytes instead of file path
- Handle base64 decoding errors, invalid PNG, oversized images

**Acceptance criteria:**
- Base64 PNG input produces valid `.pxl` source output
- File path input produces valid `.pxl` source output
- `max_colors` controls palette size
- `name` overrides auto-derived sprite name
- `analyze` enables role/relationship inference in output
- Invalid base64 or corrupt PNG returns clear error
- Integration test with a small test PNG

**Files:** `src/mcp/tools/import.rs`

---

### MCP-6: Format + Prime + Palettes + Analyze + Scaffold Tools (`TTP-pybpe`)

**Goal:** Complete the tool set with remaining CLI command wrappers.

**Work:**
- Implement `pixelsrc_format` tool
  - Input: `source: String`
  - Run formatter, return formatted source
  - Reuse `src/cli/validate.rs` (run_fmt)
- Implement `pixelsrc_prime` tool
  - Input: `brief: Option<bool>`, `section: Option<String>`
  - Return format guide text
  - Reuse `src/cli/info.rs` (run_prime)
- Implement `pixelsrc_palettes` tool
  - Input: `action: Option<String>` (list/show), `name: Option<String>`
  - Return palette data as JSON
  - Reuse `src/cli/info.rs` (run_palettes)
- Implement `pixelsrc_analyze` tool
  - Input: `source`/`path`
  - Return corpus metrics as JSON
  - Reuse `src/cli/validate.rs` (run_analyze)
- Implement `pixelsrc_scaffold` tool
  - Input: `asset_type: String`, `name: String`, `palette: Option<String>`, `size: Option<[u32; 2]>`
  - Generate skeleton `.pxl` source
  - Reuse `src/cli/build.rs` (run_new) — may need to refactor to return string instead of writing file

**Acceptance criteria:**
- Format tool reformats messy `.pxl` source correctly
- Prime tool returns format guide (brief and full modes)
- Palettes list returns all built-in palette names
- Palettes show returns full color mapping for named palette
- Analyze returns sprite counts, token stats, co-occurrence data
- Scaffold generates valid `.pxl` skeleton for each asset type (sprite, animation, palette, composition)
- All tools return structured output suitable for AI consumption

**Files:** `src/mcp/tools/format.rs`, `src/mcp/tools/prime.rs`, `src/mcp/tools/palettes.rs`, `src/mcp/tools/analyze.rs`, `src/mcp/tools/scaffold.rs`

---

### MCP-7: Integration Tests + Claude Code Config (`TTP-lm83s`)

**Goal:** Verify the full stack end-to-end and document client configuration.

**Work:**
- Write integration tests that spawn `pxl mcp` as a subprocess
  - Send `initialize` → verify handshake
  - Send `tools/list` → verify all 11 tools listed with valid schemas
  - Send `tools/call` for each tool → verify correct response format
  - Test error cases: invalid tool name, malformed params, missing required fields
- Create `.mcp.json` example for Claude Code configuration
- Create `claude_desktop_config.json` example for Claude Desktop
- Document setup in plan doc or README section
- Test with actual Claude Code session (manual verification)

**Acceptance criteria:**
- Integration test suite passes: handshake, tool list, all 11 tool calls
- Error cases handled gracefully (no panics, clear error messages)
- Claude Code config example works when placed in project root
- Claude Desktop config example works for local server
- Documentation covers: installation, configuration, available tools, example usage

**Files:** `tests/mcp_integration.rs`, `examples/mcp/mcp.json`, `examples/mcp/claude_desktop_config.json`

---

## Level 2: MCP Resources

### MCP-8: Static Resources (`TTP-osg1v`)

**Goal:** Expose format spec and palette catalog as MCP Resources.

**Work:**
- Add `resources` capability to server info
- Implement `resources/list` handler returning static resource entries
- Implement `resources/read` handler for:
  - `pixelsrc://format-spec` → full format reference (from embedded primer docs)
  - `pixelsrc://format-brief` → condensed guide (~2000 tokens)
  - `pixelsrc://palettes` → JSON array of palette names + color counts
- Resources are text content with appropriate MIME types

**Acceptance criteria:**
- `resources/list` returns 3 static resources with URIs, names, descriptions
- `resources/read` for each URI returns correct content
- Format spec matches `pxl prime --section full` output
- Palette catalog includes all built-in palettes with color counts
- Integration tests for list and read operations

**Files:** `src/mcp/resources/mod.rs`, `src/mcp/resources/static_resources.rs`

---

### MCP-9: Resource Templates (`TTP-48nff`)

**Goal:** Dynamic Resources for individual palettes, examples, and prompt templates.

**Work:**
- Implement `resources/templates/list` returning URI templates
- Implement dynamic `resources/read` for:
  - `pixelsrc://palette/{name}` → full palette JSON (all colors, token mappings)
  - `pixelsrc://example/{name}` → example `.pxl` file content (coin, hero, walk_cycle, etc.)
  - `pixelsrc://template/{type}` → GenAI prompt template (character, item, tileset, animation)
- Template names resolved from built-in palette registry and embedded example files

**Acceptance criteria:**
- `resources/templates/list` returns 3 URI templates
- `pixelsrc://palette/gameboy` returns the gameboy palette definition
- `pixelsrc://palette/nonexistent` returns clear error
- `pixelsrc://example/coin` returns the coin example `.pxl` source
- `pixelsrc://template/character` returns the character prompt template
- Integration tests for template resolution and error cases

**Files:** `src/mcp/resources/templates.rs`

---

## Level 3: MCP Prompts

### MCP-10: Prompt Templates (`TTP-sk5f3`)

**Goal:** User-facing workflow templates that inject correct context for common tasks.

**Work:**
- Add `prompts` capability to server info
- Implement `prompts/list` returning available prompts
- Implement `prompts/get` with argument substitution for:
  - `create_sprite` — args: description, size, palette, style → messages with format spec + palette data
  - `create_animation` — args: description, frames, fps → messages with format spec + animation examples
  - `review_pxl` — args: source → messages with validation rules + common mistakes
  - `pixel_art_guide` — args: genre → messages with palette recommendations + style tips
- Each prompt returns an array of messages (role + content) that can embed resources

**Acceptance criteria:**
- `prompts/list` returns 4 prompts with argument definitions
- `prompts/get` for each prompt returns well-structured messages
- Arguments are substituted into message content
- Optional arguments have sensible defaults
- Prompts appear as selectable options in Claude Desktop
- Integration tests for list and get operations

**Files:** `src/mcp/prompts/mod.rs`, `src/mcp/prompts/templates.rs`
