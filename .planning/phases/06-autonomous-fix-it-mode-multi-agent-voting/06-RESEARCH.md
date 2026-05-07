# Phase 06: Autonomous "Fix It" Mode & Multi-agent Voting - Research

**Researched:** 2026-05-08
**Domain:** Rust local-agent safety policy, quorum-based LLM orchestration, and non-bypassable execution controls
**Confidence:** MEDIUM

## Summary

Phase 06 should extend the existing trust-and-policy seam instead of introducing a separate autonomous subsystem. The repo already has a permission tier model in [agent-rs/src/security/policy.rs](/home/omprakash/helix-agent/agent-rs/src/security/policy.rs), a TUI security setting in [agent-rs/src/tui/state.rs](/home/omprakash/helix-agent/agent-rs/src/tui/state.rs), tool-execution approval hooks in [agent-rs/src/agent_core/tool_runtime.rs](/home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs), and a hard blocklist path in [agent-rs/src/security/policy.rs](/home/omprakash/helix-agent/agent-rs/src/security/policy.rs) plus [scripts/config.py](/home/omprakash/helix-agent/scripts/config.py). That makes the controlling design pattern a policy-first execution pipeline with a quorum gate in front of high-risk actions [VERIFIED: workspace code].

The standard implementation stack is already visible in the repo and current upstream docs: Tokio for parallel fan-out and timeouts, `async-openai` for OpenAI-compatible local providers, `schemars` + `serde_json` for typed vote envelopes and tool schemas, `bollard` for Docker-backed sandbox execution, `rusqlite` for append-only state persistence, and `axum` only if the Guardian controller is exposed over HTTP [CITED: docs.rs/tokio/latest/tokio/; CITED: docs.rs/async-openai/latest/async_openai/; CITED: docs.rs/schemars/latest/schemars/; CITED: docs.rs/bollard/latest/bollard/; CITED: docs.rs/rusqlite/latest/rusqlite/; CITED: docs.rs/axum/latest/axum/]. Current crates.io checks show newer releases are available for `async-openai`, `tokio`, `axum`, `bollard`, `reqwest`, `inquire`, and `ratatui` than the versions pinned in the repo, so the plan should treat upgrades as optional unless a specific API is needed [VERIFIED: crates.io registry via cargo search; VERIFIED: workspace Cargo.toml].

