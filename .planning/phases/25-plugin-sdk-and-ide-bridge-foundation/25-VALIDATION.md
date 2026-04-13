---
phase: 25
slug: plugin-sdk-and-ide-bridge-foundation
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-13
---

# Phase 25 - Validation Strategy

> Nyquist validation reconstruction from completed execution artifacts (State B).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust integration tests via `cargo test` |
| **Config file** | `agent-rs/Cargo.toml` |
| **Quick run command** | `cargo test -q --manifest-path agent-rs/Cargo.toml --test plugin_sdk_ide_bridge_validation` |
| **Full suite command** | `cargo test -q --manifest-path agent-rs/Cargo.toml` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -q --manifest-path agent-rs/Cargo.toml --test plugin_sdk_ide_bridge_validation`
- **After every plan wave:** Run `cargo test -q --manifest-path agent-rs/Cargo.toml`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 25-01-01 | 01 | 1 | ENT-02 | T-25-01, T-25-02 | Tool registry exposes all built-ins and preserves persona-based tool filtering | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test plugin_sdk_ide_bridge_validation` | yes | green |
| 25-01-02 | 01 | 1 | IDE-01 | T-25-03 | Bridge status/context endpoints return local, deterministic discovery metadata | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test plugin_sdk_ide_bridge_validation` | yes | green |
| 25-01-03 | 01 | 1 | ENT-02, IDE-01 | T-25-03 | `/v1/tools` discovery payload matches the registry contract used by the IDE bridge | integration | `cargo test -q --manifest-path agent-rs/Cargo.toml --test plugin_sdk_ide_bridge_validation` | yes | green |

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
