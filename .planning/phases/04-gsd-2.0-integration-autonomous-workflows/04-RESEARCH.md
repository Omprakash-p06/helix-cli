# Phase 04: GSD 2.0 Integration & Autonomous Workflows - Research

**Researched:** 2026-04-25
**Domain:** Multi-phase orchestration, context lifecycle control, autonomous recovery operators, and loop-safe execution.
**Confidence:** HIGH

## Summary

This research defines how Helix Agent should integrate GSD 2.0 as a true orchestration layer for long-running repair workflows. The current Rust runtime already has strong primitives (tool lifecycle events, policy gates, transactional safety loop, audit logging, retry paths), but it does not yet expose a first-class phase state machine (`discover -> discuss -> plan -> execute -> verify -> close`) with deterministic context resets.

The recommended path is an "Orchestrator Adapter" architecture: keep `agent-rs` as the secure execution core, and add a phase coordinator that can either call GSD Pi SDK/CLI directly or run an equivalent local phase driver with the same contract and artifacts.

**Primary recommendation:** implement a dedicated orchestration module in `agent-rs` that persists per-phase artifacts, enforces phase boundaries, and exposes `/gsd plan` and `/gsd execute` commands through a single state transition API.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GSD-01 | Pi SDK orchestration as primary state machine | Defined Orchestrator Adapter and phase transition contract. |
| GSD-02 | Phase-based context resets | Defined per-phase context snapshot/build strategy and reset checkpoints. |
| GSD-03 | Autonomous recovery (RETRY/DECOMPOSE/PRUNE) | Defined repair operator decision matrix and escalation policy. |
</phase_requirements>

## Standard Stack

### Core
| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| `gsd-tools.cjs` (CLI bridge) | local | Orchestration entrypoint and project state tooling | Already present in workspace tasks and phase workflows. [VERIFIED: `.vscode/tasks.json`] |
| `tokio` | 1.43.x | Async phase coordinator and cancellation/timeouts | Already the runtime backbone for agent execution. [VERIFIED: `Cargo.toml`] |
| `serde` / `serde_json` | 1.0.x | Phase artifact schema serialization | Existing data contract stack for tool calls/messages. [VERIFIED: `Cargo.toml`] |
| `rusqlite` | 0.39.x | Durable phase/session checkpoints and recovery markers | Existing local persistence pattern in audit subsystem. [VERIFIED: `Cargo.toml`] |

### Supporting
| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `reqwest` | 0.12.x | Health probes during verify/close gates | Verify model/backend liveness before phase advance. |
| Existing `AuditStore` | current | Immutable phase transition trail | Record transition attempts/outcomes for forensic replay. |
| Existing `ToolRuntime` lifecycle events | current | Real-time phase execution telemetry | Emit user-visible progress during `/gsd execute`. |

**Installation:**
```bash
# No mandatory new dependencies for Wave 0.
# Reuse existing runtime crates already in agent-rs/Cargo.toml.
```

## Architecture Patterns

### Recommended Project Structure
```
agent-rs/src/
├── agent_core/
│   ├── orchestration/
│   │   ├── mod.rs                  # Public orchestration API
│   │   ├── phase_state.rs          # Discover/Discuss/Plan/Execute/Verify/Close FSM
│   │   ├── context_reset.rs        # Context rebuild and boundary guards
│   │   ├── recovery.rs             # RETRY/DECOMPOSE/PRUNE decision logic
│   │   └── artifacts.rs            # Read/write phase plans and execution receipts
│   └── tool_runtime.rs             # Existing secure execution layer (unchanged contract)
└── main.rs                         # Slash command routing to orchestration API
```

### Pattern 1: Orchestrator Adapter
**What:** Wrap all phase transitions behind one Rust API (e.g., `advance_phase(phase, input)`).
**When to use:** Every `/gsd plan` and `/gsd execute` request.
**Why:** Keeps GSD integration replaceable (CLI/Pi SDK/direct API) while preserving a stable core contract.

