# Phase 17: LSP Support

**Goal:** Enable live validation, grid alignment assistance, and GenAI tooling for Pixelsrc files via Language Server Protocol

**Status:** Not Started

**Depends on:** Phase 0 (Core CLI), Phase 15 (AI Tools - Validator), Phase 22 (CSS Integration - for Wave 6)

---

## Scope

Phase 17 adds:
- `pxl lsp` - Hidden command that starts LSP server over stdio
- Real-time diagnostics from existing `Validator`
- Grid alignment features specifically designed for GenAI assistance
- Completion suggestions for tokens and structure
- Hover information showing grid coordinates and token details
- **CSS-aware features** (Wave 6, requires Phase 22):
  - CSS color previews with computed `hsl()`, `oklch()`, `color-mix()` values
  - CSS variable completions and hover resolution
  - Timing function visualization
  - Transform explanation

**Not in scope:** Full IDE plugin development (users can use Generic LSP Client), visual sprite previews in editor

---

## Motivation

### For Humans
IDE integration provides immediate feedback on pixelsrc files - syntax errors, undefined tokens, and row mismatches highlighted as you type.

### For GenAI Agents
The LSP becomes a **structured verification API** that agents can use programmatically:

1. **Grid Alignment Verification** - The #1 failure mode for AI-generated sprites is inconsistent row lengths. LSP provides machine-readable diagnostics before the agent declares "done."

2. **Coordinate Awareness** - Agents can query "what token is at position (x,y)?" and "how wide should this row be?" to maintain grid consistency.

3. **Self-Correction Loop** - Agent drafts content → sends to LSP → receives diagnostics → fixes issues → re-validates. Eliminates syntax errors before rendering.

4. **Context Discovery** - Agents use `textDocument/documentSymbol` to discover project structure and `textDocument/completion` to ground generation in existing palettes.

---

## Task Dependency Diagram

```
                          PHASE 17 TASK FLOW
═══════════════════════════════════════════════════════════════════

PREREQUISITES
┌─────────────────────────────────────────────────────────────────┐
│     Phase 0 (CLI)    +    Phase 15 (Validator exists)          │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 1 (Foundation)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   17.1 LSP Server Infrastructure                         │   │
│  │   - tower-lsp integration                                │   │
│  │   - Basic initialize/shutdown                            │   │
│  │   - Hidden `pxl lsp` command                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (Validation Bridge)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   17.2 Diagnostics Integration                           │   │
│  │   - Wire Validator to LSP diagnostics                    │   │
│  │   - didOpen/didChange handlers                           │   │
│  │   - Map ValidationIssue → Diagnostic                     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 3 (Grid Alignment - GenAI Focus)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────┐  ┌──────────────────────────┐  │
│  │   17.3                     │  │   17.4                   │  │
│  │  Grid Coordinate Hover     │  │  Row Length Diagnostics  │  │
│  │  (show x,y at cursor)      │  │  (expected vs actual)    │  │
│  └────────────────────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 4 (Completions & Symbols)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────┐  ┌──────────────────────────┐  │
│  │   17.5                     │  │   17.6                   │  │
│  │  Token Completions         │  │  Document Symbols        │  │
│  │  (suggest {tokens})        │  │  (palette, sprite list)  │  │
│  └────────────────────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 5 (GenAI Agent Bridge)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   17.7 Agent Bridge Tool                                 │   │
│  │   - Simple CLI wrapper for LSP                           │   │
│  │   - `pxl agent-verify <content>` command                 │   │
│  │   - JSON output for agent consumption                    │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 6 (CSS-Aware Features - Requires Phase 22)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────┐  ┌──────────────────────────┐  │
│  │   17.8                     │  │   17.9                   │  │
│  │  CSS Color Provider        │  │  CSS Variable Support    │  │
│  │  (swatches, color-mix)     │  │  (completions, hover)    │  │
│  └────────────────────────────┘  └──────────────────────────┘  │
│                                                                 │
│  ┌────────────────────────────┐  ┌──────────────────────────┐  │
│  │   17.10                    │  │   17.11                  │  │
│  │  Timing Function Viz       │  │  Transform Explainer     │  │
│  │  (easing curve preview)    │  │  (describe effect)       │  │
│  └────────────────────────────┘  └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 7 (Agent CSS Bridge)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   17.12 Agent CSS Tools                                  │   │
│  │   - `pxl agent-verify --resolve-colors`                  │   │
│  │   - Show computed values for var(), color-mix()          │   │
│  │   - JSON with resolved color hex values                  │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 17.1                        (1 task, foundation)       │
│  Wave 2: 17.2                        (1 task, needs 17.1)       │
│  Wave 3: 17.3 + 17.4                 (2 tasks in parallel)      │
│  Wave 4: 17.5 + 17.6                 (2 tasks in parallel)      │
│  Wave 5: 17.7                        (1 task, agent tooling)    │
│  Wave 6: 17.8 + 17.9 + 17.10 + 17.11 (4 tasks, needs Phase 22)  │
│  Wave 7: 17.12                       (1 task, agent CSS tools)  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 17.1: LSP Server Infrastructure

**Wave:** 1 (foundation)

Set up the basic LSP server framework.

**Deliverables:**

1. Add dependencies to `Cargo.toml`:
   ```toml
   tower-lsp = "0.20"
   tokio = { version = "1", features = ["full"] }
   ```

2. Create `src/lsp.rs`:
   ```rust
   use tower_lsp::jsonrpc::Result;
   use tower_lsp::lsp_types::*;
   use tower_lsp::{Client, LanguageServer, LspService, Server};

   pub struct PixelsrcLsp {
       client: Client,
   }

   impl PixelsrcLsp {
       pub fn new(client: Client) -> Self {
           Self { client }
       }
   }

   #[tower_lsp::async_trait]
   impl LanguageServer for PixelsrcLsp {
       async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
           Ok(InitializeResult {
               capabilities: ServerCapabilities {
                   text_document_sync: Some(TextDocumentSyncCapability::Kind(
                       TextDocumentSyncKind::FULL,
                   )),
                   ..Default::default()
               },
               ..Default::default()
           })
       }

       async fn shutdown(&self) -> Result<()> {
           Ok(())
       }
   }

   pub async fn run_server() {
       let stdin = tokio::io::stdin();
       let stdout = tokio::io::stdout();

       let (service, socket) = LspService::new(PixelsrcLsp::new);
       Server::new(stdin, stdout, socket).serve(service).await;
   }
   ```

3. Add hidden CLI command in `src/cli.rs`:
   ```rust
   /// Start LSP server (for IDE integration)
   #[command(hide = true)]
   Lsp,
   ```

4. Wire up in main:
   ```rust
   Commands::Lsp => {
       tokio::runtime::Runtime::new()
           .unwrap()
           .block_on(lsp::run_server());
   }
   ```

**Verification:**
```bash
cargo build
./target/release/pxl lsp --help  # Should not appear in help (hidden)

