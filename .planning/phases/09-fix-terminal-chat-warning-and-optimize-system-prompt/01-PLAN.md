---
wave: 1
depends_on: []
files_modified:
  - agent-rs/src/main.rs
autonomous: true
requirements: []
---

# Phase 9: Fix terminal chat warning and optimize system prompt

<objective>
Remove the `unused_mut` warning in `agent-rs/src/main.rs` and optimize the terminal chat system prompt so that simple chat queries do not overthink.
</objective>

## Tasks

<task>
<description>
Remove the unused `mut` keyword from the `res` variable mapping the HTTP response.
</description>
<read_first>
- `agent-rs/src/main.rs` (lines 280-300)
</read_first>
<action>
In `agent-rs/src/main.rs` around line 285:
- Change `let mut res = match client.post(&url).json(&request_body).send().await {`
- To: `let res = match client.post(&url).json(&request_body).send().await {`
</action>
<acceptance_criteria>
- `agent-rs/src/main.rs` contains `let res = match client.post(&url).json(&request_body).send().await {` (with no `mut`).
- Executing `cargo build` inside `agent-rs/` yields no `unused_mut` warning for `res`.
</acceptance_criteria>
</task>

<task>
<description>
Prevent the terminal chat from artificially injecting a system prompt to avoid excessive model overthinking.
</description>
<read_first>
- `agent-rs/src/main.rs` (lines 115-155)
</read_first>
<action>
In `agent-rs/src/main.rs` around lines 142-150:
Change the unconditional pushing of the `system` message:
```rust
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: Some(system_prompt.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];
```
To only push it if `system_prompt` is not empty.
First, update the `system_prompt` definition at line 118:
```rust
    let system_prompt = if is_chat_mode {
        ""
    } else {
        match persona.as_str() { ...
```
Then conditionally push it:
```rust
    let mut messages = vec![];
    if !system_prompt.is_empty() {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: Some(system_prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }
```
</action>
<acceptance_criteria>
- `agent-rs/src/main.rs` assigns `""` to `system_prompt` when `is_chat_mode` is true.
- `agent-rs/src/main.rs` initializes `messages` as `vec![]` and only pushes the system message if `!system_prompt.is_empty()`.
</acceptance_criteria>
</task>

<must_haves>
- The rust compiler must not output a warning about `res` not needing to be mutable.
- Terminal chat mode (when `exec_mode == "chat"`) must not send a `system` instruction, ensuring identical upstream prompt arrays to the native Web UI.
</must_haves>
