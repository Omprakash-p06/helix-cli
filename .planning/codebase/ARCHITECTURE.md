<!-- refreshed: 2026-07-30 -->
# Architecture

**Analysis Date:** 2026-07-30

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                             PYTHON LAYER                                     │
│  setup.py  start.py  scripts/start_server.py  scripts/config.py              │
│             Hardware detection, LLM server boot, config generation           │
└──────────────────────────────────────────────────┬──────────────────────────┘
                                                   │ spawns
                                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RUST AGENT ORCHESTRATOR (agent-rs)                    │
│                                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Config Load  │  │  LLM Client  │  │  Tool Engine  │  │  Watchdog/Audit  │  │
│  │  `config.rs`  │  │  (reqwest)   │  │  `tools.rs`   │  │  `watchdog.rs`   │  │
│  │               │  │  Streaming   │  │  `tool_runtime│  │  `audit.rs`      │  │
│  │  Loads config │  │  SSE parser  │  │  .rs`         │  │                  │  │
│  │  via Python   │  │  `stream.rs` │  │               │  │  Tamper-evident  │  │
│  │  subprocess   │  │              │  │  12 built-in   │  │  audit log       │  │
│  └──────┬────────┘  └──────┬───────┘  │  tools        │  └──────────────────┘  │
│         │                  │          └──────┬────────┘                        │
│         ▼                  ▼                 ▼                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                        SECURITY LAYER                                   │  │
│  │  `security/policy.rs`  `security/capabilities.rs`  `security/sandbox.rs` │  │
│  │  PermissionTier (ReadOnly/WorkspaceWrite/FullExec)                       │  │
│  │  PolicyEngine (allowlist + metacharacter + path traversal blocking)      │  │
│  │  DockerSandbox (bollard) — alpine container, no network, read-only rootfs│  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                           │                                                   │
│                           ▼                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                        CONTEXT ENGINE                                    │  │
│  │  `context/`                                                              │  │
│  │  ├── Indexer   (Tree-sitter → SQLite symbol cache)                      │  │
│  │  ├── Retrieval (Exact + FTS5 + Budget-bounded results)                   │  │
│  │  ├── Memory    (SQLite sessions/memory/edit_ledger + FTS5)              │  │
│  │  ├── Skeleton  (Signature extraction, body elision)                     │  │
│  │  └── Budget    (tiktoken-rs token counting)                              │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                           │                                                   │
│                           ▼                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                        AGENT CORE                                       │  │
│  │  `agent_core/`                                                          │  │
│  │  ├── Guardian     (multi-specialist quorum voting)                      │  │
│  │  ├── Diagnostics  (OS-level: system, logs, Bayesian reasoning)          │  │
│  │  ├── Repair       (snapshots, rollback, scoring, safety loop)           │  │
│  │  ├── Web Research (classifier→planner→worker pool→evidence→synthesizer) │  │
│  │  └── Orchestration (GSD phase state machine, artifacts, recovery)       │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                           │                                                   │
│         ┌─────────────────┼─────────────────────┐                             │
│         ▼                 ▼                     ▼                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐                    │
│  │  TUI (ratatui)│  │  Web Server  │  │  Terminal REPL   │                    │
│  │  `tui.rs`     │  │  `server.rs` │  │  `input.rs`      │                    │
│  │  Streaming,   │  │  Axum REST   │  │  rustyline        │                    │
│  │  command      │  │  /chat (SSE) │  │  multi-line       │                    │
│  │  palette,     │  │  /health     │  │  input            │                    │
│  │  themes,      │  │  /v1/status  │  │                   │                    │
│  │  approval UI  │  │  /v1/tools   │  │                   │                    │
│  └──────────────┘  │  /v1/context │  └──────────────────┘                    │
│                    └──────────────┘                                          │
└─────────────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       LLM BACKEND (OpenAI-compatible)                        │
│  llama.cpp server  or  KoboldCPP  —  /v1/chat/completions endpoint           │
└─────────────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           EXTERNAL SYSTEMS                                   │
│  Docker daemon (sandbox)  —  SQLite DBs (.helix/)  —  HuggingFace (models)   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | Key Files |
|-----------|----------------|-----------|
| Python Config/Loader | Hardware detection, model download, llama.cpp build, config generation | `setup.py`, `scripts/config.py`, `scripts/system_check.py`, `scripts/model_install.py` |
| Python Launcher | Boot LLM server, wait for readiness, launch Rust binary | `start.py`, `scripts/start_server.py` |
| Config Bridge | Execute Python subprocess to load config as JSON | `agent-rs/src/config.rs` |
| LLM Client | HTTP client to OpenAI-compatible endpoint, streaming SSE parser | `agent-rs/src/main.rs`, `agent-rs/src/stream.rs` |
| Tool Registry | 13 built-in tools: terminal, file R/W, list dir, code search, system stats, process/service/logs, repair tools | `agent-rs/src/tools.rs` |
| Tool Runtime | Execute tools with policy check, sandbox, audit, rollback lifecycle | `agent-rs/src/agent_core/tool_runtime.rs` |
| Policy Engine | Command allowlist, metacharacter blocking, path traversal prevention, prompt injection detection | `agent-rs/src/security/policy.rs` |
| Capability Model | Map tool names to required capabilities; CapabilitySet (read_only/guided_repair/autonomous) | `agent-rs/src/security/capabilities.rs` |
| Docker Sandbox | bollard-based Docker container execution with no network, read-only rootfs, ALL cap_drop | `agent-rs/src/security/sandbox.rs` |
| Context Indexer | Tree-sitter Rust parsing → SQLite symbol cache with SHA-256 incremental invalidation | `agent-rs/src/context/indexer.rs` |
| Context Retrieval | Exact + LIKE symbol search, budget-bounded (tiktoken) result assembly | `agent-rs/src/context/retrieval.rs` |
| Durable Memory | SQLite sessions, memory (FTS5 searchable), edit_ledger; survives compaction cycles | `agent-rs/src/context/memory.rs` |
| Skeleton | Function signature extraction with body elision to `{ /* ... */ }` | `agent-rs/src/context/skeleton.rs` |
| Token Budget | tiktoken-rs cl100k_base token counting with ceiling enforcement | `agent-rs/src/context/budget.rs` |
| Context Engine Facade | Orchestrates Indexer + Retrieval + Memory + Web Research | `agent-rs/src/context/mod.rs` |
| Diagnostics System | Process listing, system stats, network interfaces, service status, secret redaction | `agent-rs/src/agent_core/diagnostics/system.rs` |
| Diagnostics Logs | Linux journalctl / Windows EVTX log retrieval | `agent-rs/src/agent_core/diagnostics/logs.rs` |
| Diagnostics Reasoning | State machine (Observe→Hypothesize→Test→Synthesize→Done), Bayesian confidence scoring | `agent-rs/src/agent_core/diagnostics/reasoning.rs` |
| Repair Snapshots | tar.gz (Linux) / VSS (Windows) snapshot create/restore | `agent-rs/src/agent_core/repair/snapshots.rs` |
| Repair Safety Loop | Snapshot → execute → validate → rollback transactional pattern | `agent-rs/src/agent_core/repair/workflow.rs` |
| Repair Tools | Service management, package management, permission repair | `agent-rs/src/agent_core/repair/tools.rs` |
| Guardian | Multi-specialist quorum voting (Allow/Deny/Abstain) with configurable thresholds | `agent-rs/src/agent_core/guardian.rs` |
| Web Research Classifier | Keyword/regex-based freshness check, configurable TTL | `agent-rs/src/agent_core/web_research/classifier.rs` |
| Web Research Planner | Decompose query → DuckDuckGo + docs.rs fetch tasks | `agent-rs/src/agent_core/web_research/planner.rs` |
| Web Research Worker | Concurrent fetch pool with SSRF protection, rate limiting (governor) | `agent-rs/src/agent_core/web_research/worker.rs` |
| Web Research Store | SQLite-backed sources, evidence, citations, freshness cache | `agent-rs/src/agent_core/web_research/store.rs` |
| Web Research Synthesizer | Budget-bounded (~1500 tok) brief assembly with citations | `agent-rs/src/agent_core/web_research/synthesizer.rs` |
| GSD Orchestration | Phase state machine (Discover→Discuss→Plan→Execute→Verify→Close), artifacts, recovery | `agent-rs/src/agent_core/orchestration/` |
| Audit | Hash-chained (SHA-256) tamper-evident SQLite audit log | `agent-rs/src/audit.rs` |
| Watchdog | Server health monitoring with restart budget, exponential backoff | `agent-rs/src/watchdog.rs` |
| TUI | ratatui-based streaming terminal UI, command palette, themes (Dark/Light/Nord/Gruvbox) | `agent-rs/src/tui.rs`, `agent-rs/src/tui/` |
| Web Server | Axum REST API with SSE streaming, CORS | `agent-rs/src/server.rs` |
| Session | JSON-based session persistence with atomic write | `agent-rs/src/session.rs` |

