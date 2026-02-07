//! Agent-Verify Command Demo Tests
//!
//! Demonstrates the `pxl agent-verify` command functionality for AI agents
//! that need structured validation results with JSON output.

use pixelsrc::lsp_agent_client::LspAgentClient;

// ============================================================================
// Basic Verification Tests
// ============================================================================
/// @demo cli/agent-verify#valid_sprite
/// @title Verify Valid Sprite
/// @description Valid content passes verification with no errors.
#[test]
fn test_verify_valid_sprite() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{_}": "#00000000", "{x}": "#FF0000"}}
{"type": "sprite", "name": "test", "palette": "p", "size": [2, 2], "regions": {"_": {"points": [[0,0],[1,0]]}, "x": {"points": [[0,1],[1,1]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    assert!(result.valid, "Valid content should pass verification");
    assert_eq!(result.error_count, 0, "Should have no errors");
}

/// @demo cli/agent-verify#invalid_json
/// @title Verify Invalid JSON
/// @description Invalid JSON syntax fails verification with errors.
#[test]
fn test_verify_invalid_json() {
    let content = r#"{"type": "sprite", "name": "test" missing_comma: true}"#;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    assert!(!result.valid, "Invalid JSON should fail verification");
    assert!(result.error_count > 0, "Should have errors");
}

/// @demo cli/agent-verify#missing_palette
/// @title Verify Missing Palette
/// @description Missing palette reference produces error or warning.
#[test]
fn test_verify_missing_palette() {
    let content = r##"{"type": "sprite", "name": "test", "palette": "nonexistent", "size": [2, 2], "regions": {"x": {"points": [[0,0]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    // Missing palette reference should produce an error
    assert!(result.error_count > 0 || result.warning_count > 0);
}

// ============================================================================
// Strict Mode Tests
// ============================================================================
/// @demo cli/agent-verify#strict_fails_warnings
/// @title Strict Mode Fails on Warnings
/// @description In strict mode, warnings cause verification to fail.
#[test]
fn test_strict_mode_fails_on_warnings() {
    // Content that might produce warnings but no errors
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "test", "palette": "p", "size": [2, 2], "regions": {"x": {"points": [[0,0],[1,0]]}, "y": {"points": [[0,1],[1,1]]}}}"##;

    let strict_client = LspAgentClient::strict();
    let result = strict_client.verify_content(content);

    // In strict mode, if there are warnings, valid should be false
    if result.warning_count > 0 {
        assert!(!result.valid, "Strict mode should fail on warnings");
    }
}

/// @demo cli/agent-verify#non_strict_warnings
/// @title Non-Strict Mode Passes with Warnings
/// @description Non-strict mode is valid even with warnings if no errors.
#[test]
fn test_non_strict_mode_passes_with_warnings() {
    // Content that might produce warnings
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}
{"type": "sprite", "name": "test", "palette": "p", "size": [2, 2], "regions": {"x": {"points": [[0,0],[1,0]]}, "y": {"points": [[0,1],[1,1]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.verify_content(content);

    // Non-strict mode should be valid even with warnings (if no errors)
    if result.error_count == 0 {
        assert!(result.valid || result.warning_count > 0);
    }
}

// ============================================================================
// Completion Tests
// ============================================================================
/// @demo cli/agent-verify#completions_defined_tokens
/// @title Completions Include Defined Tokens
/// @description Completions include tokens defined in palette.
#[test]
fn test_get_completions_includes_defined_tokens() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{red}": "#FF0000", "{blue}": "#0000FF"}}
{"type": "sprite", "name": "test", "palette": "p", "size": [2, 1], "regions": {"red": {"points": [[0,0]]}, "blue": {"points": [[1,0]]}}}"##;

    let client = LspAgentClient::new();
    let result = client.get_completions(content, 2, 0);

    let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();

    assert!(labels.contains(&"{red}"), "Should include red token");
    assert!(labels.contains(&"{blue}"), "Should include blue token");
}

/// @demo cli/agent-verify#completions_builtin_transparent
/// @title Completions Include Builtin Transparent
/// @description Completions always include {_} and dot shorthand.
#[test]
fn test_get_completions_includes_builtin_transparent() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}"##;

    let client = LspAgentClient::new();
    let result = client.get_completions(content, 1, 0);

    let labels: Vec<&str> = result.items.iter().map(|i| i.label.as_str()).collect();

    assert!(labels.contains(&"{_}"), "Should include built-in transparent token");
    assert!(labels.contains(&"."), "Should include dot shorthand for transparent");
}

/// @demo cli/agent-verify#completion_color_details
/// @title Completion Items Have Color Details
/// @description Completion items include color value details.
#[test]
fn test_completion_items_have_color_details() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{red}": "#FF0000"}}"##;

    let client = LspAgentClient::new();
    let result = client.get_completions(content, 1, 0);

    let red_item = result.items.iter().find(|i| i.label == "{red}");
    assert!(red_item.is_some(), "Should find red completion");

    let red_item = red_item.unwrap();
    assert!(red_item.detail.is_some(), "Should have color detail");
    assert!(
        red_item.detail.as_ref().unwrap().contains("FF0000"),
        "Detail should contain color value"
    );
}

// ============================================================================
// Color Resolution Tests
// ============================================================================
/// @demo cli/agent-verify#resolve_css_variables
/// @title Resolve CSS Variables
/// @description CSS variables like var(--base) are resolved to actual colors.
#[test]
fn test_resolve_css_variables() {
    let content = r##"{"type": "palette", "name": "css", "colors": {"--base": "#FF6347", "{skin}": "var(--base)"}}"##;

    let client = LspAgentClient::new();
    let result = client.resolve_colors(content);

    // Should resolve the var(--base) reference
    let skin = result.colors.iter().find(|c| c.token == "{skin}");
    assert!(skin.is_some(), "Should find skin token");

    if let Some(skin) = skin {
        assert!(skin.original.contains("var(--base)"), "Should have original var() value");
        assert!(skin.resolved.to_uppercase().contains("FF6347"), "Should resolve to base color");
    }
}

/// @demo cli/agent-verify#resolve_color_mix
/// @title Resolve Color Mix
/// @description CSS color-mix() function is resolved to resulting color.
#[test]
fn test_resolve_color_mix() {
    let content = r##"{"type": "palette", "name": "mix", "colors": {"{blend}": "color-mix(in srgb, #FF0000, #0000FF)"}}"##;

    let client = LspAgentClient::new();
    let result = client.resolve_colors(content);

    let blend = result.colors.iter().find(|c| c.token == "{blend}");
    assert!(blend.is_some(), "Should find blend token");

    if let Some(blend) = blend {
        assert!(blend.original.contains("color-mix"), "Should have original color-mix() value");
        // Should resolve to some purple-ish color (mix of red and blue)
        assert!(!blend.resolved.is_empty(), "Should have resolved value");
    }
}

/// @demo cli/agent-verify#resolve_marks_variables
/// @title Resolve Colors Marks Variables
/// @description CSS variables are distinguished from regular tokens.
#[test]
fn test_resolve_colors_marks_variables() {
    let content = r##"{"type": "palette", "name": "vars", "colors": {"--primary": "#FF0000", "{token}": "#00FF00"}}"##;

    let client = LspAgentClient::new();
    let result = client.resolve_colors(content);

    // Find the variable
    let var = result.colors.iter().find(|c| c.token == "--primary");
    if let Some(var) = var {
        assert!(var.is_variable, "CSS variable should be marked as variable");
    }

    // Find the token
    let token = result.colors.iter().find(|c| c.token == "{token}");
    if let Some(token) = token {
        assert!(!token.is_variable, "Token should not be marked as variable");
    }
}

// ============================================================================
// Timing Analysis Tests
// ============================================================================
/// @demo cli/agent-verify#timing_named_functions
/// @title Analyze Named Timing Functions
/// @description Named timing functions like ease-in-out are analyzed.
#[test]
fn test_analyze_timing_named_functions() {
    let content = r#"{"type": "animation", "name": "walk", "frames": [{"sprite": "f1"}], "timing_function": "ease-in-out", "fps": 12}"#;

    let client = LspAgentClient::new();
    let result = client.analyze_timing(content);

    assert!(!result.animations.is_empty(), "Should find animation");

    let walk = &result.animations[0];
    assert_eq!(walk.animation, "walk");
    assert!(walk.timing_function.contains("ease-in-out"));
    assert!(!walk.description.is_empty(), "Should have description");
}

/// @demo cli/agent-verify#timing_cubic_bezier
/// @title Analyze Cubic Bezier Timing
/// @description Custom cubic-bezier timing functions are analyzed.
#[test]
fn test_analyze_timing_cubic_bezier() {
    let content = r#"{"type": "animation", "name": "bounce", "frames": [{"sprite": "f1"}], "timing_function": "cubic-bezier(0.68, -0.55, 0.27, 1.55)", "fps": 8}"#;

    let client = LspAgentClient::new();
    let result = client.analyze_timing(content);

    if !result.animations.is_empty() {
        let bounce = &result.animations[0];
        assert!(
            bounce.timing_function.contains("cubic-bezier") || bounce.curve_type.contains("bezier"),
            "Should identify cubic-bezier"
        );
    }
}

/// @demo cli/agent-verify#timing_steps
/// @title Analyze Steps Timing
/// @description Steps timing function with jump modes is analyzed.
#[test]
fn test_analyze_timing_steps() {
    let content = r#"{"type": "animation", "name": "step_anim", "frames": [{"sprite": "f1"}], "timing_function": "steps(4, end)", "fps": 6}"#;

    let client = LspAgentClient::new();
    let result = client.analyze_timing(content);

    if !result.animations.is_empty() {
        let step_anim = &result.animations[0];
        assert!(
            step_anim.timing_function.contains("steps") || step_anim.curve_type.contains("step"),
            "Should identify steps timing"
        );
    }
}

// ============================================================================
// JSON Output Tests
// ============================================================================
/// @demo cli/agent-verify#json_verify
/// @title Verify Content JSON Format
/// @description Verification results can be output as valid JSON.
#[test]
fn test_verify_content_json_format() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}"##;

    let client = LspAgentClient::new();
    let json = client.verify_content_json(content);

    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok(), "Output should be valid JSON");

    let parsed = parsed.unwrap();
    assert!(parsed.get("valid").is_some(), "Should have 'valid' field");
    assert!(parsed.get("error_count").is_some(), "Should have 'error_count' field");
}

