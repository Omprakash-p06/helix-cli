---
phase: 10-gemma-4-e4b-integration-evaluation-suite
verified: 2026-08-03T23:15:00Z
status: human_needed
score: 9/10 must-haves verified
behavior_unverified: 1
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 6/10
  gaps_closed:
    - "Switch the default test model to Gemma 4 E4B-it (DEFAULT_MODEL_NAME resolves to Gemma-4-E4B; test_qwen_config.py assertion updated)"
    - "Long-session retention benchmark shows >=90% constraint recall after 3 compaction cycles (sc3_retention_recall_rate_3_cycles exists and passes)"
    - "Prompt-injection tests show 0% instruction-following from untrusted content (sc5_injection_instruction_following_rate_is_zero measures 0% escape)"
    - "All 8 evaluation scenarios pass (eval_suite_reporting gate raised to automated_pass >= 8; SC8 marked passed: true)"
  gaps_remaining: []
  regressions: []
behavior_unverified_items:
  - truth: "All 8 evaluation scenarios pass (including the live-model SC8 variant)"
    test: "Download gemma-4-E4B-it-Q4_K_M.gguf (+ mmproj-F16.gguf), start llama-server on 127.0.0.1:8080 with --jinja, then run `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`"
    expected: "The test passes (200 OK, content contains primes/2/3), demonstrating the full Gemma 4 E4B native-function-calling path works end-to-end"
    why_human: "sc8_live_model_produces_tool_call_response is #[ignore]d and has never been executed; requires a physical GGUF download and a running llama-server — cannot be verified programmatically"
human_verification:
  - test: "Run the live Gemma 4 E4B end-to-end scenario: download gemma-4-E4B-it-Q4_K_M.gguf (+ mmproj-F16.gguf), start llama-server on 127.0.0.1:8080 with --jinja, then run `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`"
    expected: "The test passes (200 OK, content contains primes/2/3), demonstrating the full Gemma 4 E4B integration works end-to-end with native function calling"
    why_human: "Requires a physical GGUF model download and a running llama-server; cannot be verified programmatically in this environment"
  - test: "Run probe_backend_capabilities against a live llama-server: start the server, launch the agent, and observe the '[Runtime] Backend capabilities probed: model=..., function_calling=..., context_window=...' log line"
    expected: "function_calling=true (model_id contains gemma or /props has_jinja) and context_window matches the model profile; the probe's HTTP → capability path is exercised"
    why_human: "The probe's /v1/models and /props HTTP paths are never exercised by any test (only the defaults unit test exists); requires a live server"
---

# Phase 10: Gemma 4 E4B Integration & Evaluation Suite Verification Report (Re-verification)

**Phase Goal:** Switch the default test model to Gemma 4 E4B-it (128K context, native function calling), wire it into the updated context-engineering stack, and run the minimum evaluation suite to benchmark repo comprehension, tool-call correctness, long-session retention, research factuality, prompt-injection resistance, policy-escape resistance, and rollback correctness.
**Verified:** 2026-08-03T23:15:00Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (plan 10-05)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Default test model switched to Gemma 4 E4B-it | ✓ VERIFIED | config.py:388 `DEFAULT_MODEL_NAME = "Gemma-4-E4B" if "Gemma-4-E4B" in MODEL_CATALOG else (...)`; test_qwen_config.py:50 asserts `== "Gemma-4-E4B"`; `python -c "from scripts.config import DEFAULT_MODEL_NAME"` → `Gemma-4-E4B`; pytest 14 passed. Was FAILED (gap 1) — now closed |
| 2 | Gemma-4-E4B catalog entry with 3 VRAM-tiered variants (8GB/4GB/CPU) carrying `jinja: True` + `mmproj_filename` | ✓ VERIFIED | config.py `MODEL_CATALOG['Gemma-4-E4B']`: 3 variants, all jinja=True + mmproj-F16.gguf (verified by execution); regression pass |
| 3 | start_server.py passes `--jinja` and `--mmproj <path>` to llama-server from model profile metadata | ✓ VERIFIED | start_server.py:156-167 — profile-driven; mmproj existence-guarded with warning fallback; regression pass |
| 4 | Gemma 4 E4B downloadable presets from `unsloth/gemma-4-E4B-it-GGUF` | ✓ VERIFIED | download_model.py:36-56 MODEL_PRESETS (Q4_K_M, Q8_0, mmproj); `--preset` CLI wired; regression pass |
| 5 | BackendCapabilities probed at startup — function_calling, context_window, model_id populated and wired into main.rs | ✓ VERIFIED | config.rs:147 `probe_backend_capabilities`; main.rs:583-584 assigns probed caps; wiring regression pass (live HTTP path → human verification) |
| 6 | Evaluation suite covers all 8 benchmark areas with green automated tests | ✓ VERIFIED | All 8 eval binaries pass: retention now 4 tests, security now 7 tests (29 automated total, 1 ignored live) |
| 7 | All 8 evaluation scenarios pass (including the live-model SC8 variant) | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | Automated 8/8 gate verified (`eval_suite_reporting.rs:45 assert!(automated_pass >= 8)` passes; prints "8/8"; eval-results.json `automated_pass: 8`); SC8 marked `passed: true` with manual-run notes. But sc8_live_model_produces_tool_call_response remains `#[ignore]`d and has never been executed — live-model behavior unexercised → human verification |
| 8 | Long-session retention benchmark shows ≥90% constraint recall after 3 compaction cycles | ✓ VERIFIED | eval_suite_retention.rs:88-133 `sc3_retention_recall_rate_3_cycles`: injects 10 constraints, drops/re-opens engine 3× (compaction cycles), asserts `recall_rate >= 0.90` on 4th reopen. Named test passes. Was FAILED (gap 2) — now closed |
| 9 | Prompt-injection tests show 0% instruction-following from untrusted content | ✓ VERIFIED | eval_suite_security.rs:90-117 `sc5_injection_instruction_following_rate_is_zero`: 10 injection attempts through `sanitize_html_to_markdown`, asserts `escaped_to_trusted == 0`. Named test passes. Was FAILED (gap 3) — now closed |
| 10 | Full verification gates green: cargo test, clippy -D warnings, pytest | ✓ VERIFIED | `cargo test --package agent-rs -q` → all binaries 0 failures; `cargo clippy --package agent-rs -- -D warnings` → exit 0; `pytest tests/ -q` → 52 passed; regression pass |

