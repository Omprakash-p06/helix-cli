# Technology Stack

**Analysis Date:** 2025-05-15

## Languages

**Primary:**
- Rust (2024 Edition) - Core agent logic, server, TUI, and security engine in `agent-rs/`.
- TypeScript - Web UI and GSD 2.0 Pi SDK orchestration logic in `web-ui/` and GSD integration.

**Secondary:**
- Python 3.10+ - Setup, configuration, and model management scripts in `scripts/`.
- C++ - High-performance LLM inference engine in `llama.cpp/`.

## Runtime

**Environment:**
- Rust Toolchain - For compiling `agent-rs/`.
- Node.js (Vite/React) - For the `web-ui/`.
- Python 3.x - For automation and fallback scripts.

**Package Manager:**
- Cargo (Rust) - Lockfile: `agent-rs/Cargo.lock`
- npm (Node.js) - Lockfile: `web-ui/package-lock.json`
- pip (Python) - Requirements: `requirements.txt`

## Frameworks

**Core:**
- Axum - Rust web framework for the agent API in `agent-rs/src/server.rs`.
- React + Vite - Frontend framework in `web-ui/`.
- GSD 2.0 (Pi SDK) - Orchestration state machine for autonomous workflows.

**Testing:**
- Pytest - Python-based integration and system tests in `tests/`.
- Cargo Test - Unit tests for Rust core in `agent-rs/src/`.

**Build/Dev:**
- CMake - For building `llama.cpp/`.
- Tailwind CSS - For `web-ui/` styling.

## Key Dependencies

**Critical:**
- `llama.cpp` - Local LLM inference provider.
- `async-openai` - Rust client for OpenAI-compatible APIs (local server).
- `tokio` - Async runtime for the Rust agent.
- `ratatui` - Terminal UI framework for the Rust agent.

**Infrastructure:**
- `rusqlite` - SQLite storage for sessions and audit logs.
- `sysinfo` - Hardware-aware profiling for model selection.
- `gbnf` - Grammar-based sampling for structured tool calling.

## Configuration

**Environment:**
- `.env` (Existence only) - Contains backend hints and model paths.
- `scripts/config.py` - Python-side configuration for model paths and server flags.

**Build:**
- `agent-rs/Cargo.toml`
- `web-ui/package.json`
- `llama.cpp/CMakeLists.txt`

## Platform Requirements

**Development:**
- Rustup, Node.js, Python 3, CMake, C++ Compiler.

**Production:**
- Consumer hardware with 8GB-24GB+ VRAM (NVIDIA/Apple Silicon/AMD) for Qwen 3.6 local execution.

---

*Stack analysis: 2025-05-15*