**Primary recommendation:** implement Safe Mode / Auto Mode as a persisted permission tier that feeds `PolicyContext`, route high-risk decisions through a typed Guardian quorum coordinator, and keep irreversible-action denial enforced in the policy engine so it cannot be bypassed by prompts or UI state [VERIFIED: workspace code; CITED: docs.rs/tokio/latest/tokio/; CITED: docs.rs/schemars/latest/schemars/].

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| User-configurable trust level (Safe Mode vs. Auto Mode) | Browser / Client | API / Backend | The visible toggle belongs in the TUI settings surface, but the actual enforcement must live in backend policy context so execution cannot diverge from UI state [VERIFIED: workspace code]. |
| Guardian multi-agent voting | API / Backend | Database / Storage | Vote fan-out, aggregation, and quorum decisions are orchestration concerns; audit and decision history should be persisted so the result is replayable and reviewable [VERIFIED: workspace code; CITED: docs.rs/rusqlite/latest/rusqlite/]. |
| Hardcoded blocklist for irreversible actions | API / Backend | — | Non-bypassable denial belongs in the policy engine and tool runtime, not in prompts, UI labels, or model instructions [VERIFIED: workspace code]. |
| High-risk repair execution | API / Backend | Database / Storage | Execution must pass through transactional repair + rollback plumbing before it can mutate system state [VERIFIED: workspace code; CITED: docs.rs/bollard/latest/bollard/]. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.52.2 [VERIFIED: crates.io registry via cargo search] | Async runtime, spawned tasks, timeouts, and join macros for parallel Guardian probes | Tokio is the standard async runtime for Rust applications and the docs explicitly recommend `full` for application work and `spawn`/`join!` for task fan-out [CITED: docs.rs/tokio/latest/tokio/]. |
| async-openai | 0.37.0 [VERIFIED: crates.io registry via cargo search] | OpenAI-compatible client for local LLM backends and typed request/response calls | The crate is designed for OpenAI-compatible providers, supports BYOT request/response payloads, and is the cleanest fit for local llama.cpp/koboldcpp endpoints [CITED: docs.rs/async-openai/latest/async_openai/]. |
| schemars | 1.2.1 [VERIFIED: crates.io registry via cargo search] | JSON Schema generation for vote envelopes and tool arguments | The crate is built around `JsonSchema` + `schema_for!`, matches Serde behavior, and keeps vote payloads typed instead of ad hoc [CITED: docs.rs/schemars/latest/schemars/]. |
| serde | 1.0.228 [VERIFIED: crates.io registry via cargo search] | Serialization/deserialization backbone for policy, votes, and session state | Serde is the de facto Rust serialization layer and `schemars` explicitly aligns schema generation with Serde attributes [CITED: docs.rs/schemars/latest/schemars/]. |
| serde_json | 1.0.149 [VERIFIED: crates.io registry via cargo search] | JSON value transport for tool calls and vote envelopes | The repo already exchanges tool arguments and messages as JSON, so this is the standard lossless interchange format for quorum payloads [VERIFIED: workspace code]. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| axum | 0.8.9 [VERIFIED: crates.io registry via cargo search] | HTTP surface for Guardian control endpoints or telemetry | Use only if Phase 06 needs a service API; `axum` is built on Tower middleware and keeps handlers, extractors, and shared state simple [CITED: docs.rs/axum/latest/axum/]. |
| bollard | 0.21.0 [VERIFIED: crates.io registry via cargo search] | Docker/Podman API client for sandboxed execution paths | Use for container execution and engine negotiation instead of shelling out to Docker CLI wrappers [CITED: docs.rs/bollard/latest/bollard/]. |
| rusqlite | 0.39.0 [VERIFIED: crates.io registry via cargo search] | Persistent append-only audit and decision store | Use SQLite when you need local, lightweight, transaction-safe persistence for policy and quorum decisions [CITED: docs.rs/rusqlite/latest/rusqlite/]. |
| reqwest | 0.13.3 [VERIFIED: crates.io registry via cargo search] | Compatibility probes and fallback HTTP transport | Use for health checks and compatibility requests only; the primary model client should remain `async-openai` [CITED: docs.rs/async-openai/latest/async_openai/]. |
| inquire | 0.9.4 [VERIFIED: crates.io registry via cargo search] | Interactive confirmation prompt in the TUI | Use for explicit human approval flows that remain in Safe Mode [VERIFIED: workspace code]. |
| ratatui | 0.30.0 [VERIFIED: crates.io registry via cargo search] | Terminal UI rendering for trust-mode controls and approval state | Use for the existing TUI surface, not for policy enforcement [VERIFIED: workspace code]. |

**Installation:**
```bash
cargo add tokio async-openai schemars serde serde_json axum bollard rusqlite reqwest inquire ratatui
```

**Version verification:** the current releases above were checked against crates.io on 2026-05-08, and the upstream docs confirm the APIs that matter for this phase: `async-openai` supports OpenAI-compatible providers and BYOT, `tokio` provides `spawn`, `join!`, `spawn_blocking`, and timeouts, `schemars` generates schema from Serde types, `axum` uses extractor-based state, `bollard` connects to Docker/Podman, and `rusqlite` provides transactional SQLite access [CITED: docs.rs/async-openai/latest/async_openai/; CITED: docs.rs/tokio/latest/tokio/; CITED: docs.rs/schemars/latest/schemars/; CITED: docs.rs/axum/latest/axum/; CITED: docs.rs/bollard/latest/bollard/; CITED: docs.rs/rusqlite/latest/rusqlite/].

## Architecture Patterns

### System Architecture Diagram

