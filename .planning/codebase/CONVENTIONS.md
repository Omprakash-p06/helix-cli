# CONVENTIONS.md

## Code Style & Patterns
*   **Python:** Uses basic procedural scripts (`setup.py`, `start.py`, `system_check.py`). Very little OOP structure. Heavy use of `subprocess.Popen` and `subprocess.check_call` for booting binaries, managing threads, and managing system state.
*   **Rust:** Embraces typical `tokio` async patterns. The `main.rs` function runs an infinite interactive loop for the agent, maintaining an array of `ChatMessage` objects in memory.
*   **Configuration Handoff:** Uses a unique pattern where `setup.py` detects hardware, generates a hard-coded Python `config.py` dictionary. `agent-rs/src/config.rs` invokes a Python sub-process to dump `config.py` into a temporary JSON format, reads it via `serde_json`, and uses the strongly typed struct in Rust.

## Error Handling
*   **Python:** Often wrapped in basic `try/except` blocks to prevent crashes on missing OS dependencies (e.g., catching failed HTTP lookups).
*   **Rust:** Uses `Result` comprehensively without panic unwrapping for network requests (`reqwest`), giving detailed failure prints. Invalid tool executions return a `ToolResult` struct containing an error message rather than crashing the orchestrator, and an inline synthetic `Critic` prompt is automatically appended to help the LLM self-correct.

## Naming
*   Standard PEP8 snake_case for Python.
*   Standard Rust idiomatic naming (snake_case for functions/variables, PascalCase for Structs/Enums).
