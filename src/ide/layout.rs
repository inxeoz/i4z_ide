use crate::ide::app::{IdeApp, FocusedPanel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_ide(frame: &mut Frame, app: &mut IdeApp) {
    let size = frame.area();

    // Check for overlays first
    if app.show_command_help {
        draw_command_help_overlay(frame, size);
        return;
    }

    if app.show_api_config {
        draw_api_config_overlay(frame, size);
        return;
    }

    if app.show_help {
        draw_help_overlay(frame, size);
        return;
    }

    // File operation dialogs
    if app.has_active_dialog() {
        // Draw main IDE first, then overlay dialog
        draw_main_ide_layout(frame, app, size);
        draw_dialog_overlay(frame, app, size);
        return;
    }

    draw_main_ide_layout(frame, app, size);
}

fn draw_sidebar(frame: &mut Frame, app: &IdeApp, area: Rect) {
    if app.show_notifications && !app.notifications.is_empty() {
        // Split sidebar vertically: [File Explorer] [Notifications] [Chat]
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),                            // File explorer (flexible, minimum 8 lines)
                Constraint::Length(6),                         // Notifications (fixed height)
                Constraint::Length(app.layout.chat_height),    // Chat (fixed height)
            ])
            .split(area);

        // Draw file explorer
        app.sidebar.file_explorer.draw(
            frame, 
            sidebar_chunks[0], 
            app.focused_panel == FocusedPanel::FileExplorer
        );

        // Draw notifications
        app.sidebar.notifications.draw(
            frame,
            sidebar_chunks[1],
            &app.notifications,
            false // Notifications are not focusable for now
        );

        // Draw chat
        app.sidebar.chat.draw(
            frame, 
            sidebar_chunks[2], 
            app.focused_panel == FocusedPanel::Chat
        );
    } else {
        // Split sidebar vertically: [File Explorer] [Chat] (original layout)
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),                           // File explorer (flexible)
                Constraint::Length(app.layout.chat_height),    // Chat (fixed height)
            ])
            .split(area);

        // Draw file explorer
        app.sidebar.file_explorer.draw(
            frame, 
            sidebar_chunks[0], 
            app.focused_panel == FocusedPanel::FileExplorer
        );

        // Draw chat
        app.sidebar.chat.draw(
            frame, 
            sidebar_chunks[1], 
            app.focused_panel == FocusedPanel::Chat
        );
    }
}

fn draw_main_area(frame: &mut Frame, app: &mut IdeApp, area: Rect) {
    // Split main area vertically: [Editor with tabs] [Status bar]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),        // Editor area
            Constraint::Length(1),     // Status bar
        ])
        .split(area);

    // Draw editor with tabs
    draw_editor_area(frame, app, main_chunks[0]);
    
    // Draw status bar
    let status_info = app.get_status_info();
    app.statusbar.draw(frame, main_chunks[1], &status_info);
}

fn draw_editor_area(frame: &mut Frame, app: &mut IdeApp, area: Rect) {
    if app.editor.has_open_files() {
        // Split editor area: [Tabs] [Editor content]
        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // Tab bar
                Constraint::Min(5),        // Editor content
            ])
            .split(area);

        // Draw tabs
        draw_tabs(frame, app, editor_chunks[0]);
        
        // Draw editor content
        app.editor.draw(
            frame, 
            editor_chunks[1], 
            app.focused_panel == FocusedPanel::Editor,
            app.mode
        );
    } else {
        // Draw welcome screen when no files are open
        draw_welcome_screen(frame, area);
    }
}

