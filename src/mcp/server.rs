//! Core MCP server implementation.

use std::future::Future;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt};

use super::resources;

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

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        let result = resources::read_static_resource(&request.uri);
        std::future::ready(match result {
            Some(r) => Ok(r),
            None => Err(McpError::new(
                ErrorCode::INVALID_PARAMS,
                format!("Unknown resource URI: {}", request.uri),
                None,
            )),
        })
    }
}

/// Run the MCP server on stdin/stdout
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let server = PixelsrcMcpServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