## Pattern Overview

**Overall:** Hybrid Python/Rust layered agent architecture. Python handles environment setup (hardware detection, model download, build). Rust handles the agent runtime — LLM orchestration, tool execution, policy enforcement, context management, diagnostics, and UI.

The agent operates in a **tool-calling loop** against an OpenAI-compatible LLM backend:
1. Send messages + tool definitions to the LLM
2. Parse the response for tool calls
3. Execute tools through the ToolRuntime (with policy/sandbox/audit)
4. Feed results back to the LLM
5. Repeat until a final text response is produced

**Key Characteristics:**
- **Defense-in-depth security**: PolicyEngine (allowlist + metacharacter + path traversal + injection checks) + DockerSandbox + Capability tiers
- **Symbol-aware context**: Tree-sitter-based indexing replaces naive chunking, with incremental re-indexing via SHA-256 hashing
- **Durable memory survives compaction**: SQLite-backed MemoryEngine persists goals, constraints, decisions, and edit history across context resets
- **Tamper-evident audit**: Hash-chained (SHA-256) audit log with chain verification
- **Self-healing**: Watchdog with restart backoff; SafetyLoop with snapshot/rollback; DiagnosticEngine for root-cause analysis
- **Provenance tracking**: ContentSource enum (Workspace/System/Research/Untrusted) prevents untrusted data from reaching sensitive context positions
- **Web research pipeline**: FreshnessClassifier → ResearchPlanner → WorkerPool → EvidenceStore → EvidenceSynthesizer

