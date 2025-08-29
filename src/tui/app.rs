use crate::api::GroqClient;
use crate::config::Config;
use crate::conversation::Conversation;
use crate::clipboard::ClipboardManager;
use crate::tui::panels::{ChatPanel, EditorPanel, FileExplorer, StatusBar};
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Agentic,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    FileExplorer,
    Editor,
    Chat,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub mode: AppMode,
    pub active_panel: ActivePanel,
    pub current_directory: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub should_quit: bool,
    pub show_help: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Normal,
            active_panel: ActivePanel::FileExplorer,
            current_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            selected_file: None,
            should_quit: false,
            show_help: false,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub config: Config,
    pub groq_client: GroqClient,
    pub conversation: Conversation,
    pub clipboard: ClipboardManager,
    pub chat_panel: ChatPanel,
    pub editor_panel: EditorPanel,
    pub file_explorer: FileExplorer,
    pub status_bar: StatusBar,
    pub session_id: Uuid,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let api_key = config.get_groq_key()
            .ok_or_else(|| anyhow::anyhow!("Groq API key not configured. Run: agent config --groq-key YOUR_KEY"))?;
        
        let groq_client = GroqClient::new(api_key);
        let conversation = Conversation::new();
        let clipboard = ClipboardManager::new()?;
        let state = AppState::default();
        
        let chat_panel = ChatPanel::new();
        let editor_panel = EditorPanel::new();
        let file_explorer = FileExplorer::new(state.current_directory.clone())?;
        let status_bar = StatusBar::new();
        let session_id = Uuid::new_v4();

        Ok(Self {
            state,
            config,
            groq_client,
            conversation,
            clipboard,
            chat_panel,
            editor_panel,
            file_explorer,
            status_bar,
            session_id,
        })
    }

    pub fn switch_mode(&mut self, mode: AppMode) {
        self.state.mode = mode;
    }

    pub fn switch_panel(&mut self, panel: ActivePanel) {
        self.state.active_panel = panel;
    }

    pub fn toggle_agentic_mode(&mut self) {
        self.state.mode = match self.state.mode {
            AppMode::Agentic => AppMode::Normal,
            _ => AppMode::Agentic,
        };
    }

    pub fn quit(&mut self) {
        self.state.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.state.show_help = !self.state.show_help;
    }

    pub async fn send_message(&mut self, message: String, include_image: bool) -> Result<()> {
        let groq_message = if include_image {
            match self.clipboard.get_image_as_base64().await {
                Ok(image_data) => {
                    self.chat_panel.add_system_message("📷 Image from clipboard included".to_string());
                    crate::api::GroqClient::create_image_message("user", &message, &image_data)
                }
                Err(e) => {
                    self.chat_panel.add_system_message(format!("⚠️ Failed to get image: {}", e));
                    crate::api::GroqClient::create_text_message("user", &message)
                }
            }
        } else {
            crate::api::GroqClient::create_text_message("user", &message)
        };

        self.conversation.add_message(groq_message);
        self.chat_panel.add_user_message(message.clone());

        // Get AI response
        match self.get_ai_response().await {
            Ok(response) => {
                self.conversation.add_message(crate::api::GroqClient::create_text_message("assistant", &response));
                self.chat_panel.add_assistant_message(response);
            }
            Err(e) => {
                self.chat_panel.add_system_message(format!("❌ Error: {}", e));
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

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        self.state.selected_file = Some(path.clone());
        self.editor_panel.open_file(path)?;
        self.switch_panel(ActivePanel::Editor);
        Ok(())
    }

    pub fn save_current_file(&mut self) -> Result<()> {
        self.editor_panel.save_current_file()
    }

    pub fn navigate_to_directory(&mut self, path: PathBuf) -> Result<()> {
        self.state.current_directory = path.clone();
        self.file_explorer.navigate_to(path)?;
        Ok(())
    }

    pub fn execute_agentic_action(&mut self, action: String) -> Result<()> {
        // This will be expanded with actual agent capabilities
        self.chat_panel.add_system_message(format!("🤖 Executing: {}", action));
        Ok(())
    }
}