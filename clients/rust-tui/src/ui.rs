use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear, Widget},
    Frame,
    buffer::Buffer,
};
use tui_markdown::from_str;

pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AppState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll: u16,
    pub status: Option<String>,
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
            status: None,
        }
    }
}

pub fn draw_ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Chat Area
    let chat_block = Block::default().borders(Borders::ALL).title("Chat");
    let inner_area = chat_block.inner(chunks[0]);
    f.render_widget(chat_block, chunks[0]);

    // Prepare markdown content
    let mut markdown_content = String::new();
    for msg in &state.messages {
        let role_prefix = match msg.role.as_str() {
            "user" => "**User**:\n",
            "assistant" => "**Assistant**:\n",
            "tool" => "**Tool Output**:\n",
            "system" => "**System**:\n",
            _ => "**Unknown**:\n",
        };
        markdown_content.push_str(role_prefix);
        markdown_content.push_str(&msg.content);
        markdown_content.push_str("\n\n---\n\n");
    }

    let markdown = from_str(&markdown_content);

    // Manual Scrolling Implementation
    let content_height = 5000;
    let width = inner_area.width;

    if width > 0 {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, content_height));

        // Render markdown to this temporary buffer
        markdown.render(buffer.area, &mut buffer);

        let scroll = state.scroll;
        let visible_height = inner_area.height;

        for y in 0..visible_height {
            let src_y = scroll + y;
            if src_y >= content_height {
                break;
            }
            let dest_y = inner_area.y + y;

            for x in 0..width {
                let dest_x = inner_area.x + x;
                // Use get (returns &Cell) - copy to dest
                // Safe because we created buffer with width.
                // Note: buffer.cell((x, src_y)) is preferred but get works.
                let cell = &buffer[(x, src_y)];

                // Ratatui buffer access
                // Ensure we are within frame bounds (inner_area is inside f.area)
                let area = f.area();
                if dest_x < area.width && dest_y < area.height {
                     let dest_cell = f.buffer_mut().cell_mut((dest_x, dest_y));
                     if let Some(c) = dest_cell {
                         *c = cell.clone();
                     }
                }
            }
        }
    }

    // Input Area
    let input_block = Block::default().borders(Borders::ALL).title("Input");
    let input = Paragraph::new(state.input.as_str())
        .block(input_block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(input, chunks[1]);

    // Status Area
    if let Some(status) = &state.status {
        let status_area = chunks[2];
        let status_block = Block::default().style(Style::default().bg(Color::Blue));
        let status_text = Paragraph::new(Span::styled(status, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
            .block(status_block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(status_text, status_area);
    }

    // Set cursor
    let input_width = chunks[1].width.saturating_sub(2);
    let cursor_x = chunks[1].x + 1 + (state.input.len() as u16 % input_width);
    let cursor_y = chunks[1].y + 1 + (state.input.len() as u16 / input_width);

    f.set_cursor_position((cursor_x, cursor_y));
}
