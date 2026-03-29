# INTEGRATIONS.md

## Snapshot
Last refreshed: 2026-03-29
This project is mostly local-only and integrates with local processes/services rather than SaaS APIs.

## Runtime Service Integrations

### Local LLM API
- Endpoint: `http://127.0.0.1:8080/v1`
- API style: OpenAI-compatible chat completions
- Caller: Rust orchestrator (`agent-rs/src/main.rs`, `agent-rs/src/server.rs`)
- Transport: HTTP via `reqwest`

### Rust Agent Web API
- Endpoint: `http://127.0.0.1:3000`
- Routes:
  - `POST /chat` (SSE stream of agent events)
  - `GET /health`
- Server implementation: Axum in `agent-rs/src/server.rs`
- Consumer: React app (`web-ui/src/App.tsx`)

## Process Integrations
- `start.py` launches `scripts/start_server.py`, then launches Rust orchestrator.
- In web mode, `start.py` also launches Vite dev server for `web-ui`.
- `scripts/start_server.py` starts `llama-server`; falls back to KoboldCPP if needed.

## OS-Level Integrations
- Process cleanup:
  - Linux/macOS: `pkill -f llama-server|koboldcpp`
  - Windows: `taskkill /IM llama-server.exe|koboldcpp.exe`
- Tool execution in Rust uses sandbox checks and runs commands in project directory (`agent-rs/src/tools.rs`).

## Filesystem Integrations
- Models from `models/*.gguf`
- Logs written under `logs/`
- Python config bridge reads `scripts/config.py`
- Rust tool sandbox root resolves to repository root

## Not Present
- No cloud auth providers
- No remote database integration in primary flow
- No managed queue/broker dependency
