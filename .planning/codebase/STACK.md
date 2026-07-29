# Technology Stack

**Analysis Date:** 2026-07-30

## Languages

**Primary:**
- **Rust** (edition 2024) — Core orchestrator binary (`agent-rs/src/`). The `helix-agent` binary (`agent-rs/src/main.rs`) and library crate `agent_rs` (`agent-rs/src/lib.rs`).
- **Python 3** — Entry point launcher (`start.py`), setup/install (`setup.py`), LLM server bootstrap (`scripts/start_server.py`), model installation (`scripts/model_install.py`), hardware detection (`scripts/system_check.py`), configuration generation (`scripts/config.py`), branding, download helpers, and eval harness.

**Secondary:**
- **TypeScript** (5.9) — Optional web UI frontend (`web-ui/src/App.tsx`, `web-ui/src/main.tsx`).
- **CSS** — Tailwind-styled UI via `web-ui/src/App.css` and `web-ui/src/index.css`.

## Runtime

**Environment:**
- Rust binary: compiled `helix-agent` — runs directly on host OS.
- Python runtime: CPython 3.x (required at minimum for launcher/setup scripts).

**Package Managers:**
- **Cargo** — Rust crate manager. Lockfile: `agent-rs/Cargo.lock` (present).
- **pip** — Python dependency manager. Dependencies: `requests`, `tqdm`, `openai`, `huggingface_hub`, `cmake`, `openvino`.
- **npm** — Node/TypeScript frontend dependencies. Lockfile: `web-ui/package-lock.json` (present).

## Frameworks

**Core — Rust:**
| Framework | Version | Purpose |
|-----------|---------|---------|
| `tokio` | 1.43.0 | Async runtime (full feature set: multi-thread, net, io, sync, process) |
| `axum` | 0.7 | HTTP REST + SSE web server for web UI mode |
| `tower-http` | 0.5 | CORS middleware for axum |
| `reqwest` | 0.12.9 | HTTP client (JSON + stream features, used for LLM API calls & web research) |
| `serde` / `serde_json` | 1.0 | JSON serialization across the entire codebase |
| `schemars` | 1.2.1 | JSON Schema generation for tool definitions |
| `tiktoken-rs` | 0.9.1 | OpenAI cl100k_base token counting |
| `ratatui` | 0.26 | TUI framework (terminal user interface) |
| `crossterm` | 0.27 | Terminal backend for ratatui |
| `async-openai` | 0.33.1 | OpenAI API client (used for LLM completions) |

**Core — Python:**
| Library | Purpose |
|---------|---------|
| `requests` | HTTP downloads (HuggingFace models, KoboldCPP binary, LLM API probes) |
| `openai` | Python-side OpenAI client (eval harness) |
| `huggingface_hub` | HuggingFace model repository interaction (setup.py) |
| `tqdm` | Download progress bars |

**Web UI — TypeScript/React:**
| Framework | Version | Purpose |
|-----------|---------|---------|
| React | 19.2.4 | UI component library |
| Vite | 8.0.1 | Build tool and dev server |
| Tailwind CSS | 3.4.19 | Utility-first CSS |
| TypeScript | 5.9.3 | Type-safe frontend code |
| react-markdown | 10.1.0 | Markdown rendering in chat messages |
| lucide-react | 0.577.0 | Icon library |
| rehype-raw | 7.0.0 | Raw HTML passthrough for markdown |

**Testing:**
| Framework | Version | Purpose |
|-----------|---------|---------|
| Rust `#[cfg(test)]` | — | Inline unit tests in source files |
| Rust integration tests | — | 15 test files in `agent-rs/tests/` |
| `mockall` | 0.13 | Rust mock object framework (dev-dependency) |
| `tempfile` | 3.10 | Temp directory helpers for tests (dev-dependency) |
| Python `pytest` | — | Python test suite (`tests/test_*.py`) |

## Key Dependencies

**Critical — Rust:**
| Crate | Version | Why It Matters |
|-------|---------|----------------|
| `async-openai` | 0.33.1 | Primary interface to LLM backends via OpenAI-compatible API |
| `reqwest` | 0.12.9 | All HTTP communication: LLM API calls, web research fetching, HuggingFace |
| `rusqlite` | 0.39.0 | SQLite for audit chain (`src/audit.rs`) and context engine (`src/context/indexer.rs`) |
| `tree-sitter` | 0.24 | Symbol extraction from source code in context engine |
| `bollard` | 0.20.2 | Docker API — sandboxed command execution (`src/security/sandbox.rs`) |
| `tiktoken-rs` | 0.9.1 | Token counting for context budget management |
| `scraper` | 0.21 | HTML parsing for web research pipeline |
| `petgraph` | 0.6 | Graph-based dependency tracking in context engine (import edges) |
| `sha2` | 0.11.0 | SHA-256 hashing for audit chain integrity and file invalidation |
| `governor` | 0.6 | Rate limiting for web research worker pool |
| `chrono` | 0.4.44 | Timestamps for audit events and logs |
| `regex` | 1 | Pattern matching in policy engine |
| `shell-sanitize` | 0.1.0 | Shell argument sanitization for security |
| `path-security` | 0.2.0 | Path traversal protection |
| `gbnf` | 0.2.6 | GBNF grammar generation for constrained tool-calling |
| `evtx` | 0.8.1 | Windows Event Log parsing for diagnostics |

