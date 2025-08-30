pub mod app;
pub mod layout;
pub mod sidebar;
pub mod editor;
pub mod statusbar;
pub mod events;

pub use app::{IdeApp, NotificationType};
pub use events::EventHandler;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use crate::config::Config;

pub async fn run_ide(config: Config) -> Result<()> {
    let app = IdeApp::new(config).await?;
    run_ide_with_app(app).await
}

pub async fn run_ide_with_app(mut app: IdeApp) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut event_handler = EventHandler::new();

    // Run the main loop
    let result = run_ide_loop(&mut terminal, &mut app, &mut event_handler).await;

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

async fn run_ide_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut IdeApp,
    event_handler: &mut EventHandler,
) -> Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            layout::draw_ide(frame, app);
        })?;

        // Handle events
        if let Some(event) = event_handler.poll_event()? {
            app.handle_event(event).await?;
        }

        // Check if we should quit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}