## Layers

**Python Layer:**
- Purpose: Environment setup, hardware detection, model download, LLM server boot, config generation
- Location: `setup.py`, `start.py`, `scripts/`
- Contains: Hardware detection, package management, llama.cpp build, server launcher, interactive setup
- Depends on: pip packages (requests, tqdm, huggingface_hub, openai), Rust toolchain, cmake
- Used by: User (setup entry point), Rust config bridge (config values loaded via subprocess)

**Rust Entry Point:**
- Purpose: Load config from Python, initialize all subsystems, enter UI/REPL loop
- Location: `agent-rs/src/main.rs`
- Contains: Server flavor detection, model readiness probing, watchdog state machine, multi-mode dispatch (TUI/web/terminal), GSD slash commands, main LLM request/response loop
- Depends on: All other Rust modules
- Used by: Direct invocation or via `start.py`

**Config Layer:**
- Purpose: Bridge Python config to Rust via subprocess, define AppConfig struct
- Location: `agent-rs/src/config.rs`
- Contains: `AppConfig` struct, `load_from_python()` which runs Python inline script to extract config values as JSON
- Depends on: Python subprocess execution
- Used by: `main.rs`, `server.rs`

**Security Layer:**
- Purpose: Multi-layer tool call security — permission tiers, command validation, capability enforcement, Docker sandbox
- Location: `agent-rs/src/security/`
- Contains: `policy.rs` (PolicyEngine, evaluate_tool_call, RiskLevel, PermissionTier, TrustLevel, command allowlist, prompt injection detection), `capabilities.rs` (CapabilitySet: read_only/guided_repair/autonomous), `sandbox.rs` (DockerSandbox with bollard)
- Depends on: path-security, shell-sanitize, soft-canonicalize, shell-words, bollard, regex
- Used by: `tool_runtime.rs`, `tools.rs`