# Test server starts (will hang waiting for input, Ctrl+C to exit)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' | ./target/release/pxl lsp
```

**Dependencies:** Phase 0 complete

---

### Task 17.2: Diagnostics Integration

**Wave:** 2 (needs 17.1)

Wire the existing `Validator` to produce LSP diagnostics.

**Deliverables:**

1. Add document state tracking to `PixelsrcLsp`:
   ```rust
   use std::sync::RwLock;
   use std::collections::HashMap;

   pub struct PixelsrcLsp {
       client: Client,
       documents: RwLock<HashMap<Url, String>>,
   }
   ```

2. Implement `did_open` and `did_change`:
   ```rust
   async fn did_open(&self, params: DidOpenTextDocumentParams) {
       let uri = params.text_document.uri;
       let text = params.text_document.text;
       self.documents.write().unwrap().insert(uri.clone(), text.clone());
       self.validate_and_publish(&uri, &text).await;
   }

   async fn did_change(&self, params: DidChangeTextDocumentParams) {
       let uri = params.text_document.uri;
       if let Some(change) = params.content_changes.into_iter().next() {
           self.documents.write().unwrap().insert(uri.clone(), change.text.clone());
           self.validate_and_publish(&uri, &change.text).await;
       }
   }
   ```

3. Create validation bridge:
   ```rust
   use crate::validate::{Validator, ValidationIssue, Severity};

   impl PixelsrcLsp {
       async fn validate_and_publish(&self, uri: &Url, content: &str) {
           let mut validator = Validator::new();
           for (line_num, line) in content.lines().enumerate() {
               validator.validate_line(line_num + 1, line);
           }

           let diagnostics: Vec<Diagnostic> = validator
               .issues()
               .iter()
               .map(|issue| self.issue_to_diagnostic(issue))
               .collect();

           self.client
               .publish_diagnostics(uri.clone(), diagnostics, None)
               .await;
       }

       fn issue_to_diagnostic(&self, issue: &ValidationIssue) -> Diagnostic {
           Diagnostic {
               range: Range {
                   start: Position { line: (issue.line - 1) as u32, character: 0 },
                   end: Position { line: (issue.line - 1) as u32, character: u32::MAX },
               },
               severity: Some(match issue.severity {
                   Severity::Error => DiagnosticSeverity::ERROR,
                   Severity::Warning => DiagnosticSeverity::WARNING,
               }),
               message: issue.message.clone(),
               ..Default::default()
           }
       }
   }
   ```

**Verification:**
```bash
# Create test file with error
echo '{"type": "sprite", "name": "test", "grid": ["{a}{a}", "{a}"]}' > /tmp/test.pxl

