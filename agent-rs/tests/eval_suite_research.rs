//! Eval Scenario 4: Research Factuality
//! Verifies that the EvidenceStore correctly persists citations and that
//! EvidenceSynthesizer assembles a ResearchBrief with correct source attribution.

use agent_rs::agent_core::web_research::store::EvidenceStore;
use agent_rs::agent_core::web_research::synthesizer::{EvidenceSynthesizer, Citation};

/// EvidenceStore stores a source and retrieves it in evidence_for_session
#[test]
fn sc4_evidence_store_round_trip() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    let mut store = EvidenceStore::new(conn).unwrap();

    let source_id = store.insert_source(
        "https://docs.rs/reqwest",
        Some("reqwest docs"),
        "abc123",
        200,
    ).unwrap();

    let ev_id = store.insert_evidence(
        "abc123",
        "reqwest is a high-level HTTP client for Rust",
        0.9,
    ).unwrap();

    store.insert_citation(ev_id, source_id, Some("§1")).unwrap();

    let evidence = store.evidence_for_session("test-session");
    assert!(!evidence.is_empty(), "Evidence must be stored and retrievable");
    assert!(evidence[0].markdown_value.contains("reqwest"), "Evidence content must match inserted value");
}

/// EvidenceSynthesizer.compile_brief produces brief with citation
#[test]
fn sc4_synthesizer_includes_citation_in_brief() {
    use agent_rs::agent_core::web_research::store::EvidenceRow;

    let synth = EvidenceSynthesizer::new();
    let evidence = vec![
        EvidenceRow {
            id: 1,
            content_hash: "hash1".to_string(),
            markdown_value: "Gemma 4 E4B supports 128K context".to_string(),
            relevance_score: 0.95,
        }
    ];
    let citations = vec![
        Citation { url: "https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF".to_string(), locator: None }
    ];

    let brief = synth.compile_brief(evidence, citations);
    let ctx = brief.to_context_string();

    assert!(ctx.contains("## Research Brief"), "Brief must have Research Brief header");
    assert!(ctx.contains("Gemma 4"), "Brief must contain the evidence text");
    assert!(ctx.contains("huggingface.co"), "Brief must include citation URL");
    assert!(brief.estimated_tokens() <= 2000, "Brief must stay within 2k token budget");
}

/// Freshness classifier correctly routes stale queries to live search
#[test]
fn sc4_freshness_classifier_routes_stale_query() {
    use rusqlite::Connection;
    use agent_rs::agent_core::web_research::{FreshnessClassifier, EvidenceStore};

    let conn = Connection::open_in_memory().unwrap();
    let store = EvidenceStore::new(conn).unwrap();
    let classifier = FreshnessClassifier::new();

    // "latest" keyword must trigger live search (no cache → needs_live_search = true)
    assert!(classifier.needs_live_search("latest reqwest version", &store));
    // Stable query without cache → default false (use local knowledge)
    assert!(!classifier.needs_live_search("how does tokio mpsc work", &store));
}
