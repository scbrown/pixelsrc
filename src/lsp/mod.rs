//! Language Server Protocol implementation for Pixelsrc
//!
//! Provides LSP support for .pxl files in editors like VS Code, Neovim, etc.
//!
//! # Structured Format Support
//!
//! This LSP provides intelligent completions for the structured region format:
//! - Shape primitives: `points`, `line`, `rect`, `stroke`, `ellipse`, `circle`, `polygon`, `path`, `fill`
//! - Modifiers: `symmetric`, `z`, `round`, `thickness`, `repeat`, `spacing`, `transform`, `jitter`
//! - Constraints: `within`, `adjacent-to`, `x`, `y`
//! - Roles: `boundary`, `anchor`, `fill`, `shadow`, `highlight`
//! - Relationships: `derives-from`, `contained-within`, `adjacent-to`, `paired-with`

mod color_utils;
mod completions;
mod hover;
mod server;
mod symbols;
mod timing_utils;
mod transform_utils;
mod types;

// Re-export public items
pub use server::{run_server, PixelsrcLanguageServer};
pub use types::{ColorMatch, CompletionContext, TimingFunctionInfo, TransformInfo};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::{Interpolation, StepPosition};
    use crate::validate::{IssueType, ValidationIssue};
    use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, SymbolKind};

    #[test]
    fn test_issue_to_diagnostic_error() {
        let issue = ValidationIssue::error(5, IssueType::JsonSyntax, "Invalid JSON");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 4); // 0-indexed
        assert_eq!(diagnostic.range.end.line, 4);
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostic.message, "Invalid JSON");
        assert_eq!(diagnostic.source, Some("pixelsrc".to_string()));
        assert_eq!(diagnostic.code, Some(NumberOrString::String("json_syntax".to_string())));
    }

    #[test]
    fn test_issue_to_diagnostic_warning() {
        let issue = ValidationIssue::warning(10, IssueType::UndefinedToken, "Undefined token {x}");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 9); // 0-indexed
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostic.message, "Undefined token {x}");
    }

    #[test]
    fn test_issue_to_diagnostic_with_suggestion() {
        let issue = ValidationIssue::warning(3, IssueType::UndefinedToken, "Undefined token {skni}")
            .with_suggestion("did you mean {skin}?");
        let diagnostic = PixelsrcLanguageServer::issue_to_diagnostic(&issue);

        assert_eq!(diagnostic.range.start.line, 2); // 0-indexed
        assert_eq!(diagnostic.message, "Undefined token {skni} (did you mean {skin}?)");
    }

    // === Structured Format Context Detection Tests ===

    #[test]
    fn test_detect_completion_context_sprite_regions() {
        let content =
            r#"{"type": "sprite", "name": "test", "regions": {"eye": {"rect": [0, 0, 8, 8]}}}"#;
        let line = content;
        let context = PixelsrcLanguageServer::detect_completion_context(content, line, 50);
        assert_eq!(context, CompletionContext::Regions);
    }

    #[test]
    fn test_detect_completion_context_palette_roles() {
        let content = r#"{"type": "palette", "name": "test", "roles": {"eye": "anchor"}}"#;
        let line = content;
        let context = PixelsrcLanguageServer::detect_completion_context(content, line, 50);
        assert_eq!(context, CompletionContext::Roles);
    }

    #[test]
    fn test_detect_completion_context_state_rules() {
        let content = r#"{"type": "state_rules", "name": "test", "rules": []}"#;
        let line = content;
        let context = PixelsrcLanguageServer::detect_completion_context(content, line, 30);
        assert_eq!(context, CompletionContext::StateRules);
    }

    #[test]
    fn test_detect_completion_context_other() {
        let content = r#"{"type": "animation", "name": "test", "frames": []}"#;
        let line = content;
        let context = PixelsrcLanguageServer::detect_completion_context(content, line, 30);
        assert_eq!(context, CompletionContext::Other);
    }

    // === Structured Format Hover Tests ===

    #[test]
    fn test_get_structured_format_hover_role() {
        // Position on "boundary" keyword
        let line = r#"  "outline": "boundary","#;
        let hover = hover::get_structured_format_hover(line, 15);
        assert!(hover.is_some());
        assert!(hover.unwrap().contains("Role: Boundary"));
    }

    #[test]
    fn test_get_structured_format_hover_shape() {
        // Position on "rect" keyword
        let line = r#"  "eye": { "rect": [0, 0, 8, 8] }"#;
        let hover = hover::get_structured_format_hover(line, 12);
        assert!(hover.is_some());
        assert!(hover.unwrap().contains("Shape: Rectangle"));
    }

    #[test]
    fn test_get_structured_format_hover_modifier() {
        // Position on "symmetric" keyword
        let line = r#"  "symmetric": "x","#;
        let hover = hover::get_structured_format_hover(line, 5);
        assert!(hover.is_some());
        assert!(hover.unwrap().contains("Modifier: Symmetric"));
    }

    #[test]
    fn test_get_structured_format_hover_no_match() {
        // Position on a value, not a keyword
        let line = r#"  "name": "test","#;
        let hover = hover::get_structured_format_hover(line, 12);
        // "test" is not a recognized keyword
        assert!(hover.is_none());
    }

    // === Shape Completions Tests ===

    #[test]
    fn test_get_shape_completions() {
        let completions = completions::get_shape_completions();
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "rect"));
        assert!(completions.iter().any(|c| c.label == "circle"));
        assert!(completions.iter().any(|c| c.label == "polygon"));
    }

    #[test]
    fn test_get_role_completions() {
        let completions = completions::get_role_completions();
        assert_eq!(completions.len(), 5);
        assert!(completions.iter().any(|c| c.label == "boundary"));
        assert!(completions.iter().any(|c| c.label == "anchor"));
        assert!(completions.iter().any(|c| c.label == "fill"));
    }

    #[test]
    fn test_get_modifier_completions() {
        let completions = completions::get_modifier_completions();
        assert!(completions.iter().any(|c| c.label == "symmetric"));
        assert!(completions.iter().any(|c| c.label == "within"));
        assert!(completions.iter().any(|c| c.label == "z"));
    }

    #[test]
    fn test_collect_defined_tokens_single_palette() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00", "--var": "100"}}"##;
        let tokens = symbols::collect_defined_tokens(content);

        assert_eq!(tokens.len(), 2); // Only {a} and {b}, not --var
        assert!(tokens.iter().any(|(t, c, _)| t == "{a}" && c == "#FF0000"));
        assert!(tokens.iter().any(|(t, c, _)| t == "{b}" && c == "#00FF00"));
    }

    #[test]
    fn test_collect_defined_tokens_multiple_palettes() {
        let content = r##"{"type": "palette", "name": "p1", "colors": {"{red}": "#FF0000"}}
{"type": "palette", "name": "p2", "colors": {"{blue}": "#0000FF"}}
{"type": "sprite", "name": "s", "size": [2,1], "regions": {"red": {"points": [[0,0]]}, "blue": {"points": [[1,0]]}}}"##;
        let tokens = symbols::collect_defined_tokens(content);

        assert_eq!(tokens.len(), 2);
        assert!(tokens.iter().any(|(t, _, _)| t == "{red}"));
        assert!(tokens.iter().any(|(t, _, _)| t == "{blue}"));
    }

    #[test]
    fn test_collect_defined_tokens_no_palettes() {
        let content = r#"{"type": "sprite", "name": "s", "size": [2,1], "regions": {"a": {"points": [[0,0]]}, "b": {"points": [[1,0]]}}}"#;
        let tokens = symbols::collect_defined_tokens(content);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_collect_defined_tokens_empty_content() {
        let tokens = symbols::collect_defined_tokens("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_collect_defined_tokens_with_roles() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"{outline}": "#000000", "{fill}": "#FF0000"}, "roles": {"{outline}": "boundary", "{fill}": "fill"}}"##;
        let tokens = symbols::collect_defined_tokens(content);

        assert_eq!(tokens.len(), 2);
        let outline_token = tokens.iter().find(|(t, _, _)| t == "{outline}");
        assert!(outline_token.is_some());
        assert_eq!(outline_token.unwrap().2, Some("boundary".to_string()));

        let fill_token = tokens.iter().find(|(t, _, _)| t == "{fill}");
        assert!(fill_token.is_some());
        assert_eq!(fill_token.unwrap().2, Some("fill".to_string()));
    }

    #[test]
    fn test_collect_defined_tokens_partial_roles() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"{a}": "#FF0000", "{b}": "#00FF00"}, "roles": {"{a}": "anchor"}}"##;
        let tokens = symbols::collect_defined_tokens(content);

        assert_eq!(tokens.len(), 2);
        let a_token = tokens.iter().find(|(t, _, _)| t == "{a}");
        assert!(a_token.is_some());
        assert_eq!(a_token.unwrap().2, Some("anchor".to_string()));

        let b_token = tokens.iter().find(|(t, _, _)| t == "{b}");
        assert!(b_token.is_some());
        assert_eq!(b_token.unwrap().2, None); // {b} has no role
    }

    // === Document Symbol Tests ===

    #[test]
    fn test_extract_symbols_single_palette() {
        let content = r##"{"type": "palette", "name": "hero", "colors": {"{a}": "#FF0000"}}"##;
        let syms = symbols::extract_symbols(content);

        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].0, "hero");
        assert_eq!(syms[0].1, "palette");
        assert_eq!(syms[0].2, 0);
    }

    #[test]
    fn test_extract_symbols_single_sprite() {
        let content = r#"{"type": "sprite", "name": "player", "size": [1,1], "regions": {"a": {"points": [[0,0]]}}}"#;
        let syms = symbols::extract_symbols(content);

        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].0, "player");
        assert_eq!(syms[0].1, "sprite");
        assert_eq!(syms[0].2, 0);
    }

    #[test]
    fn test_extract_symbols_animation() {
        let content = r#"{"type": "animation", "name": "walk_cycle", "frames": ["frame1"]}"#;
        let syms = symbols::extract_symbols(content);

        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].0, "walk_cycle");
        assert_eq!(syms[0].1, "animation");
    }

    #[test]
    fn test_extract_symbols_composition() {
        let content = r#"{"type": "composition", "name": "scene1", "layers": []}"#;
        let syms = symbols::extract_symbols(content);

        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].0, "scene1");
        assert_eq!(syms[0].1, "composition");
    }

    #[test]
    fn test_extract_symbols_multiple_objects() {
        let content = r##"{"type": "palette", "name": "colors", "colors": {"{a}": "#FF0000"}}
{"type": "sprite", "name": "hero", "size": [1,1], "regions": {"a": {"points": [[0,0]]}}}
{"type": "sprite", "name": "enemy", "size": [1,1], "regions": {"a": {"points": [[0,0]]}}}
{"type": "animation", "name": "idle", "frames": ["hero"]}"##;
        let syms = symbols::extract_symbols(content);

        assert_eq!(syms.len(), 4);

        // Check names in order
        assert_eq!(syms[0].0, "colors");
        assert_eq!(syms[0].1, "palette");
        assert_eq!(syms[0].2, 0);

        assert_eq!(syms[1].0, "hero");
        assert_eq!(syms[1].1, "sprite");
        assert_eq!(syms[1].2, 1);

        assert_eq!(syms[2].0, "enemy");
        assert_eq!(syms[2].1, "sprite");
        assert_eq!(syms[2].2, 2);

        assert_eq!(syms[3].0, "idle");
        assert_eq!(syms[3].1, "animation");
        assert_eq!(syms[3].2, 3);
    }

    #[test]
    fn test_extract_symbols_skips_invalid_json() {
        let content = r##"this is not json
{"type": "palette", "name": "valid", "colors": {}}
also not json"##;
        let syms = symbols::extract_symbols(content);

        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].0, "valid");
        assert_eq!(syms[0].2, 1); // Line 1 (0-indexed)
    }

    #[test]
    fn test_extract_symbols_skips_missing_type() {
        let content = r#"{"name": "no_type", "colors": {}}"#;
        let syms = symbols::extract_symbols(content);
        assert!(syms.is_empty());
    }

    #[test]
    fn test_extract_symbols_skips_missing_name() {
        let content = r#"{"type": "palette", "colors": {}}"#;
        let syms = symbols::extract_symbols(content);
        assert!(syms.is_empty());
    }

    #[test]
    fn test_extract_symbols_empty_content() {
        let syms = symbols::extract_symbols("");
        assert!(syms.is_empty());
    }

    #[test]
    fn test_type_to_symbol_kind_palette() {
        assert_eq!(symbols::type_to_symbol_kind("palette"), SymbolKind::CONSTANT);
    }

    #[test]
    fn test_type_to_symbol_kind_sprite() {
        assert_eq!(symbols::type_to_symbol_kind("sprite"), SymbolKind::CLASS);
    }

    #[test]
    fn test_type_to_symbol_kind_animation() {
        assert_eq!(symbols::type_to_symbol_kind("animation"), SymbolKind::FUNCTION);
    }

    #[test]
    fn test_type_to_symbol_kind_composition() {
        assert_eq!(symbols::type_to_symbol_kind("composition"), SymbolKind::MODULE);
    }

    #[test]
    fn test_type_to_symbol_kind_unknown() {
        assert_eq!(symbols::type_to_symbol_kind("unknown"), SymbolKind::OBJECT);
    }

    // === CSS Variable Tests ===

    #[test]
    fn test_collect_css_variables_single_palette() {
        let content = r##"{"type": "palette", "name": "hero", "colors": {"--primary": "#FF0000", "{body}": "var(--primary)"}}"##;
        let variables = symbols::collect_css_variables(content);

        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].0, "--primary");
        assert_eq!(variables[0].1, "#FF0000");
        assert_eq!(variables[0].2, 0); // Line 0
        assert_eq!(variables[0].3, "hero"); // Palette name
    }

    #[test]
    fn test_collect_css_variables_multiple_palettes() {
        let content = r##"{"type": "palette", "name": "p1", "colors": {"--color1": "#FF0000"}}
{"type": "palette", "name": "p2", "colors": {"--color2": "#00FF00", "--color3": "var(--color1)"}}"##;
        let variables = symbols::collect_css_variables(content);

        assert_eq!(variables.len(), 3);

        // Check that we have all variables
        assert!(variables.iter().any(|(n, _, _, _)| n == "--color1"));
        assert!(variables.iter().any(|(n, _, _, _)| n == "--color2"));
        assert!(variables.iter().any(|(n, _, _, _)| n == "--color3"));
    }

    #[test]
    fn test_collect_css_variables_no_variables() {
        let content = r##"{"type": "palette", "name": "simple", "colors": {"{red}": "#FF0000"}}"##;
        let variables = symbols::collect_css_variables(content);
        assert!(variables.is_empty());
    }

    #[test]
    fn test_build_variable_registry() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000", "--secondary": "var(--primary)"}}"##;
        let registry = symbols::build_variable_registry(content);

        assert!(registry.contains("--primary"));
        assert!(registry.contains("--secondary"));
        assert_eq!(registry.resolve_var("--primary").unwrap(), "#FF0000");
        assert_eq!(registry.resolve_var("--secondary").unwrap(), "#FF0000");
    }

    #[test]
    fn test_find_variable_definition_exists() {
        let content =
            r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000"}}"##;
        let result = symbols::find_variable_definition(content, "--primary");

        assert!(result.is_some());
        let (line, start, end) = result.unwrap();
        assert_eq!(line, 0);
        assert!(start > 0);
        assert!(end > start);
    }

    #[test]
    fn test_find_variable_definition_without_dashes() {
        let content =
            r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000"}}"##;
        // Should work even without the -- prefix
        let result = symbols::find_variable_definition(content, "primary");

        assert!(result.is_some());
    }

    #[test]
    fn test_find_variable_definition_not_found() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"{red}": "#FF0000"}}"##;
        let result = symbols::find_variable_definition(content, "--missing");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_variable_at_position_var_reference() {
        let line = r#""{body}": "var(--primary)""#;
        // Position inside var(--primary)
        let pos = line.find("--primary").unwrap() as u32;
        let result = symbols::extract_variable_at_position(line, pos);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "--primary");
    }

    #[test]
    fn test_extract_variable_at_position_var_with_fallback() {
        let line = r#""{body}": "var(--primary, #FF0000)""#;
        let pos = line.find("--primary").unwrap() as u32;
        let result = symbols::extract_variable_at_position(line, pos);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "--primary");
    }

    #[test]
    fn test_extract_variable_at_position_not_on_variable() {
        let line = r##""{body}": "#FF0000""##;
        let result = symbols::extract_variable_at_position(line, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_css_variable_completion_context_after_var_open() {
        let line = r#""{body}": "var("#;
        assert!(symbols::is_css_variable_completion_context(line, line.len() as u32));
    }

    // === Timing Function Visualization Tests ===

    #[test]
    fn test_parse_timing_function_context_ease_in() {
        let line = r#"{"type": "animation", "name": "bounce", "timing_function": "ease-in", "frames": []}"#;
        // Find position within "ease-in" value
        let value_start = line.find("\"ease-in\"").unwrap() + 1;
        let info = timing_utils::parse_timing_function_context(line, value_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.function_str, "ease-in");
        assert!(matches!(info.interpolation, Interpolation::EaseIn));
    }

    #[test]
    fn test_parse_timing_function_context_cubic_bezier() {
        let line = r#"{"type": "animation", "name": "custom", "timing_function": "cubic-bezier(0.25, 0.1, 0.25, 1.0)", "frames": []}"#;
        let value_start = line.find("\"cubic-bezier").unwrap() + 1;
        let info = timing_utils::parse_timing_function_context(line, value_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.function_str, "cubic-bezier(0.25, 0.1, 0.25, 1.0)");
        assert!(matches!(info.interpolation, Interpolation::Bezier { .. }));
    }

    #[test]
    fn test_parse_timing_function_context_steps() {
        let line = r#"{"type": "animation", "name": "step", "timing_function": "steps(4, jump-end)", "frames": []}"#;
        let value_start = line.find("\"steps").unwrap() + 1;
        let info = timing_utils::parse_timing_function_context(line, value_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.function_str, "steps(4, jump-end)");
        assert!(matches!(info.interpolation, Interpolation::Steps { count: 4, .. }));
    }

    #[test]
    fn test_is_css_variable_completion_context_after_var_dashes() {
        let line = r#""{body}": "var(--"#;
        assert!(symbols::is_css_variable_completion_context(line, line.len() as u32));
    }

    #[test]
    fn test_is_css_variable_completion_context_typing_name() {
        let line = r#""{body}": "var(--pri"#;
        assert!(symbols::is_css_variable_completion_context(line, line.len() as u32));
    }

    #[test]
    fn test_is_css_variable_completion_context_not_in_var() {
        let line = r##""{body}": "#FF0000""##;
        assert!(!symbols::is_css_variable_completion_context(line, line.len() as u32));
    }

    #[test]
    fn test_is_css_variable_completion_context_after_close() {
        let line = r#""{body}": "var(--primary)"#;
        // After the closing paren, should not be in context
        assert!(!symbols::is_css_variable_completion_context(line, line.len() as u32));
    }

    #[test]
    fn test_parse_timing_function_context_not_animation() {
        let line = r#"{"type": "sprite", "name": "test", "timing_function": "ease"}"#;
        let info = timing_utils::parse_timing_function_context(line, 50);
        assert!(info.is_none());
    }

    // === Transform Context Tests (LSP-12) ===

    #[test]
    fn test_parse_transform_context_single_transform() {
        let line = r#"{"type": "sprite", "name": "flipped", "source": "original", "transform": ["mirror-h"]}"#;
        // Find position within the "mirror-h" string
        let transform_start = line.find("[\"mirror-h\"]").unwrap() + 2;
        let info = transform_utils::parse_transform_context(line, transform_start as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "mirror-h");
        assert_eq!(info.object_type, "sprite");
        assert_eq!(info.object_name, "flipped");
        assert_eq!(info.index, 0);
        assert_eq!(info.total, 1);
    }

    #[test]
    fn test_parse_transform_context_multiple_transforms_first() {
        let line = r#"{"type": "animation", "name": "walk_left", "source": "walk", "transform": ["mirror-h", "rotate:90"]}"#;
        // Position within first transform
        let first_transform_pos = line.find("[\"mirror-h\"").unwrap() + 2;
        let info = transform_utils::parse_transform_context(line, first_transform_pos as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "mirror-h");
        assert_eq!(info.index, 0);
        assert_eq!(info.total, 2);
    }

    #[test]
    fn test_parse_transform_context_multiple_transforms_second() {
        let line = r#"{"type": "animation", "name": "walk_left", "source": "walk", "transform": ["mirror-h", "rotate:90"]}"#;
        // Position within second transform
        let second_transform_pos = line.find("\"rotate:90\"").unwrap() + 1;
        let info = transform_utils::parse_transform_context(line, second_transform_pos as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "rotate:90");
        assert_eq!(info.index, 1);
        assert_eq!(info.total, 2);
    }

    #[test]
    fn test_parse_transform_context_not_a_transform() {
        let line = r#"{"type": "sprite", "name": "test", "size": [2,1], "regions": {}}"#;
        let info = transform_utils::parse_transform_context(line, 30);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_transform_context_before_array() {
        let line = r#"{"type": "sprite", "name": "test", "transform": ["mirror-h"]}"#;
        // Position before the transform array
        let info = transform_utils::parse_transform_context(line, 10);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_timing_function_context_cursor_outside_value() {
        let line =
            r#"{"type": "animation", "name": "test", "timing_function": "ease", "frames": []}"#;
        // Position in "name" field, not timing_function
        let info = timing_utils::parse_timing_function_context(line, 20);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_transform_context_composition() {
        let line = r#"{"type": "composition", "name": "scene", "layers": [], "transform": ["scale:2.0,2.0"]}"#;
        let transform_pos = line.find("\"scale:2.0,2.0\"").unwrap() + 1;
        let info = transform_utils::parse_transform_context(line, transform_pos as u32);

        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.raw, "scale:2.0,2.0");
        assert_eq!(info.object_type, "composition");
        assert_eq!(info.object_name, "scene");
    }

    #[test]
    fn test_render_easing_curve_linear() {
        let curve = timing_utils::render_easing_curve(&Interpolation::Linear, 10, 5);
        // Linear should produce a diagonal line
        assert!(curve.contains("█"));
        assert!(curve.contains("1.0"));
        assert!(curve.contains("0.0"));
        assert!(curve.contains("→ 1"));
    }

    #[test]
    fn test_render_easing_curve_ease_in() {
        let curve = timing_utils::render_easing_curve(&Interpolation::EaseIn, 10, 5);
        // Ease-in starts slow, should have more blocks in lower rows initially
        assert!(curve.contains("█"));
    }

    #[test]
    fn test_render_easing_curve_steps() {
        let curve = timing_utils::render_easing_curve(
            &Interpolation::Steps { count: 4, position: StepPosition::JumpEnd },
            20,
            6,
        );
        // Steps should produce a staircase pattern
        assert!(curve.contains("█"));
    }

    #[test]
    fn test_describe_interpolation_all_types() {
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::Linear),
            "Constant speed (no easing)"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::EaseIn),
            "Slow start, fast end (acceleration)"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::EaseOut),
            "Fast start, slow end (deceleration)"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::EaseInOut),
            "Smooth S-curve (slow start and end)"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::Bounce),
            "Overshoot and settle back"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::Elastic),
            "Spring-like oscillation"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::Bezier {
                p1: (0.0, 0.0),
                p2: (1.0, 1.0)
            }),
            "Custom cubic bezier curve"
        );
        assert_eq!(
            timing_utils::describe_interpolation(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpEnd
            }),
            "Discrete step function"
        );
    }

    #[test]
    fn test_interpolation_to_css_named() {
        assert_eq!(timing_utils::interpolation_to_css(&Interpolation::Linear), "linear");
        assert_eq!(timing_utils::interpolation_to_css(&Interpolation::EaseIn), "ease-in");
        assert_eq!(timing_utils::interpolation_to_css(&Interpolation::EaseOut), "ease-out");
        assert_eq!(timing_utils::interpolation_to_css(&Interpolation::EaseInOut), "ease-in-out");
    }

    #[test]
    fn test_interpolation_to_css_bezier() {
        assert_eq!(
            timing_utils::interpolation_to_css(&Interpolation::Bezier {
                p1: (0.25, 0.1),
                p2: (0.25, 1.0)
            }),
            "cubic-bezier(0.25, 0.1, 0.25, 1)"
        );
    }

    #[test]
    fn test_interpolation_to_css_steps() {
        assert_eq!(
            timing_utils::interpolation_to_css(&Interpolation::Steps {
                count: 1,
                position: StepPosition::JumpEnd
            }),
            "step-end"
        );
        assert_eq!(
            timing_utils::interpolation_to_css(&Interpolation::Steps {
                count: 1,
                position: StepPosition::JumpStart
            }),
            "step-start"
        );
        assert_eq!(
            timing_utils::interpolation_to_css(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpEnd
            }),
            "steps(4)"
        );
        assert_eq!(
            timing_utils::interpolation_to_css(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpStart
            }),
            "steps(4, jump-start)"
        );
        assert_eq!(
            timing_utils::interpolation_to_css(&Interpolation::Steps {
                count: 4,
                position: StepPosition::JumpBoth
            }),
            "steps(4, jump-both)"
        );
    }

    // === Color Provider Tests ===

    #[test]
    fn test_extract_colors_from_line_hex() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{red}": "#FF0000", "{blue}": "#0000FF"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = color_utils::extract_colors_from_line(line, 0, &registry);

        assert_eq!(colors.len(), 2);
        // Check that we found the colors
        let has_red = colors.iter().any(|(c, _)| {
            (c.rgba.0 - 1.0).abs() < 0.01 && c.rgba.1.abs() < 0.01 && c.rgba.2.abs() < 0.01
        });
        let has_blue = colors.iter().any(|(c, _)| {
            c.rgba.0.abs() < 0.01 && c.rgba.1.abs() < 0.01 && (c.rgba.2 - 1.0).abs() < 0.01
        });
        assert!(has_red, "Should find red color");
        assert!(has_blue, "Should find blue color");
    }

    #[test]
    fn test_extract_colors_from_line_css_functions() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{red}": "rgb(255, 0, 0)", "{green}": "hsl(120, 100%, 50%)"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = color_utils::extract_colors_from_line(line, 0, &registry);

        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_extract_colors_from_line_with_vars() {
        let content = r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000", "{red}": "var(--primary)"}}"##;
        let registry = symbols::build_variable_registry(content);
        let colors = color_utils::extract_colors_from_line(content, 0, &registry);

        // Should have 1 color (the {red} token, --primary is skipped as a variable definition)
        assert_eq!(colors.len(), 1);
        // The resolved color should be red
        let (color, _) = &colors[0];
        assert!((color.rgba.0 - 1.0).abs() < 0.01, "Red component should be 1.0");
        assert!(color.rgba.1.abs() < 0.01, "Green component should be 0.0");
        assert!(color.rgba.2.abs() < 0.01, "Blue component should be 0.0");
    }

    #[test]
    fn test_extract_colors_from_line_color_mix() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"{purple}": "color-mix(in srgb, red 50%, blue)"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = color_utils::extract_colors_from_line(line, 0, &registry);

        assert_eq!(colors.len(), 1);
        // Color should be a purple-ish mix
        let (color, _) = &colors[0];
        assert!(color.rgba.0 > 0.4, "Should have red component");
        assert!(color.rgba.2 > 0.4, "Should have blue component");
    }

    #[test]
    fn test_extract_colors_from_line_not_palette() {
        let line = r#"{"type": "sprite", "name": "test", "size": [1,1], "regions": {}}"#;
        let registry = crate::variables::VariableRegistry::new();
        let colors = color_utils::extract_colors_from_line(line, 0, &registry);

        assert!(colors.is_empty(), "Should not extract colors from sprites");
    }

    #[test]
    fn test_extract_colors_skips_css_vars() {
        let line = r##"{"type": "palette", "name": "test", "colors": {"--primary": "#FF0000"}}"##;
        let registry = crate::variables::VariableRegistry::new();
        let colors = color_utils::extract_colors_from_line(line, 0, &registry);

        // CSS variable definitions should be skipped
        assert!(colors.is_empty(), "Should skip CSS variable definitions");
    }

    #[test]
    fn test_rgba_to_hex_no_alpha() {
        let hex = color_utils::rgba_to_hex(1.0, 0.0, 0.0, 1.0);
        assert_eq!(hex, "#FF0000");

        let hex = color_utils::rgba_to_hex(0.0, 1.0, 0.0, 1.0);
        assert_eq!(hex, "#00FF00");

        let hex = color_utils::rgba_to_hex(0.0, 0.0, 1.0, 1.0);
        assert_eq!(hex, "#0000FF");
    }

    #[test]
    fn test_rgba_to_hex_with_alpha() {
        let hex = color_utils::rgba_to_hex(1.0, 0.0, 0.0, 0.5);
        assert_eq!(hex, "#FF000080");

        let hex = color_utils::rgba_to_hex(1.0, 1.0, 1.0, 0.0);
        assert_eq!(hex, "#FFFFFF00");
    }

    #[test]
    fn test_rgba_to_rgb_functional() {
        let rgb = color_utils::rgba_to_rgb_functional(1.0, 0.0, 0.0, 1.0);
        assert_eq!(rgb, "rgb(255, 0, 0)");

        let rgba = color_utils::rgba_to_rgb_functional(1.0, 0.0, 0.0, 0.5);
        assert_eq!(rgba, "rgba(255, 0, 0, 0.50)");
    }

    #[test]
    fn test_rgba_to_hsl() {
        // Pure red
        let hsl = color_utils::rgba_to_hsl(1.0, 0.0, 0.0, 1.0);
        assert_eq!(hsl, "hsl(0, 100%, 50%)");

        // Pure green
        let hsl = color_utils::rgba_to_hsl(0.0, 1.0, 0.0, 1.0);
        assert_eq!(hsl, "hsl(120, 100%, 50%)");

        // Pure blue
        let hsl = color_utils::rgba_to_hsl(0.0, 0.0, 1.0, 1.0);
        assert_eq!(hsl, "hsl(240, 100%, 50%)");

        // White
        let hsl = color_utils::rgba_to_hsl(1.0, 1.0, 1.0, 1.0);
        assert_eq!(hsl, "hsl(0, 0%, 100%)");

        // Black
        let hsl = color_utils::rgba_to_hsl(0.0, 0.0, 0.0, 1.0);
        assert_eq!(hsl, "hsl(0, 0%, 0%)");
    }

    #[test]
    fn test_rgba_to_hsl_with_alpha() {
        let hsla = color_utils::rgba_to_hsl(1.0, 0.0, 0.0, 0.5);
        assert_eq!(hsla, "hsla(0, 100%, 50%, 0.50)");
    }
}
