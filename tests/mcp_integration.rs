//! MCP server integration tests.
//!
//! These tests spawn `pxl mcp` as a subprocess and communicate via JSON-RPC 2.0
//! over stdin/stdout, verifying the full MCP protocol handshake, tool listing,
//! tool invocation, resource listing/reading, and error handling.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Get the path to the pxl binary (must be built with --features mcp).
fn pxl_binary() -> PathBuf {
    let release = Path::new("target/release/pxl");
    if release.exists() {
        return release.to_path_buf();
    }
    let debug = Path::new("target/debug/pxl");
    if debug.exists() {
        return debug.to_path_buf();
    }
    panic!("pxl binary not found. Run `cargo build --features mcp` first.");
}

/// A lightweight MCP client that talks to a `pxl mcp` subprocess.
struct McpClient {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpClient {
    fn spawn() -> Self {
        let mut child = Command::new(pxl_binary())
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn pxl mcp");

        let stdin = child.stdin.take().expect("no stdin");
        let stdout = child.stdout.take().expect("no stdout");
        let reader = BufReader::new(stdout);

        McpClient { child, stdin, reader, next_id: 1 }
    }

    /// Send a JSON-RPC request and return the parsed response.
    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;

        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let line = serde_json::to_string(&msg).unwrap();
        writeln!(self.stdin, "{}", line).expect("write to stdin failed");
        self.stdin.flush().expect("flush stdin failed");

        // Read response line
        let mut buf = String::new();
        self.reader.read_line(&mut buf).expect("read from stdout failed");
        serde_json::from_str(&buf)
            .unwrap_or_else(|e| panic!("failed to parse response JSON: {}\nraw: {}", e, buf))
    }

    /// Send the initialize handshake and return the result.
    fn initialize(&mut self) -> serde_json::Value {
        let resp = self.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "0.1.0"
                }
            }),
        );

        // Send initialized notification (no id, no response expected)
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        });
        let line = serde_json::to_string(&notif).unwrap();
        writeln!(self.stdin, "{}", line).expect("write notification failed");
        self.stdin.flush().expect("flush notification failed");

        // Small delay to let server process the notification
        std::thread::sleep(Duration::from_millis(50));

        resp
    }

    /// Call tools/list and return the result.
    fn list_tools(&mut self) -> serde_json::Value {
        self.request("tools/list", serde_json::json!({}))
    }

    /// Call a specific tool with the given arguments.
    fn call_tool(&mut self, name: &str, args: serde_json::Value) -> serde_json::Value {
        self.request(
            "tools/call",
            serde_json::json!({
                "name": name,
                "arguments": args,
            }),
        )
    }

    /// Call resources/list and return the result.
    fn list_resources(&mut self) -> serde_json::Value {
        self.request("resources/list", serde_json::json!({}))
    }

    /// Call resources/read for a specific URI.
    fn read_resource(&mut self, uri: &str) -> serde_json::Value {
        self.request("resources/read", serde_json::json!({ "uri": uri }))
    }

    /// Call prompts/list and return the result.
    fn list_prompts(&mut self) -> serde_json::Value {
        self.request("prompts/list", serde_json::json!({}))
    }

    /// Call prompts/get for a specific prompt with arguments.
    fn get_prompt(&mut self, name: &str, args: serde_json::Value) -> serde_json::Value {
        self.request(
            "prompts/get",
            serde_json::json!({
                "name": name,
                "arguments": args,
            }),
        )
    }

    /// Shut down by closing stdin, which causes the server to exit.
    fn shutdown(mut self) {
        drop(self.stdin);
        let _ = self.child.wait();
    }
}

// ── Handshake ─────────────────────────────────────────────────────────

#[test]
fn test_mcp_initialize_handshake() {
    let mut client = McpClient::spawn();
    let resp = client.initialize();

    // Must have a result (not an error)
    let result = resp.get("result").expect("initialize should return result");

    // Check protocol version
    assert_eq!(
        result["protocolVersion"].as_str().unwrap(),
        "2024-11-05",
        "protocol version mismatch"
    );

    // Check server info
    let info = &result["serverInfo"];
    assert_eq!(info["name"].as_str().unwrap(), "pixelsrc-mcp");
    assert!(info["version"].as_str().is_some(), "version should be present");

    // Check capabilities
    let caps = &result["capabilities"];
    assert!(caps.get("tools").is_some(), "tools capability should be present");
    assert!(caps.get("resources").is_some(), "resources capability should be present");
    assert!(caps.get("prompts").is_some(), "prompts capability should be present");

    client.shutdown();
}

// ── Tool Listing ──────────────────────────────────────────────────────

