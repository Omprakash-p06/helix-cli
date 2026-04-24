# Phase 03: Guided Repair & Human-Approved Fixes - Research

**Researched:** 2026-04-25  
**Domain:** Human-in-the-loop repair orchestration for a Rust TUI/Web UI agent [VERIFIED: /home/omprakash/helix-agent/README.md; /home/omprakash/helix-agent/agent-rs/Cargo.toml; /home/omprakash/helix-agent/web-ui/package.json]  
**Confidence:** HIGH

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FIX-01 | Approval Gate (Human-in-the-Loop) | Use the existing shared `ToolRuntime` / policy decision path and elevate `PolicyDecision::RequireApproval` into a real pause/resume gate in both TUI and web paths [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs; /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs; /home/omprakash/helix-agent/agent-rs/src/main.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs]. |
| FIX-02 | Rollback Snapshots | Reuse the existing session/audit persistence surfaces, but add a pre-repair snapshot abstraction so fixes can be rolled back before any state change runs [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs; /home/omprakash/helix-agent/agent-rs/src/session.rs; /home/omprakash/helix-agent/agent-rs/src/tools.rs]. |
| FIX-03 | Confidence Scoring | Keep confidence as a first-class structured field on repair recommendations, separate from policy risk and approval state [ASSUMED]. |
</phase_requirements>

## Summary

The repo already has the right control plane for this phase: a single policy decision point in `ToolRuntime::execute`, a `PolicyDecision::RequireApproval` branch in the security layer, audit logging with hash chaining, resumable session snapshots, a ratatui-based TUI, and an Axum SSE web server [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs; /home/omprakash/helix-agent/agent-rs/src/security/policy.rs; /home/omprakash/helix-agent/agent-rs/src/audit.rs; /home/omprakash/helix-agent/agent-rs/src/session.rs; /home/omprakash/helix-agent/agent-rs/src/tui.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs]. The missing piece is not policy classification; it is turning approval from a text message into a real stateful gate that pauses repair execution until the user explicitly accepts or rejects the exact action payload [ASSUMED].

