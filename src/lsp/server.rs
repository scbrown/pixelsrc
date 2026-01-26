//! Core LSP server implementation.

use crate::transforms::explain_transform;
use crate::validate::{Severity, ValidationIssue, Validator};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use super::color_utils::{
    extract_colors_from_line, rgba_to_hex, rgba_to_hsl, rgba_to_rgb_functional,
};
use super::completions::{
    get_compound_completions, get_modifier_completions, get_relationship_completions,
    get_role_completions, get_shape_completions, get_state_apply_completions,
    get_state_selector_completions,
};
use super::hover::get_structured_format_hover;
use super::symbols::{
    build_variable_registry, collect_css_variables, collect_defined_tokens, extract_symbols,
    extract_variable_at_position, find_variable_definition, is_css_variable_completion_context,
    type_to_symbol_kind,
};
use super::timing_utils::{
    describe_interpolation, interpolation_to_css, parse_timing_function_context,
    render_easing_curve,
};
use super::transform_utils::parse_transform_context;
use super::types::CompletionContext;

/// The Pixelsrc Language Server
pub struct PixelsrcLanguageServer {
    client: Client,
    /// Document state tracking for open files
    documents: RwLock<HashMap<Url, String>>,
}

impl PixelsrcLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client, documents: RwLock::new(HashMap::new()) }
    }

    /// Validate document content and publish diagnostics
    async fn validate_and_publish(&self, uri: &Url, content: &str) {
        let mut validator = Validator::new();
        for (line_num, line) in content.lines().enumerate() {
            validator.validate_line(line_num + 1, line);
        }

        let diagnostics: Vec<Diagnostic> =
            validator.issues().iter().map(Self::issue_to_diagnostic).collect();

        self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }

    /// Convert a ValidationIssue to an LSP Diagnostic
    pub fn issue_to_diagnostic(issue: &ValidationIssue) -> Diagnostic {
        let severity = match issue.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        };

        // Build the message with optional suggestion
        let message = if let Some(ref suggestion) = issue.suggestion {
            format!("{} ({})", issue.message, suggestion)
        } else {
            issue.message.clone()
        };

        Diagnostic {
            range: Range {
                start: Position { line: (issue.line - 1) as u32, character: 0 },
                end: Position { line: (issue.line - 1) as u32, character: u32::MAX },
            },
            severity: Some(severity),
            code: Some(NumberOrString::String(issue.issue_type.to_string())),
            source: Some("pixelsrc".to_string()),
            message,
            ..Default::default()
        }
    }

    /// Detect completion context for the structured format
    ///
    /// Analyzes the JSON structure to determine what kind of completions should be offered.
    pub fn detect_completion_context(
        content: &str,
        line: &str,
        _char_pos: u32,
    ) -> CompletionContext {
        // Try to parse the line as JSON to determine object type
        if let Ok(obj) = serde_json::from_str::<Value>(line) {
            if let Some(obj) = obj.as_object() {
                let obj_type = obj.get("type").and_then(|t| t.as_str());

                match obj_type {
                    Some("sprite") => {
                        // Check if we're in the regions field
                        if line.contains("\"regions\"") {
                            return CompletionContext::Regions;
                        }
                    }
                    Some("palette") => {
                        if line.contains("\"roles\"") {
                            return CompletionContext::Roles;
                        }
                        if line.contains("\"relationships\"") {
                            return CompletionContext::Relationships;
                        }
                    }
                    Some("state_rules") | Some("state-rules") => {
                        if line.contains("\"apply\"") {
                            return CompletionContext::StateRuleApply;
                        }
                        return CompletionContext::StateRules;
                    }
                    _ => {}
                }
            }
        }

        // For multi-line JSON5 files, check the full content for context
        if let Ok(obj) = serde_json::from_str::<Value>(content) {
            if let Some(obj) = obj.as_object() {
                let obj_type = obj.get("type").and_then(|t| t.as_str());

                match obj_type {
                    Some("sprite") if line.contains(':') && !line.contains("\"type\"") => {
                        // Likely inside a region definition
                        return CompletionContext::RegionDef;
                    }
                    Some("palette") if line.contains(':') => {
                        if content.contains("\"roles\"") && !line.contains("\"colors\"") {
                            return CompletionContext::Roles;
                        }
                    }
                    _ => {}
                }
            }
        }

        CompletionContext::Other
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for PixelsrcLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "pixelsrc-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        "-".to_string(),
                        "(".to_string(),
                    ]),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client.log_message(MessageType::INFO, "Pixelsrc LSP initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store document content
        self.documents.write().unwrap().insert(uri.clone(), text.clone());

        // Validate and publish diagnostics
        self.validate_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // Get the full text from the first change (we use FULL sync)
        if let Some(change) = params.content_changes.into_iter().next() {
            // Store updated content
            self.documents.write().unwrap().insert(uri.clone(), change.text.clone());

            // Validate and publish diagnostics
            self.validate_and_publish(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from tracking
        self.documents.write().unwrap().remove(&uri);

        // Clear diagnostics for closed document
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents); // Release lock before async work

        // Get the line at the cursor position
        let line = match content.lines().nth(pos.line as usize) {
            Some(l) => l,
            None => return Ok(None),
        };

        // Check for structured format elements (roles, shapes, modifiers)
        if let Some(hover_text) = get_structured_format_hover(line, pos.character) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: None,
            }));
        }

        // Check for CSS variable reference
        if let Some(var_name) = extract_variable_at_position(line, pos.character) {
            let css_variables = collect_css_variables(&content);
            let registry = build_variable_registry(&content);

            // Find the variable's info
            let var_info = css_variables.iter().find(|(name, _, _, _)| name == &var_name);

            if let Some((_, raw_value, _, palette_name)) = var_info {
                // Try to resolve the variable
                let resolved = registry.resolve_var(&var_name);

                let hover_text = match resolved {
                    Ok(resolved_value) => {
                        if &resolved_value == raw_value {
                            format!(
                                "**CSS Variable**: `{}`\n\n\
                                 **Value**: `{}`\n\n\
                                 **Palette**: `{}`",
                                var_name, raw_value, palette_name
                            )
                        } else {
                            format!(
                                "**CSS Variable**: `{}`\n\n\
                                 **Raw Value**: `{}`\n\n\
                                 **Resolved**: `{}`\n\n\
                                 **Palette**: `{}`",
                                var_name, raw_value, resolved_value, palette_name
                            )
                        }
                    }
                    Err(e) => {
                        format!(
                            "**CSS Variable**: `{}`\n\n\
                             **Raw Value**: `{}`\n\n\
                             **Error**: {}\n\n\
                             **Palette**: `{}`",
                            var_name, raw_value, e, palette_name
                        )
                    }
                };

                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: None,
                }));
            } else {
                // Variable referenced but not defined
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "**CSS Variable**: `{}`\n\n\
                             ⚠ **Undefined** - This variable is not defined in any palette",
                            var_name
                        ),
                    }),
                    range: None,
                }));
            }
        }

        // Try to parse timing function context at the cursor position
        if let Some(timing_info) = parse_timing_function_context(line, pos.character) {
            // Render the ASCII easing curve (25 chars wide, 8 rows tall)
            let curve = render_easing_curve(&timing_info.interpolation, 25, 8);
            let description = describe_interpolation(&timing_info.interpolation);
            let css_form = interpolation_to_css(&timing_info.interpolation);

            let hover_text = format!(
                "**Timing Function**: `{}`\n\n\
                 **Type**: {}\n\n\
                 **CSS**: `{}`\n\n\
                 ```\n{}\n```",
                timing_info.function_str, description, css_form, curve,
            );

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: None,
            }));
        }

        // Try to parse transform context at the cursor position
        if let Some(transform_info) = parse_transform_context(line, pos.character) {
            let explanation = explain_transform(&transform_info.transform);

            // Build the hover text with context
            let position_text = if transform_info.total == 1 {
                String::new()
            } else {
                format!(
                    "\n\n**Position**: {} of {} transforms",
                    transform_info.index + 1,
                    transform_info.total
                )
            };

            let hover_text = format!(
                "**Transform**: `{}`\n\n\
                 **Effect**: {}\n\n\
                 **Applied to**: {} `{}`{}",
                transform_info.raw,
                explanation,
                transform_info.object_type,
                transform_info.object_name,
                position_text,
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Get the current line
        let current_line = content.lines().nth(pos.line as usize).unwrap_or("");

        // Check if we're in a CSS variable completion context (inside var())
        if is_css_variable_completion_context(current_line, pos.character) {
            let css_variables = collect_css_variables(&content);
            let registry = build_variable_registry(&content);

            let mut completions: Vec<CompletionItem> = Vec::new();

            for (var_name, raw_value, _, palette_name) in css_variables {
                // Try to resolve the variable for the detail
                let resolved = registry.resolve_var(&var_name);
                let detail = match resolved {
                    Ok(resolved_value) => {
                        if resolved_value == raw_value {
                            format!("{} ({})", raw_value, palette_name)
                        } else {
                            format!("{} → {} ({})", raw_value, resolved_value, palette_name)
                        }
                    }
                    Err(_) => format!("{} ({})", raw_value, palette_name),
                };

                completions.push(CompletionItem {
                    label: var_name.clone(),
                    detail: Some(detail),
                    kind: Some(CompletionItemKind::VARIABLE),
                    insert_text: Some(var_name),
                    ..Default::default()
                });
            }

            return Ok(Some(CompletionResponse::Array(completions)));
        }

        // Detect structured format completion context
        let context = Self::detect_completion_context(&content, current_line, pos.character);

        let mut completions: Vec<CompletionItem> = Vec::new();

        match context {
            CompletionContext::Regions | CompletionContext::RegionDef => {
                // Inside regions - offer shape primitives, compounds, and modifiers
                completions.extend(get_shape_completions());
                completions.extend(get_compound_completions());
                completions.extend(get_modifier_completions());
            }
            CompletionContext::Roles => {
                // Inside roles - offer role values
                completions.extend(get_role_completions());
            }
            CompletionContext::Relationships => {
                // Inside relationships - offer relationship types
                completions.extend(get_relationship_completions());
            }
            CompletionContext::StateRules => {
                // Inside state rules - offer selector patterns
                completions.extend(get_state_selector_completions());
            }
            CompletionContext::StateRuleApply => {
                // Inside state rule apply - offer applicable properties
                completions.extend(get_state_apply_completions());
            }
            CompletionContext::Other => {
                // Fall back to token completions for palettes
                let defined_tokens = collect_defined_tokens(&content);

                // Add built-in transparent token (v2 format uses bare token names)
                completions.push(CompletionItem {
                    label: "_".to_string(),
                    detail: Some("Transparent (built-in)".to_string()),
                    kind: Some(CompletionItemKind::COLOR),
                    insert_text: Some("_".to_string()),
                    ..Default::default()
                });

                // Add defined tokens from palettes
                for (token, color, role) in defined_tokens {
                    // Strip braces for v2 format display
                    let display_token = token.trim_start_matches('{').trim_end_matches('}');
                    let detail = match role {
                        Some(r) => format!("{} ({})", color, r),
                        None => color,
                    };
                    completions.push(CompletionItem {
                        label: display_token.to_string(),
                        detail: Some(detail),
                        kind: Some(CompletionItemKind::COLOR),
                        insert_text: Some(display_token.to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Extract symbols using helper method
        let extracted = extract_symbols(&content);

        // Convert to SymbolInformation
        let symbols: Vec<SymbolInformation> = extracted
            .into_iter()
            .map(|(name, obj_type, line_num)| {
                let line = content.lines().nth(line_num).unwrap_or("");
                #[allow(deprecated)]
                SymbolInformation {
                    name,
                    kind: type_to_symbol_kind(&obj_type),
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                    },
                    container_name: None,
                }
            })
            .collect();

        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = &params.text_document.uri;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(Vec::new()),
        };
        drop(documents);

        // Build variable registry from all palettes for var() resolution
        let var_registry = build_variable_registry(&content);

        // Extract all colors from palette definitions
        let mut colors = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_colors = extract_colors_from_line(line, line_num as u32, &var_registry);
            for (color_match, line_idx) in line_colors {
                colors.push(ColorInformation {
                    range: Range {
                        start: Position { line: line_idx, character: color_match.start },
                        end: Position { line: line_idx, character: color_match.end },
                    },
                    color: Color {
                        red: color_match.rgba.0,
                        green: color_match.rgba.1,
                        blue: color_match.rgba.2,
                        alpha: color_match.rgba.3,
                    },
                });
            }
        }

        Ok(colors)
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let color = params.color;
        let r = color.red;
        let g = color.green;
        let b = color.blue;
        let a = color.alpha;

        // Provide multiple format options when user picks a color
        let mut presentations = Vec::new();

        // Hex format (most common for pixel art)
        let hex = rgba_to_hex(r, g, b, a);
        presentations.push(ColorPresentation {
            label: hex.clone(),
            text_edit: Some(TextEdit { range: params.range, new_text: hex }),
            additional_text_edits: None,
        });

        // RGB functional format
        let rgb = rgba_to_rgb_functional(r, g, b, a);
        presentations.push(ColorPresentation {
            label: rgb.clone(),
            text_edit: Some(TextEdit { range: params.range, new_text: rgb }),
            additional_text_edits: None,
        });

        // HSL format
        let hsl = rgba_to_hsl(r, g, b, a);
        presentations.push(ColorPresentation {
            label: hsl.clone(),
            text_edit: Some(TextEdit { range: params.range, new_text: hsl }),
            additional_text_edits: None,
        });

        Ok(presentations)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        // Get the document content
        let documents = self.documents.read().unwrap();
        let content = match documents.get(uri) {
            Some(c) => c.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Get the line at the cursor position
        let line = match content.lines().nth(pos.line as usize) {
            Some(l) => l,
            None => return Ok(None),
        };

        // Check if cursor is on a CSS variable reference
        if let Some(var_name) = extract_variable_at_position(line, pos.character) {
            // Find where this variable is defined
            if let Some((def_line, start_char, end_char)) =
                find_variable_definition(&content, &var_name)
            {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position { line: def_line as u32, character: start_char },
                        end: Position { line: def_line as u32, character: end_char },
                    },
                })));
            }
        }

        Ok(None)
    }
}

/// Run the LSP server on stdin/stdout
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(PixelsrcLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
