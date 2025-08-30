use crate::ide::app::{NotificationMessage, NotificationType};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct NotificationPanel {
    pub list_state: ListState,
}

impl NotificationPanel {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            list_state,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, notifications: &[NotificationMessage], is_focused: bool) {
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let items: Vec<ListItem> = notifications
            .iter()
            .rev() // Show newest first
            .take(5) // Show only the last 5 notifications to fit in the space
            .map(|notification| {
                let (icon, color) = match notification.notification_type {
                    NotificationType::MouseHover => ("🔍", Color::Gray),
                    NotificationType::MouseClick => ("🖱️", Color::Yellow),
                    NotificationType::FileOperation => ("📄", Color::Green),
                    NotificationType::Info => ("ℹ️", Color::Blue),
                    NotificationType::Debug => ("🐛", Color::Magenta),
                };

                // Format timestamp (show seconds)
                let elapsed = notification.timestamp
                    .elapsed()
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs();
                
                let time_str = if elapsed < 60 {
                    format!("{}s", elapsed)
                } else if elapsed < 3600 {
                    format!("{}m", elapsed / 60)
                } else {
                    format!("{}h", elapsed / 3600)
                };

                let line = Line::from(vec![
                    Span::styled(icon, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(
                        format!("{} ({})", notification.message, time_str),
                        Style::default().fg(Color::White)
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title(" 📋 Notifications ")
                .borders(Borders::ALL)
                .border_style(border_style))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(if is_focused { Color::Cyan } else { Color::DarkGray })
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_stateful_widget(list, area, &mut self.list_state.clone());
    }

    pub fn scroll_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected > 0 {
                self.list_state.select(Some(selected - 1));
            }
        }
    }

    pub fn scroll_down(&mut self, max_items: usize) {
        if let Some(selected) = self.list_state.selected() {
            if selected < max_items.saturating_sub(1) {
                self.list_state.select(Some(selected + 1));
            }
        }
    }
}