# Use VS Code with Generic LSP Client configured to use `pxl lsp`
# Should see diagnostics for undefined token and row mismatch
```

**Dependencies:** Task 17.1

---

### Task 17.3: Grid Coordinate Hover

**Wave:** 3 (parallel with 17.4)

Show grid coordinates when hovering over tokens - critical for GenAI to understand spatial positions.

**Deliverables:**

1. Add hover capability:
   ```rust
   ServerCapabilities {
       hover_provider: Some(HoverProviderCapability::Simple(true)),
       ...
   }
   ```

2. Implement hover handler:
   ```rust
   async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
       let uri = &params.text_document_position_params.text_document.uri;
       let pos = params.text_document_position_params.position;

       let documents = self.documents.read().unwrap();
       let content = documents.get(uri)?;
       let line = content.lines().nth(pos.line as usize)?;

       // Parse line to find grid context
       if let Some(grid_info) = self.parse_grid_context(line, pos.character) {
           let hover_text = format!(
               "**Grid Position**: ({}, {})\n\n\
                **Token**: `{}`\n\n\
                **Row Width**: {} tokens\n\n\
                **Expected Width**: {} tokens",
               grid_info.x, grid_info.y,
               grid_info.token,
               grid_info.row_width,
               grid_info.expected_width
           );

           return Ok(Some(Hover {
               contents: HoverContents::Markup(MarkupContent {
                   kind: MarkupKind::Markdown,
                   value: hover_text,
               }),
               range: None,
           }));
       }

       Ok(None)
   }
   ```

3. Grid context parser:
   ```rust
   struct GridInfo {
       x: usize,        // Column (0-indexed)
       y: usize,        // Row (0-indexed within grid array)
       token: String,   // The token at this position
       row_width: usize,
       expected_width: usize,
   }

   fn parse_grid_context(&self, line: &str, char_pos: u32) -> Option<GridInfo> {
       // Parse JSON line, find "grid" array
       // Determine which row and which token based on char_pos
       // Calculate expected width from size field or first row
   }
   ```

**Verification:**
```bash
# In VS Code, hover over a token in a grid
# Should show: "Grid Position: (3, 2)" etc.
```

**Dependencies:** Task 17.2

---

### Task 17.4: Row Length Diagnostics

**Wave:** 3 (parallel with 17.3)

Enhanced diagnostics specifically for grid alignment issues.

**Deliverables:**

1. Extend `ValidationIssue` for grid-specific errors:
   ```rust
   pub enum IssueType {
       // ... existing types
       RowTooShort { row: usize, actual: usize, expected: usize },
       RowTooLong { row: usize, actual: usize, expected: usize },
       GridHeightMismatch { actual: usize, expected: usize },
   }
   ```

2. Add detailed grid alignment messages:
   ```rust
   fn format_grid_issue(issue: &IssueType) -> String {
       match issue {
           IssueType::RowTooShort { row, actual, expected } => {
               format!(
                   "Row {} has {} tokens but should have {}. Add {} more token(s) to align grid.",
                   row, actual, expected, expected - actual
               )
           }
           IssueType::RowTooLong { row, actual, expected } => {
               format!(
                   "Row {} has {} tokens but should have {}. Remove {} token(s) to align grid.",
                   row, actual, expected, actual - expected
               )
           }
           IssueType::GridHeightMismatch { actual, expected } => {
               format!(
                   "Grid has {} rows but size specifies height of {}.",
                   actual, expected
               )
           }
           // ...
       }
   }
   ```

3. Add code actions for quick fixes:
   ```rust
   // Suggest padding with {_} for short rows
   // Suggest which tokens to remove for long rows
   ```

**Verification:**
```bash
# Create misaligned grid
cat > /tmp/misaligned.pxl << 'EOF'
{"type": "palette", "name": "p", "colors": {"{a}": "#FF0000", "{_}": "#00000000"}}
{"type": "sprite", "name": "s", "size": [4, 3], "palette": "p", "grid": ["{a}{a}{a}{a}", "{a}{a}{a}", "{a}{a}{a}{a}{a}"]}
EOF

