# 🦀 Rust Coding Agent - VSCode-like TUI IDE

A complete terminal-based IDE built in Rust with beautiful TUI interface, similar to VSCode but running entirely in your terminal. Features AI chat integration, file management, and full editing capabilities.

## ✨ Features

### 🎨 **Complete IDE Interface**
- **VSCode-like Layout**: Resizable sidebar + main editor + status bar
- **File Explorer**: Tree view with folder navigation and file icons
- **Multi-tab Editor**: Open multiple files with tab management
- **AI Chat Panel**: Integrated AI assistant in sidebar
- **Status Bar**: File info, cursor position, mode indicators

### 🤖 **AI Integration**
- **Groq API Support**: All models (llama, mixtral, gemma)
- **Image Support**: Paste images from clipboard (Ctrl+I)
- **Agentic Mode**: AI can execute file operations and commands
- **Context Awareness**: Maintains conversation history

### ⚡ **Developer Experience**
- **Vim-like Navigation**: Familiar keyboard shortcuts
- **Syntax Highlighting**: File type recognition with icons
- **Responsive Layout**: Adapts to terminal size
- **Fast Performance**: Built in Rust for speed

## 🚀 Quick Start

### 1. Build & Configure
```bash
cargo build --release
./target/release/agent config --groq-key YOUR_GROQ_API_KEY
```

### 2. Launch IDE
```bash
./target/release/agent
```

## 🎯 Interface Layout

```
┌────────────────────────────────────────────────────────────────────┐
│ 📁 project/                │ main.rs    lib.rs     [+]              │
│ ├─ 📁 src/                  ├──────────────────────────────────────│
│ │  ├─ 🦀 main.rs            │   1 │ fn main() {                   │
│ │  ├─ 🦀 lib.rs             │   2 │     println!("Hello!");       │
│ │  └─ 🦀 utils.rs           │   3 │ }                             │
│ ├─ ⚙️ Cargo.toml            │   4 │                               │
│ └─ 📄 README.md             │                                      │
├────────────────────────────│                                      │
│ 💬 AI Chat                  │                                      │
│ ┌────────────────────────┐  │                                      │
│ │🧑 How to implement...  │  │                                      │  
│ │🤖 Here's how you can...│  │                                      │
│ └────────────────────────┘  │                                      │
│ > Type your question...     │                                      │
├─────────────────────────────┼──────────────────────────────────────┤
│ NORMAL │ main.rs │ Ln 1, Col 1 │ RUST │ UTF-8 │ 14:30:25         │
└─────────────────────────────────────────────────────────────────────┘
```

## ⌨️ Keyboard Shortcuts

### 🌍 **Global Controls**
| Key | Action |
|-----|--------|
| `F1` / `?` | Toggle help screen |
| `Tab` | Cycle between panels |
| `Ctrl+Q` | Quit application |
| `Ctrl+A` | Toggle agentic mode |

### 📁 **File Explorer (Alt+1)**
| Key | Action |
|-----|--------|
| `↑` `↓` / `j` `k` | Navigate files |
| `Enter` | Open file/expand folder |
| `Space` | Toggle folder expand |
| `Ctrl+R` | Refresh file tree |

### 📝 **Editor (Alt+2)**
| Key | Action |
|-----|--------|
| `i` | Enter insert mode |
| `Esc` | Normal mode |
| `↑` `↓` `←` `→` | Move cursor |
| `Ctrl+S` | Save file |
| `Ctrl+N` | New file |
| `Ctrl+W` | Close file |

### 💬 **AI Chat (Alt+3)**
| Key | Action |
|-----|--------|
| Type + `Enter` | Send message |
| `Ctrl+Enter` | Send message |
| `Ctrl+I` | Send with clipboard image |
| `Ctrl+L` | Clear chat history |

### 📏 **Layout Resizing**
| Key | Action |
|-----|--------|
| `Ctrl+←` `→` | Resize sidebar width |
| `Ctrl+↑` `↓` | Resize chat panel height |

## 🎯 Usage Modes

### 🟢 **Normal Mode (Default)**
- Safe file operations
- Standard text editing
- Read-only AI assistance

### 🟣 **Agentic Mode (Ctrl+A)**
- AI can execute file operations
- System command execution
- Code analysis and manipulation
- **Use with caution!**

### 🟡 **Insert Mode (i)**
- Text insertion in editor
- Character-by-character editing
- Press `Esc` to return to normal

