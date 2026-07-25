//! Just-in-time context retrieval pipeline.
//!
//! Retrieval stages (in order):
//! 1. Exact symbol name match (fastest — SQLite index lookup)
//! 2. Substring symbol search (LIKE query, case-insensitive)
//! 3. Budget-aware skeleton assembly (tiktoken-counted, priority-ranked)
//!
//! Returns formatted context string ready for LLM injection.

use crate::context::{ContextQuery, ContextResult};
use crate::context::indexer::Indexer;
use crate::context::budget::{count_tokens, DEFAULT_TOKEN_BUDGET};
use crate::context::skeleton::{format_skeleton_entry, format_repo_skeleton};

/// The retrieval engine — wraps Indexer and applies budget-bounded selection.
pub struct RetrievalEngine<'a> {
    indexer: &'a Indexer,
}

impl<'a> RetrievalEngine<'a> {
    pub fn new(indexer: &'a Indexer) -> Self {
        Self { indexer }
    }

    /// Search the codebase for symbols matching `query` within `budget` tokens.
    ///
    /// Returns a list of `ContextResult` items, ranked by relevance,
    /// guaranteed to fit within the token budget.
    pub fn search(&self, query: &ContextQuery) -> Vec<ContextResult> {
        let budget = query.token_budget.min(DEFAULT_TOKEN_BUDGET);
        let mut candidates: Vec<ContextResult> = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        // Stage 1: Exact name match (highest priority)
        for sym in self.indexer.find_symbol(&query.query) {
            if seen_names.insert(format!("{}::{}", sym.file_path, sym.symbol_name)) {
                let token_count = count_tokens(&sym.signature);
                candidates.push(ContextResult {
                    file_path: sym.file_path,
                    symbol_name: sym.symbol_name,
                    symbol_kind: sym.symbol_kind,
                    content: sym.signature,
                    rank: 10.0, // Highest rank for exact matches
                    line_range: (sym.line_start, sym.line_end),
                    token_count,
                });
            }
        }

        // Stage 2: Substring search (lower priority)
        for sym in self.indexer.search_symbols(&query.query) {
            let key = format!("{}::{}", sym.file_path, sym.symbol_name);
            if seen_names.insert(key) {
                let token_count = count_tokens(&sym.signature);
                candidates.push(ContextResult {
                    file_path: sym.file_path,
                    symbol_name: sym.symbol_name,
                    symbol_kind: sym.symbol_kind,
                    content: sym.signature,
                    rank: 5.0, // Lower rank for substring matches
                    line_range: (sym.line_start, sym.line_end),
                    token_count,
                });
            }
        }

        // Stage 3: Apply token budget (highest rank first)
        let (selected, _total_tokens) = crate::context::budget::apply_budget(candidates, budget);
        selected
    }

    /// Build a repo skeleton: all symbols at Level 0 (signatures only), within budget.
    ///
    /// Used for automatic pre-injection at session start.
    /// Returns formatted string with skeleton header.
    pub fn build_repo_skeleton(&self, token_budget: usize) -> String {
        let all_symbols = self.indexer.list_all_symbols();
        let total_symbols = all_symbols.len();

        let mut entries = Vec::new();
        let mut remaining_budget = token_budget;

        for (file_path, symbol_name) in &all_symbols {
            let syms = self.indexer.find_symbol(symbol_name);
            for sym in syms {
                if &sym.file_path != file_path {
                    continue;
                }
                let entry = format_skeleton_entry(
                    &sym.file_path,
                    sym.line_start,
                    sym.line_end,
                    &sym.signature,
                );
                let tok = count_tokens(&entry);
                if tok <= remaining_budget {
                    remaining_budget -= tok;
                    entries.push(entry);
                }
                if remaining_budget == 0 {
                    break;
                }
            }
            if remaining_budget == 0 {
                break;
            }
        }

        format_repo_skeleton(&entries, total_symbols)
    }
}

/// Format retrieval results as a string for LLM context injection.
pub fn format_results(results: &[ContextResult]) -> String {
    if results.is_empty() {
        return "No matching symbols found in the codebase.\n".to_string();
    }
    let mut out = format!("=== CODEBASE SEARCH RESULTS ({} symbols) ===\n", results.len());
    for r in results {
        out.push_str(&format!(
            "\n// {}:{}-{} [{}]\n{}\n",
            r.file_path, r.line_range.0, r.line_range.1, r.symbol_kind, r.content
        ));
    }
    out.push_str("============================================\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::indexer::{Indexer, parse_rust_file};
    use rusqlite::Connection;

    fn make_indexer_with_source(source: &str) -> Indexer {
        let conn = Connection::open_in_memory().unwrap();
        let mut indexer = Indexer::new(conn, ".").unwrap();
        let symbols = parse_rust_file(source, "src/test_file.rs");
        indexer.store_file_symbols_pub("src/test_file.rs", "test_hash", &symbols).unwrap();
        indexer
    }

    #[test]
    fn test_exact_match_returns_highest_rank() {
        let source = "pub fn target_fn(x: u32) -> u32 { x }";
        let indexer = make_indexer_with_source(source);
        let engine = RetrievalEngine::new(&indexer);
        let query = ContextQuery {
            query: "target_fn".into(),
            token_budget: 40_000,
            depth: 0,
        };
        let results = engine.search(&query);
        assert!(!results.is_empty(), "Must find target_fn");
        assert_eq!(results[0].symbol_name, "target_fn");
        assert!(results[0].rank >= 10.0, "Exact match must have rank >= 10.0");
    }

    #[test]
    fn test_budget_limits_results() {
        let source = r#"
pub fn fn_one(x: u32) -> u32 { x }
pub fn fn_two(x: u32) -> u32 { x }
pub fn fn_three(x: u32) -> u32 { x }
"#;
        let indexer = make_indexer_with_source(source);
        let engine = RetrievalEngine::new(&indexer);
        // Very tight budget — only 1 token (nothing should fit except tiny signatures)
        let query = ContextQuery {
            query: "fn".into(),
            token_budget: 1,
            depth: 0,
        };
        let results = engine.search(&query);
        // With a 1-token budget, no signatures should fit
        for r in &results {
            assert!(r.token_count <= 1, "Each result must fit within budget");
        }
    }

    #[test]
    fn test_format_results_non_empty() {
        let result = ContextResult {
            file_path: "src/tools.rs".into(),
            symbol_name: "execute".into(),
            symbol_kind: "fn".into(),
            content: "fn execute() { /* ... */ }".into(),
            rank: 10.0,
            line_range: (1, 5),
            token_count: 8,
        };
        let formatted = format_results(&[result]);
        assert!(formatted.contains("src/tools.rs"), "Must contain file path");
        assert!(formatted.contains("execute"), "Must contain symbol name");
    }
}
