# GPT-OSS Agent — Local LLM Automation

A cross-platform local LLM-powered automation agent with tool-calling capabilities. Runs entirely on your machine using open-source models via llama.cpp or KoboldCPP.

## What This Project Is

GPT-OSS Agent is a local-first AI automation stack that combines:

- A Python setup and server layer for model management and inference runtime
- A Rust orchestrator for safe, structured tool-calling and execution
- OpenAI-compatible local API serving for browser and client integrations

The project is designed for users who want strong control over privacy, model choice, and system-level automation without relying on cloud-hosted LLM backends.

## Problems This Project Solves

- **Cloud dependence**: Run fully local inference and tooling workflows on your own machine.
- **Unsafe tool execution**: Enforce strict sandbox boundaries and typed tool arguments.
- **Complex local setup**: Automate model download, hardware-aware defaults, and backend fallback.
- **Single-backend fragility**: Use llama.cpp first with KoboldCPP fallback for better reliability.
- **Frontend lock-in**: Expose an OpenAI-compatible endpoint that works with many UIs.

## Architecture

High-level request flow:

1. User starts the stack with `start.py` or two-terminal mode.
2. Python server launcher boots local inference backend (`llama-server` or KoboldCPP fallback).
3. Rust orchestrator (`agent-rs`) runs the interactive agent loop.
4. Orchestrator calls local model endpoint and invokes constrained tools.
5. Tool outputs are fed back into the agent loop for next-step reasoning.

Component map:

- **Setup Layer (Python)**: `setup.py`, `system_check.py`, generated `config.py`
- **Serving Layer (Python)**: `start_server.py` (backend startup and fallback)
- **Orchestration Layer (Rust)**: `agent-rs/` (core agent logic, tools, token management)
- **Model Runtime Layer**: `llama.cpp` and optional KoboldCPP binary
- **Client Integration Layer**: OpenAI-compatible API at `http://127.0.0.1:8080/v1`

## Tech Stack Used

- **Languages**: Python 3, Rust
- **Inference backends**: llama.cpp, KoboldCPP (fallback)
- **Model format**: GGUF
- **API compatibility**: OpenAI-style REST API
- **Python libraries**: `requests`, `tqdm`, `openai`
- **Rust ecosystem**: Cargo-based project (`agent-rs/Cargo.toml`)
- **Build tools**: CMake (for llama.cpp builds), optional CUDA/OpenVINO/Vulkan backends

## Features

- **Rust Orchestrator** — High-performance, memory-safe agent core (`agent-rs`)
- **Strict Sandbox** — OS-level canonicalization strictly blocks execution/reads outside the allowed directory
- **Typed Tools** — Strict JSON-schema deserialization auto-catches and self-heals LLM hallucinations
- **Multi-model support** — GPT-OSS-20B and Qwen3.5-9B-Uncensored
- **Automatic setup** — Downloads models, installs dependencies, configures GPU layers
- **Dual backend** — llama.cpp (primary) with KoboldCPP fallback
- **Cross-platform** — Linux and Windows support
- **Browser compatible** — Works with Open WebUI or any OpenAI-compatible UI

---

## How To Run

### Recommended (single command)

After initial setup, start everything with:

```bash
source venv/bin/activate
python start.py
```

`start.py` will:

1. Start the local LLM server in the background
2. Wait for the API to become ready
3. Launch the Rust orchestrator and persona menu
4. Teardown server process when you exit

### First-time setup

```bash
python3 -m venv venv
source venv/bin/activate
pip install requests tqdm openai
python setup.py
```

### Manual two-terminal mode

Terminal 1:

```bash
source venv/bin/activate
python scripts/start_server.py
```

Terminal 2:

```bash
source venv/bin/activate
cd agent-rs && cargo run
```

---

## Setup — Linux

### Prerequisites

```bash
# Debian/Ubuntu
sudo apt update && sudo apt install -y python3 python3-venv python3-pip git build-essential cmake

# Arch Linux
sudo pacman -S python python-pip git base-devel cmake

# Fedora
sudo dnf install python3 python3-pip git gcc gcc-c++ cmake
```

