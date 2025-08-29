use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use std::fs;
use std::path::PathBuf;
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxSet, SyntaxReference},
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use tui_textarea::TextArea;

pub struct EditorPanel {
    pub textarea: TextArea<'static>,
    pub current_file: Option<PathBuf>,
    pub is_modified: bool,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_syntax: Option<&'static SyntaxReference>,
}

impl EditorPanel {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📝 Editor ")
                .border_style(Style::default().fg(Color::Gray))
        );
        textarea.set_placeholder_text("Open a file or start typing...");

        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Self {
            textarea,
            current_file: None,
            is_modified: false,
            syntax_set,
            theme_set,
            current_syntax: None,
        }
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        let content = fs::read_to_string(&path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        self.textarea = TextArea::new(lines);
        self.current_file = Some(path.clone());
        self.is_modified = false;
        self.update_syntax_for_file(&path);
        self.update_title();
        
        Ok(())
    }

    pub fn save_current_file(&mut self) -> Result<()> {
        if let Some(path) = &self.current_file {
            let content = self.textarea.lines().join("\n");
            fs::write(path, content)?;
            self.is_modified = false;
            self.update_title();
        }
        Ok(())
    }

    pub fn new_file(&mut self) {
        self.textarea = TextArea::default();
        self.current_file = None;
        self.is_modified = false;
        self.current_syntax = None;
        self.update_title();
    }

    pub fn close_file(&mut self) {
        self.textarea = TextArea::default();
        self.current_file = None;
        self.is_modified = false;
        self.current_syntax = None;
        self.update_title();
    }

    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        // Mark as modified when content changes
        let content_before = self.textarea.lines().join("\n");
        
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                self.textarea.input(tui_textarea::Input::from(key));
                self.mark_modified_if_changed(content_before);
            }
            crossterm::event::KeyCode::Backspace | 
            crossterm::event::KeyCode::Delete | 
            crossterm::event::KeyCode::Enter |
            crossterm::event::KeyCode::Tab => {
                self.textarea.input(tui_textarea::Input::from(key));
                self.mark_modified_if_changed(content_before);
            }
            _ => {
                self.textarea.input(tui_textarea::Input::from(key));
            }
        }
    }

    fn mark_modified_if_changed(&mut self, content_before: String) {
        let content_after = self.textarea.lines().join("\n");
        if content_before != content_after {
            self.is_modified = true;
            self.update_title();
        }
    }

    fn update_syntax_for_file(&mut self, path: &PathBuf) {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            // Note: This is a simplified approach. In a real implementation,
            // you'd want to use a more robust syntax detection method.
            self.current_syntax = match extension {
                "rs" => self.syntax_set.find_syntax_by_extension("rs"),
                "py" => self.syntax_set.find_syntax_by_extension("py"),
                "js" | "ts" => self.syntax_set.find_syntax_by_extension("js"),
                "html" => self.syntax_set.find_syntax_by_extension("html"),
                "css" => self.syntax_set.find_syntax_by_extension("css"),
                "json" => self.syntax_set.find_syntax_by_extension("json"),
                "md" => self.syntax_set.find_syntax_by_extension("md"),
                _ => self.syntax_set.find_syntax_plain_text(),
            };
        }
    }

    fn update_title(&mut self) {
        let title = match &self.current_file {
            Some(path) => {
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown");
                let modified_indicator = if self.is_modified { " •" } else { "" };
                format!(" 📝 {} {}", filename, modified_indicator)
            }
            None => " 📝 Editor ".to_string(),
        };

        let border_color = if self.is_modified { Color::Yellow } else { Color::Gray };
        
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color))
        );
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_active: bool) {
        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if self.is_modified {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let mut textarea = self.textarea.clone();
        let title = match &self.current_file {
            Some(path) => {
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown");
                let modified_indicator = if self.is_modified { " •" } else { "" };
                format!(" 📝 {}{} ", filename, modified_indicator)
            }
            None => " 📝 Editor ".to_string(),
        };

        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        );

        textarea.render(area, buf);
    }

    pub fn get_current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }
}