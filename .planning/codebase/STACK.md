# Technology Stack

**Last Updated:** 2026-07-24

## Overview
Helix CLI is an autonomous local troubleshooting agent stack that combines a Rust-based high-performance agent runtime (`agent-rs`), a Python orchestration layer (`start.py` & `scripts/`), a local LLM inference server (`llama-server` / `koboldcpp`), and a modern Web UI (`web-ui`).

## Component Stack

### 1. Core Agent Engine (`agent-rs/`)
- **Language & Edition:** Rust (Edition 2024)
- **Binary Target:** `helix-agent` (`src/main.rs`)
- **Library Crate:** `agent_rs` (`src/lib.rs`)
- **Async Runtime:** Tokio 1.43 (`tokio` with `full` features), `futures-util`, `tokio-stream`
- **TUI & Terminal UI:** `ratatui` 0.26, `crossterm` 0.27, `tui-input` 0.8, `tachyonfx` 0.11, `throbber-widgets-tui` 0.4, `rustyline` 15 (file history), `inquire` 0.9
- **HTTP Server & API:** `axum` 0.7, `tower-http` 0.5 (CORS support), `bytes` 1.5
- **HTTP Client & AI API:** `async-openai` 0.33, `reqwest` 0.12 (JSON & streaming)
- **Database & Storage:** `rusqlite` 0.39 (bundled SQLite engine), `serde` 1.0, `serde_json` 1.0, `schemars` 1.2
- **Security & Sandbox:** `bollard` 0.20 (Docker API client), `path-security` 0.2, `soft-canonicalize` 0.5, `shell-sanitize` 0.1, `shell-words` 1.0
- **OS Diagnostics & System:** `sysinfo` 0.38, `evtx` 0.8 (Windows Event Log parser), `network-interface` 1.1, `service-manager` 0.11, `windows-service` 0.8, `num_cpus` 1.17
- **Tokenizer & Grammar:** `tiktoken-rs` 0.9, `gbnf` 0.2 (BNF/GBNF grammar parser for local LLM sampling)

### 2. Local LLM Server & Management (`scripts/`, `start.py`)
- **Language:** Python 3.10+
- **Inference Binary:** `llama-server` (primary GGUF runner) / `koboldcpp` (fallback GGUF runner)
- **Protocol:** OpenAI-compatible HTTP API (`http://127.0.0.1:8080/v1`)
- **Dependencies:** `requests`, standard library (`subprocess`, `os`, `pathlib`, `json`)
- **Model Storage:** GGUF quantized models located in `models/` directory

### 3. Web Interface (`web-ui/`)
- **Framework:** React 19.2 (`react`, `react-dom`)
- **Language:** TypeScript 5.9 (`typescript`, `typescript-eslint`)
- **Build Tool & Dev Server:** Vite 8.0 (`vite`, `@vitejs/plugin-react`)
- **Styling:** Tailwind CSS 3.4 (`tailwindcss`, `postcss`, `autoprefixer`)
- **Icons & Rendering:** Lucide React 0.577 (`lucide-react`), React Markdown 10.1 (`react-markdown`, `rehype-raw`)
- **Linting:** ESLint 9.39 (`eslint`, `@eslint/js`, `eslint-plugin-react-hooks`)

## Build & Run Dependencies

| Environment | Tool / Binary | Purpose |
| --- | --- | --- |
| Rust Compilation | `cargo` (Rustup 2024 edition support) | Compiles `agent-rs` binary and runs tests |
| Python Environment | Python 3.10+ | Orchestrates server startup, model downloads, system checks |
| Web UI Build | Node.js 18+, `npm` | Vite dev server and production web assets bundler |
| Local AI Runner | `llama-server` or `koboldcpp` | Local CPU/GPU GGUF model execution |
| Docker (Optional) | `docker` daemon | Containerized sandbox execution mode for high-risk tools |
