# Phase 11: Output Polish and Streaming
## Discussion Log

**Date:** 2026-03-29

### Q1: Streaming Performance
**Options Presented:** Raw token-by-token or buffered chunking?
**User Decision:** Raw tokens but with a time-based batching (e.g. 30ms flush) using `tokio::time::interval`.

### Q2: `<think>` Block Rendering
**Options Presented:** How do we visually separate AI thoughts? (Dimmed inline, Collapsible, Dedicated sidebar)
**User Decision:** Dimmed inline text using `ratatui::style::Style::dim()`. Maintain a `Vec<(String, Style)>` per ChatEntry. Add `Ctrl+T` hotkey to toggle visibility.

### Q3: Tool Execution States
**Options Presented:** How do we display autonomous actions?
**User Decision:** Insert directly into chat log as separate entries. Style with monospaced font and subtle background. Use `TuiEvent::ToolStart` and `ToolResult`. Add `Ctrl+C` interrupt for long-running tools.
