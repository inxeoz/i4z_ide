# Rust Coding Agent - Terminal IDE Project

## 📖 Project Overview

This repository implements a **Rust‑based terminal IDE** that feels like a lightweight VS Code inside your terminal. It provides a full‑screen TUI built with **ratatui** and **crossterm**, offering:

* A **file explorer** with a tree view, icons, and folder navigation.
* A **multi‑tab editor** with line numbers, cursor handling, and syntax‑type icons.
* An **AI chat panel** powered by the **Groq** LLM API (text + optional clipboard image).
* **Agentic mode** – the AI can safely execute file‑system actions and system commands when explicitly enabled.
* Vim‑style navigation, resizable panels, a status bar, notifications, and mouse support.

The code is organized into clear modules: core CLI, configuration, API client, conversation/history, clipboard handling, the IDE UI, an optional lightweight TUI, and the agent that interprets AI‑generated actions.

---

## ✨ Key Features

| Feature | Description |
|--------|-------------|
| **VSCode‑like layout** | Resizable sidebar (file explorer + chat + notifications) + main editor area + status bar. |
| **File Explorer** | Tree view with folder expand/collapse, file icons, hidden‑file filtering, and CRUD operations (create, rename, delete). |
| **Multi‑tab Editor** | Open many files, switch tabs, close tabs, drag‑reorder tabs, show modified indicator, line numbers. |
| **AI Chat Panel** | Integrated Groq chat; `Ctrl+Enter` sends a message, `Ctrl+I` attaches a clipboard image. |
| **Agentic Mode** (`Ctrl+A`) | AI can execute file operations (create, delete, rename) and run system commands safely. |
| **Vim‑like navigation** | `h/j/k/l`, `i` for insert mode, `Esc` for normal mode, `Tab` to cycle panels, etc. |
| **Mouse support** | Click to focus panels, open files, select tabs, drag tabs, view notifications. |
| **Status Bar** | Shows mode, focused panel, current file, cursor position, modification flag, total open files, time, encoding. |
| **Notifications** | Real‑time feedback for mouse hover/click, file ops, info messages. |
| **Safety** | Restricted paths, explicit activation of agentic mode, visual mode indicators, conservative defaults. |

---

## 📂 Project Structure (high‑level)

```
src/
├─ api.rs                # Groq API client (HTTP, auth, model selection)
├─ cli.rs                # Legacy single‑question chat interface
├─ config.rs             # Load/save user settings (API keys, models)
├─ clipboard.rs          # Clipboard manager – text & image → base64
├─ conversation.rs       # Chat history storage for LLM context
├─ ide/                  # Full‑featured IDE UI
│   ├─ app.rs            # Central IdeApp state & business logic
│   ├─ layout.rs         # UI composition, overlays, tab drawing
│   ├─ editor.rs         # Multi‑tab editor, file I/O, cursor handling
│   ├─ statusbar.rs      # Bottom status line rendering
│   ├─ events.rs         # Keyboard/mouse → IdeEvent conversion
│   └─ sidebar/
│        ├─ chat.rs          # AI chat buffer, input handling, rendering
│        ├─ file_explorer.rs # File tree navigation, CRUD ops
│        └─ notifications.rs # Notification panel rendering
├─ agent/                # Agentic capabilities
│   ├─ actions.rs        # Parse AI responses into actions, safety checks
│   └─ executor.rs       # Execute file‑system actions, run commands
├─ tui/                  # Lightweight alternative UI (panels only)
│   ├─ app.rs
│   ├─ ui.rs
│   ├─ events.rs
│   └─ panels/…
└─ main.rs               # CLI entry point, dispatches to IDE/TUI/Chat/Config/Ask
```

### Core Modules (`src/`)

| File | Role |
|------|------|
| **`main.rs`** | Parses CLI (`clap`) → routes to IDE (`ide::run_ide`), legacy chat, config commands, or single‑question mode. |
| **`api.rs`** | `GroqClient` – async HTTP wrapper for Groq LLM, builds text & image messages. |
| **`config.rs`** | Loads `~/.config/agent/config.toml`, stores API key & default model, provides getters (`get_groq_key`, `get_model`). |
| **`conversation.rs`** | Holds a `Vec<GroqMessage>` that is sent with each request, preserving context. |
| **`clipboard.rs`** | `ClipboardManager` – reads system clipboard, encodes images to base64 for the API. |
| **`cli.rs`** | Simple terminal chat interface (legacy, not the full IDE). |

