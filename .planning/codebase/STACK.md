# Technology Stack

**Analysis Date:** 2026-08-03

## Languages

**Primary:**
- Python 3.10+ - Launcher (`start.py`), installer (`setup.py`), LLM server orchestration, model management, hardware detection (`scripts/`, `tests/`)
- Rust (edition 2024) - Core agent: orchestrator, tool runtime, TUI, HTTP API, context engine (`agent-rs/`)
- TypeScript 5.9 - Web UI (`web-ui/`)

**Secondary:**
- C/C++ - llama.cpp inference engine, compiled from source at setup time (`llama.cpp/`, cloned during setup, gitignored build output)
- Bash - Server launch helper scripts (`scripts/start_agent.sh`, `scripts/start_server.sh`)
- JavaScript - Vite/ESLint config files (`web-ui/vite.config.ts`, `web-ui/eslint.config.js`)

## Runtime

**Environment:**
- Python 3.10+ (required by `README.md`; no `.python-version` file)
- Rust toolchain (edition 2024 requires recent stable, ~1.85+; auto-installed via rustup by `setup.py`; no `rust-toolchain.toml`)
- Node.js for web-ui dev (Vite 8 requires Node 20.19+ / 22.12+; no `.nvmrc` pinned)
- No cloud runtime — everything executes locally on the user's machine

**Package Manager:**
- pip - Python (no `requirements.txt`; `setup.py` installs inline: `requests`, `tqdm`, `openai`, `huggingface_hub`, `cmake`, `openvino`)
- cargo - Rust, lockfile `agent-rs/Cargo.lock` present
- npm - web-ui, lockfile `web-ui/package-lock.json` present

## Frameworks

**Core:**
- axum 0.7 - Rust HTTP API server (web mode, binds `127.0.0.1:3000` in `agent-rs/src/server.rs`)
- tokio 1.43 - Async runtime (features `["full"]`)
- React 19.2 - Web UI (`web-ui/src/App.tsx`)
- Vite 8 - Web UI dev server / bundler (`web-ui/vite.config.ts`, no proxy configured)
- Tailwind CSS 3.4 - Web UI styling (`web-ui/tailwind.config.js`)
- ratatui 0.26 + crossterm 0.27 - Terminal TUI (`agent-rs/src/tui.rs`, `agent-rs/src/tui/`)
- llama.cpp - Local LLM inference backend, built via CMake with GGML_CUDA/GGML_VULKAN/GGML_OPENVINO flags (`setup.py` `build_llama_cpp`)
- KoboldCPP - Fallback inference binary (downloaded from GitHub releases, `setup.py` `KOBOLD_URLS`)

**Testing:**
- pytest - Python unit tests (9 files in `tests/test_*.py`, run via `python -m pytest`)
- cargo test - Rust unit + integration tests (24 integration files in `agent-rs/tests/`; dev-deps: `mockall 0.13`, `mockito 1.4`, `tempfile 3.10`)
- `tests/eval.py` - Agentic benchmark suite (drives the built agent binary against a local model judge; outputs `tests/benchmark_results.md`)
- No JavaScript test framework configured in `web-ui/package.json`

**Build/Dev:**
- CMake - llama.cpp build (setup.py invokes `cmake -S llama.cpp -B llama.cpp/build` with backend flags)
- TypeScript compiler (`tsc -b`) - web-ui type checking/build
- Vite - web-ui bundling
- `scripts/build_zip.py` - Packaged zip distribution builder

## Key Dependencies

**Critical:**
- async-openai 0.33.1 - OpenAI-compatible client used to talk to the local llama-server/koboldcpp `/v1` API (`agent-rs/src/main.rs`)
- reqwest 0.12.9 - HTTP client (local LLM calls, web research, backend probing) (`agent-rs/src/config.rs`, `agent-rs/src/server.rs`)
- rusqlite 0.39 (bundled) - Embedded SQLite for command audit log (`agent-rs/src/audit.rs`, writes `logs/audit.db`)
- bollard 0.20.2 - Docker SDK for sandboxed command execution (`agent-rs/src/security/sandbox.rs`)
- huggingface_hub - GGUF model downloads (with `requests` fallback) (`scripts/model_install.py`, `scripts/download_model.py`)
- gbnf 0.2.6 - GBNF grammar sampling for constrained tool-call generation (`agent-rs/src/main.rs`)

