# Codebase Structure

**Analysis Date:** 2026-07-30

## Directory Layout

```
helix-cli/
├── agent-rs/              # Rust agent orchestrator (core binary + library)
│   ├── src/               # Rust source code
│   │   ├── main.rs        # Binary entry point — LLM loop, TUI/web/terminal dispatch
│   │   ├── lib.rs          # Library root — module declarations
│   │   ├── config.rs       # AppConfig: loads from Python via subprocess
│   │   ├── types.rs        # Core types: ChatMessage, ChatResponse, PermissionRequest, Provenance
│   │   ├── tools.rs        # Tool trait + ToolRegistry + 13 built-in tools
│   │   ├── stream.rs       # SSE parser for streaming LLM responses
│   │   ├── tokens.rs       # tiktoken-rs wrapper for token counting
│   │   ├── utils.rs        # Output cleaning, code block protection
│   │   ├── session.rs      # Session persistence (JSON files)
│   │   ├── watchdog.rs     # Server health monitor + restart backoff + PID management
│   │   ├── runtime_profile.rs  # CPU/GPU profile selection (Latency/Balanced/Safe)
│   │   ├── audit.rs        # Hash-chained tamper-evident SQLite audit log
│   │   ├── server.rs       # Axum REST API server (chat/health/status/tools/context)
│   │   ├── input.rs        # rustyline multi-line editor
│   │   ├── tui.rs           # Main TUI module — ratatui terminal UI
│   │   ├── tui/            # TUI sub-modules
│   │   │   ├── api.rs      # Type definitions (LayoutMode, ConnectionState, ChatMessage, channels)
│   │   │   ├── approval.rs # inquire-based permission approval prompts
│   │   │   ├── commands.rs # Command palette entries and filtering
│   │   │   ├── events.rs   # Keyboard event handling for command palette
│   │   │   ├── state.rs    # TuiState — all UI state, scrolling, tool timeline, themes
│   │   │   └── themes.rs   # Ratatui color theme definitions
│   │   ├── security/       # Security layer
│   │   │   ├── mod.rs      # Module declarations
│   │   │   ├── policy.rs   # PolicyEngine: allowlist, metacharacter, path traversal, injection detection
│   │   │   ├── capabilities.rs  # Capability model (read_only/guided_repair/autonomous)
│   │   │   └── sandbox.rs  # DockerSandbox: bollard-based container execution
│   │   ├── context/        # Context engine
│   │   │   ├── mod.rs      # ContextEngine facade
│   │   │   ├── indexer.rs  # Tree-sitter symbol indexer + SQLite cache + DOT graph export
│   │   │   ├── retrieval.rs # Exact/substring search + budget-bounded results
│   │   │   ├── memory.rs   # Durable SQLite memory (sessions, FTS5, edit ledger)
│   │   │   ├── skeleton.rs # Signature extraction with body elision
│   │   │   └── budget.rs   # tiktoken-rs token counting + budget enforcement
│   │   └── agent_core/     # Core agent subsystems
│   │       ├── mod.rs      # Module declarations
│   │       ├── tool_runtime.rs  # Tool execution lifecycle: policy → approval → sandbox → audit
│   │       ├── guardian.rs # Multi-specialist quorum voting system
│   │       ├── diagnostics/ # OS-level diagnostics
│   │       │   ├── mod.rs
│   │       │   ├── system.rs    # SystemProvider: processes, stats, services, network, secret redaction
│   │       │   ├── logs.rs      # LogProvider: Linux journalctl / Windows EVTX
│   │       │   └── reasoning.rs # DiagnosticEngine: Observe→Hypothesize→Test→Synthesize→Done
│   │       ├── repair/      # Self-healing — snapshots, rollback, scoring
│   │       │   ├── mod.rs
│   │       │   ├── snapshots.rs # SnapshotManager: tar.gz (Linux) / VSS (Windows)
│   │       │   ├── scoring.rs   # Bayesian confidence calculator
│   │       │   ├── tools.rs     # ServiceRepair, PackageRepair, PermissionRepair tools
│   │       │   └── workflow.rs  # SafetyLoop: snapshot → execute → validate → rollback
│   │       ├── web_research/ # Live web research pipeline
│   │       │   ├── mod.rs       # WebResearchPipeline facade
│   │       │   ├── classifier.rs # FreshnessClassifier (keyword/regex, TTL)
│   │       │   ├── planner.rs   # ResearchPlanner (crate + DuckDuckGo search)
│   │       │   ├── worker.rs    # WorkerPool (concurrent fetch, SSRF, rate limiting)
│   │       │   ├── store.rs     # EvidenceStore (SQLite sources/evidence/citations/freshness)
│   │       │   ├── sanitize.rs  # HTML→markdown sanitization + SSRF safety
│   │       │   └── synthesizer.rs # EvidenceSynthesizer (budget-bounded brief)
│   │       └── orchestration/ # GSD phase orchestration
│   │           ├── mod.rs       # advance_phase() — phase state machine entry point
│   │           ├── phase_state.rs # Phase enum (Discover→Discuss→Plan→Execute→Verify→Close)
│   │           ├── artifacts.rs # PhaseArtifact persistence to .planning/phases/
│   │           ├── context_reset.rs # ContextResetter: rebuild prompts after reset
│   │           └── recovery.rs # Phase recovery logic
│   └── tests/              # Integration tests (Rust)
│       ├── audit_log_mvp.rs
│       ├── context_integration.rs
│       ├── diagnostic_validation.rs
│       ├── gsd_orchestration_validation.rs
│       ├── os_diagnostics_integration.rs
│       ├── plugin_sdk_ide_bridge_validation.rs
│       ├── runtime_profile_watchdog.rs
│       ├── runtime_profile_watchdog_validation.rs
│       ├── security_guardrails.rs
│       ├── streaming_tui_refinement.rs
│       ├── test_secure_execution.rs
│       ├── test_session_persistence.rs
│       ├── tool_execution.rs
│       ├── tool_runtime_contracts.rs
│       └── web_research_adversarial.rs
├── scripts/               # Python scripts
│   ├── config.py           # Generated config (model path, GPU layers, server params)
│   ├── start_server.py     # LLM server launcher (llama.cpp / KoboldCPP)
│   ├── start_server.sh     # Shell wrapper for start_server.py
│   ├── start_agent.sh      # Shell wrapper for start.py
│   ├── system_check.py     # Hardware detection (CPU/GPU/Vulkan/OpenVINO)
│   ├── model_install.py    # HuggingFace model download with verification
│   ├── download_model.py   # Direct model download utility
│   ├── helix_branding.py   # ASCII logo / branding
│   ├── helix.py            # Additional Helix utility script
│   ├── onboarding_profile.py # User preference persistence
│   └── build_zip.py        # Build packaging
├── tests/                  # Python tests
│   ├── test_system_check.py
│   ├── test_start_server_runtime_profile.py
│   ├── test_start_server_runtime_profile_validation.py
│   ├── test_qwen_config.py
│   ├── test_onboarding_profile.py
│   ├── test_model_install.py
│   ├── test_model_discovery.py
│   ├── test_download_model.py
│   ├── test_accuracy.py
│   └── eval.py             # Agentic benchmark evaluation suite
├── web-ui/                 # Frontend web application (Vite + React/TypeScript)
│   ├── src/                # Source code
│   ├── public/             # Static assets
│   ├── index.html          # HTML entry point
│   ├── vite.config.ts      # Vite configuration
│   ├── tsconfig.json       # TypeScript configuration
│   ├── tailwind.config.js  # Tailwind CSS configuration
│   ├── postcss.config.js   # PostCSS configuration
│   ├── eslint.config.js    # ESLint configuration
│   └── package.json        # NPM dependencies
├── setup.py                # Phase 9 unified installer: hardware detection, model download, llama.cpp build, benchmarks
├── start.py                # Interactive launcher: model/interface/mode selection, server boot, Rust launch
├── apply_fix.py            # Auto-fix application script
├── 03-VALIDATION.md        # Validation documentation
├── llama.cpp/              # llama.cpp source (cloned during setup, may be absent in source)
├── logs/                   # Runtime logs (start_server.{stdout,stderr}.log)
├── models/                 # Downloaded GGUF model files
├── .planning/              # GSD planning artifacts
│   ├── ROADMAP.md          # Phase roadmap
│   ├── PROJECT.md          # Project overview
│   ├── REQUIREMENTS.md     # Requirements
│   ├── STATE.md            # Project state
│   ├── config.json         # GSD configuration
│   ├── codebase/           # Codebase analysis documents (this directory)
│   ├── phases/             # Phase artifacts
│   └── research/           # Research documents
├── misc/                   # Miscellaneous (architecture DOT graphs)
├── .gitignore
└── README.md
```

