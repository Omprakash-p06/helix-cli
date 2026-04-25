# Phase 01 Validation Plan: Foundation & Security Sandbox

This document records the Nyquist validation coverage for Phase 01 and maps each completed requirement to executable tests.

## Test Infrastructure

| Framework | Scope | Primary Commands |
|-----------|-------|------------------|
| Pytest | Qwen configuration and readiness checks | `pytest tests/test_qwen_config.py -v`, `pytest tests/test_system_check.py -v` |
| Pytest | Model installer safeguards | `pytest tests/test_model_install.py -v` |
| Cargo test | Rust policy, audit, and runtime guardrails | `cd agent-rs && cargo test` |

## Requirement Mapping

| Requirement ID | Description | Coverage Status | Verification Source | Automated Command |
|----------------|-------------|-----------------|---------------------|-------------------|
| **MOD-01** | Qwen 3.6 model foundation | COVERED | `tests/test_qwen_config.py`, `tests/test_system_check.py`, `scripts/system_check.py` | `pytest tests/test_qwen_config.py -v && pytest tests/test_system_check.py -v && python3 scripts/system_check.py` |
| **SEC-01** | Docker Sandbox isolation | COVERED | `agent-rs/tests/test_secure_execution.rs` | `cd agent-rs && cargo test --test test_secure_execution` |
| **SEC-02** | Command Policy Engine | COVERED | `agent-rs/src/security/policy.rs`, `agent-rs/tests/security_guardrails.rs`, `agent-rs/tests/tool_runtime_contracts.rs` | `cd agent-rs && cargo test` |
| **SEC-03** | Immutable Audit Log | COVERED | `agent-rs/tests/audit_log_mvp.rs`, `agent-rs/tests/test_secure_execution.rs` | `cd agent-rs && cargo test --test audit_log_mvp --test test_secure_execution` |

## Per-Task Map

| Task | Requirement | Status | Notes |
|------|-------------|--------|-------|
| 01-01 | MOD-01 | COVERED | Direct Qwen tier selection tests now assert the VRAM-aware model profile, and the system check output is verified against config-derived values. |
| 01-02 | SEC-01, SEC-02 | COVERED | Docker sandbox execution is validated by integration coverage; policy logic is covered by unit tests in `policy.rs` plus runtime guardrail tests. |
| 01-03 | SEC-03 | COVERED | Audit append/query/tamper-detection behavior is exercised by `audit_log_mvp.rs`, with end-to-end logging also asserted by `test_secure_execution.rs`. |

## Test Scenarios

### 1. Model Readiness (MOD-01)
- **Goal:** Verify hardware-aware Qwen 3.6 selection and readiness reporting.
- **Success Criteria:**
    - `scripts/config.py` selects the expected Qwen 3.6 variant for each VRAM tier.
    - `scripts/system_check.py` reports the selected model, artifact name, and quantization guidance from config.
    - `python3 scripts/system_check.py` remains a valid smoke check for local readiness.

### 2. Sandbox Isolation (SEC-01)
- **Goal:** Ensure commands run in an isolated Docker container with no host access.
- **Success Criteria:**
    - `cargo test --test test_secure_execution` exercises sandboxed execution and audit logging.
    - Sandbox execution rejects blocked commands before they reach the container.

### 3. Policy Enforcement (SEC-02)
- **Goal:** Block dangerous operators and normalize paths.
- **Success Criteria:**
    - `agent-rs/src/security/policy.rs` unit tests reject shell metacharacters and destructive commands.
    - Path traversal attempts are rejected and in-workspace paths are normalized.

### 4. Audit Integrity (SEC-03)
- **Goal:** Maintain a tamper-evident record of all actions.
- **Success Criteria:**
    - `audit_log_mvp.rs` verifies append/query behavior and chain integrity.
    - `test_secure_execution.rs` confirms policy and execution events are both recorded.

## Validation Audit 2026-04-24

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

## Manual-Only

None.

## Sign-Off

Phase 01 is Nyquist-compliant with direct automated coverage for the foundation, sandbox, policy, and audit requirements.
