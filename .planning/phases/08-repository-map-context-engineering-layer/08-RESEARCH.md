# Phase 08: Repository Map & Context Engineering Layer — Research

**Researched:** 2026-07-25
**Phase Goal:** Build a hierarchical context system — Tree-sitter/LSP symbol extraction, dependency graph, just-in-time retrieval, and a durable agent memory layer — so the agent can reason over large codebases without concatenating raw files into the prompt.

---

## Executive Summary

- **Tree-sitter is the right tool** for symbol extraction in Rust. The `tree-sitter` (v0.24) + `tree-sitter-rust` (v0.23) crate combination uses an S-expression query DSL to extract `function_item`, `struct_item`, `impl_item`, and `use_declaration` nodes with sub-millisecond parse times. Superior to `syn` because it handles broken/incomplete code.
- **Aider repomap approach (AST + PageRank) beats naive RAG** for code understanding. Instead of chunking raw text, we build a symbol-to-symbol reference graph, rank nodes by PageRank, and emit only signatures (not bodies) within a configurable token budget.
- **`petgraph` (DiGraph) is the standard Rust graph library** for storing call graphs and module dependency relationships. Use `StableGraph` for structures where nodes can be deleted.
- **SQLite with FTS5 is the correct backing store** for the durable agent memory layer. `rusqlite` is already a project dependency. FTS5 provides sub-10ms full-text search. WAL mode enables concurrent reads.
- **The existing `src/rag.rs` must be replaced, not augmented.** It uses naive character-chunking with a broken `fastembed` import. Phase 08 builds a proper `src/context/` module.
- **Three success metric targets drive the architecture**: <3s symbol lookup via SQLite index + petgraph; <40k tokens via budget-aware skeleton extraction; durable state via SQLite sessions table with FTS5.

---

## Technology Recommendations

| Concern | Recommended Crate | Rationale |
|---------|-------------------|-----------|
| Symbol extraction | `tree-sitter = "0.24"` + `tree-sitter-rust = "0.23"` | Fast, query-DSL, handles broken code |
| Dependency graph | `petgraph = "0.6"` | Standard Rust graph lib, DOT export |
| Incremental hashing | `sha2` (already in Cargo.toml) | SHA-256 of file content for cache invalidation |
| Memory store | `rusqlite` FTS5 (already in Cargo.toml) | Persistent, fast keyword search, no extra dep |
| Token counting | `tiktoken-rs` (already in Cargo.toml) | Budget enforcement in retrieval pipeline |

**No new mandatory dependencies** except `tree-sitter` and `tree-sitter-rust`. All other building blocks are already in the project.

---

## Detailed Findings

### 1. Tree-sitter Symbol Extraction

**Crate:** `tree-sitter = "0.24"` + `tree-sitter-rust = "0.23"`

Tree-sitter is an incremental, error-recovering parser. It **tolerates incomplete or broken code** — inserts ERROR nodes but continues parsing. Essential for live codebases under edit.

**Key node types in the Rust grammar:**

| Symbol Type | Tree-sitter Node | Query Pattern |
|-------------|------------------|---------------|
| Function | `function_item` | `(function_item name: (identifier) @name)` |
| Struct | `struct_item` | `(struct_item name: (type_identifier) @name)` |
| Enum | `enum_item` | `(enum_item name: (type_identifier) @name)` |
| Trait | `trait_item` | `(trait_item name: (type_identifier) @name)` |
| Impl block | `impl_item` | `(impl_item type: (_) @type)` |
| Use declaration | `use_declaration` | `(use_declaration argument: (_) @path)` |
| Mod declaration | `mod_item` | `(mod_item name: (identifier) @name)` |

**Skeleton extraction:** Use Tree-sitter captures to get function signature span (name + params + return type), then exclude the `body: (block) @body` capture. This gives signatures without bodies — ~10-20 tokens vs 50-500 tokens for full functions.

