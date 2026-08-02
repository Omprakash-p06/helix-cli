//! Tree-sitter symbol indexer and SQLite symbol cache.
//!
//! Uses `tree-sitter-rust` grammar to extract symbols from `.rs` files,
//! with SHA-256-based incremental invalidation to avoid re-parsing unchanged files.

use rusqlite::{Connection, Result as SqlResult, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// A single extracted symbol from the codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolNode {
    pub file_path: String,
    pub symbol_name: String,
    pub symbol_kind: String,   // "fn" | "struct" | "enum" | "trait" | "impl" | "mod" | "use"
    pub signature: String,     // Body-elided source (Level 0 skeleton)
    pub line_start: u32,
    pub line_end: u32,
    pub doc_comment: Option<String>,
}

/// The main indexer — builds and queries the symbol cache.
pub struct Indexer {
    conn: Connection,
    workspace_root: PathBuf,
}

impl Indexer {
    /// SQL schema for the symbol cache and import edge tables.
    const SCHEMA: &'static str = r#"
        CREATE TABLE IF NOT EXISTS symbol_cache (
            file_path    TEXT NOT NULL,
            file_hash    TEXT NOT NULL,
            indexed_at   INTEGER NOT NULL,
            PRIMARY KEY  (file_path)
        );

        CREATE TABLE IF NOT EXISTS symbols (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path    TEXT NOT NULL,
            symbol_name  TEXT NOT NULL,
            symbol_kind  TEXT NOT NULL,
            signature    TEXT NOT NULL,
            line_start   INTEGER NOT NULL,
            line_end     INTEGER NOT NULL,
            doc_comment  TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(symbol_name);
        CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path);

        CREATE TABLE IF NOT EXISTS import_edges (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            from_file    TEXT NOT NULL,
            to_path      TEXT NOT NULL
        );

        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
    "#;

    /// Open or create an indexer backed by the given SQLite connection.
    pub fn new(conn: Connection, workspace_root: impl Into<PathBuf>) -> SqlResult<Self> {
        conn.execute_batch(Self::SCHEMA)?;
        Ok(Self {
            conn,
            workspace_root: workspace_root.into(),
        })
    }

    /// Walk the workspace and index all `.rs` files, skipping unchanged files.
    pub fn index_workspace(&mut self) -> Result<IndexStats, Box<dyn std::error::Error>> {
        let mut stats = IndexStats::default();
        let walker = ignore::WalkBuilder::new(&self.workspace_root)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry in walker.flatten() {
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let path_str = path.to_string_lossy().to_string();
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let hash = sha256_hex(&source);
            if self.is_cached(&path_str, &hash)? {
                stats.skipped += 1;
                continue;
            }
            let symbols = parse_rust_file(&source, &path_str);
            let import_edges = extract_import_edges(&source, &path_str);
            self.store_file_symbols(&path_str, &hash, &symbols, &import_edges)?;
            stats.indexed += 1;
            stats.symbols_found += symbols.len();
        }
        Ok(stats)
    }