**Score:** 9/10 truths verified (1 present, behavior-unverified — live SC8 variant)

### Gap Closure Status (plan 10-05)

All 4 gaps from the previous verification (status `gaps_found`, 6/10) are **closed**:

| # | Gap | Closure Evidence | Status |
|---|-----|-----------------|--------|
| 1 | `DEFAULT_MODEL_NAME` resolves to Gemma-4-E4B + updated assertion | config.py:388 explicit catalog-membership check; test_qwen_config.py:50 asserts `"Gemma-4-E4B"`; commit 5f888e3 diff is the exact minimal change; behavioral: `python -c` → `Gemma-4-E4B`, `MODEL_NAME` → `Gemma-4-E4B`; pytest 14 passed | ✓ CLOSED |
| 2 | `sc3_retention_recall_rate_3_cycles` benchmark, ≥90% recall after 3 cycles | Test exists (10 constraints, 3 re-open cycles, `recall_rate >= 0.90` assertion); `cargo test ... sc3_retention_recall_rate_3_cycles -- --exact` → ok; binary 4/4 | ✓ CLOSED |
| 3 | `sc5_injection_instruction_following_rate_is_zero` measures 0% escape | Test exists (10 attempts, `escaped_to_trusted == 0`); `cargo test ... sc5_injection_instruction_following_rate_is_zero -- --exact` → ok; binary 7/7. Adaptation documented: `Provenance` has no `Trusted` variant (types.rs:56-64 — Workspace/System/Research/Untrusted); escape measured as `!= Provenance::Untrusted`, preserving the plan's 0%-escape intent | ✓ CLOSED |
| 4 | `eval_suite_reporting` gate `automated_pass >= 8`, SC8 `passed: true` | eval_suite_reporting.rs:45 `assert!(automated_pass >= 8)`; SC8 block `passed: true` with "requires manual run" notes; println totals corrected; test prints "Automated scenarios passed: 8/8"; eval-results.json `"automated_pass": 8, "total": 8` | ✓ CLOSED |

### Deferred Items

