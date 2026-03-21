# ARCHITECTURE.md

## Core Architectural Pattern
Helix Agent uses a **multi-process hybrid architecture** separating environment management from fast, safe execution.

1.  **Boot & Infrastructure Layer (Python):** Provisions dependencies, builds binaries, benchmarks hardware, and manages the lifecycle of the local server daemon.
2.  **Inference Layer (C++):** A standalone local LLM HTTP server (`llama-server`) processing inputs without blocking the main workflow.
3.  **Orchestration Layer (Rust):** The cognitive loop. It handles LLM interactions, enforces schemas, executes local system tools, and handles memory limits.

## Component Interactions & Data Flow
1.  **Launch (`start.py`):** The user invokes Python, which spawns the `start_server.py` daemon in the background to host the LLM. Wait gates are used to ensure the HTTP endpoint is active.
2.  **Handoff:** Python calls `cargo run` to boot the `agent-rs` Rust binary, transferring foreground terminal control to Rust.
3.  **Agent Loop (`agent-rs/src/main.rs`):**
    *   Rust gathers system config via `config.py` (parsed from Python).
    *   Sends context + system prompt + tool schemas to LLM via local HTTP.
    *   Receives response. If `tool_calls` are present, Rust maps them to native functions (`tools::execute_read_file`, etc.).
    *   Appends `tool` response roles to the context window and loops.
4.  **Memory Compaction:** The Rust loop monitors total tokens (via `tiktoken-rs`). Once a threshold (e.g., 70% of context size) is breached, it interrupts the loop. It slices the oldest messages, sends them to the LLM with a "summarize" directive, and replaces the history chunk with a single dense summary message to free up context space.
5.  **Teardown:** On Rust exit, the `start.py` orchestrator catches the return and successfully terminates the background `llama-server` process.

## Extension & Customization Points
*   **Personas:** The Rust orchestrator determines tool access based on the `AGENT_PERSONA` env var (e.g., `os_assistant` gets terminal commands, `coder` is restricted to file I/O).
*   **Memory Critic Feedback:** Rust intercepts tool usage schema errors and command failures, manually injecting synthetic "Critic" prompts to dynamically guide the LLM to self-correct during the next iteration.