    /// Find symbols by name (exact match). Fast: uses SQLite index on `symbol_name`.
    pub fn find_symbol(&self, name: &str) -> Vec<SymbolNode> {
        let mut stmt = match self.conn.prepare(
            "SELECT file_path, symbol_name, symbol_kind, signature, line_start, line_end, doc_comment
             FROM symbols WHERE symbol_name = ?1"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map(params![name], |row| {
            Ok(SymbolNode {
                file_path: row.get(0)?,
                symbol_name: row.get(1)?,
                symbol_kind: row.get(2)?,
                signature: row.get(3)?,
                line_start: row.get(4)?,
                line_end: row.get(5)?,
                doc_comment: row.get(6)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// Find symbols where the name contains `substr` (case-insensitive LIKE query).
    pub fn search_symbols(&self, substr: &str) -> Vec<SymbolNode> {
        let pattern = format!("%{}%", substr);
        let mut stmt = match self.conn.prepare(
            "SELECT file_path, symbol_name, symbol_kind, signature, line_start, line_end, doc_comment
             FROM symbols WHERE symbol_name LIKE ?1 COLLATE NOCASE LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map(params![pattern], |row| {
            Ok(SymbolNode {
                file_path: row.get(0)?,
                symbol_name: row.get(1)?,
                symbol_kind: row.get(2)?,
                signature: row.get(3)?,
                line_start: row.get(4)?,
                line_end: row.get(5)?,
                doc_comment: row.get(6)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// List all symbols (for PageRank computation). Returns file_path + symbol_name pairs.
    pub fn list_all_symbols(&self) -> Vec<(String, String)> {
        let mut stmt = match self.conn.prepare(
            "SELECT file_path, symbol_name FROM symbols"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    fn is_cached(&self, file_path: &str, hash: &str) -> SqlResult<bool> {
        let existing: Option<String> = self.conn.query_row(
            "SELECT file_hash FROM symbol_cache WHERE file_path = ?1",
            params![file_path],
            |row| row.get(0),
        ).ok();
        Ok(existing.as_deref() == Some(hash))
    }

    /// Public wrapper for tests — same as `store_file_symbols` but pub.
    pub fn store_file_symbols_pub(&mut self, file_path: &str, hash: &str, symbols: &[SymbolNode], import_edges: &[(String, String)]) -> SqlResult<()> {
        self.store_file_symbols(file_path, hash, symbols, import_edges)
    }

    fn store_file_symbols(&mut self, file_path: &str, hash: &str, symbols: &[SymbolNode], import_edges: &[(String, String)]) -> SqlResult<()> {
        let tx = self.conn.transaction()?;
        // Remove stale data for this file
        tx.execute("DELETE FROM symbols WHERE file_path = ?1", params![file_path])?;
        tx.execute("DELETE FROM import_edges WHERE from_file = ?1", params![file_path])?;
        tx.execute("DELETE FROM symbol_cache WHERE file_path = ?1", params![file_path])?;

        // Insert new symbols
        for sym in symbols {
            tx.execute(
                "INSERT INTO symbols (file_path, symbol_name, symbol_kind, signature, line_start, line_end, doc_comment)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    sym.file_path, sym.symbol_name, sym.symbol_kind,
                    sym.signature, sym.line_start, sym.line_end, sym.doc_comment
                ],
            )?;
        }

        // Insert import edges for dependency graph
        for (from, to_path) in import_edges {
            tx.execute(
                "INSERT INTO import_edges (from_file, to_path) VALUES (?1, ?2)",
                params![from, to_path],
            )?;
        }

        // Update cache entry
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        tx.execute(
            "INSERT INTO symbol_cache (file_path, file_hash, indexed_at) VALUES (?1, ?2, ?3)",
            params![file_path, hash, now],
        )?;
        tx.commit()
    }

    /// Export the import graph as a Graphviz DOT string.
    /// The caller is responsible for writing to misc/architecture_YYYY-MM-DD.dot
    pub fn export_dot_graph(&self) -> String {
        // Minimal DOT representation from import_edges table
        let mut stmt = match self.conn.prepare("SELECT from_file, to_path FROM import_edges LIMIT 500") {
            Ok(s) => s,
            Err(_) => return "digraph G {}".to_string(),
        };
        let edges: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map(|r| r.flatten().collect()).unwrap_or_default();

        let mut dot = String::from("digraph helix_imports {\n  rankdir=LR;\n");
        for (from, to) in &edges {
            let from_short = from.split('/').next_back().unwrap_or(from);
            let to_short = to.split("::").last().unwrap_or(to);
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", from_short, to_short));
        }
        dot.push_str("}\n");
        dot
    }
}

/// Parse a Rust source file and extract all top-level symbols using Tree-sitter.
pub fn parse_rust_file(source: &str, file_path: &str) -> Vec<SymbolNode> {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Parser, Query, QueryCursor};

    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return vec![];
    }

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return vec![],
    };

    // Combined query for all top-level symbol types
    let query_src = r#"
        (function_item name: (identifier) @name) @fn
        (struct_item name: (type_identifier) @name) @struct
        (enum_item name: (type_identifier) @name) @enum
        (trait_item name: (type_identifier) @name) @trait
        (impl_item) @impl
        (mod_item name: (identifier) @name) @mod
        (use_declaration) @use
    "#;

    let query = match Query::new(&language, query_src) {
        Ok(q) => q,
        Err(_) => return vec![],
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut symbols = Vec::new();

    while let Some(m) = matches.next() {
        let outer_node = m.captures.iter().find(|c| {
            let cn = query.capture_names()[c.index as usize];
            matches!(cn, "fn" | "struct" | "enum" | "trait" | "impl" | "mod" | "use")
        });
        let name_capture = m.captures.iter().find(|c| {
            query.capture_names()[c.index as usize] == "name"
        });

        let outer = match outer_node { Some(c) => c, None => continue };
        let outer_kind = query.capture_names()[outer.index as usize];

        let symbol_kind = match outer_kind {
            "fn" => "fn",
            "struct" => "struct",
            "enum" => "enum",
            "trait" => "trait",
            "impl" => "impl",
            "mod" => "mod",
            "use" => "use",
            _ => continue,
        };

        let node = outer.node;
        let start_line = node.start_position().row as u32 + 1;
        let end_line = node.end_position().row as u32 + 1;

        let symbol_name = if let Some(nc) = name_capture {
            let start = nc.node.start_byte();
            let end = nc.node.end_byte();
            source.get(start..end).unwrap_or("?").to_string()
        } else {
            // For impl blocks: derive name from type
            let raw = source.get(node.start_byte()..node.end_byte().min(node.start_byte() + 60))
                .unwrap_or("impl ?");
            raw.lines().next().unwrap_or("impl ?").trim().to_string()
        };

        // Build skeleton: find body node and elide it
        let signature = build_signature(source, node, symbol_kind);

        symbols.push(SymbolNode {
            file_path: file_path.to_string(),
            symbol_name,
            symbol_kind: symbol_kind.to_string(),
            signature,
            line_start: start_line,
            line_end: end_line,
            doc_comment: None, // TODO: extract doc comments in a follow-up
        });
    }

    symbols
}

/// Extract import edges from `use` declarations in a Rust source file.
///
/// Returns a list of `(from_file, to_path)` pairs where `from_file` is the
/// absolute/normalized path of the current file and `to_path` is the text of
/// the import path (e.g. "std::collections::HashMap", "crate::context::Indexer").
///
/// Uses a dedicated Tree-sitter query targeting `use_declaration` nodes to
/// capture their argument (the scoped identifier path). This is separate from
/// `parse_rust_file` — it focuses only on import relationships for the
/// dependency graph.
pub fn extract_import_edges(source: &str, file_path: &str) -> Vec<(String, String)> {
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Parser, Query, QueryCursor};

    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return vec![];
    }

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return vec![],
    };

    let query = match Query::new(&language, "(use_declaration) @import_path") {
        Ok(q) => q,
        Err(_) => return vec![],
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut edges = Vec::new();
    while let Some(m) = matches.next() {
        if let Some(capture) = m.captures.first() {
            let start = capture.node.start_byte();
            let end = capture.node.end_byte();
            if let Some(path) = source.get(start..end) {
                let clean_path = path.trim_start_matches("use ").trim_end_matches(';').trim();
                edges.push((file_path.to_string(), clean_path.to_string()));
            }
        }
    }
    edges
}

/// Build a skeleton signature by eliding the body block if present.
fn build_signature(source: &str, node: tree_sitter::Node, _kind: &str) -> String {
    // Find child named "body" or "block" to elide
    let body_child = node.children(&mut node.walk()).find(|c| {
        matches!(c.kind(), "block" | "declaration_list" | "enum_variant_list" | "field_declaration_list")
    });

    let full_source = source.get(node.start_byte()..node.end_byte()).unwrap_or("");

    if let Some(body) = body_child {
        let body_rel_start = body.start_byte() - node.start_byte();
        let body_rel_end = body.end_byte() - node.start_byte();
        crate::context::skeleton::elide_body(full_source, body_rel_start, body_rel_end)
    } else {
        full_source.to_string()
    }
}

/// Compute SHA-256 hex digest of a string.
fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

/// Statistics from an indexing run.
#[derive(Debug, Default)]
pub struct IndexStats {
    pub indexed: usize,
    pub skipped: usize,
    pub symbols_found: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory_indexer() -> Indexer {
        let conn = Connection::open_in_memory().unwrap();
        Indexer::new(conn, ".").unwrap()
    }

    #[test]
    fn test_parse_rust_file_finds_functions() {
        let source = r#"
pub fn hello_world(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct Greeter {
    pub name: String,
}
"#;
        let symbols = parse_rust_file(source, "test.rs");
        assert!(symbols.iter().any(|s| s.symbol_name == "hello_world" && s.symbol_kind == "fn"),
            "Must find function hello_world");
        assert!(symbols.iter().any(|s| s.symbol_name == "Greeter" && s.symbol_kind == "struct"),
            "Must find struct Greeter");
    }

    #[test]
    fn test_parse_partial_code_no_panic() {
        // Tree-sitter must tolerate broken/incomplete code
        let broken = "fn foo(x: i32) -> { let y =";
        let symbols = parse_rust_file(broken, "broken.rs");
        // Should not panic — result may be empty or partial
        let _ = symbols; // just verify no panic
    }

    #[test]
    fn test_find_symbol_round_trip() {
        let mut indexer = in_memory_indexer();
        let source = "pub fn my_target_fn(x: u32) -> u32 { x + 1 }";
        let symbols = parse_rust_file(source, "src/test.rs");
        indexer.store_file_symbols("src/test.rs", "abc123", &symbols, &[]).unwrap();
        let found = indexer.find_symbol("my_target_fn");
        assert!(!found.is_empty(), "Must find my_target_fn after storing");
        assert_eq!(found[0].symbol_kind, "fn");
    }

    #[test]
    fn test_incremental_skip_unchanged() {
        let source = "pub fn stable_fn() {}";
        let hash = "fixed_hash_abc";
        let mut indexer = in_memory_indexer();
        let symbols = parse_rust_file(source, "src/stable.rs");
        indexer.store_file_symbols("src/stable.rs", hash, &symbols, &[]).unwrap();
        // Second call with same hash should say "cached"
        assert!(indexer.is_cached("src/stable.rs", hash).unwrap(),
            "Same-hash file must be detected as cached");
        assert!(!indexer.is_cached("src/stable.rs", "different_hash").unwrap(),
            "Different hash must not be cached");
    }

    #[test]
    fn benchmark_symbol_lookup_under_3s() {
        // Validates the <3s SLA from Phase 08 success metrics
        let mut indexer = in_memory_indexer();
        let source = "pub fn target_function(x: u32, y: u32) -> u32 { x + y }";
        let symbols = parse_rust_file(source, "src/benchmark_target.rs");
        indexer.store_file_symbols("src/benchmark_target.rs", "bench_hash", &symbols, &[]).unwrap();

        let start = std::time::Instant::now();
        let found = indexer.find_symbol("target_function");
        let elapsed = start.elapsed();

        assert!(!found.is_empty(), "Symbol must be found");
        assert!(elapsed.as_secs() < 3, "Symbol lookup must complete in <3s, got {:?}", elapsed);
    }

    #[test]
    fn test_extract_import_edges_basic() {
        let source = r#"
use std::collections::HashMap;
use crate::context::Indexer;
use tokio::runtime::Runtime;
"#;
        let edges = extract_import_edges(source, "src/main.rs");
        assert_eq!(edges.len(), 3, "Must extract all 3 use declarations");
        assert!(edges.iter().any(|(f, p)| f == "src/main.rs" && p == "std::collections::HashMap"),
            "Must find std::collections::HashMap import");
        assert!(edges.iter().any(|(f, p)| f == "src/main.rs" && p == "crate::context::Indexer"),
            "Must find crate::context::Indexer import");
        assert!(edges.iter().any(|(f, p)| f == "src/main.rs" && p == "tokio::runtime::Runtime"),
            "Must find tokio::runtime::Runtime import");
    }

    #[test]
    fn test_extract_import_edges_no_imports() {
        let source = "fn no_imports() -> i32 { 42 }";
        let edges = extract_import_edges(source, "src/standalone.rs");
        assert!(edges.is_empty(), "File with no use declarations must produce zero edges");
    }

    #[test]
    fn test_import_edges_stored_and_queried() {
        let source = r#"
use std::sync::Arc;
use crate::tool::ToolRuntime;
"#;
        let conn = Connection::open_in_memory().unwrap();
        let mut indexer = Indexer::new(conn, ".").unwrap();
        let symbols = parse_rust_file(source, "src/test_imports.rs");
        let edges = extract_import_edges(source, "src/test_imports.rs");
        indexer.store_file_symbols("src/test_imports.rs", "hash123", &symbols, &edges).unwrap();

        let dot = indexer.export_dot_graph();
        assert!(dot.contains("digraph helix_imports"), "DOT graph must have header");
        assert!(dot.contains("Arc"), "DOT graph must contain imported symbol 'Arc'");
        assert!(dot.contains("ToolRuntime"), "DOT graph must contain imported symbol 'ToolRuntime'");
        assert!(dot.contains("->"), "DOT graph must contain at least one arrow (edge)");
    }
}
