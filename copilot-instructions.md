<!-- GSD:project-start source:PROJECT.md -->
## Project

**Helix Agent**

Helix Agent is a local-first, agentic CLI stack that runs fast on consumer and low-end laptops. It provides reliable tool-calling, local model serving, and practical automation workflows without cloud lock-in.

**Core Value:** A local agent that stays usable, fast, and reliable on low-end hardware while still completing real tool-driven tasks.

### Constraints

- **Hardware**: Must run on low-end laptops - defaults must avoid heavy memory pressure.
- **Runtime**: Local-only by default - no mandatory cloud dependency for core workflows.
- **Usability**: Setup/start should avoid brittle steps (admin lockouts, long preflights by default).
- **Compatibility**: Must handle both llama.cpp and KoboldCPP endpoint behavior.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Python for setup and runtime launching.
- Rust for orchestration and tool execution.
- C/C++ build artifacts from llama.cpp backend runtime.
## Runtime Components
- Python runtime entry: `start.py`.
- Python server launcher: `scripts/start_server.py`.
- Rust orchestrator binary: `agent-rs/target/debug/agent-rs.exe`.
- Inference backend binary: `llama.cpp/build/bin/llama-server`.
- Fallback backend binary: `koboldcpp.exe` at repo root.
## Core Dependencies
- Python deps installed in setup: `requests`, `tqdm`, `openai`.
- Rust deps in `agent-rs/Cargo.toml`: `tokio`, `reqwest`, `serde`, `serde_json`, `schemars`, `sysinfo`, `tiktoken-rs`, `ignore`.
## Build and Toolchain
- Python setup orchestration in `setup.py`.
- Rust build toolchain via `cargo` from `agent-rs/`.
- llama.cpp CMake output under `llama.cpp/build/`.
## Configuration Sources
- Generated runtime config in `scripts/config.py`.
- Hardware tiering and tuning logic in `scripts/system_check.py`.
- Runtime model override via env vars in `scripts/start_server.py`:
## Context and Throughput Defaults
- Minimum user context is enforced in `setup.py` with `MIN_USER_CONTEXT_SIZE = 8192`.
- Eval preflight defaults are constrained in `tests/eval.py` via env-filtered task selection.
## Notes
- Repo is brownfield and already has mixed-language build artifacts.
- Startup path is designed for local/offline inference with OpenAI-compatible endpoint behavior.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Python Conventions
- Most scripts are executable with shebang `#!/usr/bin/env python3`.
- Logging style is CLI-oriented with explicit status prefixes.
- Config and behavior often controlled through environment variables.
- Setup and launcher scripts avoid framework-heavy abstractions.
## Rust Conventions
- Serde models use explicit structs/enums for tool schemas.
- Tool argument contracts are defined in `agent-rs/src/tools.rs`.
- JSON interaction uses `serde_json::Value` where schema flexibility is needed.
- Error handling prefers explicit match blocks and user-readable messages.
## Runtime UX Conventions
- Helix branding banner is centralized in `scripts/helix_branding.py`.
- Startup flows print major stage separators.
- Logs for startup diagnostics are written to `logs/`.
## Safety Conventions
- Tool execution enforces sandbox path checks in Rust tool layer.
- Dangerous terminal command checks exist before execution.
- Setup enforces minimum context constraints for user runtime.
## File and Path Conventions
- Project-relative paths are preferred in Python launchers.
- Binary path selection includes OS-specific branching.
- Startup model selection is path-based (`models/*.gguf`) with env overrides.
## Documentation Conventions
- README describes architecture and run commands in practical order.
- GSD workflow docs are structured with explicit step gates.
- Mapping docs use markdown headings and concrete file references.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## High-Level Pattern
- Hybrid orchestrator architecture:
## Main Runtime Flow
## Server Layer
- Backend process is independent from orchestrator process.
- Readiness is based on HTTP polling against `/v1/models`.
- Startup diagnostics are persisted to logs in `logs/`.
## Orchestrator Layer
- Message struct and tool-call loop live in `agent-rs/src/main.rs`.
- Tool schemas and implementations live in `agent-rs/src/tools.rs`.
- Token counting and memory compaction are built into the Rust loop.
## Setup Layer
- `setup.py` performs hardware detection, dependency install, backend build, model download, and config generation.
- Context minimum and preflight gate behavior are controlled in setup.
## Failure and Fallback Behavior
- llama.cpp launch failure triggers fallback to KoboldCPP in `scripts/start_server.py`.
- Startup timeout and log-tail surfacing are handled in `start.py`.
- Eval preflight can be skipped by setup gate policy.
## Current Architectural Strengths
- Local-first design with explicit fallback path.
- Strong separation between backend serving and orchestration logic.
- Typed tool schemas in Rust reduce malformed execution risk.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
