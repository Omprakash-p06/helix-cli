# Phase 19: TUI Redesign - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning
**Source:** Conversation context (user-provided design specifications)

<domain>
## Phase Boundary

Redesign the Terminal UI (TUI) to support a three-panel layout with advanced input handling, conversation rendering, and sidebar features including context files, tool timeline, command palette, and theming support.

</domain>

<decisions>
## Implementation Decisions

### UI Specification
- **Three-panel layout:**
  - Input area with ghost autocomplete and token counter
  - Conversation area with markdown/code block rendering
  - Sidebar with context files and tool timeline

### Features
- **Command palette** — Quick access to actions
- **Themes** — Visual customization support
- **Ghost autocomplete** — Inline suggestions during typing
- **Token counter** — Character/token count display

### API Contract
- `TuiAction` enum — Actions the TUI can perform
- `TuiEvent` enum — Events emitted by the TUI
- `ChatMessage` type — Message structure in conversation
- `ToolCall` type — Tool invocation structure
- `mpsc` channel communication — Inter-thread messaging

### Technical Design
- **ratatui architecture** — Use ratatui framework
- **State management** — Centralized state handling
- **Event handling** — Input and system events
- **Rendering strategy** — Efficient screen updates
- **Testing approach** — Unit and integration tests

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/STATE.md` — Project state and milestone context
- `.planning/ROADMAP.md` — Phase 19 goal and dependencies
- `.planning/REQUIREMENTS.md` — Active requirements (STREAM, TOOL, CODE)

### Prior Phase Context
- `.planning/phases/17-non-blocking-tool-execution/17-CONTEXT.md` — Phase 19 depends on Phase 17 completion

### Codebase
- `agent-rs/src/tui.rs` — Existing TUI implementation
- `agent-rs/src/main.rs` — Orchestrator entry point

</canonical_refs>

<code_context>
## Existing Code Insights

### Established Patterns
- ratatui framework already in use
- TuiEvent::ToolStart, ToolResult events implemented
- tokio async runtime available
- Existing chat message rendering in terminal mode

### Integration Points
- TUI initialization in `run_llm_loop_tui()`
- Chat history state management
- Tool execution events

</code_context>

<specifics>
## Specific Ideas

- Three-panel layout: sidebar (context files, tool timeline), conversation area, input area
- Input area: ghost autocomplete, token counter
- Conversation area: markdown rendering, code block highlighting
- Command palette: overlay for quick actions
- Themes: configurable color schemes

No additional specifics required — standard approaches acceptable.

</specifics>

<deferred>
## Deferred Ideas

None — phase scope defined by user specifications.

</deferred>

---

*Phase: 19-tui-redesign*
*Context gathered: 2026-04-08*