**Infrastructure:**
- tree-sitter 0.24 + tree-sitter-rust 0.23 - Codebase search/indexing (`agent-rs/src/context/indexer.rs`)
- tiktoken-rs 0.9.1 - `cl100k_base` token counting for context budgeting (`agent-rs/src/tokens.rs`)
- scraper 0.21 + htmd 0.1 - HTML scraping → markdown for web research (`agent-rs/src/agent_core/web_research/`)
- rustyline 15 - Terminal REPL (`agent-rs/src/main.rs`)
- tokio-stream / futures-util / bytes - SSE streaming for web mode (`agent-rs/src/server.rs`)
- evtx 0.8.1 - Windows event log parsing (diagnostics)
- governor 0.6 - Rate limiting (tool execution)
- windows-service 0.8.0 / service-manager 0.11 - Windows service management (diagnostics)
- sysinfo 0.38 / network-interface / num_cpus - System stats collection

## Configuration

**Environment:**
- No `.env` files and no dotenv; configuration via environment variables (all prefixed `HELIX_`) plus generated `scripts/config.py`
- Key env vars: `HELIX_MODEL_NAME`, `HELIX_MODEL_PATH`, `HELIX_SERVER_PORT` (default 8080), `HELIX_CONTEXT_SIZE`, `HELIX_GPU_LAYERS`, `HELIX_BACKEND_HINT`, `HELIX_BATCH_SIZE`, `HELIX_UBATCH_SIZE`, `HELIX_CPU_THREADS`, `HELIX_UI_MODE` (`terminal`/`tui`/`web`), `HELIX_EXEC_MODE` (`agentic`/`chat`), `HELIX_SANDBOX`, `HELIX_SESSION_DIR`, `HELIX_PROFILE_PATH`, `HELIX_RUNTIME_PROFILE`, `HELIX_MIN_CONTEXT_SIZE`, `HELIX_MIN_TOK_S`, `HELIX_JUDGE_URL`, `HELIX_EVAL_MAX_TASKS`, `KOBOLDCPP_ARGS`
- Runtime settings stored in `scripts/config.py` (generated by `setup.py`; static model catalog + detection fallback also present there)
- Rust agent loads config by executing Python bridge (`agent-rs/src/config.rs` `load_from_python` → `python -c` → JSON)
- User profile persisted at `~/.helix/onboarding_profile.json`; sessions at `~/.helix/sessions/` (`scripts/onboarding_profile.py`)

**Build:**
- `agent-rs/Cargo.toml` - Rust manifest
- `web-ui/tsconfig.json`, `tsconfig.app.json`, `tsconfig.node.json` - TypeScript compiler options
- `web-ui/vite.config.ts` - Vite config
- `web-ui/tailwind.config.js`, `web-ui/postcss.config.js` - CSS pipeline
- `web-ui/eslint.config.js` - ESLint 9 flat config
- No CI configuration (no `.github/` directory)

## Platform Requirements

**Development:**
- Cross-platform: Windows, Linux, macOS (detected via `platform.system()` throughout `setup.py`)
- Windows: Visual Studio Build Tools with C++ workload required for llama.cpp build (auto-detected via `vswhere`/`vcvars64.bat`, auto-installed via winget); Rust toolchain installed via winget/rustup-init
- CUDA toolkit optional (auto-installed via winget); OpenVINO installed via pip on Intel-hinted setups
- llama.cpp cloned from `https://github.com/ggerganov/llama.cpp.git` at setup if `llama.cpp/CMakeLists.txt` is absent

**Production:**
- Distributed as source + built artifacts; packaged via `scripts/build_zip.py` (zip archive)
- Runs entirely offline after setup — no server-side deployment target
- LLM backends: `llama-server` (primary, from local llama.cpp build) with CUDA → Vulkan → CPU fallback chain; KoboldCPP as last-resort fallback

---

*Stack analysis: 2026-08-03*
*Update after major dependency changes*
