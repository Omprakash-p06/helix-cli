# Research: Stack Findings for v1.2

## New Dependencies Required

### Streaming & Buffering
- **`tokio-util`** — BufReader traits and async I/O adapters (already available, extend usage)
- **`bytes` crate** — Efficient byte slicing and pooling for streaming chunks (already available)
- **`futures-util`** — Combinator functions for stream processing (already available, extend for StreamExt)

### Chat Mode Output Filtering
- **`regex` crate** — Pattern matching for stripping `<think>` blocks (lightweight, standard)
- **`nom` or manual parsing** — Structured token filtering (already have serde_json, use manual)

### Non-blocking Tool Execution
- **`tokio::task::spawn_blocking`** — Offload potentially blocking tool operations (already have tokio)
- **`dashmap`** — Concurrent tool state tracking (optional; `Arc<Mutex<HashMap>>` works for MVP)
- **`tracing` + `tracing-subscriber`** — Structured logging for tool lifecycle (recommended, not mandated)

### Async Coordination
- **`tokio::sync::broadcast`** — Tool result broadcast to UI (already have tokio)
- **`tokio::sync::watch`** — Tool status watcher for live UI updates (already have tokio)

---

## Current Stack Audit

| Component | Current Version | Needed? | Notes |
|-----------|-----------------|---------|-------|
| tokio | v1.x | ✓ Extend | Already core; expand task spawn patterns |
| axum | v0.7.x | ✓ Keep | SSE already uses; add tool routes |
| serde_json | latest | ✓ Keep | Already used for delta parsing |
| ratatui | latest | ✓ Extend | TUI layer; add tool status rendering |
| crossterm | latest | ✓ Keep | Terminal I/O |
| futures-util | v0.3.x | ✓ Add | For StreamExt trait methods |
| regex | v1.x | ⊕ Optional | For think-block stripping (3KB optional dep) |
| tracing | v0.1.x | ⊕ Optional | Better than println!; recommended but MVP works without |

---

## Integration Points

### Streaming Layer
- **Replace:** Line buffering in SSE parser → Raw byte-at-a-time consumption
- **Add:** Flush hooks after every N bytes or timer tick (30ms similar to v1.1)
- **Leverage:** tokio::io::{BufReader, take} for byte-precise reading

### Chat Mode Processing
- **Input:** model output stream (SSE chunks with delta)
- **Filter:** Extract visible text, strip `<think>...</think>` sections
- **Output:** Filtered tokens to UI (async channel)
- **Tool:** Custom filter function in main.rs (no new dep needed)

### Tool Execution
- **Spawn:** `tokio::task::spawn` for each tool call
- **Track:** Arc<Mutex<ToolState>> or Vec<JoinHandle> for status
- **Broadcast:** tokio::sync::broadcast to TUI event channel
- **Result:** Synthetic ChatMessage::ToolResult injected into main loop

---

## Build Impact

- **Compile time:** Minimal (no heavy dependencies added; mostly tokio extensions)
- **Binary size:** +0 (existing deps extended, not new)
- **Dependencies:** Stays at ~15 core crates (same as current)

---

## Recommended Action

1. **Start:** Use existing tokio utilities (no new ops needed)
2. **Defer:** tracing infrastructure (useful but add after v1.2 MVP)
3. **Manual:** Implement regex-like `<think>` stripping inline (avoid regex dep for MVP)
4. **Async:** Use tokio::task::spawn directly; Arc<Mutex<_>> for state (no dashmap needed for MVP)

**Stack addition verdict:** No new compile-time dependencies. Implementation uses existing crate utilities in new patterns. Add tracing post-MVP if log volume becomes issue.
