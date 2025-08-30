mod api;
mod config;
mod clipboard;
mod conversation;
mod ide;
mod agent;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;

#[derive(Parser)]
#[command(name = "agent")]
#[command(about = "A Rust-based TUI IDE with integrated AI coding agent")]
#[command(long_about = "
A terminal-based IDE with integrated AI assistant powered by Groq API.
Features:
• Multi-tab file editor with syntax highlighting
• File explorer with create/delete/rename operations  
• AI chat with image support and agentic mode
• Vim-like navigation and keyboard shortcuts
• Mouse support for clicking and scrolling

Run without arguments to start the IDE. Use 'config' subcommand to set API keys.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure the agent (API keys, models, etc.)
    Config {
        /// Set Groq API key
        #[arg(long)]
        groq_key: Option<String>,
        /// Set default model
        #[arg(long)]
        model: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Some(Commands::Config { groq_key, model }) => {
            let mut config = config;
            let mut updates = Vec::new();
            
            if let Some(key) = groq_key {
                config.set_groq_key(key)?;
                updates.push("Groq API key updated");
            }
            if let Some(model) = model {
                config.set_model(model)?;
                updates.push("Default model updated");
            }
            
            if updates.is_empty() {
                // No changes made, start TUI with info
                let mut app = ide::IdeApp::new(config).await?;
                app.add_notification("Use config subcommand with --groq-key or --model to configure".to_string(), ide::NotificationType::Info);
                return ide::run_ide_with_app(app).await;
            } else {
                // Changes made, start TUI with success notification  
                let mut app = ide::IdeApp::new(config).await?;
                for update in updates {
                    app.add_notification(format!("✅ {}", update), ide::NotificationType::Info);
                }
                return ide::run_ide_with_app(app).await;
            }
        }
        None => {
            // Always run TUI IDE by default
            ide::run_ide(config).await?;
        }
    }

    Ok(())
}