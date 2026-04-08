# Phase 17: Non-Blocking Tool Execution - Context

**Gathered:** 2026-04-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement async tool spawning, concurrent execution, and status feedback without blocking the orchestrator loop. This converts synchronous tool execution (blocking the orchestrator loop) to non-blocking async execution with parallel support.

</domain>

<decisions>
## Implementation Decisions

### Status Display (TOOL-02)
- **D-01:** Tool status displayed inline in chat area (like Claude Code)
  - Works in both terminal and TUI modes
  - Users see progress naturally within conversation flow
  - Status bar is less discoverable and can be ignored

### Timeout Handling (TOOL-05)
- **D-02:** Return timeout error to LLM (let LLM decide retry/fallback)
  - Gives the LLM agency to handle errors intelligently (e.g., try different approach, ask user, or fall back)
  - Auto-retry may repeat the same failure and waste time
  - Prompting user breaks automation

### Parallel Failure Strategy (TOOL-04)
- **D-03:** Report all results to LLM (including failures) – let critic decide
  - Provides full information so LLM can make informed decision (proceed with partial results, retry failed tools, or ask for help)
  - Stopping at first failure may discard successful parallel work

### Result Ordering (TOOL-04)
- **D-04:** Original call order (preserves LLM expectations)
  - LLMs expect tool results in same order they were called
  - Mixing order can confuse reasoning and break dependencies
  - Correctness > speed for agent reliability

### the agent's Discretion
- TUI event timing (immediate vs debounced) — standard approach acceptable
- Specific logging for tool lifecycle events — standard debug logging acceptable

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Research
- `.planning/phases/17-non-blocking-tool-execution/17-RESEARCH.md` — Technical approach (spawn_blocking, timeout, join_all)

### Prior Phase Context
- `.planning/phases/15-fix-blank-ui-output-when-model-streams-tokens/15-CONTEXT.md` — Phase 17 deferred from Phase 15
- `.planning/phases/15-fix-blank-ui-output-when-model-streams-tokens/15-DISCUSSION-LOG.md` — "Non-blocking parallel tool execution is deferred to Phase 17"

### Project Requirements
- `.planning/REQUIREMENTS.md` — TOOL-01 through TOOL-05 requirements
- `.planning/ROADMAP.md` — Phase 17 goal and success criteria

### Codebase
- `agent-rs/src/main.rs` — Existing tool execution logic (lines ~1805-1904)
- `agent-rs/src/tools.rs` — Sync tool execution functions
- `agent-rs/src/tui.rs` — TuiEvent::ToolStart, ToolResult events

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tokio` (1.43.0 with "full" feature) — Already in Cargo.toml, used for async runtime
- `futures-util` (0.3) — Already in Cargo.toml, used for join_all
- `TuiEvent::ToolStart` and `TuiEvent::ToolResult` — Already implemented in tui.rs, just need to emit with async execution

### Established Patterns
- ChatMessage with role "tool" already implemented at main.rs lines ~1896-1902
- tools::execute_* functions are synchronous and CPU/IO-bound — wrap in spawn_blocking
- Existing indices sorting pattern for result ordering

### Integration Points
- Tool execution block in `run_llm_loop_tui()` (lines ~1805-1904)
- Terminal mode tool execution (lines ~1263-1365)
- Config::AppConfig for timeout settings

</code_context>

<specifics>
## Specific Ideas

- Use `tokio::task::spawn_blocking` to wrap sync tool execution functions
- Use `tokio::time::timeout` for per-tool 30s limits
- Use `futures::future::join_all` for parallel execution
- Preserve TUI events (ToolStart before spawn, ToolResult after completion)
- Keep ChatMessage injection pattern for tool results

No additional specifics required — standard approaches from research acceptable.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 17-non-blocking-tool-execution*
*Context gathered: 2026-04-06*