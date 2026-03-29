---
phase: 11-output-polish-and-streaming
plan: 01
subsystem: ui
tags: [ratatui, streaming, tokio, tui, think-blocks]

requires:
  - phase: 10-terminal-ui-foundation
    provides: TuiApp, ChatEntry, TuiEvent, ratatui terminal framework
provides:
  - TokenChunk batched streaming with 30ms flushing
  - ChatSpan-based styled rendering for think blocks
  - ToolStart/ToolResult chronological event display
  - Ctrl+T toggle for think-block visibility
  - TUI mode path in main.rs with full async event loop
affects: [12-conversation-persistence, tui-features]

tech-stack:
  added: []
  patterns: [time-based-batch-flushing, span-based-rich-rendering, tag-parsing-state-machine]

key-files:
  created: []
  modified:
    - agent-rs/src/tui.rs
    - agent-rs/src/main.rs

key-decisions:
  - "Re-parse entire streaming content for spans on each chunk (correct across boundaries vs. incremental)"
  - "TUI mode activated via HELIX_UI_MODE=tui env var, preserving terminal/web modes unchanged"
  - "30ms flush interval via tokio::time::interval racing stream.next() in tokio::select!"

patterns-established:
  - "ChatSpan pattern: Vec<ChatSpan> on ChatEntry for styled rich text rendering"
  - "Tag state-machine parsing: <think>/<\/think> boundaries tracked via in_think_block flag"
  - "TUI event flow: ToolStart before execution, ToolResult after, with auto-StatusBar updates"

requirements-completed: []

duration: 8min
completed: 2026-03-29
---

# Phase 11: Output Polish and Streaming Summary

**Live token streaming with 30ms batch flushing, `<think>` block span rendering with Ctrl+T toggle, and chronological ToolStart/ToolResult event display in ratatui TUI**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-03-29T12:16:43+05:30
- **Completed:** 2026-03-29T12:23:00+05:30
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Extended TuiEvent enum: `TokenChunk(String)`, `ToolStart(ToolInfo)`, `ToolResult(ToolResultInfo)`
- Extended ChatEntry with `Vec<ChatSpan>` for styled inline rendering of think blocks
- Implemented `<think>`/`</think>` tag parser as a state machine in `append_token_chunk()`
- Added `Ctrl+T` hotkey to toggle think-block visibility (`show_thoughts` flag)
- Created `run_llm_loop_tui()` in main.rs with `tokio::select!` racing `stream.next()` and `flush_interval.tick()` at 30ms
- Wired `ToolStart`/`ToolResult` events around every tool execution in the TUI code path

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor TuiEvent, ChatEntry, and ratatui UI rendering** - `6e591a4` (feat)
2. **Task 2: Wire tokio async event loop and batch interval** - `0ca7bfe` (feat)

## Files Created/Modified
- `agent-rs/src/tui.rs` — New structs (ToolInfo, ToolResultInfo, ChatSpan), TuiEvent refactor, think-tag parser, Ctrl+T toggle, span-based rendering in draw_chat_area
- `agent-rs/src/main.rs` — New `run_llm_loop_tui()` function with tokio::select! streaming loop, TUI mode branch, ToolStart/ToolResult emission

## Decisions Made
- Re-parse entire streaming_content on each chunk arrival (simpler, handles tag boundaries across HTTP chunks correctly)
- Kept existing `terminal` and `web` modes untouched; `tui` mode is opt-in via `HELIX_UI_MODE=tui`
- Think-block detection uses DIM modifier + DarkGray foreground to identify spans for show/hide

## Deviations from Plan
None - plan executed as written with minor Rust syntax fix (str indexing requires ranges).

## Issues Encountered
- Rust string indexing with single `usize` doesn't compile — fixed by using `(pos + N)..` range syntax

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- TUI is fully wired for streaming with batch flushing
- Think blocks render inline with toggle support
- Tool executions display chronologically in chat
- Ready for conversation persistence (Phase 12) or further TUI polish

---
*Phase: 11-output-polish-and-streaming*
*Completed: 2026-03-29*
