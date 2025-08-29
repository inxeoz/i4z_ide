use crate::api::GroqClient;
use crate::config::Config;
use crate::conversation::Conversation;
use crate::clipboard::ClipboardManager;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Clear},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Agentic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Chat,
    Help,
}

pub struct SimpleTuiApp {
    pub config: Config,
    pub groq_client: GroqClient,
    pub conversation: Conversation,
    pub clipboard: ClipboardManager,
    pub mode: AppMode,
    pub active_panel: ActivePanel,
    pub should_quit: bool,
    pub show_help: bool,
    pub messages: Vec<String>,
    pub input: String,
    pub session_id: Uuid,
}

impl SimpleTuiApp {
    pub async fn new(config: Config) -> Result<Self> {
        let api_key = config.get_groq_key()
            .ok_or_else(|| anyhow::anyhow!("Groq API key not configured. Run: agent config --groq-key YOUR_KEY"))?;
        
        let groq_client = GroqClient::new(api_key);
        let conversation = Conversation::new();
        let clipboard = ClipboardManager::new()?;
        let session_id = Uuid::new_v4();

        Ok(Self {
            config,
            groq_client,
            conversation,
            clipboard,
            mode: AppMode::Normal,
            active_panel: ActivePanel::Chat,
            should_quit: false,
            show_help: false,
            messages: vec!["Welcome to Rust Coding Agent! 🦀".to_string()],
            input: String::new(),
            session_id,
        })
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.active_panel = if self.show_help {
            ActivePanel::Help
        } else {
            ActivePanel::Chat
        };
    }

    pub fn toggle_agentic_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Normal => AppMode::Agentic,
            AppMode::Agentic => AppMode::Normal,
        };
    }

    pub async fn send_message(&mut self, include_image: bool) -> Result<()> {
        if self.input.trim().is_empty() {
            return Ok(());
        }

        let message = self.input.clone();
        self.messages.push(format!("🧑 You: {}", message));

        let groq_message = if include_image {
            match self.clipboard.get_image_as_base64().await {
                Ok(image_data) => {
                    self.messages.push("📷 Image from clipboard included".to_string());
                    crate::api::GroqClient::create_image_message("user", &message, &image_data)
                }
                Err(e) => {
                    self.messages.push(format!("⚠️ Failed to get image: {}", e));
                    crate::api::GroqClient::create_text_message("user", &message)
                }
            }
        } else {
            crate::api::GroqClient::create_text_message("user", &message)
        };

        self.conversation.add_message(groq_message);
        self.input.clear();

        // Show typing indicator
        self.messages.push("🤖 Assistant is typing...".to_string());

        // Get AI response
        match self.get_ai_response().await {
            Ok(response) => {
                // Remove typing indicator
                self.messages.pop();
                self.messages.push(format!("🤖 Assistant: {}", response));
                self.conversation.add_message(crate::api::GroqClient::create_text_message("assistant", &response));
            }
            Err(e) => {
                // Remove typing indicator
                self.messages.pop();
                self.messages.push(format!("❌ Error: {}", e));
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

    pub fn clear_chat(&mut self) {
        self.messages.clear();
        self.conversation.clear();
        self.messages.push("Chat cleared! 🧹".to_string());
    }
}

pub async fn run_simple_tui(config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = SimpleTuiApp::new(config).await?;

    // Run the main loop
    let result = run_tui_loop(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut SimpleTuiApp,
) -> Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| draw_ui(frame, app))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.quit();
                        }
                        KeyCode::Char('?') => {
                            app.toggle_help();
                        }
                        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.toggle_agentic_mode();
                        }
                        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.clear_chat();
                        }
                        KeyCode::Enter => {
                            if !app.show_help {
                                app.send_message(false).await?;
                            }
                        }
                        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !app.show_help {
                                app.send_message(true).await?;
                            }
                        }
                        KeyCode::Char(c) => {
                            if !app.show_help {
                                app.input.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if !app.show_help {
                                app.input.pop();
                            }
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal was resized, will be handled on next draw
                }
                _ => {}
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn draw_ui(frame: &mut Frame, app: &SimpleTuiApp) {
    let size = frame.size();

    if app.show_help {
        draw_help(frame, app);
        return;
    }

    // Main layout: [Header] [Messages] [Input] [Footer]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(5),      // Messages
            Constraint::Length(3),   // Input
            Constraint::Length(1),   // Footer
        ])
        .split(size);

    // Header
    let mode_text = match app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Agentic => "🤖 AGENTIC",
    };

    let mode_color = match app.mode {
        AppMode::Normal => Color::Green,
        AppMode::Agentic => Color::Magenta,
    };

    let header = Paragraph::new(format!("🦀 Rust Coding Agent - {} MODE", mode_text))
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));

    frame.render_widget(header, chunks[0]);

    // Messages
    let messages: Vec<ListItem> = app.messages
        .iter()
        .map(|msg| ListItem::new(Line::from(msg.as_str())))
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" 💬 Chat ")
            .border_style(Style::default().fg(Color::Blue)));

    frame.render_widget(messages_list, chunks[1]);

    // Input
    let input_title = if app.mode == AppMode::Agentic {
        " 🤖 Agent Input (Enter: Send | Ctrl+I: Send with Image) "
    } else {
        " 💬 Your Message (Enter: Send | Ctrl+I: Send with Image) "
    };

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(input_title)
            .border_style(Style::default().fg(Color::Yellow)));

    frame.render_widget(input, chunks[2]);

    // Footer
    let footer = Paragraph::new("Ctrl+Q: Quit | Ctrl+A: Toggle Agentic | Ctrl+L: Clear | ?: Help")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[3]);
}

