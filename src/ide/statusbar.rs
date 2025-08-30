use crate::ide::app::{AppMode, FocusedPanel};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use chrono::Local;

#[derive(Debug)]
pub struct StatusInfo {
    pub mode: AppMode,
    pub focused_panel: FocusedPanel,
    pub current_file: Option<String>,
    pub cursor_position: (usize, usize), // (line, column)
    pub is_modified: bool,
    pub total_files: usize,
}

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, status_info: &StatusInfo) {
        let current_time = Local::now().format("%H:%M:%S").to_string();

        // Left side: Mode and file info
        let mode_text = match status_info.mode {
            AppMode::Normal => "NORMAL",
            AppMode::Insert => "INSERT",
            AppMode::Agentic => "AGENTIC",
        };

        let mode_color = match status_info.mode {
            AppMode::Normal => Color::Green,
            AppMode::Insert => Color::Yellow,
            AppMode::Agentic => Color::Magenta,
        };

        let panel_text = match status_info.focused_panel {
            FocusedPanel::FileExplorer => "FILES",
            FocusedPanel::Editor => "EDITOR",
            FocusedPanel::Chat => "CHAT",
            FocusedPanel::Notifications => "NOTIFICATIONS",
        };

        // File information
        let file_info = if let Some(filename) = &status_info.current_file {
            let modified_indicator = if status_info.is_modified { " ●" } else { "" };
            let (line, col) = status_info.cursor_position;
            if line > 0 && col > 0 {
                format!(" {} {} | Ln {}, Col {}{}", 
                    get_file_icon(filename),
                    filename,
                    line,
                    col,
                    modified_indicator
                )
            } else {
                format!(" {} {}{}", 
                    get_file_icon(filename),
                    filename,
                    modified_indicator
                )
            }
        } else {
            " No file open".to_string()
        };

        // Tab count info
        let tab_info = if status_info.total_files > 0 {
            format!(" ({} files)", status_info.total_files)
        } else {
            String::new()
        };

        // Build left side
        let mut left_spans = vec![
            Span::styled(
                format!(" {} ", mode_text),
                Style::default()
                    .fg(Color::Black)
                    .bg(mode_color)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                format!(" {} ", panel_text),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                file_info,
                Style::default().fg(Color::White)
            ),
        ];

        if !tab_info.is_empty() {
            left_spans.push(Span::styled(
                tab_info,
                Style::default().fg(Color::Gray)
            ));
        }

        // Right side: Encoding, file type, and time
        let file_type = status_info.current_file
            .as_ref()
            .and_then(|filename| {
                std::path::Path::new(filename)
                    .extension()
                    .and_then(|ext| ext.to_str())
            })
            .unwrap_or("Plain Text");

        let right_spans = vec![
            Span::styled(
                format!(" UTF-8 "),
                Style::default().fg(Color::Gray)
            ),
            Span::styled(
                format!(" {} ", file_type.to_uppercase()),
                Style::default().fg(Color::Cyan)
            ),
            Span::styled(
                format!(" {} ", current_time),
                Style::default().fg(Color::White).bg(Color::DarkGray)
            ),
        ];

        // Calculate spacing
        let left_width = left_spans.iter().map(|span| span.content.len()).sum::<usize>() as u16;
        let right_width = right_spans.iter().map(|span| span.content.len()).sum::<usize>() as u16;
        let available_width = area.width.saturating_sub(left_width + right_width);

        // Create the complete status line
        let mut all_spans = left_spans;
        all_spans.push(Span::raw(" ".repeat(available_width as usize)));
        all_spans.extend(right_spans);

        let status_line = Line::from(all_spans);
        let status_paragraph = Paragraph::new(status_line)
            .style(Style::default().bg(Color::DarkGray));

        frame.render_widget(status_paragraph, area);
    }
}

fn get_file_icon(filename: &str) -> &'static str {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension {
        "rs" => "🦀",
        "py" => "🐍", 
        "js" | "ts" => "📜",
        "html" => "🌐",
        "css" => "🎨",
        "json" => "📋",
        "md" => "📄",
        "txt" => "📃",
        "toml" | "yaml" | "yml" => "⚙️",
        _ => "📄",
    }
}