# LSP should report:
# - Row 2: has 3 tokens, expected 4 (add 1)
# - Row 3: has 5 tokens, expected 4 (remove 1)
```

**Dependencies:** Task 17.2

---

### Task 17.5: Token Completions

**Wave:** 4 (parallel with 17.6)

Suggest tokens when typing inside grid strings.

**Deliverables:**

1. Add completion capability:
   ```rust
   ServerCapabilities {
       completion_provider: Some(CompletionOptions {
           trigger_characters: Some(vec!["{".to_string()]),
           ..Default::default()
       }),
       ...
   }
   ```

2. Implement completion handler:
   ```rust
   async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
       let uri = &params.text_document_position.text_document.uri;

       // Find all defined tokens in the document
       let tokens = self.collect_defined_tokens(uri);

       // Include built-in tokens
       let mut completions: Vec<CompletionItem> = vec![
           CompletionItem {
               label: "{_}".to_string(),
               detail: Some("Transparent".to_string()),
               kind: Some(CompletionItemKind::COLOR),
               ..Default::default()
           },
       ];

       for (token, color) in tokens {
           completions.push(CompletionItem {
               label: token.clone(),
               detail: Some(color),
               kind: Some(CompletionItemKind::COLOR),
               ..Default::default()
           });
       }

       Ok(Some(CompletionResponse::Array(completions)))
   }
   ```

**Verification:**
```bash
# In VS Code, type "{" inside a grid string
# Should show completion menu with all defined tokens
```

**Dependencies:** Task 17.2

---

### Task 17.6: Document Symbols

**Wave:** 4 (parallel with 17.5)

Provide outline view of palettes, sprites, animations.

**Deliverables:**

1. Add symbols capability and handler:
   ```rust
   async fn document_symbol(
       &self,
       params: DocumentSymbolParams,
   ) -> Result<Option<DocumentSymbolResponse>> {
       let uri = &params.text_document.uri;
       let documents = self.documents.read().unwrap();
       let content = documents.get(uri)?;

       let mut symbols = Vec::new();

       for (line_num, line) in content.lines().enumerate() {
           if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
               let obj_type = obj.get("type").and_then(|t| t.as_str());
               let name = obj.get("name").and_then(|n| n.as_str());

               if let (Some(t), Some(n)) = (obj_type, name) {
                   symbols.push(SymbolInformation {
                       name: n.to_string(),
                       kind: match t {
                           "palette" => SymbolKind::COLOR,
                           "sprite" => SymbolKind::CLASS,
                           "animation" => SymbolKind::FUNCTION,
                           "composition" => SymbolKind::MODULE,
                           _ => SymbolKind::OBJECT,
                       },
                       location: Location {
                           uri: uri.clone(),
                           range: Range {
                               start: Position { line: line_num as u32, character: 0 },
                               end: Position { line: line_num as u32, character: line.len() as u32 },
                           },
                       },
                       ..Default::default()
                   });
               }
           }
       }

       Ok(Some(DocumentSymbolResponse::Flat(symbols)))
   }
   ```

**Verification:**
```bash
# In VS Code, open Outline panel (Cmd+Shift+O)
# Should show palettes, sprites, animations as navigable symbols
```

**Dependencies:** Task 17.2

---

### Task 17.7: Agent Bridge Tool

**Wave:** 5 (GenAI tooling)

A simplified CLI wrapper that makes LSP features accessible to AI agents without full LSP protocol.

**Deliverables:**

1. Add new CLI command:
   ```rust
   /// Verify pixelsrc content for AI agents (returns JSON diagnostics)
   #[command(name = "agent-verify")]
   AgentVerify {
       /// Content to verify (reads from stdin if not provided)
       #[arg(long)]
       content: Option<String>,

       /// Include grid coordinate info for each sprite
       #[arg(long)]
       grid_info: bool,

       /// Include token suggestions for completion
       #[arg(long)]
       suggest_tokens: bool,
   }
   ```

2. Implement in `src/agent.rs`:
   ```rust
   use serde::Serialize;

   #[derive(Serialize)]
   pub struct AgentVerifyResult {
       pub valid: bool,
       pub diagnostics: Vec<AgentDiagnostic>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub grid_info: Option<Vec<GridSpriteInfo>>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub available_tokens: Option<Vec<TokenInfo>>,
   }

   #[derive(Serialize)]
   pub struct AgentDiagnostic {
       pub line: usize,
       pub severity: String,  // "error" | "warning"
       pub message: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub fix_suggestion: Option<String>,
   }

   #[derive(Serialize)]
   pub struct GridSpriteInfo {
       pub name: String,
       pub size: [usize; 2],        // [width, height]
       pub actual_rows: usize,
       pub row_widths: Vec<usize>,  // Width of each row
       pub aligned: bool,           // All rows same width?
   }

   #[derive(Serialize)]
   pub struct TokenInfo {
       pub token: String,
       pub color: String,
       pub palette: String,
   }
   ```

3. JSON output format:
   ```json
   {
     "valid": false,
     "diagnostics": [
       {
         "line": 2,
         "severity": "warning",
         "message": "Row 2 has 3 tokens but should have 4",
         "fix_suggestion": "Add {_} to pad row to 4 tokens"
       }
     ],
     "grid_info": [
       {
         "name": "hero",
         "size": [16, 16],
         "actual_rows": 16,
         "row_widths": [16, 16, 15, 16, ...],
         "aligned": false
       }
     ],
     "available_tokens": [
       {"token": "{skin}", "color": "#FFCC99", "palette": "hero"},
       {"token": "{_}", "color": "#00000000", "palette": "hero"}
     ]
   }
   ```

**Verification:**
```bash
# Verify content from stdin
echo '{"type":"sprite","name":"x","grid":["{a}{a}","{a}"]}' | pxl agent-verify
# Returns JSON with diagnostics

