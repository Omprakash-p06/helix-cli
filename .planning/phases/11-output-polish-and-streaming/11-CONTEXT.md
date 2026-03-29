# Phase 11: Output Polish and Streaming - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Wiring the live token stream from the `llama-server` API into the new `ratatui` UI, and polishing the visual rendering of the agent's reasoning, tool execution states, and asynchronous UI updates.
</domain>

<decisions>
## Implementation Decisions

### Streaming Performance
- **D-01:** Implement a time-based batching stream (e.g., flush every 30ms) over the `mpsc` channel. Avoid raw one-by-one token rendering to prevent UI thread flickering, while maintaining a real-time low-latency feel.
- **D-02:** Use `tokio::time::interval` on the stream task to periodically flush accumulated tokens.

### `<think>` Block Rendering
- **D-03:** Render thoughts inline within the chat log but visually separated using dimmed/greyed text (`ratatui::style::Style::dim()`).
- **D-04:** Extend `ChatEntry` to maintain spans `Vec<(String, Style)>` so that `</think>` tags correctly sequence the active style.
- **D-05:** Implement a hotkey (`Ctrl+T`) to toggle the visibility of think blocks dynamically.

### Tool Execution States
- **D-06:** Insert tool execution states directly into the chat log chronologically as distinct styled entries (e.g., monospaced font, subtle blue background).
- **D-07:** Emit specific events: `TuiEvent::ToolStart(ToolInfo)` showing a spinner/indicator (e.g. "🔧 Executing `read_file`..."), and `TuiEvent::ToolResult(ToolResult)` summarizing the outcome.
- **D-08:** Add support for an interrupt hotkey (`Ctrl+C`) during long-running tool executions.

### the agent's Discretion
- Exact struct definitions for `ToolInfo` and `ToolResult` inside the events.
- The specific visual styling colors (e.g. which ANSI blues or greys) for the tool states.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `.planning/phases/10-terminal-ui-foundation/10-01-PLAN.md` — Defines current `TuiApp`, `ChatEntry`, and `TuiEvent` structures that must be extended.
- `agent-rs/src/tui.rs` — The active TUI implementation to be wired.
- `agent-rs/src/main.rs` — The orchestrator loop that needs to send events to the TUI.

</canonical_refs>

<specifics>
## Specific Ideas
- Extend the `TuiEvent` enum to contain: `TokenChunk(String)`, `ThinkStart`, `ThinkEnd`, `ToolStart(ToolInfo)`, `ToolResult(ToolResult)`.
</specifics>

<deferred>
## Deferred Ideas
None — Scope perfectly aligned.
</deferred>

---

*Phase: 11-output-polish-and-streaming*
*Context gathered: 2026-03-29*
