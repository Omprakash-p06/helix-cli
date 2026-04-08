# 1. Frontend UI Specification (UI Description)

**File:** `TUI_UI_SPEC.md`

## 1.1 Overall Layout

Three‑panel design (configurable via `--layout` flag, default `wide`):

```
┌─────────────────────────────┬─────────────────┐
│ Conversation                │ Context / Tools │
│ (chat area)                 │ (sidebar)       │
│                             │                 │
│ User: Hey!                  │ 📁 Files in ctx │
│                             │   main.rs       │
│ Helix: Hello! How can I     │   config.toml   │
│ help?                       │                 │
│                             │ 🔧 Tool timeline│
│                             │   ✓ read_file   │
│                             │   ⏳ run_cmd    │
├─────────────────────────────┴─────────────────┤
│ Input area (multiline)                         │
│ [Context: 120/8192 tok]  [model: llama3]      │
└────────────────────────────────────────────────┘
```

- **Conversation area** (70% width) – scrollable list of messages.
- **Sidebar** (30% width) – collapsible sections: context files, tool call timeline, session info.
- **Input area** – fixed height, expands with content (max 10 lines).
- **Status bar** – bottom line with model name, token usage, dry‑run mode, connection status, key hints.

## 1.2 Input Area

- Multiline editor with **line numbers** (optional).
- **Ghost autocomplete** – faint text for command completions (e.g., `/help`), accept with `→` or `Tab`.
- **Token counter** – real‑time `current / max` (e.g., `45/8192 tok`) with a small bar graph (10 chars wide) showing usage.
- **Key bindings**:
  - `Enter` – newline.
  - `Alt+Enter` – submit prompt.
  - `Ctrl+E` – open external editor (`$EDITOR`) for long prompts.
  - `Up` / `Down` – navigate command history (when input empty → scroll conversation).
- **Slash commands** – type `/` to open a fuzzy‑searchable command palette (with preview of each command).

## 1.3 Conversation Area

- **Message bubbles** – user messages right‑aligned (light background), assistant left‑aligned (default background). Colours configurable via theme.
- **Markdown rendering**:
  - Headings, bold, italics, lists, blockquotes.
  - Fenced code blocks with syntax highlighting (using `syntect`).
  - Inline diffs (green/red lines) inside code blocks when the output is a patch.
- **Tool call entries** – appear as **collapsible blocks** inside assistant messages:
  ```
  ▶ read_file("src/main.rs")   [45ms]
    (expanded shows output or result)
  ```
  - Expand/collapse with `Enter` on the line.
  - Show execution time, status icon (✓ / ✗ / ⏳).
- **Thinking traces** – in agentic mode only, shown inside a dimmed `<thinking>` block that can be collapsed.
- **Scrolling** – smooth scrolling with `PageUp` / `PageDown` and `Ctrl+u` / `Ctrl+d`. Maintain scroll position when new messages arrive (unless user is scrolled up, then show a “New messages” indicator).

## 1.4 Sidebar (Right Panel)

Sections (toggleable with `Ctrl+B`):

- **Context Files** – list of files currently in the LLM’s context window (pinned or auto‑loaded). Shows file names and token counts. Clickable (or `Enter`) to open a preview.
- **Tool Timeline** – horizontal bar or vertical list of recent tool calls, grouped by turn. Shows name, duration, status. Click to jump to that point in conversation.
- **Session Info** – model name, total tokens used, session duration, connection status (green dot for connected).

## 1.5 Command Palette

- Triggered by `/` at beginning of input line.
- Fuzzy search over all available slash commands (`/help`, `/clear`, `/model`, `/agent`, `/chat`, `/undo`, `/save`, `/load`, `/export`, `/theme`, `/layout`).
- Each command shows a short description and example.
- Selected command inserts into input line or executes immediately.

## 1.6 Animations & Feedback

