# Architecture

**Analysis Date:** 2025-05-15

## Pattern Overview

**Overall:** Defense-in-depth AI Orchestrator.

**Key Characteristics:**
- **Local-First:** All inference and execution happen on the user's hardware.
- **Autonomous Orchestration:** Uses GSD 2.0 to manage multi-step troubleshooting phases.
- **Policy-Driven Security:** Every tool call is intercepted and evaluated against a permission tier.

## Layers

**Orchestration Layer (GSD 2.0):**
- Purpose: Manages the `Discover → Discuss → Plan → Execute → Verify → Close` lifecycle.
- Location: Integrated via GSD 2.0 Pi SDK into `agent-rs`.
- Contains: Phase state machine, context reset logic, and autonomous recovery operators (RETRY, DECOMPOSE).
- Depends on: `agent-rs` core.
- Used by: User TUI/Web UI.

**Intelligence Layer (Qwen 3.6):**
- Purpose: Provides reasoning and tool-calling capabilities.
- Location: `llama.cpp/` (Inference engine).
- Contains: Qwen 3.6 (27B/35B MoE) model weights and sampler logic.
- Depends on: Hardware (CUDA/Metal/CPU).
- Used by: Orchestration Layer.

**Execution Layer (Tool Runtime):**
- Purpose: Executes terminal commands and filesystem operations.
- Location: `agent-rs/src/agent_core/tool_runtime.rs`.
- Contains: Tool registry and execution logic.
- Depends on: Security Layer.
- Used by: Orchestration Layer.

**Security Layer:**
- Purpose: Validates tool calls and enforces safety policies.
- Location: `agent-rs/src/security/`.
- Contains: Policy engine, command risk evaluation, and audit logging.
- Depends on: `agent-rs` core.
- Used by: Execution Layer.

## Data Flow

**Troubleshooting Flow:**

1. **Discovery:** Agent identifies OS issue via diagnostic tools.
2. **Planning:** GSD 2.0 generates a multi-step repair plan.
3. **Approval:** User reviews plan in TUI/Web UI.
4. **Execution:** Tool Runtime executes steps; Security Layer validates each command.
5. **Verification:** GSD 2.0 verifies the repair (e.g., checking if service is back up).
6. **Closure:** Audit log is finalized and session closed.

**State Management:**
- Orchestration state: Managed by GSD 2.0 (Pi SDK).
- Persistence: SQLite database via `rusqlite`.

## Key Abstractions

**Tool Runtime (`agent-rs/src/agent_core/tool_runtime.rs`):**
- Purpose: Standardizes how AI actions are executed, audited, and sandboxed.
- Pattern: Strategy pattern for tool implementations.

**Policy Engine (`agent-rs/src/security/policy.rs`):**
- Purpose: Decouples safety rules from execution logic.
- Pattern: Interceptor/Middleware.

## Entry Points

**Rust CLI/TUI:**
- Location: `agent-rs/src/main.rs`.
- Triggers: User execution of `helix-agent`.
- Responsibilities: Bootstrapping the server, starting the TUI, and managing the local LLM lifecycle.

**Web API:**
- Location: `agent-rs/src/server.rs`.
- Triggers: HTTP requests from `web-ui`.
- Responsibilities: Exposing OpenAI-compatible endpoints and orchestration hooks.

## Error Handling

**Strategy:** Autonomous recovery with human escalation.

**Patterns:**
- **GSD 2.0 Operators:** RETRY, DECOMPOSE, and PRUNE for failed tasks.
- **Graceful Fallback:** Switching between model tiers (27B vs 35B) if VRAM is constrained.

## Cross-Cutting Concerns

**Logging:** Centralized audit store in `agent-rs/src/audit.rs`.
**Validation:** Regex-based command scanning and injection detection in `agent-rs/src/security/policy.rs`.
**Authentication:** Permission tiers (ReadOnly/WorkspaceWrite/FullExec) determined at startup.

---

*Architecture analysis: 2025-05-15*
