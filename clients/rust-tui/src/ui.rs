use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Frame,
};

pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AppState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll: u16,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome to the Open WebUI TUI! Type your message below.".to_string(),
            }],
            input: String::new(),
            scroll: 0,
        }
    }
}

pub fn draw_ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Chat Area
    let messages: Vec<Line> = state.messages.iter().map(|msg| {
        let role_style = match msg.role.as_str() {
            "user" => Style::default().fg(Color::Cyan),
            "assistant" => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::Yellow),
        };
        Line::from(vec![
            Span::styled(format!("{}: ", msg.role), role_style),
            Span::raw(&msg.content),
        ])
    }).collect();

    let chat_block = Block::default().borders(Borders::ALL).title("Chat");
    let chat = Paragraph::new(messages)
        .block(chat_block)
        .wrap(Wrap { trim: true })
        .scroll((state.scroll, 0));

    f.render_widget(chat, chunks[0]);

    // Input Area
    let input_block = Block::default().borders(Borders::ALL).title("Input");
    let input = Paragraph::new(state.input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(input, chunks[1]);

    // Set cursor position
    f.set_cursor_position((
        chunks[1].x + state.input.len() as u16 + 1,
        chunks[1].y + 1,
    ));
}