**NVIDIA GPU (optional):**
```bash
# Debian/Ubuntu
sudo apt install -y nvidia-cuda-toolkit

# Arch Linux
sudo pacman -S cuda

# Verify
nvidia-smi
```

### Installation

```bash
# 1. Clone or extract the project
cd ~/
git clone <your-repo-url> gpt-oss-agent   # or extract the release zip
cd gpt-oss-agent

# 2. Create and activate a virtual environment
python3 -m venv venv
source venv/bin/activate

# 3. Install Python dependencies
pip install requests tqdm openai

# 4. (Optional) Build llama.cpp from source
cd llama.cpp
mkdir -p build && cd build

# CPU only:
cmake .. && cmake --build . --config Release -j$(nproc)

# With CUDA (NVIDIA GPU):
cmake .. -DGGML_CUDA=ON && cmake --build . --config Release -j$(nproc)

cd ~/gpt-oss-agent
```

### Run Setup

```bash
source venv/bin/activate
python setup.py
```

You'll be prompted to:
1. Choose a model: **GPT-OSS-20B**, **Qwen3.5-9B**, or **ALL**
2. KoboldCPP will be downloaded automatically as a fallback
3. Set GPU layers (see [GPU Guide](#gpu-configuration-guide) below)
4. `config.py` is generated for you

### Start the Agent

```bash
# Terminal 1 — Start the LLM server
source venv/bin/activate
python start_server.py

# Terminal 2 — Start the Rust agent
source venv/bin/activate
cd agent-rs && cargo run
```

---

## Setup — Windows

### Prerequisites

1. **Python 3.10+** — Download from [python.org](https://www.python.org/downloads/). During install, check **"Add Python to PATH"**.
2. **Git** — Download from [git-scm.com](https://git-scm.com/download/win).
3. **(Optional) NVIDIA GPU drivers** — Install the latest from [nvidia.com/drivers](https://www.nvidia.com/drivers).
4. **(Optional) Visual Studio Build Tools** — Required only if building llama.cpp from source. Download from [visualstudio.microsoft.com](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

### Installation

Open **Command Prompt** or **PowerShell**:

```powershell
# 1. Clone or extract the project
cd %USERPROFILE%
git clone <your-repo-url> gpt-oss-agent
cd gpt-oss-agent

# 2. Create and activate a virtual environment
python -m venv venv
.\venv\Scripts\activate

# 3. Install Python dependencies
pip install requests tqdm openai

# 4. (Optional) Build llama.cpp from source
cd llama.cpp
mkdir build && cd build

# CPU only (requires Visual Studio Build Tools):
cmake .. && cmake --build . --config Release

# With CUDA:
cmake .. -DGGML_CUDA=ON && cmake --build . --config Release

cd %USERPROFILE%\gpt-oss-agent
```

### Run Setup

```powershell
.\venv\Scripts\activate
python setup.py
```

You'll be prompted to:
1. Choose a model: **GPT-OSS-20B**, **Qwen3.5-9B**, or **ALL**
2. KoboldCPP `.exe` will be downloaded automatically as a fallback
3. Set GPU layers (see [GPU Guide](#gpu-configuration-guide) below)
4. `config.py` is generated for you

### Start the Agent

```powershell
# Terminal 1 — Start the LLM server
.\venv\Scripts\activate
python start_server.py

# Terminal 2 — Start the Rust agent
.\venv\Scripts\activate
cd agent-rs && cargo run
```

**One-shot mode (both OS):**  
*(Note: One-shot mode will be re-added to the Rust agent in a future update. For now, use interactive mode.)*

---

## Browser UI (Optional)

The server exposes an OpenAI-compatible API at `http://127.0.0.1:8080/v1`.

You can connect any compatible frontend:
- **[Open WebUI](https://github.com/open-webui/open-webui)** — Full-featured chat UI
- Any app supporting the OpenAI API format (e.g., Chatbox, BetterChatGPT)

Just point the API base URL to `http://127.0.0.1:8080/v1`.

---

## Available Models

| Model | File Size | Quantization | Best For |
|-------|-----------|-------------|----------|
| GPT-OSS-20B | ~11.8 GB | IQ4_NL | General tasks, creative writing |
| Qwen3.5-9B-Uncensored | ~9.5 GB | Q4_K_M | Instruction following, coding |

---

## GPU Configuration Guide

| GPU VRAM | Recommended Layers | Notes |
|----------|-------------------|-------|
| CPU only | `0` | Slowest, but works everywhere |
| 4 GB | `4–8` | Partial offload, good for light use |
| 6 GB | `10–20` | Decent speed for the 9B model |
| 8 GB | `25–35` | Good speed for both models |
| 12 GB+ | `-1` (full offload) | Maximum speed |

> **Tip:** If you run out of VRAM, reduce the number of layers. The remaining layers run on CPU.

---

## Project Structure

```
gpt-oss-agent/
├── setup.py           # Interactive setup — downloads models, detects specs, auto-configs
├── system_check.py    # Hardware detection and tier rating (CPU, RAM, GPU)
├── start_server.py    # Server launcher (llama.cpp → KoboldCPP fallback)
├── agent-rs/          # High-performance Rust Orchestrator
├── agent.py           # (Legacy) Python Agent CLI
├── tools.py           # Tool definitions (terminal, file I/O)
├── config.py          # Generated configuration (created by setup.py)
├── build_zip.py       # Package source for distribution
├── README.md          # This file
├── models/            # Downloaded model files (not in git)
├── logs/              # Command logs (not in git)
├── venv/              # Python virtual environment (not in git)
└── llama.cpp/         # llama.cpp source and build (optional)
```

---

## Switching Models

After setup, edit `config.py` to change the active model:

```python
# Switch to the other downloaded model
MODEL_NAME = "Qwen3.5-9B-Uncensored"
MODEL_PATH = os.path.join(PROJECT_DIR, "models", "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf")
```

Or re-run `python setup.py` to reconfigure everything with fresh auto-detection.

Then restart the server (`Ctrl+C` in Terminal 1, then `python start_server.py` again).

---

## Packaging for Distribution

```bash
python build_zip.py
```

Creates `gpt-oss-agent-release.zip` containing only source files (~18 KB). Excludes `models/`, `venv/`, `.git/`, `llama.cpp/`, `logs/`.

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `ModuleNotFoundError: No module named 'requests'` | Activate venv first: `source venv/bin/activate` |
| `Cannot connect to the LLM server` | Start the server first: `python start_server.py` |
| `llama-server crashed immediately` | KoboldCPP fallback should trigger automatically. If both fail, check your model path in `config.py` |
| Out of VRAM | Reduce `GPU_LAYERS` in `config.py` and restart the server |
| `externally-managed-environment` (Arch/Ubuntu 23+) | Use the venv: `source venv/bin/activate` |

---

## Configuration Reference

All settings in `config.py` (auto-generated by `setup.py` based on system detection):

| Setting | Description | Default |
|---------|-------------|---------|
| `MODEL_NAME` | Active model name | Auto-selected by tier |
| `MODEL_PATH` | Path to the `.gguf` file | Project-relative |
| `AVAILABLE_MODELS` | Dict of all downloaded models | Set during setup |
| `GPU_LAYERS` | Layers offloaded to GPU | Auto-detected by tier |
| `CONTEXT_SIZE` | Context window size | Auto-detected (2048–16384) |
| `CPU_THREADS` | CPU threads for inference | Auto-detected |
| `BATCH_SIZE` | Processing batch size | Auto-detected (256–1024) |
| `UBATCH_SIZE` | Micro-batch size | `BATCH_SIZE / 2` |
| `SERVER_PORT` | API server port | `8080` |
| `REQUIRE_CONFIRMATION` | Ask before running commands | `True` |
| `KOBOLD_BIN` | KoboldCPP binary filename | Auto-detected by OS |

---

## License

Open source. See individual model licenses for usage terms.
