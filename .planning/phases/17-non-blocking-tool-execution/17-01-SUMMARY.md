---
plan: 17-01
phase: 17-non-blocking-tool-execution
status: complete
completed: "2026-04-06T18:05:00Z"
---

# Plan 17-01: Async Tool Execution

## Objective
Convert synchronous tool execution to async spawning without blocking the orchestrator loop. Implement status feedback display in chat area.

## What Was Built
- **Async wrapper functions**: `execute_tool_async` with `tokio::task::spawn_blocking` and 30s timeout per tool
- **TUI LLM loop integration**: Replaced sync `for tc in tool_calls` with `join_all` concurrent execution, `TuiEvent::ToolStart`/`ToolResult` events emitted
- **Terminal mode integration**: Same async pattern applied to non-TUI execution path (no TUI events, but same concurrency model)

## Key Files Modified
- `agent-rs/src/main.rs`: Async execution wrapper, TUI loop integration, terminal mode integration

## Commits
1. `feat(17-01): add async tool execution wrapper functions`
2. `feat(17-01): integrate async tool execution into TUI LLM loop`
3. `feat(17-01): apply async tool execution to terminal mode`

## Requirements Met
- TOOL-01: Tools spawn as async tasks
- TOOL-02: Tool status displayed via TuiEvent::ToolStart
- TOOL-03: Tool results injected as ChatMessage with role "tool"
- TOOL-05: 30s timeout enforced per tool

## Self-Check: PASSED
- `spawn_blocking` used for blocking tool execution
- `tokio::time::timeout` with 30s Duration
- `TuiEvent::ToolStart` and `TuiEvent::ToolResult` events emitted
- Both TUI and terminal modes use async pattern
