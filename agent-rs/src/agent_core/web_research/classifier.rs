use crate::agent_core::web_research::store::EvidenceStore;
use regex::Regex;

/// Rule-based freshness classifier for determining if a query requires live web research.
pub struct FreshnessClassifier {
    stale_keywords: Regex,
    error_patterns: Regex,
    cache_ttl_secs: i64,
    dynamic_ttl_secs: i64,
}

impl FreshnessClassifier {
    /// Constructs a new `FreshnessClassifier` with default regex patterns and TTL values.
    pub fn new() -> Self {
        let stale_keywords = Regex::new(
            r"(?i)\b(latest|new release|migration|deprecated|breaking change|rust 2024|MSRV|cve|security advisory|yanked)\b"
        ).expect("valid stale_keywords regex");

        let error_patterns = Regex::new(
            r"(?i)(error\[E\d{4}\]|trait .* not implemented|no method named|cannot find type|unresolved import)"
        ).expect("valid error_patterns regex");

        Self {
            stale_keywords,
            error_patterns,
            cache_ttl_secs: 604800,  // 7 days default
            dynamic_ttl_secs: 86400, // 1 day for volatile queries
        }
    }

    /// Determines if a natural language query requires live web research.
    pub fn needs_live_search(&self, query: &str, store: &EvidenceStore) -> bool {
        // 1. If query contains stale keywords, check if cached result exists and its age
        if self.stale_keywords.is_match(query) {
            if let Some(age) = store.query_age(query) {
                if age > self.dynamic_ttl_secs {
                    return true;
                }
            } else {
                return true;
            }
        }

        // 2. If query matches compiler error patterns (signals API drift)
        if self.error_patterns.is_match(query) {
            return true;
        }

        // 3. Check general cache freshness
        if let Some(age) = store.query_age(query) {
            return age >= self.cache_ttl_secs;
        }

        // Default: no staleness signals and no cache → use local knowledge
        false
    }

    /// Returns the recommended TTL in seconds for a query.
    pub fn select_ttl(&self, query: &str) -> i64 {
        if self.stale_keywords.is_match(query) {
            self.dynamic_ttl_secs
        } else {
            self.cache_ttl_secs
        }
    }
}

impl Default for FreshnessClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_store() -> EvidenceStore {
        let conn = Connection::open_in_memory().unwrap();
        EvidenceStore::new(conn).unwrap()
    }

    #[test]
    fn classifier_routes_stale_keywords_to_live_search() {
        let classifier = FreshnessClassifier::new();
        let store = test_store();

        assert!(classifier.needs_live_search("latest reqwest version", &store));
        assert!(classifier.needs_live_search("deprecated tokio features in breaking change", &store));
    }

    #[test]
    fn classifier_routes_fresh_topic_to_local() {
        let classifier = FreshnessClassifier::new();
        let store = test_store();

        assert!(!classifier.needs_live_search("how does tokio mpsc work", &store));
    }
}
