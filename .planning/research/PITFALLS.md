# Research: Pitfalls & Prevention for v1.2

## Common Mistakes When Adding Chat Mode Filtering

### Pitfall 1: Filtering Readable-Looking Thinking Variations

**Problem:** Models output reasoning in many formats:
- `<think>...</think>` (Claude-style)
- `<thinking>...</thinking>` (variations)
- `<analysis>...</analysis>` (some models)
- `[Internal reasoning...]` (some models)
- Implicit reasoning lines (e.g., "Let me work through this...")

**Naive approach:** Only strip `<think>`, leaving remnants visible.

**Result:** User sees partial reasoning: "Let me work through this... the answer is 42."

**Prevention:**
- Create exhaustive list of reasoning pattern markers
- Test against multiple model outputs (Qwen, Llama, Mistral)
- Make strip_think_blocks idempotent (safe to run twice)
- Log what's being stripped (for UAT validation)

**Recommended:**
```rust
const THINKING_MARKERS: &[(&str, &str)] = &[
    ("<think>", "</think>"),
    ("<thinking>", "</thinking>"),
    ("<analysis>", "</analysis>"),
    ("<internal>", "</internal>"),
];

fn strip_all_thinking(s: &str) -> String {
    // Iterate through markers and strip each
    // Be careful of marker overlap/nesting
}
```

---

### Pitfall 2: Chat vs. Agentic Mode Leakage

**Problem:** System prompt set to chat mode, but user queries agentic behavior.

Example:
```
USER: "Surf the web and find the weather"
SYSTEM: "No tools, just chat"
MODEL: "I can't do that, I don't have tools"
EXPECTED: Model calls tool_use, then provides answer
```

**Naive approach:** Set system prompt once at startup.

**Result:** Chat mode never allows thinking/tool prep; agentic mode shows reasoning.

**Prevention:**
- Detect user intent BEFORE sending to model (explicit mode switch in CLI)
- Or: Use two separate system prompts and inject dynamically per turn
- Test: "Show all system prompts being sent in each mode"

**Recommended:**
```rust
match mode {
    ExecutionMode::Chat => {
        // Chat system prompt (no tools mention, no reasoning)
        system_prompt = CHAT_PROMPT;
    }
    ExecutionMode::Agentic => {
        // Agentic system prompt (tools allowed, reasoning encouraged)
        system_prompt = AGENTIC_PROMPT;
    }
}

// Verify prompt is set correctly before every request
assert_eq!(get_mode(), mode);
```

---

### Pitfall 3: Overshooting Output Filtering

**Problem:** Stripping too aggressively, removing legit content.

Example:
```
RAW: "My thoughts: <think>this is complex</think> Solution: use binary search."
OVER-FILTERED: "My thoughts:  Solution: use binary search."
RESULT: Nonsensical output
```

**Naive approach:** Strip all `<...>` tags.

**Result:** Removes code variables, XML examples, angle brackets in normal text.

**Prevention:**
- Only strip specific reasoning tags (white-list, not black-list)
- Preserve code blocks, examples, XML
- Test filtering on diverse outputs (code, examples, narratives)

**Recommended:**
```rust
// WHITE-LIST approach (safer)
const STRIP_ONLY: &[&str] = &[
    "<think>", "</think>",
    "<thinking>", "</thinking>",
];

// Don't strip generic tags like <div>, <span>, etc.
// Don't strip if inside code block (check for ``` context)
```

---

### Pitfall 4: Deduplication Creates Readability Gaps

**Problem:** Removing consecutive phrases makes responses disjointed.

Example:
```
RAW: "I can help. I can help with that. Let me write code."
DEDUP: "I can help. Let me write code."  <- Okay
OVER-DEDUP: "I can help. Let me." <- Broken!
```

**Naive approach:** Global phrase dedup without context.

**Result:** Legitimate repeated words removed ("The the" → "The"), or broken sentences.

**Prevention:**
- Only deduplicate EXACT consecutive sentences, not phrases
- Preserve variant phrasings ("I can help" vs "I can assist")
- Make dedup conservative (only 100% exact dups)

**Recommended:**
```rust
fn deduplicate_sentences(s: &str) -> String {
    let sentences: Vec<&str> = s.split('.').collect();
    let mut result = vec![];
    
    for sentence in sentences {
        if result.is_empty() || result.last() != Some(&sentence) {
            result.push(sentence);
        }
    }
    
    result.join(".")
}

