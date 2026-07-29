# External Integrations

**Analysis Date:** 2026-07-30

## APIs & External Services

**LLM Backend (OpenAI-compatible API):**
- **Primary:** Local llama.cpp server (`llama-server`) — runs as a subprocess, exposes OpenAI-compatible `/v1/` REST API
- **Fallback:** KoboldCPP binary — downloaded from GitHub releases, same API surface
- **Protocol:** OpenAI Chat Completions API (`/v1/chat/completions`), Models API (`/v1/models`), Completions API (`/v1/completions`)
- **Communication:** HTTP via `reqwest` (Rust) and `requests` (Python)
- **Auth:** None (local-only, bound to `127.0.0.1`)
- **Discovery:** Rust agent probes `/v1/models` and inspects response body for "kobold" vs "llama" strings to detect server flavor (`agent-rs/src/main.rs:19-39`)
- **Configuration:** `scripts/config.py` defines `BASE_URL` (default `http://127.0.0.1:8080/v1`), `MODEL_NAME`, `SERVER_PORT`

**HuggingFace Hub API:**
- **Purpose:** Model discovery and GGUF file downloads during `setup.py`
- **Endpoints:**
  - `https://huggingface.co/api/models` — Search for models by query (`setup.py:297`)
  - `https://huggingface.co/api/models/{repo}/tree/main` — List files in model repo (`setup.py:316`)
  - `https://huggingface.co/{repo}/resolve/main/{path}` — Direct file download (`setup.py:383`)
- **SDK/Client:** `requests` library (raw HTTP), `huggingface_hub` package
- **Auth:** None (public repositories)

**GitHub Releases (KoboldCPP):**
- **Purpose:** Download KoboldCPP fallback binary during setup
- **URL:** `https://github.com/LostRuins/koboldcpp/releases/latest/download/koboldcpp-{platform}`
- **Platform binaries:** `koboldcpp.exe` (Windows), `koboldcpp-linux-x64` (Linux), `koboldcpp-mac-x64` (macOS)
- **SDK/Client:** `requests` library, streaming download with `tqdm` progress

**GSD SDK CLI:**
- **Purpose:** GSD phase commands from within the TUI (`/gsd`, `/gsd-*` slash commands)
- **Binary:** External `gsd-sdk` process called via `std::process::Command`
- **Usage:** `gsd-sdk progress --model <name>`, `gsd-sdk init`, etc.
- **Communication:** Process stdout/stderr capture
- **Type:** Optional integration — GSD commands silently fail if binary not found (`agent-rs/src/main.rs:972-1122`)

## Data Storage

**Databases:**
- **SQLite** via `rusqlite` (Rust) — Two separate databases:
  1. **Audit Chain DB** — `logs/audit.db` — Tamper-evident event log with SHA-256 hash chaining (`agent-rs/src/audit.rs:32-50`). Schema: `audit_logs` table with columns for timestamp, actor, path, event_type, tool_name, decision, outcome, prev_hash, event_hash.
  2. **Context Engine DB** — `.helix/helix_context.db` — Symbol cache and import edges (`agent-rs/src/context/indexer.rs:31-61`). Schema: `symbol_cache`, `symbols`, `import_edges` tables with WAL journal mode.

**File Storage:**
- **Local filesystem only** — Model files stored in `models/` directory as `.gguf` files.
- Logs stored in `logs/` directory.
- Configuration stored in `scripts/config.py`.
- Snapshots stored in `.helix/backups/` directory.

**Caching:**
- **Context engine symbol cache** — SQLite-backed incremental cache with SHA-256 file hashing for invalidation (`agent-rs/src/context/indexer.rs:76-90`). Only re-indexes changed files.
- **Web research evidence store** — In-memory `EvidenceStore` (`agent-rs/src/agent_core/web_research/store.rs`) with freshness classifier to skip redundant live searches (`agent-rs/src/agent_core/web_research/classifier.rs`).

## Authentication & Identity

**Auth Provider:**
- **None** — All services operate locally on `127.0.0.1`. No external auth tokens, OAuth providers, or identity services.
- **Local user identity** — `PermissionRequester` trait (`agent-rs/src/types.rs:47-49`) implemented by `InquirePermissionRequester` (`agent-rs/src/tui/approval.rs`) for interactive tool execution approval.

## Monitoring & Observability

**Error Tracking:**
- **None** — No Sentry, DataDog, or similar services. All errors go to stderr/stdout logs.

**Logs:**
- **File-based** — `logs/start_server.stdout.log` and `logs/start_server.stderr.log` capture LLM server output.
- **Console** — Rust agent logs to stdout/stderr during operation.
- **Audit chain** — SQLite-backed structured log of all tool execution decisions (`agent-rs/src/audit.rs`).

