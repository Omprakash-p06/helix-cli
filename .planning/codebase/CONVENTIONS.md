# Coding Conventions

**Analysis Date:** 2026-07-30

## Naming Patterns

**Rust Files:**
- `snake_case.rs` for all filenames — e.g., `tool_runtime.rs`, `runtime_profile.rs`, `agent_core/mod.rs`
- Source files under `src/` mirror module hierarchy with `mod.rs` for submodule roots

**Python Files:**
- `snake_case.py` for all filenames — e.g., `start_server.py`, `download_model.py`, `onboarding_profile.py`

**Functions:**
- **Rust:** `snake_case` — `detect_server_flavor()`, `expose_think_blocks()`, `enforce_sandbox()`, `execute_read_file()`
- **Python:** `snake_case` — `discover_models()`, `choose_default_model()`, `apply_runtime_overrides()`, `resolve_model_ref()`

**Variables:**
- **Rust:** `snake_case` — `app_config`, `tool_runtime`, `server_flavor`, `allowed_dir`
- **Python:** `snake_case` — `selected_model`, `interface_choice`, `env_model_name`

**Types (Rust):**
- `CamelCase` for structs, enums, traits:
  - Structs: `AppConfig`, `ChatMessage`, `AuditStore`, `ToolRuntime`, `ContextEngine`
  - Enums: `ServerFlavor`, `PermissionResponse`, `Provenance`, `Capability`, `ToolLifecycle`
  - Traits: `Tool`, `PermissionRequester`, `LogProvider`
- `SCREAMING_SNAKE_CASE` for constants and statics:
  - `READ_FILE_MAX_CHARS`, `CMD_OUTPUT_MAX_CHARS`, `DIAGNOSTIC_PATH_ALLOWLIST`
  - `TUI_HTTP_CONNECT_FAILS` (static AtomicUsize)

**Python Types:**
- `CamelCase` for classes and dataclasses — `ModelEntry`, `TestScanModelsDirectory`
- `SCREAMING_SNAKE_CASE` for module-level constants — `PROJECT_DIR`, `MODELS_DIR`, `KOBOLD_URLS`, `DEFAULT_MODELS`

## Code Style

**Rust:**
- **Edition:** 2024 (from `Cargo.toml`: `edition = "2024"`)
- **Formatting:** Default rustfmt (no `.rustfmt.toml` present)
- **Linting:** No clippy config file present; one `#[allow(clippy::too_many_arguments)]` annotation on `audit.rs:AuditStore::append_event`
- **`let`/`match` idioms:** Heavy use of Rust 2024 edition `let` chains — e.g.:
  ```rust
  if let Some(parent) = std::path::Path::new(path).parent()
      && !parent.as_os_str().is_empty()
  { ... }
  ```
  This is the dominant conditional pattern throughout the codebase.
- **`if let` with `&&`:** Used extensively for chained fallible conditions rather than nested `match`

**Python:**
- **Formatting:** No `.prettierrc`, `pyproject.toml`, or formatter config detected
- **Style:** Standard PEP 8 with 4-space indentation
- **Type hints:** Light use of `typing` module (`Optional`, `List`, `Dict`, `Any`, `Path`) in `scripts/config.py`; minimal elsewhere

## Import Organization

**Rust imports (`agent-rs/src/main.rs`):**
1. `use agent_rs::{...}` — crate-level imports first (braced multi-import)
2. `use crate::...` — internal module imports
3. `use std::...` — standard library
4. External crate imports — `use serde_json::...`, `use tokio::...`, `use reqwest::...`
5. `use static` items at file level (before functions)
- Groups separated by blank lines
- No consistent ordering within groups (e.g., `std::sync::Arc` may appear before or after `futures_util`)

**Python imports (`start.py`):**
1. Standard library: `import os`, `import sys`, `import time`, etc.
2. Third-party: `import requests`
3. Internal: `from scripts.helix_branding import ...`, `from scripts import config`
- Groups with blank line separators