```text
User input / orchestration state
        |
        v
Trust mode settings in TUI or config
        |
        v
PolicyContext + permission tier + hard blocklist
        |
        +-----------------------------+
        |                             |
        v                             v
Guardian specialist calls        Immediate deny
(parallel tokio tasks,           for blocked or
typed vote envelopes)            non-allowlisted actions
        |
        v
Quorum / consensus evaluator
        |
        +-----------------------------+
        |                             |
        v                             v
Low-risk allow                 High-risk require approval
        |                             |
        v                             v
ToolRuntime + sandbox / repair loop <-+
        |
        v
Audit log + state persistence + response
```

The primary use case should be traceable end-to-end: user trust mode influences policy context, Guardian fan-out collects structured votes, the quorum gate decides whether the action can proceed, and the policy engine still owns the final deny for blocked commands [VERIFIED: workspace code; CITED: docs.rs/tokio/latest/tokio/].

### Recommended Project Structure

```text
src/
├── security/        # canonical policy, blocklist, trust-tier enforcement
├── agent_core/      # Guardian quorum, repair orchestration, state machine
├── tui/             # trust-mode UI, approval prompts, operator feedback
└── tools.rs         # typed tool schemas and execution bridges
```

### Pattern 1: Policy-First Execution
**What:** compute the policy decision before any tool runs, and treat UI trust mode as input to policy rather than as a separate source of truth [VERIFIED: workspace code].
**When to use:** every action that can mutate system state or launch a shell command [VERIFIED: workspace code].
**Example:**
```rust
// Source: [workspace code](/home/omprakash/helix-agent/agent-rs/src/security/policy.rs)
let decision = evaluate_tool_call(&func_name, &parsed_args, &policy_context);
match decision {
    PolicyDecision::Allow => {}
    PolicyDecision::RequireApproval { .. } => { /* request human confirmation */ }
    PolicyDecision::Deny { .. } => return ToolResult { success: false, output: "denied".into() },
}
```

### Pattern 2: Typed Quorum Votes
**What:** every Guardian agent returns the same typed vote schema so the coordinator can aggregate votes without parsing freeform prose [CITED: docs.rs/schemars/latest/schemars/; CITED: docs.rs/async-openai/latest/async_openai/].
**When to use:** any high-risk action that needs consensus rather than a single model judgment [VERIFIED: workspace code].
**Example:**
```rust
// Source: [docs.rs/schemars/latest/schemars/](https://docs.rs/schemars/latest/schemars/)
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VoteEnvelope {
    action_id: String,
    verdict: String,
    confidence: f32,
}

let _schema = schema_for!(VoteEnvelope);
```

### Anti-Patterns to Avoid
- **Prompt-only safety:** if Safe Mode is only a prompt instruction, the runtime can still execute the action; put the gate in `PolicyContext` and `PolicyEngine` instead [VERIFIED: workspace code].
- **Freeform vote text:** if Guardian agents answer in prose, the coordinator will eventually misparse or over-trust the output; use typed vote envelopes instead [CITED: docs.rs/schemars/latest/schemars/].
- **UI-only trust state:** if the TUI toggle is not propagated into backend policy, the visible mode and the actual execution mode will drift [VERIFIED: workspace code].

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Local LLM client plumbing | A custom HTTP client and hand-built request shapes for every provider | `async-openai` with OpenAI-compatible providers and BYOT | It already supports compatible providers, configurable paths, and typed or JSON payloads [CITED: docs.rs/async-openai/latest/async_openai/]. |
| Vote payload validation | Ad hoc string parsing or regex-based consensus logic | `serde` + `schemars` + `serde_json` | Schema-derived payloads stay aligned with Serde and fail fast on shape drift [CITED: docs.rs/schemars/latest/schemars/]. |
| Parallel Guardian fan-out | Manual thread management or blocking loops | Tokio tasks plus `join!`, `spawn`, and `timeout` | Tokio is the standard async runtime and its task/time primitives are designed for this pattern [CITED: docs.rs/tokio/latest/tokio/]. |
| Sandbox execution | Shelling out to Docker CLI strings or custom container wrappers | `bollard` or the existing `DockerSandbox` layer | Bollard is the async Docker/Podman client and the repo already centralizes sandbox execution around Docker [CITED: docs.rs/bollard/latest/bollard/; VERIFIED: workspace code]. |
| Persistent policy history | Flat files or ad hoc logs for audit decisions | `rusqlite` append-only tables | SQLite transactions give local durability without introducing a server dependency [CITED: docs.rs/rusqlite/latest/rusqlite/]. |

