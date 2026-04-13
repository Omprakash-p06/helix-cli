---
phase: 26
slug: open-source-integration-references
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-13
---

# Phase 26 - Validation Strategy

> Nyquist validation reconstruction from completed execution artifacts (State B).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust integration tests via `cargo test` |
| **Config file** | `agent-rs/Cargo.toml` |
| **Quick run command** | `cargo test -q --manifest-path agent-rs/Cargo.toml --test streaming_tui_refinement --test tool_runtime_contracts --test security_guardrails` |
| **Full suite command** | `cargo test -q --manifest-path agent-rs/Cargo.toml` |
| **Estimated runtime** | ~40 seconds |

---

## Sampling Rate

- **After every task commit:** Run the targeted phase suites for the changed plan area
- **After every plan wave:** Run `cargo test -q --manifest-path agent-rs/Cargo.toml`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 26-01-01 | 01 | 1 | STREAM-01 | T-26-01, T-26-03 | Streaming emits tokens immediately from SSE bytes with UTF-8-safe reconstruction | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test streaming_tui_refinement` | yes | green |
| 26-01-02 | 01 | 1 | UX-01 | T-26-02 | TUI tool timeline and collapsible output remain readable during live streaming | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test streaming_tui_refinement` | yes | green |
| 26-01-03 | 01 | 1 | STREAM-01, UX-01 | T-26-01, T-26-02 | Full Rust suite validates no regressions in streaming or TUI rendering | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml` | yes | green |
| 26-02-01 | 02 | 2 | TOOL-01 | T-26-04, T-26-06 | Shared async tool runtime executes with lifecycle events and timeout enforcement | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test tool_runtime_contracts` | yes | green |
| 26-02-02 | 02 | 2 | CODE-01 | T-26-04, T-26-06 | Terminal, web, and TUI paths share the centralized runtime contract without diverging behavior | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test tool_runtime_contracts --test security_guardrails` | yes | green |
| 26-02-03 | 02 | 2 | TOOL-01, CODE-01 | T-26-05, T-26-06 | Full Rust suite validates unified runtime and security guardrails across all callers | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml` | yes | green |

Status legend: pending, green, red, flaky

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-04-13

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands or Wave 0 dependencies
- [x] Sampling continuity preserved (no 3 consecutive tasks without automation)
- [x] Wave 0 not required for this phase
- [x] No watch-mode flags in validation commands
- [x] Feedback latency below 60 seconds
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-13
