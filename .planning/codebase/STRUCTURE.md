# STRUCTURE.md

## Directory Layout
*   `agent-rs/`: The Rust orchestrator project.
    *   `src/main.rs`: Core execution loop, HTTP API interaction, memory compaction.
    *   `src/tools.rs`: Defines and executes local tool calls (Terminal, Filesystem, Stats).
    *   `src/config.rs`: Bridges Python `config.py` settings into Rust.
    *   `src/tokens.rs`: Token counting logic.
*   `scripts/`: Python utilities and server launchers.
    *   `start_server.py`: Launches `llama.cpp` or `koboldcpp`.
    *   `system_check.py`: Checks host hardware mappings.
    *   `config.py`: Generated configuration values.
    *   `helix_branding.py`: Reusable ASCII UI components.
*   `models/`: Directory where `.gguf` weights are downloaded and stored.
*   `logs/`: Runtime standard out / error logs for the isolated LLM server process.
*   `llama.cpp/`: The upstream C++ inference engine submodule (built locally).
*   `/`: Root deployment items.
    *   `setup.py`: Unified installation, benchmarking, and build script.
    *   `start.py`: One-command runtime stack launcher.

## Key Boundaries
*   **Python <-> Rust:** There is no direct FFI. Python prepares the environment and spawns Rust as a sub-process. Rust reads Python configs by spawning Python specifically to dump the `config.py` object out as JSON.
*   **Orchestrator <-> Engine:** Completely decoupled via HTTP REST APIs over `127.0.0.1:8080`.