**Performance:** Tree-sitter parses ~1MB of Rust in ~100ms. Incremental reparse via `parser.parse(new_source, Some(old_tree))` reduces re-parse to microseconds for single-function edits.

**Limitations:**
- Does NOT resolve names semantically (cannot tell if `foo::bar` in file A refers to `mod foo` in file B without a second resolution pass)
- Only need `tree-sitter-rust` grammar for this project (Rust-only scope for Phase 08)

**Alternative considered: `syn` crate** — requires valid, complete Rust. Fails on macro-heavy code. Tree-sitter is strictly better for this use case.

---

### 2. Dependency Graph Construction

**Graph schema using `petgraph::graph::DiGraph`:**

```
Nodes: SymbolNode { file_path, symbol_name, symbol_kind, signature, line_range, doc_comment }
Edges: ReferenceEdge { kind: "calls" | "imports" | "implements" | "defines_in" }
```

**Building edges from Tree-sitter:**
1. **Import edges:** Parse `use_declaration` nodes → `file A` imports path → add edge `A → B`
2. **Call edges:** Parse `call_expression` nodes → add edge `caller → callee`
3. **Impl edges:** Parse `impl_item` → `impl Trait for Type` → add edge `Type → Trait`

**Incremental update strategy:**
- Store SHA-256(file_content) in SQLite `symbol_cache` table
- On re-index: compare current hash vs stored; if unchanged skip; if changed remove old nodes and re-add

Use `StableGraph` (not `Graph`) so node indices remain valid when removing stale file nodes.

**PageRank for relevance ranking:**
- Use `petgraph::algo::page_rank()`
- Files/symbols with many incoming edges (heavily referenced) get higher PageRank
- When building context, sort by PageRank × relevance_score, pick top-N within token budget