fn draw_help(frame: &mut Frame, app: &SimpleTuiApp) {
    let size = frame.size();

    // Clear the area
    frame.render_widget(Clear, size);

    let help_text = vec![
        "🦀 Rust Coding Agent - TUI Help",
        "",
        "📝 Chat Commands:",
        "  Enter       - Send message",
        "  Ctrl+I      - Send message with clipboard image",
        "  Ctrl+L      - Clear chat history",
        "",
        "🔧 Mode Controls:",
        "  Ctrl+A      - Toggle agentic mode",
        "  ?           - Toggle this help screen",
        "",
        "🚪 System:",
        "  Ctrl+Q      - Quit application",
        "  Esc         - Also quits (with Ctrl)",
        "",
        "🤖 Agentic Mode:",
        "  In agentic mode, the AI can execute",
        "  file operations and system commands",
        "  based on your messages.",
        "",
        "💡 Tips:",
        "  - Copy images to clipboard before using Ctrl+I",
        "  - Use natural language for file operations",
        "  - Ask questions about your code",
        "",
        "Press ? to return to chat",
    ];

    let help_lines: Vec<Line> = help_text
        .iter()
        .map(|line| {
            if line.starts_with('🦀') || line.starts_with('📝') || line.starts_with('🔧') || 
               line.starts_with('🚪') || line.starts_with('🤖') || line.starts_with('💡') {
                Line::from(Span::styled(*line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            } else if line.starts_with("  ") {
                let parts: Vec<&str> = line.splitn(2, '-').collect();
                if parts.len() == 2 {
                    Line::from(vec![
                        Span::styled(parts[0], Style::default().fg(Color::Yellow)),
                        Span::raw("- "),
                        Span::styled(parts[1].trim(), Style::default().fg(Color::Gray)),
                    ])
                } else {
                    Line::from(Span::styled(*line, Style::default().fg(Color::Gray)))
                }
            } else {
                Line::from(Span::raw(*line))
            }
        })
        .collect();

    let help_paragraph = Paragraph::new(help_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" ❓ Help ")
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    frame.render_widget(help_paragraph, size);
}