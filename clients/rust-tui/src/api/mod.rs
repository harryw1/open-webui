use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    // Content can be null in some tool call responses, but we handle that with Option or defaulting
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>, // used when role == "tool"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>, // sometimes used for tool role
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tool {
    pub r#type: String, // "function"
    pub function: ToolFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value, // JSON Schema
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionChunk {
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionChoice {
    pub delta: ChatCompletionDelta,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallChunk>>,
}

#[derive(Deserialize, Debug)]
pub struct ToolCallChunk {
    pub index: i32,
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<ToolCallFunctionChunk>,
}

#[derive(Deserialize, Debug)]
pub struct ToolCallFunctionChunk {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

pub struct OpenWebUIClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl OpenWebUIClient {
    pub fn new() -> Result<Self> {
        let base_url = env::var("OPEN_WEBUI_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let api_key = env::var("OPEN_WEBUI_API_KEY").expect("OPEN_WEBUI_API_KEY must be set");

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
        })
    }

    pub async fn chat_completions(&self, request: ChatCompletionRequest) -> Result<reqwest::Response> {
        let url = format!("{}/api/chat/completions", self.base_url);

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        Ok(response)
    }
}
