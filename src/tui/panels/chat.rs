use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Widget,
};
use tui_textarea::{TextArea, Input, Key};

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
}

pub struct ChatPanel {
    pub messages: Vec<ChatMessage>,
    pub input: TextArea<'static>,
    pub scroll_offset: usize,
    pub include_image: bool,
    list_state: ListState,
}

impl ChatPanel {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💬 Chat Input ")
                .border_style(Style::default().fg(Color::Cyan))
        );
        input.set_placeholder_text("Type your message... (Ctrl+Enter to send, Ctrl+I for image)");

        Self {
            messages: Vec::new(),
            input,
            scroll_offset: 0,
            include_image: false,
            list_state: ListState::default(),
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage::new(MessageType::User, content));
        self.scroll_to_bottom();
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage::new(MessageType::Assistant, content));
        self.scroll_to_bottom();
    }

    pub fn add_system_message(&mut self, content: String) {
        self.messages.push(ChatMessage::new(MessageType::System, content));
        self.scroll_to_bottom();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
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

    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<String> {
        match key.code {
            crossterm::event::KeyCode::Enter if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                let message = self.input.lines().join("\n");
                if !message.trim().is_empty() {
                    self.input.select_all();
                    self.input.cut();
                    Some(message)
                } else {
                    None
                }
            }
            crossterm::event::KeyCode::Char('i') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.include_image = !self.include_image;
                self.update_input_placeholder();
                None
            }
            _ => {
                self.input.input(Input::from(key));
                None
            }
        }
    }

    fn update_input_placeholder(&mut self) {
        let placeholder = if self.include_image {
            "💾 Image mode: Type message + Enter (Ctrl+I to toggle image)"
        } else {
            "Type your message... (Ctrl+Enter to send, Ctrl+I for image)"
        };
        self.input.set_placeholder_text(placeholder);
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),      // Messages area
                Constraint::Length(3),   // Input area
            ])
            .split(area);

        self.render_messages(chunks[0], buf, is_active);
        self.render_input(chunks[1], buf, is_active);
    }

    fn render_messages(&mut self, area: Rect, buf: &mut Buffer, is_active: bool) {
        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 💬 Chat History ")
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.messages.is_empty() {
            let empty_text = Paragraph::new("No messages yet. Start chatting!")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            empty_text.render(inner, buf);
            return;
        }

        let items: Vec<ListItem> = self.messages
            .iter()
            .map(|msg| self.format_message(msg))
            .collect();

        let messages_list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        messages_list.render(inner, buf);
    }

    fn format_message(&self, message: &ChatMessage) -> ListItem {
        let time_str = message.timestamp.format("%H:%M:%S").to_string();
        
        let (prefix, style) = match message.message_type {
            MessageType::User => ("🧑", Style::default().fg(Color::Green)),
            MessageType::Assistant => ("🤖", Style::default().fg(Color::Blue)),
            MessageType::System => ("ℹ️ ", Style::default().fg(Color::Yellow)),
        };

        let header = Line::from(vec![
            Span::styled(
                format!("{} [{}] ", prefix, time_str),
                Style::default().fg(Color::Gray)
            )
        ]);

        let content_lines: Vec<Line> = message.content
            .lines()
            .map(|line| Line::from(Span::styled(line, style)))
            .collect();

        let mut lines = vec![header];
        lines.extend(content_lines);
        lines.push(Line::from(""));

        ListItem::new(Text::from(lines))
    }

    fn render_input(&mut self, area: Rect, buf: &mut Buffer, is_active: bool) {
        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let mut input = self.input.clone();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(if self.include_image { " 💾 Chat Input (Image Mode) " } else { " 💬 Chat Input " })
                .border_style(border_style)
        );

        input.render(area, buf);
    }

    pub fn is_image_mode(&self) -> bool {
        self.include_image
    }
}