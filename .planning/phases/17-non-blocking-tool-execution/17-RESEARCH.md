# Phase 17: Non-Blocking Tool Execution - Research

**Researched:** 2026-04-06
**Domain:** Rust async concurrency, tokio task management, LLM orchestrator
**Confidence:** HIGH

## Summary

This phase converts synchronous tool execution (blocking the orchestrator loop) to non-blocking async execution with parallel support. The codebase already has tokio and futures-util as dependencies. The TUI already emits `ToolStart` events during execution, providing the UI feedback hook. The main work is wrapping existing `tools::*` functions in async tasks, adding timeout enforcement, and running multiple tool calls concurrently.

**Primary recommendation:** Use `tokio::task::spawn_blocking` to wrap the sync `tools::execute_*` functions, `tokio::time::timeout` for per-tool 30s limits, and `futures::future::join_all` for parallel execution. Results are injected as `ChatMessage` with role "tool" (already supported).

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.43.0 | Async runtime with spawn, timeout, select | Already in Cargo.toml with "full" feature |
| futures-util | 0.3 | join_all for concurrent task collection | Already in Cargo.toml |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::time | (via tokio) | Duration, timeout for per-tool limits | Required for TOOL-05 (30s timeout) |
| tokio::task | (via tokio) | spawn_blocking for sync tool functions | Wrapping CPU-bound tool execution |

**Installation:** No new dependencies required. Both tokio (with "full") and futures-util are already present.

## Architecture Patterns

### Recommended Project Structure

The tool execution logic lives in `main.rs` inside the LLM loop functions:
- `run_llm_loop_tui()` handles TUI mode tool execution (lines ~1805-1904)
- Terminal mode tool execution (lines ~1263-1365)

**Changes needed within existing files:**
- `agent-rs/src/main.rs`: Modify tool execution block to spawn async tasks
- `agent-rs/src/tools.rs`: Wrap execute_* functions in async-compatible wrappers if needed

### Pattern 1: Async Tool Spawn with Timeout

**What:** Wrap each tool execution in a tokio task with timeout enforcement

**When to use:** For every individual tool call requiring non-blocking execution (TOOL-01, TOOL-05)

**Example:**
```rust
use tokio::time::{timeout, Duration};
use tokio::task::spawn_blocking;

// Execute a single tool with 30s timeout
async fn execute_tool_with_timeout<F, T>(tool_name: String, f: F) -> ToolResult
where
    F: FnOnce() -> T + Send + 'static,
    T: Into<ToolResult>,
{
    let result = timeout(
        Duration::from_secs(30),
        spawn_blocking(f)
    ).await;

    match result {
        Ok(Ok(result)) => result.into(),
        Ok(Err(_)) => ToolResult {
            success: false,
            output: format!("Tool '{}' panicked or was cancelled", tool_name),
        },
        Err(_) => ToolResult {
            success: false,
            output: format!("Tool '{}' timed out after 30 seconds", tool_name),
        },
    }
}
```

**Source:** Tokio documentation on spawn_blocking and time::timeout - https://tokio.rs/tokio/tutorial/select, https://docs.rs/tokio/latest/tokio/time/fn.timeout.html

### Pattern 2: Parallel Tool Execution with join_all

**What:** Execute multiple independent tool calls concurrently using futures-util

**When to use:** When the LLM returns multiple tool calls in a single response (TOOL-04)

**Example:**
```rust
use futures::future::join_all;

// Execute multiple tools in parallel, collecting results
async fn execute_tools_parallel(tool_calls: Vec<ToolCall>) -> Vec<ToolResult> {
    let tasks: Vec<_> = tool_calls.into_iter()
        .map(|tc| async move {
            execute_tool_with_timeout(tc.name.clone(), || {
                execute_tool_sync(tc) // sync wrapper
            }).await
        })
        .collect();

    join_all(tasks).await
}
```

**Source:** https://redandgreen.co.uk/asynchronous-programming-in-rust-with-join_all/rust-programming/

### Pattern 3: Status Feedback via TUI Events

**What:** Send ToolStart event before spawning, ToolResult after completion

**When to use:** For tool status display in TUI (TOOL-02)

**Existing infrastructure:** The TUI already emits `TuiEvent::ToolStart(ToolInfo)` and `TuiEvent::ToolResult(ToolResultInfo)`. The current code sends these events synchronously - the pattern just needs to be preserved with async execution.

**Example:**
```rust
// Before spawning task
let _ = event_tx.send(tui::TuiEvent::ToolStart(tui::ToolInfo {
    name: func_name.clone(),
    arguments: parsed_args.to_string(),
}));

// After task completes
let _ = event_tx.send(tui::TuiEvent::ToolResult(tui::ToolResultInfo {
    name: func_name.clone(),
    output: tool_result.output.clone(),
    success: tool_result.success,
}));
```

### Pattern 4: Tool Result as ChatMessage

**What:** Inject completed tool results as ChatMessage with role "tool"

