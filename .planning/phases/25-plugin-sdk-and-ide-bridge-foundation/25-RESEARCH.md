# Phase 25 Research: Plugin SDK and IDE Bridge Foundation

## Domain Analysis: Agent Extensibility

Helix currently uses a hardcoded `match` block in `main.rs` to dispatch tool calls to functions in `tools.rs`. To satisfy **ENT-02** (Plugin SDK) and **IDE-01** (IDE Bridge), we need a decoupled architecture that allows:
1.  **Dynamic Tool Registration**: Adding new capabilities without recompiling the orchestrator.
2.  **Standardized API Contract**: A predictable interface for IDE extensions (VS Code, JetBrains) to communicate with the local agent.
3.  **Security Boundaries**: Ensuring plugins respect the configured `PermissionTier`.

### 1. Plugin SDK Patterns (ENT-02)

#### Option A: Wasm-based Plugins
- **Pros**: Strong sandboxing, cross-language.
- **Cons**: High complexity for initial MVP, overhead in passing complex state.

#### Option B: External Process (MCP-like)
- **Pros**: Language agnostic, simple protocol (JSON-RPC over stdio/HTTP), isolated memory.
- **Cons**: Requires managing child processes.

#### Option C: Built-in Registry (Phase 25 Focus)
- **Refactor**: Move from `ToolCallArgs` enum matching to a `ToolRegistry` that maps tool names to trait objects or async function pointers.
- **SDK**: Provide a set of macros and traits in a dedicated crate (`helix-plugin-sdk`) that allows developers to define tools with auto-generated JSON schemas.

### 2. IDE Bridge Foundation (IDE-01)

#### Local API Contract
The existing `server.rs` provides a `/chat` endpoint. For IDE usage, we need specialized endpoints:
-   `GET /tools`: List active tools and their schemas.
-   `POST /execute`: Manually trigger a tool call (for IDE-side actions).
-   `GET /context`: Fetch current project context (files, git status).

#### Extension Bridge Spec
-   **Discovery**: IDE extensions look for a `.helix/port` file or use a fixed port (e.g., 3000) to find the orchestrator.
-   **Events**: Use SSE (already implemented in `server.rs`) to stream token-by-token updates to the IDE sidebar.

## Gap Closure: Security (SEC-01..03)
Plugins must declare their required `PermissionTier`. The `AuditStore` (Phase 24) will record which plugin/tool was invoked, providing enterprise-grade traceability.

## Proposed Technical Stack
-   **SDK**: Rust Trait-based registration.
-   **Protocol**: JSON-RPC 2.0 for external plugin communication.
-   **API**: Axum (existing) with additional REST/SSE routes.

## Success Criteria (Initial)
-   [ ] `ToolRegistry` replaces hardcoded match blocks.
-   [ ] A "Hello World" external plugin can be registered via config.
-   [ ] A `/v1/status` API endpoint exists for IDE health checks.
