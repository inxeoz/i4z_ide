use anyhow::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn new(path: PathBuf, depth: usize) -> Result<Self> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_dir = path.is_dir();
        let mut children = Vec::new();

        if is_dir {
            if let Ok(entries) = fs::read_dir(&path) {
                let mut valid_entries: Vec<_> = entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        // Filter out hidden files and common ignored directories
                        if let Some(file_name) = entry.file_name().to_str() {
                            !file_name.starts_with('.') && 
                            file_name != "target" && 
                            file_name != "node_modules"
                        } else {
                            false
                        }
                    })
                    .collect();

                // Sort: directories first, then files, both alphabetically
                valid_entries.sort_by(|a, b| {
                    let a_is_dir = a.path().is_dir();
                    let b_is_dir = b.path().is_dir();
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                for entry in valid_entries {
                    if let Ok(child_node) = FileNode::new(entry.path(), depth + 1) {
                        children.push(child_node);
                    }
                }
            }
        }

        Ok(Self {
            path,
            name,
            is_dir,
            is_expanded: false,
            depth,
            children,
        })
    }

    pub fn get_display_name(&self) -> String {
        let indent = "  ".repeat(self.depth);
        
        if self.is_dir {
            let expand_indicator = if self.is_expanded { "▼" } else { "▶" };
            let folder_icon = if self.is_expanded { "📂" } else { "📁" };
            format!("{}{} {} {}", indent, expand_indicator, folder_icon, self.name)
        } else {
            let file_icon = get_file_icon(&self.name);
            // Add some spacing to align with folders
            format!("{}  {} {}", indent, file_icon, self.name)
        }
    }

    pub fn toggle_expand(&mut self) {
        if self.is_dir {
            self.is_expanded = !self.is_expanded;
        }
    }

    pub fn get_flat_list(&self) -> Vec<&FileNode> {
        let mut result = vec![self];
        
        if self.is_dir && self.is_expanded {
            for child in &self.children {
                result.extend(child.get_flat_list());
            }
        }
        
        result
    }

    pub fn find_node_at_index(&mut self, index: usize) -> Option<&mut FileNode> {
        let target_path = {
            let flat_list = self.get_flat_list();
            if index < flat_list.len() {
                flat_list[index].path.clone()
            } else {
                return None;
            }
        };
        
        self.find_node_by_path(&target_path)
    }
    
    fn find_node_by_path(&mut self, target_path: &std::path::Path) -> Option<&mut FileNode> {
        if self.path == target_path {
            return Some(self);
        }
        
        if self.is_dir && self.is_expanded {
            for child in &mut self.children {
                if let Some(found) = child.find_node_by_path(target_path) {
                    return Some(found);
                }
            }
        }
        
        None
    }

    pub fn find_node_by_path_read_only(&self, target_path: &std::path::Path) -> Option<&FileNode> {
        if self.path == target_path {
            return Some(self);
        }
        
        if self.is_dir && self.is_expanded {
            for child in &self.children {
                if let Some(found) = child.find_node_by_path_read_only(target_path) {
                    return Some(found);
                }
            }
        }
        
        None
    }
}

pub struct FileExplorer {
    pub root: FileNode,
    pub list_state: ListState,
    pub current_directory: PathBuf,
}

impl FileExplorer {
    pub fn new(root_path: &Path) -> Result<Self> {
        let root = FileNode::new(root_path.to_path_buf(), 0)?;
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Ok(Self {
            root,
            list_state,
            current_directory: root_path.to_path_buf(),
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        let selected_index = self.list_state.selected().unwrap_or(0);
        self.root = FileNode::new(self.current_directory.clone(), 0)?;
        
        // Try to maintain selection after refresh
        let flat_list = self.root.get_flat_list();
        let new_selected = selected_index.min(flat_list.len().saturating_sub(1));
        self.list_state.select(Some(new_selected));
        
        Ok(())
    }

    pub fn navigate_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected > 0 {
                self.list_state.select(Some(selected - 1));
            }
        }
    }

    pub fn navigate_down(&mut self) {
        let flat_list = self.root.get_flat_list();
        if let Some(selected) = self.list_state.selected() {
            if selected < flat_list.len().saturating_sub(1) {
                self.list_state.select(Some(selected + 1));
            }
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(selected_index) = self.list_state.selected() {
            if let Some(node) = self.root.find_node_at_index(selected_index) {
                node.toggle_expand();
            }
        }
    }

    pub fn get_selected(&self) -> Option<PathBuf> {
        if let Some(selected_index) = self.list_state.selected() {
            let flat_list = self.root.get_flat_list();
            flat_list.get(selected_index).map(|node| node.path.clone())
        } else {
            None
        }
    }

    pub fn create_file(&mut self, name: &str) -> Result<PathBuf> {
        let selected_dir = self.get_selected_directory();
        let file_path = selected_dir.join(name);
        
        if file_path.exists() {
            return Err(anyhow::anyhow!("File already exists: {}", name));
        }
        
        fs::File::create(&file_path)?;
        self.refresh()?;
        Ok(file_path)
    }

    pub fn create_folder(&mut self, name: &str) -> Result<PathBuf> {
        let selected_dir = self.get_selected_directory();
        let folder_path = selected_dir.join(name);
        
        if folder_path.exists() {
            return Err(anyhow::anyhow!("Folder already exists: {}", name));
        }
        
        fs::create_dir_all(&folder_path)?;
        self.refresh()?;
        Ok(folder_path)
    }

    pub fn delete_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist"));
        }
        
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        
        self.refresh()?;
        Ok(())
    }

    pub fn rename_file(&mut self, old_path: &Path, new_name: &str) -> Result<PathBuf> {
        if !old_path.exists() {
            return Err(anyhow::anyhow!("File does not exist"));
        }
        
        let parent_dir = old_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot get parent directory"))?;
        let new_path = parent_dir.join(new_name);
        
        if new_path.exists() {
            return Err(anyhow::anyhow!("Target name already exists: {}", new_name));
        }
        
        fs::rename(old_path, &new_path)?;
        self.refresh()?;
        Ok(new_path)
    }

    fn get_selected_directory(&self) -> PathBuf {
        if let Some(selected_path) = self.get_selected() {
            if selected_path.is_dir() {
                selected_path
            } else {
                selected_path.parent().unwrap_or(&self.current_directory).to_path_buf()
            }
        } else {
            self.current_directory.clone()
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let flat_list = self.root.get_flat_list();
        let items: Vec<ListItem> = flat_list
            .iter()
            .map(|node| {
                let display_name = node.get_display_name();
                let style = if node.is_dir {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(display_name, style)))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title(format!(" 📁 {} ", self.current_directory.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Root")))
                .borders(Borders::ALL)
                .border_style(border_style))
            .highlight_style(
                Style::default()
                    .bg(if is_focused { Color::Cyan } else { Color::DarkGray })
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            );

        frame.render_stateful_widget(list, area, &mut self.list_state.clone());
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
        "png" | "jpg" | "jpeg" | "gif" => "🖼️",
        "svg" => "🎨",
        "xml" => "📰",
        "csv" => "📊",
        "pdf" => "📕",
        "zip" | "tar" | "gz" => "📦",
        _ => "📄",
    }
}