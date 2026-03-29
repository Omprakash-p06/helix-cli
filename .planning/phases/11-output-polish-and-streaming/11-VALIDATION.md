---
phase: 11
slug: output-polish-and-streaming
date_created: 2026-03-29
status: pending_verification
---

# Phase 11 Validation Strategy

## 1. Unit Tests (Dimension 1)
- Validation of `<think>` block state transitions when token chunks arrive:
  - Input: chunks `["<th", "ink>", "reason", "</think", "> text"]`
  - Output: Proper partitioning of spans in `ChatEntry.spans` with exactly matching visual styles (`Modifier::DIM` vs default).

## 2. Integration Tests (Dimension 2 & 3)
- Verification that `tui.rs` correctly draws ToolExecution blocks inside `draw_chat_area` using dummy `ToolStart` and `ToolResult` mpsc messages.

## 3. End-to-End & UAT (Dimension 4 & 5)
- **Tool States Rendering**: User must manually observe that when the orchestrator starts a tool (e.g., `list_directory`), the chat panel displays `🔧 Executing list_directory...` with a blue background, and updating to `✅ Success / ❌ Error` when returning.
- **Batching & Fidelity**: User must verify that streaming text does not freeze the terminal thread and renders efficiently due to the `tokio::time::interval` 30ms flush. CPU usage via `htop` should be stable.
- **Thinking UI**: Manual observation that toggle shortcut (`Ctrl+T`) successfully collapses or hides `Dim` formatted thinking spans.
