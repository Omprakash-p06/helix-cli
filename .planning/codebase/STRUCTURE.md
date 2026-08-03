# Codebase Structure

**Analysis Date:** 2026-08-03

## Directory Layout

```
helix-cli/
├── agent-rs/              # Rust agent (orchestrator, tools, TUI, context, security)
│   ├── src/               # Rust sources (library `agent_rs` + binary `helix-agent`)
│   ├── tests/             # Rust integration tests (cargo test)
│   ├── Cargo.toml         # Rust manifest (tokio, axum, ratatui, rusqlite, tree-sitter…)
│   ├── .planning/         # Nested GSD planning state for the Rust workstream
│   └── target/            # Rust build output (gitignored)
├── scripts/               # Python runtime helpers (config, server, model, setup)
├── tests/                 # Python pytest tests
├── web-ui/                # React + Vite + Tailwind frontend (web mode)
├── llama.cpp/             # Vendored llama.cpp source (build output under build/)
├── models/                # GGUF model files (gitignored)
├── logs/                  # Runtime logs — start_server.{stdout,stderr}.log (gitignored)
├── misc/                  # Generated architecture diagram exports (gitignored)
├── .helix/                # Runtime state: context DB, backups (created at runtime)
├── .planning/             # GSD planning docs (incl. codebase/ maps)
├── start.py               # One-command launcher (Python)
├── setup.py               # Hardware detection + full setup (Python)
├── apply_fix.py           # Small patch utility
├── 03-VALIDATION.md       # Validation checklist doc
└── README.md              # User-facing docs
```

## Directory Purposes