### IDE Module (`src/ide/`)

| File | What it does |
|------|--------------|
| **`app.rs`** | Central `IdeApp` struct – stores config, Groq client, conversation, clipboard, UI panels (`sidebar`, `editor`, `statusbar`), layout sizes, mode/focus state, dialogs, mouse tracking, notifications, tab‑drag state, session ID, current directory. Provides methods to mutate state, handle events, send chat messages, and expose status info. |
| **`layout.rs`** | Draws the whole UI: decides which overlay (help, API config, dialogs) to show, splits the screen into sidebar & main area, draws file explorer, notifications, chat, editor tabs, editor content, status bar, and welcome screen. Contains helpers for centered rectangles and tab‑click hit‑testing. |
| **`editor.rs`** | `EditorTab` (file path, name, content, lines, cursor, scroll, modified flag, unique ID) and `Editor` (vector of tabs, active index). Handles file I/O, text editing primitives, cursor movement, tab management (new, open, close, reorder, switch), and rendering of the editor pane with line numbers and syntax‑type icons. |
| **`statusbar.rs`** | `StatusInfo` DTO + `StatusBar::draw` – builds left/right spans (mode, panel, file info, tab count, encoding, file type, clock) and renders a full‑width status line. |
| **`events.rs`** | `IdeEvent` enum (all possible user actions) and `EventHandler` that polls `crossterm` events, translates them into `IdeEvent`s (keyboard shortcuts, mouse clicks, scrolls, resize). |
| **`sidebar/chat.rs`** | Chat buffer (`ChatMessage` with type & timestamp). Methods to add user/assistant/system messages, scroll, edit input, clear, and draw the chat area (messages + input box). |
| **`sidebar/file_explorer.rs`** | `FileNode` tree (path, name, dir flag, expanded flag, depth, children). Builds a flat list for rendering, supports navigation, expand/collapse, create/delete/rename files & folders, and draws the explorer with icons. |
| **`sidebar/notifications.rs`** | Simple panel that shows recent notifications (mouse hover/click, file ops, info) with icons, colors, and relative timestamps. |

### Agent Module (`src/agent/`)

| File | Role |
|------|------|
| **`actions.rs`** | Parses AI‑generated responses into structured actions, validates safety (e.g., path restrictions). |
| **`executor.rs`** | Executes the validated actions: file creation, deletion, renaming, running system commands, etc. Integrated with `IdeApp` when `AppMode::Agentic` is active. |

### TUI Module (`src/tui/`)

A lighter UI alternative (not the main VSCode‑like layout). Contains its own `app.rs`, `ui.rs`, `events.rs`, and panel implementations (`chat.rs`, `editor.rs`, `file_explorer.rs`, `status_bar.rs`). Used when the binary is invoked in a different mode.

---

## 🔧 Core Flow (runtime)

1. **Entry** – `main.rs` parses CLI arguments.
2. **IDE start** – `ide::run_ide(config)` sets up raw mode, alternate screen, mouse capture.
3. **App state** – `IdeApp::new(config)` creates sidebar (`FileExplorer`, `Chat`, `NotificationPanel`), editor (empty), status bar, and loads the current directory.
4. **Main loop** (`run_ide_loop`):
   * `terminal.draw(|f| layout::draw_ide(f, &mut app))` → UI rendered.
   * `event_handler.poll_event()` → returns an `IdeEvent` (or `None`).
   * `app.handle_event(event)` updates the model (open file, move cursor, send chat, resize panels, etc.).
   * Loop repeats until `app.should_quit()` becomes `true`.
