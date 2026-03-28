# Phase 10: Terminal UI Foundation - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning
**Source:** User's Architecture Suggestion

<domain>
## Phase Boundary

Replace the primitive CLI with a robust Terminal UI (TUI) foundation using `ratatui` and async I/O (`tokio`). Establish the basic layout, input handling, and status bars.

</domain>

<decisions>
## Implementation Decisions

### Input Layer
- Implement ghost autocomplete using `tui-input` or custom widget.
- Support editable multiline input (using custom input handler or spawning `$EDITOR`).
- Implement command preview panel (summary before model execution).
- Slash-command autocomplete (popup triggered on `/`).
- Token/character counter in the status bar (real-time).

### Foundational Stack
- `ratatui` for TUI framework.
- `crossterm` for low-level terminal control.
- `tokio` for async runtime to handle user input non-blockingly.

### Visual Polish
- Welcome banner on startup (version, mode, tip).
- Consistent padding/margins.
- Disable styling if `NO_COLOR` is set.
</decisions>

<canonical_refs>
## Canonical References
No external specs — requirements fully captured in decisions above.
</canonical_refs>