- **Typing indicator** – three bouncing dots (`. . .`) when the model is generating but no tokens yet (TTFT phase).
- **Spinner** – a rotating bar for long‑running operations (tool execution, model loading).
- **Fade‑in** – new messages appear with a slight delay (50ms) and a transition (if terminal supports it, else just instant).
- **Progress steps** – when a multi‑step plan is executed (e.g., agentic mode), show `[1/3] Reading file...` inline, then updates.
- **Error notifications** – non‑modal, appear as a red banner at the top of the conversation area, with a retry button (click or `r`).

## 1.7 Themes & Customisation

- Default dark theme (high contrast).
- Light theme (optional).
- User can define custom themes via `~/.config/helix-agent/themes/my_theme.toml`:
  ```toml
  [colors]
  user_message_bg = "#2A2A2A"
  assistant_message_bg = "#1E1E1E"
  accent = "#00FFCC"
  error = "#FF5555"
  ```
- Support `NO_COLOR` environment variable for monochrome output.

---

# 2. API Documentation (Frontend ↔ Backend Contract)

**File:** `TUI_API_CONTRACT.md`

## 2.1 Communication Model

- The TUI runs in the main thread, the backend (orchestrator + LLM) runs in a separate `tokio` task or subprocess.
- Two unidirectional channels:
  - `TuiAction` (UI → backend)
  - `TuiEvent` (backend → UI)

## 2.2 Shared Types (in `agent_core` crate)

```rust
// agent_core/src/lib.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: u64,
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: Option<String>,
    pub status: ToolStatus,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMode {
    Chat,      // concise, no thinking
    Agentic,   // tools allowed, may show reasoning
}
```

## 2.3 UI → Backend Actions (`TuiAction`)

```rust
pub enum TuiAction {
    SubmitPrompt {
        content: String,
        mode: ChatMode,          // user can override with /agent or /chat
    },
    CancelGeneration,
    SetMode(ChatMode),
    RunSlashCommand {
        command: String,
        args: Vec<String>,
    },
    PinFile(String),
    UnpinFile(String),
    UndoLastAction,
    ClearHistory,               // both UI and LLM context
    // Session management
    SaveSession(String),
    LoadSession(String),
    // Layout / UI
    SetTheme(String),
    ToggleSidebar,
    // Tool control
    ConfirmTool(String, bool),  // for dangerous tools
}
```

## 2.4 Backend → UI Events (`TuiEvent`)

```rust
pub enum TuiEvent {
    // Streaming
    TokenChunk(String),
    GenerationComplete(ChatMessage),   // final message
    GenerationInterrupted(String),     // partial output

    // Tool lifecycle
    ToolCallStart {
        id: u64,
        name: String,
        args: serde_json::Value,
    },
    ToolCallProgress {
        id: u64,
        message: String,
    },
    ToolCallEnd {
        id: u64,
        result: String,
        status: ToolStatus,
        duration_ms: u64,
    },

    // Context updates
    ContextUpdate {
        tokens_used: usize,
        max_tokens: usize,
        files: Vec<String>,
    },

    // Mode changes
    ModeChanged(ChatMode),

    // System events
    Error(String),
    Warning(String),
    Info(String),

    // Session events
    SessionSaved(String),
    SessionLoaded(String),
}
```

## 2.5 Transport

- **In‑process**: Use `tokio::sync::mpsc::unbounded_channel` for both directions.
- **Out‑of‑process** (if backend is separate binary): Use JSON‑RPC over stdin/stdout or a local socket. The TUI spawns the backend as a child process.

## 2.6 Example Flow

1. User types `/agent` → UI sends `TuiAction::SetMode(Agentic)`.
2. User submits `"read main.rs"` → `TuiAction::SubmitPrompt { content: "read main.rs", mode: Agentic }`.
3. Backend starts generation. Sends `TuiEvent::TokenChunk("I'll read")`, then `TuiEvent::ToolCallStart { id: 1, name: "read_file", args: {"path":"main.rs"} }`.
4. While tool runs, backend sends `TuiEvent::ToolCallProgress { id: 1, message: "reading..." }`.
5. Tool completes → `TuiEvent::ToolCallEnd { id: 1, result: "fn main() {...}", status: Completed, duration_ms: 45 }`.
6. Backend continues streaming with tool result injected.
7. Finally `TuiEvent::GenerationComplete(message)`.

