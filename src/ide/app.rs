use crate::api::GroqClient;
use crate::config::Config;
use crate::conversation::Conversation;
use crate::clipboard::ClipboardManager;
use crate::ide::{sidebar, editor, statusbar, events::IdeEvent};
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    MouseHover,
    MouseClick,
    FileOperation,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Insert,
    Agentic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    FileExplorer,
    Editor,
    Chat,
}

pub struct LayoutState {
    pub sidebar_width: u16,
    pub chat_height: u16,
    pub min_sidebar_width: u16,
    pub max_sidebar_width: u16,
    pub min_chat_height: u16,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_width: 30,
            chat_height: 12,
            min_sidebar_width: 20,
            max_sidebar_width: 60,
            min_chat_height: 8,
        }
    }
}

pub struct IdeApp {
    // Core components
    pub config: Config,
    pub groq_client: GroqClient,
    pub conversation: Conversation,
    pub clipboard: ClipboardManager,
    
    // IDE components
    pub sidebar: sidebar::Sidebar,
    pub editor: editor::Editor,
    pub statusbar: statusbar::StatusBar,
    
    // State management
    pub mode: AppMode,
    pub focused_panel: FocusedPanel,
    pub layout: LayoutState,
    pub should_quit: bool,
    pub show_help: bool,
    pub show_command_help: bool,
    pub show_api_config: bool,
    
    // File operation dialogs
    pub show_create_file_dialog: bool,
    pub show_create_folder_dialog: bool,
    pub show_rename_dialog: bool,
    pub dialog_input: String,
    pub operation_target: Option<PathBuf>,
    
    // Mouse tracking and notifications
    pub mouse_position: (u16, u16),
    pub last_click_position: Option<(u16, u16)>,
    pub notifications: Vec<NotificationMessage>,
    pub show_notifications: bool,
    
    // Session
    pub session_id: Uuid,
    pub current_directory: PathBuf,
}

impl IdeApp {
    pub async fn new(config: Config) -> Result<Self> {
        let api_key = config.get_groq_key()
            .ok_or_else(|| anyhow::anyhow!("Groq API key not configured. Run: agent config --groq-key YOUR_KEY"))?;
        
        let groq_client = GroqClient::new(api_key);
        let conversation = Conversation::new();
        let clipboard = ClipboardManager::new()?;
        let session_id = Uuid::new_v4();
        let current_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        // Initialize components
        let sidebar = sidebar::Sidebar::new(&current_directory)?;
        let editor = editor::Editor::new();
        let statusbar = statusbar::StatusBar::new();
        
        Ok(Self {
            config,
            groq_client,
            conversation,
            clipboard,
            sidebar,
            editor,
            statusbar,
            mode: AppMode::Normal,
            focused_panel: FocusedPanel::FileExplorer,
            layout: LayoutState::default(),
            should_quit: false,
            show_help: false,
            show_command_help: false,
            show_api_config: false,
            show_create_file_dialog: false,
            show_create_folder_dialog: false,
            show_rename_dialog: false,
            dialog_input: String::new(),
            operation_target: None,
            mouse_position: (0, 0),
            last_click_position: None,
            notifications: Vec::new(),
            show_notifications: false,
            session_id,
            current_directory,
        })
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_command_help(&mut self) {
        self.show_command_help = !self.show_command_help;
    }

    pub fn toggle_api_config(&mut self) {
        self.show_api_config = !self.show_api_config;
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn toggle_agentic_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Agentic => AppMode::Normal,
            _ => AppMode::Agentic,
        };
    }

    pub fn focus_panel(&mut self, panel: FocusedPanel) {
        self.focused_panel = panel;
    }

