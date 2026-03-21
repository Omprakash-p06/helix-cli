# Testing

## Current Test Style
- Script-driven evaluation rather than unit-test-first approach.
- Main benchmark harness in `tests/eval.py`.
- Dataset-driven tasks defined in `tests/dataset.json`.

## Preflight and Benchmarking
- Setup can run token speed gates in `setup.py`.
- Agentic benchmark preflight is now optional and scope-limited by default.
- Eval supports env-driven task limits and category filtering:
  - `HELIX_EVAL_MAX_TASKS`
  - `HELIX_EVAL_CATEGORIES`

## Validation Practices Used
- Syntax validation via `python -m py_compile` for changed Python scripts.
- Rust compile validation via `cargo build` in `agent-rs/`.
- Runtime smoke tests via one-shot prompt and server startup checks.

## Gaps and Risks
- Limited automated unit tests for Python startup orchestration paths.
- Limited deterministic integration tests for llama.cpp/KoboldCPP switching.
- High reliance on manual runtime checks and log inspection.

## Recommended Next Testing Layers
- Add Python unit tests for startup/model selection functions in `start.py`.
- Add regression tests for startup timeout and log-tail behavior.
- Add Rust tests for message display sanitization (`<think>` stripping).
- Add backend detection tests for binary naming on Windows/Linux.

## Operational Test Artifacts
- Benchmark report output in `tests/benchmark_results.md`.
- Runtime logs in `logs/start_server.stdout.log` and `logs/start_server.stderr.log`.