---

# 3. Technical Design Document (Data Mapping & State Logic)

**File:** `TUI_TECH_DESIGN.md`

## 3.1 Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    TUI Process                      │
│  ┌─────────────┐   ┌─────────────┐   ┌───────────┐ │
│  │ crossterm   │   │   ratatui   │   │   Tokio   │ │
│  │ event loop  │◄─►│   render    │   │  runtime  │ │
│  └─────────────┘   └─────────────┘   └─────┬─────┘ │
│         │                                   │       │
│         ▼                                   ▼       │
│  ┌─────────────────────────────────────────────┐   │
│  │           AppState (shared Arc<Mutex>)      │   │
│  │  - conversation: Vec<ChatMessage>           │   │
│  │  - tool_calls: HashMap<u64, ToolCall>       │   │
│  │  - scroll_offset, input_buffer, mode, etc. │   │
│  └─────────────────────────────────────────────┘   │
│         ▲                                   │       │
│         │                                   ▼       │
│         │                          ┌─────────────┐  │
│         └──────────────────────────│ mpsc channel│  │
│                                    │ (TuiEvent)  │  │
│                                    └──────┬──────┘  │
└───────────────────────────────────────────┼─────────┘
                                            │
                                    (spawns child)
                                            ▼
                              ┌─────────────────────────┐
                              │   Backend Orchestrator  │
                              │   (LLM, tools, etc.)    │
                              └─────────────────────────┘
```

## 3.2 Data Structures (State)

```rust
// tui.rs

pub struct AppState {
    // Conversation
    pub messages: Vec<ChatMessage>,
    pub current_assistant_response: String,   // streaming buffer
    pub current_tool_calls: HashMap<u64, ToolCall>,

    // UI state
    pub scroll_offset: u16,
    pub input_buffer: String,
    pub input_history: Vec<String>,
    pub history_index: usize,
    pub mode: ChatMode,
    pub sidebar_visible: bool,
    pub theme: Theme,

    // Context tracking
    pub context_tokens: usize,
    pub context_max: usize,
    pub context_files: Vec<String>,

