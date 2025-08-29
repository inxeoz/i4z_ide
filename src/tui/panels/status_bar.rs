use crate::tui::app::{AppMode, ActivePanel};
use chrono::Local;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct StatusBar {
    pub show_help: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            show_help: false,
        }
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        app_mode: AppMode,
        active_panel: ActivePanel,
        current_file: Option<&std::path::PathBuf>,
        is_modified: bool,
    ) {
        let current_time = Local::now().format("%H:%M:%S").to_string();

        // Left side: Mode and panel info
        let mode_text = match app_mode {
            AppMode::Normal => "NORMAL",
            AppMode::Agentic => "AGENTIC",
            AppMode::Insert => "INSERT",
        };

        let panel_text = match active_panel {
            ActivePanel::FileExplorer => "FILE",
            ActivePanel::Editor => "EDIT",
            ActivePanel::Chat => "CHAT",
        };

        let mode_color = match app_mode {
            AppMode::Normal => Color::Green,
            AppMode::Agentic => Color::Magenta,
            AppMode::Insert => Color::Yellow,
        };

        // File info
        let file_info = match current_file {
            Some(path) => {
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown");
                let modified_indicator = if is_modified { " •" } else { "" };
                format!(" {} {}{}", "📝", filename, modified_indicator)
            }
            None => " No file".to_string(),
        };

        let left_spans = vec![
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
                Style::default().fg(Color::Cyan)
            ),
        ];

        // Right side: Help and time
        let help_text = if self.show_help {
            " Press ? to hide help "
        } else {
            " ? for help "
        };

        let right_spans = vec![
            Span::styled(
                help_text,
                Style::default().fg(Color::Gray)
            ),
            Span::styled(
                format!(" {} ", current_time),
                Style::default().fg(Color::White)
            ),
        ];

        // Create the status line
        let left_text = Line::from(left_spans);
        let right_text = Line::from(right_spans);

        // Calculate spacing
        let left_width = left_text.width() as u16;
        let right_width = right_text.width() as u16;
        let available_width = area.width.saturating_sub(left_width + right_width);

        let status_line = Line::from(vec![
            left_text.spans,
            vec![Span::raw(" ".repeat(available_width as usize))],
            right_text.spans,
        ].into_iter().flatten().collect::<Vec<_>>());

        let status_paragraph = Paragraph::new(status_line)
            .style(Style::default().bg(Color::DarkGray));

        status_paragraph.render(area, buf);
    }

    pub fn render_help(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_help {
            return;
        }

        let help_text = vec![
            "🚀 Rust Coding Agent - TUI Mode",
            "",
            "📁 File Explorer:",
            "  j/k or ↑/↓  - Navigate files",
            "  Enter       - Open file/directory",
            "  Backspace   - Go to parent directory",
            "",
            "📝 Editor:",
            "  i           - Insert mode",
            "  Esc         - Normal mode", 
            "  Ctrl+S      - Save file",
            "  Ctrl+N      - New file",
            "",
            "💬 Chat:",
            "  Ctrl+Enter  - Send message",
            "  Ctrl+I      - Toggle image mode",
            "  Ctrl+L      - Clear chat",
            "",
            "🔧 Global:",
            "  Tab         - Switch panels",
            "  Alt+1/2/3   - Direct panel access",
            "  Ctrl+A      - Toggle agentic mode",
            "  Ctrl+Q      - Quit",
            "  ?           - Toggle this help",
        ];

        let help_lines: Vec<Line> = help_text
            .iter()
            .map(|line| {
                if line.starts_with('�') {
                    Line::from(Span::styled(*line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                } else if line.starts_with("  ") {
                    let parts: Vec<&str> = line.splitn(2, '-').collect();
                    if parts.len() == 2 {
                        Line::from(vec![
                            Span::styled(parts[0], Style::default().fg(Color::Yellow)),
                            Span::raw("- "),
                            Span::styled(parts[1].trim(), Style::default().fg(Color::Gray)),
                        ])
                    } else {
                        Line::from(Span::raw(*line))
                    }
                } else {
                    Line::from(Span::raw(*line))
                }
            })
            .collect();

        let help_paragraph = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ❓ Help ")
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Left);

        help_paragraph.render(area, buf);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}