**Key insight:** the hard part is not voting or trust toggles; it is preserving a single authoritative policy gate between model output and execution [VERIFIED: workspace code].

## Common Pitfalls

### Pitfall 1: Quorum drift
**What goes wrong:** agents vote on different question framings or output shapes, so the coordinator cannot compare the votes reliably [CITED: docs.rs/schemars/latest/schemars/].
**Why it happens:** freeform text and inconsistent schemas make consensus look stronger than it is [CITED: docs.rs/async-openai/latest/async_openai/].
**How to avoid:** use one vote schema, one risk taxonomy, and one quorum threshold per action class [VERIFIED: workspace code].
**Warning signs:** votes are manually reworded before aggregation or the coordinator needs string matching to decide [VERIFIED: workspace code].

### Pitfall 2: Blocklist bypass through another path
**What goes wrong:** a command is blocked in one UI path but can still be executed via another tool or execution branch [VERIFIED: workspace code].
**Why it happens:** duplicate policy logic and prompt-level filtering create inconsistent enforcement [VERIFIED: workspace code].
**How to avoid:** keep the blocklist in the policy engine and call it from every execution path, including approvals and automated repair tools [VERIFIED: workspace code].
**Warning signs:** the same destructive command is checked in multiple places with slightly different matching logic [VERIFIED: workspace code].

### Pitfall 3: Safe Mode that is only cosmetic
**What goes wrong:** the UI shows a safe state but `PolicyContext` still allows execution as if Auto Mode were enabled [VERIFIED: workspace code].
**Why it happens:** the settings surface is updated without threading the value through runtime policy [VERIFIED: workspace code].
**How to avoid:** treat trust mode as persisted policy input and re-evaluate before every tool call [VERIFIED: workspace code].
**Warning signs:** the toggle changes labels in the TUI but not `PermissionTier` or `require_confirmation` [VERIFIED: workspace code].

### Pitfall 4: TOCTOU between decision and execution
**What goes wrong:** a model is allowed to vote on a command that later changes before execution, or the decision is cached after the input changes [VERIFIED: workspace code].
**Why it happens:** the quorum step and the execution step are not bound to the same action hash [VERIFIED: workspace code].
**How to avoid:** hash the canonical action payload and bind the audit record, vote set, and execution result to that hash [VERIFIED: workspace code; CITED: docs.rs/rusqlite/latest/rusqlite/].
**Warning signs:** audit entries cannot be matched back to a single canonical action [VERIFIED: workspace code].

## Code Examples

Verified patterns from official sources:

### Parallel Guardian probes
```rust
// Source: [docs.rs/tokio/latest/tokio/](https://docs.rs/tokio/latest/tokio/)
let (a, b, c) = tokio::join!(probe_a(), probe_b(), probe_c());
```

### OpenAI-compatible client for local providers
```rust
// Source: [docs.rs/async-openai/latest/async_openai/](https://docs.rs/async-openai/latest/async_openai/)
use async_openai::{Client, config::{Config, OpenAIConfig}};

let cfg = Box::new(OpenAIConfig::default()) as Box<dyn Config>;
let client: Client<Box<dyn Config>> = Client::with_config(cfg);
```

