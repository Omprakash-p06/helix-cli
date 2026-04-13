# Phase 24: Enterprise Audit Log MVP - Research

**Researched:** 2026-04-13
**Domain:** Structured audit logging for tool execution and security policy decisions
**Confidence:** HIGH (code integration points), MEDIUM (initial retention/query defaults)

## Summary

Phase 24 should add a single, structured audit pipeline that records both:
1. policy decisions (`Allow`, `RequireApproval`, `Deny`) and
2. tool execution outcomes (success/failure, duration, output hash/size),
across terminal and web execution paths.

The current code already has the right insertion points:
- policy gate in `agent-rs/src/main.rs` and `agent-rs/src/server.rs` via `evaluate_tool_call`.
- tool execution wrappers and return payloads in `execute_tool_async` / `execute_tool_sync`.
- deterministic denial/approval message templates already shared by terminal and web paths.

What is missing for ENT-01:
- No durable audit event store.
- No unified audit event schema.
- No query API/CLI surface for audit review.
- No tamper-evident linkage across events.

Primary recommendation:
1. Implement an append-only SQLite audit store with a strict event schema.
2. Emit audit events at pre-execution policy decision time and post-execution result time.
3. Add hash chaining (`prev_hash`, `event_hash`) for basic tamper evidence.
4. Add a minimal query surface (time range, actor/path, decision/outcome filters) for enterprise-readiness.

## Standard Stack

### Core (Use)
| Library / Module | Purpose | Why |
|---|---|---|
| `rusqlite` (Rust) | Durable, queryable local audit database | Minimal operational overhead, strong local query support, good fit for MVP |
| `serde` + `serde_json` | Structured payload serialization for args/result metadata | Already in stack and heavily used across runtime |
| `sha2` (Rust) | Event payload digest + hash chaining | Provides deterministic tamper-evidence primitives |
| Existing policy/execution hooks in `main.rs` and `server.rs` | Event emission points | Already centralize policy decisions and tool calls |

### Optional (Only if needed)
| Library | Use case |
|---|---|
| `chrono` | RFC3339 timestamps for human-facing exports |
| `csv` | Fast export path for external audit review |

## Architecture Patterns

### Pattern 1: Single Audit Event Envelope

**What:** Use one event envelope for both policy and execution events.

Recommended fields:
- `event_id` (UUID)
- `timestamp_utc`
- `request_path` (`terminal` or `web`)
- `actor` (agent persona + mode)
- `event_type` (`policy_decision` or `tool_execution`)
- `tool_name`
- `decision` (`allow`/`approval_required`/`deny`) for policy events
- `outcome` (`success`/`failure`) for execution events
- `args_hash` (SHA-256 of normalized tool args)
- `output_hash` (SHA-256 of tool output, optional)
- `reason_code` and `remediation` where applicable
- `duration_ms` where applicable
- `prev_hash` and `event_hash`

**Why:** One schema keeps terminal/web logs queryable without path-specific parsers.

### Pattern 2: Write-Ahead Emission (Policy Then Execution)

**What:** Emit two event classes in order:
1. Policy decision event before tool execution starts.
2. Tool execution event after completion (or timeout/error).

**Why:** Captures intent and outcome separately, allowing forensic reconstruction when execution never happens due to policy denials.

### Pattern 3: Tamper-Evident Hash Chain

**What:** Each event includes `prev_hash`, and `event_hash = sha256(prev_hash + canonical_event_json)`.

**Why:** MVP-level integrity check for append-only logs without requiring external ledger infrastructure.

### Pattern 4: Bounded Sensitive Data Exposure

**What:** Store hashes and selective metadata by default, not raw secrets.

Recommendation:
- Hash full command args and tool outputs.
- Persist redacted previews only (size-limited, with secret/token redaction pass).
- Never persist raw secret-bearing env output (`printenv`, keys) in clear text.

**Why:** Enterprise audit needs traceability without creating a new secret-leak surface.