5. **Chat** – When the user presses **Enter** in the chat panel, `IdeApp::send_chat_message` builds a Groq request (text or image), sends it, shows a typing indicator, receives the response, and appends it to the chat buffer.
6. **Agentic mode** – Toggled with **Ctrl+A**. In this mode the AI can issue file‑system actions; those are parsed by `agent::actions` and executed by `agent::executor`, with notifications shown in the sidebar.

---

## 🎮 Usage Modes

| Mode | Activation | Behaviour |
|------|------------|-----------|
| **Normal** (default) | – | Safe editing, read‑only AI assistance, no automatic file ops. |
| **Agentic** | `Ctrl+A` | AI may execute file operations & system commands (requires explicit activation). |
| **Insert** | `i` (in editor) | Direct text insertion; `Esc` returns to Normal mode. |
| **Command Help** | `Ctrl+H` | Shows a modal with all key bindings. |
| **API Config** | `Ctrl+,` | Shows a modal with current Groq configuration. |
| **Help Overlay** | `F1` / `?` | General help overlay. |

---

## 🤖 Supported Groq Models

* `llama-3.1-70b-versatile` – default, best for coding.
* `llama-3.1-8b-instant` – faster, lighter.
* `mixtral-8x7b-32768` – strong reasoning.
* `gemma-7b-it` – lightweight.
* `gemma-9b-it` – balanced performance.

---

## 🛡️ Safety Features

* **Restricted Paths** – Agentic actions cannot touch system directories.
* **Explicit Activation** – Agentic mode must be toggled (`Ctrl+A`).
* **Visual Mode Indicators** – Status bar shows `NORMAL`, `INSERT`, or `AGENTIC`.
* **Conservative Defaults** – Permissions and file operations are safe by default.

---

## 📦 Key Dependencies

| Crate | Purpose |
|-------|---------|
| **ratatui** | Terminal UI framework (widgets, layout, styling). |
| **crossterm** | Low‑level terminal control, raw mode, mouse events. |
| **reqwest** | HTTP client for Groq API calls. |
| **tokio** | Async runtime for network I/O. |
| **syntect** | (planned) syntax highlighting for editor content. |
| **arboard** | Clipboard access (text & images). |
| **clap** | CLI argument parsing. |

---

## 📚 Quick Reference – “Which file does what?”

| Feature | Primary source file(s) |
|---------|------------------------|
| **CLI entry & command routing** | `src/main.rs` |
| **Groq API client** | `src/api.rs` |
| **Configuration handling** | `src/config.rs` |
| **Conversation/history** | `src/conversation.rs` |
| **Clipboard (image) handling** | `src/clipboard.rs` |
| **Legacy chat interface** | `src/cli.rs` |
| **IDE bootstrap & main loop** | `src/ide/mod.rs` |
| **Overall IDE state & business logic** | `src/ide/app.rs` |
| **Event polling & conversion** | `src/ide/events.rs` |
| **UI layout, overlays, tab bar** | `src/ide/layout.rs` |
| **File explorer tree & CRUD** | `src/ide/sidebar/file_explorer.rs` |
| **Chat panel & message handling** | `src/ide/sidebar/chat.rs` |
| **Notifications panel** | `src/ide/sidebar/notifications.rs` |
| **Multi‑tab editor, cursor, file I/O** | `src/ide/editor.rs` |
| **Status bar rendering** | `src/ide/statusbar.rs` |
| **Agentic action parsing** | `src/agent/actions.rs` |
| **Agentic action execution** | `src/agent/executor.rs` |
| **Alternative lightweight TUI** | `src/tui/*` |

---

## 🏁 Summary

The **Rust Coding Agent** delivers a powerful, AI‑enhanced coding environment that lives entirely inside your terminal. Its architecture cleanly separates concerns:

* **Core utilities** (`api`, `config`, `clipboard`, `conversation`) handle external interactions.
* **IDE module** (`app`, `layout`, `editor`, `statusbar`, `events`, `sidebar/*`) manages UI state, rendering, and user input.
* **Agent module** gives the AI safe, controllable power to manipulate the file system.

All components are written in safe, idiomatic Rust, leveraging async I/O (`tokio`) for LLM calls and a high‑performance TUI (`ratatui`). The result is a responsive, extensible, and secure coding environment that runs completely in the terminal.


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