## Directory Purposes

**`agent-rs/` — Rust Agent Orchestrator:**
- Purpose: Core agent binary and library — LLM orchestration, tool execution, security, context management, diagnostics, repair, web research, UI
- Contains: Rust source (`src/`), tests (`tests/`), Cargo build files
- Key files:
  - `Cargo.toml`: 64+ dependencies including async-openai, axum, ratatui, tree-sitter, bollard, rusqlite, tiktoken-rs
  - `src/main.rs`: Binary entry point (1100+ lines)
  - `src/lib.rs`: Library root, module re-exports
- Configuration: Environment variables (`HELIX_*`, `AGENT_PERSONA`, `GPU_LAYERS`)

**`scripts/` — Python Utilities:**
- Purpose: Environment setup, config generation, model management, server boot, branding
- Contains: Python scripts invoked by user and by Rust config bridge
- Key files:
  - `config.py`: Auto-generated at setup; read by Rust via subprocess
  - `start_server.py`: Server launcher for llama.cpp or KoboldCPP
  - `system_check.py`: Hardware detection for CPU/GPU/Vulkan/OpenVINO
  - `model_install.py`: HuggingFace model download with hash verification
- Configuration: `config.py` values overrideable via `HELIX_*` environment variables

**`web-ui/` — Frontend UI:**
- Purpose: Browser-based UI for the Helix agent (Vite + React + TypeScript)
- Contains: React components, Vite build config, Tailwind CSS
- Key files: `vite.config.ts`, `package.json`
- Backend: Communicates with the Axum REST API at `http://127.0.0.1:3000`

