use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    // We will add tools here later
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
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
