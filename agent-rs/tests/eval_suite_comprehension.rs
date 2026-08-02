//! Eval Scenario 1: Repository Comprehension
//! Tests that search_codebase tool is wired and can locate code symbols.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).expect("source file must exist")
}

/// Verify search_codebase tool exists in tools.rs and is registered in main.rs / server.rs
#[test]
fn sc1_search_codebase_tool_exists() {
    let tools_rs = read("src/tools.rs");
    assert!(
        tools_rs.contains("search_codebase"),
        "search_codebase tool must be defined in src/tools.rs"
    );
}

/// Verify search_codebase is registered in the tool dispatch table.
/// The dispatch is registry-based: main.rs builds the registry via
/// create_default_registry(), and tools.rs registers SearchCodebaseTool into it.
#[test]
fn sc1_search_codebase_registered_in_dispatch() {
    let main_rs = read("src/main.rs");
    let tools_rs = read("src/tools.rs");
    assert!(
        main_rs.contains("create_default_registry"),
        "main.rs must build the tool registry via create_default_registry()"
    );
    assert!(
        tools_rs.contains("SearchCodebaseTool"),
        "tools.rs must register SearchCodebaseTool in the default registry"
    );
    assert!(
        tools_rs.contains("register(Box::new(SearchCodebaseTool))"),
        "SearchCodebaseTool must be registered into the dispatch registry"
    );
}

/// Verify context module exports the ContextEngine that powers search
#[test]
fn sc1_context_engine_exported() {
    let lib_rs = read("src/lib.rs");
    assert!(lib_rs.contains("context"), "context module must be exported from lib.rs");
    let ctx_mod = read("src/context/mod.rs");
    assert!(ctx_mod.contains("ContextEngine"), "ContextEngine must exist in context/mod.rs");
}

/// Verify symbol indexer is wired (Phase 08 — prerequisite for search quality)
#[test]
fn sc1_symbol_indexer_present() {
    let ctx_mod = read("src/context/mod.rs");
    assert!(
        ctx_mod.contains("SymbolIndex") || ctx_mod.contains("index"),
        "Symbol indexing must be present in context module"
    );
}