### Schema-backed vote envelope
```rust
// Source: [docs.rs/schemars/latest/schemars/](https://docs.rs/schemars/latest/schemars/)
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GuardianVote {
    action_id: String,
    verdict: String,
}

let _schema = schema_for!(GuardianVote);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Prompt-only approval text | Policy-engine enforcement with typed allow / deny / approval decisions | Current repo design [VERIFIED: workspace code] | Makes the trust mode enforceable outside the model. |
| Single-model opinion on risky actions | Parallel specialist calls with quorum evaluation | Current orchestration pattern [VERIFIED: workspace code; CITED: docs.rs/tokio/latest/tokio/] | Reduces single-model false positives and gives the coordinator a stable decision surface. |
| Freeform JSON-ish output | Serde + schema-derived payloads | Current Rust stack [CITED: docs.rs/schemars/latest/schemars/] | Lowers parse ambiguity and catches shape drift early. |

**Deprecated/outdated:**
- Prompt-only Safe Mode is not sufficient for non-bypassable execution control [VERIFIED: workspace code].
- Ad hoc destructive-command checks in multiple places are riskier than a single policy gate [VERIFIED: workspace code].

## Assumptions Log

> All claims in this research were either verified from the workspace, confirmed in official docs, or checked against crates.io. No unresolved [ASSUMED] claims remain.

## Open Questions

1. **What is the quorum threshold for a high-risk action?**
   - What we know: the current codebase already supports approval gates and transactional rollback [VERIFIED: workspace code].
   - What’s unclear: whether Guardian should require unanimity, majority, or a weighted confidence threshold for different action classes.
   - Recommendation: lock the threshold per risk tier before implementation so the vote schema and tests are stable.

2. **Should Auto Mode still require human approval for the most destructive actions?**
   - What we know: the repo already blocks known destructive commands at the policy layer [VERIFIED: workspace code].
   - What’s unclear: whether Auto Mode is full autonomy for all non-blocked actions or still a gated fast path for only routine maintenance.
   - Recommendation: define Auto Mode semantics in the plan, not in prompts, so the policy layer can enforce them consistently.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Cargo | Rust build and test workflow | ✓ | 1.95.0 | — |
| rustc | Rust compilation | ✓ | 1.95.0 | — |
| Docker | Sandbox-backed execution and container tests | ✓ | 29.2.1 | — |
| Docker daemon | Actual sandbox execution | ✓ | 29.4.1 | — |
| journalctl | Linux log introspection | ✓ | systemd 260 | — |
| python3 | Python bridge and helper scripts | ✓ | 3.14.4 | — |
| git | Repo operations and phase commits | ✓ | 2.54.0 | — |

**Missing dependencies with no fallback:**
- None — the needed local build/runtime tools are present on this machine [VERIFIED: workspace environment probes].

**Missing dependencies with fallback:**
- None — the phase does not currently require a missing external runtime [VERIFIED: workspace environment probes].

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust integration/unit tests via `cargo test` [VERIFIED: workspace Cargo.toml] |
| Config file | `agent-rs/Cargo.toml` and workspace test discovery [VERIFIED: workspace code] |
| Quick run command | `cargo test -p agent-rs risk:: -- --nocapture` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Requirement | Behavior | Test Type | Automated Command | File Exists? |
|------------|----------|-----------|-------------------|-------------|
| Safe Mode / Auto Mode trust tier | Trust-mode value propagates into `PolicyContext` and execution gating | unit | `cargo test -p agent-rs test_tool_permission_tier -- --nocapture` | ✅ existing config/state plumbing, new end-to-end test needed |
| Guardian multi-agent voting | High-risk actions are blocked until quorum is satisfied | unit/integration | `cargo test -p agent-rs guardian -- --nocapture` | ❌ Wave 0 gap |
| Hard blocklist enforcement | Destructive commands are denied even if the UI or model requests them | unit | `cargo test -p agent-rs blocked_command_reason -- --nocapture` | ✅ existing policy tests, new bypass test needed |
| Rollback on failed repair | Transactional repair restores state on validation failure | unit | `cargo test -p agent-rs test_safety_loop_rollback_on_failure -- --nocapture` | ✅ existing test in `agent_rs::agent_core::repair::workflow` |

### Sampling Rate
- **Per task commit:** `cargo test -p agent-rs risk:: -- --nocapture`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** full workspace suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Add Guardian quorum tests around conflicting votes, threshold edges, and deterministic tie handling.
- [ ] Add Safe Mode / Auto Mode propagation tests that prove the UI setting reaches `PolicyContext` and cannot be bypassed by a direct tool call.
- [ ] Add a bypass regression test that asserts the hard blocklist denies blocked commands from every execution path.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Local trust-mode control does not introduce user identity authentication in this phase [VERIFIED: workspace code]. |
| V3 Session Management | no | Session handling already exists elsewhere in the app; this phase does not add a new session boundary [VERIFIED: workspace code]. |
| V4 Access Control | yes | `PermissionTier` + `PolicyEngine` + non-bypassable denylist [VERIFIED: workspace code]. |
| V5 Input Validation | yes | `schemars` + `serde_json`-backed vote and tool schemas [CITED: docs.rs/schemars/latest/schemars/]. |
| V6 Cryptography | no | No new cryptographic primitive is required; use existing audit hashing and storage primitives [VERIFIED: workspace code]. |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Prompt injection into a voting agent | Spoofing / Tampering | Typed vote envelopes, separate policy gate, and backend enforcement [VERIFIED: workspace code; CITED: docs.rs/schemars/latest/schemars/]. |
| Command injection | Tampering / Elevation | Canonicalize, allowlist, and hard-deny destructive patterns in the policy engine [VERIFIED: workspace code]. |
| Quorum poisoning | Tampering | Require consistent vote schema and bind votes to a canonical action hash [VERIFIED: workspace code; CITED: docs.rs/tokio/latest/tokio/]. |
| Audit tampering | Repudiation | Append-only SQLite-backed audit writes and hash-chained events [VERIFIED: workspace code; CITED: docs.rs/rusqlite/latest/rusqlite/]. |
| Sandbox escape attempt | Elevation | Docker-backed sandboxing through `bollard`/`DockerSandbox` and explicit path control [VERIFIED: workspace code; CITED: docs.rs/bollard/latest/bollard/]. |

## Sources

### Primary (HIGH confidence)
- [agent-rs/src/security/policy.rs](/home/omprakash/helix-agent/agent-rs/src/security/policy.rs)
- [agent-rs/src/agent_core/tool_runtime.rs](/home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs)
- [agent-rs/src/tui/state.rs](/home/omprakash/helix-agent/agent-rs/src/tui/state.rs)
- [agent-rs/src/agent_core/repair/workflow.rs](/home/omprakash/helix-agent/agent-rs/src/agent_core/repair/workflow.rs)
- [scripts/config.py](/home/omprakash/helix-agent/scripts/config.py)
- [scripts/model_install.py](/home/omprakash/helix-agent/scripts/model_install.py)
- [docs.rs/async-openai/latest/async_openai/](https://docs.rs/async-openai/latest/async_openai/)
- [docs.rs/tokio/latest/tokio/](https://docs.rs/tokio/latest/tokio/)
- [docs.rs/schemars/latest/schemars/](https://docs.rs/schemars/latest/schemars/)
- [docs.rs/axum/latest/axum/](https://docs.rs/axum/latest/axum/)
- [docs.rs/rusqlite/latest/rusqlite/](https://docs.rs/rusqlite/latest/rusqlite/)
- [docs.rs/bollard/latest/bollard/](https://docs.rs/bollard/latest/bollard/)

### Secondary (MEDIUM confidence)
- [crates.io registry via cargo search](https://crates.io/) [VERIFIED in session: async-openai 0.37.0, tokio 1.52.2, axum 0.8.9, bollard 0.21.0, reqwest 0.13.3, inquire 0.9.4, ratatui 0.30.0, serde 1.0.228]

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - current upstream docs and crates.io versions were checked for the core libraries; the repository already uses the same architectural family [CITED: docs.rs/async-openai/latest/async_openai/; CITED: docs.rs/tokio/latest/tokio/; VERIFIED: workspace code].
- Architecture: HIGH - the repo already exposes the exact policy and approval seams this phase needs [VERIFIED: workspace code].
- Pitfalls: MEDIUM - the safety pitfalls are well supported by the code path, but the final quorum threshold and Auto Mode semantics still need plan decisions [VERIFIED: workspace code].

**Research date:** 2026-05-08
**Valid until:** 2026-06-07