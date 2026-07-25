---
phase: 08-repository-map-context-engineering-layer
plan: 05
subsystem: context
tags: indexer, imports, dependency-graph, dot, tree-sitter
requires:
  - phase: 08-02
    provides: Tree-sitter symbol indexer (Indexer, parse_rust_file, symbol cache)
  - phase: 08-04
    provides: ContextEngine wiring, export_dot_graph method
provides:
  - extract_import_edges() function using Tree-sitter use_declaration query
  - import_edges table populated during index_workspace indexing
  - DOT graph with meaningful dependency edges instead of empty graph
  - 3 unit tests validating edge extraction, no-import edge case, and stored edge verification
affects: main, tools, server
tech-stack:
  added: none
  patterns: Tree-sitter query for import extraction, dependency graph persistence
key-files:
  modified:
    - agent-rs/src/context/indexer.rs — added extract_import_edges, wired store_file_symbols, 3 tests
    - agent-rs/src/context/retrieval.rs — updated make_indexer_with_source signature
key-decisions:
  - "extract_import_edges uses a separate Tree-sitter query rather than reusing parse_rust_file's combined query, keeping concerns separated"
  - "Import edges are persisted as (from_file, to_path) pairs in the import_edges table alongside symbol rows in the same transaction"
requirements-completed: []
duration: 5min
completed: 2026-07-25
---

# Plan 08-05: import_edges Table Population Summary

**Tree-sitter-based import edge extraction populating the dependency graph with meaningful edges from Rust use declarations**

## Performance

- **Duration:** 5 min
- **Tasks:** 2 (code + tests already committed; SUMMARY.md created)
- **Files modified:** 2

## Accomplishments
- `extract_import_edges()` — parses Rust source with Tree-sitter, queries `use_declaration` nodes, returns `(from_file, to_path)` pairs
- `store_file_symbols()` updated to accept and persist import edges in the same transaction
- `export_dot_graph()` now renders meaningful arrow edges from the import_edges table
- 3 tests: basic extraction, no-imports edge case, stored edges verified via DOT output

## Task Commits

1. **Task 1: Add extract_import_edges + wire store_file_symbols** — `d03a2fb` (fix)
2. **Task 2: Add 3 tests** — `d03a2fb` (same commit)

## Files Modified
- `agent-rs/src/context/indexer.rs` — added `extract_import_edges()`, updated `store_file_symbols()` signature and body, added 3 tests
- `agent-rs/src/context/retrieval.rs` — updated `make_indexer_with_source()` test helper to pass empty import_edges

## Decisions & Deviations
- No deviations — plan executed exactly as written
- `extract_import_edges` kept as a standalone function (not a method on Indexer) for testability; the same pattern as `parse_rust_file`

## Gap Closure Impact
- **Gap 1 (import_edges never populated):** Closed — `index_workspace()` now calls `extract_import_edges()` and passes results to `store_file_symbols()`
- **Gap 2 (DOT exports empty graph):** Closed — `export_dot_graph()` queries populated import_edges table; `test_import_edges_stored_and_queried` validates it contains arrows

## Next Phase Readiness
- Phase 08 dependency graph is complete with real import edges
- Ready for Phase 08 verification pass (expect `status: passed`)
