# Phase 7: Generation Speed & Thought UI - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning
**Source:** User Directive (v1.1 Initialization)

<domain>
## Phase Boundary

This phase introduces optimizations to Time-To-First-Token (TTFT) by tuning the LLM flags, and completely overhauls the internal `<think>` block handling inside the Rust orchestrator to pass the raw reasoning tags seamlessly down to the React UI and the terminal for visual rendering.

</domain>

<decisions>
## Implementation Decisions

### Thought Visibility
- The Rust orchestrator must be updated to stop stripping `<think>` tags via `strip_think_blocks`.
- Instead, it will map internal `<think>` tokens into a format-friendly `<thinking>` tag structure.
- The web frontend will utilize `rehype-raw` to interpret the raw HTML tags inside its markdown blocks, and global CSS styling will apply a glassmorphism/muted aesthetic globally.

### TTFT Optimization
- The fallback `koboldcpp` arguments (`KOBOLDCPP_ARGS`) must be injected with `--flashattention` by default to accelerate token calculation.

</decisions>
