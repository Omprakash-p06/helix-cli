---
status: testing
phase: 10-gemma-4-e4b-integration-evaluation-suite
source:
  - 10-VERIFICATION.md
started: 2026-08-03T02:23:30Z
updated: 2026-08-03T23:15:00Z
---

## Current Test

number: 6
name: Live Gemma 4 E4B end-to-end inference (SC8 live variant)
expected: |
  Download gemma-4-E4B-it-Q4_K_M.gguf (+ mmproj-F16.gguf), start llama-server on
  127.0.0.1:8080 with --jinja, then run:
  `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`
  The test passes (200 OK, content contains primes/2/3), demonstrating the full Gemma 4
  E4B integration works end-to-end with native function calling.
awaiting: user response

## Tests

### 1. Gemma 4 E4B Model Catalog, Server Flags & Downloader Preset (10-01)
expected: 3 VRAM variants (8GB, 4GB, CPU) in config.py, --jinja/--mmproj in start_server.py, presets in download_model.py
result: pass
source: automated
coverage_id: 10-01-D1..D3

### 2. BackendCapabilities Probing & AppConfig Wiring (10-02)
expected: probe_backend_capabilities probes /v1/models & /props, wires capability flags into AppConfig
result: pass
source: automated
coverage_id: 10-02-D1..D2

### 3. Evaluation Suite Core Scenarios 1-7 & EvalResult Type (10-03)
expected: 24 integration tests green for SC1 comprehension, SC2 tool_call, SC3 retention, SC4 research, SC5/6 security, SC7 rollback
result: pass
source: automated
coverage_id: 10-03-D1..D7

### 4. End-to-End Pipeline Scenario & Eval Reporter (10-04)
expected: SC8 e2e pipeline tests (WorkerPool, FreshnessClassifier, EvidenceSynthesizer) & eval_suite_reporting
result: pass
source: automated
coverage_id: 10-04-D1..D2

### 5. Gap Closure — Default Model, Retention Benchmark, Injection Rate & SC8 Reporting (10-05)
expected: DEFAULT_MODEL_NAME switched to Gemma-4-E4B, 3-cycle >=90% recall benchmark, 0%-escape injection measurement, 8/8 reporting gate
result: pass
source: automated
coverage_id: 10-05-D1..D4

### 6. Live Gemma 4 E4B end-to-end inference (SC8 live variant)
expected: |
  Download gemma-4-E4B-it-Q4_K_M.gguf (+ mmproj-F16.gguf), start llama-server on
  127.0.0.1:8080 with --jinja, then run:
  `cargo test --package agent-rs --test eval_suite_e2e sc8_live_model -- --ignored`
  The test passes (200 OK, content contains primes/2/3) — the full Gemma 4 E4B
  native-function-calling path works end-to-end.
result: [pending]
source: human_verification

### 7. Live backend capabilities probe (probe_backend_capabilities HTTP path)
expected: |
  Start the server, launch the agent, and observe the log line:
  `[Runtime] Backend capabilities probed: model=..., function_calling=..., context_window=...`
  function_calling=true (model_id contains gemma or /props has_jinja) and context_window
  matches the model profile; the probe's /v1/models and /props HTTP paths are exercised.
result: [pending]
source: human_verification

## Summary

total: 7
passed: 5
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps

- Gap 1 (10-05): 3-cycle >=90% recall benchmark added — CLOSED
- Gap 2 (10-05): 0% instruction-following rate measurement added — CLOSED
- Gap 3 (10-05): SC8 reporting gate raised to 8/8, SC8 marked passed — CLOSED
- Gap 4 (10-05): DEFAULT_MODEL_NAME switched to Gemma-4-E4B — CLOSED
- Remaining: 2 human verification items (tests 6 & 7) require a live llama-server + Gemma 4 E4B GGUF
