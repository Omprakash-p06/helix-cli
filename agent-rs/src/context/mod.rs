//! Hierarchical context engineering layer for Helix Agent.
//!
//! Provides symbol-aware, budget-bounded context retrieval from the local
//! workspace, replacing the naive character-chunking approach in the removed rag.rs.
//!
//! # Architecture
//! ```text
//! ContextEngine
//!   ├── Indexer     (Tree-sitter → SQLite symbol_cache)
//!   ├── Retrieval   (Exact + FTS5 + Graph → Budget-bounded results)
//!   ├── Memory      (SQLite sessions/memory/edit_ledger for durable state)
//!   ├── Skeleton    (Signature extraction without function bodies)
//!   └── Budget      (tiktoken-rs token counting + ceiling enforcement)
//! ```

pub mod budget;
pub mod indexer;
pub mod memory;
pub mod retrieval;
pub mod skeleton;

use serde::{Deserialize, Serialize};

/// A query issued to the context engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuery {
    /// Natural language description of the task or symbol being searched.
    pub query: String,
    /// Maximum tokens allowed in the returned context (default: 40_000).
    pub token_budget: usize,
    /// 0 = signatures only (fast), 1 = full bodies (detailed).
    pub depth: u8,
}

impl Default for ContextQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            token_budget: 40_000,
            depth: 0,
        }
    }
}

/// A single context result item — a symbol with its location and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResult {
    /// Relative path from workspace root.
    pub file_path: String,
    /// Fully-qualified symbol name (e.g., "ToolRuntime::execute").
    pub symbol_name: String,
    /// Human-readable kind: "fn", "struct", "enum", "trait", "impl", "mod".
    pub symbol_kind: String,
    /// Signature without body (Level 0), or full source (Level 1).
    pub content: String,
    /// Relevance rank (higher = more relevant). PageRank × match score.
    pub rank: f32,
    /// Line range in source file (start, end), 1-indexed.
    pub line_range: (u32, u32),
    /// Estimated token count of `content`.
    pub token_count: usize,
}

/// The central facade for all context operations.
///
/// Lazily initialized — call `ContextEngine::new()` once at startup;
/// the index is built/restored from SQLite cache automatically.
pub struct ContextEngine {
    pub db_path: std::path::PathBuf,
    pub workspace_root: std::path::PathBuf,
}

impl ContextEngine {
    /// Open or create a context engine backed by `db_path`.
    /// `workspace_root` is the directory to index (e.g., `./agent-rs/src`).
    pub fn new(db_path: impl Into<std::path::PathBuf>, workspace_root: impl Into<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db_path: db_path.into(),
            workspace_root: workspace_root.into(),
        })
    }

    /// Build context for `query` within `token_budget` tokens.
    /// Returns a formatted string ready for LLM injection.
    pub fn build_context(&self, query: &ContextQuery) -> Result<String, Box<dyn std::error::Error>> {
        // Delegated to retrieval module (implemented in Plan 08-04)
        Ok(format!("[Context placeholder for: {}]", query.query))
    }
}
