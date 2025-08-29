mod api;
mod cli;
mod config;
mod clipboard;
mod conversation;
mod ide;
mod agent;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cli::TerminalInterface;
use config::Config;

#[derive(Parser)]
#[command(name = "agent")]
#[command(about = "A Rust-based coding agent with Groq API support")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start TUI interface (default)
    Tui,
    /// Start interactive chat session (legacy CLI)
    Chat,
    /// Configure the agent
    Config {
        /// Set Groq API key
        #[arg(long)]
        groq_key: Option<String>,
        /// Set default model
        #[arg(long)]
        model: Option<String>,
    },
    /// Send a single message
    Ask {
        /// The message to send
        message: String,
        /// Include clipboard image
        #[arg(long)]
        image: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Some(Commands::Tui) | None => {
            ide::run_ide(config).await?;
        }
        Some(Commands::Chat) => {
            let mut terminal = TerminalInterface::new(config).await?;
            terminal.start_interactive_session().await?;
        }
        Some(Commands::Config { groq_key, model }) => {
            let mut config = config;
            if let Some(key) = groq_key {
                config.set_groq_key(key)?;
                println!("Groq API key updated");
            }
            if let Some(model) = model {
                config.set_model(model)?;
                println!("Default model updated");
            }
        }
        Some(Commands::Ask { message, image }) => {
            let mut terminal = TerminalInterface::new(config).await?;
            terminal.ask_single_question(message, image).await?;
        }
    }

    Ok(())
}