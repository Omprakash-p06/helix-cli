# Phase 01 Validation Plan: Foundation & Security Sandbox

This document defines the verification requirements for Phase 01, mapping roadmap requirements to specific automated tests as per Nyquist compliance.

## Requirement Mapping

| Requirement ID | Description | Test Type | Verification Source | Automated Command |
|----------------|-------------|-----------|---------------------|-------------------|
| **MOD-01** | Qwen 3.6 model foundation | Smoke | `scripts/system_check.py` | `python3 scripts/system_check.py` |
| **SEC-01** | Docker Sandbox isolation | Integration | `agent-rs/tests/test_secure_execution.rs` | `cd agent-rs && cargo test --test test_secure_execution` |
| **SEC-02** | Command Policy Engine | Unit | `agent-rs/src/security/policy.rs` | `cd agent-rs && cargo test security::policy` |
| **SEC-03** | Immutable Audit Log | Unit | `agent-rs/src/security/audit.rs` | `cd agent-rs && cargo test security::audit` |

## Test Scenarios

### 1. Model Readiness (MOD-01)
- **Goal:** Verify hardware compatibility and model availability.
- **Success Criteria:** 
    - `scripts/config.py` correctly identifies VRAM.
    - Model files for Qwen 3.6 (27B/35B MoE) are found in the local model directory.
    - `llama.cpp` can initialize the model.

### 2. Sandbox Isolation (SEC-01)
- **Goal:** Ensure commands run in an isolated Docker container with no host access.
- **Success Criteria:**
    - `cargo test` demonstrates that a command in the sandbox cannot see files outside the mounted workspace.
    - Network access is restricted (if configured).
    - Resource limits (CPU/MEM) are applied.

### 3. Policy Enforcement (SEC-02)
- **Goal:** Block dangerous operators and normalize paths.
- **Success Criteria:**
    - `ls ; rm -rf /` is rejected.
    - `cat ../../../etc/shadow` is rejected.
    - `ls -l` is permitted and normalized.

### 4. Audit Integrity (SEC-03)
- **Goal:** Maintain a tamper-evident record of all actions.
- **Success Criteria:**
    - Every execution (permitted or denied) creates a row in `audit.db`.
    - `prev_hash` of row `N` matches the hash of row `N-1`.
    - Deleting a row breaks the chain verification.

## Global Standard Verification

As per Phase 03 plan:
- **Standard 2:** `cargo clippy` must return 0 warnings/errors.
- **Standard 2:** `cargo test` must pass 100%.
- **Standard 3:** `misc/architecture_[YYYY-MM-DD].svg` must be updated to reflect Phase 01 components.
