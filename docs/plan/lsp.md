# LSP Support Implementation Plan

## Objective
Enable live validation and developer tooling for Pixelsrc files (`.pxl`, `.jsonl`) in IDEs (like VS Code) by implementing the Language Server Protocol (LSP).

## Architecture

The implementation will add a new `lsp` module to the existing `pixelsrc` library, exposed via a hidden `pxl lsp` CLI command.

### Components
1.  **LSP Server**: Uses `tower-lsp` to handle JSON-RPC communication.
2.  **Validator Bridge**: Wraps the existing `Validator` struct (from `src/validate.rs`) to convert `ValidationIssue`s into LSP `Diagnostic` objects.
3.  **CLI Entrypoint**: A new `lsp` subcommand in `src/cli.rs` that starts the server over stdio.

## Implementation Steps

### Phase 1: Core Infrastructure
1.  **Dependencies**: Add `tower-lsp` and `tokio` to `Cargo.toml`.
2.  **Server Module**: Create `src/lsp.rs` with a basic `LanguageServer` implementation.
3.  **CLI Command**:
    *   Add `Lsp` variant to `Commands` enum in `src/cli.rs`.
    *   Implement `run_lsp()` to start the server.
    *   Mark command as `#[command(hide = true)]`.

### Phase 2: Validation Integration
1.  Implement `did_open` and `did_change` handlers in `src/lsp.rs`.
2.  Inside these handlers:
    *   Parse the document content.
    *   Instantiate `Validator`.
    *   Run `validate_line` for each line.
    *   Map `ValidationIssue` to `tower_lsp::lsp_types::Diagnostic`.
    *   Publish diagnostics back to the client.

### Phase 3: Client Integration (VS Code)
1.  Users can use the "Generic LSP Client" extension.
2.  Configuration:
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

## Future Enhancements

### 1. Visual & Color Features
*   **Color Provider**: Support standard LSP color icons and pickers next to hex codes.
*   **Hover Previews**: 
    *   Show color details and previews when hovering over palette tokens.
    *   Show sprite metadata or even low-res previews when hovering over sprite names.

### 2. Navigation & Discovery
*   **Go to Definition**: Jump from a grid token (e.g., `{s}`) to its palette definition, or from an animation frame to the sprite definition.
*   **Find References**: List all sprites/animations using a specific token or palette.
*   **Document Symbols (Outline)**: Populate the editor's outline view with Sprites, Palettes, and Animations for easy navigation.

### 3. Intelligent Editing
*   **Completions (Intellisense)**: 
    *   Suggest palette tokens when typing `{` inside a grid.
    *   Suggest sprite names inside animation frame lists.
    *   Suggest built-in palettes starting with `@`.
*   **Rename Symbol**: Safely rename tokens or sprites across the entire file/project.
*   **Document Formatting**: Integrate `pxl fmt` to support "Format on Save" directly via the LSP.

### 4. Code Actions (Quick Fixes)
*   Offer automatic fixes for validation errors (e.g., "Add missing token to palette").
## GenAI Agent Integration

The LSP server is not limited to human-facing IDEs. It provides a structured, standard protocol for AI agents to interact with the codebase with semantic understanding.

### Use Cases for Agents

1.  **Verification Loop (Self-Correction)**:
    *   Agents can draft content, send it to the LSP, and receive machine-readable diagnostics (errors) to fix issues *before* presenting the result to the user.
    *   Eliminates syntax errors and "undefined token" hallucinations.

2.  **Context Discovery (RAG)**:
    *   Agents can use `textDocument/documentSymbol` to "map" the project structure without reading raw files.
    *   Agents can use `textDocument/completion` to discover valid palette tokens or sprite names, ensuring generated code is grounded in the existing project reality.

3.  **Safe Refactoring**:
    *   Agents can delegate complex changes (e.g., "Rename `{skin}` to `{flesh}` everywhere") to the LSP's `textDocument/rename` capability, ensuring AST-level correctness rather than fragile regex replacements.

### Implementation Strategy: `pxl-agent-bridge`

We will build a lightweight wrapper tool (likely a small CLI or Python script) that exposes LSP capabilities as simple function calls for agents.

**Proposed Agent Tool Interface:**
```json
{
  "name": "verify_pixelsrc",
  "description": "Validates Pixelsrc code using the LSP server",
  "parameters": { "content": "string" }
}
```

**Code Example (Rust Agent Tool Wrapper):**

```rust
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use serde_json::{json, Value};
use std::process::Stdio;

pub struct LspAgentClient {
    child: Child,
    stdin: tokio::process::ChildStdin,
    reader: BufReader<tokio::process::ChildStdout>,
}

impl LspAgentClient {
    pub async fn spawn(bin_path: &str) -> anyhow::Result<Self> {
        let mut child = Command::new(bin_path)
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        Ok(Self { child, stdin, reader })
    }

    pub async fn verify_content(&mut self, content: &str) -> anyhow::Result<Value> {
        // 1. Initialize
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "capabilities": {} }
        })).await?;
        let _init_res = self.read_message().await?;

        // 2. Open Document
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///virtual/check.pxl",
                    "languageId": "pixelsrc",
                    "version": 1,
                    "text": content
                }
            }
        })).await?;

        // 3. Read Diagnostics (the server will publish these as a notification)
        loop {
            let msg = self.read_message().await?;
            if msg["method"] == "textDocument/publishDiagnostics" {
                return Ok(msg["params"]["diagnostics"].clone());
            }
        }
    }

    async fn send(&mut self, msg: Value) -> anyhow::Result<()> {
        let body = serde_json::to_string(&msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        self.stdin.write_all(header.as_bytes()).await?;
        self.stdin.write_all(body.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read_message(&mut self) -> anyhow::Result<Value> {
        let mut line = String::new();
        let mut content_length = 0;

        // Read headers
        loop {
            line.clear();
            self.reader.read_line(&mut line).await?;
            if line == "\r\n" || line.is_empty() { break; }
            if line.starts_with("Content-Length: ") {
                content_length = line.trim_start_matches("Content-Length: ")
                    .trim().parse::<usize>()?;
            }
        }

        // Read body
        let mut body = vec![0u8; content_length];
        self.reader.read_exact(&mut body).await?;
        Ok(serde_json::from_slice(&body)?)
    }
}
```