// Only dedup at sentence boundary, not word level
```

---

## Common Mistakes When Adding Live Streaming

### Pitfall 5: Partial UTF-8 Sequences Break Rendering

**Problem:** Streaming byte-by-byte can split multi-byte UTF-8 characters.

Example:
```
Bytes: [0xE2, 0x9C, 0x93]  (UTF-8 for ✓)
Stream reads: [0xE2] → invalid UTF-8 → render fails
```

**Naive approach:** Convert bytes to String immediately without validation.

**Result:** Terminal shows "?" or crashes.

**Prevention:**
- Buffer bytes until you have a complete valid UTF-8 sequence
- Use `String::from_utf8_lossy()` for partial streams
- Validate UTF-8 at chunk boundaries

**Recommended:**
```rust
fn safe_utf8_push(buffer: &mut String, chunk: &[u8]) -> Result<()> {
    match String::from_utf8(chunk.to_vec()) {
        Ok(s) => buffer.push_str(&s),
        Err(e) => {
            // Partial UTF-8; buffer bytes and recheck later
            // OR use String::from_utf8_lossy as fallback
            let s = String::from_utf8_lossy(chunk);
            buffer.push_str(&s);
        }
    }
    Ok(())
}
```

---

### Pitfall 6: Rendering Performance Degrades at High Token Rates

**Problem:** TUI redraws at 60 Hz, but model produces 100 tokens/sec.

Result: Dropped tokens, stuttering, CPU thrashing.

**Naive approach:** Redraw on every token event.

**Result:** TUI thread spends 100% time redrawing, can't keep up.

**Prevention:**
- Batch tokens (buffer 5-10ms worth before redraw)
- Use debounce/throttle on render loop
- Track frame budget (16ms per frame @ 60 Hz)

**Recommended:**
```rust
// Batch tokens with 10ms window
let mut token_batch = String::new();
let mut last_render = std::time::Instant::now();

for delta in stream {
    token_batch.push_str(&delta.text);
    
    if last_render.elapsed() >= Duration::from_millis(10) {
        render_ui(&token_batch);
        token_batch.clear();
        last_render = std::time::Instant::now();
    }
}

// Flush remaining after stream ends
render_ui(&token_batch);
```

---

### Pitfall 7: Interrupt Path Doesn't Flush Live Tokens

**Problem:** User presses Ctrl+C during streaming. Partial response lost.

Example:
```
Model streaming: "The answer is..."
User: Ctrl+C
Expected: "The answer is" shown
Actual: Blank (tokens were buffered, not displayed)
```

**Naive approach:** No special handling for interrupt.

**Result:** Ctrl+C loses streaming tokens.

**Prevention:**
- Flush any pending tokens on interrupt signal
- Store partial response in persistent buffer
- Test: Generate 100-token response, press Ctrl+C at 2s, verify partial shown

**Recommended:**
```rust
tokio::select! {
    result = stream_fut => {
        // Normal completion
        render_final(&buffer);
    }
    _ = interrupt_signal => {
        // User pressed Ctrl+C
        flush_buffer_to_ui(&buffer);  // Show what we have so far
        break;
    }
}
```

---

## Common Mistakes When Adding Non-blocking Tools

### Pitfall 8: Tool Result Arrives After Context Window Moves On

**Problem:** Tool runs async. By the time result arrives, model is already generating next response.

Example:
```
T=0ms: Model outputs tool_use(cmd: "ls")
T=100ms: Loop continues, doesn't wait; sends model a dummy response
T=500ms: Actual tool result arrives, too late
Result: Tool output not included in model's next response
```

**Naive approach:** Fire-and-forget tool spawn without result tracking.

**Result:** Model never sees tool output; acts as if tool didn't run.

**Prevention:**
- Wait for tool completion before continuing model loop
- Use JoinHandle + await in orchestrator
- Breadcrumb: Add tool_result synthetic message before next model turn

**Recommended:**
```rust
// Fire tools async
let mut handles = vec![];
for tool_call in tool_calls {
    let h = tokio::spawn(execute(&tool_call));
    handles.push((tool_call.id, h));
}