### Pattern 2: Artifact-First Phases
**What:** Each phase emits structured artifacts (`plan.json`, `execution.json`, `verify.json`) under phase-scoped directories.
**When to use:** On every phase boundary and recovery attempt.
**Why:** Prevents context rot by reconstructing prompts from artifacts rather than chat history.

### Pattern 3: Deterministic Recovery Operator
**What:** On verification failure, apply ordered recovery: RETRY (bounded), then DECOMPOSE (split step), then PRUNE (remove optional step), then escalate.
**When to use:** `execute` and `verify` failures.
**Why:** Enables autonomous progression while keeping bounded risk.

### Anti-Patterns to Avoid
- Chat-history-only phase transitions (causes drift/context rot).
- Unbounded retry loops without loop fingerprints.
- Coupling orchestration state directly to UI widgets instead of persisted phase state.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Long-running agent state | Ad-hoc in-memory flags only | Persisted phase FSM + artifacts | Survives crashes and allows replay. |
| Failure recovery | Manual if/else retries scattered in call-sites | Central recovery operator (RETRY/DECOMPOSE/PRUNE) | Keeps behavior testable and auditable. |
| Loop detection | String compare on last error only | Signature-based loop detector (tool + args hash + outcome) | Catches repeated failure cycles reliably. |
| Orchestration telemetry | Print-only logs | Existing ToolLifecycle + audit events | Supports TUI/Web progress and postmortems. |

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | `.planning/STATE.md`, phase folders, audit DB | Add phase-04 artifact schema and transition records. |
| Live service config | Backend endpoints and retry env controls in `main.rs` | Reuse for verify-phase health checks. |
| Orchestration hooks | `ToolRuntime`, `SafetyLoop`, critic retry messages | Lift into explicit phase executor abstraction. |
| Existing phase artifacts | Phases 01-03 research/plans/validation docs | Use as blueprint for phase-04 artifact conventions. |
| GSD tooling | `gsd-tools.cjs` usage in VS Code tasks | Formalize command routing via slash command handlers. |

## Common Pitfalls

### Pitfall 1: Context Rot Across Multi-step Sessions
**What goes wrong:** The model accumulates stale constraints and repeats failed paths.
**How to avoid:** Rebuild each phase prompt from artifacts + current system evidence only.

### Pitfall 2: False Recovery Success
**What goes wrong:** Retry appears successful, but verify criteria were too weak.
**How to avoid:** Define hard verify predicates (service state, process health, error log deltas) before execute.

### Pitfall 3: Infinite Recovery Loop
**What goes wrong:** RETRY/DECOMPOSE cycles oscillate without progress.
**How to avoid:** Track loop signature and enforce max attempts per signature before escalation.

### Pitfall 4: Unsafe Auto-advancement
**What goes wrong:** High-risk commands run without explicit approval due to orchestration shortcuts.
**How to avoid:** Keep policy and HITL checks in `ToolRuntime`; orchestration cannot bypass policy decisions.

## Code Examples

### Phase Transition Contract (Rust)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Discover,
    Discuss,
    Plan,
    Execute,
    Verify,
    Close,
}

pub struct PhaseOutcome {
    pub next: Option<Phase>,
    pub summary: String,
    pub artifact_path: String,
}