None. Phase 10 is the final phase of the v1.0 milestone (ROADMAP.md ends at Phase 10); no later phase addresses any residual item.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `scripts/config.py` | Gemma-4-E4B as DEFAULT_MODEL_NAME + catalog entry | ✓ VERIFIED | Line 388 explicit membership check; 3 variants with jinja/mmproj passthrough |
| `tests/test_qwen_config.py` | Assertion matches new default | ✓ VERIFIED | Line 50 asserts `DEFAULT_MODEL_NAME == "Gemma-4-E4B"`; 14 tests pass |
| `scripts/start_server.py` | --jinja / --mmproj flag injection | ✓ VERIFIED | Profile-driven, existence-guarded (lines 156-167) |
| `scripts/download_model.py` | MODEL_PRESETS + --preset CLI | ✓ VERIFIED | 3 Gemma presets, `--preset` wired |
| `agent-rs/src/config.rs` | `probe_backend_capabilities` async fn | ✓ VERIFIED | Line 147; defaults unit test exists |
| `agent-rs/src/main.rs` | Startup probe call + assignment | ✓ VERIFIED | Lines 583-584 |
| `agent-rs/tests/eval_suite_retention.rs` | 4 tests incl. recall-rate benchmark | ✓ VERIFIED | `sc3_retention_recall_rate_3_cycles` added; 4/4 green |
| `agent-rs/tests/eval_suite_security.rs` | 7 tests incl. injection-rate measurement | ✓ VERIFIED | `sc5_injection_instruction_following_rate_is_zero` added; 7/7 green |
| `agent-rs/tests/eval_suite_reporting.rs` | Gate >= 8, SC8 passed: true | ✓ VERIFIED | Prints "8/8"; SC3/SC5 notes acknowledge new tests |
| `agent-rs/eval-results.json` | Runtime-generated report | ✓ VERIFIED | `"automated_pass": 8, "total": 8` (gitignored, regenerated per run) |
| All other eval binaries | comprehension/tool_call/research/rollback/e2e | ✓ VERIFIED | Green (regression pass) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| config.py catalog | start_server.py flags | `build_model_entry()` passthrough of `jinja`/`mmproj_filename` | ✓ WIRED | Runtime verified |
| config.py catalog | download_model.py presets | Filename match | ✓ WIRED | Exact match between catalog variants and MODEL_PRESETS |
| config.rs probe | main.rs startup | `probe_backend_capabilities` call + assignment | ✓ WIRED | main.rs:583-584 |
| BackendCapabilities | context engineering stack | `function_calling`/`context_window` consumed downstream | ✓ WIRED | Probe populated at startup before runtime-profile selection |
| EvalResult type | Reporting test | `use agent_rs::eval::EvalResult` | ✓ WIRED | eval_suite_reporting.rs:6 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| config.py `DEFAULT_MODEL_NAME` | default model | Explicit `"Gemma-4-E4B" in MODEL_CATALOG` check | ✓ `Gemma-4-E4B` (behavioral) | ✓ FLOWING |
| config.py `MODEL_PROFILE` | `jinja` / `mmproj_filename` | Static catalog → `build_model_entry` | ✓ real values per variant | ✓ FLOWING |
| start_server.py server args | `--jinja` / `--mmproj` | `model_profile.get(...)` | ✓ profile-driven, existence-guarded | ✓ FLOWING |
| eval_suite_retention.rs recall rate | `found / 10.0` | Real SQLite MemoryEngine across 3 re-opens | ✓ real measured rate ≥ 0.90 | ✓ FLOWING |
| eval_suite_security.rs escape rate | `escaped_to_trusted` | 10 sanitizer passes | ✓ real measured count = 0 | ✓ FLOWING |
| eval-results.json scenarios | SC1-SC8 pass flags | `EvalResult` markers; SC8 notes document live variant manual-only | ✓ automated_pass 8/8 | ✓ FLOWING (live variant behavior → human verification) |
| config.rs `BackendCapabilities` | HTTP probe | `/v1/models` + `/props` (live server) | ⚠️ code-path present; never exercised by a test | ⚠️ STATIC-UNTESTED (routes to human verification) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Default model resolution | `python -c "from scripts.config import DEFAULT_MODEL_NAME, MODEL_NAME; ..."` | `Gemma-4-E4B` / `Gemma-4-E4B` | ✓ PASS |
| Qwen config suite | `pytest tests/test_qwen_config.py -q` | 14 passed | ✓ PASS |
| Python suite green | `pytest tests/ -q` | 52 passed | ✓ PASS |
| Retention recall benchmark | `cargo test --package agent-rs --test eval_suite_retention sc3_retention_recall_rate_3_cycles -- --exact` | ok (1 passed) | ✓ PASS |
| Injection-rate measurement | `cargo test --package agent-rs --test eval_suite_security sc5_injection_instruction_following_rate_is_zero -- --exact` | ok (1 passed) | ✓ PASS |
| Retention + security binaries | `cargo test --package agent-rs --test eval_suite_retention --test eval_suite_security -q` | 4 passed + 7 passed | ✓ PASS |
| Reporting gate | `cargo test --package agent-rs --test eval_suite_reporting -- --nocapture` | "Automated scenarios passed: 8/8"; ok | ✓ PASS |
| Generated report | read `agent-rs/eval-results.json` | `"automated_pass": 8, "total": 8` | ✓ PASS |
| Full Rust suite | `cargo test --package agent-rs -q` | all binaries 0 failures | ✓ PASS |
| Clippy gate | `cargo clippy --package agent-rs -- -D warnings` | exit 0 | ✓ PASS |

### Probe Execution

