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
## test-registry-mismatch — Test failure due to outdated hardcoded tool list
- **Date:** 2026-04-25
- **Error patterns:** assertion left == right failed, plugin_sdk_ide_bridge_validation, registry_exposes_all_builtins_and_persona_filters_payloads
- **Root cause:** The test had hardcoded expectations for the number and names of tools in the registry, which were not updated after new diagnostic tools were added.
- **Fix:** Updated the expected tool list and adjusted payload length assertions in the test.
- **Files changed:** agent-rs/tests/plugin_sdk_ide_bridge_validation.rs
---

## model-install-test-failures — Model installation tests failed due to security and registry updates
- **Date:** 2025-01-24
- **Error patterns:** AssertionError, resolve_model_ref, install_model_spec, TRUSTED_MODELS, Qwen, revision, sha256
- **Root cause:** Tests in `tests/test_model_install.py` were not updated after the security pivot that introduced strict revision pinning and SHA256 validation in `scripts/model_install.py`, and after the trusted model registry was updated to Qwen-only.
- **Fix:** Updated `tests/test_model_install.py` to use `qwen-3.6-27b-moe` for resolution tests and added valid mock `revision` and 64-char `sha256` to the test model specs.
- **Files changed:** tests/test_model_install.py
---
