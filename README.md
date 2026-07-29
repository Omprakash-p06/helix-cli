# Helix Agent

Helix Agent is a local-first AI automation stack for running an LLM with tool-calling on your own machine.

## What problem this project solves

Most AI automation workflows depend on cloud APIs, limited tool permissions, or fragile one-off scripts. Helix solves this by combining:

- Local model inference (privacy + offline-friendly workflows)
- A Rust orchestrator with typed tool calls and safer execution boundaries
- A hardware-aware setup flow that tunes backend/runtime values for your system
- A stable OpenAI-compatible API endpoint for existing clients and UIs

## System architecture

Helix is a Py + Rust hybrid stack. The Rust orchestrator (`agent-rs/`) does all LLM interaction, tool execution, and context management. Python handles setup, server lifecycle, and model installation.

### 1. Setup layer (Python)

- `setup.py` detects hardware, installs dependencies, builds llama.cpp, downloads models, benchmarks throughput, and generates `scripts/config.py`.
- `scripts/model_install.py` and `scripts/download_model.py` handle model acquisition.

### 2. Runtime server layer (Python)

- `scripts/start_server.py` launches `llama-server` from llama.cpp.
- Falls back to KoboldCPP when available.
- `start.py` is the single-command launcher: prompts model selection, boots server, waits for readiness, hands off to Rust orchestrator, tears down server on exit.

### 3. Orchestration layer (Rust)

- `agent-rs/src/main.rs` runs the orchestrator (CLI or TUI mode).
- Calls the OpenAI-compatible endpoint, evaluates tool-call plans, executes tools, and loops with memory compaction logic.
- Supports two personas: `coder` (read/write files) and `researcher` (read-only exploration).
- HTTP retry with exponential backoff and server watchdog (auto-restart on failure).

### 4. Context Engine (Rust)

Hierarchical context engineering layer replacing naive chunking. Initialized at agent startup:

- **Indexer** — Tree-sitter based symbol indexer over workspace source files. Incremental re-indexing of changed files only. Stores symbol metadata in SQLite.
- **Retrieval** — Multi-strategy search (exact match, FTS5 full-text, import graph traversal with PageRank). Budget-bounded results enforced via tiktoken-rs.
- **Memory** — Durable agent memory layer with SQLite FTS5. Stores sessions, edit ledger, and contextual state across agent restarts.
- **Skeleton** — Signature extraction without function bodies. Used for session-start repo skeleton pre-injection (~5K token budget).
- **Budget** — Token budget enforcement with tiktoken-rs ceiling.
- **Architecture export** — Generates DOT graph of the dependency graph at startup (`misc/architecture_<date>.dot`).

### 5. Web Research Agent (Rust)

Built-in deep research pipeline for live web queries:

- **FreshnessClassifier** — Determines if a query needs a live web search or can be answered from cached evidence.
- **ResearchPlanner** — Decomposes queries into parallel search tasks.
- **WorkerPool** — Concurrent HTTP workers (SSRF-protected via URL sanitization).
- **EvidenceStore** — SQLite-backed cache with TTL-based freshness management.
- **EvidenceSynthesizer** — Produces structured `ResearchBrief` with citations from stored evidence.
- Integrated into the context engine via `ContextEngine::enrich_with_research()`.

### 6. Security subsystem (Rust)

- **Policy engine** — Role-based tool access control with `PolicyContext`. Routes tool requests through permission checks.
- **Sandbox** — Command execution with path allowlisting, shell injection detection, and argument sanitization.
- **Capabilities** — Declarative capability model for granular tool permissions.

### 7. Repair subsystem (Rust)

Human-in-the-loop guided repair with safety guarantees:

- **Snapshot manager** — Filesystem snapshots before repair operations (tarball on Linux, VSS on Windows).
- **Confidence scoring** — Bayesian calibration combining token probabilities and evidence strength.
- **Safety loop** — Transactional rollback on validation failure after repair execution.
- **Approval gate** — Interactive `inquire`-based permission prompts requiring user confirmation for high-risk actions.

### 8. Audit subsystem (Rust)

- Tamper-evident audit chain using SHA-256 hashed linked events.
- Queryable via `--audit-query` CLI flag.
- Stores actor, tool, decision, outcome, and chain verification status.

### 9. Diagnostic engine (Rust)

- Multi-source diagnostic reasoning combining log analysis, system state, and confidence-weighted evidence.
- OS-level diagnostics via `system.rs` (process, disk, network, OOM detection).
- Structured hypothesis generation and evaluation.

### 10. TUI layer (Rust)

ratatui-based interactive terminal UI:

- Streaming response display with real-time token rendering.
- HUD with context usage, connection state, and mode indicator.
- Slash commands: `/mode chat|agentic`, `/clear`, `/gsd` subcommands.
- Two layout modes: `wide` (default) and `compact`.
- History management.

### 11. Web server mode (Rust)

- Axum-based HTTP server exposing the agent via REST API.
- Enabled via `HELIX_UI_MODE=web` or server exec mode.

### 12. GSD orchestration integration (Rust)

