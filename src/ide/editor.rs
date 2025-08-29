use crate::ide::app::AppMode;
use anyhow::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub file_path: Option<PathBuf>,
    pub file_name: String,
    pub content: String,
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub is_modified: bool,
}

impl EditorTab {
    pub fn new() -> Self {
        Self {
            file_path: None,
            file_name: "Untitled".to_string(),
            content: String::new(),
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            is_modified: false,
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)?;
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(Self {
            file_path: Some(path),
            file_name,
            content,
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            is_modified: false,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path {
            self.content = self.lines.join("\n");
            fs::write(path, &self.content)?;
            self.is_modified = false;
        }
        Ok(())
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_col <= line.len() {
                line.insert(self.cursor_col, c);
                self.cursor_col += 1;
                self.is_modified = true;
            }
        }
    }

    pub fn insert_newline(&mut self) {
        if self.cursor_line < self.lines.len() {
            let current_line = self.lines[self.cursor_line].clone();
            let (left, right) = current_line.split_at(self.cursor_col);
            
            self.lines[self.cursor_line] = left.to_string();
            self.lines.insert(self.cursor_line + 1, right.to_string());
            
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.is_modified = true;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            // Delete character before cursor
            if self.cursor_line < self.lines.len() {
                let line = &mut self.lines[self.cursor_line];
                if self.cursor_col <= line.len() {
                    line.remove(self.cursor_col - 1);
                    self.cursor_col -= 1;
                    self.is_modified = true;
                }
            }
        } else if self.cursor_line > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
            self.is_modified = true;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.adjust_cursor_col();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len().saturating_sub(1) {
            self.cursor_line += 1;
            self.adjust_cursor_col();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines.get(self.cursor_line)
                .map(|line| line.len())
                .unwrap_or(0);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_line) {
            if self.cursor_col < line.len() {
                self.cursor_col += 1;
            } else if self.cursor_line < self.lines.len().saturating_sub(1) {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        }
    }

    fn adjust_cursor_col(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_line) {
            self.cursor_col = self.cursor_col.min(line.len());
        }
    }

    pub fn ensure_cursor_visible(&mut self, visible_lines: usize) {
        // Adjust scroll to keep cursor visible
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor_line.saturating_sub(visible_lines - 1);
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabInfo {
    pub file_name: String,
    pub is_modified: bool,
}

pub struct Editor {
    pub tabs: Vec<EditorTab>,
    pub active_tab: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
        }
    }

    pub fn has_open_files(&self) -> bool {
        !self.tabs.is_empty()
    }

    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn new_file(&mut self) {
        let new_tab = EditorTab::new();
        self.tabs.push(new_tab);
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        // Check if file is already open
        for (index, tab) in self.tabs.iter().enumerate() {
            if let Some(tab_path) = &tab.file_path {
                if tab_path == &path {
                    self.active_tab = index;
                    return Ok(());
                }
            }
        }

        // Open new tab
        let new_tab = EditorTab::from_file(path)?;
        self.tabs.push(new_tab);
        self.active_tab = self.tabs.len() - 1;
        Ok(())
    }

    pub fn close_current_file(&mut self) {
        if !self.tabs.is_empty() {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn save_current_file(&mut self) -> Result<()> {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.save()?;
        }
        Ok(())
    }

    pub fn get_current_tab(&self) -> Option<&EditorTab> {
        self.tabs.get(self.active_tab)
    }

    pub fn get_current_tab_mut(&mut self) -> Option<&mut EditorTab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.insert_char(c);
        }
    }

    pub fn insert_newline(&mut self) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.insert_newline();
        }
    }

    pub fn backspace(&mut self) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.backspace();
        }
    }

    pub fn move_cursor_up(&mut self) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.move_cursor_up();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.move_cursor_down();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.move_cursor_left();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(tab) = self.get_current_tab_mut() {
            tab.move_cursor_right();
        }
    }

    pub fn get_tab_info(&self) -> Vec<TabInfo> {
        self.tabs.iter().map(|tab| TabInfo {
            file_name: tab.file_name.clone(),
            is_modified: tab.is_modified,
        }).collect()
    }

    pub fn get_active_tab_index(&self) -> usize {
        self.active_tab
    }

    pub fn get_current_file_info(&self) -> Option<String> {
        self.get_current_tab().map(|tab| tab.file_name.clone())
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        self.get_current_tab()
            .map(|tab| (tab.cursor_line + 1, tab.cursor_col + 1))
            .unwrap_or((0, 0))
    }

    pub fn is_current_file_modified(&self) -> bool {
        self.get_current_tab()
            .map(|tab| tab.is_modified)
            .unwrap_or(false)
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, is_focused: bool, mode: AppMode) {
        if let Some(tab) = self.get_current_tab_mut() {
            // Calculate visible lines
            let visible_lines = area.height.saturating_sub(2) as usize; // Account for borders
            tab.ensure_cursor_visible(visible_lines);

            let border_style = if is_focused {
                match mode {
                    AppMode::Insert => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    AppMode::Normal => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    AppMode::Agentic => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                }
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let title = format!(" {} {}", 
                get_file_icon(&tab.file_name),
                tab.file_name
            );

            // Create editor content with line numbers
            let mut content_lines = Vec::new();
            let start_line = tab.scroll_offset;
            let end_line = (start_line + visible_lines).min(tab.lines.len());

            for (i, line) in tab.lines[start_line..end_line].iter().enumerate() {
                let line_number = start_line + i + 1;
                let is_cursor_line = start_line + i == tab.cursor_line;
                
                let line_style = if is_cursor_line && is_focused {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                // Add line number and content
                let line_content = if line.is_empty() {
                    format!("{:3} │ ", line_number)
                } else {
                    format!("{:3} │ {}", line_number, line)
                };

                content_lines.push(Line::from(Span::styled(line_content, line_style)));
            }

            // Show cursor position in insert mode
            if is_focused && mode == AppMode::Insert {
                // This is a simplified cursor representation
                // In a real implementation, you'd want to show the actual cursor position
            }

            let editor_content = Paragraph::new(content_lines)
                .block(Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style))
                .style(Style::default().fg(Color::White));

            frame.render_widget(editor_content, area);
        }
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