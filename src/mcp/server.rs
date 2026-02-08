//! Core MCP server implementation.

use std::future::Future;
use std::path::PathBuf;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt};

use super::resources;
use super::tools::{
    analyze::AnalyzeInput, format::FormatInput, import::ImportInput, palettes::PalettesInput,
    prime::PrimeInput, scaffold::ScaffoldInput,
};
use crate::analyze::{collect_files, AnalysisReport};
use crate::palettes;
use crate::prime::{get_primer, list_sections, PrimerSection};
use crate::scaffold;

/// The Pixelsrc MCP Server
///
/// Exposes pixelsrc capabilities (render, validate, explain, etc.) as MCP tools
/// and static resources (format spec, palette catalog) that AI models can access
/// over the Model Context Protocol.
#[derive(Debug, Clone)]
pub struct PixelsrcMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PixelsrcMcpServer {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    // ── pixelsrc_format ──────────────────────────────────────────────

    #[tool(
        name = "pixelsrc_format",
        description = "Format .pxl source for readability. Expands sprite regions, \
                        composition layer maps, and normalises whitespace between objects."
    )]
    pub fn pixelsrc_format(
        &self,
        Parameters(input): Parameters<FormatInput>,
    ) -> Result<String, String> {
        crate::fmt::format_pixelsrc(&input.source)
    }

    // ── pixelsrc_prime ───────────────────────────────────────────────

    #[tool(
        name = "pixelsrc_prime",
        description = "Return the .pxl format guide text. Use brief=true for a compact reference, \
                        or specify a section (format, examples, tips, full) for detailed content."
    )]
    pub fn pixelsrc_prime(
        &self,
        Parameters(input): Parameters<PrimeInput>,
    ) -> Result<String, String> {
        let section = match input.section.as_deref() {
            None => PrimerSection::Full,
            Some(s) => s.parse::<PrimerSection>().map_err(|e| {
                format!("{}\nAvailable sections: {}", e, list_sections().join(", "))
            })?,
        };

        Ok(get_primer(section, input.brief).to_string())
    }

    // ── pixelsrc_palettes ────────────────────────────────────────────

    #[tool(
        name = "pixelsrc_palettes",
        description = "List or show built-in palettes. Use action=\"list\" to get all palette names, \
                        or action=\"show\" with a name to get the full color mapping as JSON."
    )]
    pub fn pixelsrc_palettes(
        &self,
        Parameters(input): Parameters<PalettesInput>,
    ) -> Result<String, String> {
        match input.action.as_str() {
            "list" => {
                let names = palettes::list_builtins();
                let json = serde_json::json!({
                    "palettes": names.iter().map(|n| format!("@{}", n)).collect::<Vec<_>>(),
                });
                serde_json::to_string_pretty(&json)
                    .map_err(|e| format!("JSON serialization error: {}", e))
            }
            "show" => {
                let name = input
                    .name
                    .as_deref()
                    .ok_or_else(|| "name is required when action is \"show\"".to_string())?;
                let palette_name = name.strip_prefix('@').unwrap_or(name);

                match palettes::get_builtin(palette_name) {
                    Some(palette) => {
                        let mut colors = serde_json::Map::new();
                        let mut sorted: Vec<_> = palette.colors.iter().collect();
                        sorted.sort_by_key(|(k, _)| *k);
                        for (key, value) in sorted {
                            colors.insert(key.clone(), serde_json::Value::String(value.clone()));
                        }
                        let json = serde_json::json!({
                            "name": format!("@{}", palette_name),
                            "colors": colors,
                        });
                        serde_json::to_string_pretty(&json)
                            .map_err(|e| format!("JSON serialization error: {}", e))
                    }
                    None => {
                        let available = palettes::list_builtins();
                        Err(format!(
                            "Unknown palette '{}'. Available: {}",
                            name,
                            available
                                .iter()
                                .map(|n| format!("@{}", n))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    }
                }
            }
            other => Err(format!("Unknown action '{}'. Use \"list\" or \"show\".", other)),
        }
    }

    // ── pixelsrc_analyze ─────────────────────────────────────────────

    #[tool(
        name = "pixelsrc_analyze",
        description = "Analyze .pxl source or files and return corpus metrics as JSON. \
                        Provide either inline source text or a file/directory path. \
                        Returns sprite counts, token stats, dimension distribution, and more."
    )]
    pub fn pixelsrc_analyze(
        &self,
        Parameters(input): Parameters<AnalyzeInput>,
    ) -> Result<String, String> {
        let mut report = AnalysisReport::new();

        if let Some(source) = &input.source {
            let tmp = std::env::temp_dir().join("pixelsrc_mcp_analyze.pxl");
            std::fs::write(&tmp, source)
                .map_err(|e| format!("Failed to write temp file: {}", e))?;
            let result = report.analyze_file(&tmp);
            let _ = std::fs::remove_file(&tmp);
            result.map_err(|e| format!("Analysis failed: {}", e))?;
        } else if let Some(path_str) = &input.path {
            let path = PathBuf::from(path_str);
            if path.is_file() {
                report.analyze_file(&path).map_err(|e| format!("Analysis failed: {}", e))?;
            } else if path.is_dir() {
                let files = collect_files(&[], Some(&path), input.recursive)
                    .map_err(|e| format!("File collection failed: {}", e))?;
                for file in &files {
                    if let Err(e) = report.analyze_file(file) {
                        report.files_failed += 1;
                        report.failed_files.push((file.clone(), e));
                    }
                }
            } else {
                return Err(format!("Path not found: {}", path_str));
            }
        } else {
            return Err(
                "Provide either 'source' (inline .pxl text) or 'path' (file/directory path)."
                    .to_string(),
            );
        }

        let top_tokens: Vec<serde_json::Value> = report
            .token_counter
            .top_n(20)
            .iter()
            .map(|(token, count)| {
                serde_json::json!({
                    "token": token,
                    "count": count,
                    "percentage": report.token_counter.percentage(token),
                })
            })
            .collect();

        let co_occurrence: Vec<serde_json::Value> = report
            .co_occurrence
            .top_n(10)
            .iter()
            .map(|((t1, t2), count)| serde_json::json!({ "pair": [t1, t2], "count": count }))
            .collect();

        let dimensions: Vec<serde_json::Value> = report
            .dimension_stats
            .sorted_by_frequency()
            .iter()
            .take(10)
            .map(|((w, h), count)| serde_json::json!({ "size": [w, h], "count": count }))
            .collect();

        let families: Vec<serde_json::Value> = report
            .token_families()
            .iter()
            .take(10)
            .map(|f| {
                serde_json::json!({
                    "prefix": f.prefix,
                    "tokens": f.tokens,
                    "total_count": f.total_count,
                })
            })
            .collect();

        let json = serde_json::json!({
            "files_analyzed": report.files_analyzed,
            "files_failed": report.files_failed,
            "total_sprites": report.total_sprites,
            "total_palettes": report.total_palettes,
            "total_compositions": report.total_compositions,
            "total_animations": report.total_animations,
            "total_variants": report.total_variants,
            "unique_tokens": report.token_counter.unique_count(),
            "total_token_occurrences": report.token_counter.total(),
            "avg_palette_size": report.avg_palette_size(),
            "top_tokens": top_tokens,
            "co_occurrence": co_occurrence,
            "dimensions": dimensions,
            "token_families": families,
        });

        serde_json::to_string_pretty(&json).map_err(|e| format!("JSON serialization error: {}", e))
    }

    // ── pixelsrc_scaffold ────────────────────────────────────────────

    #[tool(
        name = "pixelsrc_scaffold",
        description = "Generate a skeleton .pxl source for a given asset type. \
                        Supports: sprite, animation, palette, composition. \
                        Returns the .pxl content as a string (does not write to disk)."
    )]
    pub fn pixelsrc_scaffold(
        &self,
        Parameters(input): Parameters<ScaffoldInput>,
    ) -> Result<String, String> {
        let width = input.width.unwrap_or(16);
        let height = input.height.unwrap_or(16);

        match input.asset_type.to_lowercase().as_str() {
            "sprite" => {
                let tokens: Vec<String> = input
                    .tokens
                    .as_deref()
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();

                Ok(scaffold::generate_sprite(
                    &input.name,
                    width,
                    height,
                    input.palette.as_deref(),
                    &tokens,
                ))
            }
            "animation" | "anim" => {
                let palette_name = input.palette.as_deref().unwrap_or("main");
                let frame1 = format!("{}_1", input.name);
                let frame2 = format!("{}_2", input.name);

                let tokens: Vec<String> = input
                    .tokens
                    .as_deref()
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();

                let mut output =
                    scaffold::generate_sprite(&frame1, width, height, Some(palette_name), &tokens);

                output.push_str(&format!(
                    "\n{{\"type\": \"sprite\", \"name\": \"{}\", \"size\": [{}, {}], \
                     \"palette\": \"{}_palette\", \
                     \"regions\": {{\"_\": {{\"rect\": [0, 0, {}, {}]}}}}}}\n",
                    frame2, width, height, palette_name, width, height
                ));

                output.push_str(&format!(
                    "\n{{\"type\": \"animation\", \"name\": \"{}\", \
                     \"frames\": [\"{}\", \"{}\"], \"duration\": 200}}\n",
                    input.name, frame1, frame2
                ));

                Ok(output)
            }
            "palette" => scaffold::generate_palette_scaffold(
                &input.name,
                input.palette.as_deref(),
                input.colors.as_deref(),
                "c",
            ),
            "composition" => {
                let cell_w = input.cell_width.unwrap_or(8);
                let cell_h = input.cell_height.unwrap_or(8);
                scaffold::generate_composition(
                    &input.name,
                    width,
                    height,
                    cell_w,
                    cell_h,
                    input.palette.as_deref(),
                )
            }
            other => Err(format!(
                "Unknown asset type '{}'. Available: sprite, animation, palette, composition",
                other
            )),
        }
    }

    // ── pixelsrc_import ──────────────────────────────────────────────

    /// Convert a PNG image to .pxl source format. Accepts base64 PNG or file
    /// path, runs color quantization, and generates .pxl JSONL output.
    #[tool(
        description = "Convert a PNG image to .pxl source format. Accepts base64 PNG or file path, runs color quantization, and generates .pxl JSONL output."
    )]
    fn pixelsrc_import(
        &self,
        Parameters(input): Parameters<ImportInput>,
    ) -> Result<String, String> {
        super::tools::import::run_import(input)
    }
}

