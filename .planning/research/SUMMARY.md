# Research: Summary for v1.2 Planning

**Milestone:** v1.2 Chat Mode Polish & Streaming Reliability
**Research date:** 2026-03-29
**Dimensions:** Stack | Features | Architecture | Pitfalls

---

## Key Findings

### 1. Stack: No New Dependencies Required ✓

All v1.2 features use existing Cargo crates (tokio, serde_json, axum, ratatui, crossterm).

**Implication:** Compile time, binary size, and dependency surface unchanged. Complexity is localized to new patterns within existing code.

**Note:** tracing/logging infrastructure deferred to post-MVP (useful but not blocking).

---

### 2. Features: All Achievable Within Estimate ✓

| Feature | Complexity | Timeline | Risk |
|---------|-----------|----------|------|
| Chat mode prompt + filtering | Low | 1-2 days | Low |
| Live streaming (byte-level) | Medium | 1 day | Medium |
| Non-blocking tools | Medium | 1.5 days | Medium |
| Parallel tool execution | Medium | 1 day | Low |
| Tool status UI | Medium | 1 day | Low |

**MVP (chat mode + live streaming):** 2-3 days
**Full feature set:** 5-6 days (matches user estimate)

---

### 3. Architecture: Evolutionary, Not Revolutionary ✓

- No subsystem replacements (incremental improvements)
- Integration points clearly defined
- ~430 LOC new code across main.rs, tui.rs, filters.rs, types.rs
- No existing churn (additive changes)

**Confidence:** High (patterns are well-established: tokio async, event channels, TUI state).

---

## Critical Integration Sequence

**Phase 1A: System Prompt Foundation**
- Add ExecutionMode enum (Chat vs. Agentic)
- Implement get_system_prompt() function
- Verify mode is passed to model correctly

**Phase 1B: Output Filtering**
- Implement strip_think_blocks() manually (no regex dep)
- Create filters.rs module
- Apply filtering in chat-mode-only path

**Phase 2: Live Streaming Refactor** (sequential after 1B)
- Change from line buffering → byte-level reads
- Immediate render (remove 30ms accumulation timer)
- Verify TTFT < 50ms in both terminal and TUI

**Phase 3: Non-blocking Tools** (parallel to 2 after checkpoint)
- Spawn tool tasks instead of blocking
- Add ToolCalling + ToolResult events
- Broadcast tool results to TUI

**Phase 4: Parallel Execution** (after 3)
- Collect all tool calls, spawn all, await all
- Manage tool result ordering/injection

---

## Pitfalls to Prevent

### High Priority (Must Address in v1.2)
1. **Partial thinking marker stripping** → Exhaustive marker list, test against multiple models
2. **Chat/agentic mode leakage** → System prompt isolation, explicit mode detection
3. **Partial UTF-8 sequences** → Buffer until complete, use `String::from_utf8_lossy()` for fallback
4. **Tool result timeline** → Collect tool results before next model turn
5. **Tool timeout enforcement** → 30s timeout per tool, no infinite waits

### Medium Priority (Test, May Skip MVP)
6. Over-filtering readable content → White-list approach, preserve code blocks
7. Deduplication creates gaps → Sentence-level only, not word-level
8. Rendering performance degradation → Batch tokens with 10ms window
9. Tool interference (race conditions) → Document conflicts, serialize if needed
10. Interrupt path loses tokens → Flush on Ctrl+C, persist partial

---

## Recommendations

### Do
✓ Implement chat mode system prompt + filtering first (low risk, immediate UX improvement)
✓ Refactor streaming to byte-level reads (mechanical, well-understood)
✓ Add tool async spawning (standard tokio pattern)
✓ Use existing tokio utilities (no new deps)
✓ Focus on 5 main pitfalls (1-5 above)
✓ Test with multiple models (Qwen, Llama, Mistral) to cover thinking marker variation

### Don't
✗ Add dependencies (regex, dashmap, etc.) without strong justification
✗ Mix chat filtering with agentic reasoning (separate code paths, explicit modes)
✗ Ignore UTF-8 partial sequences (will cause sporadic rendering crashes)
✗ Assume tool ordering is implicit (explicit result collection required)
✗ Skip interrupt handler testing (lost tokens are UAT failure)

### Consider
◐ Tracing/logging post-MVP (useful for debugging, not blocking)
◐ Confirmation prompts for dangerous tools (implementation detail, nice-to-have)
◐ Tool chaining (sequential tool dependency, Phase 5+ feature)
◐ Timeout configuration UI (hardcode 30s for now, parameterize later)

---

## Estimated Effort Breakdown

| Component | Days | Notes |
|-----------|------|-------|
| Phase 1A: System prompt + modes | 0.5 | Trivial |
| Phase 1B: Filtering + think-block strip | 1 | Manual parsing, no deps |
| Phase 2: Live streaming refactor | 1.5 | Byte-level reads, render timing |
| Phase 3: Non-blocking tools | 1.5 | Event channels, broadcasts |
| Phase 4: Parallel tools | 1 | Collect handles, await concurrently |
| Testing + UAT | 1 | Multiple models, edge cases |
| Codebase cleanup (clippy, etc.) | 0.5 | Standard polish |
| **Total** | **~7 days** | Conservative (5-6 user estimate + buffer) |

---

## Confidence Assessment

| Dimension | Confidence | Why |
|-----------|-----------|------|
| **Stack** | 🟢 Very High | No unknowns, all existing crates |
| **Features** | 🟢 Very High | Patterns are standard (async, filtering, rendering) |
| **Architecture** | 🟢 Very High | Evolution of v1.1, clear integration |
| **Pitfalls** | 🟡 High | Some edge cases (UTF-8, tool ordering), mitigations known |
| **Timeline** | 🟡 High | 5-6 days achievable; +1 day buffer if edge cases emerge |

---

## Ready to Plan?

✓ Research complete. All dimensions covered.
✓ No blockers identified. Patterns understood.
✓ Pitfalls catalogued. Prevention strategies documented.
✓ Effort estimate is realistic (within user's 5-6 day projection).

**Next step:** `/gsd-plan-phase 15 --with-research` will use this research to generate phase plans with concrete tasks.

---

## Research Artifacts

- `STACK.md` — Dependency inventory, no additions needed
- `FEATURES.md` — Chat filtering patterns, streaming patterns, tool patterns
- `ARCHITECTURE.md` — Integration points, code locations, file modifications
- `PITFALLS.md` — 10 common mistakes, prevention strategies, verification checklist

**All files ready for roadmapper consumption.**
