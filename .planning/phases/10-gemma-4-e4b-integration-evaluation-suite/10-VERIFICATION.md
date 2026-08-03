---
phase: 10-gemma-4-e4b-integration-evaluation-suite
verified: 2026-08-03T22:30:00Z
status: gaps_found
score: 6/10 must-haves verified
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "Switch the default test model to Gemma 4 E4B-it"
    status: failed
    reason: "DEFAULT_MODEL_NAME is still 'Qwen-3.6-27B-MoE' (scripts/config.py:388 — list(MODEL_CATALOG.keys())[0] picks the first catalog key; Gemma-4-E4B was appended last). MODEL_NAME = DEFAULT_MODEL_NAME (line 389), and tests/test_qwen_config.py:50 even asserts DEFAULT_MODEL_NAME == 'Qwen-3.6-27B-MoE'. Gemma 4 E4B was ADDED as an option but never made the default. No commit in this phase touches the default."
    artifacts:
      - path: "scripts/config.py"
        issue: "DEFAULT_MODEL_NAME resolves to Qwen-3.6-27B-MoE, not Gemma-4-E4B"
      - path: "tests/test_qwen_config.py"
        issue: "Line 50 asserts config.DEFAULT_MODEL_NAME == 'Qwen-3.6-27B-MoE' — locks in the Qwen default"
    missing:
      - "Change the default model resolution so the active default test model is Gemma-4-E4B (e.g., explicit DEFAULT_MODEL_NAME = 'Gemma-4-E4B', or reorder catalog keys), and update test_qwen_config.py's assertion"
  - truth: "Long-session retention benchmark shows >=90% constraint recall after 3 compaction cycles"
    status: failed
    reason: "No recall-percentage benchmark and no 3-compaction-cycle scenario exists. eval_suite_retention.rs verifies (1) a single constraint survives ONE engine re-open (a single simulated compaction cycle), (2) FTS5 search works, (3) resume_session() recalls a constraint. None compute a recall rate or run 3 cycles. The roadmap success metric is not implemented."
    artifacts:
      - path: "agent-rs/tests/eval_suite_retention.rs"
        issue: "Presence-based retention tests only; no % recall measurement, no 3-cycle loop"
    missing:
      - "A benchmark that injects N constraints, runs 3 compaction cycles, measures recall rate, and asserts >= 90%"
  - truth: "All 8 evaluation scenarios pass"
    status: failed
    reason: "7 scenarios pass fully and SC8's 4 automated tests pass (28 automated eval tests green), but the SC8 live-model variant (sc8_live_model_produces_tool_call_response) is #[ignore]d and has never been executed or passed. eval_suite_reporting.rs hardcodes SC8 as passed: false ('Requires live Gemma 4 model'), and the generated eval-results.json reports automated_pass: 7, total: 8. The metric 'all 8 pass' is therefore not demonstrated."
    artifacts:
      - path: "agent-rs/tests/eval_suite_e2e.rs"
        issue: "sc8_live_model_produces_tool_call_response is #[ignore]d — never run in CI or manually during this phase"
      - path: "agent-rs/tests/eval_suite_reporting.rs"
        issue: "Line 19-20 hardcodes SC8 as passed: false regardless of actual state"
    missing:
      - "Run and pass the live SC8 test with a real llama-server + Gemma 4 E4B GGUF, or re-scope the metric explicitly"
  - truth: "Prompt-injection tests show 0% instruction-following from untrusted content"
    status: failed
    reason: "The sanitization MECHANISM is verified (untrusted_web_content sandboxing, breakout-delimiter escaping, script-payload stripping — all 6 SC5/SC6 tests pass), which is the mechanism that yields 0% instruction-following. But no test MEASURES instruction-following rate from untrusted content (that requires a live-model or harness-level behavioral measurement). The metric as written is not measured."
    artifacts:
      - path: "agent-rs/tests/eval_suite_security.rs"
        issue: "Mechanism-level assertions only; no instruction-following rate measurement"
    missing:
      - "A test (mock or live) that measures whether injected instructions are followed, asserting a 0% rate"
human_verification:
  - test: "Run the live Gemma 4 E4B end-to-end scenario: download gemma-4-E4B-it-Q4_K_M.gguf (+ mmproj-F16.gguf), start llama-server on 127.0.0.1:8080 with --jinja, then run `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`"
    expected: "The test passes (200 OK, content contains primes/2/3), demonstrating the full Gemma 4 E4B integration works end-to-end with native function calling"
    why_human: "Requires a physical GGUF model download and a running llama-server; cannot be verified programmatically in this environment"
  - test: "Run probe_backend_capabilities against a live llama-server: start the server, launch the agent, and observe the '[Runtime] Backend capabilities probed: model=..., function_calling=..., context_window=...' log line"
    expected: "function_calling=true (model_id contains gemma or /props has_jinja) and context_window matches the model profile; the probe's HTTP → capability path is exercised"
    why_human: "The probe's /v1/models and /props HTTP paths are never exercised by any test (only the defaults unit test exists); requires a live server"
