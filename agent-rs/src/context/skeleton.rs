//! Skeleton extraction: produces function/type signatures without bodies.
//!
//! This is the "Level 0" distillation level — it emits only the signature
//! of each symbol, reducing token cost from 50-500 tokens (full body)
//! to 10-30 tokens (signature only). Allows ~2000 signatures in 40k tokens.
//!
//! Implementation note: This module operates on raw source text using
//! Tree-sitter output (byte ranges). The indexer provides these ranges;
//! skeleton.rs converts them into formatted strings.

/// Produces a "skeleton" representation of a symbol from its raw source.
///
/// Given the full source of a file and the byte range of a function body,
/// replaces the body with `{ /* ... */ }` while preserving the signature.
///
/// # Arguments
/// * `source` — Full UTF-8 source of the file
/// * `body_start` — Byte offset where the `{` of the body starts
/// * `body_end` — Byte offset where the `}` of the body ends (exclusive)
///
/// # Returns
/// The function source with body replaced by `{ /* ... */ }`.
pub fn elide_body(source: &str, body_start: usize, body_end: usize) -> String {
    if body_start >= source.len() || body_end > source.len() || body_start >= body_end {
        return source.to_string();
    }
    let before = &source[..body_start];
    let after = &source[body_end..];
    format!("{}{{ /* ... */ }}{}", before, after)
}

/// Format a skeleton entry for LLM context injection.
///
/// Produces a compact, readable representation:
/// ```text
/// // src/tools.rs:42-87
/// fn execute_command(cmd: &str, sandbox: &Sandbox) -> Result<Output, ToolError> { /* ... */ }
/// ```
pub fn format_skeleton_entry(file_path: &str, line_start: u32, line_end: u32, signature: &str) -> String {
    format!(
        "// {}:{}-{}\n{}\n",
        file_path, line_start, line_end, signature.trim()
    )
}

/// Format a full repo skeleton header for LLM injection.
///
/// Produces a block like:
/// ```text
/// === CODEBASE SKELETON (N symbols) ===
/// [entries...]
/// =====================================
/// ```
pub fn format_repo_skeleton(entries: &[String], symbol_count: usize) -> String {
    let mut out = format!("=== CODEBASE SKELETON ({} symbols) ===\n", symbol_count);
    for entry in entries {
        out.push_str(entry);
    }
    out.push_str("=====================================\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elide_body_basic() {
        let source = "fn foo() { let x = 1; }";
        // body starts at index 9 ('{'), ends at index 23 ('}' exclusive = 24)
        let body_start = 9;
        let body_end = source.len();
        let result = elide_body(source, body_start, body_end);
        assert!(result.contains("{ /* ... */ }"), "Body must be elided, got: {}", result);
        assert!(result.starts_with("fn foo()"), "Signature must be preserved");
    }

    #[test]
    fn test_elide_body_out_of_range() {
        let source = "fn foo() {}";
        let result = elide_body(source, 1000, 2000);
        assert_eq!(result, source, "Out-of-range input must return original source unchanged");
    }

    #[test]
    fn test_format_skeleton_entry() {
        let entry = format_skeleton_entry("src/tools.rs", 42, 87, "fn execute_command(cmd: &str) -> Result<(), Error>");
        assert!(entry.contains("// src/tools.rs:42-87"), "Must contain file:line range");
        assert!(entry.contains("fn execute_command"), "Must contain signature");
    }
}