# With grid info
cat sprite.jsonl | pxl agent-verify --grid-info
# Returns JSON with grid alignment details

# Full agent workflow
pxl agent-verify --content '...' --grid-info --suggest-tokens
```

**Dependencies:** Tasks 17.2-17.6 (uses same underlying logic)

---

### Task 17.8: CSS Color Provider

**Wave:** 6 (parallel with 17.9-17.11, requires Phase 22)

Implement LSP Color Provider for CSS color syntax - swatches, pickers, and computed previews.

**Deliverables:**

1. Add color provider capability:
   ```rust
   ServerCapabilities {
       color_provider: Some(ColorProviderCapability::Simple(true)),
       ...
   }
   ```

2. Implement `document_color` handler:
   ```rust
   async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
       let uri = &params.text_document.uri;
       let content = self.documents.read().unwrap().get(uri)?.clone();

       let mut colors = Vec::new();

       for (line_num, line) in content.lines().enumerate() {
           // Find color values in palette definitions
           for color_match in find_colors_in_line(line) {
               let resolved = resolve_color(&color_match.value, &self.var_registry)?;
               colors.push(ColorInformation {
                   range: color_match.range_at_line(line_num),
                   color: rgba_to_lsp_color(&resolved),
               });
           }
       }

       Ok(colors)
   }
   ```

3. Support color formats:
   - Hex: `#FF0000`, `#F00`
   - CSS functions: `rgb()`, `hsl()`, `oklch()`, `hwb()`
   - `color-mix()` - resolve and show computed result
   - `var(--name)` - resolve and show computed result

4. Implement `color_presentation` for color picker edits:
   ```rust
   async fn color_presentation(&self, params: ColorPresentationParams) -> Result<Vec<ColorPresentation>> {
       // Offer multiple format options when user picks a color
       let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
       let hsl = format!("hsl({}, {}%, {}%)", h, s, l);
       let oklch = format!("oklch({}% {} {})", l, c, h);

       Ok(vec![
           ColorPresentation { label: hex, .. },
           ColorPresentation { label: hsl, .. },
           ColorPresentation { label: oklch, .. },
       ])
   }
   ```

**GenAI benefit:** Agents can "see" what `color-mix(in oklch, var(--primary) 70%, black)` actually resolves to.

**Verification:**
```bash
# In VS Code, color values should show colored squares
# Clicking square opens color picker
# color-mix() shows computed result, not the function
```

**Dependencies:** Task 17.2, Phase 22 (CSS-3 color parsing)

---

### Task 17.9: CSS Variable Support

**Wave:** 6 (parallel with 17.8, 17.10, 17.11, requires Phase 22)

Completions, hover, and go-to-definition for CSS custom properties.

**Deliverables:**

1. Extend completion handler for `var(--`:
   ```rust
   async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
       // ... existing token completions

       // CSS variable completions
       if is_inside_var_function(&context) {
           let vars = self.collect_css_variables(uri);
           for (name, value) in vars {
               completions.push(CompletionItem {
                   label: name.clone(),           // "--primary"
                   detail: Some(value.clone()),   // "hsl(220, 60%, 50%)"
                   kind: Some(CompletionItemKind::VARIABLE),
                   insert_text: Some(name),
                   ..Default::default()
               });
           }
       }

       Ok(Some(CompletionResponse::Array(completions)))
   }
   ```