**agent-rs/**
- Purpose: The complete Rust agent — orchestration loop, tool layer, security, context engine, audit, three UIs
- Contains: `src/*.rs` (top-level modules), `src/agent_core/`, `src/context/`, `src/security/`, `src/tui/` (submodule dirs), `tests/*.rs` integration tests
- Key files: `src/main.rs` (binary entry, ~2000 lines), `src/lib.rs` (module root), `src/tools.rs` (tool registry, ~840 lines), `src/tui.rs` (~1900 lines), `src/server.rs` (axum web API)
- Subdirectories:
  - `src/agent_core/` — `tool_runtime.rs`, `guardian.rs`, `diagnostics/`, `repair/`, `orchestration/`, `web_research/`
  - `src/context/` — `indexer.rs`, `retrieval.rs`, `memory.rs`, `skeleton.rs`, `budget.rs`
  - `src/security/` — `policy.rs`, `sandbox.rs`, `capabilities.rs`
  - `src/tui/` — `api.rs`, `approval.rs`, `commands.rs`, `events.rs`, `state.rs`, `themes.rs`

**scripts/**
- Purpose: Python runtime — configuration, inference-server bootstrap, model install, hardware checking
- Contains: `*.py` modules, all project-relative paths
- Key files: `config.py` (generated config constants + model scanning), `start_server.py` (llama-server/KoboldCPP launcher), `system_check.py` (hardware tiering), `model_install.py` / `download_model.py` (HuggingFace GGUF install), `onboarding_profile.py` (user defaults), `helix_branding.py` (logo), `build_zip.py` (release packaging)
- Subdirectories: None (flat; `__pycache__` is gitignored)

**tests/**
- Purpose: Python pytest suite for the bootstrap layer
- Contains: `test_*.py` (one per scripts module), `eval.py` + `dataset.json` (accuracy evaluation harness)
- Key files: `test_system_check.py`, `test_start_server_runtime_profile.py`, `test_model_install.py`, `test_download_model.py`, `test_accuracy.py`
- Subdirectories: None (flat)

**web-ui/**
- Purpose: React frontend for web mode; consumes the Rust SSE API on `127.0.0.1:3000`
- Contains: `src/` (TSX/CSS), `public/` (static assets), Vite/Tailwind/TS/ESLint configs
- Key files: `src/App.tsx` (chat UI + SSE parsing), `src/main.tsx`, `vite.config.ts`, `tailwind.config.js`, `package.json`

**llama.cpp/**
- Purpose: Vendored llama.cpp source; built by `setup.py` to `llama.cpp/build/bin/llama-server`
- Contains: upstream llama.cpp tree (git submodule/clone of `origin/main`)
- Key files: `build/bin/llama-server(.exe)` (runtime binary, gitignored)

**misc/**
- Purpose: Generated artifacts only
- Contains: `architecture_*.svg` exports from the context indexer (`main.rs` writes `misc/architecture_{date}.dot`→svg)
- Committed: No (gitignored)

**logs/**
- Purpose: Runtime log capture for the inference server and stack
- Contains: `start_server.stdout.log`, `start_server.stderr.log` (written by `start.py`)
- Committed: No (gitignored)

**.helix/**
- Purpose: Runtime state created by the Rust agent on first run
- Contains: `helix_context.db` (SQLite symbol index/memory/research), `backups/` (repair snapshots)
- Committed: No (runtime-generated)

**models/**
- Purpose: Downloaded GGUF model files
- Contains: `*.gguf`, `.staging/` for verified downloads
- Committed: No (gitignored)

## Key File Locations

**Entry Points:**
- `start.py` — full-stack launcher (model select → server boot → agent handoff → teardown)
- `setup.py` — first-time setup: hardware detect, llama.cpp build, config generation, preflight
- `agent-rs/src/main.rs` — Rust agent binary (`helix-agent`); UI dispatch and LLM loop
- `agent-rs/src/server.rs` — web-mode axum API (`127.0.0.1:3000`), invoked from `main.rs:695`
- `scripts/start_server.py` — standalone inference-server launcher
- `scripts/helix.py` — small Python CLI wrapper for scripts

**Configuration:**
- `scripts/config.py` — generated config (GPU_LAYERS, CONTEXT_SIZE, SERVER_PORT, backend hints, model catalog)
- `agent-rs/src/config.rs` — Rust-side `AppConfig` + backend capability probing
- `agent-rs/src/runtime_profile.rs` — CPU/GPU runtime profile selection (PERF-04)
- `agent-rs/Cargo.toml` — Rust dependency manifest
- `web-ui/vite.config.ts`, `web-ui/tailwind.config.js`, `web-ui/tsconfig*.json`, `web-ui/eslint.config.js` — frontend tooling
- `.gitignore` — excludes logs/, models/, misc/, target/, llama.cpp/build/, *.db test artifacts

**Core Logic:**
- `agent-rs/src/main.rs` — orchestrator: LLM round-trip loop, tool dispatch, context compaction, watchdog, mode switching
- `agent-rs/src/tools.rs` — `Tool` trait, `ToolRegistry`, `create_default_registry()`, tool implementations, GBNF grammar generation
- `agent-rs/src/agent_core/tool_runtime.rs` — `ToolRuntime` (policy/permission/timeout/lifecycle wrapper)
- `agent-rs/src/security/policy.rs` — permission tiers, blocked commands, trust levels
- `agent-rs/src/context/mod.rs` — `ContextEngine` facade (indexer/retrieval/memory/skeleton/budget)
- `agent-rs/src/agent_core/orchestration/mod.rs` — GSD phase state machine + artifact persistence
- `agent-rs/src/audit.rs` — tamper-evident audit chain
- `scripts/start_server.py` — inference process launch + OOM fallback chain

**Testing:**
- `agent-rs/tests/*.rs` — Rust integration tests (audit chain, tool runtime, security guardrails, eval suites, context integration)
- `tests/test_*.py` — Python unit tests for scripts modules
- `tests/eval.py` + `tests/dataset.json` — accuracy evaluation harness

**Documentation:**
- `README.md` — quick start, commands, modes, config table
- `03-VALIDATION.md` — validation checklist at repo root
- `agent-rs/.planning/` — nested GSD planning for the Rust workstream

## Naming Conventions

**Files:**
- `snake_case.rs` for Rust modules: `main.rs`, `tool_runtime.rs`, `web_research/`
- `snake_case.py` for Python modules: `start_server.py`, `model_install.py`
- `test_*.py` for Python tests (mirrors module name: `test_system_check.py`)
- `*.rs` (no prefix) for Rust integration tests in `agent-rs/tests/`: `tool_runtime_contracts.rs`, `eval_suite_security.rs`
- `PascalCase.tsx` for React components: `App.tsx`
- `UPPERCASE.md` for repo-root docs: `README.md`, `03-VALIDATION.md`

**Directories:**
- `snake_case` throughout: `agent_core/`, `web_research/`, `start_server.py`
- Submodule dirs group a domain: `agent_core/{diagnostics,repair,orchestration,web_research}`, `context/`, `security/`, `tui/`

**Special Patterns:**
- `HELIX_*` env var prefix: `HELIX_UI_MODE`, `HELIX_EXEC_MODE`, `HELIX_MODEL_NAME`, `HELIX_SERVER_PORT`, `HELIX_CONTEXT_SIZE`, `HELIX_GPU_LAYERS`
- `UPPER_CASE` Python constants in `scripts/config.py`: `GPU_LAYERS`, `CONTEXT_SIZE`, `SERVER_PORT`, `BACKEND_HINT`
- Rust code style: `PascalCase` structs/enums (`ToolRegistry`, `ContextEngine`, `ServerFlavor`, `RuntimeProfile`), `snake_case` fns/methods, `camelCase` JSON schema field names (`#[serde(rename_all = "camelCase")]` in `tools.rs`)

## Where to Add New Code

**New Tool (model-callable capability):**
- Implementation: implement `Tool` trait in `agent-rs/src/tools.rs` or `agent-rs/src/agent_core/repair/tools.rs`; register in `create_default_registry()` (`agent-rs/src/tools.rs`)
- Input schema: derive `JsonSchema` next to the tool input struct in `tools.rs`
- Tests: new integration test in `agent-rs/tests/` (e.g., follow `agent-rs/tests/tool_runtime_contracts.rs`)

**New Context Capability:**
- Implementation: `agent-rs/src/context/` (e.g., new submodule alongside `indexer.rs`/`memory.rs`), wired through `ContextEngine` in `agent-rs/src/context/mod.rs`
- Tests: `agent-rs/tests/context_integration.rs`

**New Security Rule:**
- Implementation: `agent-rs/src/security/policy.rs` (blocked patterns, `TrustLevel`, `evaluate_tool_call`)
- Tests: `agent-rs/tests/security_guardrails.rs`

**New UI/Feature in Web Mode:**
- Backend endpoint: `agent-rs/src/server.rs` (add route + handler to `start_web_server`)
- Frontend: `web-ui/src/` (component + `App.tsx`); new deps via `web-ui/package.json`

**New Python Helper / Config:**
- Implementation: `scripts/{name}.py`; constants in `scripts/config.py`; hardware logic in `scripts/system_check.py`
- Tests: `tests/test_{name}.py`

**New GSD Phase Logic:**
- Implementation: `agent-rs/src/agent_core/orchestration/` (phase handlers, artifacts)
- Tests: inline `#[cfg(test)]` in `agent-rs/src/agent_core/orchestration/mod.rs`

**New Rust Integration Test:**
- Location: `agent-rs/tests/{snake_case}.rs` (cargo auto-discovers)

**Utilities/Shared helpers:**
- Rust: `agent-rs/src/utils.rs`, `agent-rs/src/tokens.rs` (token counting)
- Python: `scripts/helix_branding.py` (logo/branding)

## Special Directories

**agent-rs/target/**
- Purpose: Rust build artifacts
- Source: `cargo build` / `cargo test`
- Committed: No (gitignored)

**llama.cpp/build/**
- Purpose: llama.cpp compiled binaries (llama-server)
- Source: `setup.py` build step
- Committed: No (gitignored)

**logs/**
- Purpose: Runtime logs (server stdout/stderr)
- Source: `start.py` file redirection
- Committed: No (gitignored)

**models/**
- Purpose: GGUF model files + `.staging/`
- Source: downloaded via `scripts/model_install.py` / `scripts/download_model.py`
- Committed: No (gitignored)

**.helix/**
- Purpose: Runtime SQLite state (`helix_context.db`) and repair backups
- Source: created by the Rust agent at startup
- Committed: No (runtime-generated)

**misc/**
- Purpose: Generated architecture diagram exports (`architecture_*.svg`)
- Source: written by `main.rs` from the context indexer DOT graph
- Committed: No (gitignored)

**agent-rs/.planning/**
- Purpose: Nested GSD planning for the Rust workstream (phases, config)
- Source: GSD tooling
- Committed: Yes (planning docs)

**web-ui/public/**
- Purpose: Static assets served by Vite (favicon, icons)
- Source: hand-written
- Committed: Yes

---

*Structure analysis: 2026-08-03*
*Update when directory structure changes*
