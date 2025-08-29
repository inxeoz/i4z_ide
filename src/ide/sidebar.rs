pub mod file_explorer;
pub mod chat;
pub mod notifications;

use anyhow::Result;
use std::path::Path;

pub struct Sidebar {
    pub file_explorer: file_explorer::FileExplorer,
    pub chat: chat::Chat,
    pub notifications: notifications::NotificationPanel,
}

impl Sidebar {
    pub fn new(root_path: &Path) -> Result<Self> {
        let file_explorer = file_explorer::FileExplorer::new(root_path)?;
        let chat = chat::Chat::new();
        let notifications = notifications::NotificationPanel::new();
        
        Ok(Self {
            file_explorer,
            chat,
            notifications,
        })
    }
}