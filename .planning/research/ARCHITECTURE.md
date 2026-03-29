# Research: Architecture & Integration for v1.2

## Current Architecture (v1.1)

```
main.rs
  ├─ fn main() { orchestrator loop }
  │   ├─ Read user input (rustyline multiline)
  │   ├─ Dispatch to llama-server via reqwest SSE
  │   ├─ Parse SSE deltas (stream::parse_delta)
  │   ├─ Extract visible text (extract_visible_delta_text)
  │   ├─ Buffer tokens (token_buffer)
  │   ├─ Flush every 30ms or on stream-end
  │   └─ Inject into TUI via event channel
  │
  ├─ fn handle_tool_call(json_args) → CmdOutput
  │   └─ Executes synchronously, blocks loop
  │
  ├─ SSE streaming (split on "data:" lines)
  │   ├─ Buffered line-by-line reading
  │   └─ Complete JSON parsing per line
  │
  └─ Terminal vs TUI dispatch
      ├─ Terminal mode: naive println!
      └─ TUI mode: event channel to tui.rs

tui.rs
  ├─ Event loop (tokio, crossterm)
  ├─ TokenChunk → append to chat buffer
  ├─ StreamingHeartbeat → show progress indicator
  └─ Render on every input or timeout
```

---

## Proposed v1.2 Architecture

### Layer 1: Chat Mode Detection & Prompting

**New:** fn get_system_prompt(mode: ExecutionMode) → String

- **Chat mode:** "You are a concise, direct assistant. No reasoning, no steps, just answers."
- **Agentic mode:** Current system prompt (with reasoning allowance)

**Location:** Add to main.rs or new modes.rs module

```rust
fn get_system_prompt(mode: ExecutionMode) -> String {
    match mode {
        ExecutionMode::Chat => {
            "Answer directly and concisely. No reasoning steps. \
             If you need to show working, use code blocks. \
             Do not output <think> blocks. Do not apologize."
        }
        ExecutionMode::Agentic => {
            // Current prompt with <think> allowed
        }
    }
}
```

---

### Layer 2: Live Streaming (Buffering Strategy)

**Current (v1.1):**
```
SSE stream
  → line buffering (BufRead)
  → 30ms flush timer
  → TUI redraw on tick
  ├─ Problem: Line boundaries add latency
  └─ Problem: Terminal mode has separate buffering
```

**Proposed (v1.2):**
```
SSE stream
  → byte-level reading (tokio::io::AsyncRead + take())
  → immediate parse on complete JSON delta
  → immediate render (no 30ms wait)
  → TUI redraws on every event loop tick
  ├─ Benefit: TTFT < 10ms instead of 30+ms
  └─ Benefit: Streaming appears as typed, not chunked
```

**Code pattern:**
```rust
// BEFORE: Line-based
let reader = BufReader::new(response.bytes_stream());
for line in reader.lines() {
    let json = parse_json(&line)?;
    buffer_token(&json); // 30ms wait
}

// AFTER: Byte-based
let mut response = response.bytes_stream();
while let Some(chunk) = response.next().await {
    for delta in parse_chunk(&chunk) {  // May have multiple deltas in one chunk
        render_immediately(&delta);     // No wait
    }
}
```

---

### Layer 3: Output Filtering (Chat Mode)

**New module:** filters.rs (or inline in main.rs)

```rust
fn filter_chat_response(raw: &str) -> String {
    let mut output = raw.to_string();
    
    // 1. Strip <think>...</think>
    output = strip_think_blocks(&output);
    
    // 2. Strip <thinking>...</thinking>
    output = strip_thinking_blocks(&output);
    
    // 3. Normalize quotes and markdown artifacts
    output = normalize_quotes(&output);
    
    // 4. Deduplicate consecutive phrases
    output = deduplicate_phrases(&output);
    
    output.trim().to_string()
}

fn strip_think_blocks(s: &str) -> String {
    // Manual parsing (no regex dependency)
    let mut result = String::new();
    let mut in_think = false;
    let mut i = 0;
    let bytes = s.as_bytes();
    
    while i < bytes.len() {
        if i + 7 <= bytes.len() && &bytes[i..i+7] == b"<think>" {
            in_think = true;
            i += 7;
        } else if i + 8 <= bytes.len() && &bytes[i..i+8] == b"</think>" {
            in_think = false;
            i += 8;
        } else if !in_think {
            result.push(bytes[i] as char);
            i += 1;
        } else {
            i += 1;
        }
    }
    result
}
```

**Integration point:** Apply after stream complete (before final ChatMessage insertion) OR stream-time (for immediate filtering).

