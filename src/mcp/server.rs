//! Core MCP server implementation.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::*;
use rmcp::{tool_handler, tool_router, ServerHandler, ServiceExt};

/// The Pixelsrc MCP Server
///
/// Exposes pixelsrc capabilities (render, validate, explain, etc.) as MCP tools
/// that AI models can call over the Model Context Protocol.
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
            capabilities: ServerCapabilities::builder().enable_tools().build(),
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
}

/// Run the MCP server on stdin/stdout
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let server = PixelsrcMcpServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
