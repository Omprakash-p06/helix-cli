---
phase: 20
slug: security-execution-guardrails
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-13
---

# Phase 20 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | rust `cargo test` + `cargo clippy` |
| **Config file** | agent-rs/Cargo.toml |
| **Quick run command** | `cd agent-rs && cargo test --test security_guardrails -- --nocapture` |
| **Full suite command** | `cd agent-rs && cargo fmt && cargo test && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~20-30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd agent-rs && cargo test --test security_guardrails -- --nocapture`
- **After every plan wave:** Run `cd agent-rs && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 20-01-01 | 01 | 1 | SEC-01 | T-20-01/T-20-02/T-20-03 | Permission tiers and policy decision contracts are typed and deterministic. | unit | `cd agent-rs && cargo test security::policy -- --nocapture` | agent-rs/src/security/policy.rs | ✅ green |
| 20-01-02 | 01 | 1 | SEC-01 | T-20-01 | Config bridge maps `TOOL_PERMISSION_TIER` to typed runtime tier with fallback behavior. | unit | `cd agent-rs && cargo test config -- --nocapture` | agent-rs/src/config.rs | ✅ green |
| 20-02-01 | 02 | 2 | SEC-02 | T-20-04/T-20-06 | Unsafe operators/destructive commands denied; medium-risk commands require approval. | unit | `cd agent-rs && cargo test security::policy::risk -- --nocapture` | agent-rs/src/security/policy.rs | ✅ green |
| 20-02-02 | 02 | 2 | SEC-03 | T-20-05/T-20-06 | Prompt-injection patterns and disallowed command patterns are denied before side effects. | unit | `cd agent-rs && cargo test security::policy::risk -- --nocapture` | agent-rs/src/security/policy.rs | ✅ green |
| 20-03-01 | 03 | 3 | SEC-01/SEC-02/SEC-03 | T-20-07/T-20-08/T-20-09 | Terminal and web dispatch enforce identical deny/approval policy envelopes. | integration | `cd agent-rs && cargo test --test security_guardrails -- --nocapture` | agent-rs/tests/security_guardrails.rs | ✅ green |
| 20-03-02 | 03 | 3 | SEC-01/SEC-02/SEC-03 | T-20-07/T-20-08/T-20-09 | Cross-path parity checks remain stable under full project quality gates. | integration | `cd agent-rs && cargo fmt && cargo test && cargo clippy -- -D warnings` | agent-rs/src/main.rs | ✅ green |
| 20-03-03 | 03 | 3 | SEC-01/SEC-02/SEC-03 | T-20-07/T-20-08/T-20-09 | End-to-end quality gate enforces no bypass regressions across runtime modes. | e2e quality gate | `cd agent-rs && cargo fmt && cargo test && cargo clippy -- -D warnings` | agent-rs/src/server.rs | ✅ green |

*Status: ✅ green - covered and passing*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-13
