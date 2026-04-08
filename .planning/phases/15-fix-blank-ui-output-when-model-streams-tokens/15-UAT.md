---
status: testing
phase: 15-fix-blank-ui-output-when-model-streams-tokens
source: [.planning/phases/15-fix-blank-ui-output-when-model-streams-tokens/15-01-SUMMARY.md, .planning/phases/15-fix-blank-ui-output-when-model-streams-tokens/15-02-SUMMARY.md]
started: 2026-03-31T16:57:47Z
updated: 2026-03-31T16:57:47Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 1
name: Chat mode hides internal reasoning traces
expected: |
  Start Helix in chat mode and send a prompt that usually triggers long reasoning.
  The visible assistant output should be direct and concise, and must not contain
  any <think>, <thinking>, or <analysis> tags.
awaiting: user response

## Tests

### 1. Chat mode hides internal reasoning traces
expected: Start Helix in chat mode and send a prompt that usually triggers long reasoning. Visible output is direct/concise and contains no <think>, <thinking>, or <analysis> tags.
result: [pending]

### 2. Agentic mode preserves reasoning transparency behavior
expected: Start Helix in agentic mode and run a tool-capable request. Agentic output should still expose reasoning/transparency behavior (for example visible thinking blocks) rather than being chat-sanitized.
result: [pending]

### 3. Chat output deduplicates immediate repeated sentences
expected: In chat mode, if the model emits the same sentence consecutively, the final visible output keeps one copy instead of repeating the sentence back-to-back.
result: [pending]

### 4. Chat output normalizes quote artifacts
expected: In chat mode, curly quote artifacts are normalized in final visible output so text appears clean and professional.
result: [pending]

### 5. Chat cleaner preserves fenced code and tool-like JSON
expected: In chat mode, fenced code blocks, inline code, and tool-like JSON snippets remain intact and readable after cleanup.
result: [pending]

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps

[none yet]