**Context Engine:**
- Purpose: Symbol-aware, budget-bounded context retrieval with durable memory
- Location: `agent-rs/src/context/`
- Contains: `indexer.rs` (Tree-sitter parser, SQLite symbol cache, incremental invalidation), `retrieval.rs` (exact+LIMIT+substring search, budget enforcement), `memory.rs` (SQLite sessions, FTS5 memory search, edit ledger), `skeleton.rs` (body elision), `budget.rs` (tiktoken-rs wrapper), `mod.rs` (ContextEngine facade)
- Depends on: tree-sitter, tree-sitter-rust, rusqlite, sha2, tiktoken-rs, ignore
- Used by: `main.rs`, `tools.rs` (search_codebase tool)

**Tool Runtime:**
- Purpose: Execute tool calls with full lifecycle: policy check → approval → sandbox → audit → result
- Location: `agent-rs/src/agent_core/tool_runtime.rs`
- Contains: `ToolRuntime` struct, `execute()` async method, `execute_sync()` blocking method, DockerSandbox dispatch for `run_terminal_command`, SafetyLoop-wrapped transactional tool execution
- Depends on: security/ (policy + sandbox), tools/ (ToolRegistry), audit/, repair/ (SafetyLoop)
- Used by: `main.rs`, `server.rs`, `orchestration/mod.rs`

**Agent Core Subsystems:**
- Guardian: Multi-specialist quorum voting (Allow/Deny/Abstain) with configurable thresholds per RiskLevel
- Diagnostics: SystemProvider (processes/stats/services/logs) + DiagnosticEngine (Observe→Hypothesize→Test→Synthesize→Done state machine) with Bayesian confidence
- Repair: SnapshotManager (tar.gz/VSS) + SafetyLoop (snapshot→execute→validate→rollback) + Service/Package/Permission repair tools
- Web Research: FreshnessClassifier (rule-based TTL) → ResearchPlanner (crate+DuckDuckGo) → WorkerPool (concurrent fetch, SSRF protection, rate limiting) → EvidenceStore (SQLite) → EvidenceSynthesizer (brief assembly)
- Orchestration: Phase state machine (Discover→Discuss→Plan→Execute→Verify→Close) with artifact persistence and context reset

**TUI Layer:**
- Purpose: Rich terminal UI with streaming responses, command palette, multiple themes
- Location: `agent-rs/src/tui.rs` + `agent-rs/src/tui/`
- Contains: Main TUI event loop, state management (TuiState), command palette with filtering, approval prompts (inquire), themes (Dark/Light/Nord/Gruvbox), sidebar with context files + tool timeline
- Depends on: ratatui, crossterm, tui-input, throbber-widgets-tui, tachyonfx, inquire
- Used by: `main.rs` (when `HELIX_UI_MODE=tui`)

**Web Server Layer:**
- Purpose: Axum-based REST API for browser UI
- Location: `agent-rs/src/server.rs`
- Contains: POST `/chat` (SSE streaming), GET `/health`, GET `/v1/status`, GET `/v1/tools`, GET `/v1/context`
- Depends on: axum, tower-http (CORS), tokio-stream
- Used by: `main.rs` (when `HELIX_UI_MODE=web`), `web-ui/` (Vite frontend)

**Audit:**
- Purpose: Tamper-evident event logging with hash chain verification
- Location: `agent-rs/src/audit.rs`
- Contains: `AuditEvent` struct, `AuditStore` (SQLite-backed), `append_event()` (computes SHA-256 chain hash), `query_events()` (filters by time/path/tool/decision/outcome), `verify_chain()` (replays and validates entire hash chain)
- Depends on: rusqlite, sha2
- Used by: `tool_runtime.rs`, `diagnostics/reasoning.rs`, `main.rs`

