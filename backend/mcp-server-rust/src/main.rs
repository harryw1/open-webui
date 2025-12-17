use anyhow::Result;
use rmcp::{
    ErrorData as McpError,
    model::*,
    tool, tool_router,
    handler::server::{
        tool::{ToolRouter, ToolCallContext},
        wrapper::Parameters,
        ServerHandler // Trait
    },
    service::{RequestContext, RoleServer},
    ServiceExt,
    transport,
    schemars::JsonSchema,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

mod tools;

#[derive(Clone)]
struct MyMcpServer {
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, JsonSchema)]
struct PathParams {
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct CmdParams {
    cmd: String,
}

#[tool_router]
impl MyMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Reads content from the local workspace.")]
    async fn read_file(&self, params: Parameters<PathParams>) -> Result<CallToolResult, McpError> {
        match tools::read_file(params.0.path) {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            }),
        }
    }

    #[tool(description = "Lists files in a directory.")]
    async fn list_directory(&self, params: Parameters<PathParams>) -> Result<CallToolResult, McpError> {
        match tools::list_directory(params.0.path) {
            Ok(entries) => {
                let text = entries.join("\n");
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            }),
        }
    }

    #[tool(description = "Executes a safe terminal command.")]
    async fn shell_command(&self, params: Parameters<CmdParams>) -> Result<CallToolResult, McpError> {
        match tools::shell_command(params.0.cmd) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            }),
        }
    }
}

impl ServerHandler for MyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "mcp-server-rust".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_call_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_call_context).await
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let _ = request; // Consume unused
        let tools = self.tool_router.list_all();
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let transport = transport::stdio();

    let server = MyMcpServer::new();
    let _ = server.serve(transport).await?;
    Ok(())
}