**When to use:** After each tool completes to feed results back to LLM (TOOL-03)

**Existing pattern:** Already implemented - see main.rs lines ~1896-1902:
```rust
messages.push(ChatMessage {
    role: "tool".to_string(),
    content: Some(tool_result.output),
    tool_calls: None,
    tool_call_id: Some(id),
    name: Some(func_name),
});
```

This pattern remains valid - just the tool_result comes from async execution now.

### Anti-Patterns to Avoid

- **Don't use tokio::spawn for CPU-bound work:** The tool execution functions (file I/O, process spawning, sysinfo) are CPU-bound. Use `spawn_blocking` to run on blocking thread pool, not the async executor.
- **Don't forget to handle timeout result unwrapping:** `timeout()` returns `Result<Result<T>, Elapsed>`. Must handle both the inner Result and the timeout case.
- **Don't execute tools sequentially when they could run parallel:** If multiple tool calls arrive in one LLM response, they should run concurrently via join_all, not sequentially.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async task spawning | Custom thread pool | tokio::spawn_blocking | Built-in blocking task pool, proper cancellation |
| Concurrent futures | Manual Vec + loop | futures::future::join_all | Handles all futures, returns all results, proven correct |
| Timeout enforcement | Custom timer | tokio::time::timeout | Integrated with tokio runtime, no extra threads |
| Tool result ordering | Track indices manually | Preserve tool call order from LLM | LLM provides deterministic ordering |

**Key insight:** The existing tools::execute_* functions are synchronous and CPU/IO-bound. Wrapping them in spawn_blocking is the standard pattern - no custom async wrappers needed.

## Common Pitfalls

### Pitfall 1: Blocking the Async Executor

**What goes wrong:** Using `tokio::spawn` (not spawn_blocking) for CPU-bound tool execution exhausts the async thread pool.

**Why it happens:** tokio's default executor has limited threads. Blocking calls inside async tasks prevent other async tasks from running.

**How to avoid:** Use `tokio::task::spawn_blocking` for all tool execution wrappers. The blocking pool has more threads and is designed for CPU/IO-bound work.

**Warning signs:** "too many concurrent operations" errors, stalled UI during tool execution.

### Pitfall 2: Dropping Task Results on Timeout

**What goes wrong:** Timeout elapses but the spawned task continues running (leak), or result is dropped without cleanup.

**Why it happens:** tokio::timeout returns Err(Elapsed) but doesn't cancel the inner future. The task continues running in background.

**How to avoid:** Currently no perfect solution without std::task::Abort API. Acceptable: log timeout, return timeout error, let task leak (low impact for tool execution). Alternative: use select! with a cancellation flag but adds complexity.

**Warning signs:** Tools continue running after timeout, resources not released.

### Pitfall 3: Ordering Mismatch Between Tool Calls and Results

**What goes wrong:** Parallel execution completes in random order, but LLM expects results in specific sequence.

**Why it happens:** join_all returns results in task order, but if tasks complete at different times, the Vec ordering may not match original tool_calls ordering.

**How to avoid:** Map results back to original tool call indices. The current code already sorts indices: `indices.sort()` before pushing to final_tool_calls. Ensure this pattern is preserved.

**Warning signs:** LLM gets wrong tool result for a specific tool_call_id.

### Pitfall 4: Missing ToolResult Structure Conversion

**What goes wrong:** The sync tools::execute_* functions return tools::ToolResult, but async wrapper needs to return this across task boundary.

**Why it happens:** tools::ToolResult is a custom struct (not Send by default unless configured).

**How to avoid:** Ensure tools::ToolResult is marked `#[derive(Clone, Debug)]` - it already is. spawn_blocking requires the closure to be `FnOnce() -> T + Send + 'static` - ToolResult satisfies this.

## Code Examples

### Complete Async Tool Execution Pattern

```rust
// Wrapper to make sync tool execution async-compatible
fn execute_tool_sync(tc: &Value) -> ToolResult {
    // Parse tool call, call appropriate tools::execute_* function
    // (same logic as current sync code)
}

// Execute single tool with timeout
async fn spawn_tool_execution(
    tc: Value,
    app_config: &config::AppConfig,
) -> (String, ToolResult, Value) {
    let id = tc["id"].as_str().unwrap_or("").to_string();
    let func_name = tc["function"]["name"].as_str().unwrap_or("").to_string();
    let args_value = &tc["function"]["arguments"];
    let parsed_args = /* parse args */;

    let tool_result = timeout(
        Duration::from_secs(30),
        spawn_blocking(move || {
            execute_tool_sync(&tc)
        })
    ).await.unwrap_or_else(|_| ToolResult {
        success: false,
        output: format!("Tool '{}' timed out after 30 seconds", func_name),
    });

    (id, tool_result, tc)  // Return all needed data
}

// Execute multiple tools in parallel
async fn execute_all_tools(
    tool_calls: Vec<Value>,
    app_config: &config::AppConfig,
) -> Vec<(String, ToolResult, Value)> {
    let tasks: Vec<_> = tool_calls.into_iter()
        .map(|tc| spawn_tool_execution(tc, app_config))
        .collect();
    
    join_all(tasks).await
}
```