    // Backend connection
    pub backend_tx: mpsc::UnboundedSender<TuiAction>,
    pub backend_handle: Option<JoinHandle<()>>,
}
```

## 3.3 State Logic (Event Handling)

**Main loop** (pseudocode):

```rust
loop {
    // 1. Handle crossterm events (key presses, resize)
    let crossterm_event = event::read()?;
    let action = match crossterm_event {
        KeyEvent { code: Char('/'), modifiers: NONE } if input_buffer.is_empty() => {
            open_command_palette(&mut app_state);
            None
        }
        KeyEvent { code: Enter, modifiers: ALT } => {
            Some(TuiAction::SubmitPrompt {
                content: app_state.input_buffer.clone(),
                mode: app_state.mode,
            })
        }
        KeyEvent { code: Char('c'), modifiers: CTRL } if is_generating => {
            Some(TuiAction::CancelGeneration)
        }
        // ... other mappings
        _ => handle_input_editing(&mut app_state, crossterm_event),
    };

    if let Some(action) = action {
        app_state.backend_tx.send(action)?;
    }

    // 2. Poll backend events (non‑blocking)
    while let Ok(event) = app_state.backend_rx.try_recv() {
        apply_event(&mut app_state, event);
    }

    // 3. Render
    terminal.draw(|f| ui(f, &app_state))?;
}
```

**Event application** (`apply_event`):

```rust
fn apply_event(state: &mut AppState, event: TuiEvent) {
    match event {
        TuiEvent::TokenChunk(chunk) => {
            state.current_assistant_response.push_str(&chunk);
            // If user is not scrolled up, auto‑scroll to bottom.
            if state.scroll_offset == 0 {
                state.scroll_offset = calculate_max_scroll(state);
            }
        }
        TuiEvent::GenerationComplete(msg) => {
            state.messages.push(msg);
            state.current_assistant_response.clear();
        }
        TuiEvent::ToolCallStart { id, name, args } => {
            let tool = ToolCall { id, name, args, status: Running, .. };
            state.current_tool_calls.insert(id, tool);
        }
        TuiEvent::ToolCallEnd { id, result, status, duration_ms } => {
            if let Some(tool) = state.current_tool_calls.get_mut(&id) {
                tool.result = Some(result);
                tool.status = status;
                tool.duration_ms = Some(duration_ms);
            }
        }
        TuiEvent::ContextUpdate { tokens_used, max_tokens, files } => {
            state.context_tokens = tokens_used;
            state.context_max = max_tokens;
            state.context_files = files;
        }
        // ...
    }
}
```

## 3.4 Rendering (Widget Composition)

- Use `ratatui::layout::Layout` to split into three areas.
- **Conversation area**: `List` widget where each item is a `ChatMessage`. Each message can be a `Paragraph` with styled spans (using `Text`).
- **Tool calls in sidebar**: `Table` with columns: icon, name, duration, status.
- **Input area**: Custom `InputWidget` that handles cursor movement, line wrapping, and ghost text.
- **Command palette**: Overlay `Popup` (using `Clear` background and `Block` with border).

## 3.5 Performance Optimisations

- **Redraw throttling**: For very fast token streams, redraw at most every 16ms (60 FPS) using a `tokio::time::interval`.
- **Partial rendering**: Use `ratatui`’s `Buffer` and only update changed areas (though ratatui does this automatically if you re‑draw the whole frame efficiently).
- **Off‑screen buffering**: Keep a `Vec<Line>` of the rendered conversation to avoid re‑computing markdown for old messages on every redraw.
- **Asynchronous syntax highlighting**: Use `syntect` in a background thread, cache results.

## 3.6 Testing Strategy

- **Unit tests** for state transitions (`apply_event`).
- **Snapshot tests** for rendering using `ratatui::TestBackend`.
- **Integration tests** with a mock backend that sends predefined `TuiEvent` sequences and verifies UI output.
- **End‑to‑end** with a real tiny model (e.g., `TinyLlama`) and scripted user inputs.

## 3.7 Implementation Phases (TUI focus)

| Phase | Deliverable |
|-------|--------------|
| 1 | Three‑panel layout, basic message rendering. |
| 2 | Input area with ghost autocomplete and token counter. |
| 3 | Command palette (`/` fuzzy search). |
| 4 | Tool call timeline in sidebar. |
| 5 | Animations (spinner, typing indicator, fade‑in). |
| 6 | Diff rendering and markdown improvements. |
| 7 | Theming and configuration. |
| 8 | Performance tuning and testing. |

---

**Goal:** Create a terminal user interface (TUI) for the Helix Agent that is as polished as `opencode` (or better), with three panels, live token streaming, tool call timeline, command palette, animations, and a professional look.

**Current state:** The TUI is very basic – just a chat log, no sidebars, no token counter, no animations. We need to rebuild it using Rust + `ratatui` + `crossterm` + `tokio`.

**Deliverables:**

1. **Design documents** (placed in `misc/design/`):
   - `UI_SPECIFICATION.md` – detailed visual and behavioural spec.
   - `API_CONTRACT.md` – events and data structures between TUI and backend.
   - `TECHNICAL_DESIGN.md` – state management, rendering, performance.

2. **Implementation** (modify existing `src/tui.rs`, `src/tui/api.rs`, `src/main.rs`):
   - A working three‑panel TUI with all features described below.
   - No unused imports, no layout overlaps, no dead code.

---

## 1. Features to Emulate from `opencode`

- **Multiline input** with line numbers, ghost autocomplete, token counter.
- **Markdown rendering** (headers, lists, code blocks with syntax highlighting).
- **Tool calls** as collapsible blocks inside assistant messages (show name, args, result, duration).
- **Spinner / typing indicator** while waiting for first token.
- **Command palette** triggered by `/` – fuzzy search over slash commands.
- **Smooth scrolling** (PageUp/Down, Ctrl+U/Ctrl+D).
- **Copy/paste** – `y` to copy a code block, mouse selection works.
- **Status bar** at bottom: model name, token usage, connection status, key hints.

---

## 2. Unique Enhancements (Beyond opencode)

- **Three‑panel layout**:
  - Left (70%): conversation area.
  - Right (30%): sidebar with two tabs: "Context Files" and "Tool Timeline".
- **Real‑time token counter** inside the input line (e.g., `[45/8192 tok]`) plus a small bar graph (10 chars).
- **Tool timeline** – horizontal bar or vertical list showing recent tool calls with icons, durations, and status (✓ / ✗ / ⏳).
- **Inline diff view** – when a tool returns a diff, render it with green/red lines directly in chat.
- **Vim mode** (optional, can be toggled) – use `h/j/k/l` for navigation.
- **Custom themes** – load from `~/.config/helix-agent/theme.toml` (dark by default, light optional).
- **Session tabs** – `Ctrl+Tab` to switch between named sessions (bonus, can be added later).

---

## 3. Detailed UI Specification

### 3.1 Colors (Dark Theme Default)

| Element | Color |
|---------|-------|
| Background | `#0A0E14` |
| User message bubble | `#2A2F3A` background, `#FFFFFF` text |
| Assistant message bubble | `#1A1E26` background, `#D4D4D4` text |
| Accent (highlight, commands) | `#00FFCC` (cyan) |
| Error | `#FF5555` |
| Warning | `#FFA500` |
| Success | `#55FF55` |
| Tool call – running | `#FFD700` (yellow) |
| Tool call – completed | `#55FF55` |
| Tool call – failed | `#FF5555` |
| Dimmed text (ghost, timestamps) | `#6A6F78` |

