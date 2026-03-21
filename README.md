# Helix Agent

Helix Agent is a local-first AI automation stack for running an LLM with tool-calling on your own machine.

## What problem this project solves

Most AI automation workflows depend on cloud APIs, limited tool permissions, or fragile one-off scripts. Helix solves this by combining:

- Local model inference (privacy + offline-friendly workflows)
- A Rust orchestrator with typed tool calls and safer execution boundaries
- A hardware-aware setup flow that tunes backend/runtime values for your system
- A stable OpenAI-compatible API endpoint for existing clients and UIs

## Current system architecture

Helix is a Py + Rust hybrid stack:

1. Setup layer (Python)
- `setup.py` detects hardware, installs dependencies, builds llama.cpp, downloads models, benchmarks throughput, and generates `scripts/config.py`.

2. Runtime server layer (Python)
- `scripts/start_server.py` launches `llama-server` from llama.cpp.
- If llama.cpp fails, it falls back to KoboldCPP when available.

3. Orchestration layer (Rust)
- `agent-rs/src/main.rs` runs the local CLI orchestrator.
- It calls the OpenAI-compatible endpoint, evaluates tool-call plans, executes tools, and loops with memory compaction logic.

4. Launch orchestration (Python)
- `start.py` is the single-command launcher:
  - prompts model selection,
  - boots model server in background,
  - waits for readiness,
  - hands off to Rust orchestrator,
  - tears down server on exit.

5. Branding/UI layer
- `scripts/helix_branding.py` provides shared terminal branding used by setup/server startup.

## Repository layout (important files)

- `setup.py` - full setup + benchmark gate + config generation
- `start.py` - one-command stack launcher
- `scripts/start_server.py` - model backend launcher/fallback
- `scripts/helix_branding.py` - shared Helix terminal logo utilities
- `scripts/system_check.py` - hardware detection + tier mapping
- `agent-rs/` - Rust orchestrator and tool runtime
- `models/` - local GGUF models
- `logs/` - runtime logs
- `llama.cpp/` - inference backend source/build

## How to run the project

### Quick start (recommended)

After setup is complete:

Linux/macOS:

```bash
source venv/bin/activate
python start.py
```

Windows (PowerShell):

```powershell
.\venv\Scripts\Activate.ps1
python start.py
```

### First-time setup

Linux/macOS:

```bash
python3 -m venv venv
source venv/bin/activate
pip install requests tqdm openai
python setup.py
```

Windows (PowerShell):

```powershell
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install requests tqdm openai
python setup.py
```

### Two-terminal mode (manual)

Terminal 1 (server):

Linux/macOS:

```bash
source venv/bin/activate
python scripts/start_server.py
```

Windows (PowerShell):

```powershell
.\venv\Scripts\Activate.ps1
python scripts/start_server.py
```

Terminal 2 (orchestrator):

```bash
cd agent-rs
cargo run
```

## API endpoint

When server is running:

- Base URL: `http://127.0.0.1:8080/v1`
- Compatible with OpenAI-style clients and many local UIs.

## Models

The setup currently supports these defaults:

- GPT-OSS-20B (IQ4_NL)
- Qwen3.5-9B-Uncensored (Q4_K_M)

Model choice and tuned runtime values are written to `scripts/config.py`.

## Troubleshooting

- If setup says `config.py` missing: run `python setup.py` first.
- If server startup fails: confirm model exists under `models/` and check `llama.cpp/build/bin`.
- If throughput gate blocks setup: reduce expectations for low-VRAM cards or rerun after adjusting runtime config.
- If `cargo` is missing: install Rust toolchain (`rustup`) and retry.

## Notes

- Helix is designed for local automation and experimentation.
- Use with care when enabling high-permission tool personas.
