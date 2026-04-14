# Phase 26 Research: Open-Source Integration References

## Integration Plan: Leveraging Open-Source References for Helix Agent (v1.2)

This plan focuses on **five selected repositories** that offer the most value for the current milestone (Chat Mode Polish, Live Streaming, Non-blocking Tools, Codebase Quality).

---

## 📦 Phase A: Token Counter HUD

**Repository:** [tiktoken-rs](https://github.com/mikaelsouza/tiktoken-rs)

**Tasks:**
- Add `tiktoken-rs` dependency to `Cargo.toml`.
- Implement token counting for input buffer and conversation context.
- Render `[current/max]` gauge in TUI status/input area using `ratatui`.
- Use `cl100k_base` and cache the tokenizer instance.

---

## ⚙️ Phase B: Parallel Tool Execution & Agent Loop

**Repository:** [open-multi-agent-rs](https://github.com/Supernova1744/open-multi-agent-rs)

**Tasks:**
- Adapt dependency-aware task scheduling patterns.
- Implement async `ToolExecutor` using `tokio::spawn` and `futures::join_all`.
- Emit `ToolCallStart`, `ToolCallProgress`, `ToolCallEnd` events via existing channels.

---

## 🖥️ Phase C: TUI Widgets for Tool Calls & Streaming

**Repository:** [o7](https://github.com/clankercode/o7)

**Tasks:**
- Implement collapsible tool call blocks inside assistant messages.
- Expansion of tool calls on `Enter` to show arguments, results, and duration.
- Throttle `terminal.draw()` for smoother token-by-token streaming.

---

## 🛠️ Phase D: Tool Design Reference (Optional)

**Repository:** [kbot](https://github.com/isaacsight/kernel)

**Tasks:**
- Implement a `ToolRegistry` with `HashMap<String, Tool>`.
- Support JSON schema argument validation via `serde_json::Value`.
- Add permission tiers (`read_only`, `write`, `dangerous`).

---

## 📚 Phase E: Learn from OpenDev Paper

**Resource:** [OpenDev technical report](https://arxiv.org/abs/2603.11710)

**Tasks:**
- Study scaffolding, harness design, and context engineering.
- Apply retry logic and graceful degradation to `orchestrator.rs`.
- Document lessons learned on tool call reliability.

---

## 🧪 Testing & Validation
- Parallel execution: verify multiple file reads execute concurrently.
- TUI rendering: confirm tool calls are collapsible and show status icons without flicker.
- Streaming: confirm zero-latency rendering of incoming tokens.