#[tool_handler]
impl ServerHandler for PixelsrcMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().enable_resources().build(),
            server_info: Implementation {
                name: "pixelsrc-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Pixelsrc MCP server — render, validate, explain, and manipulate \
                 pixel art in the .pxl format. Use pixelsrc_render to generate PNGs, \
                 pixelsrc_validate to check source, and pixelsrc_prime to get the \
                 format reference."
                    .into(),
            ),
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult {
            meta: None,
            resources: resources::list_static_resources(),
            next_cursor: None,
        }))
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourceTemplatesResult {
            meta: None,
            resource_templates: resources::list_resource_templates(),
            next_cursor: None,
        }))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        // Try static resources first, then template-based dynamic resources.
        let uri = &request.uri;

        if let Some(result) = resources::read_static_resource(uri) {
            return std::future::ready(Ok(result));
        }

        if let Some(result) = resources::read_template_resource(uri) {
            return std::future::ready(match result {
                Ok(r) => Ok(r),
                Err(msg) => Err(McpError::new(ErrorCode::INVALID_PARAMS, msg, None)),
            });
        }

        std::future::ready(Err(McpError::new(
            ErrorCode::INVALID_PARAMS,
            format!("Unknown resource URI: {}", request.uri),
            None,
        )))
    }
}

/// Run the MCP server on stdin/stdout
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let server = PixelsrcMcpServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
