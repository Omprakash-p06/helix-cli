# Phase 23-01 Summary: CPU TTFT and Runtime Watchdog

## Changes

### 1. Runtime Profile System (PERF-04)
- Created `agent-rs/src/runtime_profile.rs` defining typed profiles: `latency_cpu`, `balanced_cpu`, and `safe_recovery`.
- Implemented automatic profile selection based on CPU core count and GPU availability.
- Profiles enforce deterministic `ttft_target_ms`, batch sizes, and thread counts.
- Profile settings are passed to the Python launcher via new `HELIX_` environment variables.

### 2. Stateful Watchdog Recovery (PERF-05)
- Created `agent-rs/src/watchdog.rs` with a state machine: `Healthy`, `Degraded`, `Recovering`, `Cooldown`.
- Implemented a restart budget (max 3 attempts) and an exponential backoff policy for recovery attempts.
- Enforced a 5-minute cooldown period after budget exhaustion to prevent infinite restart loops.
- Integrated watchdog into the `send_with_recovery` and `maybe_boot_model_server` paths in `main.rs`.

### 3. Launcher and Config Bridging
- Updated `scripts/start_server.py` to honor `HELIX_RUNTIME_PROFILE` and associated performance overrides.
- Refactored `agent-rs/src/main.rs` to replace ad-hoc retry logic with state-machine-driven recovery.
- Added explicit status messages for profile selection and watchdog transitions.

### 4. Regression Testing
- Created `agent-rs/tests/runtime_profile_watchdog.rs` covering:
    - Profile selection logic and core-count sensitivity.
    - Watchdog state transitions and restart budget enforcement.
    - Cooldown lockout and backoff timing.
- Created `tests/test_start_server_runtime_profile.py` verifying that environment variables correctly override `config.py` defaults in the launcher.

## Verification Results
- `cd agent-rs && cargo test -q --test runtime_profile_watchdog` passed.
- `cd agent-rs && cargo test -q` passed (54 tests).
- `pytest -q tests/test_start_server_runtime_profile.py` passed.

## Threat Model Mitigation
- **T-23-01 (Recovery Loop):** Mitigated by watchdog restart budget and cooldown lockout.
- **T-23-02 (Env Overrides):** Mitigated by deterministic profile-to-env mapping in Rust.
- **T-23-03 (Observability):** Mitigated by explicit TUI/CLI status messages for all state transitions.