**Health Checks:**
- **Watchdog** — Internal `Watchdog` struct (`agent-rs/src/watchdog.rs`) tracks LLM server health with states: Healthy, Degraded, Recovering, Cooldown, Unhealthy. Supports auto-restart with exponential backoff up to 3 restarts with 5-minute cooldown.
- **Probe** — Rust agent sends periodic `/v1/chat/completions` probe requests to verify model readiness (`agent-rs/src/main.rs:390-413`).

## CI/CD & Deployment

**Hosting:**
- **Self-hosted / local desktop** — No cloud deployment. The entire stack runs on the user's machine.

**CI Pipeline:**
- **None** — No GitHub Actions, Jenkins, or similar pipelines detected.

## Environment Configuration

**Required env vars:**
- `HELIX_MODEL_NAME`, `HELIX_MODEL_PATH` — Model to load
- `HELIX_EXEC_MODE` — `chat` or `agentic`
- `HELIX_UI_MODE` — `tui` or `web`
- `HELIX_SERVER_PORT` — Server port (default 8080)

**Optional env vars** (see STACK.md for full list):
- `HELIX_GPU_LAYERS`, `HELIX_GPU_VRAM_GB`, `HELIX_BACKEND_HINT`, `HELIX_BATCH_SIZE`, `HELIX_UBATCH_SIZE`, `HELIX_CPU_THREADS`, `HELIX_CONTEXT_SIZE`, `HELIX_RUNTIME_PROFILE`, `HELIX_FORCE_TOOL_GRAMMAR`, `HELIX_CHAT_MAX_TOKENS`, `HELIX_MIN_TOK_S`, `HELIX_SERVER_STARTUP_TIMEOUT_S`, `HELIX_RECOVERY_RETRY_ATTEMPTS`, `HELIX_HTTP_RETRY_DELAY_MS`, `HELIX_RUN_AGENTIC_PREFLIGHT`, `HELIX_MIN_CONTEXT_SIZE`, `HELIX_LOW_END_MODE`, `AGENT_PERSONA`

**Secrets location:**
- **None** — No secrets management. The system is entirely local with no API keys or credentials.
- `.env` file present — contains environment configuration (contents not read — see forbidden files policy).

## Webhooks & Callbacks

**Incoming:**
- **None** — The axum web server (`agent-rs/src/server.rs`) exposes REST endpoints (`/v1/status`, `/v1/tools`, `/v1/context`, `/chat`), but these are consumed only by the local web UI frontend, not by external webhooks.

**Outgoing:**
- **None** — No webhook callbacks to external services.

## System-Level Integrations

**nvidia-smi:**
- **Purpose:** GPU VRAM detection for optimal model offload configuration
- **Method:** `subprocess.run(["nvidia-smi", "--query-gpu=memory.total", ...])` from `scripts/config.py:46-52`
- **Fallback:** Environment variable `HELIX_GPU_VRAM_GB` override; returns `None` if nvidia-smi unavailable

**Docker Daemon:**
- **Purpose:** Sandboxed command execution via `bollard` crate (`agent-rs/src/security/sandbox.rs`)
- **Method:** `Docker::connect_with_local_defaults()` — connects to local Docker socket
- **Capabilities:** Container creation, start, wait, log capture, and removal. Uses `helix_sandbox` prefix for container naming.
- **Default image:** `ubuntu:latest` with workspace mounted at `/workspace`
- **Fallback:** Native command execution when Docker is unavailable (controlled by `sandbox_interpreters` config flag)

**Windows Service Management:**
- **Purpose:** System service control (status, start, stop) via `windows-service` (`agent-rs/Cargo.toml`)
- **Used by:** `tools.rs` — `GetServiceStatus` tool for querying service status
- **Also:** `service-manager` crate for cross-platform service management

**Windows Event Log:**
- **Purpose:** Parse Windows Event Log (`.evtx`) files for system diagnostics
- **Library:** `evtx` crate (`agent-rs/Cargo.toml`)
- **Used by:** `agent_core/diagnostics/` module

**Network Interface Detection:**
- **Purpose:** Enumerate local network interfaces
- **Library:** `network-interface` crate (`agent-rs/Cargo.toml`)

## Web Research Pipeline

**Web Fetching:**
- **Library:** `reqwest` (Rust HTTP client) — used in `agent_core/web_research/worker.rs`
- **Rate limiting:** `governor` crate — 2 requests/second per worker pool
- **Timeout:** 30 seconds per request
- **Body limit:** 1 MB max response body
- **SSRF protection:** `sanitize.rs` — `is_ssrf_safe()` validation on URLs

**HTML Processing:**
- **Parsing:** `scraper` crate for HTML parsing
- **Markdown conversion:** `htmd` crate — HTML-to-Markdown transformation (`agent-rs/Cargo.toml`)
- **Sanitization:** `sanitize_html_to_markdown()` in `agent_core/web_research/sanitize.rs`

---

*Integration audit: 2026-07-30*
