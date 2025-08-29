use crate::api::{GroqClient, GroqMessage};
use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::conversation::Conversation;
use anyhow::Result;
use console::{style, Term};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, Write};

pub struct TerminalInterface {
    config: Config,
    groq_client: GroqClient,
    conversation: Conversation,
    clipboard: ClipboardManager,
    term: Term,
}

impl TerminalInterface {
    pub async fn new(config: Config) -> Result<Self> {
        let api_key = config.get_groq_key()
            .ok_or_else(|| anyhow::anyhow!("Groq API key not configured. Run: agent config --groq-key YOUR_KEY"))?;
        
        let groq_client = GroqClient::new(api_key);
        let conversation = Conversation::new();
        let clipboard = ClipboardManager::new()?;
        let term = Term::stdout();

        Ok(Self {
            config,
            groq_client,
            conversation,
            clipboard,
            term,
        })
    }

    pub async fn start_interactive_session(&mut self) -> Result<()> {
        self.print_welcome();
        
        loop {
            self.print_prompt();
            
            match self.read_user_input().await? {
                UserInput::Message(text) => {
                    self.handle_message(text, false).await?;
                }
                UserInput::MessageWithImage(text) => {
                    self.handle_message(text, true).await?;
                }
                UserInput::Command(cmd) => {
                    match self.handle_command(cmd).await? {
                        true => continue, // Continue session
                        false => break,  // Exit session
                    }
                }
                UserInput::Exit => break,
            }
        }

        Ok(())
    }

    pub async fn ask_single_question(&mut self, message: String, include_image: bool) -> Result<()> {
        self.handle_message(message, include_image).await?;
        Ok(())
    }

    fn print_welcome(&self) {
        println!("{}", style("🤖 Rust Coding Agent").cyan().bold());
        println!("{}", style("Type your message and press Enter. Use Ctrl+C to exit.").dim());
        println!("{}", style("Commands:").dim());
        println!("{}", style("  /image - Include clipboard image with next message").dim());
        println!("{}", style("  /clear - Clear conversation history").dim());
        println!("{}", style("  /help  - Show this help").dim());
        println!("{}", style("  /exit  - Exit the application").dim());
        println!();
    }

    fn print_prompt(&self) {
        print!("{} ", style("You:").green().bold());
        io::stdout().flush().unwrap();
    }

    async fn read_user_input(&self) -> Result<UserInput> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            return Ok(UserInput::Message(String::new()));
        }

        match input.as_str() {
            "/exit" | "/quit" => Ok(UserInput::Exit),
            "/image" => {
                print!("{} ", style("You (with image):").green().bold());
                io::stdout().flush().unwrap();
                let mut text = String::new();
                io::stdin().read_line(&mut text)?;
                Ok(UserInput::MessageWithImage(text.trim().to_string()))
            }
            cmd if cmd.starts_with('/') => Ok(UserInput::Command(cmd.to_string())),
            _ => Ok(UserInput::Message(input)),
        }
    }

    async fn handle_message(&mut self, text: String, include_image: bool) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        let message = if include_image {
            match self.clipboard.get_image_as_base64().await {
                Ok(image_data) => {
                    println!("{}", style("📷 Image from clipboard included").dim());
                    GroqClient::create_image_message("user", &text, &image_data)
                }
                Err(e) => {
                    println!("{}", style(format!("⚠️  Failed to get image from clipboard: {}", e)).yellow());
                    GroqClient::create_text_message("user", &text)
                }
            }
        } else {
            GroqClient::create_text_message("user", &text)
        };

        self.conversation.add_message(message);

        println!("\n{}", style("Assistant:").blue().bold());
        
        match self.get_ai_response().await {
            Ok(response) => {
                println!("{}", response);
                self.conversation.add_message(GroqClient::create_text_message("assistant", &response));
            }
            Err(e) => {
                println!("{}", style(format!("Error: {}", e)).red());
            }
        }

        println!();
        Ok(())
    }

    async fn handle_command(&mut self, command: String) -> Result<bool> {
        let result = match command.as_str() {
            "/clear" => {
                self.conversation.clear();
                println!("{}", style("🧹 Conversation cleared").dim());
                true
            }
            "/help" => {
                self.print_welcome();
                true
            }
            "/exit" | "/quit" => {
                println!("{}", style("👋 Goodbye!").dim());
                false
            }
            _ => {
                println!("{}", style(format!("Unknown command: {}", command)).yellow());
                true
            }
        };
        Ok(result)
    }

    async fn get_ai_response(&self) -> Result<String> {
        let messages = self.conversation.get_messages().clone();
        let model = self.config.get_model();
        
        self.groq_client
            .send_message(model, messages, 0.7)
            .await
    }
}

enum UserInput {
    Message(String),
    MessageWithImage(String),
    Command(String),
    Exit,
}