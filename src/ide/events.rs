use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum IdeEvent {
    // Application control
    Quit,
    ToggleHelp,
    ToggleCommandHelp,  // Ctrl+H
    ToggleAgenticMode,
    ShowApiConfig,
    ClearNotifications,  // Ctrl+K
    
    // Panel focus
    FocusFileExplorer,
    FocusEditor,
    FocusChat,
    CycleFocus,
    
    // Mode changes
    InsertMode,
    NormalMode,
    
    // Layout resizing
    ResizeSidebarExpand,
    ResizeSidebarShrink,
    ResizeChatExpand,
    ResizeChatShrink,
    
    // File operations
    OpenFile(PathBuf),
    SaveFile,
    SaveAsFile,
    NewFile,
    NewFolder,
    CloseFile,
    DeleteFile(PathBuf),
    RenameFile(PathBuf),
    
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    
    // Text editing
    InsertChar(char),
    Backspace,
    Delete,
    Enter,
    Tab,
    
    // Chat operations
    SendMessage,
    SendMessageWithImage,
    ClearChat,
    
    // File tree operations
    RefreshFileTree,
    ToggleFileExpand,
    
    // Mouse events
    MouseClick(u16, u16),
    MouseMove(u16, u16),
    MouseRelease(u16, u16),
    MouseScroll(i8),

    // Tab management events
    CloseTab(u32), // Close tab by ID
    SwitchToTab(usize), // Switch to tab by index
    NextTab,
    PreviousTab,
    ReorderTab { from_index: usize, to_index: usize },
    StartTabDrag(usize), // Start dragging tab at index
    EndTabDrag, // End tab dragging
    UpdateTabDrag(u16), // Update drag position
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

    pub fn poll_event(&self) -> Result<Option<IdeEvent>> {
        if event::poll(self.timeout)? {
            match event::read()? {
                Event::Key(key) => Ok(self.handle_key_event(key)),
                Event::Mouse(mouse) => Ok(self.handle_mouse_event(mouse)),
                Event::Resize(_, _) => Ok(None), // Handle resize in main loop
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn handle_key_event(&self, key: KeyEvent) -> Option<IdeEvent> {
        match key.modifiers {
            m if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) => {
                self.handle_ctrl_shift_key(key.code)
            }
            KeyModifiers::CONTROL => self.handle_ctrl_key(key.code),
            KeyModifiers::ALT => self.handle_alt_key(key.code),
            _ => self.handle_normal_key(key),
        }
    }

    fn handle_ctrl_shift_key(&self, key_code: KeyCode) -> Option<IdeEvent> {
        match key_code {
            KeyCode::Tab => Some(IdeEvent::PreviousTab),
            _ => None,
        }
    }

    fn handle_ctrl_key(&self, key_code: KeyCode) -> Option<IdeEvent> {
        match key_code {
            // Application control
            KeyCode::Char('q') | KeyCode::Char('c') => Some(IdeEvent::Quit),
            KeyCode::Char('h') => Some(IdeEvent::ToggleCommandHelp),  // Ctrl+H help
            
            // File operations
            KeyCode::Char('s') => Some(IdeEvent::SaveFile),
            KeyCode::Char('n') => Some(IdeEvent::NewFile),
            KeyCode::Char('w') => Some(IdeEvent::CloseFile),
            KeyCode::Char('o') => Some(IdeEvent::FocusFileExplorer),
            KeyCode::Char('d') => Some(IdeEvent::NewFolder),  // Create directory
            
            // Chat operations
            KeyCode::Char('l') => Some(IdeEvent::ClearChat),
            KeyCode::Enter => Some(IdeEvent::SendMessage),
            KeyCode::Char('i') => Some(IdeEvent::SendMessageWithImage),
            
            // Mode toggles
            KeyCode::Char('a') => Some(IdeEvent::ToggleAgenticMode),
            KeyCode::Char(',') => Some(IdeEvent::ShowApiConfig),  // Settings
            KeyCode::Char('k') => Some(IdeEvent::ClearNotifications),  // Clear notifications
            
            // Layout resizing
            KeyCode::Right => Some(IdeEvent::ResizeSidebarExpand),
            KeyCode::Left => Some(IdeEvent::ResizeSidebarShrink),
            KeyCode::Up => Some(IdeEvent::ResizeChatShrink),
            KeyCode::Down => Some(IdeEvent::ResizeChatExpand),
            
            // File tree
            KeyCode::Char('r') => Some(IdeEvent::RefreshFileTree),

            // Tab management
            KeyCode::Tab => Some(IdeEvent::NextTab),
            KeyCode::Char('t') => Some(IdeEvent::NewFile), // Ctrl+T for new tab

            _ => None,
        }
    }

    fn handle_alt_key(&self, key_code: KeyCode) -> Option<IdeEvent> {
        match key_code {
            // Panel focus shortcuts (Alt + number)
            KeyCode::Char('1') => Some(IdeEvent::FocusFileExplorer),
            KeyCode::Char('2') => Some(IdeEvent::FocusEditor),
            KeyCode::Char('3') => Some(IdeEvent::FocusChat),
            _ => None,
        }
    }

    fn handle_normal_key(&self, key: KeyEvent) -> Option<IdeEvent> {
        match key.code {
            // Help
            KeyCode::F(1) | KeyCode::Char('?') => Some(IdeEvent::ToggleHelp),
            
            // Mode changes
            KeyCode::Esc => Some(IdeEvent::NormalMode),
            KeyCode::Char('i') => Some(IdeEvent::InsertMode),
            
            // File operations (in normal mode)
            KeyCode::F(2) => Some(IdeEvent::RenameFile(PathBuf::new())), // F2 to rename
            KeyCode::Delete => Some(IdeEvent::DeleteFile(PathBuf::new())), // Delete key
            
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => Some(IdeEvent::NavigateUp),
            KeyCode::Down | KeyCode::Char('j') => Some(IdeEvent::NavigateDown),
            KeyCode::Left | KeyCode::Char('h') => Some(IdeEvent::NavigateLeft),
            KeyCode::Right | KeyCode::Char('l') => Some(IdeEvent::NavigateRight),
            
            // Selection/Enter
            KeyCode::Enter => Some(IdeEvent::Select),
            KeyCode::Char(' ') => Some(IdeEvent::ToggleFileExpand),
            
            // Panel cycling
            KeyCode::Tab => Some(IdeEvent::CycleFocus),
            
            // Text input (only in insert mode or chat)
            KeyCode::Char(c) => Some(IdeEvent::InsertChar(c)),
            KeyCode::Backspace => Some(IdeEvent::Backspace),
            
            _ => None,
        }
    }

    fn handle_mouse_event(&self, mouse: MouseEvent) -> Option<IdeEvent> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                Some(IdeEvent::MouseClick(mouse.column, mouse.row))
            }
            MouseEventKind::Up(MouseButton::Left) => {
                Some(IdeEvent::MouseRelease(mouse.column, mouse.row))
            }
            MouseEventKind::Moved => {
                Some(IdeEvent::MouseMove(mouse.column, mouse.row))
            }
            MouseEventKind::ScrollUp => Some(IdeEvent::MouseScroll(-1)),
            MouseEventKind::ScrollDown => Some(IdeEvent::MouseScroll(1)),
            _ => None,
        }
    }
}