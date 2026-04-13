# Phase 25: Plugin SDK and IDE Bridge Foundation - Research

**Researched:** 2026-04-13
**Domain:** Extensibility contracts (plugin SDK), permission-aware registration, and local IDE bridge foundation
**Confidence:** HIGH (current integration points), MEDIUM (first bridge protocol scope)

## Summary

Phase 25 should implement extensibility by extending the existing tool-call pipeline, not by creating a second execution engine.

The repository already has the key foundation pieces:
- Tool contract surface and schema generation in `agent-rs/src/tools.rs` (`ToolCallArgs`, JSON schema-derived grammar flow).
- Central tool registration payload assembly in `agent-rs/src/main.rs` (`build_tools`).
- Policy-tiered authorization in `agent-rs/src/security/policy.rs` (`PermissionTier`, `evaluate_tool_call`).
- Shared web interaction bridge in `agent-rs/src/server.rs` (`/chat` SSE event stream with tool lifecycle events).
- Existing audit/event persistence that can be reused for plugin actions in `agent-rs/src/audit.rs`.

What is still missing for ENT-02 and IDE-01:
- No plugin registration contract (manifest, tool metadata, declared permissions).
- No runtime plugin registry that merges built-in tools with plugin tools.
- No sandbox boundary contract for plugin execution.
- No IDE-focused local API contract beyond chat streaming.

Primary recommendation:
1. Add a manifest-based Plugin SDK contract (JSON manifest + stdio command bridge) that maps plugin tools into the existing tool payload schema.
2. Introduce a plugin registry that validates manifests, merges tool metadata into `tools_payload`, and dispatches plugin calls via a single execution adapter.
3. Enforce permission declarations by mapping each plugin tool to existing `PermissionTier` checks before execution.
4. Add an IDE bridge API (local-only) for tool discovery, permission preview, and request lifecycle events, reusing current SSE patterns.

## Standard Stack

### Core (Use)
| Library / Module | Purpose | Why |
|---|---|---|
| Existing `serde` + `serde_json` | Plugin manifest parsing and tool schema payload serialization | Already used across runtime/tool pathways |
| Existing `schemars` | Manifest/tool input schema generation and validation alignment | Already used for built-in tool schema generation |
| Existing `axum` + SSE in `server.rs` | IDE bridge transport (streaming events + command status) | Proven in web `/chat` path |
| Existing `tokio` process/time primitives | Plugin subprocess execution with timeout and cancellation | Already used for async execution patterns |
| Existing `security/policy.rs` | Permission declaration enforcement per plugin tool | Prevents bypassing tier model |
| Existing `audit.rs` | Log plugin registration + execution events | Gives traceability from day one |

### Optional (Only if needed)
| Library | Use case |
|---|---|
| `jsonschema` (Rust crate) | Strict runtime manifest JSON Schema validation if serde-only validation is insufficient |
| `uuid` | Stable plugin registration/request IDs if existing IDs are not reused |

## Architecture Patterns

### Pattern 1: Manifest-First Plugin Contract (ENT-02)

**What:** Use a static manifest per plugin as the source of truth.

Recommended manifest fields:
- `plugin_id` (unique, stable)
- `version`
- `entrypoint` (command path)
- `tools[]` with:
  - `name`
  - `description`
  - `input_schema` (JSON schema)
  - `permission_tier` (`read_only`, `workspace_write`, `full_exec`)
  - `timeout_ms`

**Why:** Registration, permissions, and runtime dispatch stay deterministic and reviewable.

### Pattern 2: Registry Merge into Existing Tool Payload

**What:** Extend `build_tools` flow so plugin tools are appended to the built-in tool list.

Flow:
1. Build built-in tools as today.
2. Load approved plugin manifests from a deterministic directory.
3. Convert each manifest tool to OpenAI function-tool shape.
4. Merge and regenerate grammar from final tool list.

**Why:** Keeps one model-facing contract path and preserves grammar enforcement behavior.

### Pattern 3: Permission-Tier Mapping as Hard Gate

**What:** Every plugin tool call must pass both:
1. global runtime permission tier gate, and
2. plugin-declared minimum tier.

Decision matrix:
- Runtime tier lower than plugin requirement -> deny with structured remediation.
- Runtime tier equal/higher -> proceed to execution adapter.

**Why:** Prevents plugins from sidestepping existing safety model.

### Pattern 4: Stdio Adapter Boundary for Plugin Execution

**What:** Execute plugins via a constrained subprocess protocol over stdin/stdout JSON.

Recommended adapter behavior:
- Send JSON request `{tool, args, request_id}`.
- Capture JSON response `{ok, output, error?}`.
- Enforce timeout and output size caps.
- Normalize failures into existing `ToolResult` shape.

**Why:** Simple cross-language SDK path without unsafe dynamic linking.

### Pattern 5: IDE Bridge as Local API Facade (IDE-01)

**What:** Add bridge endpoints that expose capabilities and execution state to editor extensions.

Recommended initial endpoints:
- `GET /bridge/tools` -> tool catalog (built-in + plugin + required tier)
- `POST /bridge/execute` -> submit tool call request envelope
- `GET /bridge/events` (SSE) -> lifecycle events (`queued`, `policy`, `started`, `completed`, `failed`)

