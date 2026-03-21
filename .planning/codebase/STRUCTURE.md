# Structure

## Repository Layout
- Root launcher and setup:
  - `start.py`
  - `setup.py`
- Python runtime support:
  - `scripts/start_server.py`
  - `scripts/system_check.py`
  - `scripts/helix_branding.py`
- Rust orchestrator crate:
  - `agent-rs/Cargo.toml`
  - `agent-rs/src/main.rs`
  - `agent-rs/src/tools.rs`
  - `agent-rs/src/config.rs`
- Inference backend source/build:
  - `llama.cpp/`
  - `llama.cpp/build/`
- Models and logs:
  - `models/`
  - `logs/`
- Testing/eval:
  - `tests/eval.py`
  - `tests/dataset.json`

## Planning and Workflow Files
- GSD skills under `.github/skills/`.
- GSD workflows under `.github/get-shit-done/workflows/`.
- Newly mapped docs under `.planning/codebase/`.

## Generated/Transient Areas
- Python cache under `__pycache__/`.
- Rust artifacts under `agent-rs/target/`.
- llama.cpp build artifacts under `llama.cpp/build/`.

## Naming and Organization Patterns
- Top-level scripts are operational entrypoints.
- Script utilities grouped under `scripts/`.
- Rust code grouped by concern in `agent-rs/src/`.
- Tests are lightweight harness style, centered around benchmark/eval scripts.

## Observed Coupling Points
- `start.py` and `scripts/start_server.py` share env and log contracts.
- `setup.py` and `scripts/config.py` share generated configuration model.
- `tests/eval.py` depends on built Rust binary path conventions.
