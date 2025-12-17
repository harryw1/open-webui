use anyhow::{Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::stdout;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::StreamExt;

mod ui;
mod api;
mod mcp;

use ui::{AppState, draw_ui, ChatMessage};
use api::{OpenWebUIClient, ChatCompletionRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Check environment variables
    if std::env::var("OPEN_WEBUI_API_KEY").is_err() {
        eprintln!("Error: OPEN_WEBUI_API_KEY environment variable is not set.");
        std::process::exit(1);
    }

    // Initialize API Client
    let api_client = Arc::new(OpenWebUIClient::new()?);

    // Initialize MCP Client
    let mcp_server_path = std::env::current_exe()?
        .parent().unwrap()
        .parent().unwrap() // release/debug
        .parent().unwrap() // target
        .join("debug/mcp-server-rust");

    let mcp_cmd = if mcp_server_path.exists() {
        mcp_server_path.to_string_lossy().to_string()
    } else {
        "mcp-server-rust".to_string()
    };

    let mcp_service = match mcp::McpClient::start(&mcp_cmd).await {
        Ok(service) => Some(Arc::new(Mutex::new(service))),
        Err(e) => {
            eprintln!("Warning: Failed to start MCP server: {}", e);
            None
        }
    };

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let app_state = Arc::new(Mutex::new(AppState::new()));

    let mut event_rx = {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            loop {
                if event::poll(std::time::Duration::from_millis(16)).unwrap() {
                    if let Event::Key(key) = event::read().unwrap() {
                        tx.send(key).await.unwrap();
                    }
                }
            }
        });
        rx
    };

    loop {
        {
            let state_guard = app_state.lock().await;
            terminal.draw(|f| draw_ui(f, &state_guard))?;
        }

        tokio::select! {
            Some(key) = event_rx.recv() => {
                if key.kind == KeyEventKind::Press {
                    let mut state_guard = app_state.lock().await;
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char(c) => {
                            state_guard.input.push(c);
                        }
                        KeyCode::Backspace => {
                            state_guard.input.pop();
                        }
                        KeyCode::Enter => {
                            let input = state_guard.input.drain(..).collect::<String>();
                            state_guard.messages.push(ChatMessage {
                                role: "user".to_string(),
                                content: input.clone(),
                            });

                            // Spawn a task to handle the request
                            let api_client = api_client.clone();
                            let state_arc = app_state.clone();
                            let input_clone = input.clone();
                            let _mcp_service = mcp_service.clone();

                            tokio::spawn(async move {
                                let request = ChatCompletionRequest {
                                    model: "gpt-3.5-turbo".to_string(), // Default or config
                                    messages: vec![Message { role: "user".to_string(), content: input_clone }],
                                    stream: true,
                                };

                                match api_client.chat_completions(request).await {
                                    Ok(response) => {
                                        let mut stream = response.bytes_stream();
                                        let mut assistant_message = String::new();

                                        // We need to append a new empty message for assistant
                                        {
                                            let mut state = state_arc.lock().await;
                                            state.messages.push(ChatMessage {
                                                role: "assistant".to_string(),
                                                content: String::new(),
                                            });
                                        }

                                        while let Some(item) = stream.next().await {
                                            if let Ok(bytes) = item {
                                                let chunk_str = String::from_utf8_lossy(&bytes);
                                                // Need to parse SSE format "data: {...}"
                                                for line in chunk_str.lines() {
                                                    if line.starts_with("data: ") {
                                                        let json_str = &line[6..];
                                                        if json_str == "[DONE]" { continue; }
                                                        if let Ok(chunk) = serde_json::from_str::<api::ChatCompletionChunk>(json_str) {
                                                            if let Some(content) = &chunk.choices[0].delta.content {
                                                                assistant_message.push_str(content);
                                                                let mut state = state_arc.lock().await;
                                                                if let Some(last) = state.messages.last_mut() {
                                                                    last.content.push_str(content);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let mut state = state_arc.lock().await;
                                        state.messages.push(ChatMessage {
                                            role: "system".to_string(),
                                            content: format!("Error: {}", e),
                                        });
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