**DOT export for architecture visualization** (satisfies ROADMAP Global Process Standard #3):
```rust
use petgraph::dot::{Dot, Config};
let dot = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
// Write to misc/architecture_YYYY-MM-DD.svg via graphviz pipeline
```

---

### 3. Just-in-Time Context Retrieval

**Strategy: Hybrid Structural + Keyword retrieval (Aider-inspired)**

The Aider repo map approach outperforms naive RAG for code because structure (call relationships, signatures) is more useful than raw text similarity.

**Retrieval pipeline:**
```
Query/Task
  → [1] Symbol Name Matching (exact string match in symbol_cache)
  → [2] BM25 over signatures (SQLite FTS5)
  → [3] Graph traversal (callers/callees of matched symbols via petgraph)
  → [4] PageRank ranking of candidates
  → [5] Budget-aware skeleton emission (signatures only, tiktoken budget)
  → Context payload <= 40k tokens
```

**Context budget management:**
- Use `tiktoken-rs` to count tokens before inserting into context
- Emit in priority order: exact matches → PageRank-ranked related → fallback file summaries
- Skeleton mode: emit `fn signature(args) -> ReturnType { /* ... */ }` instead of full bodies
  - ~10-20 tokens per function vs 50-500 tokens for full bodies
  - Allows ~2000 function signatures in 40k tokens vs ~200 full functions

**Context distillation levels:**
1. **Level 0 — Symbol skeleton:** Signature + doc comment only (10-30 tokens)
2. **Level 1 — Full function:** Complete source (50-500 tokens)
3. **Level 2 — File summary:** Top-level symbols only

**Tool vs. automatic injection:**
- **BOTH.** Pre-inject compact repo skeleton (Level 0 for all symbols, ~5k tokens) at session start
- Expose `search_codebase(query: str, depth: int)` tool the LLM can call for Level 1 detail on demand
- Mirrors Aider's approach: auto-inject repomap + allow drill-down

---

### 4. Durable Agent Memory Layer

**Backing store: SQLite with FTS5 (via existing `rusqlite` dependency)**

**Schema design:**

```sql
-- Session state: survives compaction/context reset
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    goal        TEXT,
    status      TEXT DEFAULT 'active'
);

-- Durable memory entries
CREATE TABLE memory (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    kind        TEXT NOT NULL,  -- 'goal'|'constraint'|'decision'|'failure'|'edit'|'observation'
    content     TEXT NOT NULL,
    metadata    TEXT,           -- JSON blob
    created_at  INTEGER NOT NULL,
    importance  REAL DEFAULT 1.0
);

-- FTS5 virtual table for fast keyword search
CREATE VIRTUAL TABLE memory_fts USING fts5(
    content, kind,
    content='memory', content_rowid='id'
);

-- Edit ledger: immutable record of all file changes
CREATE TABLE edit_ledger (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL,
    file_path   TEXT NOT NULL,
    edit_kind   TEXT NOT NULL,  -- 'insert'|'delete'|'replace'
    diff_patch  TEXT,
    applied_at  INTEGER NOT NULL,
    reverted    INTEGER DEFAULT 0
);

-- Symbol index cache for incremental re-indexing
CREATE TABLE symbol_cache (
    file_path   TEXT NOT NULL PRIMARY KEY,
    file_hash   TEXT NOT NULL,
    symbols_json TEXT NOT NULL,
    indexed_at  INTEGER NOT NULL
);
```

**WAL mode (mandatory for concurrent access):**
```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
```

**Compaction cycle survival:** LLM context reset = compaction cycle. On resume:
1. Load `sessions` row → goal + status
2. Query `memory WHERE session_id = ? ORDER BY importance DESC LIMIT 50`
3. Load recent entries from `edit_ledger` for file change context
4. Re-inject as compact "session resume" block at top of new context

**Memory DB location:** Per-workspace `.helix/helix_context.db` (not user-global, to avoid cross-project privacy issues)

---

### 5. Integration Points

**Replace `src/rag.rs`:** The current rag.rs has a broken `fastembed` import (not in Cargo.toml), uses character-chunking with no structural awareness, no persistence. Phase 08 replaces with `src/context/` module.

**CRITICAL: Remove `use fastembed::...` from rag.rs BEFORE adding tree-sitter — otherwise the build will fail.**

**New module structure:**
```
src/context/
  mod.rs        -- Public API: ContextEngine, ContextQuery, ContextResult
  indexer.rs    -- Tree-sitter parse + symbol extraction + petgraph graph build
  retrieval.rs  -- JIT retrieval pipeline (exact match → FTS5 → graph → budget)
  memory.rs     -- SQLite memory layer (sessions, memory, edit_ledger)
  skeleton.rs   -- Distillation: extract signatures without bodies
  budget.rs     -- Token budget management using tiktoken-rs
```

**Integration with cognitive loop in `src/main.rs`:**
- At startup: `ContextEngine::new(db_path, workspace_root)` → loads/validates index
- Before LLM call: `engine.build_context(task, budget_tokens)` → returns formatted context
- After tool execution: `engine.memory.record_edit(file_path, diff)` → persists to edit_ledger
- On context reset: `engine.memory.resume_session(session_id)` → reconstructs compact state

**Tool JSON schema:**
```json
{
  "name": "search_codebase",
  "description": "Search the codebase for symbols, functions, or definitions. Returns ranked results with signatures and file locations.",
  "parameters": {
    "query": { "type": "string", "description": "Symbol name or natural language description" },
    "depth": { "type": "integer", "description": "0=signatures only (fast), 1=full bodies (detailed)", "default": 0 }
  }
}
```

**Impact on audit log:** The context DB (`.helix/helix_context.db`) is a SEPARATE file from the existing audit SQLite DB. No schema conflicts.

---

## Validation Architecture

> Required for Nyquist validation (Dimension 8).

### Test Suite

**1. Symbol Lookup Latency Benchmark (SLA: <3 seconds)**
```rust
#[test]
fn benchmark_symbol_lookup_under_3s() {
    let engine = ContextEngine::open_with_test_index("./agent-rs/src");
    let start = std::time::Instant::now();
    let result = engine.find_symbol("ToolRuntime");
    let elapsed = start.elapsed();
    assert!(result.is_some(), "Symbol must be found");
    assert!(elapsed.as_secs() < 3, "Must be <3s, got {:?}", elapsed);
}
```

**2. Context Budget Enforcement (SLA: <=40k tokens)**
```rust
#[test]
fn test_context_budget_enforced() {
    let engine = ContextEngine::open_with_test_index("./agent-rs/src");
    let context = engine.build_context("repair memory leak in session handler", 40_000);
    let token_count = count_tokens(&context);
    assert!(token_count <= 40_000, "Context must be <=40k tokens, got {}", token_count);
}
```

**3. Durable State Survives Compaction Cycles**
```rust
#[test]
fn test_session_survives_compaction() {
    let db = tempfile::NamedTempFile::new().unwrap();
    // Phase 1: write state
    {
        let engine = ContextEngine::new(db.path(), ".");
        let sid = engine.memory.new_session("Fix memory leak in session.rs");
        engine.memory.record(sid, MemoryKind::Constraint, "Never modify audit schema");
        engine.memory.record(sid, MemoryKind::Failure, "Attempt 1: Arc<Mutex<>> race condition");
    }  // Drop = simulated compaction
    // Phase 2: resume in fresh engine instance
    {
        let engine = ContextEngine::new(db.path(), ".");
        let sessions = engine.memory.list_active_sessions();
        assert_eq!(sessions.len(), 1);
        let memories = engine.memory.load_session(sessions[0].id);
        assert!(memories.iter().any(|m| m.kind == MemoryKind::Constraint));
        assert!(memories.iter().any(|m| m.kind == MemoryKind::Failure));
    }
}
```

**4. Adversarial Test Cases**
```rust
#[test] fn test_circular_import_no_panic() { /* index files with A imports B, B imports A */ }
#[test] fn test_large_file_indexed_under_5s() { /* benchmark 103KB main.rs parse time */ }
#[test] fn test_duplicate_symbols_distinguished() { /* same fn name in 2 modules → 2 results */ }
#[test] fn test_partial_code_no_panic() { /* parse truncated/broken Rust source */ }
```

### Verification Commands
```bash
cargo test context:: -- --nocapture
cargo test --release -- benchmark -- --nocapture
cargo clippy -p agent-rs -- -D warnings
```

---

## Open Questions

1. **Multi-language scope:** Index Rust only (`agent-rs/src/`) or also `web-ui/` (JS) and `scripts/` (Python)? **Recommendation:** Rust-only for Phase 08.
2. **Vector embeddings:** Include semantic (vector) search or rely on BM25 + structural graph? **Recommendation:** FTS5 + structural graph only; meets success metrics without fastembed complexity.
3. **Memory DB location:** Per-workspace `.helix/helix_context.db` or user-global `~/.helix/`? **Recommendation:** Per-workspace for Phase 08.
4. **LLM tool vs. auto-injection:** **Recommendation:** Both — auto-inject skeleton at session start, expose tool for drill-down.

---

## Implementation Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| `tree-sitter-rust` grammar gaps for Rust 2024 syntax | Medium | Pin to tested version; add explicit error tests |
| Broken `fastembed` import in rag.rs causes compile failure | **HIGH** | Remove `use fastembed::...` FIRST before any new deps |
| Petgraph PageRank slow on large graphs | Low | Limit graph to same-workspace symbols; use sparse representation |
| SQLite WAL + concurrent access from multiple agent threads | Medium | Single-writer connection pattern or `r2d2_sqlite` pool |
| Replacing rag.rs breaks existing callers in main.rs | **HIGH** | Audit all `rag` usages in main.rs before removal; provide shim |
| Token counting overhead on every retrieval call | Low | Cache token counts alongside signatures in symbol_cache table |

---

*Research complete. Ready for planning.*
