# Roadmap

## Phase 1: Boot Upgrades & Mode Selection
**Goal:** Enhance the single-command launcher (`start.py`) to provide the necessary UX prompts for UI and Agentic mode selection.
*   **Requirements:** REQ-04, REQ-05
*   **Success Metrics:** Executing `start.py` visibly pauses for user input, successfully captures variables for Web/Terminal and Agentic/Chat modes, and passes these variables gracefully into the Rust orchestrator environment.

## Phase 2: Rich Terminal Upgrades
**Goal:** Replace the naive standard input loop in `agent-rs/src/main.rs` with a robust text-area handling crate.
*   **Requirements:** REQ-07
*   **Success Metrics:** A user can copy a 50-line code snippet, paste it into the Rust terminal prompt, and naturally submit the entire block without the terminal fracturing the input lines or dropping characters.

## Phase 3: Grammar-Enforced Tool Calling
**Goal:** Ensure 100% JSON schema compliance natively at the token generation layer for local models.
*   **Requirements:** REQ-06
*   **Success Metrics:** The Rust orchestrator automatically converts `tools::ToolCallArgs` into a valid GBNF grammar string and successfully passes it as an enforced constraint to the `llama-server` `/v1/chat/completions` API endpoint during agentic turns. Hallucinated schemas drop to zero.

## Phase 4: KoboldCPP Fallback Accuracy
**Goal:** Ensure the fallback inference backend achieves the same exact tool-calling accuracy.
*   **Requirements:** REQ-06
*   **Success Metrics:** If the system is booted using `koboldcpp`, the engine is correctly invoked with accurate structural formatting (or API-compatible grammar equivalents) mimicking Phase 3's reliability.

## Phase 5: Modern Web Frontend
**Goal:** Build the alternative "Web Interface" option requested during the boot sequence.
*   **Requirements:** REQ-08
*   **Success Metrics:** A standalone web application (React/Vue/Svelte) can boot simultaneously, connect to the local server, display streaming tool executions, and handle user chat seamlessly.
