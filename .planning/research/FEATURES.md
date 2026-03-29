# Research: Features & Patterns for v1.2

## Chat Mode Output Filtering

### Table Stakes
1. **System prompt carve-out** — Chat mode gets strict system prompt (no reasoning, no hesitation)
2. **Think-block stripping** — Remove `<think>...</think>` sections before display
3. **Duplicate phrase removal** — Filter repeated phrases/sentences that models sometimes emit
4. **Quote normalization** — Clean up stray quotes, backticks, markdown artifacts

### Differentiators
- **Intent detection** — Detect when user is asking something agentic vs. conversational
- **Confidence scoring** — Mark uncertain responses with low-confidence signal
- **Fallback prompts** — Different system prompts for edge cases (math, code, reasoning-required)

### Implementation Patterns
```
Chat mode response → raw delta stream
  ↓
Extract visible text (content, text, response keys)
  ↓
Stream to buffer (token by token)
  ↓
Apply filters:
  - Strip <think>...</think>
  - Remove duplicate consecutive phrases
  - Normalize quotes/markdown
  ↓
Render to TUI/Terminal immediately
```

**Anti-pattern:** Buffering entire response before filtering (causes perceived latency).

---

## Live Streaming Without Buffering

### Table Stakes
1. **Raw byte reading** — No line buffering; byte-by-byte or small chunk (1-8 bytes)
2. **Immediate render** — Token appears in UI within 50ms of reception
3. **No accumulation** — Each byte pushed to renderer; no "wait for full word"
4. **Interrupt safety** — Ctrl+C shows partial, not blank

### Differentiators
- **TTFT tracking** — Show when first token arrives (time-to-first-token metric)
- **Generation speed graph** — Live tokens/sec display in TUI
- **Stutter detection** — Alert if generation speed drops below threshold
- **Adaptive flushing** — Adjust flush interval based on token rate

### Implementation Patterns
```
SSE stream (HTTP framing)
  ↓
Read raw bytes (do NOT wait for \n)
  ↓
Buffer until complete JSON delta
  ↓
Parse JSON (extract text field)
  ↓
Send to rendering immediately (< 1ms)
  ↓
No wait-for-accumulation
  ↓
TUI renders on every event loop tick
Terminal mode flushes after each chunk
```

**Anti-pattern:** Waiting for full line, full sentence, or "readable chunk" (adds 100ms+ latency).

---

## Non-blocking Tool Execution

### Table Stakes
1. **Async spawn** — Each tool call gets its own tokio task
2. **Non-blocking UI** — TUI continues rendering while tools run
3. **Status display** — User sees "tool: running..." in chat area
4. **Result injection** — Tool result becomes synthetic ChatMessage

### Differentiators
- **Parallel tools** — Multiple tools run concurrently (T1 + T2 at same time, not T1 then T2)
- **Tool chaining** — T1 result feeds into T2 input automatically
- **Confirmation prompts** — Dangerous tools (rm, sudo) ask before executing
- **Timeout handling** — Long-running tools abort after N seconds

### Implementation Patterns
```
User message with tool call → parsed tools: [T1, T2, T3]
  ↓
For each tool:
  - Spawn tokio::task::spawn with tool execution
  - Generate synthetic "tool_call" ChatMessage for UI
  - Track JoinHandle for completion monitoring
  ↓
TUI shows "T1, T2, T3 running..." (live status)
  ↓
As each completes:
  - Receive result from channel
  - Generate synthetic "tool_result" ChatMessage
  - Update TUI status to "T1 ✓, T2 ✓, T3 running"
  ↓
When all done:
  - Inject tool_call + results into chat history
  - Resume model context window for next response
```

**Anti-pattern:** Spawning one tool, waiting for completion, then spawning next (defeats parallelism).

---

## Output Quality (Chat Mode Only)

### Visible Thinking Stripping

**Pattern:** Match `<think>...</think>` (or `<thinking>...</thinking>`) and remove.

Example:
```
RAW: "I think about this... <think>The user wants math. I should use calculator.</think> The answer is 42."
CLEAN: "The answer is 42."
```

**Regex (if used):** `(?s)<think>.*?</think>` or manual parsing.

**Manual approach (recommended for MVP):**
- Split on `<think>` and `</think>`
- Keep even-indexed segments (before/between thinks)
- Rejoin

### Duplicate Phrase Removal

**Pattern:** Detect and remove consecutive identical phrases.

Example:
```
RAW: "I can help you. I can help you. What do you need?"
CLEAN: "I can help you. What do you need?"
```

**Approach:** Simple dedup on sentence/phrase boundaries.

### Professional Formatting

**Pattern:** Ensure chat-mode responses are single-paragraph or short sentences.

Rules:
- No "Let me think..." or deliberation phrases
- No bullet lists unless explicitly asked
- No numbered steps unless required
- No markdown unless code block
- No emojis or personality

Example:
```
USER: "What's 2+2?"
BAD: "Let me think... 2+2=4. That's simple addition."
GOOD: "4"

USER: "Write a function to sort an array"
BAD: "Here are some steps:
1. Choose a sorting algorithm
2. Implement it
3. Test it"
GOOD: "Here's a quicksort implementation:
<code block>"
```

---

## Features Complexity Assessment

| Feature | Complexity | Risk | Timeline |
|---------|-----------|------|----------|
| System prompt + intent detection | Low | Low | 1 day |
| Think-block stripping | Low | Low | 0.5 day |
| Live streaming (byte-level) | Medium | Medium | 1 day |
| Non-blocking tools | Medium | Medium | 1.5 days |
| Parallel tool execution | Medium | Medium | 1 day |
| Tool status UI | Medium | Low | 1 day |
| Output quality filters | Low | Low | 0.5 day |

**Total:** 6-7 days for full feature set (MVP is 3-4 days: prompt, stripping, live streaming).

---

## Anti-patterns to Avoid

1. **Blocking tool execution** — Spawning tasks but not actually awaiting them in parallel
2. **Buffered streaming** — Waiting for newlines or "complete words" before display
3. **Chat mode leakage** — System prompt for chat mode accidentally inheriting agentic behaviors
4. **No status feedback** — User queries hang silently during tool execution
5. **Synchronous filtering** — Applying regex/parsing on hot path (filter off-path if possible)
6. **Incomplete interrupt handling** — Tool results lost when Ctrl+C pressed mid-execution

---

## Verdict

**All features achievable within 5-6 day estimate.** Chat mode prompt swap is trivial. Streaming refactor is mechanical. Tool parallelism is standard tokio pattern. No architectural surprises.
