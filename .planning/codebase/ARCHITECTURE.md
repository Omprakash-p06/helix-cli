# System Architecture

**Last Updated:** 2026-07-24

## Architecture Overview

Helix CLI follows a multi-tiered, security-sandboxed agent architecture designed for local-first execution. It decouples interface interaction (TUI / Web UI), process orchestration (Python bootstrap), high-performance agent runtime & security enforcement (Rust `agent-rs`), and local neural inference (`llama-server`).

```mermaid
graph TB
    subgraph Client Layer
        TUI["Terminal UI (Ratatui / Crossterm)"]
        WebUI["Web UI (React 19 / Tailwind / Vite)"]
    end

    subgraph Rust Agent Engine ("agent-rs")
        Server["Axum REST / SSE Server (src/server.rs)"]
        TuiHandler["TUI Event Loop (src/tui/events.rs)"]
        AgentCore["Agent Core Controller (src/agent_core/mod.rs)"]
        Guardian["Security Guardian (src/agent_core/guardian.rs)"]
        ToolRuntime["Tool Execution Runtime (src/agent_core/tool_runtime.rs)"]
        Diagnostics["OS Diagnostics Suite (src/agent_core/diagnostics/)"]
        RepairEngine["Guided Repair Engine (src/agent_core/repair/)"]
        Orchestration["GSD Workflow Engine (src/agent_core/orchestration/)"]
        SessionDB["Session Store (rusqlite / src/session.rs)"]
        Sandbox["Security Sandbox & Policy (src/security/)"]
    end

    subgraph Engine & Inference Layer
        PyLauncher["Python Bootstrapper (start.py / scripts/)"]
        LocalEngine["Local llama-server / koboldcpp Engine"]
    end

    WebUI -->|HTTP / SSE| Server
    TUI --> TuiHandler
    Server --> AgentCore
    TuiHandler --> AgentCore

    AgentCore --> ToolRuntime
    ToolRuntime -->|v1/chat/completions| LocalEngine

    AgentCore --> Guardian
    Guardian --> Sandbox
    Sandbox -->|Approved Execution| HostSystem["Host Operating System"]
    Sandbox -->|Containerized Sandbox| DockerSystem["Docker Container Engine"]

    AgentCore --> Diagnostics
    AgentCore --> RepairEngine
    AgentCore --> Orchestration
    AgentCore --> SessionDB

    PyLauncher -->|Spawn Process| LocalEngine
    PyLauncher -->|Handover| TUI
    PyLauncher -->|Handover| Server
```

## Data & Decision Control Flow

1. **Bootstrap & Model Selection (`start.py`):**
   - Discovers `.gguf` models in `models/` or resolves environment-specified models.
   - Cleans orphaned GPU processes.
   - Spawns `scripts/start_server.py` which launches `llama-server` on `127.0.0.1:8080`.
   - Waits for `/v1/models` health check readiness.
   - Hands over control to `agent-rs` in either TUI or Web mode.

2. **User Request Processing (`agent-rs/src/main.rs` & `agent_core/mod.rs`):**
   - User submits a prompt through Ratatui TUI or Web UI (via Axum SSE endpoint).
   - `AgentCore` appends the prompt to the active session state stored in `SessionDB`.
   - Prompt context, tool schema definitions, system instructions, and diagnostic logs are formatted by `ToolRuntime`.

3. **Model Inference & Tool Decision:**
   - `ToolRuntime` sends an HTTP request to `http://127.0.0.1:8080/v1/chat/completions`.
   - Local LLM responds with text content or structured tool call requests (JSON schema matched).

4. **Security Guardian & Sandbox Gatekeeping (`agent-rs/src/security/` & `guardian.rs`):**
   - Tool calls pass through `Guardian`.
   - Dangerous operations (file modification, system service control, command execution) check path security (`path-security`) and policy configurations (`policy.rs`).
   - If human confirmation is required, `Guardian` halts execution and sends an approval event to the UI (TUI dialog or Web modal).
   - Upon approval, tools run either natively with canonicalized path enforcement or inside a Docker sandbox container (`sandbox.rs`).

5. **State Update & Result Delivery:**
   - Tool outputs (stdout, stderr, snapshot diffs) are fed back to `AgentCore`.
   - `SessionDB` persists the interaction step.
   - Response streams real-time to TUI or Web UI via Server-Sent Events.

## Core Module Responsibilities

| Subsystem | Directory / Key Files | Responsibilities |
| --- | --- | --- |
| **Agent Core** | [agent-rs/src/agent_core/mod.rs](file:///home/omprakash/helix-cli/agent-rs/src/agent_core/mod.rs) | Coordinates LLM inference loop, state transitions, tool execution dispatch |
| **Security Sandbox** | [agent-rs/src/security/policy.rs](file:///home/omprakash/helix-cli/agent-rs/src/security/policy.rs) | Validates command safety, enforces allowed directory boundaries, handles container isolation |
| **Diagnostics Module** | [agent-rs/src/agent_core/diagnostics/](file:///home/omprakash/helix-cli/agent-rs/src/agent_core/diagnostics/) | System metrics (`system.rs`), Windows Event Log / Syslog parsing (`logs.rs`), diagnostic reasoning |
| **Repair & Recovery** | [agent-rs/src/agent_core/repair/](file:///home/omprakash/helix-cli/agent-rs/src/agent_core/repair/) | Manages filesystem snapshots (`snapshots.rs`), rollback mechanisms, fix scoring (`scoring.rs`) |
| **TUI Interface** | [agent-rs/src/tui/](file:///home/omprakash/helix-cli/agent-rs/src/tui/) | Terminal rendering, themes (`themes.rs`), user input handling, interactive approval modals |
| **Web Server** | [agent-rs/src/server.rs](file:///home/omprakash/helix-cli/agent-rs/src/server.rs) | REST endpoints, SSE stream broadcasting, frontend static file serving |
| **Session Persistence** | [agent-rs/src/session.rs](file:///home/omprakash/helix-cli/agent-rs/src/session.rs) | SQLite schema, conversation transcript logging, state checkpoint recovery |
