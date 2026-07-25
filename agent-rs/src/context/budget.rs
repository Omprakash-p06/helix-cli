//! Token budget management using tiktoken-rs.
//!
//! Enforces the 40k active token ceiling defined in Phase 08 success metrics.

use tiktoken_rs::cl100k_base;

/// Default token ceiling for a single context payload.
pub const DEFAULT_TOKEN_BUDGET: usize = 40_000;

/// Estimates token count for an arbitrary string using cl100k_base encoding.
///
/// Falls back to `text.len() / 4` if the encoder cannot be loaded (defensive fallback).
pub fn count_tokens(text: &str) -> usize {
    match cl100k_base() {
        Ok(bpe) => bpe.encode_with_special_tokens(text).len(),
        Err(_) => text.len() / 4, // Conservative estimate: 4 chars ≈ 1 token
    }
}

/// Selects items from `candidates` in priority order (highest rank first)
/// until the running token total would exceed `budget`.
///
/// Returns the selected items and the final token total.
pub fn apply_budget<T: HasTokenCount + HasRank>(
    mut candidates: Vec<T>,
    budget: usize,
) -> (Vec<T>, usize) {
    candidates.sort_by(|a, b| b.rank().partial_cmp(&a.rank()).unwrap_or(std::cmp::Ordering::Equal));

    let mut selected = Vec::new();
    let mut total_tokens = 0usize;

    for item in candidates {
        let item_tokens = item.token_count();
        if total_tokens + item_tokens <= budget {
            total_tokens += item_tokens;
            selected.push(item);
        }
        // If a single item exceeds budget, skip it
    }

    (selected, total_tokens)
}

/// Trait for items that carry a token count.
pub trait HasTokenCount {
    fn token_count(&self) -> usize;
}

/// Trait for items that carry a relevance rank.
pub trait HasRank {
    fn rank(&self) -> f32;
}

// Implement for ContextResult
impl HasTokenCount for crate::context::ContextResult {
    fn token_count(&self) -> usize {
        self.token_count
    }
}

impl HasRank for crate::context::ContextResult {
    fn rank(&self) -> f32 {
        self.rank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_non_empty() {
        let count = count_tokens("fn main() { println!(\"hello\"); }");
        assert!(count > 0, "Token count must be >0 for non-empty input");
    }

    #[test]
    fn test_count_tokens_empty() {
        assert_eq!(count_tokens(""), 0, "Empty string must be 0 tokens");
    }

    #[test]
    fn test_budget_enforced_ceiling() {
        use crate::context::ContextResult;
        // Create a result that would be 100 tokens; budget is 50 tokens → should be excluded
        let large = ContextResult {
            file_path: "src/main.rs".into(),
            symbol_name: "main".into(),
            symbol_kind: "fn".into(),
            content: "a".repeat(400), // ~100 tokens
            rank: 1.0,
            line_range: (1, 5),
            token_count: 100,
        };
        let (selected, total) = apply_budget(vec![large], 50);
        assert_eq!(selected.len(), 0, "Item exceeding budget must be excluded");
        assert_eq!(total, 0);
    }
}
