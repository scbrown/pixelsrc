//! Agent Command Demo Tests
//!
//! Demonstrates the `pxl agent` command functionality for AI/CLI integration.
//! This command provides structured access to validation, completions, and
//! position information through subcommands.

use pixelsrc::lsp_agent_client::LspAgentClient;

// ============================================================================
// Agent Verify Subcommand Tests
// ============================================================================
/// Tests verify subcommand functionality (mirrors agent-verify but via subcommand)
#[test]
fn test_agent_verify_returns_valid_result() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "s", "palette": "p", "size": [1, 1], "regions": {"x": {"points": [[0,0]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    assert!(result.valid);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_agent_verify_strict_mode() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "s", "palette": "p", "size": [2, 1], "regions": {"x": {"points": [[0,0]]}, "y": {"points": [[1,0]]}}}"##;

    // Compare strict vs non-strict
    let normal_client = LspAgentClient::new();
    let strict_client = LspAgentClient::strict();

    let normal_result = normal_client.verify_content(content);
    let strict_result = strict_client.verify_content(content);

    // Strict mode may change validity status if there are warnings
    if normal_result.warning_count > 0 {
        // Normal should be valid (warnings don't fail), strict may not be
        assert!(normal_result.valid || normal_result.error_count > 0);
    }

    // Both should report same counts
    assert_eq!(normal_result.error_count, strict_result.error_count);
    assert_eq!(normal_result.warning_count, strict_result.warning_count);
}

// ============================================================================
// Agent Completions Subcommand Tests
// ============================================================================
#[test]
fn test_agent_completions_at_line() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{a}": "#FF0000", "{b}": "#00FF00"}}
{"type": "sprite", "name": "s", "palette": "p", "size": [2, 1], "regions": {"a": {"points": [[0,0]]}, "b": {"points": [[1,0]]}}}"##;

    let client = LspAgentClient::new();

    // Get completions at line 2 (sprite line)
    let result = client.get_completions(content, 2, 0);

    // Should include tokens from the palette
    let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"{a}"));
    assert!(labels.contains(&"{b}"));
}

#[test]
fn test_agent_completions_includes_builtin() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}"##;

    let client = LspAgentClient::new();
    let result = client.get_completions(content, 1, 0);

    // Should always include built-in tokens
    let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"{_}"), "Should include transparent token");
    assert!(labels.contains(&"."), "Should include dot shorthand");
}

#[test]
fn test_agent_completions_json_output() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}"##;

    let client = LspAgentClient::new();
    let json = client.get_completions_json(content, 1, 0);

    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok(), "Should produce valid JSON");

    let parsed = parsed.unwrap();
    assert!(parsed.get("items").is_some(), "Should have 'items' field");
    assert!(parsed["items"].is_array(), "items should be an array");
}

#[test]
fn test_agent_completions_different_lines() {
    let content = r##"{"type": "palette", "name": "colors", "colors": {"{skin}": "#FFCC99", "{hair}": "#442200"}}
{"type": "sprite", "name": "hero", "palette": "colors", "size": [3, 3], "regions": {"skin": {"points": [[0,0],[1,0]]}, "hair": {"points": [[2,0]]}}}"##;

    let client = LspAgentClient::new();

    // Line 1 = palette line
    let line1_result = client.get_completions(content, 1, 0);
    // Line 2 = sprite line
    let line2_result = client.get_completions(content, 2, 0);

    // Both should have completions (palette defines tokens for both contexts)
    assert!(!line1_result.items.is_empty());
    assert!(!line2_result.items.is_empty());
}

// ============================================================================
// Agent Position Subcommand Tests
// ============================================================================
// Note: Position info works with legacy grid format, not regions format
// The regions format doesn't have inline grid definitions to query positions in

#[test]
fn test_agent_position_outside_grid() {
    // Using regions format - no grid positions available
    let content = r##"{"type": "sprite", "name": "test", "size": [2, 2], "regions": {"x": {"points": [[0,0]]}}}"##;

    let client = LspAgentClient::new();

    // Position queries on regions-format content return None
    let result = client.get_grid_position(content, 1, 10);
    assert!(result.is_none(), "Regions format doesn't have grid positions");
}

#[test]
fn test_agent_position_invalid_line() {
    let content = r##"{"type": "sprite", "name": "test"}"##;

    let client = LspAgentClient::new();

    // Line 0 is invalid (lines are 1-indexed)
    let result = client.get_grid_position(content, 0, 0);
    assert!(result.is_none(), "Line 0 should return None");

    // Line beyond content
    let result = client.get_grid_position(content, 100, 0);
    assert!(result.is_none(), "Line beyond content should return None");
}

// ============================================================================
// Integration Tests
// ============================================================================
#[test]
fn test_agent_workflow_verify_then_complete() {
    // Simulate a typical agent workflow: verify content, then get completions
    let content = r##"{"type": "palette", "name": "ui", "colors": {"{bg}": "#333333", "{fg}": "#FFFFFF", "{accent}": "#00AAFF"}}
{"type": "sprite", "name": "button", "palette": "ui", "size": [4, 2], "regions": {"bg": {"rect": [0,0,4,2]}, "fg": {"points": [[1,0],[2,0]]}}}"##;

    let client = LspAgentClient::new();

    // Step 1: Verify content is valid
    let verification = client.verify_content(content);
    assert!(verification.valid, "Content should be valid");

    // Step 2: Get available completions for editing
    let completions = client.get_completions(content, 2, 0);
    let labels: Vec<&str> = completions.items.iter().map(|i| i.label.as_str()).collect();

    // Should see all palette tokens
    assert!(labels.contains(&"{bg}"));
    assert!(labels.contains(&"{fg}"));
    assert!(labels.contains(&"{accent}"));
}

#[test]
fn test_agent_workflow_with_errors() {
    // Content with intentional issues
    let content = r##"{"type": "sprite", "name": "bad", "palette": "missing_palette", "size": [1, 1], "regions": {"x": {"points": [[0,0]]}}}"##;

    let client = LspAgentClient::new();

    // Verification should detect issues
    let result = client.verify_content(content);
    assert!(result.error_count > 0 || result.warning_count > 0);

    // Completions still work (provide available tokens)
    let completions = client.get_completions(content, 1, 0);
    assert!(!completions.items.is_empty(), "Should still provide built-in completions");
}

// ============================================================================
// Edge Cases
// ============================================================================
#[test]
fn test_agent_with_multiline_content() {
    let content = r##"{"type": "palette", "name": "p1", "colors": {"{a}": "#FF0000"}}
{"type": "palette", "name": "p2", "colors": {"{b}": "#00FF00"}}
{"type": "sprite", "name": "s1", "palette": "p1", "size": [1, 1], "regions": {"a": {"points": [[0,0]]}}}
{"type": "sprite", "name": "s2", "palette": "p2", "size": [1, 1], "regions": {"b": {"points": [[0,0]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    // Multiple definitions should all be validated
    assert!(result.valid);
}

#[test]
fn test_agent_completions_with_no_palettes() {
    let content = r##"{"type": "sprite", "name": "inline", "palette": {"x": "#FF0000"}, "size": [1, 1], "regions": {"x": {"points": [[0,0]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.get_completions(content, 1, 0);

    // Should still include built-ins even with inline palette
    let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"{_}"));
}

#[test]
fn test_agent_verify_with_css_variables() {
    // CSS variables (--name) are allowed in palette definitions
    let content = r##"{"type": "palette", "name": "css", "colors": {"--primary": "#123456", "--secondary": "#789ABC", "{x}": "#123456"}}"##;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    // CSS variable definitions should be accepted
    assert!(result.valid);
}