## Module Organization

**Rust (`src/lib.rs`):**
- All modules declared at top of `lib.rs` as `pub mod name;`
- No nested `pub use` re-exports except `pub use types::{ChatMessage, ChatResponse, Choice, ServerFlavor};`
- Helper functions (`critic_message`, `expose_think_blocks`) live directly in `lib.rs`
- Submodules use `mod.rs` convention: `security/mod.rs`, `tui/mod.rs`, `context/mod.rs`, `agent_core/mod.rs`

**Python (`scripts/`):**
- Each file is a flat module with no package `__init__.py` (no `scripts/__init__.py` detected)
- Entry point scripts (`start.py`, `setup.py`) use `sys.path.insert(0, ...)` for module discovery

## Error Handling

**Rust — Layered approach:**
- **Main error type:** `Result<(), Box<dyn std::error::Error>>` in `main()` returns — catches-all
- **Internal tool errors:** Custom `ToolResult { success: bool, output: String }` struct (NOT Rust `Result` type)
  - Used in `tools.rs` and `agent_core/tool_runtime.rs`
  - `success: false` with descriptive `output` string is the error pattern
- **Propagation patterns:**
  - `expect("descriptive message")` — used heavily in main.rs for infallible operations
  - `map_err(|e| format!("...{}", e))?` — converting errors to `String`
  - `unwrap_or_else(|e| { eprintln!(...); default })` — graceful degradation with warning
  - `.ok()` — converting `Result` to `Option` when errors are tolerable
- **No custom error enums** — `Box<dyn std::error::Error>` and `String`-based errors are the pattern
- **Security errors:** `enforce_sandbox()` returns `Result<PathBuf, String>` where `String` is a human-readable violation message

**Python:**
- `try/except` with specific exception types — `except (FileNotFoundError, OSError, subprocess.SubprocessError):`
- `except Exception: pass` for cleanup operations (e.g., orphaned process cleanup)
- Broad `except Exception` for fallback paths
- No custom exception classes defined

## Result / Option Idioms

**Rust `unwrap()` / `expect()` usage (high):**
- `unwrap()` used freely in test code and in cases where failure is logically impossible
- `expect()` used in main.rs for configuration loading, CLI argument parsing
- Some `unwrap()` calls in `main.rs` on `Recv`, `Arc::clone`, and mpsc operations that are safe
- `unwrap_or_default()`, `unwrap_or_else()`, `unwrap_or()` for safe fallback

## Logging

**Rust:**
- `println!()` and `eprintln!()` for all output — no structured logging crate
- Prefix conventions:
  - `[Watchdog]` — watchdog events
  - `[Runtime]` — runtime profile selection
  - `[Context]` — context engine operations
  - `[Audit Warning]` — audit store failure
  - `[Config Warning]` — config fallback
  - `[Mode]` — mode switching
  - `[GSD]` / `[GSD Error]` — GSD orchestration feedback
  - `[Unknown command]` — TUI command errors
- `eprintln!` reserved for warnings and non-fatal errors

**Python:**
- `print()` for all output — no logging module
- Prefix convention: `[i]` (info), `[✓]` (success), `[!]` (warning/error)
- No logging levels or structured logging

## Async Patterns

**Rust:**
- **Runtime:** `tokio` with `#[tokio::main]` entry point
- **`tokio::sync::Mutex`** over `std::sync::Mutex` for async-shared state (e.g., `DiagnosticEngine`, `EvidenceStore`)
- **`spawn_blocking`** — synchronous tool execution dispatched via `tokio::task::spawn_blocking` in `tool_runtime.rs`
- **`tokio::time::timeout`** — 30-second outer timeout on tool execution
- **`futures_util::future::join_all`** — concurrent tool execution
- **`mpsc` channels** — for TUI event passing and `ToolLifecycle` events
- **`Arc`** for shared ownership of long-lived state

## Comments & Documentation