### 3.2 Layout (ASCII representation)

```
┌─────────────────────────────────────────────┬───────────────────┐
│  Conversation Area (scrollable)             │ Sidebar           │
│                                             │ ┌───────────────┐ │
│  [User] Hey!                                │ │ Context Files │ │
│                                             │ │  main.rs      │ │
│  [Helix] Hello! How can I help?             │ │  config.toml  │ │
│                                             │ └───────────────┘ │
│  ...                                        │ ┌───────────────┐ │
│                                             │ │ Tool Timeline │ │
│                                             │ │ ✓ read_file   │ │
│                                             │ │ ⏳ run_cmd    │ │
│                                             │ └───────────────┘ │
├─────────────────────────────────────────────┴───────────────────┤
│ Input area (multiline)                                           │
│ > [45/8192 tok] ████░░░░░░  |  model: llama3  |  chat mode     │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

- The sidebar is collapsible (toggle with `Ctrl+B`).
- The input area shows token usage as a bar graph (10 chars) and numeric counter.
- Status bar at very bottom: `Ready | Ctrl+C stop | / commands`.

### 3.3 Animations & Feedback

- **Typing indicator** – three bouncing dots (`. . .`) in the assistant area before first token.
- **Spinner** – rotating bar (`◐ ◓ ◑ ◒`) during tool execution or long operations.
- **Fade‑in** – new messages appear with a 50ms delay and a smooth transition (if terminal supports it; otherwise instant).
- **Progress steps** – when the agent runs a multi‑step plan, show `[1/3] Reading file...` inline.
- **Error banner** – red bar at the top of conversation area, dismissible with `Esc`.

### 3.4 Key Bindings

| Action | Keys |
|--------|------|
| Submit prompt | `Alt+Enter` |
| Newline in input | `Enter` |
| Cancel generation | `Ctrl+C` (only when generating) |
| Open command palette | `/` (at start of input) |
| Toggle sidebar | `Ctrl+B` |
| Scroll up/down | `PageUp` / `PageDown`, `Ctrl+U` / `Ctrl+D` |
| Copy selected text | `Ctrl+C` or `y` (when a code block is focused) |
| Switch chat mode | `Ctrl+M` (cycle Chat/Agentic) |
| Quit app | `Ctrl+C` twice (or when idle) |
| Vim mode navigation (if enabled) | `j` / `k` (scroll), `gg` / `G` (top/bottom) |

### 3.5 Command Palette

- Triggered by typing `/` at the beginning of the input line.
- Fuzzy search over commands: `/help`, `/clear`, `/model`, `/agent`, `/chat`, `/undo`, `/save`, `/load`, `/export`, `/theme`, `/layout`, `/vim`.
- Shows a short description and example for each.
- Selected command either executes immediately (if no arguments) or inserts into input line.

---

## 4. API Contract (Backend ↔ TUI)

- **Communication:** `tokio::sync::mpsc::unbounded_channel`.
- **TUI sends `TuiAction`** (defined in `src/tui/api.rs`).
- **Backend sends `TuiEvent`** (same file).
- Data structures: `ChatMessage`, `ToolCall`, `ToolStatus`, `ChatMode` – all in `agent_core` crate.

See the earlier `API_CONTRACT.md` for exact enums. Ensure no unused imports.

---

## 5. Technical Design Summary

- **State:** `AppState` struct holding messages, input buffer, scroll offset, tool calls, sidebar visibility, theme, etc.
- **Event loop:** `crossterm` events → map to `TuiAction` → send to backend; receive `TuiEvent` → update `AppState` → redraw.
- **Rendering:** `ratatui` with custom widgets for input area, conversation list, sidebar, command palette popup.
- **Redraw throttle:** Use `tokio::time::interval` to cap redraws at 60 FPS if token stream is very fast.
- **Testing:** Unit tests for state transitions; snapshot tests for rendering using `TestBackend`.

---

## 6. Implementation Steps (Priority Order)

1. **Fix layout overlapping** – rewrite `tui.rs` with proper `Layout` constraints.
2. **Implement three‑panel layout** – conversation, sidebar, input, status bar.
3. **Add input area** with token counter and bar graph (use `ratatui::widgets::Gauge`).
4. **Add command palette** (popup with fuzzy search using `skim` or a custom list).
5. **Add tool call timeline** in sidebar – show recent tool calls with icons and durations.
6. **Add animations** – spinner, typing indicator.
7. **Add markdown rendering** – use `ratatui::text::Text` with spans, or integrate `comrak` + `syntect`.
8. **Add diff rendering** – color lines starting with `+` green, `-` red.
9. **Add theming** – load from config file.
10. **Add vim mode** (optional, can be later).

---

## 7. Files to Create/Modify

- `misc/design/UI_SPECIFICATION.md` – copy the spec from this prompt.
- `misc/design/API_CONTRACT.md` – write the enums and channel definitions.
- `misc/design/TECHNICAL_DESIGN.md` – state machine, event loop, performance notes.
- `src/tui.rs` – main UI logic.
- `src/tui/api.rs` – `TuiAction` and `TuiEvent` definitions.
- `src/tui/widgets/input.rs` – custom input widget.
- `src/tui/widgets/sidebar.rs` – context and tool timeline.
- `src/tui/widgets/command_palette.rs` – popup.
- `src/main.rs` – spawn TUI and backend tasks.

---

## 8. Acceptance Criteria

- [ ] The TUI shows three panels with correct alignment.
- [ ] No overlapping text or stray characters.
- [ ] Input area has token counter and bar graph updating in real time.
- [ ] Typing `/` opens a fuzzy command palette.
- [ ] Tool calls appear in sidebar with icons and durations.
- [ ] Markdown and diffs render correctly.
- [ ] `Ctrl+C` cancels generation gracefully.
- [ ] All unused imports removed, `cargo clippy` passes.
- [ ] The TUI feels responsive and smooth (no lag on token stream).
