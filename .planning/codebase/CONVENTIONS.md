# CONVENTIONS.md

## Snapshot
Last refreshed: 2026-03-29
Conventions are inferred from current implementation and planning artifacts.

## Runtime and Mode Conventions
- Environment variables drive mode selection:
  - `HELIX_UI_MODE` (`tui` or `web`)
  - `HELIX_EXEC_MODE` (`agentic` or `chat`)
- Model/runtime values are sourced from Python `scripts/config.py` via Rust bridge.

## Rust Code Conventions
- Async-first style with `tokio` for I/O and streaming paths.
- Message and API payloads represented via typed structs in `types.rs`.
- Tool execution returns structured success/failure (`ToolResult`).
- Sandboxing required for file operations (`enforce_sandbox` in `tools.rs`).
- Event-based UI updates (`TuiAction` and `TuiEvent`).

## API/Event Conventions
- Web agent route uses SSE with typed event labels:
  - `text`, `tool_start`, `tool_result`, `system`, `error`, `done`
- Rust web API is permissive CORS in current implementation.

## UI Conventions
- TUI: interaction-centric with explicit context and status updates.
- Web UI: assistant messages can include both final text and intermediate events.
- Markdown rendering enabled in web UI (`react-markdown` + `rehype-raw`).

## Planning Workflow Conventions
- Work is tracked under `.planning/` with phase-numbered artifacts.
- Plans and summaries are kept per phase directory.
- Codebase maps live under `.planning/codebase/` and are refreshable snapshots.

## Operational Safety Conventions
- Dangerous shell commands may be blocked unless confirmation is disabled in config.
- Tool output is truncated to prevent context blow-ups.
- Context compaction is triggered near configured token threshold.
