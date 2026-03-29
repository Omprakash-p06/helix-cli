# CONCERNS.md

## Snapshot
Last refreshed: 2026-03-29
This file tracks architectural and operational risks observed in current codebase state.

## High-Risk Concerns

### 1) Repository Noise from Vendored `llama.cpp`
- Symptom: search and indexing are heavily polluted by upstream files.
- Impact: slower navigation and easier mistakes during edits.
- Mitigation: scope searches to Helix-owned paths (`agent-rs/`, `scripts/`, `web-ui/`, `.planning/`).

### 2) Process Orchestration Complexity in `start.py`
- Symptom: one script manages model startup, readiness checks, rust process, and optional web process.
- Impact: failure handling and teardown paths can become brittle across modes.
- Mitigation: keep logs centralized and consider isolating startup stages into testable units.

### 3) Cross-Language Config Bridge Fragility
- Symptom: Rust runtime config depends on Python import and execution success.
- Impact: missing or broken `scripts/config.py` blocks agent startup.
- Mitigation: validate config at startup with clearer preflight errors and fallback defaults.

## Medium-Risk Concerns

### 4) SSE Parsing and Stream Robustness
- Symptom: stream handling spans Rust server, parser utilities, and React client parsing loop.
- Impact: partial/malformed chunks can degrade UX or hide output.
- Mitigation: expand parser/client tests with chunk-boundary edge cases.

### 5) Tool Surface and Safety Expectations
- Symptom: agent tools include file writes and shell execution.
- Impact: misconfiguration could increase local risk.
- Mitigation: keep sandbox checks strict and dangerous command lists explicit.

### 6) Test Coverage Imbalance
- Symptom: web UI and orchestration paths have weaker automated coverage than core Rust modules.
- Impact: regressions can slip into integration flows.
- Mitigation: prioritize integration tests around `/chat` SSE and launcher modes.

## Low-Risk/Operational Observations
- Large logs can accumulate under `logs/` during repeated runs.
- Multiple interface modes increase combinatorial QA matrix.
