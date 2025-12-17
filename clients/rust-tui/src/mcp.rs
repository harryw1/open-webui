use anyhow::{Result, anyhow};
use rmcp::{
    service::{ServiceExt, RunningService},
    transport::{TokioChildProcess, ConfigureCommandExt},
    RoleClient,
};
use tokio::process::Command;

pub struct McpClient;

impl McpClient {
    pub async fn start(server_path: &str) -> Result<RunningService<RoleClient, ()>> {
        // TokioChildProcess::new returns a Result
        let transport = TokioChildProcess::new(Command::new(server_path))?;

        let client = ().serve(transport).await.map_err(|e| anyhow!("Failed to start client: {}", e))?;

        Ok(client)
    }
}
