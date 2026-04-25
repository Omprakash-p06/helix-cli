# Codebase Structure

**Analysis Date:** 2025-05-15

## Directory Layout

```
helix-agent/
├── agent-rs/           # Rust Core Agent
│   ├── src/            # Source code
│   │   ├── agent_core/ # Tool runtime and orchestration hooks
│   │   ├── security/   # Policy engine and sandbox logic
│   │   ├── tui/        # Terminal UI implementation
│   │   └── server.rs   # API server (Axum)
│   └── tests/          # Rust-specific tests
├── llama.cpp/          # Inference Engine (Submodule/Vendor)
├── web-ui/             # Frontend Application (React/Vite)
├── scripts/            # Python automation and model management
├── models/             # Local storage for GGUF weights
└── .planning/          # Project documentation and roadmap
```

## Directory Purposes

**agent-rs:**
- Purpose: The "brain" and "muscle" of the agent.
- Contains: Rust source code for AI orchestration, tool execution, and the TUI.
- Key files: `src/main.rs` (Entry), `src/agent_core/tool_runtime.rs` (Execution).

**security:**
- Purpose: Defense-in-depth safety layer.
- Contains: Policy definitions, command risk scanners, and audit trails.
- Key files: `src/security/policy.rs`.

**web-ui:**
- Purpose: Modern GUI for users who prefer a dashboard over TUI.
- Contains: React components, Vite configuration, and Tailwind styles.

**scripts:**
- Purpose: Lifecycle management.
- Contains: Hardware checks, model installers, and server launchers.
- Key files: `scripts/start_server.py`, `scripts/system_check.py`.

## Key File Locations

**Entry Points:**
- `agent-rs/src/main.rs`: CLI/TUI entry.
- `agent-rs/src/server.rs`: Web API entry.

**Configuration:**
- `scripts/config.py`: Global project configuration.
- `agent-rs/Cargo.toml`: Rust dependencies.

**Core Logic:**
- `agent-rs/src/agent_core/tool_runtime.rs`: Tool execution sandbox.
- `agent-rs/src/security/policy.rs`: Security guardrails.

**Testing:**
- `tests/`: End-to-end and integration tests (Python).
- `agent-rs/tests/`: Rust core tests.

## Naming Conventions

**Files:**
- Rust: `snake_case.rs`
- TypeScript: `PascalCase.tsx` or `camelCase.ts`
- Python: `snake_case.py`

**Directories:**
- Rust Modules: `snake_case/`
- Frontend Components: `PascalCase/` or `kebab-case/`

## Where to Add New Code

**New Tool/Skill:**
- Primary code: `agent-rs/src/tools.rs` (Registry) and implement the tool trait.
- Policy check: `agent-rs/src/security/policy.rs`.

**New UI Feature:**
- Frontend: `web-ui/src/components/`.
- Backend Hook: `agent-rs/src/server.rs`.

**New Model Integration:**
- Loading Logic: `scripts/start_server.py` and `agent-rs/src/runtime_profile.rs`.
- Download Script: `scripts/model_install.py`.

## Special Directories

**models/:**
- Purpose: Storage for multi-gigabyte GGUF files.
- Generated: No (Downloaded).
- Committed: No (Ignored in `.gitignore`).

---

*Structure analysis: 2025-05-15*