---

# Phase 10: Gemma 4 E4B Integration & Evaluation Suite Verification Report

**Phase Goal:** Switch the default test model to Gemma 4 E4B-it (128K context, native function calling), wire it into the updated context-engineering stack, and run the minimum evaluation suite to benchmark repo comprehension, tool-call correctness, long-session retention, research factuality, prompt-injection resistance, policy-escape resistance, and rollback correctness.
**Verified:** 2026-08-03T22:30:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Default test model switched to Gemma 4 E4B-it | ✗ FAILED | `DEFAULT_MODEL_NAME = list(MODEL_CATALOG.keys())[0]` → `"Qwen-3.6-27B-MoE"` (config.py:388); Gemma-4-E4B is the last catalog key; test_qwen_config.py:50 asserts the Qwen default |
| 2 | Gemma-4-E4B catalog entry with 3 VRAM-tiered variants (8GB/4GB/CPU) carrying `jinja: True` + `mmproj_filename` | ✓ VERIFIED | config.py:291-334; `MODEL_CATALOG['Gemma-4-E4B']` has 3 variants, all with jinja=True and mmproj-F16.gguf (verified by execution) |
| 3 | start_server.py passes `--jinja` and `--mmproj <path>` to llama-server from model profile metadata | ✓ VERIFIED | start_server.py:156-167 — `if model_profile.get("jinja"): cmd.append("--jinja")`; mmproj with existence check + graceful warning |
| 4 | Gemma 4 E4B downloadable presets from `unsloth/gemma-4-E4B-it-GGUF` | ✓ VERIFIED | download_model.py:36-56 MODEL_PRESETS (Q4_K_M, Q8_0, mmproj); `--preset` CLI wired (lines 212-227) |
| 5 | BackendCapabilities probed at startup — function_calling, context_window, model_id populated and wired into main.rs | ✓ VERIFIED | config.rs:147-205 `probe_backend_capabilities` (context_window from `app_config.context_size as u32`; function_calling on gemma/functionary/hermes id or `has_jinja: true`); main.rs:583-584 assigns `app_config.backend_capabilities = probed_caps` |
| 6 | Evaluation suite covers all 8 benchmark areas with green automated tests | ✓ VERIFIED | All 8 eval test binaries exist and pass: comprehension 4, tool_call 4, retention 3, research 3, security 6, rollback 4, e2e 4+1 ignored, reporting 1 (28 passed, 1 ignored) |
| 7 | All 8 evaluation scenarios pass (including the live-model SC8 variant) | ✗ FAILED | SC8 live test `#[ignore]`d (eval_suite_e2e.rs:95), never executed; eval_suite_reporting.rs hardcodes SC8 `passed: false`; eval-results.json shows automated_pass: 7 / total: 8 |
| 8 | Long-session retention benchmark shows ≥90% constraint recall after 3 compaction cycles | ✗ FAILED | eval_suite_retention.rs verifies 1 constraint across 1 engine re-open + FTS5 + resume_session; no recall %, no 3-cycle benchmark |
| 9 | Prompt-injection tests show 0% instruction-following from untrusted content | ✗ FAILED | SC5/SC6 tests (6/6 green) verify the sanitization mechanism (sandboxing, breakout escape, script stripping) but never measure instruction-following rate |
| 10 | Full verification gates green: cargo test, clippy -D warnings, pytest | ✓ VERIFIED | `cargo test --package agent-rs` → 135 lib + all integration binaries pass, 0 failures; `cargo clippy --package agent-rs -- -D warnings` → exit 0; `pytest tests/` → 52 passed |

**Score:** 6/10 truths verified (3 failed, 1 failed-by-not-measured)

### Deferred Items

