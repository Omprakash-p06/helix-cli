# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## http-error-model-unreachable — Chat mode failed with HTTP unreachable and no visible output
- **Date:** 2026-03-30
- **Error patterns:** HTTP Error, error sending request, model server unreachable, 127.0.0.1:8080, chat/completions, CUDA out of memory, reasoning_content only
- **Root cause:** Intermittent model-server availability plus startup race/fallback timing and reasoning-heavy chat output caused prompt failures or apparent hangs.
- **Fix:** Added Rust-side recovery retries and safe auto-boot, enabled launcher env overrides and CPU fallback defaults, disabled chat thinking via `chat_template_kwargs.enable_thinking=false`, and added non-empty visible fallback handling.
- **Files changed:** agent-rs/src/main.rs, scripts/start_server.py, scripts/config.py, .planning/debug/resolved/http-error-model-unreachable.md
---
