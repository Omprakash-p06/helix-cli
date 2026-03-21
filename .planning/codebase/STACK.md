# STACK.md

## Overview
Helix Agent uses a hybrid Python/Rust stack to manage local AI inference and autonomous agent orchestration.

## Primary Languages & Runtimes
*   **Python (3.x):** Used for the setup, provisioning, benchmarking, and launcher layers.
*   **Rust (2024 edition):** Used for the high-performance orchestration layer, handling LLM iterative loops and tool execution.
*   **C/C++:** Powering the `llama.cpp` underlying inference engine.

## Inference Backends
*   **llama.cpp:** Primary inference engine, compiled locally during setup with optimizations (CUDA, Vulkan, OpenVINO, or CPU AVX/AMX).
*   **KoboldCPP:** Used as a fallback inference engine if `llama.cpp` fails.

## Key Dependencies (Python)
*   `requests`, `tqdm`: Used for HTTP requests and progress bars (e.g., downloading `.gguf` models).
*   `openai`: For API interactions (though custom HTTP calls are also used).

## Key Dependencies (Rust)
Found in `agent-rs/Cargo.toml`:
*   `tokio`: Async runtime.
*   `reqwest`: HTTP client for talking to the local OpenAI endpoint.
*   `serde_json`, `serde`, `schemars`: JSON serialization and JSON Schema generation for OpenAI tool calling.
*   `async-openai`, `tiktoken-rs`: Token counting and OpenAI schema definitions.
*   `sysinfo`, `ignore`: Utilities for system metrics and filesystem exploration.

## Data & Configuration
*   **Configuration:** Auto-generated `scripts/config.py` created by `setup.py` after hardware benchmarking.
*   **Model Weights:** Stored as `.gguf` files in the `models/` directory.

## Build Tools
*   **Cargo & rustup:** For compiling the `agent-rs` orchestrator.
*   **CMake / MSBuild (Windows) / Make:** For compiling `llama.cpp` natively.