**Why:** IDE clients need deterministic contracts and low-latency status, not direct internal coupling.

### Pattern 6: Unified Event/Audit Emission

**What:** Emit plugin events through same audit and status channels used by built-in tools.

Minimum events:
- registration accepted/rejected
- policy allow/deny for plugin call
- execution success/failure with duration

**Why:** Keeps enterprise visibility consistent across built-in and plugin surfaces.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Plugin loading | in-process dynamic library loader (`dlopen`/`libloading`) | subprocess stdio adapter | Isolation, language-agnostic SDK, easier crash containment |
| Permission model | plugin-specific custom auth logic | existing `PermissionTier` + policy decision flow | Single source of truth for safety behavior |
| Bridge protocol | ad hoc extension-private socket protocol | local HTTP + SSE contract in existing `axum` server | Reuses proven transport and event style |
| Tool schema format | plugin-only special schema format | same function-tool schema shape used by built-in tools | Keeps model prompt/tool pipeline uniform |
| Plugin observability | separate plugin log format | existing audit store and event envelopes | Enterprise traceability and debugging parity |

## Common Pitfalls

### 1) Creating a second tool execution path for plugins
If plugin calls bypass the existing policy and audit pipeline, ENT-02 is functionally incomplete.

### 2) Allowing plugin tools without explicit tier declaration
Implicit permissions create hidden escalation risk. Require manifest-declared tier for every tool.

### 3) Treating IDE bridge as a UI-only wrapper
IDE usage needs capability discovery and lifecycle events, not just chat passthrough.

### 4) Letting plugin schemas drift from built-in tool shape
Different schemas force branch logic in grammar generation and increase failure rate on small models.

### 5) Missing timeout/output caps for plugin subprocesses
Unbounded plugin execution can degrade orchestrator responsiveness and memory safety.

### 6) Not auditing registration and policy denials
Compliance and debugging require rejected actions as much as successful ones.

## Code Examples

### Example 1: Plugin manifest contract

```rust
#[derive(Debug, Clone, serde::Deserialize)]
struct PluginManifest {
    plugin_id: String,
    version: String,
    entrypoint: String,
    tools: Vec<PluginToolSpec>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PluginToolSpec {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    permission_tier: String, // read_only | workspace_write | full_exec
    timeout_ms: Option<u64>,
}
```

### Example 2: Merge plugin tools into model payload

```rust
fn merge_plugin_tools(mut builtin: Vec<serde_json::Value>, plugins: &[PluginManifest]) -> Vec<serde_json::Value> {
    for plugin in plugins {
        for tool in &plugin.tools {
            builtin.push(serde_json::json!({
                "type": "function",
                "function": {
                    "name": format!("plugin__{}__{}", plugin.plugin_id, tool.name),
                    "description": tool.description,
                    "strict": true,
                    "parameters": tool.input_schema,
                }
            }));
        }
    }
    builtin
}
```

### Example 3: Permission gate for plugin tool call

```rust
fn plugin_tool_allowed(runtime_tier: PermissionTier, required: PermissionTier) -> bool {
    use PermissionTier::*;
    matches!((runtime_tier, required),
        (FullExec, _) |
        (WorkspaceWrite, ReadOnly | WorkspaceWrite) |
        (ReadOnly, ReadOnly)
    )
}
```

### Example 4: IDE bridge event envelope

```rust
#[derive(serde::Serialize)]
struct BridgeEvent {
    event_type: String,   // queued | policy | started | completed | failed
    request_id: String,
    tool_name: String,
    status: String,
    detail: String,
}
```

### Example 5: Plugin subprocess adapter skeleton

```rust
async fn run_plugin_tool(
    entrypoint: &str,
    tool_name: &str,
    args: serde_json::Value,
    timeout_ms: u64,
) -> Result<String, String> {
    // Spawn constrained subprocess, write JSON request on stdin,
    // read JSON response from stdout, enforce timeout.
    // Normalize to ToolResult-compatible output.
    unimplemented!()
}
```

## Source Evidence

- `agent-rs/src/tools.rs` defines core tool argument models and grammar generation from the tools payload.
- `agent-rs/src/main.rs` centralizes model-facing tool payload creation via `build_tools`.
- `agent-rs/src/security/policy.rs` already provides permission-tier gates and policy decision contract.
- `agent-rs/src/server.rs` already exposes SSE-based lifecycle events and tool execution flow over `/chat`.
- `agent-rs/src/audit.rs` already supports append-only structured event storage with hash chaining.
- `web-ui/src/App.tsx` consumes SSE event types (`text`, `tool_start`, `tool_result`, `system`, `error`), showing an existing bridge event pattern usable by IDE clients.

## Implementation Readiness

Ready for `/gsd-plan-phase 25`.

Recommended planning order:
1. Define Plugin SDK contracts (manifest schema, naming rules, permission declaration model).
2. Implement plugin registry and merge plugin tools into unified `tools_payload` + grammar pipeline.
3. Add plugin execution adapter with tier enforcement, timeout/output limits, and audit hooks.
4. Add local IDE bridge endpoints for capability discovery and execution/event streaming.
5. Add tests for manifest validation, permission gating, dispatch correctness, and bridge event contract stability.
