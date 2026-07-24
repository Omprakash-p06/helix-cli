# Repository Structure

**Last Updated:** 2026-07-24

## Directory Layout Overview

```
helix-cli/
├── .planning/               # GSD (Get Stuff Done) workflow specifications and tracking
│   ├── codebase/            # Codebase mapping documentation (7 standard docs)
│   ├── debug/               # Knowledge base and resolved bug investigation logs
│   ├── phases/              # Phase planning documents, research, and validation records
│   ├── PROJECT.md           # Project vision, milestone roadmap, and goals
│   ├── REQUIREMENTS.md      # Core project functional & security requirements
│   ├── ROADMAP.md           # Project milestone progress tracker
│   └── STATE.md             # Active development state and context
├── agent-rs/                # High-performance Rust Agent Engine
│   ├── src/                 # Rust source code
│   │   ├── agent_core/      # Core agent logic, tool runtime, diagnostics, repair
│   │   │   ├── diagnostics/ # System inspection, logs, metrics, reasoning
│   │   │   ├── orchestration/ # Workflow state, context reset, artifacts
│   │   │   ├── repair/      # Fix scoring, file snapshots, repair execution
│   │   │   ├── guardian.rs  # Tool security wrapper
│   │   │   └── tool_runtime.rs # LLM tool call parser & prompt generator
│   │   ├── security/        # Security sandbox, path security, execution policy
│   │   ├── tui/             # Ratatui TUI implementation (state, events, rendering)
│   │   ├── audit.rs         # Execution & decision audit log writer
│   │   ├── config.rs        # Runtime configuration settings
│   │   ├── input.rs         # User input handling & readline bindings
│   │   ├── lib.rs           # Library exports
│   │   ├── main.rs          # Binary entry point (`helix-agent`)
│   │   ├── rag.rs           # Local RAG & vector embedding logic
│   │   ├── runtime_profile.rs # Adaptive performance & memory profiles
│   │   ├── server.rs        # Axum REST & SSE API server for Web UI
│   │   ├── session.rs       # SQLite conversation & session store
│   │   ├── stream.rs        # SSE streaming handlers
│   │   ├── tools.rs         # Built-in tool definitions (shell, file, inspect)
│   │   ├── types.rs         # Shared type definitions & data structures
│   │   └── watchdog.rs      # Process & health monitor
│   ├── tests/               # Rust integration test suite
│   └── Cargo.toml           # Rust package manifest & dependencies
├── scripts/                 # Python engine management & helper utilities
│   ├── build_zip.py         # Distribution packaging script
│   ├── config.py            # Global environment & port configuration
│   ├── download_model.py    # Hugging Face model fetcher
│   ├── helix.py             # CLI runner wrapper
│   ├── helix_branding.py    # Terminal logo & styling
│   ├── model_install.py     # Model discovery & installation helper
│   ├── onboarding_profile.py# User preference persistence (`.helix_profile.json`)
│   ├── start_agent.sh       # Shell launcher for Rust agent
│   ├── start_server.py      # LLM inference server bootstrap (`llama-server`)
│   ├── start_server.sh      # Shell wrapper for server startup
│   └── system_check.py      # Hardware & VRAM diagnostic tool
├── tests/                   # Python integration & accuracy tests
│   ├── test_accuracy.py     # Diagnostic accuracy evaluation
│   ├── test_model_install.py# Model installation unit tests
│   ├── test_onboarding_profile.py # Onboarding profile tests
│   └── test_system_check.py # Hardware check unit tests
├── web-ui/                  # Modern Web Interface (React 19 / Vite / Tailwind)
│   ├── src/
│   │   ├── assets/          # Images & icons
│   │   ├── App.css          # App-specific styles
│   │   ├── App.tsx          # Main React web dashboard view
│   │   ├── index.css        # Tailwind directives & CSS variable tokens
│   │   └── main.tsx         # React app DOM entry point
│   ├── package.json         # Node.js dependencies & scripts
│   ├── tailwind.config.js   # Tailwind layout tokens & theme extension
│   └── vite.config.ts       # Vite build configuration
├── models/                  # Local GGUF models storage directory (gitignored)
├── start.py                 # Main Python entry point & interactive setup wizard
├── README.md                # Project README & user documentation
└── Cargo.toml / Cargo.lock  # Root workspace configuration
```

## Key File Locations & Entry Points

| Category | Path | Description |
| --- | --- | --- |
| **Main Python Launcher** | [start.py](file:///home/omprakash/helix-cli/start.py) | Interactive startup script, handles model selection, server boot, interface selection |
| **Rust Agent Entry Point** | [agent-rs/src/main.rs](file:///home/omprakash/helix-cli/agent-rs/src/main.rs) | Initializes Rust engine, boots TUI or Axum web server based on flags |
| **LLM Server Bootstrapper** | [scripts/start_server.py](file:///home/omprakash/helix-cli/scripts/start_server.py) | Manages `llama-server` process execution, logging, and port configuration |
| **Security Policy Engine** | [agent-rs/src/security/policy.rs](file:///home/omprakash/helix-cli/agent-rs/src/security/policy.rs) | Defines security rules, path canonicalization, command blocklists |
| **TUI Application Engine** | [agent-rs/src/tui/events.rs](file:///home/omprakash/helix-cli/agent-rs/src/tui/events.rs) | Handles keyboard events, render loop, UI state transitions |
| **Web UI Dashboard** | [web-ui/src/App.tsx](file:///home/omprakash/helix-cli/web-ui/src/App.tsx) | Main React client application interface |
| **SQLite Session Store** | [agent-rs/src/session.rs](file:///home/omprakash/helix-cli/agent-rs/src/session.rs) | Database schema management for sessions and audit history |
