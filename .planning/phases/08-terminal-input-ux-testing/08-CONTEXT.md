# Phase 8: Terminal Input UX & Testing - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning
**Source:** User Directive (v1.1 Initialization)

<domain>
## Phase Boundary

This phase addresses the interaction friction in the terminal interface and introduces a programmatic safety/accuracy gate for the agent's tool-calling engine.

</domain>

<decisions>
## Implementation Decisions

### Terminal UX
- The `rustyline` multi-line validator in `agent-rs/src/input.rs` will be simplified to submit on a single `Enter` press.
- Pasting large blocks of text will still be handled correctly due to `bracketed_paste` configuration.

### Accuracy Testing
- A standalone Python script `tests/test_accuracy.py` will be created.
- This script will benchmark the `localhost:8080` API against specific tool-calling prompts to verify GBNF and logic compliance.

</decisions>
