# CONCERNS.md

## Technical Debt & Fragile Areas
*   **Environment Assumptions:** The `setup.py` makes strict, broad assumptions about package managers and dependencies (e.g., `winget`, `rustup`, `cmake`, Visual C++ Build Tools). These assumptions fail easily on customized setups, leading to difficult troubleshooting.
*   **Subprocess Management:** Python handles the LLM server lifecycle as a background `.Popen()` process without strong IPC. If either the Python parent process or the Rust Orchestrator process crash abruptly, the background inference engine can become orphaned, leaving port 8080 locked.
*   **Memory Context Slicing Loss:** The Rust memory compaction logic executes a forced LLM summarization of past messages when 70% of the token context window is full. If the LLM generates a poor semantic summary, precise factual data (like file paths or function names) from early in the conversation may be irreversibly lost.

## Security
*   **Command Execution:** The Rust `run_terminal_command` tool is extremely powerful. Currently, it triggers based on the `AGENT_PERSONA` flag (`os_assistant`). A malicious or hallucinating LLM could execute destructive file-system or network commands if they bypass `app_config.dangerous_commands` filters.

## Bugs & Known Issues
*   **KoboldCPP Fallback Tuning:** Passing the correct CLI arguments dynamically to `koboldcpp` vs `llama.cpp` causes complexity. E.g., OpenVINO hardware settings are mapped awkwardly to CPU-only if falling back to `koboldcpp`.
*   **RAG Implementation:** Vector search / semantic RAG features inside the Rust orchestrator are currently commented out (`// mod rag;` in `main.rs`), meaning it lacks internal semantic memory search out-of-the-box.