2. Extend hover for CSS variables:
   ```rust
   // When hovering over var(--primary) or --primary definition
   if let Some(var_info) = self.get_variable_info(line, char_pos) {
       let resolved = self.var_registry.resolve(&var_info.name, false)?;

       let hover_text = format!(
           "**CSS Variable**: `{}`\n\n\
            **Value**: `{}`\n\n\
            **Resolved**: `{}`",
           var_info.name,
           var_info.raw_value,
           resolved
       );
       // If it's a color, include computed hex
       if let Ok(rgba) = parse_color(&resolved) {
           hover_text.push_str(&format!("\n\n**Computed**: `#{:02X}{:02X}{:02X}`",
               rgba[0], rgba[1], rgba[2]));
       }
   }
   ```

3. Go to definition for variables:
   ```rust
   async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
       // If cursor is on var(--name), jump to --name definition
       if let Some(var_name) = extract_var_reference(&line, char_pos) {
           if let Some(def_location) = self.find_variable_definition(&var_name, uri) {
               return Ok(Some(GotoDefinitionResponse::Scalar(def_location)));
           }
       }
       Ok(None)
   }
   ```

4. Diagnostic for circular references:
   ```rust
   // Detect: --a: var(--b); --b: var(--a);
   if let Err(VariableError::Circular(chain)) = self.var_registry.resolve(&value, true) {
       diagnostics.push(Diagnostic {
           severity: Some(DiagnosticSeverity::ERROR),
           message: format!("Circular variable reference: {}", chain),
           ..
       });
   }
   ```

**GenAI benefit:** Agents discover what variables exist and what they resolve to without guessing.

**Verification:**
```bash
# Type var(-- and see completion list
# Hover over var(--primary) shows resolved value
# Cmd+Click on var(--primary) jumps to definition
# Circular refs show error squiggle
```

**Dependencies:** Task 17.5, Phase 22 (CSS-5, CSS-6 variable registry)

---

### Task 17.10: Timing Function Visualization

**Wave:** 6 (parallel with 17.8, 17.9, 17.11, requires Phase 22)

Show ASCII easing curve preview when hovering over timing functions.

**Deliverables:**

1. Extend hover for timing functions:
   ```rust
   if let Some(timing_fn) = extract_timing_function(line, char_pos) {
       let curve = render_ascii_easing_curve(&timing_fn, 20, 8);
       let description = describe_timing_function(&timing_fn);

       let hover_text = format!(
           "**Timing Function**: `{}`\n\n\
            ```\n{}\n```\n\n\
            {}",
           timing_fn.to_css_string(),
           curve,
           description
       );
   }
   ```

2. ASCII curve renderer:
   ```rust
   fn render_ascii_easing_curve(timing: &Interpolation, width: usize, height: usize) -> String {
       // Example output for ease-in-out:
       // ┌────────────────────┐
       // │                 ███│
       // │              ███   │
       // │           ███      │
       // │        ███         │
       // │      ██            │
       // │   ███              │
       // │███                 │
       // └────────────────────┘
   }
   ```

3. Timing function descriptions:
   ```rust
   fn describe_timing_function(timing: &Interpolation) -> &'static str {
       match timing {
           Interpolation::Linear => "Constant speed from start to end.",
           Interpolation::EaseIn => "Starts slow, accelerates toward end.",
           Interpolation::EaseOut => "Starts fast, decelerates toward end.",
           Interpolation::EaseInOut => "Slow start and end, fast middle.",
           Interpolation::Steps { count, position } =>
               &format!("Jumps in {} discrete steps ({:?}).", count, position),
           Interpolation::Bezier { p1, p2 } =>
               "Custom cubic bezier curve.",
           // ...
       }
   }
   ```

4. Special handling for `steps()`:
   ```
   **Timing Function**: `steps(4, jump-end)`

   ```
   ┌────────────────────┐
   │               ████│
   │          ████     │
   │     ████          │
   │████               │
   └────────────────────┘
   ```

   Jumps in 4 discrete steps (jump-end).

   **Note**: For pixel art animations, steps() affects property
   tweening (opacity, position), not frame selection. Frame
   selection is handled by the frame array.
   ```

**GenAI benefit:** Agents understand *how* an animation will feel without rendering it.

**Verification:**
```bash
# Hover over ease-in-out, cubic-bezier(), steps()
# Should show ASCII curve and description
```

**Dependencies:** Task 17.3, Phase 22 (CSS-8 timing functions)

---

### Task 17.11: Transform Explainer

**Wave:** 6 (parallel with 17.8, 17.9, 17.10, requires Phase 22)

Describe what CSS transforms will do in plain language.

**Deliverables:**

1. Extend hover for transform values:
   ```rust
   if let Some(transform) = extract_transform(line, char_pos) {
       let explanation = explain_transform(&transform);

       let hover_text = format!(
           "**Transform**: `{}`\n\n\
            **Effect**:\n{}",
           transform.to_css_string(),
           explanation
       );
   }
   ```

2. Transform explainer:
   ```rust
   fn explain_transform(transform: &Transform) -> String {
       let mut effects = Vec::new();

       if let Some((x, y)) = transform.translate {
           effects.push(format!("• Move {} pixels right, {} pixels down", x, y));
       }
       if let Some(deg) = transform.rotate {
           let direction = if deg > 0.0 { "clockwise" } else { "counter-clockwise" };
           effects.push(format!("• Rotate {:.0}° {}", deg.abs(), direction));
       }
       if let Some((sx, sy)) = transform.scale {
           if sx == sy {
               effects.push(format!("• Scale to {}%", (sx * 100.0) as i32));
           } else {
               effects.push(format!("• Scale width to {}%, height to {}%",
                   (sx * 100.0) as i32, (sy * 100.0) as i32));
           }
       }
       if transform.flip_x {
           effects.push("• Flip horizontally (mirror)".to_string());
       }
       if transform.flip_y {
           effects.push("• Flip vertically".to_string());
       }

       effects.join("\n")
   }
   ```

3. Example hover output:
   ```
   **Transform**: `rotate(90deg) scale(2) translate(8, 0)`

   **Effect**:
   • Rotate 90° clockwise
   • Scale to 200%
   • Move 8 pixels right, 0 pixels down

   **Order**: Transforms apply right-to-left (translate first, then scale, then rotate).
   ```

**GenAI benefit:** Agents understand transformation results without trial-and-error rendering.

**Verification:**
```bash
# Hover over transform values in animation keyframes
# Should show plain-language explanation
```

**Dependencies:** Task 17.3, Phase 22 (CSS-14 transforms)

---

### Task 17.12: Agent CSS Tools

**Wave:** 7 (after 17.8-17.11)

Extend agent-verify to resolve and report CSS computed values.

**Deliverables:**

1. Extend CLI command:
   ```rust
   AgentVerify {
       // ... existing args

       /// Resolve CSS variables and color-mix() to computed values
       #[arg(long)]
       resolve_colors: bool,

       /// Show timing function analysis
       #[arg(long)]
       analyze_timing: bool,
   }
   ```

2. Extend JSON output:
   ```rust
   #[derive(Serialize)]
   pub struct AgentVerifyResult {
       // ... existing fields

       #[serde(skip_serializing_if = "Option::is_none")]
       pub resolved_colors: Option<Vec<ResolvedColor>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub timing_analysis: Option<Vec<TimingAnalysis>>,
   }

   #[derive(Serialize)]
   pub struct ResolvedColor {
       pub token: String,           // "{skin}"
       pub original: String,        // "var(--skin-tone)"
       pub resolved: String,        // "#FFCC99"
       pub palette: String,
   }

   #[derive(Serialize)]
   pub struct TimingAnalysis {
       pub animation: String,
       pub timing_function: String,
       pub description: String,
       pub curve_type: String,      // "smooth" | "stepped" | "bouncy"
   }
   ```

3. Example output with `--resolve-colors`:
   ```json
   {
     "valid": true,
     "resolved_colors": [
       {
         "token": "{skin}",
         "original": "var(--skin-tone)",
         "resolved": "#FFCC99",
         "palette": "character"
       },
       {
         "token": "{shadow}",
         "original": "color-mix(in oklch, var(--skin-tone) 70%, black)",
         "resolved": "#B38F6B",
         "palette": "character"
       }
     ]
   }
   ```

4. Example output with `--analyze-timing`:
   ```json
   {
     "valid": true,
     "timing_analysis": [
       {
         "animation": "walk_cycle",
         "timing_function": "steps(4, jump-end)",
         "description": "Jumps in 4 discrete steps",
         "curve_type": "stepped"
       }
     ]
   }
   ```

**GenAI benefit:** Agents can verify their CSS expressions resolve to expected values before rendering.

**Verification:**
```bash
pxl agent-verify --resolve-colors < sprite_with_vars.pxl
# Shows all colors with original and resolved values

