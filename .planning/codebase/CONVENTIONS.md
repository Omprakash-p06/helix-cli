# Helix OS Agent Global Process Standards & Conventions

## Snapshot
Last refreshed: 2026-04-24
Conventions updated to reflect the pivot to Autonomous OS Troubleshooting (Helix OS Agent).

## 1. Security-First Execution (Non-Negotiable)
- **Mandatory Sandboxing:** All agent-initiated shell commands and scripts MUST run inside an isolated Docker or MicroVM sandbox.
- **Policy Engine Gate:** Every command must pass through the canonicalization and allowlist policy engine. No direct `sh -c` or `cmd /C` execution on the host without a policy verdict.
- **Human-in-the-Loop:** All state-modifying actions (writes, restarts, installs) REQUIRE explicit user confirmation. No "silent" system modifications allowed in default modes.

## 2. Orchestration Standards (GSD 2.0)
- **Phase-Based Lifecycle:** Work follows the GSD 2.0 protocol: `Discover → Discuss → Plan → Execute → Verify → Close`.
- **Context Management:** Reset LLM context between phases to prevent context rot. Major decision artifacts must be persisted as structured JSON/TOML, not conversation history.
- **Verification Requirement:** Every task must have a deterministic verification step. Success is defined by state change validation, not model confirmation.

## 3. Observability and Auditability
- **Immutable Audit Logging:** Every policy decision, command execution, and tool outcome must be recorded in an append-only, tamper-evident audit log.
- **Reasoning Transparency:** Internal deliberation (`<think>` blocks) must be preserved in the audit log and exposed in the UI for transparency.
- **Confidence Scoring:** Agents must output a confidence score for diagnostic hypotheses. Low confidence (<80%) triggers mandatory user re-verification.

## 4. Rust Code Conventions (Core Orchestrator)
- **Async-First:** `tokio` for all I/O, streaming, and tool spawning.
- **Type Safety:** Use `agent_core` (or `types.rs`) for all message and payload definitions.
- **Tool Registry:** Tools must be registered with explicit permission metadata (e.g., `Read`, `Write`, `Elevated`).
- **Zero-Warning Policy:** Code must be clippy-clean and free of compiler warnings.

## 5. UI/UX Standards
- **Streaming Reliability:** Raw byte-level streaming with immediate token rendering (no buffering delays).
- **Interactive Feedback:** Real-time visual status for tool lifecycles (Running, Completed, Failed, Blocked).
- **Rollback Visibility:** Users must be informed when a pre-repair snapshot is taken and offered a "Undo/Rollback" option.

## 6. Planning Workflow
- **Phase Directories:** All active work resides in `.planning/phases/XX-name/`.
- **Immutable GSD History:** Maintain an audit trail of phase transitions and plan executions.
- **Renumbering Policy:** If phase numbers conflict, renumber chronologically during milestone cleanup.
