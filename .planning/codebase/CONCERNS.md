# Technical Debt & Known Concerns

**Last Updated:** 2026-07-24

## Known Technical Debt & Areas of Concern

### 1. Process Lifecycle & Port Binding Conflicts
- **Issue:** The local LLM inference engine (`llama-server` / `koboldcpp`) defaults to port `8080`.
- **Impact:** If another local service occupies port 8080 or if a previous instance did not cleanly unbind the HTTP socket, server startup fails.
- **Mitigation / Location:** `start.py` implements a `clean_orphaned_servers()` routine prior to boot, but port conflicts still occur if non-Helix services occupy the port. Managed via [scripts/config.py](file:///home/omprakash/helix-cli/scripts/config.py) and [start.py](file:///home/omprakash/helix-cli/start.py#L214-L227).

### 2. Local LLM Context Window & VRAM Overhead
- **Issue:** Large language model GGUF quantizations (e.g. 7B/14B parameters) with extended context windows (16k+ tokens) require high VRAM or CPU RAM offloading.
- **Impact:** On systems with constrained GPU VRAM (< 8GB), high context prompts during deep troubleshooting sessions can slow inference speed or trigger out-of-memory errors.
- **Mitigation / Location:** Evaluated by [scripts/system_check.py](file:///home/omprakash/helix-cli/scripts/system_check.py) and managed by context window reset logic in [agent-rs/src/agent_core/orchestration/context_reset.rs](file:///home/omprakash/helix-cli/agent-rs/src/agent_core/orchestration/context_reset.rs).

### 3. Dual UI Interface Synchronization (TUI vs Web UI)
- **Issue:** Helix CLI supports both a terminal TUI (Ratatui + Crossterm) and a web app dashboard (React + Axum SSE server).
- **Impact:** Feature additions (such as new tool approval dialogs, streaming message controls, or status indicators) must be implemented twice — once for the terminal event loop in [agent-rs/src/tui/events.rs](file:///home/omprakash/helix-cli/agent-rs/src/tui/events.rs) and once for the REST/SSE API in [agent-rs/src/server.rs](file:///home/omprakash/helix-cli/agent-rs/src/server.rs) / React UI.

### 4. Cross-Platform Diagnostic Abstraction Differences
- **Issue:** OS-level troubleshooting features require different underlying APIs depending on the platform (Linux systemd/journalctl vs. Windows Event Logs/Win32 services).
- **Impact:** Platform-specific code branches exist in [agent-rs/src/agent_core/diagnostics/system.rs](file:///home/omprakash/helix-cli/agent-rs/src/agent_core/diagnostics/system.rs) and [agent-rs/src/agent_core/diagnostics/logs.rs](file:///home/omprakash/helix-cli/agent-rs/src/agent_core/diagnostics/logs.rs). Operating system edge cases (e.g. permission limits when reading system logs without root/admin privileges) must be handled gracefully.

### 5. Docker Sandbox Daemon Availability
- **Issue:** High-security sandbox mode relies on Docker daemon availability via `bollard` client.
- **Impact:** If Docker daemon is unavailable, stopped, or the user lacks socket permission (`/var/run/docker.sock`), containerized execution falls back to path-sanitized host execution, emitting a warning.
- **Mitigation / Location:** Handled in [agent-rs/src/security/sandbox.rs](file:///home/omprakash/helix-cli/agent-rs/src/security/sandbox.rs).

## Fragile Areas & Key Security Considerations

- **Path Traversal & Soft Canonicalization:** File operations depend on `soft-canonicalize` and `path-security` in [agent-rs/src/security/policy.rs](file:///home/omprakash/helix-cli/agent-rs/src/security/policy.rs) to ensure tools cannot write outside approved workspace boundaries.
- **Tool Repetitive Calling / Loop Prevention:** LLM tool calling loops are guarded by `watchdog.rs` and tool execution scoring limits to prevent runaway shell commands when local models get stuck in repetition loops.
