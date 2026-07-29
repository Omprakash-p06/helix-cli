---
phase: 10
slug: gemma-4-e4b-integration-evaluation-suite
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-30
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + pytest 9.0 (Python) |
| **Config file** | `agent-rs/Cargo.toml` / `pyproject.toml` |
| **Quick run command** | `cargo test --package agent-rs -q` |
| **Full suite command** | `cargo test --package agent-rs -q && pytest tests/` |
| **Estimated runtime** | ~35 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --package agent-rs -q`
- **After every plan wave:** Run `cargo test --package agent-rs -q && pytest tests/`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 35 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-T1 | 01 | 1 | Gemma 4 config catalog | unit | `pytest tests/test_model_discovery.py -q` | ❌ W0 | ⬜ pending |
| 10-01-T2 | 01 | 1 | Gemma 4 start_server jinja flag | unit | `pytest tests/test_start_server_runtime_profile.py -q` | ❌ W0 | ⬜ pending |
| 10-01-T3 | 01 | 1 | download_model Gemma 4 preset | unit | `pytest tests/test_download_model.py -q` | ❌ W0 | ⬜ pending |
| 10-02-T1 | 02 | 1 | BackendCapabilities probe | unit | `cargo test --package agent-rs config -q` | ❌ W0 | ⬜ pending |
| 10-03-T1 | 03 | 2 | Eval scenario 1 repo comprehension | integration | `cargo test --package agent-rs --test eval_suite_comprehension -q` | ❌ W0 | ⬜ pending |
| 10-03-T2 | 03 | 2 | Eval scenario 2 tool-call correctness | integration | `cargo test --package agent-rs --test eval_suite_tool_call -q` | ❌ W0 | ⬜ pending |
| 10-03-T3 | 03 | 2 | Eval scenario 3 long-session retention | integration | `cargo test --package agent-rs --test eval_suite_retention -q` | ❌ W0 | ⬜ pending |
| 10-03-T4 | 03 | 2 | Eval scenario 4 research factuality | integration | `cargo test --package agent-rs --test eval_suite_research -q` | ❌ W0 | ⬜ pending |
| 10-03-T5 | 03 | 2 | Eval scenario 5 prompt injection | integration | `cargo test --package agent-rs --test eval_suite_security -q` | ❌ W0 | ⬜ pending |
| 10-03-T6 | 03 | 2 | Eval scenario 6 policy escape | integration | `cargo test --package agent-rs --test eval_suite_security -q` | ❌ W0 | ⬜ pending |
| 10-03-T7 | 03 | 2 | Eval scenario 7 rollback correctness | integration | `cargo test --package agent-rs --test eval_suite_rollback -q` | ❌ W0 | ⬜ pending |
| 10-04-T1 | 04 | 3 | Eval scenario 8 end-to-end pipeline | integration | `cargo test --package agent-rs --test eval_suite_e2e -q` | ❌ W0 | ⬜ pending |
| 10-04-T2 | 04 | 3 | Eval result reporting | unit | `cargo test --package agent-rs eval_result -q` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `agent-rs/tests/eval_suite_comprehension.rs` — stub for scenario 1
- [ ] `agent-rs/tests/eval_suite_tool_call.rs` — stub for scenario 2
- [ ] `agent-rs/tests/eval_suite_retention.rs` — stub for scenario 3
- [ ] `agent-rs/tests/eval_suite_research.rs` — stub for scenario 4
- [ ] `agent-rs/tests/eval_suite_security.rs` — stubs for scenarios 5 & 6
- [ ] `agent-rs/tests/eval_suite_rollback.rs` — stub for scenario 7
- [ ] `agent-rs/tests/eval_suite_e2e.rs` — stub for scenario 8
- [ ] `mockito` crate added to `agent-rs/Cargo.toml` [dev-dependencies] — HTTP mocking for eval tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Gemma 4 E4B model inference | Scenario 8 end-to-end | Requires physical GGUF model and llama-server | Download gemma-4-E4B-it-Q4_K_M.gguf, start server with `--jinja`, run `cargo test --package agent-rs --test eval_suite_e2e -- --ignored` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 35s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