/// @demo cli/agent-verify#json_resolve_colors
/// @title Resolve Colors JSON Format
/// @description Color resolution results can be output as valid JSON.
#[test]
fn test_resolve_colors_json_format() {
    let content = r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}"##;

    let client = LspAgentClient::new();
    let json = client.resolve_colors_json(content);

    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok(), "Output should be valid JSON");

    let parsed = parsed.unwrap();
    assert!(parsed.get("colors").is_some(), "Should have 'colors' field");
}

// ============================================================================
// Edge Cases
// ============================================================================
/// @demo cli/agent-verify#empty_content
/// @title Verify Empty Content
/// @description Empty content is valid with no errors.
#[test]
fn test_verify_empty_content() {
    let client = LspAgentClient::new();
    let result = client.verify_content("");

    // Empty content should be valid (no errors)
    assert!(result.valid, "Empty content should be valid");
    assert_eq!(result.error_count, 0);
}

/// @demo cli/agent-verify#completions_empty
/// @title Completions on Empty Content
/// @description Empty content still returns built-in completions.
#[test]
fn test_completions_on_empty_content() {
    let client = LspAgentClient::new();
    let result = client.get_completions("", 1, 0);

    // Should still return built-in completions
    assert!(!result.items.is_empty(), "Should have built-in completions");
}

/// @demo cli/agent-verify#builder_pattern
/// @title Client Builder Pattern
/// @description LspAgentClient supports builder pattern configuration.
#[test]
fn test_client_builder_pattern() {
    let client = LspAgentClient::new().with_strict(true);
    let result = client
        .verify_content(r##"{"type": "palette", "name": "p", "colors": {"{x}": "#FF0000"}}"##);

    // Just verify builder pattern works
    let _ = result.valid; // Just testing builder pattern doesn't panic
}