## Data Flow

### Primary Request Path (TUI Mode)

1. **User Input** → `tui.rs` captures keystrokes via crossterm, assembles text in tui-input buffer
2. **Submit** → `TuiAction::Submit(text)` sent via mpsc channel to main event loop in `main.rs`
3. **Message Build** → `ChatMessage { role: "user", content: text }` pushed to message vec
4. **LLM Request** → `run_llm_loop_tui()` sends POST to `{base_url}/v1/chat/completions` with messages + tools + grammar
5. **Stream Parse** → `stream.rs::SseParser` decodes SSE `data:` lines into JSON chunks
6. **Tool Call Detection** → If response contains `tool_calls`, extract function name + arguments
7. **Policy Evaluation** → `tool_runtime.rs` calls `evaluate_tool_call()` → `policy.rs` checks PermissionTier, command allowlist, metacharacters, path traversal, prompt injection
8. **Approval** → If `RequireApproval`, `InquirePermissionRequester` prompts user via TUI dialog
9. **Sandbox Execution** → `run_terminal_command` → `DockerSandbox::run_command()` (alpine container)
10. **Audit** → `AuditStore::append_event()` records decision + outcome with hash chaining
11. **Tool Reply** → Tool result pushed as `ChatMessage { role: "tool" }` — if failed, critic injection adds corrective directive
12. **Loop** → Steps 4–11 repeat (max 20 rounds) until final text response
13. **Display** → Text response sent as `TuiEvent::StreamChunk` for real-time rendering

### Web Research Pipeline

1. **Trigger** → `ContextEngine::enrich_with_research()` called with user query
2. **Freshness Check** → `FreshnessClassifier::needs_live_search()` checks keywords (latest/new/deprecated/cve) + cache TTL
3. **Planning** → `ResearchPlanner::plan()` generates DuckDuckGo search + docs.rs URLs
4. **Fetch** → `WorkerPool::run()` spawns N concurrent workers, each respecting rate limiter (2 req/s) and SSRF block
5. **Store** → Fetched HTML → `sanitize_html_to_markdown()` → stored in `EvidenceStore` (SQLite)
6. **Synthesize** → `EvidenceSynthesizer::brief_from_store()` builds `ResearchBrief` capped at ~1500 tokens
7. **Inject** → Brief formatted as `ResearchBrief::to_context_string()` for LLM context

### Repair Safety Loop

1. **Pre-snapshot** → `SnapshotManager::create_snapshot()` (tar.gz on Linux, VSS on Windows)
2. **Execute** → Transactional tool (service_repair, package_repair, permission_repair) runs
3. **Validate** → Validation function checks result success
4. **Commit or Rollback** → On failure: `SnapshotManager::restore_snapshot()` reverts changes

### Audit Hash Chain

1. Each `append_event()` computes: `SHA-256(timestamp || actor || path || event_type || tool_name || decision || outcome || reason || remediation || args_hash || output_hash || duration_ms || prev_hash)`
2. `prev_hash` is the `event_hash` of the previous row (or 64 zeros for genesis)
3. `verify_chain()` replays all events from genesis, recomputes hashes, and checks chain integrity

**State Management:**
- **Message state**: In-memory `Vec<ChatMessage>` with compaction at 70% context size
- **Durable session**: `session.rs` JSON files at `~/.helix/sessions/session.latest.json`
- **Durable memory**: `context/memory.rs` SQLite with FTS5 (survives context resets)
- **Symbol index**: `context/indexer.rs` SQLite with SHA-256 incremental invalidation
- **Web research evidence**: `web_research/store.rs` SQLite
- **Audit log**: `audit.rs` SQLite with SHA-256 hash chain
- **Config**: Generated `scripts/config.py` loaded by Rust via subprocess

## Key Abstractions