pxl agent-verify --analyze-timing < animation.pxl
# Shows timing function analysis for each animation
```

**Dependencies:** Tasks 17.7-17.11

---

## GenAI Integration Patterns

### Pattern 1: Pre-Render Validation Loop

```python
def generate_sprite(prompt):
    content = llm.generate(prompt)

    # Verify before declaring done
    result = run("pxl agent-verify --grid-info", input=content)

    if not result["valid"]:
        # Feed diagnostics back to LLM for self-correction
        content = llm.generate(f"""
            Fix these issues in the sprite:
            {json.dumps(result["diagnostics"])}

            Original content:
            {content}
        """)
        # Re-verify...

    return content
```

### Pattern 2: Grid-Aware Generation

```python
# Get grid info to understand current state
result = run("pxl agent-verify --grid-info", input=partial_content)

for sprite in result["grid_info"]:
    if not sprite["aligned"]:
        expected_width = sprite["size"][0]
        for i, width in enumerate(sprite["row_widths"]):
            if width != expected_width:
                print(f"Row {i}: needs {expected_width - width} more tokens")
```

### Pattern 3: Token Discovery

```python
# Discover available tokens before generating
result = run("pxl agent-verify --suggest-tokens", input=existing_content)

available = [t["token"] for t in result["available_tokens"]]
# Use in prompt: "Only use these tokens: {available}"
```

### Pattern 4: CSS Color Verification (requires Phase 22)

```python
def generate_palette_with_shadows(base_colors):
    """Generate palette with computed shadow colors."""
    content = llm.generate(f"""
        Create a pixelsrc palette with these base colors: {base_colors}
        Use color-mix(in oklch, <base> 70%, black) for shadow variants.
        Use CSS variables to avoid repetition.
    """)

    # Verify computed colors resolve correctly
    result = run("pxl agent-verify --resolve-colors", input=content)

    if result["valid"]:
        # Log what colors actually resolved to
        for color in result["resolved_colors"]:
            print(f"{color['token']}: {color['original']} → {color['resolved']}")

    return content
