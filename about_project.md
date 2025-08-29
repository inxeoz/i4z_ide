# Rust Coding Agent - Terminal IDE Project

## Project Overview

This is a Rust-based terminal IDE that provides a VSCode-like interface entirely in the terminal. It features a complete TUI (Terminal User Interface) with file explorer, multi-tab editor, AI chat integration via Groq API, and agentic capabilities where the AI can execute file operations and system commands. Built with ratatui for the interface and designed for developers who want a powerful coding environment in their terminal.

## Key Features

### 🎨 Complete IDE Interface
- **VSCode-like Layout**: Resizable sidebar + main editor + status bar
- **File Explorer**: Tree view with folder navigation and file icons
- **Multi-tab Editor**: Open multiple files with tab management
- **AI Chat Panel**: Integrated AI assistant in sidebar
- **Status Bar**: File info, cursor position, mode indicators

### 🤖 AI Integration
- **Groq API Support**: All models (llama, mixtral, gemma)
- **Image Support**: Paste images from clipboard (Ctrl+I)
- **Agentic Mode**: AI can execute file operations and commands
- **Context Awareness**: Maintains conversation history

### ⚡ Developer Experience
- **Vim-like Navigation**: Familiar keyboard shortcuts
- **Syntax Highlighting**: File type recognition with icons
- **Responsive Layout**: Adapts to terminal size
- **Fast Performance**: Built in Rust for speed

## Project Structure

### Core Modules (`src/`)

**`main.rs`** - Entry point with CLI command parsing
- Handles command-line arguments (`tui`, `chat`, `config`, `ask`)
- Routes to appropriate interface based on command

**`api.rs`** - Groq API client
- Handles HTTP requests to Groq API
- Manages API authentication and model selection

**`config.rs`** - Configuration management
- Loads/saves user settings (API keys, models)
- Manages application preferences

**`conversation.rs`** - Chat history management
- Stores conversation context
- Manages message history for AI interactions

**`clipboard.rs`** - Image handling
- Processes clipboard content (text/images)
- Handles image encoding for API requests

**`cli.rs`** - Legacy CLI interface
- Simple terminal chat interface
- Single-question mode support

### IDE Module (`src/ide/`)

**`app.rs`** - Main IDE application state
- Manages overall application state
- Coordinates between different panels

**`layout.rs`** - UI layout management
- Defines the VSCode-like layout structure
- Handles panel sizing and positioning

**`editor.rs`** - Multi-tab text editor
- File editing with syntax highlighting
- Tab management and cursor handling

**`statusbar.rs`** - Status information display
- Shows file info, cursor position, mode indicators

**`events.rs`** - Keyboard event handling
- Processes user input and key bindings
- Manages event loop

**`sidebar/`** - Sidebar components
- `chat.rs` - AI chat interface panel
- `file_explorer.rs` - File tree navigation

### Agent Module (`src/agent/`)

**`actions.rs`** - Action parsing and validation
- Parses AI responses into executable actions
- Validates action safety

**`executor.rs`** - File operations execution
- Executes file system operations
- Handles command execution with safety checks

### TUI Module (`src/tui/`)

**`app.rs`** - TUI application state
**`ui.rs`** - Drawing and rendering logic
**`events.rs`** - Event handling for TUI
**`panels/`** - Individual panel implementations
- `chat.rs`, `editor.rs`, `file_explorer.rs`, `status_bar.rs`

## Key Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Terminal control and input handling
- **reqwest** - HTTP client for Groq API
- **tokio** - Async runtime
- **syntect** - Syntax highlighting
- **arboard** - Clipboard access
- **clap** - CLI argument parsing

## Architecture Flow

1. **Entry**: `main.rs` parses CLI args and routes to appropriate mode
2. **IDE Mode**: `ide::run_ide()` sets up terminal and runs the main loop
3. **TUI Loop**: Handles events, draws UI, manages state
4. **Agent Mode**: AI can execute actions through the `agent` module
5. **API**: `api.rs` handles all Groq API communications

## Usage Modes

### 🟢 Normal Mode (Default)
- Safe file operations
- Standard text editing
- Read-only AI assistance

### 🟣 Agentic Mode (Ctrl+A)
- AI can execute file operations
- System command execution
- Code analysis and manipulation
- **Use with caution!**

### 🟡 Insert Mode (i)
- Text insertion in editor
- Character-by-character editing
- Press `Esc` to return to normal

## Supported Models

- `llama-3.1-70b-versatile` (default, best for coding)
- `llama-3.1-8b-instant` (faster, good for simple tasks)
- `mixtral-8x7b-32768` (excellent for complex reasoning)
- `gemma-7b-it` (lightweight option)
- `gemma-9b-it` (balanced performance)

## Safety Features

- **Restricted Paths**: Prevents system directory modifications
- **Command Confirmation**: Agentic mode requires explicit activation
- **Visual Mode Indicators**: Clear indication of current mode
- **Safe Defaults**: Conservative permissions by default

The project follows a modular architecture with clear separation between UI components, agent capabilities, and API interactions.