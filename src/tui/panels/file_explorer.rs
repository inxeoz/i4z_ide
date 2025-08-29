use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use std::fs;
use std::path::PathBuf;
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn new(path: PathBuf) -> Result<Self> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_dir = path.is_dir();
        let mut children = Vec::new();

        if is_dir {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    // Skip hidden files and directories
                    if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                        if !file_name.starts_with('.') {
                            if let Ok(child_node) = FileNode::new(entry_path) {
                                children.push(child_node);
                            }
                        }
                    }
                }
            }
            // Sort: directories first, then files, both alphabetically
            children.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });
        }

        Ok(Self {
            path,
            name,
            is_dir,
            children,
        })
    }

    pub fn to_tree_item(&self) -> TreeItem<'static> {
        let icon = if self.is_dir {
            if self.children.is_empty() { "📁" } else { "📂" }
        } else {
            match self.path.extension().and_then(|ext| ext.to_str()) {
                Some("rs") => "🦀",
                Some("py") => "🐍",
                Some("js") | Some("ts") => "📜",
                Some("html") => "🌐",
                Some("css") => "🎨",
                Some("json") => "📋",
                Some("md") => "📄",
                Some("txt") => "📃",
                Some("toml") | Some("yaml") | Some("yml") => "⚙️",
                _ => "📄",
            }
        };

        let display_name = format!("{} {}", icon, self.name);
        let children: Vec<TreeItem> = self.children
            .iter()
            .map(|child| child.to_tree_item())
            .collect();

        TreeItem::new_leaf(display_name).children(children)
    }
}

pub struct FileExplorer {
    pub root: FileNode,
    pub tree_state: TreeState<String>,
    pub current_directory: PathBuf,
}

impl FileExplorer {
    pub fn new(directory: PathBuf) -> Result<Self> {
        let root = FileNode::new(directory.clone())?;
        let tree_state = TreeState::default();

        Ok(Self {
            root,
            tree_state,
            current_directory: directory,
        })
    }

    pub fn navigate_to(&mut self, directory: PathBuf) -> Result<()> {
        self.current_directory = directory.clone();
        self.root = FileNode::new(directory)?;
        self.tree_state = TreeState::default();
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.root = FileNode::new(self.current_directory.clone())?;
        Ok(())
    }

    pub fn navigate_up(&mut self) {
        self.tree_state.key_up();
    }

    pub fn navigate_down(&mut self) {
        self.tree_state.key_down();
    }

    pub fn expand_selected(&mut self) {
        if let Some(selected) = self.tree_state.selected().clone() {
            self.tree_state.toggle_selected();
        }
    }

    pub fn get_selected_path(&self) -> Option<PathBuf> {
        if let Some(selected_keys) = self.tree_state.selected().as_ref() {
            if selected_keys.is_empty() {
                return Some(self.current_directory.clone());
            }

            let mut current_node = &self.root;
            let mut path = self.current_directory.clone();

            // Navigate through the tree based on selected keys
            for key in selected_keys {
                if let Ok(index) = key.parse::<usize>() {
                    if let Some(child) = current_node.children.get(index) {
                        path = child.path.clone();
                        current_node = child;
                    }
                }
            }

            Some(path)
        } else {
            None
        }
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        if let Some(path) = self.get_selected_path() {
            if path.is_file() {
                Some(path)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_active: bool) {
        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" 📁 {} ", self.current_directory.display()))
            .border_style(border_style);

        let tree_items = vec![self.root.to_tree_item()];
        
        let tree = Tree::new(&tree_items)
            .expect("Failed to create tree")
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            );

        ratatui::widgets::StatefulWidget::render(tree, area, buf, &mut self.tree_state);
    }

    pub fn go_to_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.current_directory.parent() {
            self.navigate_to(parent.to_path_buf())?;
        }
        Ok(())
    }

    pub fn enter_directory(&mut self) -> Result<()> {
        if let Some(selected_path) = self.get_selected_path() {
            if selected_path.is_dir() && selected_path != self.current_directory {
                self.navigate_to(selected_path)?;
            }
        }
        Ok(())
    }
}