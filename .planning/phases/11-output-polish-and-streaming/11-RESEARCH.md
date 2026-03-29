# Phase 11: Output Polish and Streaming
## Technical Research

### 1. Wiring the TUI into main.rs
- Currently, `main.rs` uses a blocking `rustyline` loop with a single thread for I/O and server requests.
- The `tui::run_tui()` function spawns the Ratatui event loop on Tokio and returns `(mpsc::UnboundedReceiver<TuiAction>, mpsc::UnboundedSender<TuiEvent>)`.
- `main.rs` needs to be refactored to consume actions (like user submissions) from the `mpsc::UnboundedReceiver` asynchronously via `tokio::select!`.

### 2. Streaming Performance (Batching)
- **Context Decision:** Implement a time-based batching stream (30ms flush) using `tokio::time::interval`.
- **Implementation Strategy:**
  Inside `main.rs`'s `stream.next()` polling loop, introduce a secondary task or loop utilizing `tokio::select!` on a `flush_interval.tick()`. Wait, since `stream.next()` blocks waiting for the next LLM chunk, a purely synchronous buffer accumulation might block the timer. Therefore, it is better to accumulate tokens in a string buffer, and if the buffer is non-empty and 30ms have passed since the last flush, we send `TuiEvent::TokenChunk` and clear it. `tokio::select!` is ideal here to race between "new tokens from llm" and "time to flush buffer to UI".

### 3. `<think>` Block Spans Data Structure
- `ChatEntry` in `tui.rs` is currently defined as `pub struct ChatEntry { pub role: String, pub content: String }`.
- **Context Decision:** Extend `ChatEntry` to maintain `Vec<(String, Style)>` to allow the TUI to easily sequence styles without re-parsing the entire string on every frame.
- **Implementation Strategy:** 
  We should change `ChatEntry` to `pub struct ChatEntry { pub role: String, pub content: String, pub spans: Vec<(String, ratatui::style::Style)> }`. 
  As `TuiEvent::TokenChunk` arrives, we examine the string for `<think>` or `</think>` tags. If we transition into a think state, we start appending characters to a new span with `Style::default().add_modifier(Modifier::DIM)`. When exiting, we revert to `Style::default()`. 

### 4. Tool Execution Events
- **Context Decision:** Insert into chat log as separate entries, styled chronologically.
- **Implementation Strategy:**
  Add variants to `TuiEvent` in `tui.rs`:
  ```rust
  pub struct ToolInfo { pub name: String, pub arguments: String }
  pub struct ToolResult { pub output: String, pub success: bool }

  pub enum TuiEvent {
      TokenChunk(String),
      ThinkStart,
      ThinkEnd,
      ToolStart(ToolInfo),
      ToolResult(ToolResult),
      // ... existing
  }
  ```
  In `main.rs`, right before `tools::execute_*` is called (line 423), emit `TuiEvent::ToolStart(ToolInfo { name: func_name, arguments: args_str })`. After the tool finishes, emit `TuiEvent::ToolResult(ToolResult { output: tool_result.output, success: tool_result.success })`.

## Validation Architecture
- **Dimension 1 (Components):** Unit tests evaluating the parser for `<think>` tag state transitions inside `ChatEntry` span generation.
- **Dimension 2 (State & Side Effects):** Validate the `flush_interval` correctly chunks tokens and fires `TokenChunk`.
- **Dimension 4 (UAT):** Manual verification of visually dimmed `<think>` blocks and chronological tool calls injected into the chat panel. 