### Pattern 5: Query Surface for Operators

**What:** Add a minimal audit query interface (CLI and/or local endpoint).

Recommended query filters:
- time range
- request path (`terminal`/`web`)
- `tool_name`
- `decision` / `outcome`
- `reason_code`

**Why:** ENT-01 explicitly requires logs to be queryable, not just written.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Queryable storage | custom JSONL scan engine | SQLite table + indexes | Queryability and filtering are native, reliable, and simpler |
| Hashing | bespoke checksum routine | `sha2` SHA-256 | Standard, audited cryptographic primitive |
| Event schema | path-specific ad hoc structs | one canonical envelope for policy/execution | Prevents drift between terminal and web paths |
| Retention | manual file truncation logic | SQL retention job/delete policy | Deterministic and index-aware cleanup |
| Report extraction | one-off parser scripts | SQL + optional CSV export command | Reusable enterprise workflows |

## Common Pitfalls

### 1) Logging only failures
Audit requirements need full decision trace, including allowed operations.

### 2) Logging only post-execution events
Without policy-decision events, denied calls disappear from the audit trail.

### 3) Storing raw sensitive outputs
Audit logs can become a high-value secret target if raw outputs are persisted unredacted.

### 4) Divergent terminal vs web schemas
If event shapes differ by path, cross-path investigations become expensive and error-prone.

### 5) Missing indexes for core filters
Without indexes on time/path/tool/decision, query performance degrades quickly.

## Code Examples

### Example 1: Canonical event envelope

```rust
#[derive(Serialize, Deserialize)]
struct AuditEvent {
    event_id: String,
    timestamp_utc: String,
    request_path: String,
    actor: String,
    event_type: String,
    tool_name: String,
    decision: Option<String>,
    outcome: Option<String>,
    reason_code: Option<String>,
    remediation: Option<String>,
    args_hash: String,
    output_hash: Option<String>,
    duration_ms: Option<u64>,
    prev_hash: String,
    event_hash: String,
}
```

### Example 2: Policy decision audit hook

```rust
let decision = evaluate_tool_call(tool_name, &parsed_args, &policy_context);
audit_store.append_policy_event(
    request_path,
    actor,
    tool_name,
    &decision,
    sha256_json(&parsed_args),
)?;
```

### Example 3: Execution outcome audit hook

```rust
let started = std::time::Instant::now();
let result = execute_tool_sync(tc, dangerous, require_confirmation, &policy_context);
audit_store.append_execution_event(
    request_path,
    actor,
    tool_name,
    result.success,
    sha256_bytes(result.output.as_bytes()),
    started.elapsed().as_millis() as u64,
)?;
```

### Example 4: Minimal query API

```rust
fn query_by_window_and_decision(
    conn: &rusqlite::Connection,
    start_ts: &str,
    end_ts: &str,
    decision: Option<&str>,
) -> rusqlite::Result<Vec<AuditEvent>> {
    // SELECT ... WHERE timestamp_utc BETWEEN ? AND ? AND (? IS NULL OR decision = ?)
    unimplemented!()
}
```

## Source Evidence

- `agent-rs/src/main.rs` enforces policy decisions before tool execution and returns structured denial/approval strings.
- `agent-rs/src/server.rs` mirrors the same policy gate for web path tool calls.
- `agent-rs/tests/security_guardrails.rs` confirms parity between terminal and web guardrail messaging templates.
- Current runtime includes log files for server process stdout/stderr, but no structured audit store for tool/policy events.

## Implementation Readiness

Ready for `/gsd-plan-phase 24`.

Recommended planning order:
1. Define audit event schema + SQLite storage module + migration/index setup.
2. Integrate policy-decision and execution-outcome audit hooks in terminal and web paths.
3. Add query interface (CLI/API) with core filters and retention controls.
4. Add tests for parity across terminal/web, hash-chain continuity, and query correctness.