**`tests/` — Python Tests:**
- Purpose: Unit tests and the agentic benchmark evaluation suite
- Key file: `eval.py` — runs tool-calling accuracy benchmarks against the agent binary

## Key File Locations

**Entry Points:**
- `setup.py`: Full system setup (hardware detection → model download → llama.cpp build → benchmarks → config generation)
- `start.py`: Quick launcher (model selection → server boot → Rust orchestrator)
- `agent-rs/src/main.rs`: Rust binary `helix-agent` — all runtime logic

**Configuration:**
- `scripts/config.py`: Runtime config (auto-generated by `setup.py`). Values: `MODEL_NAME`, `MODEL_PATH`, `GPU_LAYERS`, `CONTEXT_SIZE`, `BACKEND_HINT`, `SERVER_PORT`, `DANGEROUS_COMMANDS`, `AUDIT_ENABLED`
- `agent-rs/src/config.rs`: Rust-side config loading via Python subprocess bridge

**Core Logic (Rust):**
- `agent-rs/src/main.rs`: Main LLM loop, TUI/web/terminal dispatch, GSD slash commands, server flavor detection, watchdog integration
- `agent-rs/src/tools.rs`: Tool trait, ToolRegistry, 13 built-in tool implementations
- `agent-rs/src/agent_core/tool_runtime.rs`: Centralized tool execution with policy, sandbox, audit, lifecycle events
- `agent-rs/src/security/policy.rs`: Command validation, permission tiers, injection detection
- `agent-rs/src/context/indexer.rs`: Tree-sitter-based symbol extraction and SQLite caching
- `agent-rs/src/audit.rs`: Hash-chained tamper-evident event logging
- `agent-rs/src/agent_core/web_research/mod.rs`: Web research pipeline facade

**Testing:**
- `agent-rs/tests/`: 16 integration test files covering audit, context, diagnostics, GSD orchestration, OS diagnostics, runtime profile, security, streaming TUI, secure execution, sessions, tools, tool runtime contracts, and web research
- `tests/`: 10 Python test files covering system check, server profile, model install, download, accuracy, and eval

**Utilities:**
- `agent-rs/src/tokens.rs`: Token counting via `tiktoken-rs` (cl100k_base)
- `agent-rs/src/stream.rs`: SSE event parser for streaming LLM responses
- `agent-rs/src/session.rs`: JSON-serialized session persistence with atomic writes
- `agent-rs/src/utils.rs`: Chat output cleaning (code block protection, thinking tag handling)

## Naming Conventions

**Files:**
- Rust source files: `snake_case.rs` (e.g., `tool_runtime.rs`, `phase_state.rs`)
- Python source files: `snake_case.py` (e.g., `start_server.py`, `model_install.py`)
- Test files: `test_<module>.py` for Python, `<feature>_<aspect>.rs` for Rust integration tests
- Config files: `config.py` (Python), `Cargo.toml` (Rust)

**Directories:**
- Rust modules: `snake_case/` (e.g., `security/`, `context/`, `agent_core/`)
- TUI sub-modules: Flat files in `tui/` directory (e.g., `api.rs`, `state.rs`, `events.rs`)
- Agent core subsystems: `snake_case/` with `mod.rs` declaration files (e.g., `diagnostics/`, `repair/`, `web_research/`)
- Top-level dirs: `kebab-case` for non-code dirs (`web-ui/`, `agent-rs/`)

