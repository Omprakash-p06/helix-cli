# ARCHITECTURE.md

## Snapshot
Last refreshed: 2026-03-29
Architecture is a hybrid local stack: Python boot/runtime control + Rust agent orchestration + optional React UI.

## Major Layers

### 1) Boot and Environment Layer (Python)
- Entry point: `start.py`
- Responsibilities:
  - Model selection and mode selection (web vs tui, agentic vs chat)
  - Start local inference server process
  - Wait for model API readiness
  - Start Rust orchestrator and optional web dev server

### 2) Inference Server Layer (Python + local binaries)
- Launcher: `scripts/start_server.py`
- Primary path: llama.cpp `llama-server`
- Fallback path: KoboldCPP
- Exposes OpenAI-compatible API at `:8080/v1`

### 3) Agent Orchestrator Layer (Rust)
- Main loop: `agent-rs/src/main.rs`
- Behavior:
  - Loads config via Python bridge (`agent-rs/src/config.rs`)
  - Builds tool schemas/grammar
  - Calls local model endpoint
  - Parses streaming deltas and tool calls
  - Supports chat, terminal UI, and web-server modes
- Safety and tooling:
  - Filesystem sandbox checks in `agent-rs/src/tools.rs`
  - Dangerous command gating based on config

### 4) Presentation Layer
- TUI path: `agent-rs/src/tui.rs` (Ratatui + Crossterm)
- Web path: `agent-rs/src/server.rs` + `web-ui/src/App.tsx`
  - Axum emits SSE events
  - React consumes incremental text and tool status events

## Data Flow (Web Mode)
1. User submits message in React app.
2. React calls `POST /chat` on Rust API (`:3000`).
3. Rust agent sends request(s) to local LLM endpoint (`:8080/v1`).
4. Rust emits SSE events (`text`, `tool_start`, `tool_result`, `error`, `done`).
5. React app incrementally renders output and tool activity.

## Data Flow (TUI Mode)
1. User submits message in TUI.
2. TUI action channel sends submit event to main loop.
3. Main loop streams model output and tool events back to TUI event channel.
4. TUI updates context HUD, content panes, and status indicators.