#[test]
fn test_mcp_tools_list() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.list_tools();
    let result = resp.get("result").expect("tools/list should return result");
    let tools = result["tools"].as_array().expect("tools should be an array");

    // We have 6 implemented tools
    assert_eq!(tools.len(), 6, "expected 6 tools, got {}", tools.len());

    // Collect tool names
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    let expected = [
        "pixelsrc_format",
        "pixelsrc_prime",
        "pixelsrc_palettes",
        "pixelsrc_analyze",
        "pixelsrc_scaffold",
        "pixelsrc_import",
    ];
    for name in &expected {
        assert!(names.contains(name), "missing tool: {}", name);
    }

    // Each tool should have a valid input schema
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            tool.get("description").and_then(|d| d.as_str()).is_some(),
            "tool {} missing description",
            name
        );
        let schema =
            tool.get("inputSchema").unwrap_or_else(|| panic!("tool {} missing inputSchema", name));
        assert_eq!(
            schema["type"].as_str().unwrap(),
            "object",
            "tool {} inputSchema should be object type",
            name
        );
    }

    client.shutdown();
}

// ── Tool Calls ────────────────────────────────────────────────────────

#[test]
fn test_mcp_tool_format() {
    let mut client = McpClient::spawn();
    client.initialize();

    let source = r##"{"type":"sprite","name":"dot","size":[1,1],"palette":{"_":"#00000000","x":"#FF0000"},"regions":{"x":{"points":[[0,0]],"z":0}}}"##;

    let resp = client.call_tool("pixelsrc_format", serde_json::json!({ "source": source }));
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    assert!(!content.is_empty(), "content should not be empty");
    assert_eq!(content[0]["type"].as_str().unwrap(), "text");

    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("sprite"), "formatted output should contain sprite type");

    client.shutdown();
}

#[test]
fn test_mcp_tool_prime() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.call_tool("pixelsrc_prime", serde_json::json!({ "brief": true }));
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    assert!(!content.is_empty());

    let text = content[0]["text"].as_str().unwrap();
    assert!(text.len() > 100, "primer should have substantial content");

    client.shutdown();
}

#[test]
fn test_mcp_tool_palettes_list() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.call_tool("pixelsrc_palettes", serde_json::json!({ "action": "list" }));
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    let text = content[0]["text"].as_str().unwrap();

    let parsed: serde_json::Value = serde_json::from_str(text).expect("should be valid JSON");
    let palettes = parsed["palettes"].as_array().expect("palettes should be array");
    assert!(!palettes.is_empty(), "should have at least one palette");
    assert!(
        palettes.iter().any(|p| p.as_str().unwrap().contains("gameboy")),
        "should include gameboy palette"
    );

    client.shutdown();
}

#[test]
fn test_mcp_tool_palettes_show() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client
        .call_tool("pixelsrc_palettes", serde_json::json!({ "action": "show", "name": "gameboy" }));
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    let text = content[0]["text"].as_str().unwrap();

    let parsed: serde_json::Value = serde_json::from_str(text).expect("should be valid JSON");
    assert_eq!(parsed["name"].as_str().unwrap(), "@gameboy");
    assert!(parsed.get("colors").is_some(), "should have colors");

    client.shutdown();
}

#[test]
fn test_mcp_tool_analyze() {
    let mut client = McpClient::spawn();
    client.initialize();

    let source = r##"{"type":"sprite","name":"dot","size":[1,1],"palette":{"_":"#00000000","x":"#FF0000"},"regions":{"x":{"points":[[0,0]],"z":0}}}"##;

    let resp = client.call_tool("pixelsrc_analyze", serde_json::json!({ "source": source }));
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    let text = content[0]["text"].as_str().unwrap();

    let parsed: serde_json::Value = serde_json::from_str(text).expect("should be valid JSON");
    assert_eq!(parsed["total_sprites"].as_u64().unwrap(), 1);
    assert_eq!(parsed["files_analyzed"].as_u64().unwrap(), 1);

    client.shutdown();
}

#[test]
fn test_mcp_tool_scaffold() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.call_tool(
        "pixelsrc_scaffold",
        serde_json::json!({
            "asset_type": "sprite",
            "name": "test_sprite",
            "width": 8,
            "height": 8,
        }),
    );
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("test_sprite"), "scaffold should contain sprite name");
    assert!(text.contains("sprite"), "scaffold should contain sprite type");

    client.shutdown();
}

