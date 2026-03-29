# STACK.md

## Snapshot
Last refreshed: 2026-03-29
Primary product: local-first AI agent stack with Rust orchestrator, Python bootstrap/runtime helpers, and optional React web UI.

## Languages
- Rust (core orchestrator in `agent-rs/`)
- Python (setup, server launcher, project bootstrap in root and `scripts/`)
- TypeScript (web UI in `web-ui/`)
- Shell (startup helpers in `scripts/*.sh`)

## Build and Runtime
- Rust build: Cargo (`agent-rs/Cargo.toml`, edition 2024)
- Python runtime: CPython with local scripts
- Web runtime: Node + Vite (`web-ui/package.json`)

## Key Rust Dependencies
- Networking/API: `reqwest`, `axum`, `tower-http`, `tokio-stream`, `futures-util`
- Serialization/schema: `serde`, `serde_json`, `schemars`
- LLM and token work: `async-openai`, `tiktoken-rs`, `gbnf`
- CLI/TUI: `rustyline`, `ratatui`, `crossterm`, `tui-input`
- System and filesystem: `sysinfo`, `ignore`

## Key Python Components
- `start.py`: top-level launcher for server + agent + optional web UI
- `scripts/start_server.py`: llama.cpp first, KoboldCPP fallback
- `setup.py`: environment and model setup workflow
- `scripts/system_check.py`: host capability detection

## Web UI Stack
- React 19 + TypeScript
- Vite 8 toolchain
- Tailwind CSS
- Markdown rendering via `react-markdown` + `rehype-raw`

## External Engine Dependency
- Local inference engine expected at OpenAI-compatible endpoint (`http://127.0.0.1:8080/v1` by default)
- Preferred backend: llama.cpp `llama-server`
- Fallback backend: KoboldCPP
