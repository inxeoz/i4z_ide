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

fn draw_sidebar(frame: &mut Frame, app: &mut IdeApp, area: Rect) {
    if app.show_notifications && !app.notifications.is_empty() {
        // Split sidebar vertically: [File Explorer] [Separator] [Notifications] [Separator] [Chat]
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),                                    // File explorer (flexible, minimum 8 lines)
                Constraint::Length(1),                                 // Separator
                Constraint::Length(app.layout.notification_height),    // Notifications (adjustable height)
                Constraint::Length(1),                                 // Separator
                Constraint::Length(app.layout.chat_height),            // Chat (adjustable height)
            ])
            .split(area);

        // Draw file explorer
        app.sidebar.file_explorer.draw(
            frame, 
            sidebar_chunks[0], 
            app.focused_panel == FocusedPanel::FileExplorer
        );

        // Draw separator between file explorer and notifications
        draw_horizontal_separator(frame, sidebar_chunks[1], "━", Color::DarkGray);

        // Draw notifications
        app.sidebar.notifications.draw(
            frame,
            sidebar_chunks[2],
            &app.notifications,
            app.focused_panel == FocusedPanel::Notifications
        );

        // Draw separator between notifications and chat
        draw_horizontal_separator(frame, sidebar_chunks[3], "━", Color::DarkGray);

        // Draw chat
        app.sidebar.chat.draw(
            frame, 
            sidebar_chunks[4], 
            app.focused_panel == FocusedPanel::Chat
        );

        // Update component areas for mouse coordinate mapping (with notifications)
        app.update_component_areas(
            sidebar_chunks[0],  // file explorer
            sidebar_chunks[2],  // notifications
            sidebar_chunks[4],  // chat
            Rect::new(0, 0, 0, 0) // editor (will be updated in main area)
        );
    } else {
        // Split sidebar vertically: [File Explorer] [Separator] [Chat] (2 blocks layout)
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),                           // File explorer (flexible)
                Constraint::Length(1),                         // Separator
                Constraint::Length(app.layout.chat_height),    // Chat (adjustable height)
            ])
            .split(area);

        // Draw file explorer
        app.sidebar.file_explorer.draw(
            frame, 
            sidebar_chunks[0], 
            app.focused_panel == FocusedPanel::FileExplorer
        );

        // Draw separator between file explorer and chat
        draw_horizontal_separator(frame, sidebar_chunks[1], "━", Color::DarkGray);

        // Draw chat
        app.sidebar.chat.draw(
            frame, 
            sidebar_chunks[2], 
            app.focused_panel == FocusedPanel::Chat
        );

        // Update component areas for mouse coordinate mapping (without notifications)
        app.update_component_areas(
            sidebar_chunks[0],  // file explorer
            Rect::new(0, 0, 0, 0), // no notifications
            sidebar_chunks[2],  // chat
            Rect::new(0, 0, 0, 0) // editor (will be updated in main area)
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
    
    // Update editor area for mouse coordinate mapping
    app.layout.editor_area = main_chunks[0];
    
    // Draw status bar
    let status_info = app.get_status_info();
    app.statusbar.draw(frame, main_chunks[1], &status_info);
}

fn draw_editor_area(frame: &mut Frame, app: &mut IdeApp, area: Rect) {
    // Editor now handles tabs internally, so just give it the full area
    app.editor.draw(
        frame, 
        area, 
        app.focused_panel == FocusedPanel::Editor,
        app.mode
    );
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

pub fn get_file_icon(filename: &str) -> &'static str {
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

pub fn get_tab_click_info(app: &crate::ide::app::IdeApp, x: u16, y: u16, area: Rect) -> Option<(usize, bool)> {
    let tabs = app.editor.get_tab_info();
    if tabs.is_empty() {
        return None;
    }

    // Tabs are now inside the editor border, so adjust for the border
    let tab_area_y = area.y + 1; // +1 for top border
    let tab_area_x = area.x + 1; // +1 for left border
    let tab_area_width = area.width.saturating_sub(2); // -2 for left and right borders
    
    // Debug the tab area calculation in layout function
    // Note: Can't add notifications from here, but this helps us understand the calculation
    
    // Allow clicks within the tab area (which is now inside the editor border)
    if y != tab_area_y || x < tab_area_x || x >= tab_area_x + tab_area_width {
        return None;
    }

    let mouse_x = x;
    let mouse_y = y;
    let is_hovering_tabs = mouse_y == tab_area_y && mouse_x >= tab_area_x && mouse_x < tab_area_x + tab_area_width;

    // Use the same logic as draw_tabs to calculate positions
    let mut tab_spans_lengths = Vec::new();
    
    for (i, tab) in tabs.iter().enumerate() {
        let is_modified = tab.is_modified;

        // Calculate tab position - tabs start at the inner area (inside border)
        let tab_start_x = tab_area_x + tab_spans_lengths.iter().sum::<u16>();
        
        // Calculate the actual tab content to get precise width (same as in draw_tabs)
        let modified_indicator = if is_modified { "●" } else { "" };
        let base_tab_text = format!(" {} {}{} ",
            get_file_icon(&tab.file_name),
            tab.file_name,
            modified_indicator
        );
        let base_tab_width = base_tab_text.len() as u16;
        let base_tab_end_x = tab_start_x + base_tab_width;
        
        // Check if mouse is hovering over this specific tab (including close button area)
        let is_hovering_this_tab = is_hovering_tabs && mouse_x >= tab_start_x && mouse_x < base_tab_end_x + 3; // +3 for close button
        let show_close_button = is_hovering_this_tab;

        // Calculate complete tab content with close button
        let close_button = if show_close_button { " ✕" } else { "" };
        let tab_text = format!(" {} {}{}{} ",
            get_file_icon(&tab.file_name),
            tab.file_name,
            modified_indicator,
            close_button
        );

        let tab_width = tab_text.len() as u16;
        let tab_end_x = tab_start_x + tab_width;

        if x >= tab_start_x && x < tab_end_x {
            // Check if click is on close button (only if it's visible)
            if show_close_button {
                let close_button_start = base_tab_end_x; // Close button starts after base content
                let close_button_end = close_button_start + 3; // " ✕ " is 3 characters
                let is_close_button = x >= close_button_start && x < close_button_end;
                
                // Debug info is now handled through notifications in the calling code
                
                return Some((i, is_close_button));
            } else {
                return Some((i, false)); // No close button visible, so not a close click
            }
        }

        // Add this tab's width to the running total (like the spans in draw_tabs)
        tab_spans_lengths.push(tab_width);
        if i < tabs.len() - 1 {
            tab_spans_lengths.push(1); // +1 for separator "│"
        }
    }

    // Check for new tab button
    let new_tab_text = " + ";
    let new_tab_start = area.x + tab_spans_lengths.iter().sum::<u16>();
    let new_tab_end = new_tab_start + new_tab_text.len() as u16;
    if x >= new_tab_start && x < new_tab_end {
        return Some((usize::MAX, false)); // Special value for new tab
    }

    None
}

fn draw_horizontal_separator(frame: &mut Frame, area: Rect, separator_char: &str, color: Color) {
    let separator_text = separator_char.repeat(area.width as usize);
    let separator = Paragraph::new(separator_text)
        .style(Style::default().fg(color));
    frame.render_widget(separator, area);
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