**Rust doc comments (`///`):**
- Used on public items: structs, enums, traits, functions
- Markdown formatted with code blocks (triple backticks)
- Architecture diagrams in module-level doc comments (`//!`) for `context/mod.rs`
- Inline `//!` module-level docs for `context/mod.rs`, `agent_core/web_research/mod.rs`
- Free-form code comments (`//`) used for section headers: `// ━━━━━━━━━━━━...`
- Changelog-style comments: `// ── GAP-1: Interpreter sandbox routing (P0-2) ──────`
- `// ===== SECTION HEADERS =====` for file organization

**Python docstrings (`"""`):**
- Module-level docstrings at file start — `tests/test_download_model.py`, `setup.py`, `scripts/start_server.py`
- Function-level docstrings where behavior is non-obvious — `tests/` use them as test descriptions
- `#` comments for inline explanation

## Serialization

**Rust:**
- `#[derive(Serialize, Deserialize)]` on all data structs
- `serde` rename attributes:
  - `#[serde(rename_all = "camelCase")]` — for tool input structs where JSON API expects camelCase
  - `#[serde(rename_all = "snake_case")]` — for security enums (`Capability`, `PermissionTier`)
  - `#[serde(tag = "type", content = "payload")]` — internally tagged enums (`ToolLifecycle`)
  - `#[serde(flatten)]` — for metadata flattening (`LogEntry`)
  - `#[serde(skip_serializing_if = "Option::is_none")]` — for optional fields (`ChatMessage`)
  - `#[serde(default = "fn")]` — for default values (`AppConfig.sandbox_interpreters`)
  - `#[serde(skip)]` — for computed fields (`AppConfig.permission_tier`, `AppConfig.backend_capabilities`)

## Security Patterns

- **`enforce_sandbox()`** in `tools.rs` — canonicalize + prefix check against allowed directory and diagnostic allowlist
- **`PolicyContext`** passed to every tool execution — bundles `PermissionTier`, `TrustLevel`, `exec_mode`, `workspace_root`
- **`evaluate_tool_call()`** — centralized security gate in `security/policy.rs`
- **`CapabilitySet`** — fine-grained capability model with `read_only()`, `guided_repair()`, `autonomous()` tiers
- **`DockerSandbox`** — optional containerized execution for interpreter commands
- **Secrets redaction** — `redact_secrets()` in `agent_core/diagnostics/system.rs`

## Configuration Pattern

- **Python-first config:** Rust loads config via `AppConfig::load_from_python()` which executes `config.py` as a subprocess and reads JSON from stdout
- **Environment overrides:** `HELIX_*` env vars override config values (e.g., `HELIX_GPU_LAYERS`, `HELIX_BATCH_SIZE`)
- **`RuntimeProfile` enum** (`LatencyCpu`, `BalancedCpu`, `SafeRecovery`) — profile selection via `select_runtime_profile()`

## Trait Design

- **`Tool` trait:** `name()`, `description()`, `schema()`, `execute()` — all tools in `tools.rs` implement it
- **`PermissionRequester` trait:** single `request_permission()` async method
- **`LogProvider` trait:** single `get_logs()` method with platform-specific impls
- **`#[async_trait]`** for async trait methods

## Function Design

**Rust:**
- Pure utility functions (`clean_chat_output`, `expose_think_blocks`) are free functions in `lib.rs` or `utils.rs`
- Tool execution functions prefixed `execute_` — `execute_read_file()`, `execute_run_terminal_command()`
- Boolean helper functions prefixed `should_` or `is_` — `should_retry_non_stream_after_stream_error()`, `is_transient_http_error()`
- Tool structs are zero-sized unit structs implementing `Tool` trait

**Python:**
- Discovery functions prefixed `discover_` / `resolve_` / `choose_`
- Side-effect functions prefixed `ensure_` / `apply_` / `clean_`
- Boolean helpers: `ask_yes_no()`, `has_latest()` — matches Python std naming

---

*Convention analysis: 2026-07-30*
