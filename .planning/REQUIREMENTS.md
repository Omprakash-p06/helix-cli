# Requirements

## Core Objective
Deliver a local-first, privacy-respecting AI automation stack that runs fast on low-end hardware while matching the tool-calling accuracy of cloud agents like Claude.

## Validated Core Features
*   **REQ-01: Local Inference Backends** - The system must wrap and manage `llama.cpp` natively, while retaining a robust fallback to `koboldcpp` if compilation fails or hardware lacks support.
*   **REQ-02: Isolated Orchestrator** - A Rust executable (`agent-rs`) must handle the cognitive loops, memory compaction token limits, and local system tool execution completely isolated from the inference engine.
*   **REQ-03: Boot Lifecycle Mapping** - `setup.py` and `start.py` must control hardware detection, prompt for model downloads natively, and synchronize the two processes via localhost port 8080.

## New Feature Requirements (Active)

### REQ-04: Dual UI Launcher Prompt
When the user executes `start.py`, before launching the Rust orchestrator, the CLI must prompt the user with two distinct options:
1. Web Interface
2. Terminal Chat

### REQ-05: Dual Execution Modes
Alongside the UI prompt, `start.py` must allow the user to select the behavioral mode:
1. Agentic Mode (autonomous tool-calling enabled)
2. Chat Mode (pure chat, no autonomous tool execution)

### REQ-06: Accurate Tool Calling via Grammar
Small 8B-14B local models frequently hallucinate JSON schema formats. To counteract this:
*   **llama.cpp:** The Rust orchestrator must dynamically convert its active JSON Schema tools into GBNF (GGML BNF) Grammar and enforce this via the backend API.
*   **koboldcpp:** Implement structural enforcement or rely on specific prompt engineering formatting if native grammar support differs for the fallback backend.

### REQ-07: Rich Terminal Input
The overarching Rust terminal loop currently uses standard stdin. It must be upgraded to support a rich terminal text area (e.g., using `rustyline` or `inquire`) that inherently supports pasting multi-line code/logs without breaking the input buffer or submitting prematurely.

### REQ-08: Modern Web Frontend
Design and build a lightweight, fast modern Javascript frontend (React/Vue/Svelte) that connects to the underlying local Orchestrator/API. It must be decoupled from the core terminal loop but provide the exact same agentic capabilities.

### REQ-09: Cross-Platform Parity
The execution environments, terminal commands, and python boot scripts must operate with identical reliability across Windows, generic Linux distributions, and specifically Arch Linux. Pathing and dependency resolutions must be fully OS-agnostic.

### REQ-10: Clinical Persona Enforcement
Completely strip all "helpful assistant" personality traits, greetings, and conversational loops from the LLM outputs. The orchestrator must enforce strict, concise, and purely functional responses. The agent should only output its reasoning and execute tools without pleasantries.

## Out of Scope
*   Handling external cloud LLM APIs (e.g., Anthropic, OpenAI) for the primary loop.
*   Building UI prototypes with Gradio/Streamlit (as rejected in favor of a modern JS framework).