**Tool Trait:**
- Purpose: Pluggable tool definition with schema, execution, and transactional support
- Examples: `RunTerminalCommandTool`, `ReadFileTool`, `WriteFileTool`, `ServiceRepairTool` in `agent-rs/src/tools.rs` and `agent-rs/src/agent_core/repair/tools.rs`
- Pattern: `trait Tool: Send + Sync { fn name() -> String; fn description() -> String; fn schema() -> Value; fn execute(...) -> ToolResult; fn is_transactional() -> bool }`

**ToolRuntime:**
- Purpose: Central executor for all tool calls with policy approval, Docker sandboxing, audit logging, and SafetyLoop integration
- File: `agent-rs/src/agent_core/tool_runtime.rs`
- Pattern: Holds `PermissionRequester` (for HITL approval) and `SafetyLoop` (for transactional rollback); wraps blocking execution in `spawn_blocking` with 30s timeout

**PolicyEngine:**
- Purpose: Validate and sanitize shell commands against allowlist, metacharacter, path traversal, and injection rules
- File: `agent-rs/src/security/policy.rs`
- Pattern: Chain of validators — metacharacter check → shell-word parse → blocked command check → dangerous command check → allowlist check → path canonicalization → security context

**AuditStore:**
- Purpose: Tamper-evident append-only SQLite event log with hash chain
- File: `agent-rs/src/audit.rs`
- Pattern: Each event includes `prev_hash` pointing to previous event's SHA-256 hash; `verify_chain()` replays entire log for integrity

**ContextEngine:**
- Purpose: Facade over Indexer + Retrieval + Memory + Web Research
- File: `agent-rs/src/context/mod.rs`
- Pattern: Lazy initialization — `new()` creates shell, `initialize()` opens SQLite connections and builds index; exposes `build_context()`, `build_repo_skeleton()`, `enrich_with_research()`

**Guardian:**
- Purpose: Multi-specialist quorum voting system for tool call safety
- File: `agent-rs/src/agent_core/guardian.rs`
- Pattern: Spawns N specialist simulacra (futures), each votes Allow/Deny/Abstain; quorum check applies risk-level thresholds (Critical=100%, High=75%, Medium=51%, Low=0%)

**DiagnosticEngine:**
- Purpose: Systematic root-cause analysis with evidence collection and hypothesis testing
- File: `agent-rs/src/agent_core/diagnostics/reasoning.rs`
- Pattern: Finite state machine (Observe→Hypothesize→Test→Synthesize→Done) with Bayesian confidence scoring (`calculate_confidence(token_probs, reliability, evidence_coverage)`)

## Entry Points

**Python Setup:**
- Location: `setup.py`
- Triggers: User runs `python setup.py`
- Responsibilities: Hardware detection (CPU/GPU/Vulkan/OpenVINO), model download (HuggingFace), llama.cpp build, token speed benchmark (≥10 tok/s gate), optional agentic benchmark preflight, configuration generation

**Python Launcher:**
- Location: `start.py`
- Triggers: User runs `python start.py`
- Responsibilities: Interactive model/interface/mode selection, orphan server cleanup, LLM server boot, readiness wait, Rust binary launch, stack teardown

**Rust Binary:**
- Location: `agent-rs/src/main.rs` (produces `helix-agent` binary)
- Triggers: Direct `cargo run` or via `start.py`
- Responsibilities: Config load, runtime profile selection, watchdog init, tool registry init, context engine init, audit store init, dispatch to TUI/web/terminal mode, LLM request loop, tool execution loop

**Rust Library:**
- Location: `agent-rs/src/lib.rs`
- Triggers: Used by main.rs and tests; publishes `pub mod` declarations for all subsystems
- Responsibilities: Re-exports key types (`ChatMessage`, `ChatResponse`, `ServerFlavor`), utility functions (`critic_message`, `expose_think_blocks`)

## Architectural Constraints