fn draw_tabs(frame: &mut Frame, app: &IdeApp, area: Rect) {
    let tabs = app.editor.get_tab_info();
    let active_tab = app.editor.get_active_tab_index();
    
    if tabs.is_empty() {
        return;
    }

    let mut tab_spans = Vec::new();
    
    for (i, tab) in tabs.iter().enumerate() {
        let is_active = i == active_tab;
        let is_modified = tab.is_modified;
        
        // Tab styling
        let (bg_color, fg_color) = if is_active {
            if app.focused_panel == FocusedPanel::Editor {
                (Color::Cyan, Color::Black)
            } else {
                (Color::Blue, Color::White)
            }
        } else {
            (Color::DarkGray, Color::Gray)
        };

        let mut style = Style::default().bg(bg_color).fg(fg_color);
        if is_active {
            style = style.add_modifier(Modifier::BOLD);
        }

        // Tab content
        let modified_indicator = if is_modified { "●" } else { "" };
        let tab_text = format!(" {} {}{} ", 
            get_file_icon(&tab.file_name), 
            tab.file_name,
            modified_indicator
        );

        tab_spans.push(Span::styled(tab_text, style));
        
        // Tab separator
        if i < tabs.len() - 1 {
            tab_spans.push(Span::raw("│"));
        }
    }

    // Add new tab button
    tab_spans.push(Span::styled(" + ", Style::default().fg(Color::Gray)));

    let tabs_line = Line::from(tab_spans);
    let tabs_paragraph = Paragraph::new(tabs_line)
        .block(Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(tabs_paragraph, area);
}

fn draw_welcome_screen(frame: &mut Frame, area: Rect) {
    let welcome_text = vec![
        Line::from(Span::styled("🦀 Rust Coding Agent", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Welcome to your Rust IDE! Start by:"),
        Line::from(""),
        Line::from(Span::styled("📁 Opening a file from the explorer (Alt+1)", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("📄 Creating a new file (Ctrl+N)", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("💬 Chatting with AI (Alt+3)", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("Quick shortcuts:"),
        Line::from("• Tab - Cycle between panels"),
        Line::from("• Ctrl+H - Command help"),
        Line::from("• F1 or ? - General help"),
        Line::from("• Ctrl+Q - Quit"),
        Line::from(""),
        Line::from(Span::styled("Happy coding! 🚀", Style::default().fg(Color::Green))),
    ];

    let welcome = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .title(" Welcome ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    // Center the welcome screen
    let welcome_area = centered_rect(60, 70, area);
    frame.render_widget(welcome, welcome_area);
}

fn draw_command_help_overlay(frame: &mut Frame, area: Rect) {
    // Clear the background
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("⌨️  Command Reference - Ctrl+H", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("🔧 File Operations:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+N      - New file"),
        Line::from("  Ctrl+S      - Save file"),
        Line::from("  Ctrl+W      - Close file"),
        Line::from("  Ctrl+O      - Focus file explorer"),
        Line::from("  Ctrl+D      - New folder"),
        Line::from("  F2          - Rename (selected file)"),
        Line::from("  Delete      - Delete (selected file)"),
        Line::from(""),
        Line::from(Span::styled("📝 Editor:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  i           - Insert mode"),
        Line::from("  Esc         - Normal mode"),
        Line::from("  h/j/k/l     - Move cursor (normal mode)"),
        Line::from("  ↑/↓/←/→     - Move cursor"),
        Line::from(""),
        Line::from(Span::styled("💬 AI Chat:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+Enter  - Send message"),
        Line::from("  Ctrl+I      - Send with image"),
        Line::from("  Ctrl+L      - Clear chat"),
        Line::from("  Ctrl+K      - Clear notifications"),
        Line::from(""),
        Line::from(Span::styled("🔄 Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Tab         - Cycle panels"),
        Line::from("  Alt+1/2/3   - Direct panel access"),
        Line::from("  Space       - Toggle folder (file explorer)"),
        Line::from(""),
        Line::from(Span::styled("⚙️  System:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+A      - Toggle agentic mode"),
        Line::from("  Ctrl+,      - API configuration"),
        Line::from("  Ctrl+Q      - Quit"),
        Line::from("  F1 / ?      - General help"),
        Line::from(""),
        Line::from(Span::styled("Press Ctrl+H to close this help", Style::default().fg(Color::Gray))),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .title(" ⌨️  Commands ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    let help_area = centered_rect(70, 85, area);
    frame.render_widget(help_paragraph, help_area);
}

fn draw_api_config_overlay(frame: &mut Frame, area: Rect) {
    // Clear the background
    frame.render_widget(Clear, area);

    let config_text = vec![
        Line::from(Span::styled("⚙️  AI API Configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("🔑 Current Configuration:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  API Provider: Groq"),
        Line::from("  Model: llama-3.1-70b-versatile"),
        Line::from("  Status: ✅ Connected"),
        Line::from(""),
        Line::from(Span::styled("🔧 Available Models:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  • llama-3.1-70b-versatile (Current)"),
        Line::from("  • llama-3.1-8b-instant"),
        Line::from("  • mixtral-8x7b-32768"),
        Line::from("  • gemma-7b-it"),
        Line::from("  • gemma-9b-it"),
        Line::from(""),
        Line::from(Span::styled("⚡ Commands:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Use terminal to configure:"),
        Line::from("  ./agent config --groq-key YOUR_KEY"),
        Line::from("  ./agent config --model MODEL_NAME"),
        Line::from(""),
        Line::from(Span::styled("💡 Tips:", Style::default().fg(Color::Green))),
        Line::from("  • 70b model: Best for coding tasks"),
        Line::from("  • 8b model: Faster responses"),
        Line::from("  • Mixtral: Great for complex reasoning"),
        Line::from(""),
        Line::from(Span::styled("Press Ctrl+, to close", Style::default().fg(Color::Gray))),
    ];

    let config_paragraph = Paragraph::new(config_text)
        .block(Block::default()
            .title(" ⚙️  API Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    let config_area = centered_rect(60, 75, area);
    frame.render_widget(config_paragraph, config_area);
}

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    // Clear the background
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("🦀 Rust Coding Agent - IDE Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("🎯 Getting Started:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  1. Use Alt+1 to focus file explorer"),
        Line::from("  2. Navigate with ↑/↓ or j/k keys"),
        Line::from("  3. Press Enter to open files"),
        Line::from("  4. Use 'i' in editor for insert mode"),
        Line::from("  5. Chat with AI using Alt+3"),
        Line::from(""),
        Line::from(Span::styled("🔧 Main Features:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  • Multi-tab file editing"),
        Line::from("  • Integrated AI chat with image support"),
        Line::from("  • Vim-like keyboard navigation"),
        Line::from("  • Resizable panels"),
        Line::from("  • Agentic mode for file operations"),
        Line::from(""),
        Line::from(Span::styled("🎮 Interface:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Left: File explorer + AI chat"),
        Line::from("  Right: Code editor with tabs"),
        Line::from("  Bottom: Status bar with file info"),
        Line::from(""),
        Line::from(Span::styled("💡 Pro Tips:", Style::default().fg(Color::Green))),
        Line::from("  • Use Ctrl+H for detailed commands"),
        Line::from("  • Mouse support for clicking"),
        Line::from("  • Ctrl+A enables AI file operations"),
        Line::from("  • Ctrl+←→ to resize sidebar"),
        Line::from(""),
        Line::from(Span::styled("Press F1 or ? to close help", Style::default().fg(Color::Gray))),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .title(" ❓ Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    let help_area = centered_rect(70, 80, area);
    frame.render_widget(help_paragraph, help_area);
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

/// Create a centered rectangle with the given percentage of width and height
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_main_ide_layout(frame: &mut Frame, app: &mut IdeApp, size: Rect) {
    // Main IDE layout: [Sidebar] [Main Area] 
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(app.layout.sidebar_width),    // Sidebar
            Constraint::Min(40),                             // Main area
        ])
        .split(size);

    // Draw sidebar (file explorer + chat)
    draw_sidebar(frame, app, main_chunks[0]);
    
    // Draw main editor area
    draw_main_area(frame, app, main_chunks[1]);
}

fn draw_dialog_overlay(frame: &mut Frame, app: &IdeApp, area: Rect) {
    // Clear the background
    frame.render_widget(Clear, area);

    let (title, prompt, input_text) = if app.show_create_file_dialog {
        ("📄 Create New File", "Enter filename:", &app.dialog_input)
    } else if app.show_create_folder_dialog {
        ("📁 Create New Folder", "Enter folder name:", &app.dialog_input)
    } else if app.show_rename_dialog {
        ("✏️ Rename", "Enter new name:", &app.dialog_input)
    } else {
        return;
    };

    let dialog_text = vec![
        Line::from(Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(prompt, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}_", input_text), 
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled("Press Enter to confirm, Esc to cancel", Style::default().fg(Color::Gray))),
    ];

    let dialog = Paragraph::new(dialog_text)
        .alignment(Alignment::Left)
        .block(Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));

    // Center the dialog
    let dialog_area = centered_rect(50, 25, area);
    frame.render_widget(dialog, dialog_area);
}