#[test]
fn test_mcp_tool_import() {
    let mut client = McpClient::spawn();
    client.initialize();

    // 1x1 red PNG, base64-encoded
    let red_pixel_png =
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC";

    let resp = client.call_tool(
        "pixelsrc_import",
        serde_json::json!({
            "image": red_pixel_png,
            "name": "red_dot",
        }),
    );
    let result = resp.get("result").expect("tool call should return result");
    let content = result["content"].as_array().expect("content should be array");
    assert!(!content.is_empty(), "import should return content");

    // Import returns a summary + .pxl JSONL; check for expected keywords
    let full_text: String =
        content.iter().filter_map(|c| c["text"].as_str()).collect::<Vec<_>>().join("\n");
    assert!(
        full_text.contains("red_dot")
            || full_text.contains("sprite")
            || full_text.contains("palette")
            || full_text.contains("pxl"),
        "import output should contain sprite/palette data, got: {}",
        &full_text[..full_text.len().min(500)]
    );

    client.shutdown();
}

// ── Error Cases ───────────────────────────────────────────────────────

#[test]
fn test_mcp_error_invalid_tool_name() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.call_tool("nonexistent_tool", serde_json::json!({}));
    // Should return an error, not crash
    assert!(
        resp.get("error").is_some() || {
            // Some MCP implementations return isError in result instead of top-level error
            resp.get("result")
                .and_then(|r| r.get("isError"))
                .and_then(|e| e.as_bool())
                .unwrap_or(false)
        },
        "calling nonexistent tool should return an error: {:?}",
        resp
    );

    client.shutdown();
}

#[test]
fn test_mcp_error_missing_required_field() {
    let mut client = McpClient::spawn();
    client.initialize();

    // pixelsrc_format requires "source" field
    let resp = client.call_tool("pixelsrc_format", serde_json::json!({}));
    assert!(
        resp.get("error").is_some() || {
            resp.get("result")
                .and_then(|r| r.get("isError"))
                .and_then(|e| e.as_bool())
                .unwrap_or(false)
        },
        "missing required field should return error: {:?}",
        resp
    );

    client.shutdown();
}

#[test]
fn test_mcp_error_invalid_palette_action() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp =
        client.call_tool("pixelsrc_palettes", serde_json::json!({ "action": "invalid_action" }));
    let result = resp.get("result").expect("should have result");
    let is_error = result.get("isError").and_then(|e| e.as_bool()).unwrap_or(false);
    assert!(is_error, "invalid palette action should be an error: {:?}", resp);

    client.shutdown();
}

#[test]
fn test_mcp_error_unknown_palette_name() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.call_tool(
        "pixelsrc_palettes",
        serde_json::json!({ "action": "show", "name": "nonexistent_palette" }),
    );
    let result = resp.get("result").expect("should have result");
    let is_error = result.get("isError").and_then(|e| e.as_bool()).unwrap_or(false);
    assert!(is_error, "unknown palette name should be an error: {:?}", resp);

    client.shutdown();
}

#[test]
fn test_mcp_error_scaffold_unknown_type() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.call_tool(
        "pixelsrc_scaffold",
        serde_json::json!({ "asset_type": "bogus", "name": "test" }),
    );
    let result = resp.get("result").expect("should have result");
    let is_error = result.get("isError").and_then(|e| e.as_bool()).unwrap_or(false);
    assert!(is_error, "unknown asset type should be an error: {:?}", resp);

    client.shutdown();
}

// ── Resources ─────────────────────────────────────────────────────────

#[test]
fn test_mcp_resources_list() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.list_resources();
    let result = resp.get("result").expect("resources/list should return result");
    let resources = result["resources"].as_array().expect("resources should be array");

    assert_eq!(resources.len(), 3, "expected 3 static resources");

    let uris: Vec<&str> = resources.iter().map(|r| r["uri"].as_str().unwrap()).collect();
    assert!(uris.contains(&"pixelsrc://format-spec"), "missing format-spec resource");
    assert!(uris.contains(&"pixelsrc://format-brief"), "missing format-brief resource");
    assert!(uris.contains(&"pixelsrc://palettes"), "missing palettes resource");

    client.shutdown();
}

#[test]
fn test_mcp_resource_read_format_spec() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.read_resource("pixelsrc://format-spec");
    let result = resp.get("result").expect("resources/read should return result");
    let contents = result["contents"].as_array().expect("contents should be array");
    assert!(!contents.is_empty());

    let text = contents[0]["text"].as_str().unwrap();
    assert!(text.len() > 500, "format spec should be substantial");

    client.shutdown();
}

#[test]
fn test_mcp_resource_read_palettes() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.read_resource("pixelsrc://palettes");
    let result = resp.get("result").expect("resources/read should return result");
    let contents = result["contents"].as_array().expect("contents should be array");

    let text = contents[0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(text).expect("should be valid JSON");
    let arr = parsed.as_array().expect("palettes resource should be a JSON array");
    assert!(!arr.is_empty(), "should have at least one palette entry");

    client.shutdown();
}