// Collect ALL results before continuing
for (id, handle) in handles {
    let result = handle.await?;
    // Inject into chat history NOW before model sees it
    chat_history.push(ChatMessage::ToolResult { id, content: result });
}

// NOW model responds with tool context available
let next_response = query_model(&chat_history).await;
```

---

### Pitfall 9: Parallel Tools Interfere (Shared State)

**Problem:** Two tools modify the same resource (e.g., file, directory).

Example:
```
Tool 1: "mkdir /tmp/test"
Tool 2: "touch /tmp/test/file.txt"  (runs in parallel with T1)
Result: Race condition, "directory doesn't exist" error
```

**Naive approach:** Spawn all tasks without coordination.

**Result:** Non-deterministic failures.

**Prevention:**
- Document tool dependencies (which tools can't run together)
- Add explicit semaphore/lock for conflicting operations
- Order tools: T1, then T1_dependent_T2

**Recommended:**
```rust
// Mark tool dependencies
const TOOL_CONFLICTS: &[(&str, &str)] = &[
    ("mkdir_test", "touch_test_file"),  // Must run in order
];

// Check conflicts before spawning parallel
for (t1, t2) in TOOL_CONFLICTS {
    if all_tool_names.contains(&t1) && all_tool_names.contains(&t2) {
        // Serial execution: await T1 before spawn T2
        let r1 = execute_tool(t1).await;
        chat_history.push(ToolResult(r1));
        let r2 = execute_tool(t2).await;
        chat_history.push(ToolResult(r2));
    }
}
```

---

### Pitfall 10: Tool Timeout Not Enforced

**Problem:** Tool hangs (infinite loop, network wait). User can't escape.

Example:
```
Tool: "curl https://stalled-server.com" (waits forever)
User: Ctrl+C typed
Result: Ignored; loop blocked on std::process::Command
```

**Naive approach:** No timeout on tool execution.

**Result:** Hung tool freezes agent.

**Prevention:**
- Enforce timeout (e.g., 30s max per tool)
- Use `tokio::time::timeout`
- Return error on timeout, don't panic

**Recommended:**
```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    execute_tool_async(&tool_call)
).await;

match result {
    Ok(Ok(output)) => {
        // Tool completed within timeout
        inject_result(output);
    }
    Ok(Err(e)) => {
        // Tool errored
        inject_error(e);
    }
    Err(_elapsed) => {
        // Tool timeout
        inject_error("Tool execution timeout after 30s");
    }
}
```

---

## Verification Checklist (Before Shipping v1.2)

- [ ] Chat mode response has zero visible `<think>`, `<reasoning>`, `[Internal...]` markers
- [ ] Chat mode response on `"hey!"` is < 10 words, direct, no reasoning steps
- [ ] Live streaming: First token appears < 50ms after model starts
- [ ] Live streaming: Middle tokens appear within 100ms
- [ ] Interrupt (Ctrl+C): Partial response visible, not blank
- [ ] Tool execution: UI does not freeze, status shown as "running..."
- [ ] Parallel tools: Two file-creating tools complete without race errors
- [ ] Tool timeout: Long-running tool (e.g., sleep 60) times out after 30s, returns error
- [ ] No UTF-8 corruption: Emoji and multi-byte chars render correctly
- [ ] Chat → Agentic switch: Mode change persists across turns (not leaked to opposite mode)

---

## Risk Level: Medium

**High confidence** that core patterns are sound (async tools standard, streaming is mechanical).

**Medium confidence** on edge cases (tool conflicts, UTF-8 partial streams, filter over-shooting).

**Mitigation:** Extensive unit testing per pitfall, UAT with diverse models and toolsets, gradual rollout (chat mode first, then streaming, then tools).
