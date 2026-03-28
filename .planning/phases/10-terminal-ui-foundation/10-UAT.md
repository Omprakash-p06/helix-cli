---
status: testing
phase: 10-terminal-ui-foundation
source: [10-01-SUMMARY.md]
started: 2026-03-27T18:38:00Z
updated: 2026-03-27T18:38:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 1
name: Cold Start Smoke Test
expected: |
  Start the application from scratch (`cargo run`). The orchestrator boots without errors. The welcome banner displays the Helix ASCII art and "Py + Rust Hybrid Agent Stack".
awaiting: user response

## Tests

### 1. Cold Start Smoke Test
expected: Start the application from scratch (`cargo run`). The orchestrator boots without errors. The welcome banner displays the Helix ASCII art and "Py + Rust Hybrid Agent Stack".
result: [pending]

### 2. Ghost Autocomplete & Typing
expected: Type `/he`. Faint autocomplete text should suggest `lp` right after the cursor.
result: [pending]

### 3. Multiline Input & Character Counter
expected: Type `Hello`. The status bar should update with character and line counts. Press `Enter`. The cursor should move to a new line without submitting. Type `World`. Both lines remain in the input field.
result: [pending]

### 4. Command Preview & Submission
expected: Press `Alt+Enter` to submit the multiline text. A Command Preview popup should appear. Press `Enter` to confirm. The prompt text should be appended to the Chat History with your role ("▶ You:").
result: [pending]

### 5. History Navigation
expected: Press the `Up` arrow key. The input field should populate with the previously submitted prompt.
result: [pending]

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0

## Gaps

