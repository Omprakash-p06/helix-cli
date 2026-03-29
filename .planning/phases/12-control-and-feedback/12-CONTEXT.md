# Phase 12 Context: Control and Feedback

## Goal
Implement a highly responsive Control and Feedback layer for the Helix Ratatui interface, giving users granular control over autonomous agent execution, chat navigation, and real-time generation metrics.

## UX Decisions

### 1. Cancel Generation vs. Exit App
- **Action**: Map `Ctrl+C` inside `handle_key_event` to a dual-state handler.
- **Logic**: 
  - If `app.is_generating == true` (or actively waiting for a tool), emit `TuiAction::Interrupt`. This breaks the generation loop but retains the app context and partial tokens.
  - If idle, emit `TuiAction::Quit` (exits the app natively like a standard shell).

### 2. Chat History Scrolling
- **Action**: Map pagination keys to alter `app.scroll_offset` when visualizing chat history.
- **Logic**:
  - `PageUp` / `PageDown` strictly control scrolling of the chat buffer regardless of input state.
  - `Up` / `Down` fall back to controlling chat scroll **only** when the user's input buffer is entirely empty. If the buffer is engaged, they perform standard navigation inside the input line.

### 3. TTFT Tracking
- **Action**: Display live animation in the status bar while the engine processes the prompt (prior to streaming the first token).
- **Logic**:
  - When submission occurs, set the status to `Thinking... (0.0s)` updating every 100ms.
  - Upon receiving the first `TokenChunk`, freeze the TTFT metric (e.g., `TTFT: 1.2s`) and display it statically or append it to the generation indicators.

## Architecture Guidelines
- **TuiAction enum**: Extend with `Interrupt` variant.
- **TuiEvent enum**: Extend with `GenerationStarted` (to freeze TTFT timer) and `StatusTick` (if pushing timer updates from orchestrator vs locally in TUI).
- **tokio::select! loop**: Catch `action_rx` within the streaming loop in `main.rs` to break cleanly out of the HTTP stream upon `Interrupt`.
