use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    SwitchToChat,
    SwitchToEditor,
    SwitchToFileExplorer,
    ToggleAgenticMode,
    ToggleHelp,
    SendMessage(String),
    SendMessageWithImage(String),
    OpenFile(std::path::PathBuf),
    SaveFile,
    NewFile,
    CloseFile,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    SelectItem,
    BackspaceChar,
    InsertChar(char),
    InsertMode,
    NormalMode,
    ClearChat,
    RefreshFileTree,
}

pub struct EventHandler {
    pub timeout: Duration,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_millis(100),
        }
    }

    pub fn handle_key_event(&self, key: KeyEvent, app_mode: crate::tui::app::AppMode, active_panel: crate::tui::app::ActivePanel) -> Result<Option<AppEvent>> {
        use crate::tui::app::{AppMode, ActivePanel};

        match app_mode {
            AppMode::Insert => self.handle_insert_mode(key),
            AppMode::Normal | AppMode::Agentic => {
                match key.modifiers {
                    KeyModifiers::CONTROL => self.handle_ctrl_key(key.code),
                    KeyModifiers::ALT => self.handle_alt_key(key.code),
                    _ => self.handle_normal_key(key.code, active_panel),
                }
            }
        }
    }

    fn handle_insert_mode(&self, key: KeyEvent) -> Result<Option<AppEvent>> {
        match key.code {
            KeyCode::Esc => Ok(Some(AppEvent::NormalMode)),
            KeyCode::Char(c) => Ok(Some(AppEvent::InsertChar(c))),
            KeyCode::Backspace => Ok(Some(AppEvent::BackspaceChar)),
            KeyCode::Enter => Ok(Some(AppEvent::InsertChar('\n'))),
            KeyCode::Tab => Ok(Some(AppEvent::InsertChar('\t'))),
            _ => Ok(None),
        }
    }

    fn handle_ctrl_key(&self, key_code: KeyCode) -> Result<Option<AppEvent>> {
        match key_code {
            KeyCode::Char('q') => Ok(Some(AppEvent::Quit)),
            KeyCode::Char('c') => Ok(Some(AppEvent::Quit)),
            KeyCode::Char('s') => Ok(Some(AppEvent::SaveFile)),
            KeyCode::Char('n') => Ok(Some(AppEvent::NewFile)),
            KeyCode::Char('o') => Ok(Some(AppEvent::SwitchToFileExplorer)),
            KeyCode::Char('a') => Ok(Some(AppEvent::ToggleAgenticMode)),
            KeyCode::Char('r') => Ok(Some(AppEvent::RefreshFileTree)),
            KeyCode::Char('l') => Ok(Some(AppEvent::ClearChat)),
            _ => Ok(None),
        }
    }

    fn handle_alt_key(&self, key_code: KeyCode) -> Result<Option<AppEvent>> {
        match key_code {
            KeyCode::Char('1') => Ok(Some(AppEvent::SwitchToFileExplorer)),
            KeyCode::Char('2') => Ok(Some(AppEvent::SwitchToEditor)),
            KeyCode::Char('3') => Ok(Some(AppEvent::SwitchToChat)),
            _ => Ok(None),
        }
    }

    fn handle_normal_key(&self, key_code: KeyCode, active_panel: crate::tui::app::ActivePanel) -> Result<Option<AppEvent>> {
        use crate::tui::app::ActivePanel;

        match key_code {
            KeyCode::Char('q') => Ok(Some(AppEvent::Quit)),
            KeyCode::Char('?') => Ok(Some(AppEvent::ToggleHelp)),
            KeyCode::Char('i') => Ok(Some(AppEvent::InsertMode)),
            KeyCode::Char('a') => Ok(Some(AppEvent::ToggleAgenticMode)),
            KeyCode::Tab => self.handle_tab_navigation(active_panel),
            KeyCode::Up | KeyCode::Char('k') => Ok(Some(AppEvent::NavigateUp)),
            KeyCode::Down | KeyCode::Char('j') => Ok(Some(AppEvent::NavigateDown)),
            KeyCode::Left | KeyCode::Char('h') => Ok(Some(AppEvent::NavigateLeft)),
            KeyCode::Right | KeyCode::Char('l') => Ok(Some(AppEvent::NavigateRight)),
            KeyCode::Enter => Ok(Some(AppEvent::SelectItem)),
            KeyCode::Esc => Ok(Some(AppEvent::NormalMode)),
            _ => Ok(None),
        }
    }

    fn handle_tab_navigation(&self, current_panel: crate::tui::app::ActivePanel) -> Result<Option<AppEvent>> {
        use crate::tui::app::ActivePanel;

        let next_panel = match current_panel {
            ActivePanel::FileExplorer => AppEvent::SwitchToEditor,
            ActivePanel::Editor => AppEvent::SwitchToChat,
            ActivePanel::Chat => AppEvent::SwitchToFileExplorer,
        };

        Ok(Some(next_panel))
    }

    pub fn poll_event(&self) -> Result<Option<Event>> {
        if event::poll(self.timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}