**Infrastructure — Python:**
| Library | Purpose |
|---------|---------|
| `openvino` | Intel OpenVINO backend for llama.cpp |
| `cmake` | Build tool for compiling llama.cpp with hardware optimizations |

**Build / Toolchain:**
| Tool | Purpose |
|------|---------|
| Rust `cargo` | Rust compilation pipeline |
| Python `pip` | Python dependency installation |
| CMake | llama.cpp build system |
| `vswhere.exe` | Visual Studio detection (Windows C++ build tools) |
| `winget` | Windows package manager (VS Build Tools auto-install) |
| `nvidia-smi` | GPU VRAM detection (queried from Python `config.py`) |

## Configuration

**Environment Variables:**
- `HELIX_MODEL_NAME`, `HELIX_MODEL_PATH` — Model selection
- `HELIX_SERVER_PORT` — LLM server TCP port (default 8080)
- `HELIX_EXEC_MODE` — `chat` or `agentic`
- `HELIX_UI_MODE` — `tui` or `web`
- `HELIX_GPU_LAYERS`, `HELIX_GPU_VRAM_GB` — GPU offload settings
- `HELIX_BACKEND_HINT` — `cuda`, `vulkan`, `openvino`, `cpu`
- `HELIX_BATCH_SIZE`, `HELIX_UBATCH_SIZE`, `HELIX_CPU_THREADS`, `HELIX_CONTEXT_SIZE` — Runtime performance tuning
- `HELIX_RUNTIME_PROFILE` — `LatencyCpu`, `BalancedCpu`, `SafeRecovery`
- `HELIX_FORCE_TOOL_GRAMMAR` — Force GBNF grammar for tool calling
- `HELIX_CHAT_MAX_TOKENS` — Max tokens per chat response (default 1024)
- `HELIX_MIN_TOK_S` — Minimum token/s threshold during setup (default 10.0)
- `HELIX_MIN_CONTEXT_SIZE` — Minimum context window (default 4096)
- `HELIX_SERVER_STARTUP_TIMEOUT_S` — Server startup timeout (default 180)
- `HELIX_RECOVERY_RETRY_ATTEMPTS` — Retries after model server recovery (default 45)
- `HELIX_HTTP_RETRY_DELAY_MS` — HTTP retry delay (default 1000)
- `HELIX_RUN_AGENTIC_PREFLIGHT` — Toggle for setup benchmark suite
- `AGENT_PERSONA` — `os_assistant`, `coder`, or `researcher`

**Config Files:**
- `scripts/config.py` — Generated by `setup.py`; loaded by both Python (server launcher) and Rust (via `load_from_python()` which execs the Python config and captures JSON).
- `.helix/helix_context.db` — SQLite database for context engine symbol cache.
- `logs/audit.db` — SQLite database for tamper-evident audit chain.
- `.helix/backups/` — Snapshot directory for repair safety loop.
- `logs/` — Runtime logs (stdout, stderr from llama-server).

**Build Configuration:**
- `agent-rs/Cargo.toml` — Rust crate manifest (bin: `helix-agent`, lib: `agent_rs`)
- `setup.py` — Full installation pipeline: hardware detection, model download, llama.cpp build, Rust build, config generation, speed gate, benchmark
- `web-ui/vite.config.ts` — Vite build config
- `web-ui/tsconfig.json`, `tsconfig.app.json`, `tsconfig.node.json` — TypeScript configs
- `web-ui/tailwind.config.js` — Tailwind CSS config
- `web-ui/postcss.config.js` — PostCSS config

## Platform Requirements

**Development:**
- Rust toolchain (via rustup)
- Python 3.x with pip
- CMake (installed via pip if missing)
- C++ build tools (Visual Studio on Windows, build-essential on Linux)
- Node.js 18+ (for web-ui development)
- nvidia-smi (optional, for CUDA GPU detection)
- Docker (optional, for sandboxed execution)

**Production:**
- Compiled `helix-agent` binary (from `cargo build`)
- Python 3.x runtime
- llama.cpp compiled with hardware backend (CUDA, Vulkan, OpenVINO, or CPU)
- GGUF model files in `models/` directory
- KoboldCPP binary (fallback server)
- Docker daemon (optional, for sandboxed execution)

---

*Stack analysis: 2026-07-30*
