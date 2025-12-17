use anyhow::{Result, anyhow};
use rmcp::{
    service::{ServiceExt, RunningService},
    transport::TokioChildProcess,
    RoleClient,
    model::*,
};
use tokio::process::Command;
use serde_json::Value;

pub struct McpClient;

impl McpClient {
    pub async fn start(server_path: &str) -> Result<RunningService<RoleClient, ()>> {
        let transport = TokioChildProcess::new(Command::new(server_path))?;
        let client = ().serve(transport).await.map_err(|e| anyhow!("Failed to start client: {}", e))?;

        // Note: Explicit initialization might be required depending on server strictness.
        // rmcp might handle this or require a specific method call which is currently not found.
        // Assuming implicit handling or that we can proceed for now.
        // client.initialize(...) // Method not found in RunningService

        Ok(client)
    }

    pub async fn get_tools(service: &RunningService<RoleClient, ()>) -> Result<Vec<Tool>> {
        let result = service.list_tools(None).await
            .map_err(|e| anyhow!("Failed to list tools: {:?}", e))?;
        Ok(result.tools)
    }

    pub async fn execute_tool(
        service: &RunningService<RoleClient, ()>,
        name: &str,
        args: Value,
    ) -> Result<String> {
        let args_map = if let Value::Object(map) = args {
            map
        } else {
            return Err(anyhow!("Arguments must be a JSON object"));
        };

        let request = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: Some(args_map),
        };

        let result = service.call_tool(request).await
            .map_err(|e| anyhow!("Failed to call tool '{}': {:?}", name, e))?;

        // Format result content
        if result.content.is_empty() {
            return Ok("Tool executed successfully with no output.".to_string());
        }

        let output: Vec<String> = result.content.iter().filter_map(|c| {
            match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            }
        }).collect();

        Ok(output.join("\n"))
    }
}