- **Threading:** Tokio async runtime (single-threaded by default) for main event loop; blocking tool execution is dispatched via `spawn_blocking` with 30s timeout; blocking permission requests use `Handle::block_on` to bridge async/sync boundary
- **Global state:** `TUI_HTTP_CONNECT_FAILS` atomic counter in `main.rs` for TUI connection tracking; `PermissionRequester` and `SafetyLoop` held behind `Arc` on `ToolRuntime`
- **Circular imports:** None detected — `lib.rs` declares modules in dependency order; `agent_core` depends on `security` and `audit`; `tools` depends on `security` and `agent_core::repair::tools`
- **Python ↔ Rust bridge:** Single subprocess invocation at startup to load config; no runtime Python dependency after initialization
- **SQLite concurrency:** Multiple subsystems share the same DB file (`helix_context.db`) but each opens its own connection with WAL mode enabled
- **Error handling:** All subsystem errors propagate as `Box<dyn std::error::Error>` or formatted Strings; tool errors return `ToolResult { success: bool, output: String }`; no panics in normal tool execution paths

## Anti-Patterns

### Blocking on Async in Sync Context

**What happens:** `ToolRuntime::execute_sync()` calls `Handle::block_on` to await async permission requests from synchronous execution context.
**Why it's wrong:** Can cause thread-pool starvation if the runtime handle is saturated; deadlocks if called from within a tokio task running on the same runtime.
**Do this instead:** Make the entire execution path async up to the permission layer. The current pattern in `main.rs` and `server.rs` already calls `tool_runtime.execute().await` which is the async wrapper. The issue is only inside `execute_sync()` for the `RequireApproval` path.

### Mixed Error Handling Styles

**What happens:** Some functions return `Result<T, String>`, others use `Result<T, Box<dyn std::error::Error>>`, and tool definitions return a `ToolResult` struct with a boolean `success` flag.
**Why it's wrong:** Callers cannot use `?` uniformly; error context is lost when converting to `String`; impossible to pattern-match on structured error types.
**Do this instead:** Define a unified `AgentError` enum (or use `anyhow::Error` / `color-eyre`) across the crate, and keep `ToolResult` only for actual tool output (where success/failure is meaningful to the LLM).

## Error Handling

**Strategy:** Hybrid — structured errors in the security layer (`SecurityError` enum with Display/Error impls), boolean flags in tool execution (`ToolResult { success, output }`), and `Box<dyn std::error::Error>` for context engine operations.

**Patterns:**
- Security errors use a rich `SecurityError` enum with specific variants: `EmptyCommand`, `ParseError`, `MetacharacterBlocked`, `DangerousCommand`, `CommandNotAllowlisted`, `PathTraversal`, `Sanitization`
- Tool execution uses `ToolResult { success: bool, output: String }` — designed for LLM consumption with error messages in the output string
- Configuration errors propagate as `Result<_, String>` for simple fallback
- Policy decisions use `PolicyDecision` enum: `Allow`, `RequireApproval { reason_code, message }`, `Deny { reason_code, message, remediation }`
- Tool execution timeout: 30-second hard limit via `tokio::time::timeout` with automatic failure response
- LLM HTTP errors: retry with exponential backoff (1s base, 3 attempts) and full server recovery (watchdog reboot)
- Prompt injection: regex-based detection in tool arguments, denies with remediation suggestion
- All tool execution results are audited regardless of success/failure

## Cross-Cutting Concerns

**Logging:** `println!` / `eprintln!` throughout (no structured logging crate). Server stderr/stdout captured to `logs/start_server.{stdout,stderr}.log`. Audit events logged to SQLite with tamper evidence.

**Validation:** Defense-in-depth: (1) Schema validation via `schemars` at tool boundary, (2) PolicyEngine command validation (allowlist + metacharacters + path traversal), (3) Docker sandbox for terminal execution, (4) Prompt injection detection in tool arguments, (5) Guardian specialist voting for sensitive operations.

**Authentication:** None — all endpoints bind to `127.0.0.1` (localhost only). No auth tokens, no user authentication, no API keys for the agent's web server.

---

*Architecture analysis: 2026-07-30*