---

### Layer 4: Non-blocking Tool Execution

**Current (v1.1):**
```
Tool call detected → handle_tool_call(args) → blocks loop → returns CmdOutput → continue
  └─ Problem: UI freezes during tool execution (10-30s for slow commands)
```

**Proposed (v1.2):**
```
Tool call detected
  ├─ Emit synthetic ChatMessage::ToolCall("cmd_name", args)
  ├─ Spawn tokio::task::spawn(execute_tool_async)
  │   ├─ Run command in background (std::process::Command)
  │   ├─ Send result via broadcast channel
  │   └─ Generate synthetic ChatMessage::ToolResult
  │
  └─ TUI continues rendering
      ├─ Show "tool_name: running..." in chat
      ├─ Update status when complete
      └─ Display result immediately
```

**Code pattern:**
```rust
// Detect tool calls in response
let tool_calls = parse_tool_calls(&delta)?;

for tool_call in tool_calls {
    // 1. Emit synthetic "tool is running" message
    let call_msg = ChatMessage::ToolCall {
        id: tool_call.id.clone(),
        name: tool_call.name.clone(),
        args: tool_call.args.clone(),
    };
    chat_history.push(call_msg);
    event_tx.send(TuiEvent::ToolCalling(tool_call.clone()))?;
    
    // 2. Spawn async task
    let event_tx_clone = event_tx.clone();
    let id = tool_call.id.clone();
    
    tokio::spawn(async move {
        let result = execute_command(&tool_call).await;
        let result_msg = ChatMessage::ToolResult {
            tool_use_id: id.clone(),
            content: result.stdout,
        };
        event_tx_clone.send(TuiEvent::ToolResult(result_msg))?;
    });
}
```

**TUI updates:**
- Add ToolCalling(ToolCall) event → show "cmd_name: running..."
- Add ToolResult(ChatMessage) event → append result to chat
- Render on every tick (already done)

---

### Layer 5: Parallel Tool Execution

**Pattern:** Collect all tool calls, spawn all tasks, wait for all via JoinHandle vec.

```rust
let mut handles = vec![];

for tool_call in tool_calls {
    let handle = tokio::spawn(async move {
        execute_command(&tool_call).await
    });
    handles.push(handle);
}

// All tools running in parallel now
// Await results as they complete:
for handle in handles {
    let result = handle.await?;
    // Emit result event
}
```

---

## Integration Points (Execution Order)

1. **Phase N1:** System prompt + mode detection
   - Add ExecutionMode enum
   - Add get_system_prompt function
   - Pass mode to llama-server request

2. **Phase N2:** Live streaming refactor
   - Change from BufRead to raw byte streaming
   - Update parse_delta to handle multiple deltas per chunk
   - Immediate render (remove 30ms timer)

3. **Phase N3:** Chat mode filtering
   - Add filters.rs
   - Apply strip_think_blocks in chat-only path
   - Normalize output

4. **Phase N4:** Non-blocking tools
   - Add ToolCalling/ToolResult events to TuiEvent
   - Spawn tasks instead of blocking
   - Collect results via broadcast channel

5. **Phase N5:** Parallel execution
   - Change from single-task spawn to vec of handles
   - Await all concurrently

---

## File Modification Summary

| File | Change | Scope |
|------|--------|-------|
| main.rs | Add ExecutionMode, get_system_prompt, tool async spawning | ~200 LOC |
| tui.rs | Add ToolCalling, ToolResult events; update render | ~50 LOC |
| filters.rs (new) | strip_think_blocks, deduplicate_phrases, etc. | ~100 LOC |
| stream.rs (existing) | Refactor byte-level reading (optional, depends on current impl) | ~50 LOC |
| types.rs | Add ExecutionMode enum, tool status types | ~30 LOC |

**Total new code:** ~430 LOC (mostly new functions, not refactoring existing)

---

## Build Dependencies + Risks

**No new Cargo dependencies required.** All uses existing:
- tokio (spawn, broadcast)
- tokio::io (AsyncRead)
- serde_json (parsing)
- crossterm (TUI already)

**Risks:**
- Byte-level streaming may have edge cases with partial UTF-8 (mitigate with chunking strategy)
- Tool result injection requires careful message ordering (test with multiple simultaneous tools)
- Chat mode filtering may need language-specific tuning (start with one-size-fits-all, iterate)

---

## Verdict

**Architecture is evolutionary, not revolutionary.** No subsystem replacements. All changes are additive or localized refactors. Integration points are clear. Build touches ~430 LOC new, minimal existing churn. Ready to phase-plan.