    pub fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::FileExplorer => FocusedPanel::Editor,
            FocusedPanel::Editor => FocusedPanel::Chat,
            FocusedPanel::Chat => FocusedPanel::FileExplorer,
        };
    }

    pub fn resize_sidebar(&mut self, delta: i16) {
        let new_width = (self.layout.sidebar_width as i16 + delta).max(self.layout.min_sidebar_width as i16);
        self.layout.sidebar_width = (new_width as u16).min(self.layout.max_sidebar_width);
    }

    pub fn resize_chat(&mut self, delta: i16) {
        let new_height = (self.layout.chat_height as i16 + delta).max(self.layout.min_chat_height as i16);
        self.layout.chat_height = (new_height as u16).min(25); // Max 25 lines for chat
    }

    pub fn show_create_file_dialog(&mut self) {
        self.show_create_file_dialog = true;
        self.dialog_input.clear();
    }

    pub fn show_create_folder_dialog(&mut self) {
        self.show_create_folder_dialog = true;
        self.dialog_input.clear();
    }

    pub fn show_rename_dialog(&mut self, target_path: PathBuf) {
        self.show_rename_dialog = true;
        self.operation_target = Some(target_path.clone());
        // Pre-populate with current filename
        self.dialog_input = target_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
    }

    pub fn hide_all_dialogs(&mut self) {
        self.show_create_file_dialog = false;
        self.show_create_folder_dialog = false;
        self.show_rename_dialog = false;
        self.dialog_input.clear();
        self.operation_target = None;
    }

    pub fn has_active_dialog(&self) -> bool {
        self.show_create_file_dialog || self.show_create_folder_dialog || self.show_rename_dialog
    }

    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) {
        let notification = NotificationMessage {
            message,
            timestamp: std::time::SystemTime::now(),
            notification_type,
        };
        
        self.notifications.push(notification);
        self.show_notifications = true;
        
        // Keep only the last 10 notifications to prevent memory buildup
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
        self.show_notifications = false;
    }

    pub fn update_mouse_position(&mut self, x: u16, y: u16) {
        self.mouse_position = (x, y);
        let context = self.get_mouse_context(x, y);
        self.add_notification(
            format!("Mouse at ({}, {}) - {}", x, y, context),
            NotificationType::MouseHover
        );
    }

    fn get_mouse_context(&self, x: u16, y: u16) -> String {
        if x < self.layout.sidebar_width {
            // Calculate dynamic areas based on notification visibility
            let file_explorer_end = if self.show_notifications && !self.notifications.is_empty() {
                // When notifications are shown: file explorer takes less space
                let total_sidebar_height = 30; // Approximate terminal height available for sidebar
                let notifications_height = 6;
                let chat_height = self.layout.chat_height;
                total_sidebar_height - notifications_height - chat_height
            } else {
                // When no notifications: file explorer takes more space
                let total_sidebar_height = 30;
                let chat_height = self.layout.chat_height;
                total_sidebar_height - chat_height
            };

            if y <= file_explorer_end {
                "File Explorer"
            } else if self.show_notifications && !self.notifications.is_empty() && y <= file_explorer_end + 6 {
                "Notifications"
            } else {
                "AI Chat"
            }
        } else {
            "Editor"
        }.to_string()
    }

    fn get_clicked_file_item(&self, x: u16, y: u16) -> Option<(PathBuf, bool)> {
        // Check if click is in file explorer area
        if x >= self.layout.sidebar_width {
            return None;
        }

        // Calculate which file item was clicked based on y coordinate
        let file_explorer_start_y = 1; // Account for border
        let relative_y = y.saturating_sub(file_explorer_start_y);
        
        let flat_list = self.sidebar.file_explorer.root.get_flat_list();
        let clicked_index = relative_y as usize;
        
        if clicked_index < flat_list.len() {
            let node = flat_list[clicked_index];
            Some((node.path.clone(), node.is_dir))
        } else {
            None
        }
    }

    fn get_file_item_index(&self, target_path: &std::path::Path) -> Option<usize> {
        let flat_list = self.sidebar.file_explorer.root.get_flat_list();
        flat_list.iter().position(|node| node.path == target_path)
    }

    fn is_folder_expanded(&self, target_path: &std::path::Path) -> bool {
        self.sidebar.file_explorer.root.find_node_by_path_read_only(target_path)
            .map(|node| node.is_expanded)
            .unwrap_or(false)
    }

    async fn execute_dialog_action(&mut self) -> Result<()> {
        if self.dialog_input.trim().is_empty() {
            self.hide_all_dialogs();
            return Ok(());
        }

        if self.show_create_file_dialog {
            match self.sidebar.file_explorer.create_file(&self.dialog_input) {
                Ok(file_path) => {
                    self.add_notification(
                        format!("📄 File '{}' created successfully", self.dialog_input),
                        NotificationType::FileOperation
                    );
                    self.editor.open_file(file_path)?;
                    self.focus_panel(FocusedPanel::Editor);
                }
                Err(e) => {
                    self.add_notification(
                        format!("❌ Failed to create file: {}", e),
                        NotificationType::FileOperation
                    );
                }
            }
        } else if self.show_create_folder_dialog {
            match self.sidebar.file_explorer.create_folder(&self.dialog_input) {
                Ok(_) => {
                    self.add_notification(
                        format!("📁 Folder '{}' created successfully", self.dialog_input),
                        NotificationType::FileOperation
                    );
                }
                Err(e) => {
                    self.add_notification(
                        format!("❌ Failed to create folder: {}", e),
                        NotificationType::FileOperation
                    );
                }
            }
        } else if self.show_rename_dialog {
            if let Some(old_path) = &self.operation_target.clone() {
                match self.sidebar.file_explorer.rename_file(old_path, &self.dialog_input) {
                    Ok(_) => {
                        self.add_notification(
                            format!("✏️ Renamed to '{}'", self.dialog_input),
                            NotificationType::FileOperation
                        );
                    }
                    Err(e) => {
                        self.add_notification(
                            format!("❌ Failed to rename: {}", e),
                            NotificationType::FileOperation
                        );
                    }
                }
            }
        }

        self.hide_all_dialogs();
        Ok(())
    }

    pub async fn handle_event(&mut self, event: IdeEvent) -> Result<()> {
        match event {
            IdeEvent::Quit => self.quit(),
            
            IdeEvent::ToggleHelp => self.toggle_help(),
            IdeEvent::ToggleCommandHelp => self.toggle_command_help(),
            IdeEvent::ShowApiConfig => self.toggle_api_config(),
            IdeEvent::ToggleAgenticMode => self.toggle_agentic_mode(),
            IdeEvent::ClearNotifications => self.clear_notifications(),
            
            IdeEvent::FocusFileExplorer => self.focus_panel(FocusedPanel::FileExplorer),
            IdeEvent::FocusEditor => self.focus_panel(FocusedPanel::Editor),
            IdeEvent::FocusChat => self.focus_panel(FocusedPanel::Chat),
            IdeEvent::CycleFocus => self.cycle_focus(),
            
            IdeEvent::InsertMode => self.set_mode(AppMode::Insert),
            IdeEvent::NormalMode => {
                if self.has_active_dialog() {
                    self.hide_all_dialogs();
                } else {
                    self.set_mode(AppMode::Normal);
                }
            }
            
            IdeEvent::ResizeSidebarExpand => self.resize_sidebar(2),
            IdeEvent::ResizeSidebarShrink => self.resize_sidebar(-2),
            IdeEvent::ResizeChatExpand => self.resize_chat(2),
            IdeEvent::ResizeChatShrink => self.resize_chat(-2),
            
            // File operations
            IdeEvent::OpenFile(path) => {
                self.editor.open_file(path)?;
                self.focus_panel(FocusedPanel::Editor);
            }
            
            IdeEvent::SaveFile => {
                if let Err(e) = self.editor.save_current_file() {
                    self.add_notification(format!("❌ Save failed: {}", e), NotificationType::FileOperation);
                } else {
                    self.add_notification("💾 File saved successfully".to_string(), NotificationType::FileOperation);
                }
            }
            
            IdeEvent::SaveAsFile => {
                // TODO: Implement save as dialog
                self.sidebar.chat.add_system_message("💡 Save As not yet implemented");
            }
            
            IdeEvent::NewFolder => {
                self.show_create_folder_dialog();
            }
            
            IdeEvent::DeleteFile(path) => {
                if let Some(target_path) = if path.as_os_str().is_empty() {
                    self.sidebar.file_explorer.get_selected()
                } else {
                    Some(path)
                } {
                    match self.sidebar.file_explorer.delete_file(&target_path) {
                        Ok(()) => {
                            let item_type = if target_path.is_dir() { "Folder" } else { "File" };
                            let name = target_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown");
                            self.add_notification(
                                format!("🗑️ {} '{}' deleted successfully", item_type, name),
                                NotificationType::FileOperation
                            );
                        }
                        Err(e) => {
                            self.add_notification(
                                format!("❌ Delete failed: {}", e),
                                NotificationType::FileOperation
                            );
                        }
                    }
                } else {
                    self.add_notification(
                        "⚠️ No file selected for deletion".to_string(),
                        NotificationType::Info
                    );
                }
            }
            
            IdeEvent::RenameFile(path) => {
                let target_path = if path.as_os_str().is_empty() {
                    self.sidebar.file_explorer.get_selected()
                } else {
                    Some(path)
                };
                
                if let Some(target_path) = target_path {
                    self.show_rename_dialog(target_path);
                } else {
                    self.add_notification(
                        "⚠️ No file selected for rename".to_string(),
                        NotificationType::Info
                    );
                }
            }
            
            IdeEvent::NewFile => {
                if self.sidebar.file_explorer.get_selected().is_some() {
                    // Show dialog to create file in selected directory
                    self.show_create_file_dialog();
                } else {
                    // Create untitled file in editor
                    self.editor.new_file();
                    self.focus_panel(FocusedPanel::Editor);
                }
            }
            
            IdeEvent::CloseFile => {
                self.editor.close_current_file();
            }
            
            // Navigation
            IdeEvent::NavigateUp => {
                match self.focused_panel {
                    FocusedPanel::FileExplorer => self.sidebar.file_explorer.navigate_up(),
                    FocusedPanel::Editor => self.editor.move_cursor_up(),
                    FocusedPanel::Chat => self.sidebar.chat.scroll_up(),
                }
            }
            
            IdeEvent::NavigateDown => {
                match self.focused_panel {
                    FocusedPanel::FileExplorer => self.sidebar.file_explorer.navigate_down(),
                    FocusedPanel::Editor => self.editor.move_cursor_down(),
                    FocusedPanel::Chat => self.sidebar.chat.scroll_down(),
                }
            }
            
            IdeEvent::NavigateLeft => {
                if self.focused_panel == FocusedPanel::Editor {
                    self.editor.move_cursor_left();
                }
            }
            
            IdeEvent::NavigateRight => {
                if self.focused_panel == FocusedPanel::Editor {
                    self.editor.move_cursor_right();
                }
            }
            
            IdeEvent::Select => {
                match self.focused_panel {
                    FocusedPanel::FileExplorer => {
                        if let Some(path) = self.sidebar.file_explorer.get_selected() {
                            if path.is_file() {
                                self.editor.open_file(path)?;
                                self.focus_panel(FocusedPanel::Editor);
                            } else {
                                self.sidebar.file_explorer.toggle_expand();
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Text input (context-aware)
            IdeEvent::InsertChar(c) => {
                if self.has_active_dialog() {
                    // Handle dialog input
                    self.dialog_input.push(c);
                } else {
                    match (self.focused_panel, self.mode) {
                        (FocusedPanel::Editor, AppMode::Insert) => {
                            self.editor.insert_char(c);
                        }
                        (FocusedPanel::Chat, _) => {
                            self.sidebar.chat.add_char(c);
                        }
                        _ => {
                            // In normal mode, certain characters have special meaning
                            if self.focused_panel == FocusedPanel::Editor && self.mode == AppMode::Normal {
                                match c {
                                    'i' => self.set_mode(AppMode::Insert),
                                    'h' => self.editor.move_cursor_left(),
                                    'j' => self.editor.move_cursor_down(),
                                    'k' => self.editor.move_cursor_up(),
                                    'l' => self.editor.move_cursor_right(),
                                    _ => {} // Ignore other characters in normal mode
                                }
                            }
                        }
                    }
                }
            }
            
            IdeEvent::Backspace => {
                if self.has_active_dialog() {
                    self.dialog_input.pop();
                } else {
                    match self.focused_panel {
                        FocusedPanel::Editor if self.mode == AppMode::Insert => {
                            self.editor.backspace();
                        }
                        FocusedPanel::Chat => {
                            self.sidebar.chat.backspace();
                        }
                        _ => {}
                    }
                }
            }
            
            IdeEvent::Enter => {
                if self.has_active_dialog() {
                    self.execute_dialog_action().await?;
                } else {
                    match self.focused_panel {
                        FocusedPanel::Editor if self.mode == AppMode::Insert => {
                            self.editor.insert_newline();
                        }
                        FocusedPanel::Chat => {
                            self.send_chat_message(false).await?;
                        }
                        FocusedPanel::FileExplorer => {
                            // Open file or toggle folder
                            if let Some(path) = self.sidebar.file_explorer.get_selected() {
                                if path.is_file() {
                                    self.editor.open_file(path)?;
                                    self.focus_panel(FocusedPanel::Editor);
                                } else {
                                    self.sidebar.file_explorer.toggle_expand();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            // Mouse events
            IdeEvent::MouseMove(x, y) => {
                self.update_mouse_position(x, y);
            }
            
            IdeEvent::MouseClick(x, y) => {
                self.last_click_position = Some((x, y));
                let context = self.get_mouse_context(x, y);
                self.add_notification(
                    format!("🖱️ Clicked at ({}, {}) in {}", x, y, context),
                    NotificationType::MouseClick
                );
                
                // Handle file explorer clicks specifically
                if context == "File Explorer" {
                    if let Some((path, is_dir)) = self.get_clicked_file_item(x, y) {
                        let file_name = path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                            
                        if is_dir {
                            // Toggle folder expand/collapse
                            if let Some(selected_index) = self.get_file_item_index(&path) {
                                // Update selection to clicked item
                                self.sidebar.file_explorer.list_state.select(Some(selected_index));
                                // Toggle the folder
                                self.sidebar.file_explorer.toggle_expand();
                                
                                // Check if folder is now expanded or collapsed
                                let is_expanded = self.is_folder_expanded(&path);
                                let action = if is_expanded { "expanded" } else { "collapsed" };
                                self.add_notification(
                                    format!("📁 Folder '{}' {}", file_name, action),
                                    NotificationType::FileOperation
                                );
                            }
                        } else {
                            // Open file in editor
                            if let Err(e) = self.editor.open_file(path.clone()) {
                                self.add_notification(
                                    format!("❌ Failed to open file '{}': {}", file_name, e),
                                    NotificationType::FileOperation
                                );
                            } else {
                                self.add_notification(
                                    format!("📄 File '{}' opened", file_name),
                                    NotificationType::FileOperation
                                );
                                self.focus_panel(FocusedPanel::Editor);
                            }
                        }
                    } else {
                        // Click in file explorer area but not on a specific item
                        self.focus_panel(FocusedPanel::FileExplorer);
                        self.add_notification("Focused File Explorer".to_string(), NotificationType::Info);
                    }
                } else {
                    // Handle other area clicks (focus changes)
                    match context.as_str() {
                        "AI Chat" => {
                            self.focus_panel(FocusedPanel::Chat);
                            self.add_notification("Focused AI Chat".to_string(), NotificationType::Info);
                        }
                        "Editor" => {
                            self.focus_panel(FocusedPanel::Editor);
                            self.add_notification("Focused Editor".to_string(), NotificationType::Info);
                        }
                        "Notifications" => {
                            // Notifications panel clicked - maybe add scroll functionality later
                            self.add_notification("Clicked in notifications area".to_string(), NotificationType::Info);
                        }
                        _ => {}
                    }
                }
            }
            
            IdeEvent::MouseScroll(delta) => {
                // TODO: Handle mouse scrolling
                match self.focused_panel {
                    FocusedPanel::Editor => {
                        // Scroll editor content
                    }
                    FocusedPanel::Chat => {
                        if delta > 0 {
                            self.sidebar.chat.scroll_down();
                        } else {
                            self.sidebar.chat.scroll_up();
                        }
                    }
                    _ => {}
                }
            }
            
            // Add other missing events
            IdeEvent::Delete => {
                if self.focused_panel == FocusedPanel::Editor && self.mode == AppMode::Insert {
                    // TODO: Implement delete character
                }
            }
            
            IdeEvent::Tab => {
                if self.focused_panel == FocusedPanel::Editor && self.mode == AppMode::Insert {
                    self.editor.insert_char('\t');
                }
            }
            
            // Chat operations
            IdeEvent::SendMessage => {
                if self.focused_panel == FocusedPanel::Chat {
                    self.send_chat_message(false).await?;
                }
            }
            
            IdeEvent::SendMessageWithImage => {
                if self.focused_panel == FocusedPanel::Chat {
                    self.send_chat_message(true).await?;
                }
            }
            
            IdeEvent::ClearChat => {
                self.sidebar.chat.clear();
                self.conversation.clear();
            }
            
            // File tree operations
            IdeEvent::RefreshFileTree => {
                self.sidebar.file_explorer.refresh()?;
            }
            
            IdeEvent::ToggleFileExpand => {
                if self.focused_panel == FocusedPanel::FileExplorer {
                    self.sidebar.file_explorer.toggle_expand();
                }
            }
        }
        
        Ok(())
    }

    async fn send_chat_message(&mut self, include_image: bool) -> Result<()> {
        let message = self.sidebar.chat.get_input_and_clear();
        if message.trim().is_empty() {
            return Ok(());
        }

        // Add user message to chat
        self.sidebar.chat.add_user_message(&message);

        let groq_message = if include_image {
            match self.clipboard.get_image_as_base64().await {
                Ok(image_data) => {
                    self.sidebar.chat.add_system_message("📷 Image included");
                    crate::api::GroqClient::create_image_message("user", &message, &image_data)
                }
                Err(e) => {
                    self.sidebar.chat.add_system_message(&format!("⚠️ Image error: {}", e));
                    crate::api::GroqClient::create_text_message("user", &message)
                }
            }
        } else {
            crate::api::GroqClient::create_text_message("user", &message)
        };

        self.conversation.add_message(groq_message);

        // Show typing indicator
        self.sidebar.chat.add_system_message("🤖 AI is typing...");

        // Get AI response
        match self.get_ai_response().await {
            Ok(response) => {
                self.sidebar.chat.remove_last_message(); // Remove typing indicator
                self.sidebar.chat.add_ai_message(&response);
                self.conversation.add_message(crate::api::GroqClient::create_text_message("assistant", &response));
            }
            Err(e) => {
                self.sidebar.chat.remove_last_message(); // Remove typing indicator
                self.sidebar.chat.add_system_message(&format!("❌ Error: {}", e));
            }
        }

        Ok(())
    }

    async fn get_ai_response(&self) -> Result<String> {
        let messages = self.conversation.get_messages().clone();
        let model = self.config.get_model();
        
        self.groq_client
            .send_message(model, messages, 0.7)
            .await
    }

    pub fn get_status_info(&self) -> statusbar::StatusInfo {
        statusbar::StatusInfo {
            mode: self.mode,
            focused_panel: self.focused_panel,
            current_file: self.editor.get_current_file_info(),
            cursor_position: self.editor.get_cursor_position(),
            is_modified: self.editor.is_current_file_modified(),
            total_files: self.editor.get_tab_count(),
        }
    }
}