pub async fn advance_phase(phase: Phase, input: serde_json::Value) -> anyhow::Result<PhaseOutcome> {
    // 1) load minimal context for this phase
    // 2) run phase operation
    // 3) persist artifact and transition audit
    // 4) return deterministic next phase
    todo!()
}
```

### Recovery Operator Skeleton
```rust
pub fn recover_after_verify_failure(attempt: u8, is_optional_step: bool) -> &'static str {
    if attempt < 2 {
        "RETRY"
    } else if attempt < 4 {
        "DECOMPOSE"
    } else if is_optional_step {
        "PRUNE"
    } else {
        "ESCALATE"
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Prompt-only workflow control | Programmatic orchestration state machines | 2025-2026 | Deterministic recovery and reproducible runs. |
| Single-pass task execution | Verify-gated multi-phase execution | 2024+ | Lower silent failure rate in long tasks. |
| Human-only retries | Hybrid autonomous recovery operators | 2025+ | Better completion on complex, brittle workflows. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | GSD CLI/Pi SDK can be invoked reliably from host process | Orchestrator Adapter | Need fallback local driver if invocation is unavailable in target env. |
| A2 | Existing `ToolRuntime` hooks are sufficient for phase telemetry | Telemetry | May require additional lifecycle event types for phase granularity. |
| A3 | Current policy/HITL interception remains authoritative in execute phase | Safety | If bypass paths exist, autonomous mode can become unsafe. |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js runtime | GSD CLI bridge | likely | host-managed | Local Rust phase driver |
| `gsd-tools.cjs` path | phase init/state ops | configured in tasks | local path in `.vscode/tasks.json` | Direct phase files + manual transitions |
| Rust async stack | orchestration runtime | yes | current | N/A |
| SQLite (`rusqlite`) | persistent phase checkpoints | yes | 0.39.x | JSON checkpoint files |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` |
| Config file | `agent-rs/Cargo.toml` |
| Quick run command | `cargo test orchestration::` |
| Full suite command | `cargo test && cargo clippy -- -D warnings` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GSD-01 | Phase state transition correctness | unit | `cargo test test_phase_fsm_transitions` | no (Wave 0) |
| GSD-02 | Context reset between phases | integration | `cargo test test_context_reset_boundaries` | no (Wave 0) |
| GSD-03 | Recovery operator ordering and caps | unit | `cargo test test_recovery_operator_matrix` | no (Wave 0) |
| GSD-03 | Loop detection escalation | integration | `cargo test test_loop_detector_escalates` | no (Wave 0) |

### Wave 0 Gaps
- [ ] `agent-rs/src/agent_core/orchestration/` module scaffold.
- [ ] `agent-rs/tests/gsd_orchestration_validation.rs` for phase and recovery tests.
- [ ] slash command routing for `/gsd plan` and `/gsd execute`.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V1 Architecture | yes | Explicit phase-state boundaries and trust zones. |
| V4 Access Control | yes | HITL and policy checks remain mandatory for write/admin tools. |
| V10 Malicious Logic | yes | Loop detection + bounded autonomous recovery. |
| V12 File/Resources | yes | Artifact integrity and append-only transition audit. |

### Known Threat Patterns for Orchestration

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Loop-stall exhaustion | Denial of Service | Loop signature caps and escalation path. |
| Context poisoning across phases | Tampering | Hard context reset from persisted artifacts only. |
| Unauthorized autonomous writes | Elevation of Privilege | Policy engine and approval interceptor remain in execution path. |
| Silent verify bypass | Repudiation/Tampering | Mandatory verify artifact + transition audit append. |

## Sources

### Primary (HIGH confidence)
- Workspace architecture and roadmap artifacts:
  - `.planning/ROADMAP.md`
  - `.planning/REQUIREMENTS.md`
  - `.planning/codebase/ARCHITECTURE.md`
  - `misc/23.04.26 implementation plan.md`
- Runtime implementation references:
  - `agent-rs/src/agent_core/tool_runtime.rs`
  - `agent-rs/src/main.rs`
  - `.vscode/tasks.json`

### Secondary (MEDIUM confidence)
- General industry patterns for agent orchestration FSMs, bounded retries, and artifact-first execution (2024-2026).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Core dependencies and hooks already exist.
- Architecture: HIGH - Existing runtime primitives map directly to required orchestration behavior.
- Pitfalls: HIGH - Failure modes are known from long-running agent workflows.

**Research date:** 2026-04-25
**Valid until:** 2026-05-25