The cleanest implementation path is to keep TUI and web approval surfaces on the same event model instead of creating a second confirmation system. The TUI already has modal state and overlay rendering, and the web server already forwards tool lifecycle events over SSE, so Phase 03 should extend those event streams with an approval-request payload that carries the normalized command, risk label, snapshot reference, and confidence data [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs; https://docs.rs/axum/latest/axum/response/sse/index.html].

**Primary recommendation:** implement approval as a shared `ToolRuntime` state transition, then add a provider-based snapshot step before any repair tool executes; do not bolt the gate directly into UI widgets or individual repair tools [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs; /home/omprakash/helix-agent/agent-rs/src/tools.rs].

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | `0.8.9` [VERIFIED: crates.io] | HTTP + SSE server for the web approval path [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs] | Upstream SSE support is first-class and matches the current web bridge pattern [CITED: https://docs.rs/axum/latest/axum/response/sse/index.html]. |
| `ratatui` | `0.30.0` [VERIFIED: crates.io] | Full-frame TUI rendering for approval modals and rollback previews [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs] | Ratatui documents the exact `Terminal::draw` / widget / layout pattern already used by the repo [CITED: https://docs.rs/ratatui/latest/ratatui/]. |
| `crossterm` | `0.29.0` [VERIFIED: crates.io] | Raw mode, alternate screen, and keyboard events for interactive confirmation [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs] | Crossterm provides the terminal control primitives Ratatui expects, including alternate-screen and event handling [CITED: https://docs.rs/crossterm/latest/crossterm/]. |
| `tokio` | `1.52.1` [VERIFIED: crates.io] | Async orchestration and event fan-out for approval / repair lifecycle [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/main.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs] | The existing runtime already relies on Tokio for streaming, channels, and task spawning [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs]. |
| `rusqlite` | `0.39.0` [VERIFIED: crates.io] | Durable audit / metadata persistence for repair history [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs] | The repo already uses SQLite for tamper-evident audit rows, so it is the natural place for approval and snapshot metadata [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs]. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `reqwest` | `0.13.2` [VERIFIED: crates.io] | Control-plane HTTP client for health checks and repair orchestration [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/main.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs] | Use for internal readiness probes and repair coordination, not for user-facing command execution. |
| `bollard` | `0.20.2` [VERIFIED: crates.io] | Existing Docker sandbox integration for command execution [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/sandbox.rs] | Use when a repair step must stay inside the sandbox boundary already established in phase 01. |
| `react` | `19.2.5` [VERIFIED: npm registry] | Web approval UI component layer [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | Use for the browser approval pane if Phase 03 needs web parity. |
| `react-dom` | `19.2.5` [VERIFIED: npm registry] | Browser rendering for the web UI [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | Use with React in the existing Vite app. |
| `vite` | `8.0.10` [VERIFIED: npm registry] | Frontend build/dev server [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | Use for the current web-ui development loop. |
| `react-markdown` | `10.1.0` [VERIFIED: npm registry] | Safe-ish markdown rendering for rich recommendation text [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | Use when the approval pane needs to show model rationale or diffs. |
| `rehype-raw` | `7.0.0` [VERIFIED: npm registry] | Markdown HTML handling in the web UI [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | Use only if the web UI must render trusted HTML fragments. |
| `lucide-react` | `1.11.0` [VERIFIED: npm registry] | Small icon set for web affordances [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | Use for approval / risk icons in the existing web shell. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `ratatui` modal + shared event state | `dialoguer` / ad hoc terminal prompts | Simpler for a one-off prompt, but it fragments the UI and does not fit the existing full-frame TUI already in the repo [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs]. |
| Axum SSE for browser updates | A polling endpoint | Polling is easier to prototype, but the repo already uses SSE for streaming events, so a shared event stream is the lower-risk path [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs; CITED: https://docs.rs/axum/latest/axum/response/sse/index.html]. |
| SQLite-backed metadata | JSON-only files | JSON is simpler, but the project already uses SQLite for tamper-evident audit history and queryable state [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs]. |

**Installation:**
```bash
# No new dependency is required for the core phase 03 gate; use the existing Rust and web-ui stacks.
```

## Architecture Patterns

### Recommended Project Structure
```text
agent-rs/src/
├── security/          # policy evaluation, approval decisions, sandbox checks
├── agent_core/        # shared tool execution and lifecycle control
├── tui.rs             # terminal approval modal and rollback preview
├── server.rs          # web approval SSE bridge
├── audit.rs           # tamper-evident event log
├── session.rs         # resumable snapshots / session state
└── tools.rs           # repair actions and tool schemas
web-ui/src/
└── App.tsx            # browser approval and event feed UI
```

### Pattern 1: Policy-first pause/resume
**What:** Evaluate repair actions in the shared runtime, then pause on `RequireApproval` instead of returning a dead-end failure [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs; /home/omprakash/helix-agent/agent-rs/src/security/policy.rs].  
**When to use:** Any repair step that can modify state, restart services, or install packages [VERIFIED: /home/omprakash/helix-agent/.planning/REQUIREMENTS.md].  
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/response/sse/index.html
match evaluate_tool_call(tool_name, &args, &policy_context) {
    PolicyDecision::Allow => run_tool_now(),
    PolicyDecision::RequireApproval { reason_code, message } => {
        emit_approval_request(reason_code, message);
        wait_for_user_confirmation();
    }
    PolicyDecision::Deny { .. } => reject_immediately(),
}
```

### Pattern 2: Single event bus for TUI and web
**What:** Use one lifecycle event stream for tool start, approval request, status, and result; render that stream differently in TUI and browser, but keep the payload shape shared [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs; /home/omprakash/helix-agent/agent-rs/src/tui.rs].  
**When to use:** Whenever the same action must be visible in both terminal and web modes [VERIFIED: /home/omprakash/helix-agent/README.md; /home/omprakash/helix-agent/agent-rs/src/main.rs].  
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/response/sse/index.html
async fn approval_feed() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = ReceiverStream::new(rx).map(|payload| Ok(Event::default().json_data(payload).unwrap()));
    Sse::new(stream)
}
```

### Pattern 3: Snapshot-before-repair
**What:** Capture a restorable snapshot before any repair command runs, then attach the snapshot identifier to the approval record and audit trail [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs; /home/omprakash/helix-agent/agent-rs/src/session.rs].  
**When to use:** File edits, permission changes, package installs, and service restarts [VERIFIED: /home/omprakash/helix-agent/.planning/REQUIREMENTS.md].  
**Example:**
```text
1. Collect target paths and packages.
2. Create snapshot and record snapshot_id.
3. Show approval modal with snapshot summary.
4. Execute repair only after explicit approval.
5. Append outcome to audit log.
```

### Anti-Patterns to Avoid
- **Printing approval text without pausing execution:** the current runtime already returns `RequireApproval`, so the next step is to hold the tool rather than just surfacing a message [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs].
- **Duplicating approval logic in tool implementations:** approval belongs in the shared runtime and policy layer, not inside each repair tool [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tools.rs].
- **Using different approval rules in TUI and web UI:** the repo already has two frontends, but one policy engine [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/main.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs].
- **Running rollback after the fix:** snapshot creation must happen before the write/restart/install step [ASSUMED].

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Command safety | Ad hoc shell parsing or string matching | `PolicyEngine::validate_command` and `evaluate_command_risk` [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs] | The existing policy layer already blocks metacharacters, dangerous commands, and path traversal. |
| Approval UI | Separate approval parser for each frontend | Shared approval event payload plus TUI modal / SSE event [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs] | One state machine is easier to audit and test. |
| Rollback storage | Custom diff engine from scratch | Snapshot provider abstraction backed by filesystem or OS-native snapshot primitives [ASSUMED] | Repair rollback has too many edge cases to trust a bespoke implementation. |
| Audit trail | Plain log lines | `AuditStore` with hash chaining [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs] | The project already needs tamper evidence and queryable metadata. |
| Browser transport | Polling loop for repair progress | Axum SSE [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs; CITED: https://docs.rs/axum/latest/axum/response/sse/index.html] | The current web path already streams tool lifecycle updates. |

**Key insight:** this phase is mostly an orchestration problem, not a new algorithm problem. The repo already owns the hard parts of policy classification, auditability, and UI event delivery; Phase 03 should connect them into a gated repair transaction [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs; /home/omprakash/helix-agent/agent-rs/src/audit.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs; /home/omprakash/helix-agent/agent-rs/src/tui.rs].

## Common Pitfalls

### Pitfall 1: Approval based only on what the UI shows
**What goes wrong:** The user approves a visible command, but execution later uses a mutated payload or a different path.  
**Why it happens:** UI text is easier to store than the normalized command hash [ASSUMED].  
**How to avoid:** Bind approval to the normalized command payload, tool name, and snapshot reference before execution [ASSUMED].  
**Warning signs:** the approval record does not include a digest or an immutable action id [ASSUMED].

### Pitfall 2: Snapshot created too late
**What goes wrong:** The repair mutates state before the rollback baseline exists.  
**Why it happens:** It is tempting to snapshot after the policy check but before the tool call wrapper finishes.  
**How to avoid:** snapshot first, then show the approval prompt, then execute [ASSUMED].  
**Warning signs:** snapshot timestamps are newer than the first modification event [ASSUMED].

### Pitfall 3: TUI and web drift
**What goes wrong:** The browser says one thing and the terminal says another.  
**Why it happens:** each frontend starts owning its own business logic.  
**How to avoid:** keep approval/risk state in shared Rust types and render them in both frontends [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs; /home/omprakash/helix-agent/agent-rs/src/tui.rs].  
**Warning signs:** duplicated prompt text, duplicated approval rules, or separate retry behavior [ASSUMED].

### Pitfall 4: Confidence mixed with permission
**What goes wrong:** A low-confidence repair still executes because the approval gate only looks at policy, not recommendation quality.  
**Why it happens:** confidence and permission are easy to conflate in a single boolean field [ASSUMED].  
**How to avoid:** keep recommendation confidence, policy risk, and user approval as separate fields [ASSUMED].  
**Warning signs:** the UI cannot explain whether a warning came from the model, policy engine, or snapshot provider [ASSUMED].

## Code Examples

Verified patterns from official sources and the current repo:

### Axum SSE approval stream
```rust
// Source: https://docs.rs/axum/latest/axum/response/sse/index.html
use axum::response::sse::{Event, Sse};
use futures_util::stream::Stream;

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|payload| Ok(Event::default().json_data(payload).unwrap()));
    Sse::new(stream)
}
```

### Ratatui overlay rendering
```rust
// Source: https://docs.rs/ratatui/latest/ratatui/
terminal.draw(|frame| {
    let popup_area = centered_rect(60, 40, frame.area());
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(approval_modal, popup_area);
})?;
```

### Crossterm terminal lifecycle
```rust
// Source: https://docs.rs/crossterm/latest/crossterm/
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}};

enable_raw_mode()?;
execute!(stdout, EnterAlternateScreen)?;
// draw interactive UI here
execute!(stdout, LeaveAlternateScreen)?;
disable_raw_mode()?;
```

### Shared runtime approval branch
```rust
// Source: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs
match decision {
    PolicyDecision::Allow => {}
    PolicyDecision::RequireApproval { reason_code, message } => {
        return ToolResult {
            success: false,
            output: format!("[Approval Required: {}] {}", reason_code, message),
        };
    }
    PolicyDecision::Deny { reason_code, message, remediation } => {
        return ToolResult {
            success: false,
            output: format!("[Policy Denied: {}] {} Remediation: {}", reason_code, message, remediation),
        };
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Repo-pinned Rust/web dependencies (`axum` 0.7, `ratatui` 0.26, `crossterm` 0.27, `reqwest` 0.12.9, `react` 19.2.4, `vite` 8.0.1) [VERIFIED: /home/omprakash/helix-agent/agent-rs/Cargo.toml; /home/omprakash/helix-agent/web-ui/package.json] | Current upstream releases are `axum` 0.8.9, `ratatui` 0.30.0, `crossterm` 0.29.0, `tokio` 1.52.1, `reqwest` 0.13.2, `react` 19.2.5, `vite` 8.0.10 [VERIFIED: crates.io; npm registry] | 2025-2026 [VERIFIED: crates.io; npm registry] | The codebase is slightly behind upstream, but the architectural pattern is already the same; no dependency rewrite is required for Phase 03 [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs; /home/omprakash/helix-agent/agent-rs/src/tui.rs]. |
| Manual terminal control in scattered code paths | Shared `ratatui`/`crossterm` draw loop and event handling | Current repo design [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs] | Modal approvals fit the existing UI architecture instead of introducing a separate prompt framework [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tui.rs; CITED: https://docs.rs/ratatui/latest/ratatui/]. |
| Ad hoc browser updates | SSE event stream from Axum | Current repo design [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/server.rs] | Approval, status, and tool lifecycle events can share one transport [CITED: https://docs.rs/axum/latest/axum/response/sse/index.html]. |

**Deprecated/outdated:**
- Treating `RequireApproval` as a terminal error instead of a pause/resume state [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs].
- Letting repair tools execute directly from UI callbacks without passing through the shared policy/runtime layer [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/tools.rs; /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs].

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Approval should pause execution and resume only after explicit user acceptance of the exact normalized payload plus snapshot reference. | Summary / Architecture Patterns | The planner may need to split the work if the UI path cannot safely hold tool execution state. |
| A2 | Snapshot support should use a provider abstraction with filesystem fallback unless the phase explicitly requires an OS-native snapshot backend. | Don't Hand-Roll / Common Pitfalls | The implementation may need to change if the target OS set is narrower than expected. |
| A3 | Confidence scoring should remain separate from policy risk and user approval state. | Phase Requirements / Common Pitfalls | The planner may need to align the schema with any pre-existing recommendation model. |
| A4 | Phase 03 should keep TUI and web parity in the same shared runtime instead of treating web as a later follow-up. | Summary / Architecture Patterns | If web parity is out of scope, the browser-facing event changes can be deferred. |

## Open Questions

1. **Should rollback snapshots cover only filesystem/config changes, or also package installations and service restarts?**
   - What we know: the phase explicitly includes restart service, fix permissions, and install package workflows [VERIFIED: /home/omprakash/helix-agent/.planning/REQUIREMENTS.md].
   - What's unclear: whether the snapshot layer must capture OS-level service state or just file/config state.
   - Recommendation: define snapshot scope per repair class before implementation.

2. **Is web approval parity mandatory in the first Phase 03 deliverable, or can the terminal gate ship first?**
   - What we know: the repo already supports both TUI and web execution paths [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/main.rs; /home/omprakash/helix-agent/agent-rs/src/server.rs].
   - What's unclear: whether the milestone expects both surfaces on day one or only one shared backend with one frontend.
   - Recommendation: keep the backend shared either way, and decide the frontend rollout order in planning.

3. **What is the confidence threshold for mandatory warning escalation?**
   - What we know: the requirement says scores below a threshold (for example, 80%) trigger mandatory extra warnings [VERIFIED: /home/omprakash/helix-agent/.planning/REQUIREMENTS.md].
   - What's unclear: the exact cut-off and whether it is static or configurable.
   - Recommendation: make the threshold a config value, but default it to the requirement's example until product decides otherwise.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-----------|-----------|---------|----------|
| Cargo / Rust toolchain | Rust repair runtime and tests [VERIFIED: /home/omprakash/helix-agent/agent-rs/Cargo.toml] | ✓ | `cargo 1.95.0` [VERIFIED: command probe] | — |
| Node.js | Web UI build and package version checks [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | ✓ | `v25.9.0` [VERIFIED: command probe] | — |
| npm | Web UI package management [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json] | ✓ | `11.13.0` [VERIFIED: command probe] | — |
| Python 3 | Existing launcher/config bridge [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/config.rs; /home/omprakash/helix-agent/scripts/config.py] | ✓ | `Python 3.14.4` [VERIFIED: command probe] | — |
| Docker | Existing sandboxed command execution path [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/sandbox.rs] | ✓ | `Docker version 29.2.1` [VERIFIED: command probe] | — |

**Missing dependencies with no fallback:**
- None verified [VERIFIED: command probe].

**Missing dependencies with fallback:**
- None verified [VERIFIED: command probe].

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust integration/unit tests via `cargo test`; web UI build/lint via `npm run build` and `npm run lint` [VERIFIED: /home/omprakash/helix-agent/agent-rs/tests/*; /home/omprakash/helix-agent/web-ui/package.json] |
| Config file | `agent-rs/Cargo.toml` and `web-ui/package.json` [VERIFIED: /home/omprakash/helix-agent/agent-rs/Cargo.toml; /home/omprakash/helix-agent/web-ui/package.json] |
| Quick run command | `cd agent-rs && cargo test -q --test security_guardrails --test tool_runtime_contracts` [VERIFIED: existing test layout] |
| Full suite command | `cd agent-rs && cargo test && cargo clippy --all-targets --all-features -- -D warnings && cd ../web-ui && npm run build && npm run lint` [VERIFIED: /home/omprakash/helix-agent/agent-rs/tests/*; /home/omprakash/helix-agent/web-ui/package.json] |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|-----------|-----------|-------------------|-------------|
| FIX-01 | Approval gate blocks state-changing repairs until the user explicitly approves the normalized action | integration / contract | `cd agent-rs && cargo test -q --test tool_runtime_contracts --test security_guardrails` | ✅ existing harness [VERIFIED: /home/omprakash/helix-agent/agent-rs/tests/tool_runtime_contracts.rs; /home/omprakash/helix-agent/agent-rs/tests/security_guardrails.rs] |
| FIX-02 | Snapshot is created before repair, and rollback metadata is persisted | integration / persistence | `cd agent-rs && cargo test -q --test test_session_persistence --test audit_log_mvp` | ✅ existing persistence tests [VERIFIED: /home/omprakash/helix-agent/agent-rs/tests/test_session_persistence.rs; /home/omprakash/helix-agent/agent-rs/tests/audit_log_mvp.rs] |
| FIX-03 | Confidence score is attached to the recommendation and escalates warnings below threshold | unit / contract | `cd agent-rs && cargo test -q --test tool_runtime_contracts` | ❌ needs new test coverage [ASSUMED] |

### Sampling Rate
- **Per task commit:** `cd agent-rs && cargo test -q --test security_guardrails --test tool_runtime_contracts`
- **Per wave merge:** `cd agent-rs && cargo test && cargo clippy --all-targets --all-features -- -D warnings && cd ../web-ui && npm run build && npm run lint`
- **Phase gate:** Full suite green before `/gsd-verify-work` [VERIFIED: /home/omprakash/helix-agent/.planning/config.json]

### Wave 0 Gaps
- [ ] `agent-rs/tests/approval_gate.rs` or equivalent - covers FIX-01 approval pause/resume semantics [ASSUMED]
- [ ] `agent-rs/tests/rollback_snapshot.rs` or equivalent - covers FIX-02 pre-repair snapshot creation and rollback metadata [ASSUMED]
- [ ] `agent-rs/tests/confidence_scoring.rs` or equivalent - covers FIX-03 confidence warnings [ASSUMED]
- [ ] Web UI test harness - none exists yet, so keep browser changes thin or add Vitest if component-level approval logic grows [VERIFIED: /home/omprakash/helix-agent/web-ui/package.json; /home/omprakash/helix-agent/web-ui/src/*]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [ASSUMED] | Local CLI/TUI workflow does not currently show a separate auth boundary [VERIFIED: /home/omprakash/helix-agent/README.md]. |
| V3 Session Management | yes [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/session.rs] | Session snapshots with schema versioning and atomic rename writes [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/session.rs]. |
| V4 Access Control | yes [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs] | Policy tier checks plus approval gating in the shared runtime [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs; /home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs]. |
| V5 Input Validation | yes [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs] | Command canonicalization, shell-argument sanitization, and path validation [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs]. |
| V6 Cryptography | yes [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs] | SHA-256 hash chaining for tamper-evident audit rows [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs]. |

### Known Threat Patterns for Rust repair workflows

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Command injection | Tampering / Elevation of Privilege | Keep the existing allowlist + metacharacter block + policy gate [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs]. |
| Path traversal | Tampering | Validate against the workspace root before any repair file access [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/security/policy.rs; /home/omprakash/helix-agent/agent-rs/src/security/sandbox.rs]. |
| Unauthorized repair execution | Elevation of Privilege | Require explicit user approval before state-changing actions and record the approval in the audit trail [ASSUMED]. |
| TOCTOU between approval and execution | Tampering | Revalidate the exact normalized payload at execution time and bind approval to the stored action digest [ASSUMED]. |
| Tampered rollback history | Repudiation | Keep the rollback metadata in the same tamper-evident audit / metadata store [VERIFIED: /home/omprakash/helix-agent/agent-rs/src/audit.rs]. |

## Sources

### Primary (HIGH confidence)
- `/home/omprakash/helix-agent/agent-rs/src/security/policy.rs` - policy decisions, command validation, allowlist, dangerous-command blocking [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/src/agent_core/tool_runtime.rs` - shared execution path and approval response handling [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/src/server.rs` - Axum SSE web bridge and tool lifecycle forwarding [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/src/tui.rs` - ratatui modal/draw/event loop patterns [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/src/audit.rs` - tamper-evident audit store [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/src/session.rs` - session snapshot persistence [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/src/security/sandbox.rs` - Docker sandboxed command execution [VERIFIED].
- `/home/omprakash/helix-agent/agent-rs/Cargo.toml` - Rust dependency baseline [VERIFIED].
- `/home/omprakash/helix-agent/web-ui/package.json` - React/Vite web UI baseline [VERIFIED].
- https://docs.rs/axum/latest/axum/response/sse/index.html - SSE response pattern [CITED].
- https://docs.rs/ratatui/latest/ratatui/ - terminal drawing, layout, and event handling pattern [CITED].
- https://docs.rs/crossterm/latest/crossterm/ - raw mode, alternate screen, and event APIs [CITED].

### Secondary (MEDIUM confidence)
- crates.io / npm registry version queries for the current stack baseline [VERIFIED].

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - the repo already uses the relevant runtime/UI/sandbox stack and the current registry versions were checked [VERIFIED].
- Architecture: HIGH - approval, audit, session, and SSE paths already exist and only need to be connected [VERIFIED].
- Pitfalls: MEDIUM - the high-risk failure modes are standard for repair workflows, but the exact thresholding / snapshot policy still needs product confirmation [ASSUMED].

**Research date:** 2026-04-25  
**Valid until:** 2026-05-25
