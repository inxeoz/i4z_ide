use crate::tui::app::{App, ActivePanel, AppMode};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main layout: [Status Bar] [Main Content] [Status Line]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Status line
            Constraint::Min(10),     // Main content
            Constraint::Length(1),   // Bottom status
        ])
        .split(size);

    // Render status bar
    app.status_bar.render(
        main_chunks[0],
        frame.buffer_mut(),
        app.state.mode,
        app.state.active_panel,
        app.editor_panel.get_current_file(),
        app.editor_panel.is_modified(),
    );

    // Main content layout: [File Explorer] [Editor + Chat]
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // File explorer
            Constraint::Percentage(75),  // Editor + Chat
        ])
        .split(main_chunks[1]);

    // Right side layout: [Editor] [Chat]
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Editor
            Constraint::Percentage(40),  // Chat
        ])
        .split(content_chunks[1]);

    // Render panels
    app.file_explorer.render(
        content_chunks[0],
        frame.buffer_mut(),
        app.state.active_panel == ActivePanel::FileExplorer,
    );

    app.editor_panel.render(
        right_chunks[0],
        frame.buffer_mut(),
        app.state.active_panel == ActivePanel::Editor,
    );

    app.chat_panel.render(
        right_chunks[1],
        frame.buffer_mut(),
        app.state.active_panel == ActivePanel::Chat,
    );

    // Render help overlay if active
    if app.state.show_help {
        render_help_overlay(frame, &mut app.status_bar);
    }

    // Render mode indicator in bottom right
    render_bottom_status(frame, main_chunks[2], &app);
}

fn render_help_overlay(frame: &mut Frame, status_bar: &mut crate::tui::panels::StatusBar) {
    let size = frame.area();
    
    // Calculate centered area for help
    let help_width = 60;
    let help_height = 25;
    let help_area = Rect {
        x: (size.width.saturating_sub(help_width)) / 2,
        y: (size.height.saturating_sub(help_height)) / 2,
        width: help_width,
        height: help_height,
    };

    // Clear the area
    frame.render_widget(Clear, help_area);

    // Render help content
    status_bar.render_help(help_area, frame.buffer_mut());
}

fn render_bottom_status(frame: &mut Frame, area: Rect, app: &App) {
    let mode_text = match app.state.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Agentic => "🤖 AGENTIC",
        AppMode::Insert => "INSERT",
    };

    let mode_color = match app.state.mode {
        AppMode::Normal => Color::Green,
        AppMode::Agentic => Color::Magenta,
        AppMode::Insert => Color::Yellow,
    };

    let panel_text = match app.state.active_panel {
        ActivePanel::FileExplorer => "📁 FILE EXPLORER",
        ActivePanel::Editor => "📝 EDITOR",
        ActivePanel::Chat => "💬 CHAT",
    };

    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", mode_text),
            Style::default().fg(Color::Black).bg(mode_color)
        ),
        Span::raw(" "),
        Span::styled(panel_text, Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(
            "Tab: Switch Panel | ?: Help | Ctrl+Q: Quit",
            Style::default().fg(Color::Gray)
        ),
    ]);

    let status_paragraph = Paragraph::new(status_line)
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status_paragraph, area);
}