Step 7c: SKIPPED — no `probe-*.sh` scripts exist in the repository and no phase PLAN/SUMMARY declares probe-based verification.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| MOD-01 | 10-01, 10-02 | Multi-Model Local Integration (extended) | ✓ SATISFIED | Gemma catalog entry + `probe_backend_capabilities`; default-model switch now goal-level satisfied |
| MOD-02 | 10-01 | Dynamic Model Discovery (extended) | ✓ SATISFIED | 3 VRAM-tiered Gemma variants via `MODEL_CATALOG`/`AVAILABLE_MODELS` |
| MOD-03 | 10-01 | Hugging Face Downloader (extended) | ✓ SATISFIED | `MODEL_PRESETS` + `--preset Gemma-4-E4B-*` |
| SEC-03 | 10-03 (SC7) | Immutable audit logging | ✓ SATISFIED | eval_suite_rollback.rs asserts hash-chained audit |
| FIX-02 | 10-03 (SC7) | Rollback snapshots | ✓ SATISFIED | SnapshotManager (agent_core/repair/snapshots.rs) |
| FIX-01 | 10-03 (SC7) | Approval gate | ✓ SATISFIED | Confirmation gate + `require_confirmation` asserted |
| SEC-05 | 10-03 (SC6) | Blocklist enforcement | ✓ SATISFIED | BLOCKLIST/DANGEROUS_COMMANDS in scripts/config.py asserted |

Phase requirement IDs: null (no phase-level requirement mapping in any PLAN frontmatter — verified across 10-01…10-05). All requirement IDs referenced by plans (MOD-01/02/03, SEC-03, FIX-01/02, SEC-05) are accounted for; no orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | TBD/FIXME/XXX/PLACEHOLDER markers in phase-modified files | — | None found (all 5 files clean) |
| agent-rs/src/* | various | Pre-existing `dead_code`/`unused_imports` warnings under `cargo test` | ℹ️ Info | Non-blocking; `clippy -- -D warnings` still exits 0 (the declared gate) |

### Human Verification Required

### 1. Live Gemma 4 E4B inference (SC8 live variant)

**Test:** Download `gemma-4-E4B-it-Q4_K_M.gguf` (+ `mmproj-F16.gguf`), start llama-server on 127.0.0.1:8080 with `--jinja`, then run `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`.
**Expected:** Test passes (200 OK, content contains primes/2/3) — the full Gemma 4 E4B native-function-calling path works end-to-end.
**Why human:** Requires physical GGUF download and a running llama-server; the test is `#[ignore]`d and cannot be verified programmatically here.

### 2. probe_backend_capabilities live HTTP path

**Test:** Start the server, launch the agent, and check the `[Runtime] Backend capabilities probed: model=..., function_calling=..., context_window=...` log line.
**Expected:** `function_calling=true` (gemma model id or `has_jinja: true`) and `context_window` matching the model profile.
**Why human:** The probe's `/v1/models` + `/props` HTTP paths are never exercised by any test (only the defaults unit test exists); requires a live server.

### Gaps Summary

**All 4 gaps from the previous verification are closed.** Re-verification confirms:

1. **Gap 1 (BLOCKER) CLOSED:** `DEFAULT_MODEL_NAME` now resolves to `Gemma-4-E4B` via an explicit catalog-membership check with the dynamic first-key resolution retained as fallback. `MODEL_NAME` follows. `test_qwen_config.py:50` asserts the new default. Commit `5f888e3` is the exact minimal diff; behavioral check and pytest confirm.
2. **Gap 2 CLOSED:** `sc3_retention_recall_rate_3_cycles` is a genuine recall-rate benchmark — 10 constraints injected into a real SQLite `MemoryEngine`, 3 engine re-opens (compaction cycles), ≥90% recall asserted and passing on the 4th open.
3. **Gap 3 CLOSED:** `sc5_injection_instruction_following_rate_is_zero` measures instruction-following rate as escapes from the `Provenance::Untrusted` sandbox (10 attempts, 0 escapes, passing). The documented adaptation (no `Trusted` variant in the enum) preserves the plan's 0%-escape intent and is verified against the actual `Provenance` definition.
4. **Gap 4 CLOSED:** `eval_suite_reporting` now asserts `automated_pass >= 8`, marks SC8 `passed: true` (4/4 automated subtests; live variant documented manual-only), and the generated report shows `automated_pass: 8, total: 8`.

All declared verification gates remain green: pytest 52 passed, cargo test 0 failures (retention 4, security 7, reporting 8/8 among 29 automated eval tests), clippy `-D warnings` exit 0.

**Remaining: 1 behavior-unverified truth + 2 human verification items.** The SC8 live-model variant (`sc8_live_model_produces_tool_call_response`) is still `#[ignore]`d and has never been executed, and the `probe_backend_capabilities` live HTTP path is unexercised by any test. Both require a physical Gemma 4 E4B GGUF download and a running llama-server — a manual human run, as itemized in the Human Verification section. Automated checks pass; the phase awaits human confirmation of these two live-model behaviors.

---

_Verified: 2026-08-03T23:15:00Z_
_Verifier: the agent (gsd-verifier)_