```

### Pattern 5: Animation Timing Analysis (requires Phase 22)

```python
def verify_animation_feel(animation_content, expected_type):
    """Ensure animation timing matches expected feel."""
    result = run("pxl agent-verify --analyze-timing", input=animation_content)

    for timing in result.get("timing_analysis", []):
        if timing["curve_type"] != expected_type:
            # Re-generate with corrected timing
            return llm.generate(f"""
                Fix this animation to use {expected_type} timing instead of {timing['curve_type']}.
                Current: {timing['timing_function']}
                Content: {animation_content}
            """)

    return animation_content
```

---

## VS Code Configuration

Users can integrate with the Generic LSP Client extension:

```json
{
    "generic-lsp.server-definitions": {
        "pixelsrc": {
            "command": "pxl",
            "args": ["lsp"],
            "rootUri": "${workspaceFolder}",
            "languages": ["json", "jsonl"],
            "extensions": [".pxl", ".jsonl"]
        }
    }
}
```

---

## Verification Summary

```bash
# === WAVE 1-5: Core LSP ===

# 1. LSP server starts
./target/release/pxl lsp &
# (test with LSP client)

# 2. Diagnostics work
# Open file with errors in VS Code, should see squiggles

# 3. Hover shows coordinates
# Hover over grid tokens, should see x,y position

# 4. Completions work
# Type "{" in grid, should see token suggestions

# 5. Symbols work
# Open outline, should see palettes/sprites

# 6. Agent bridge works
echo '{"type":"sprite","name":"x","grid":["{a}{a}","{a}"]}' | pxl agent-verify
# Should return JSON with diagnostics

# === WAVE 6-7: CSS Features (requires Phase 22) ===

# 7. Color provider works
# Open palette with hsl(), color-mix() - should show color swatches
# Click swatch to open picker

# 8. CSS variable completions
# Type var(-- inside a color value, should see defined variables

# 9. CSS variable hover
# Hover over var(--primary), should show resolved value

# 10. Timing function visualization
# Hover over ease-in-out or cubic-bezier(), should show ASCII curve

# 11. Transform explanation
# Hover over rotate(90deg) scale(2), should show effect description

# 12. Agent CSS tools
pxl agent-verify --resolve-colors < sprite_with_vars.pxl
# Should show resolved color hex values

pxl agent-verify --analyze-timing < animation.pxl
# Should show timing function analysis
```

---

## Future Enhancements

| Feature | Description |
|---------|-------------|
| Go to Definition (tokens) | Jump from grid token `{skin}` to palette definition |
| Find References | Find all uses of a token across file |
| Rename Symbol | Safely rename tokens or CSS variables across file |
| Format on Save | Wire `pxl fmt` to LSP formatting |
| Semantic Tokens | Syntax highlighting via LSP |
| Grid Visualization | ASCII sprite preview in hover tooltip |
| Color Contrast Check | Warn when adjacent colors have low contrast |
| Palette Suggestions | Suggest harmonious colors based on existing palette |