**Source:** Based on existing main.rs tool execution logic + tokio spawn_blocking + futures join_all patterns

### TUI Integration Pattern

```rust
// Emit ToolStart before spawning (TOOL-02)
let _ = event_tx.send(tui::TuiEvent::ToolStart(tui::ToolInfo {
    name: func_name.clone(),
    arguments: parsed_args.to_string(),
}));

// Execute async with timeout
let tool_result = /* async execution */;

// Emit ToolResult after completion (TOOL-03)
let _ = event_tx.send(tui::TuiEvent::ToolResult(tui::ToolResultInfo {
    name: func_name.clone(),
    output: tool_result.output.clone(),
    success: tool_result.success,
}));

// Inject into message history
messages.push(ChatMessage {
    role: "tool".to_string(),
    content: Some(tool_result.output),
    tool_call_id: Some(id),
    name: Some(func_name),
    ..Default::default()
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Sequential sync tool execution | Non-blocking async with spawn_blocking | This phase | LLM loop doesn't block on tool execution |
| One tool at a time | Parallel execution via join_all | This phase | Multiple tools run concurrently |
| No timeout | 30s timeout per tool via tokio::time::timeout | This phase | Prevents hung tool execution |
| Blocking terminal loop | Async TUI event loop | Phase 16 | Already non-blocking |

**Deprecated/outdated:**
- Synchronous `for tc in tool_calls` loop - replaced with async parallel execution

## Open Questions

1. **Should timeout kill the underlying process?**
   - Current approach: timeout returns error, process continues running
   - Alternative: use process::Child::kill on timeout (complex, not critical)
   - Recommendation: Accept leaked task for v1.2, improve in future if needed

2. **How to handle mixed sync/async tool execution?**
   - Some tools may have async implementations in future
   - Current: all tools wrapped in spawn_blocking
   - Recommendation: Keep as-is, refactor when needed

3. **Should tool failures in parallel execution stop the chain?**
   - Current: all tools execute, failures become tool_result.success=false
   - Critic injections handle failure retry logic
   - Recommendation: Continue parallel execution, let critic handle failures

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| tokio | Async runtime | ✓ (in Cargo.toml) | 1.43.0 | — |
| futures-util | join_all | ✓ (in Cargo.toml) | 0.3 | — |

**Missing dependencies with no fallback:**
- None - all required libraries already present

**Missing dependencies with fallback:**
- None

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (tokio + futures) |
| Config file | None - test in-line |
| Quick run command | `cargo test --package agent-rs` |
| Full suite command | `cargo test --package agent-rs --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TOOL-01 | Async task spawn without blocking | Unit - verify spawn returns immediately | `cargo test --package agent-rs tool_execution` | No - needs creation |
| TOOL-02 | Tool status "running..." displayed | Integration - TUI event received | Manual test | Existing |
| TOOL-03 | ToolResult injected as ChatMessage | Unit - verify message added | `cargo test --package agent-rs` | Partial |
| TOOL-04 | Multiple tools executed in parallel | Integration - verify concurrent | Manual test | No |
| TOOL-05 | 30s timeout enforced | Unit - timeout test | `cargo test --package agent-rs timeout` | No |

### Sampling Rate
- **Per task commit:** `cargo test --package agent-rs --lib`
- **Per wave merge:** `cargo test --package agent-rs --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/tool_execution.rs` — covers TOOL-01, TOOL-04, TOOL-05
- [ ] `tests/tui_events.rs` — covers TOOL-02
- [ ] Integration test for parallel execution timing

## Sources

### Primary (HIGH confidence)
- https://tokio.rs/tokio/tutorial/select - Tokio select! and task spawning
- https://docs.rs/tokio/latest/tokio/time/fn.timeout.html - tokio::time::timeout
- https://docs.rs/tokio/1.46.1/tokio/macro.select.html - select! macro
- https://redandgreen.co.uk/asynchronous-programming-in-rust-with-join_all/ - join_all pattern
- agent-rs/src/main.rs - existing tool execution logic (lines ~1805-1904)
- agent-rs/src/tui.rs - TuiEvent::ToolStart, ToolResult events

### Secondary (MEDIUM confidence)
- https://users.rust-lang.org/t/need-help-understanding-tokio-timeout-mechanics/ - timeout edge cases
- https://stackoverflow.com/questions/63589668/how-to-tokiojoin-multiple-tasks - join patterns

### Tertiary (LOW confidence)
- General async patterns - verified against tokio docs

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH - tokio/futures already present, no new dependencies
- Architecture: HIGH - clear patterns from existing code + tokio best practices
- Pitfalls: MEDIUM - known tokio patterns, but edge cases (timeout leak) not fully tested

**Research date:** 2026-04-06
**Valid until:** 2026-05-06 (30 days - stable async patterns)