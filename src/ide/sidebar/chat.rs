use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub enum MessageType {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub message_type: MessageType,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

impl ChatMessage {
    pub fn new(message_type: MessageType, content: String) -> Self {
        Self {
            message_type,
            content,
            timestamp: Local::now(),
        }
    }

    pub fn to_list_item(&self) -> ListItem {
        let (prefix, style) = match self.message_type {
            MessageType::User => ("🧑", Style::default().fg(Color::Green)),
            MessageType::Assistant => ("🤖", Style::default().fg(Color::Cyan)),
            MessageType::System => ("ℹ️", Style::default().fg(Color::Yellow)),
        };

        let time_str = self.timestamp.format("%H:%M").to_string();
        let display_text = format!("{} [{}] {}", prefix, time_str, self.content);
        
        // Wrap long messages
        let wrapped_lines = wrap_text(&display_text, 25); // Approximate width for sidebar
        let lines: Vec<Line> = wrapped_lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    Line::from(Span::styled(line, style))
                } else {
                    // Indent continuation lines
                    Line::from(Span::styled(format!("   {}", line), style))
                }
            })
            .collect();

        ListItem::new(lines)
    }
}

pub struct Chat {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll_offset: usize,
    pub list_state: ListState,
}

impl Chat {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            messages: vec![
                ChatMessage::new(MessageType::System, "Welcome! Ask me anything about your code.".to_string())
            ],
            input: String::new(),
            scroll_offset: 0,
            list_state,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(ChatMessage::new(MessageType::User, content.to_string()));
        self.scroll_to_bottom();
    }

    pub fn add_ai_message(&mut self, content: &str) {
        self.messages.push(ChatMessage::new(MessageType::Assistant, content.to_string()));
        self.scroll_to_bottom();
    }

    pub fn add_system_message(&mut self, content: &str) {
        self.messages.push(ChatMessage::new(MessageType::System, content.to_string()));
        self.scroll_to_bottom();
    }

    pub fn remove_last_message(&mut self) {
        self.messages.pop();
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.messages.push(ChatMessage::new(MessageType::System, "Chat cleared.".to_string()));
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }

    pub fn add_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn get_input_and_clear(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        input
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        // Split chat area: [Messages] [Input]
        let chat_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(4),      // Messages area
                ratatui::layout::Constraint::Length(3),   // Input area
            ])
            .split(area);

        self.draw_messages(frame, chat_chunks[0], is_focused);
        self.draw_input(frame, chat_chunks[1], is_focused);
    }

    fn draw_messages(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        if self.messages.is_empty() {
            let empty_text = Paragraph::new("No messages yet...")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default()
                    .title(" 💬 AI Chat ")
                    .borders(Borders::ALL)
                    .border_style(border_style));
            frame.render_widget(empty_text, area);
            return;
        }

        // Show recent messages
        let visible_messages: Vec<ListItem> = self.messages
            .iter()
            .rev() // Show newest first
            .take(20) // Limit to recent messages
            .map(|msg| msg.to_list_item())
            .collect();

        let messages_list = List::new(visible_messages)
            .block(Block::default()
                .title(" 💬 AI Chat ")
                .borders(Borders::ALL)
                .border_style(border_style));

        frame.render_widget(messages_list, area);
    }

    fn draw_input(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_text = if self.input.is_empty() && is_focused {
            "Type your message..."
        } else {
            &self.input
        };

        let input_style = if self.input.is_empty() && is_focused {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::White)
        };

        let input_widget = Paragraph::new(input_text)
            .style(input_style)
            .block(Block::default()
                .title(" Message (Enter: Send, Ctrl+I: Image) ")
                .borders(Borders::ALL)
                .border_style(border_style));

        frame.render_widget(input_widget, area);
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > max_width && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line.clear();
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(text.to_string());
    }

    lines
}