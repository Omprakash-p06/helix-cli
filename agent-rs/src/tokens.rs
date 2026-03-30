use crate::ChatMessage;
use tiktoken_rs::cl100k_base;

/// Calculates exactly how many GPT tokens a string consumes.
/// Uses OpenAI's standard `cl100k_base` vocabulary which proxies very closely to llamas.
pub fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_with_special_tokens(text).len()
}

/// Evaluates the entire ChatMessage payload sizes in token density.
pub fn count_message_tokens(messages: &[ChatMessage]) -> usize {
    let mut total_tokens = 0;
    for msg in messages {
        // Count role text
        total_tokens += count_tokens(&msg.role);

        // Count content text
        if let Some(content) = &msg.content {
            total_tokens += count_tokens(content);
        }

        // Add 5 rough tokens per structural metadata boundaries (role/name keys, etc.)
        total_tokens += 5;
    }

    // Add additional base overhead padding
    total_tokens + 10
}