#[test]
fn test_mcp_resource_read_unknown_uri() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.read_resource("pixelsrc://nonexistent");
    assert!(
        resp.get("error").is_some(),
        "reading unknown resource should return error: {:?}",
        resp
    );

    client.shutdown();
}

// ── Prompts ───────────────────────────────────────────────────────────

#[test]
fn test_mcp_prompts_list() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.list_prompts();
    let result = resp.get("result").expect("prompts/list should return result");
    let prompts = result["prompts"].as_array().expect("prompts should be array");

    assert_eq!(prompts.len(), 4, "expected 4 prompts, got {}", prompts.len());

    let names: Vec<&str> = prompts.iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"create_sprite"), "missing create_sprite");
    assert!(names.contains(&"create_animation"), "missing create_animation");
    assert!(names.contains(&"review_pxl"), "missing review_pxl");
    assert!(names.contains(&"pixel_art_guide"), "missing pixel_art_guide");

    // Each prompt should have description and arguments
    for prompt in prompts {
        let name = prompt["name"].as_str().unwrap();
        assert!(
            prompt.get("description").and_then(|d| d.as_str()).is_some(),
            "prompt {} missing description",
            name
        );
        let args = prompt["arguments"].as_array().unwrap_or_else(|| {
            panic!("prompt {} missing arguments", name);
        });
        assert!(!args.is_empty(), "prompt {} should have at least one argument", name);
    }

    client.shutdown();
}

#[test]
fn test_mcp_prompt_create_sprite() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.get_prompt(
        "create_sprite",
        serde_json::json!({
            "description": "a small knight",
            "size": "16x16",
            "palette": "pico8",
        }),
    );
    let result = resp.get("result").expect("prompts/get should return result");
    let messages = result["messages"].as_array().expect("messages should be array");

    assert_eq!(messages.len(), 2, "create_sprite should return 2 messages");

    // First message: assistant with format spec context
    assert_eq!(messages[0]["role"].as_str().unwrap(), "assistant");
    let sys_text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(sys_text.contains("Pixelsrc"), "system msg should contain format reference");
    assert!(sys_text.contains("@pico8"), "system msg should mention chosen palette");

    // Second message: user with the request
    assert_eq!(messages[1]["role"].as_str().unwrap(), "user");
    let user_text = messages[1]["content"]["text"].as_str().unwrap();
    assert!(user_text.contains("a small knight"), "user msg should contain description");
    assert!(user_text.contains("16x16"), "user msg should contain size");

    client.shutdown();
}

#[test]
fn test_mcp_prompt_create_animation() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.get_prompt(
        "create_animation",
        serde_json::json!({
            "description": "a coin spinning",
            "frames": "6",
            "fps": "10",
        }),
    );
    let result = resp.get("result").expect("prompts/get should return result");
    let messages = result["messages"].as_array().expect("messages should be array");
    assert_eq!(messages.len(), 2);

    let user_text = messages[1]["content"]["text"].as_str().unwrap();
    assert!(user_text.contains("6-frame"), "should mention frame count");
    assert!(user_text.contains("a coin spinning"), "should mention description");

    client.shutdown();
}

#[test]
fn test_mcp_prompt_review_pxl() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.get_prompt(
        "review_pxl",
        serde_json::json!({
            "source": "{\"type\":\"sprite\",\"name\":\"test\"}",
        }),
    );
    let result = resp.get("result").expect("prompts/get should return result");
    let messages = result["messages"].as_array().expect("messages should be array");
    assert_eq!(messages.len(), 2);

    let sys_text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(sys_text.contains("Review Checklist"), "should contain review checklist");

    let user_text = messages[1]["content"]["text"].as_str().unwrap();
    assert!(user_text.contains("test"), "should embed the source");

    client.shutdown();
}

#[test]
fn test_mcp_prompt_pixel_art_guide() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.get_prompt("pixel_art_guide", serde_json::json!({ "genre": "RPG" }));
    let result = resp.get("result").expect("prompts/get should return result");
    let messages = result["messages"].as_array().expect("messages should be array");
    assert_eq!(messages.len(), 2);

    let sys_text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(sys_text.contains("@gameboy"), "should list available palettes");

    let user_text = messages[1]["content"]["text"].as_str().unwrap();
    assert!(user_text.contains("RPG"), "should mention genre");

    client.shutdown();
}

#[test]
fn test_mcp_prompt_unknown_name() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.get_prompt("nonexistent_prompt", serde_json::json!({}));
    assert!(resp.get("error").is_some(), "getting unknown prompt should return error: {:?}", resp);

    client.shutdown();
}
