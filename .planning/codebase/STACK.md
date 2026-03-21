# Stack

## Languages
- Python for setup and runtime launching.
- Rust for orchestration and tool execution.
- C/C++ build artifacts from llama.cpp backend runtime.

## Runtime Components
- Python runtime entry: `start.py`.
- Python server launcher: `scripts/start_server.py`.
- Rust orchestrator binary: `agent-rs/target/debug/agent-rs.exe`.
- Inference backend binary: `llama.cpp/build/bin/llama-server`.
- Fallback backend binary: `koboldcpp.exe` at repo root.

## Core Dependencies
- Python deps installed in setup: `requests`, `tqdm`, `openai`.
- Rust deps in `agent-rs/Cargo.toml`: `tokio`, `reqwest`, `serde`, `serde_json`, `schemars`, `sysinfo`, `tiktoken-rs`, `ignore`.

## Build and Toolchain
- Python setup orchestration in `setup.py`.
- Rust build toolchain via `cargo` from `agent-rs/`.
- llama.cpp CMake output under `llama.cpp/build/`.

## Configuration Sources
- Generated runtime config in `scripts/config.py`.
- Hardware tiering and tuning logic in `scripts/system_check.py`.
- Runtime model override via env vars in `scripts/start_server.py`:
  - `HELIX_MODEL_NAME`
  - `HELIX_MODEL_PATH`

## Context and Throughput Defaults
- Minimum user context is enforced in `setup.py` with `MIN_USER_CONTEXT_SIZE = 8192`.
- Eval preflight defaults are constrained in `tests/eval.py` via env-filtered task selection.

## Notes
- Repo is brownfield and already has mixed-language build artifacts.
- Startup path is designed for local/offline inference with OpenAI-compatible endpoint behavior.
