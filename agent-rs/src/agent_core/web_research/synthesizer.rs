use crate::agent_core::web_research::store::{EvidenceRow, EvidenceStore};

pub const MAX_BRIEF_TOKENS: usize = 1_500;
pub const CHARS_PER_TOKEN: usize = 4;
pub const MAX_BRIEF_CHARS: usize = MAX_BRIEF_TOKENS * CHARS_PER_TOKEN; // 6,000 chars

/// A provenance-stamped web source citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub url: String,
    pub locator: Option<String>,
}

/// A condensed research brief containing evidence chunks and citations.
#[derive(Debug, Clone)]
pub struct ResearchBrief {
    pub facts: Vec<EvidenceRow>,
    pub citations: Vec<Citation>,
    pub total_chars: usize,
    pub estimated_tokens: usize,
}

impl ResearchBrief {
    /// Formats the research brief as a context string for inclusion in the agent context.
    pub fn to_context_string(&self) -> String {
        let mut out = String::from("## Research Brief (Web Sources)\n\n");

        for fact in &self.facts {
            out.push_str(&fact.markdown_value);
            out.push_str("\n\n---\n\n");
        }

        out.push_str("## Sources\n");
        for citation in &self.citations {
            out.push_str(&format!("- {}\n", citation.url));
        }

        out
    }

    /// Returns estimated token count of the brief content.
    pub fn estimated_tokens(&self) -> usize {
        self.total_chars / CHARS_PER_TOKEN
    }
}

/// Synthesizes stored evidence into a token-budgeted `ResearchBrief`.
pub struct EvidenceSynthesizer;

impl EvidenceSynthesizer {
    /// Constructs a new `EvidenceSynthesizer`.
    pub fn new() -> Self {
        Self
    }

    /// Compiles evidence rows into a `ResearchBrief` bounded by `MAX_BRIEF_CHARS` (6000 chars / ~1500 tokens).
    pub fn compile_brief(&self, evidence: Vec<EvidenceRow>, citations: Vec<Citation>) -> ResearchBrief {
        let mut sorted = evidence;
        sorted.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        let mut selected = Vec::new();
        let mut total_chars = 0usize;

        for row in sorted {
            let len = row.markdown_value.len();
            if total_chars + len > MAX_BRIEF_CHARS {
                break;
            }
            total_chars += len;
            selected.push(row);
        }

        let estimated_tokens = total_chars / CHARS_PER_TOKEN;

        ResearchBrief {
            facts: selected,
            citations,
            total_chars,
            estimated_tokens,
        }
    }

    /// Fetches session evidence from the `EvidenceStore` and compiles a `ResearchBrief`.
    pub fn brief_from_store(&self, store: &EvidenceStore, session_id: &str) -> ResearchBrief {
        let evidence = store.evidence_for_session(session_id);
        let citations: Vec<Citation> = evidence
            .iter()
            .map(|row| Citation {
                url: format!("hash://{}", row.content_hash),
                locator: None,
            })
            .collect();

        self.compile_brief(evidence, citations)
    }
}

impl Default for EvidenceSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brief_respects_token_budget() {
        let synthesizer = EvidenceSynthesizer::new();
        let mut evidence = Vec::new();

        for i in 0..20 {
            evidence.push(EvidenceRow {
                id: i as i64,
                content_hash: format!("hash_{}", i),
                markdown_value: "a".repeat(400), // 400 chars per chunk
                relevance_score: 0.9,
            });
        }

        let brief = synthesizer.compile_brief(evidence, vec![]);
        assert!(brief.total_chars <= MAX_BRIEF_CHARS);
        assert!(brief.estimated_tokens() <= MAX_BRIEF_TOKENS);
        assert_eq!(brief.facts.len(), 15); // 15 * 400 = 6000
    }

    #[test]
    fn brief_preserves_highest_relevance() {
        let synthesizer = EvidenceSynthesizer::new();
        let evidence = vec![
            EvidenceRow { id: 1, content_hash: "1".to_string(), markdown_value: "low".to_string(), relevance_score: 0.3 },
            EvidenceRow { id: 2, content_hash: "2".to_string(), markdown_value: "high".to_string(), relevance_score: 0.9 },
            EvidenceRow { id: 3, content_hash: "3".to_string(), markdown_value: "mid".to_string(), relevance_score: 0.7 },
        ];

        let brief = synthesizer.compile_brief(evidence, vec![]);
        assert_eq!(brief.facts[0].relevance_score, 0.9);
        assert_eq!(brief.facts[1].relevance_score, 0.7);
        assert_eq!(brief.facts[2].relevance_score, 0.3);
    }
}
