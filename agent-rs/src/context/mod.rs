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

// Re-export IndexStats for public access
pub use crate::context::indexer::IndexStats;

use crate::context::indexer::Indexer;
use crate::context::memory::MemoryEngine;
use crate::context::retrieval::{RetrievalEngine, format_results};
use rusqlite::Connection;

/// The central facade for all context operations.
///
/// Lazily initialized — call `ContextEngine::new()` once at startup;
/// then call `initialize()` to build/restore the index and enable memory.
pub struct ContextEngine {
    pub db_path: std::path::PathBuf,
    pub workspace_root: std::path::PathBuf,
    pub indexer: Option<Indexer>,
    pub memory: Option<MemoryEngine>,
}

impl ContextEngine {
    /// Open or create a context engine.
    /// Call `initialize()` after `new()` to build the index.
    pub fn new(db_path: impl Into<std::path::PathBuf>, workspace_root: impl Into<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db_path: db_path.into(),
            workspace_root: workspace_root.into(),
            indexer: None,
            memory: None,
        })
    }

    /// Initialize: open SQLite connections, build/restore symbol index, enable memory layer.
    /// Should be called once at agent startup.
    pub fn initialize(&mut self) -> Result<IndexStats, Box<dyn std::error::Error>> {
        // Create .helix/ directory if needed
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Indexer uses a separate "symbols" table namespace in the same DB
        let conn_idx = Connection::open(&self.db_path)?;
        let mut indexer = Indexer::new(conn_idx, &self.workspace_root)?;
        let stats = indexer.index_workspace()?;
        self.indexer = Some(indexer);

        // Memory engine
        let conn_mem = Connection::open(&self.db_path)?;
        self.memory = Some(MemoryEngine::new(conn_mem)?);

        Ok(stats)
    }

    /// Build a context string for `query` within the specified token budget.
    pub fn build_context(&self, query: &ContextQuery) -> Result<String, Box<dyn std::error::Error>> {
        match &self.indexer {
            Some(idx) => {
                let engine = RetrievalEngine::new(idx);
                let results = engine.search(query);
                Ok(format_results(&results))
            }
            None => Ok("[Context engine not initialized. Call initialize() first.]".to_string()),
        }
    }

    /// Build a repo skeleton (all symbols, Level 0) for session-start injection.
    pub fn build_repo_skeleton(&self, token_budget: usize) -> String {
        match &self.indexer {
            Some(idx) => {
                let engine = RetrievalEngine::new(idx);
                engine.build_repo_skeleton(token_budget)
            }
            None => "[Index not loaded]".to_string(),
        }
    }
}