## 🤖 **AI Capabilities**

### Standard Chat
- Code explanations
- Programming questions
- Architecture advice
- Debugging help

### Agentic Mode Features
- File reading/writing
- Directory operations
- Code search and analysis
- System command execution

Example agentic commands:
- "Read the main.rs file and explain the structure"
- "Create a new module for database operations"
- "Find all TODO comments in the project"
- "Run the tests and show me the results"

## 🎛️ **Command Line Options**

```bash
# Launch IDE (default)
./target/release/agent
./target/release/agent tui

# Legacy CLI chat
./target/release/agent chat

# Single question mode
./target/release/agent ask "How do I implement async/await?"
./target/release/agent ask "Analyze this screenshot" --image

# Configuration
./target/release/agent config --groq-key YOUR_KEY
./target/release/agent config --model llama-3.1-70b-versatile
```

## 🔧 **Supported Models**

- `llama-3.1-70b-versatile` (default, best for coding)
- `llama-3.1-8b-instant` (faster, good for simple tasks)
- `mixtral-8x7b-32768` (excellent for complex reasoning)
- `gemma-7b-it` (lightweight option)
- `gemma-9b-it` (balanced performance)

## 🛠️ **Architecture**

### Core Components
```
src/
├── ide/                    # IDE implementation
│   ├── app.rs             # Main IDE application state
│   ├── layout.rs          # UI layout management
│   ├── events.rs          # Keyboard event handling
│   ├── editor.rs          # Multi-tab text editor
│   ├── statusbar.rs       # Status information display
│   └── sidebar/           # Sidebar components
│       ├── file_explorer.rs  # File tree navigation
│       └── chat.rs        # AI chat interface
├── api.rs                 # Groq API client
├── config.rs              # Configuration management
├── conversation.rs        # Chat history
├── clipboard.rs           # Image handling
└── agent/                 # Agentic capabilities
    ├── actions.rs         # Action parsing
    └── executor.rs        # File operations
```

### Key Technologies
- **ratatui**: TUI framework for beautiful interfaces
- **crossterm**: Cross-platform terminal control
- **tokio**: Async runtime for API calls
- **syntect**: Syntax highlighting support
- **arboard**: Clipboard integration

## 🎨 **Customization**

### Color Scheme
- **Cyan**: Active panel borders and highlights
- **Green**: Normal mode indicator
- **Yellow**: Insert mode and input areas
- **Magenta**: Agentic mode indicator
- **Blue**: Secondary highlights
- **Gray**: Inactive elements

### File Icons
- 🦀 Rust files (.rs)
- 🐍 Python files (.py)
- 📜 JavaScript/TypeScript (.js/.ts)
- 🌐 HTML files (.html)
- 🎨 CSS files (.css)
- 📋 JSON files (.json)
- 📄 Markdown files (.md)
- ⚙️ Config files (.toml/.yaml/.yml)

## 🔒 **Safety Features**

- **Restricted Paths**: Prevents system directory modifications
- **Command Confirmation**: Agentic mode requires explicit activation
- **Visual Mode Indicators**: Clear indication of current mode
- **Safe Defaults**: Conservative permissions by default

## 🐛 **Troubleshooting**

### Display Issues
```bash
# Ensure terminal supports colors and Unicode
export TERM=xterm-256color

# Try different terminal emulators:
# - Alacritty (recommended)
# - iTerm2 (macOS)  
# - Windows Terminal (Windows)
# - GNOME Terminal (Linux)
```

### API Issues
```bash
# Check configuration
./target/release/agent config

# Test with simple message
./target/release/agent ask "Hello"

# Verify API key permissions at console.groq.com
```

### Performance
- Large files (>1MB) may be slow to load
- Image processing requires sufficient terminal width
- File tree refresh may take time for large directories

## 🤝 **Contributing**

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Make changes and test thoroughly
4. Add/update documentation
5. Submit pull request

### Development Setup
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repo-url>
cd rust-coding-agent
cargo build

# Run with debug info
RUST_LOG=debug cargo run
```

## 📄 **License**

MIT License - See [LICENSE](LICENSE) file for details.

## 🙏 **Acknowledgments**

- **Groq** for providing fast AI inference
- **Ratatui** community for the excellent TUI framework
- **VSCode** for UI/UX inspiration
- **Rust community** for outstanding tooling and support

---

**Experience the future of terminal-based development! 🚀**

*Built with ❤️ and 🦀 by developers, for developers.*