- Built-in `/gsd` slash commands for discover, discuss, plan, execute, verify, close phases.
- Runs GSD workflow phases directly from the agent TUI.
- Phase state machine with artifact output and summary reporting.

### 13. Launch orchestration (Python)

- `start.py` — single-command launcher that handles model selection, server boot, readiness polling, Rust orchestrator handoff, and teardown.
- `scripts/start_agent.sh` — shell-based launcher alternative.

### 14. Branding/UI layer (Python)

- `scripts/helix_branding.py` provides shared terminal branding used by setup/server startup.
- `scripts/onboarding_profile.py` — interactive model selection and hardware profiling.

## Repository layout

```
setup.py              — Full setup + benchmark gate + config generation
start.py              — One-command stack launcher
scripts/
  start_server.py     — Model backend launcher/fallback
  helix_branding.py   — Shared Helix terminal logo utilities
  system_check.py     — Hardware detection + tier mapping
  model_install.py    — Model download and management
  download_model.py   — Single-model download utility
  config.py           — Generated runtime config (GPU layers, context size, etc.)
  onboarding_profile.py — Interactive first-run model selection
agent-rs/             — Rust orchestrator and tool runtime
  src/
    main.rs           — CLI + TUI entrypoint
    lib.rs            — Public API surface
    config.rs         — Python config loader
    context/          — Context engine (indexer, retrieval, memory, skeleton, budget)
    agent_core/       — Core agent modules
      tool_runtime.rs — Tool lifecycle, execution, and permission routing
      guardian.rs     — Runtime guardrails
      diagnostics/    — OS diagnostics, log analysis, reasoning engine
      repair/         — Snapshots, rollback, confidence scoring, safety loop
      orchestration/  — GSD phase state machine, artifact management, recovery
      web_research/   — Research pipeline (classifier, planner, worker, synthesizer)
    security/         — Policy engine, sandbox, capabilities
    tui/              — ratatui TUI (api, approval, commands, events, state, themes)
    server.rs         — Axum web server mode
    stream.rs         — Streaming response handling
    tokens.rs         — Token counting (tiktoken-rs)
    tools.rs          — Tool registry and grammar generation
    audit.rs          — Tamper-evident audit chain
    watchdog.rs       — Server health monitoring and restart
    runtime_profile.rs — CPU/GPU runtime profile selection
  tests/              — Integration and validation tests (15 test files)
web-ui/               — React + Vite web UI (optional)
models/               — Local GGUF models
logs/                 — Runtime logs (server stdout/stderr, error logs)
llama.cpp/            — Inference backend source/build
```

## How to run

### Quick start (recommended)

After setup is complete:

Linux/macOS:
```bash
source venv/bin/activate
python start.py
```

Windows (PowerShell):
```powershell
.\venv\Scripts\Activate.ps1
python start.py
```

### First-time setup

Linux/macOS:
```bash
python3 -m venv venv
source venv/bin/activate
pip install requests tqdm openai
python setup.py
```

Windows (PowerShell):
```powershell
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install requests tqdm openai
python setup.py
```

### TUI mode

Set `HELIX_UI_MODE=tui` before launching. The TUI supports:
- Streaming response display with real-time token rendering
- `/mode chat` / `/mode agentic` — switch execution modes
- `/clear` — clear conversation history
- `/gsd <phase>` — run GSD workflow phases inline

### Two-terminal mode (manual)

Terminal 1 (server):

Linux/macOS:
```bash
source venv/bin/activate
python scripts/start_server.py
```

Windows (PowerShell):
```powershell
.\venv\Scripts\Activate.ps1
python scripts/start_server.py
```

Terminal 2 (orchestrator):
```bash
cd agent-rs
cargo run
```

### Web server mode

Set `HELIX_UI_MODE=web` (or configure `exec_mode = "server"`) to expose the agent via the Axum-based REST API.

## API endpoint

When server is running:

- Base URL: `http://127.0.0.1:8080/v1`
- Compatible with OpenAI-style clients and many local UIs.

## Models

The setup currently supports these defaults:

- GPT-OSS-20B (IQ4_NL)
- Qwen3.5-9B-Uncensored (Q4_K_M)

Model choice and tuned runtime values are written to `scripts/config.py`. Additional models can be installed via `scripts/model_install.py`.

## Troubleshooting

- If setup says `config.py` missing: run `python setup.py` first.
- If server startup fails: confirm model exists under `models/` and check `llama.cpp/build/bin`.
- If throughput gate blocks setup: reduce expectations for low-VRAM cards or rerun after adjusting runtime config.
- If `cargo` is missing: install Rust toolchain (`rustup`) and retry.
- If TUI is slow or unresponsive: try terminal mode (`HELIX_UI_MODE=terminal`).
- For OOM issues: check `logs/start_server.stderr.log` for memory errors. Lower `GPU_LAYERS` or switch to CPU-only.

## Notes

- Helix is designed for local automation and experimentation.
- Use with care when enabling high-permission tool personas.
- Tamper-evident audit logging can be enabled via `audit_enabled` in config.
