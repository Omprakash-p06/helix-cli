pub mod config;
pub mod input;
pub mod session;
pub mod security;
pub mod server;
pub mod stream;
pub mod tokens;
pub mod tools;
pub mod tui;
pub mod types;
pub mod utils;
pub mod runtime_profile;
pub mod watchdog;
pub mod audit;
pub mod agent_core;

pub use types::{ChatMessage, ChatResponse, Choice, ServerFlavor};

/// Common function to create a critic message directive.
pub fn critic_message(text: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: Some(format!("[Rust Critic] {}", text)),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

/// Exposes internal `<think>` blocks by mapping them to `<thinking>` tags for the UI,
/// ensuring the user can see the agent's internal reasoning.
pub fn expose_think_blocks(text: &str) -> String {
    let mut s = text.replace("<think>", "\n<thinking>\n");
    s = s.replace("</think>", "\n</thinking>\n");
    if s.contains("<thinking>") && !s.contains("</thinking>") {
        s.push_str("\n</thinking>");
    }
    s.trim().to_string()
}
