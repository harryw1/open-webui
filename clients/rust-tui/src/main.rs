use anyhow::{Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
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
use std::collections::HashMap;
use std::path::PathBuf;

mod ui;
mod api;
mod mcp;

use ui::{AppState, draw_ui, ChatMessage};
use api::{OpenWebUIClient, ChatCompletionRequest, Message, ToolCall};
use mcp::McpClient;

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
    let mcp_server_path = find_mcp_server()?;

    let mcp_cmd = mcp_server_path.to_string_lossy().to_string();

    let mcp_service = match McpClient::start(&mcp_cmd).await {
        Ok(service) => Some(Arc::new(Mutex::new(service))),
        Err(e) => {
            eprintln!("Warning: Failed to start MCP server at {}: {}", mcp_cmd, e);
            None
        }
    };

    // Fetch tools if available
    let mut available_tools = Vec::new();
    if let Some(service) = &mcp_service {
        let guard = service.lock().await;
        match McpClient::get_tools(&guard).await {
            Ok(tools) => {
                // Convert rmcp tools to api tools
                available_tools = tools.into_iter().map(|t| {
                    let schema = serde_json::Value::Object((*t.input_schema).clone());

                    api::Tool {
                        r#type: "function".to_string(),
                        function: api::ToolFunction {
                            name: t.name.to_string(),
                            description: t.description.map(|d| d.to_string()),
                            parameters: schema,
                        },
                    }
                }).collect();
            }
            Err(e) => eprintln!("Warning: Failed to fetch tools: {}", e),
        }
    }

    let available_tools = Arc::new(available_tools);

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

                    // Handle scrolling globally or when busy
                    match key.code {
                        KeyCode::PageUp => {
                            state_guard.scroll = state_guard.scroll.saturating_sub(10);
                        }
                        KeyCode::PageDown => {
                            state_guard.scroll = state_guard.scroll.saturating_add(10);
                        }
                        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                             state_guard.scroll = state_guard.scroll.saturating_sub(1);
                        }
                        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                             state_guard.scroll = state_guard.scroll.saturating_add(1);
                        }
                        _ => {
                            // Only process input if not busy
                            if state_guard.status.is_none() {
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
                                        if !input.trim().is_empty() {
                                            state_guard.messages.push(ChatMessage {
                                                role: "user".to_string(),
                                                content: input.clone(),
                                            });
                                            state_guard.status = Some("Thinking...".to_string());
                                            state_guard.scroll = 0; // Reset scroll on new message

                                            let api_client = api_client.clone();
                                            let state_arc = app_state.clone();
                                            let mcp_service = mcp_service.clone();
                                            let available_tools = available_tools.clone();

                                            // Snapshot history
                                            let history: Vec<Message> = state_guard.messages.iter().map(|m| {
                                                Message {
                                                    role: m.role.clone(),
                                                    content: Some(m.content.clone()),
                                                    tool_calls: None,
                                                    tool_call_id: None,
                                                    name: None,
                                                }
                                            }).collect();

                                            tokio::spawn(async move {
                                                handle_conversation_turn(
                                                    api_client,
                                                    state_arc,
                                                    mcp_service,
                                                    available_tools,
                                                    history
                                                ).await;
                                            });
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn find_mcp_server() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        let mut path = exe.clone();
        for _ in 0..5 {
            if !path.pop() { break; }
        }
        let check = path.join("backend/mcp-server-rust/target/debug/mcp-server-rust");
        if check.exists() {
            return Ok(check);
        }
    }

    let cwd = std::env::current_dir()?;
    let check = cwd.join("backend/mcp-server-rust/target/debug/mcp-server-rust");
    if check.exists() {
        return Ok(check);
    }

    let check = cwd.join("../../backend/mcp-server-rust/target/debug/mcp-server-rust");
    if check.exists() {
        return Ok(check);
    }

    Ok(PathBuf::from("mcp-server-rust"))
}

async fn handle_conversation_turn(
    api_client: Arc<OpenWebUIClient>,
    state_arc: Arc<Mutex<AppState>>,
    mcp_service: Option<Arc<Mutex<rmcp::service::RunningService<rmcp::RoleClient, ()>>>>,
    available_tools: Arc<Vec<api::Tool>>,
    mut history: Vec<Message>,
) {
    loop {
        let tools_param = if available_tools.is_empty() {
            None
        } else {
            Some(available_tools.iter().cloned().collect())
        };

        let request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(), // Adjust as needed
            messages: history.clone(),
            stream: true,
            tools: tools_param,
        };

        match api_client.chat_completions(request).await {
            Ok(response) => {
                let mut stream = response.bytes_stream();
                let mut assistant_content = String::new();
                let mut current_tool_calls: HashMap<i32, ToolCall> = HashMap::new();

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
                        for line in chunk_str.lines() {
                            if line.starts_with("data: ") {
                                let json_str = &line[6..];
                                if json_str == "[DONE]" { continue; }
                                if let Ok(chunk) = serde_json::from_str::<api::ChatCompletionChunk>(json_str) {
                                    if chunk.choices.is_empty() { continue; }
                                    let choice = &chunk.choices[0];

                                    if let Some(content) = &choice.delta.content {
                                        assistant_content.push_str(content);
                                        let mut state = state_arc.lock().await;
                                        if let Some(last) = state.messages.last_mut() {
                                            last.content.push_str(content);
                                        }
                                    }

                                    if let Some(tool_calls_chunk) = &choice.delta.tool_calls {
                                        for tc_chunk in tool_calls_chunk {
                                            let index = tc_chunk.index;
                                            let tool_call = current_tool_calls.entry(index).or_insert(ToolCall {
                                                id: tc_chunk.id.clone().unwrap_or_default(),
                                                r#type: tc_chunk.r#type.clone().unwrap_or("function".to_string()),
                                                function: api::ToolCallFunction {
                                                    name: String::new(),
                                                    arguments: String::new(),
                                                },
                                            });

                                            if let Some(id) = &tc_chunk.id {
                                                tool_call.id = id.clone();
                                            }
                                            if let Some(fn_chunk) = &tc_chunk.function {
                                                if let Some(name) = &fn_chunk.name {
                                                    tool_call.function.name.push_str(name);
                                                }
                                                if let Some(args) = &fn_chunk.arguments {
                                                    tool_call.function.arguments.push_str(args);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let mut tool_calls_vec = Vec::new();
                if !current_tool_calls.is_empty() {
                    let mut sorted_indices: Vec<_> = current_tool_calls.keys().cloned().collect();
                    sorted_indices.sort();
                    for index in sorted_indices {
                        tool_calls_vec.push(current_tool_calls.remove(&index).unwrap());
                    }
                }

                history.push(Message {
                    role: "assistant".to_string(),
                    content: Some(assistant_content),
                    tool_calls: if tool_calls_vec.is_empty() { None } else { Some(tool_calls_vec.clone()) },
                    tool_call_id: None,
                    name: None,
                });

                if !tool_calls_vec.is_empty() {
                    let mut executed_something = false;
                    for tc in tool_calls_vec {
                        let fn_name = tc.function.name;
                        let fn_args_str = tc.function.arguments;

                        {
                            let mut state = state_arc.lock().await;
                            state.status = Some(format!("Executing {}...", fn_name));
                        }

                        let result_content = if let Some(service_mutex) = &mcp_service {
                            let args_value: serde_json::Value = serde_json::from_str(&fn_args_str)
                                .unwrap_or(serde_json::Value::Null);

                            let guard = service_mutex.lock().await;
                            match McpClient::execute_tool(&guard, &fn_name, args_value).await {
                                Ok(res) => res,
                                Err(e) => format!("Error executing tool: {}", e),
                            }
                        } else {
                            "MCP Server not available".to_string()
                        };

                        history.push(Message {
                            role: "tool".to_string(),
                            content: Some(result_content.clone()),
                            tool_calls: None,
                            tool_call_id: Some(tc.id.clone()),
                            name: Some(fn_name.clone()),
                        });

                        {
                            let mut state = state_arc.lock().await;
                            state.messages.push(ChatMessage {
                                role: "tool".to_string(),
                                content: format!("output: {}", result_content),
                            });
                        }
                        executed_something = true;
                    }

                    if executed_something {
                        {
                            let mut state = state_arc.lock().await;
                            state.status = Some("Thinking...".to_string());
                        }
                        continue;
                    }
                }

                {
                    let mut state = state_arc.lock().await;
                    state.status = None;
                }
                break;
            }
            Err(e) => {
                let mut state = state_arc.lock().await;
                state.messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: format!("Error: {}", e),
                });
                state.status = None;
                break;
            }
        }
    }
}
