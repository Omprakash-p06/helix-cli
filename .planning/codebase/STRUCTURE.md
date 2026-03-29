# STRUCTURE.md

## Snapshot
Last refreshed: 2026-03-29
Repository contains product code plus a large vendored/embedded `llama.cpp` tree.

## Top-Level Layout
- `agent-rs/`: Rust orchestrator and APIs
- `scripts/`: Python operational scripts (server start, checks, branding)
- `web-ui/`: React + Vite frontend
- `tests/`: Python evaluation scripts and dataset
- `.planning/`: roadmap, phase plans, research, codebase docs
- `llama.cpp/`: large upstream inference backend source tree
- Root scripts: `start.py`, `setup.py`, `update_milestone.py`

## Rust App Structure (`agent-rs/src`)
- `main.rs`: top-level orchestration loop and mode routing
- `server.rs`: Axum endpoints and SSE stream handling
- `tui.rs`: terminal UI state/event system
- `tools.rs`: tool schemas + execution with sandbox enforcement
- `config.rs`: Python bridge for runtime config loading
- `types.rs`: chat and protocol data structures
- `stream.rs`: SSE parser utilities
- `tokens.rs`, `utils.rs`, `input.rs`, `rag.rs`: support modules

## Web App Structure (`web-ui/src`)
- `App.tsx`: single-page chat UI and SSE client logic
- `main.tsx`: React entry point
- Styling in `App.css` and `index.css`
- Static assets under `web-ui/public` and `web-ui/src/assets`

## Planning and Process Files
- `.planning/PROJECT.md`: project vision and milestones
- `.planning/ROADMAP.md`: phase requirements and plan listings
- `.planning/STATE.md`: current execution state and rules
- `.planning/phases/*`: context, plans, summaries, validation docs
- `.planning/codebase/*`: generated codebase map documents

## Notable Scale Consideration
- `llama.cpp/` dominates file count and can drown code search results.
- Most Helix-specific product changes happen outside `llama.cpp/`.