Phase 10 is the final phase of the v1.0 milestone (ROADMAP.md ends at Phase 10; STATE.md marks milestone complete). No later phase addresses these gaps — none are deferred.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `scripts/config.py` | Gemma-4-E4B catalog entry + jinja/mmproj passthrough | ✓ VERIFIED | 3 variants (Q8_0 8GB / Q4_K_M 4GB / Q4_K_M CPU), `jinja`/`mmproj_filename` passthrough at lines 382-383 |
| `scripts/start_server.py` | --jinja / --mmproj flag injection | ✓ VERIFIED | Profile-driven, mmproj existence-guarded, warning fallback |
| `scripts/download_model.py` | MODEL_PRESETS + --preset CLI | ✓ VERIFIED | 3 Gemma presets, `find_model_preset`/`list_model_presets`, `--preset` wired |
| `agent-rs/src/config.rs` | `probe_backend_capabilities` async fn | ✓ VERIFIED | Full /v1/models + /props probe; defaults test exists |
| `agent-rs/src/main.rs` | Startup probe call + assignment | ✓ VERIFIED | Lines 583-586 |
| `agent-rs/src/eval.rs` | `EvalResult` type + 2 unit tests | ✓ VERIFIED | pass/fail constructors, 2 unit tests |
| `agent-rs/src/lib.rs` | `pub mod eval;` | ✓ VERIFIED | Line 17 |
| `agent-rs/Cargo.toml` | mockito dev-dependency | ✓ VERIFIED | Line 68: `mockito = "1.4"` |
| `agent-rs/tests/eval_suite_comprehension.rs` | 4 passing tests | ✓ VERIFIED | 4/4 green |
| `agent-rs/tests/eval_suite_tool_call.rs` | 4 passing tests | ✓ VERIFIED | 4/4 green |
| `agent-rs/tests/eval_suite_retention.rs` | 3 passing tests | ✓ VERIFIED | 3/3 green (presence-based; see gap 2) |
| `agent-rs/tests/eval_suite_research.rs` | 3 passing tests | ✓ VERIFIED | 3/3 green |
| `agent-rs/tests/eval_suite_security.rs` | 6 passing tests | ✓ VERIFIED | 6/6 green (mechanism-level; see gap 4) |
| `agent-rs/tests/eval_suite_rollback.rs` | 4 passing tests | ✓ VERIFIED | 4/4 green |
| `agent-rs/tests/eval_suite_e2e.rs` | 5 tests (4 automated + 1 #[ignore]d live) | ✓ VERIFIED | 4 pass + 1 ignored; SSRF/token-budget/capability assertions match plan |
| `agent-rs/tests/eval_suite_reporting.rs` | `eval_full_report` asserts >= 7 | ✓ VERIFIED | Passes; writes eval-results.json (7/8, SC8 failed) |
| `agent-rs/eval-results.json` | Runtime-generated report | ✓ VERIFIED | Generated (gitignored); automated_pass: 7, total: 8 |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| config.py catalog | start_server.py flags | `build_model_entry()` passthrough of `jinja`/`mmproj_filename` | ✓ WIRED | Runtime verified: `build_model_entry("Gemma-4-E4B", 8)` → jinja True, mmproj set |
| config.py catalog | download_model.py presets | Filenames `gemma-4-E4B-it-Q8_0.gguf` / `Q4_K_M.gguf` / `mmproj-F16.gguf` match | ✓ WIRED | Exact filename match between catalog variants and MODEL_PRESETS |
| config.rs probe | main.rs startup | `probe_backend_capabilities` call + assignment to `app_config.backend_capabilities` | ✓ WIRED | main.rs:583-584 |
| BackendCapabilities | context engineering stack | `function_calling`/`context_window` consumed downstream | ✓ WIRED | Probe populated at startup before runtime-profile selection |
| EvalResult type | Reporting test | `use agent_rs::eval::EvalResult` | ✓ WIRED | eval_suite_reporting.rs:6 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| config.py `MODEL_PROFILE` | `jinja` / `mmproj_filename` | Static catalog → `build_model_entry` | ✓ real values per variant | ✓ FLOWING |
| start_server.py server args | `--jinja` / `--mmproj` | `model_profile.get("jinja"/"mmproj_filename")` | ✓ driven by profile; mmproj existence-guarded | ✓ FLOWING |
| config.rs `BackendCapabilities` | `context_window` | `app_config.context_size` (from Python config) | ✓ real value | ✓ FLOWING |
| config.rs `BackendCapabilities.function_calling` | HTTP probe | `/v1/models` + `/props` (live server) | ⚠️ code-path present; never exercised by a test | ⚠️ STATIC-UNTESTED (routes to human verification) |
| eval-results.json scenarios | SC1-SC8 pass flags | Hardcoded `EvalResult::pass/fail` markers | ⚠️ self-reported markers, not live test re-execution | ⚠️ PARTIAL (SC8 always false) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Python suite green | `pytest tests/ -q` | 52 passed | ✓ PASS |
| Gemma catalog importable | `python -c "from scripts.config import MODEL_CATALOG; print(MODEL_CATALOG['Gemma-4-E4B'])"` | 3 variants, jinja+mmproj all True | ✓ PASS |
| Eval scenario tests | `cargo test --package agent-rs --test eval_suite_* -q` (8 binaries) | 28 passed, 1 ignored | ✓ PASS |
| Full Rust suite | `cargo test --package agent-rs -q` | 135 lib + all integration, 0 failures | ✓ PASS |
| Clippy gate | `cargo clippy --package agent-rs -- -D warnings` | exit 0 | ✓ PASS |
| Eval report generation | `cargo test --package agent-rs --test eval_suite_reporting -q` | 1 passed; eval-results.json written (7/8) | ✓ PASS |

### Probe Execution

Step 7c: SKIPPED — no `probe-*.sh` scripts exist in the repository and no phase PLAN/SUMMARY declares probe-based verification.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| MOD-01 | 10-01, 10-02 | Multi-Model Local Integration (extended) | ✓ SATISFIED | Gemma catalog entry + `probe_backend_capabilities` populated from live server; note: default-model-switch gap is goal-level, not requirement-level |
| MOD-02 | 10-01 | Dynamic Model Discovery (extended) | ✓ SATISFIED | 3 VRAM-tiered Gemma variants discoverable via `MODEL_CATALOG`/`AVAILABLE_MODELS` |
| MOD-03 | 10-01 | Hugging Face Downloader (extended) | ✓ SATISFIED | `MODEL_PRESETS` + `--preset Gemma-4-E4B-*` CLI reusing `download_file()` |
| SEC-03 | 10-03 (SC7) | Immutable audit logging | ✓ SATISFIED | eval_suite_rollback.rs asserts hash-chained audit in src/audit.rs |
| FIX-02 | 10-03 (SC7) | Rollback snapshots | ✓ SATISFIED | SnapshotManager (agent_core/repair/snapshots.rs) verified |
| FIX-01 | 10-03 (SC7) | Approval gate | ✓ SATISFIED | Confirmation gate + `require_confirmation` asserted |
| SEC-05 | 10-03 (SC6) | Blocklist enforcement | ✓ SATISFIED | BLOCKLIST/DANGEROUS_COMMANDS in scripts/config.py asserted |

Phase requirement IDs: null (no phase-level requirement mapping). All requirement IDs referenced by plans (MOD-01/02/03) are accounted for; no orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | TBD/FIXME/XXX/placeholder markers in phase-modified files | — | None found |
| agent-rs/src/* | various | Pre-existing `dead_code`/`unused_imports` warnings under `cargo test` | ℹ️ Info | Non-blocking; `clippy -- -D warnings` still exits 0 (the declared gate) |

### Human Verification Required

### 1. Live Gemma 4 E4B inference (SC8 live variant)

**Test:** Download `gemma-4-E4B-it-Q4_K_M.gguf` (+ `mmproj-F16.gguf`), start llama-server on 127.0.0.1:8080 with `--jinja`, then run `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`.
**Expected:** Test passes (200 OK, content contains primes/2/3) — the full Gemma 4 E4B native-function-calling path works end-to-end.
**Why human:** Requires physical GGUF download and a running llama-server; cannot be verified programmatically here.

### 2. probe_backend_capabilities live HTTP path

**Test:** Start the server, launch the agent, and check the `[Runtime] Backend capabilities probed: model=..., function_calling=..., context_window=...` log line.
**Expected:** `function_calling=true` (gemma model id or `has_jinja: true`) and `context_window` matching the model profile.
**Why human:** The probe's `/v1/models` + `/props` HTTP paths are never exercised by any test (only the defaults unit test exists); requires a live server.

### Gaps Summary

All 31 plan-level must-haves (4 plans) and all declared verification gates (pytest 52 passed, cargo test full suite green, clippy `-D warnings` exit 0, 28/28 automated eval tests green) are **verified**. The eval suite is real, wired, and green. However, **the phase goal as stated in ROADMAP.md is not fully achieved**:

1. **BLOCKER — Default model not switched.** `DEFAULT_MODEL_NAME` still resolves to `Qwen-3.6-27B-MoE`; Gemma 4 E4B was added to the catalog but never made the default, and `test_qwen_config.py:50` actively asserts the Qwen default. The goal's first clause ("Switch the default test model to Gemma 4 E4B-it") is unmet.
2. **Retention metric not implemented.** The ≥90% / 3-compaction-cycle recall benchmark does not exist; retention tests are 1-cycle presence checks.
3. **"All 8 scenarios pass" not demonstrated.** SC8's live-model test is `#[ignore]`d and never run; the reporting test and eval-results.json hardcode SC8 as failed (7/8).
4. **0% instruction-following not measured.** The sanitization mechanism is verified, but no test measures instruction-following rate from untrusted content.

These are roadmap-contract gaps, not plan-execution failures — the plans delivered exactly what they scoped. Closing them requires goal-level follow-up (default-model switch, retention benchmark, live SC8 run, injection-rate measurement).

---

_Verified: 2026-08-03T22:30:00Z_
_Verifier: the agent (gsd-verifier)_
