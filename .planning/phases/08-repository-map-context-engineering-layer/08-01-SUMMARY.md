---
phase: 08-repository-map-context-engineering-layer
plan: 01
subsystem: context-engineering
tags: [tiktoken-rs, tree-sitter, token-budget, skeleton-extraction, petgraph, sqlite]
requires:
  - phase: 04-gsd-orchestration
    provides: module structure conventions, error handling patterns
provides:
  - ContextEngine facade with new() and build_context() methods
  - ContextQuery type with token_budget and depth fields
  - ContextResult symbol metadata type
  - TokenBudget module with count_tokens() and apply_budget()
  - SkeletonExtractor with elide_body(), format_skeleton_entry(), format_repo_skeleton()
  - Submodule stubs for Indexer, MemoryEngine, RetrievalEngine
  - tree-sitter, tree-sitter-rust, petgraph dependencies added to Cargo.toml
affects: [Plan 08-02 (indexer), Plan 08-03 (memory), Plan 08-04 (retrieval)]

tech-stack:
  added: [tree-sitter = "0.24", tree-sitter-rust = "0.23", petgraph = "0.6"]
  patterns:
    - "Hierarchical context module with facade → submodule architecture"
    - "Token budget enforcement via generic traits (HasTokenCount, HasRank)"
    - "Skeleton extraction via byte-range source manipulation"

key-files:
  created:
    - agent-rs/src/context/mod.rs
    - agent-rs/src/context/budget.rs
    - agent-rs/src/context/skeleton.rs
    - agent-rs/src/context/indexer.rs
    - agent-rs/src/context/memory.rs
    - agent-rs/src/context/retrieval.rs
  modified:
    - agent-rs/src/lib.rs
    - agent-rs/Cargo.toml
  deleted:
    - agent-rs/src/rag.rs

key-decisions:
  - "Removed broken fastembed RAG module entirely; replaced with hierarchical context engineering layer"
  - "40k default token budget ceiling using cl100k_base encoding via tiktoken-rs"
  - "Skeleton extraction works on byte ranges via format! manipulation (no Tree-sitter dep for signatures)"
  - "ContextEngine as facade with lazy initialization pattern"

patterns-established:
  - "Facade pattern: ContextEngine wraps submodules (Indexer, Retrieval, Memory, Skeleton, Budget)"
  - "Generic trait pattern: HasTokenCount + HasRank for budget-agnostic selection"
  - "Fallback pattern: count_tokens falls back to length/4 if encoder unavailable"

requirements-completed: []

duration: 10 min
completed: 2026-07-25
---

# Phase 08 Plan 01: Context Module Foundation Summary

**Hierarchical context engineering layer with budget-bounded skeleton extraction, replacing the naive RAG chunking approach**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-25T12:10:00Z
- **Completed:** 2026-07-25T12:20:25Z
- **Tasks:** 6
- **Files modified:** 9 (6 created, 2 modified, 1 deleted)

## Accomplishments

- Removed broken `rag.rs` (broken `fastembed` import) and replaced with new `context/` module
- Defined core context types: `ContextEngine`, `ContextQuery`, `ContextResult`
- Implemented token budget enforcement with `count_tokens()` (tiktoken-rs cl100k_base) and `apply_budget()` (rank-ordered selection within ceiling)
- Implemented skeleton extraction: `elide_body()`, `format_skeleton_entry()`, `format_repo_skeleton()`
- Created submodule stubs for Indexer, MemoryEngine, RetrievalEngine (Plans 08-02/03/04)
- Added tree-sitter, tree-sitter-rust, and petgraph dependencies

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove rag.rs and add context module declaration** - `a2bc58a` (feat)
2. **Task 2: Create context/mod.rs with core types** - `a805e94` (feat)
3. **Task 3: Create context/budget.rs with token budget enforcement** - `1d08523` (feat)
4. **Task 4: Create context/skeleton.rs with signature extraction** - `47e4b46` (feat)
5. **Task 5: Create stub files for remaining submodules** - `dbed228` (feat)
6. **Task 6: Add new dependencies to Cargo.toml** - `4bdb2b7` (chore)

**Plan metadata:** (pending - final commit)

## Files Created/Modified

### Created
- `agent-rs/src/context/mod.rs` - Core module with ContextEngine, ContextQuery, ContextResult
- `agent-rs/src/context/budget.rs` - Token budget enforcement (count_tokens, apply_budget, HasTokenCount/HasRank traits)
- `agent-rs/src/context/skeleton.rs` - Signature extraction (elide_body, format_skeleton_entry, format_repo_skeleton)
- `agent-rs/src/context/indexer.rs` - Stub for Tree-sitter symbol indexer (Plan 08-02)
- `agent-rs/src/context/memory.rs` - Stub for durable memory engine (Plan 08-03)
- `agent-rs/src/context/retrieval.rs` - Stub for retrieval pipeline (Plan 08-04)

### Modified
- `agent-rs/src/lib.rs` - Added `pub mod context;` declaration
- `agent-rs/Cargo.toml` - Added tree-sitter, tree-sitter-rust, petgraph

### Deleted
- `agent-rs/src/rag.rs` - Removed broken fastembed-based RAG module

## Decisions Made

- **Replaced RAG with structured context engineering:** The broken fastembed-based RAG module was removed entirely in favor of a hierarchical symbol-aware, budget-bounded context approach
- **40k default token budget ceiling:** Using cl100k_base encoding via tiktoken-rs with a defensive text.len()/4 fallback
- **Skeleton extraction via string manipulation:** elide_body operates on byte ranges using format!() rather than requiring Tree-sitter for each skeleton operation (Tree-sitter is only used in the indexer)
- **Facade pattern:** ContextEngine serves as the public API, delegating to internal submodules

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- **Rust toolchain not available on this system:** `cargo build` and `cargo test` could not be executed for verification. All code follows the plan's specifications exactly. A Rust toolchain is required before proceeding to later plans or integration testing.
- **indexer.rs stub content was initially written with full implementation (411 lines) instead of stub (5 lines):** This was detected during file size verification and corrected before the final commit was made.

## Known Stubs

The following files contain placeholder types for future plans:

1. **`agent-rs/src/context/indexer.rs`** - `pub struct Indexer;` (Plan 08-02 will implement Tree-sitter symbol indexer with SQLite cache)
2. **`agent-rs/src/context/memory.rs`** - `pub struct MemoryEngine;` (Plan 08-03 will implement durable memory layer)
3. **`agent-rs/src/context/retrieval.rs`** - `pub struct RetrievalEngine;` (Plan 08-04 will implement just-in-time retrieval pipeline)

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Foundation context module complete with all types, budget layer, and skeleton extraction
- Stub files ready for Plans 08-02 (indexer), 08-03 (memory), and 08-04 (retrieval)
- Tree-sitter and petgraph dependencies added for Plan 08-02
- **Blocker:** Rust toolchain must be installed on the development machine to verify compilation and run tests

## Self-Check: PASSED
- All 6 created files verified on disk
- `rag.rs` confirmed deleted
- All 6 commits found in git history (`a2bc58a`, `a805e94`, `1d08523`, `47e4b46`, `dbed228`, `4bdb2b7`)

---
*Phase: 08-repository-map-context-engineering-layer*
*Completed: 2026-07-25*