**Rust Names:**
- Structs: `UpperCamelCase` (e.g., `ToolRuntime`, `PolicyEngine`, `AuditStore`, `ContextEngine`)
- Traits: `UpperCamelCase` (e.g., `Tool`, `PermissionRequester`, `LogProvider`)
- Functions: `snake_case` (e.g., `evaluate_tool_call`, `generate_tool_grammar`, `enforce_sandbox`)
- Enums: `UpperCamelCase` (e.g., `PermissionTier`, `RiskLevel`, `ServerFlavor`, `Provenance`)
- Type aliases: `UpperCamelCase` (e.g., `ActionSender`, `EventReceiver`)

**Python Names:**
- Functions: `snake_case` (e.g., `build_llama_cpp`, `enforce_token_speed`, `generate_config`)
- Classes: `UpperCamelCase` (not used in the Python scripts — all module-level functions)
- Constants: `UPPER_SNAKE_CASE` (e.g., `GPU_LAYERS`, `CONTEXT_SIZE`, `DANGEROUS_COMMANDS`)

## Where to Add New Code

**New Tool:**
1. Define input struct in `agent-rs/src/tools.rs` (e.g., `struct MyToolInput`)
2. Implement `Tool` trait for the tool struct in `agent-rs/src/tools.rs`
3. Register it in `create_default_registry()` in `agent-rs/src/tools.rs`
4. Add persona filters in `build_tools_payload()` if needed
5. Update `required_capabilities()` in `agent-rs/src/security/capabilities.rs`
6. Update `tool_risk_level()` in `agent-rs/src/security/policy.rs`
7. Add tests in a new `#[cfg(test)] mod tests` block in the same file
8. Add integration test in `agent-rs/tests/tool_execution.rs`
9. If transactional, label `is_transactional() -> true` and add rollback handling in `tool_runtime.rs`

**New Context Provider:**
1. Add module under `agent-rs/src/context/` (e.g., `embeddings.rs`)
2. Add `pub mod` in `agent-rs/src/context/mod.rs`
3. If it needs SQLite, open connection in `ContextEngine::initialize()`
4. Wire it into `ContextEngine` struct and expose methods in `context/mod.rs`

**New Agent Core Subsystem:**
1. Create subdirectory under `agent-rs/src/agent_core/` (e.g., `planning/`)
2. Add `pub mod planning;` to `agent-rs/src/agent_core/mod.rs`
3. Follow the module pattern: `mod.rs` for facade, separate files for concerns
4. Add integration tests in `agent-rs/tests/`

**New UI Component (TUI):**
1. Add logic to the appropriate `tui/` sub-file: `api.rs` for types, `state.rs` for state management, `events.rs` for input handling, `themes.rs` for styling
2. For new widgets, add rendering code in the `render_*` section of `tui.rs`
3. Add new `TuiAction` variants to the enum in `tui.rs`

**New API Endpoint (Web Server):**
1. Add handler function in `agent-rs/src/server.rs`
2. Add route to the `Router` in `start_web_server()`
3. Add new types to `AppState` if needed

**New Security Policy:**
1. Modify `agent-rs/src/security/policy.rs`
2. Add new `RiskLevel` variant if needed
3. Add new rules in `evaluate_tool_call()` or `evaluate_command_risk()`
4. Add new blocked command patterns to `BLOCKED_COMMAND_PATTERNS`
5. Add new allowlist entries to `ALLOWLIST`
6. Update `is_medium_risk_command()` as needed
7. Write tests

**Python-side Changes:**
- Config changes: modify `scripts/config.py` (or the generation logic in `setup.py`)
- New server provider: add in `scripts/start_server.py`
- New hardware detection: add in `scripts/system_check.py`

## Special Directories

**`.helix/`:**
- Purpose: Runtime data directory created at the workspace root
- Contains: SQLite databases (`helix_context.db`), backups, PID file (`server.pid`), sessions
- Generated: Yes, at runtime
- Committed: No

**`llama.cpp/`:**
- Purpose: llama.cpp source code (cloned from GitHub during setup)
- Generated: Yes, by `setup.py` via `git clone`
- Committed: No (listed in `.gitignore`)

**`models/`:**
- Purpose: Downloaded GGUF model files
- Generated: Yes, by `setup.py` or direct download
- Committed: No

**`logs/`:**
- Purpose: Runtime log output from the LLM server and agent
- Generated: Yes, at runtime
- Committed: No

**`misc/`:**
- Purpose: Auto-exported architecture visualizations (DOT graphs from Tree-sitter import analysis)
- Generated: Yes, at startup
- Committed: Optional — useful for documentation

**`.planning/`:**
- Purpose: GSD (Goal-oriented Software Development) planning artifacts
- Key files: `ROADMAP.md` (phase plan), `PROJECT.md` (project charter), `STATE.md` (current state), `REQUIREMENTS.md` (requirements doc)
- Generated: Manually via GSD workflow
- Committed: Yes — part of the project's planning documentation

---

*Structure analysis: 2026-07-30*
