# INTEGRATIONS.md

## Local API Endpoints
*   **OpenAI-Compatible Local Server:** The `llama-server` (or KoboldCPP fallback) binds to `127.0.0.1:8080/v1` by default.
    *   `/v1/models`: Queried by both Python launchers and Rust orchestrator to check readiness and which model is loaded.
    *   `/v1/chat/completions`: The core endpoint used by `agent-rs` to run the agentic loop.
    *   `/v1/completions`: Used by `setup.py` during hardware benchmarking to measure token-per-second throughput.

## External Services
*   **Hugging Face API:**
    *   `https://huggingface.co/api/models`: The `setup.py` scripts hit this REST API to search repositories and inspect repo trees for `.gguf` files during universal model downloading.
*   **GitHub Releases:**
    *   Used to download pre-compiled `koboldcpp` binaries directly based on the OS platform.

## Tool Integrations (Rust Orchestrator)
The Rust agent defines several built-in OS tools it executes directly on the system:
*   **File System:** `read_file`, `write_file`, `append_file`, `list_directory`.
*   **Terminal:** `run_terminal_command` (restricted to `os_assistant` persona, runs arbitrary shell commands with confirmation gating).
*   **System Stats:** `get_system_stats` interacts with the host OS natively via the `sysinfo` crate.
