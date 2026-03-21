# Integrations

## Inference Backends
- Primary backend: llama.cpp server process launched by `scripts/start_server.py`.
- Fallback backend: KoboldCPP process launched by `scripts/start_server.py`.
- Both expose local OpenAI-compatible API base path: `http://127.0.0.1:8080/v1`.

## Model Distribution
- Model downloads are orchestrated in `setup.py`.
- Default model sources reference Hugging Face repositories.
- Local model storage path: `models/`.

## Local Process Integrations
- `start.py` coordinates Python server startup and Rust orchestrator handoff.
- Rust orchestrator loads config through Python bridge in `agent-rs/src/config.rs`.
- Eval harness executes agent binary from `tests/eval.py` using subprocess.

## API-style Integration Points
- Health/readiness probing uses `GET /v1/models` in `start.py` and setup preflight.
- Inference calls use chat/completion endpoints from Rust in `agent-rs/src/main.rs`.
- Benchmark judging endpoint configured by `HELIX_JUDGE_URL` in `tests/eval.py`.

## Environment Variables in Use
- `HELIX_MODEL_NAME`, `HELIX_MODEL_PATH` for runtime model selection.
- `HELIX_SERVER_STARTUP_TIMEOUT_S` for startup wait budget.
- `HELIX_EVAL_MAX_TASKS`, `HELIX_EVAL_CATEGORIES` for preflight scope.
- `HELIX_RUN_AGENTIC_PREFLIGHT` for setup preflight opt-in/out.

## External Services
- No mandatory cloud runtime service during inference loop.
- Optional external downloads only during setup/model acquisition.

## Security Surface
- Command execution tool exists in Rust tool layer (`agent-rs/src/tools.rs`).
- Dangerous command gating exists via configured deny list and confirmation flags.
