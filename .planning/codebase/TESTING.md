# TESTING.md

## Snapshot
Last refreshed: 2026-03-29
Testing exists but is uneven across Python, Rust, and web layers.

## Existing Test Assets

### Rust Unit Tests (inline modules)
- `agent-rs/src/main.rs` has `mod tests` section.
- `agent-rs/src/stream.rs` has parser-focused tests.
- `agent-rs/src/utils.rs` has utility tests.

### Python Evaluation Scripts
- `tests/test_accuracy.py`: endpoint-driven functional checks for tool call selection behavior.
- `tests/eval.py`: additional evaluation harness.
- `tests/dataset.json`: prompt dataset used by evaluation scripts.

## How Tests Are Typically Run
- Rust: `cd agent-rs && cargo test`
- Python eval: run scripts under `tests/` after local server is available
- Web UI: no dedicated automated test suite currently detected (lint/build scripts only)

## Current Gaps
- No clearly integrated end-to-end CI pipeline at repository root.
- Web UI lacks unit/integration test framework setup.
- Some validation relies on manual runtime checks and phase-level UAT docs.

## Suggested Priority Improvements
1. Add a repeatable root test command that runs Rust + Python checks.
2. Add web UI component/integration tests.
3. Add smoke tests for `start.py` orchestration paths (tui/web, chat/agentic).
