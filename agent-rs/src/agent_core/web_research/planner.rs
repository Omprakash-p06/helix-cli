/// A search task representing a target URL to fetch for web research.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTask {
    pub url: String,
    pub query_hint: String,
}

/// Planner for decomposing a natural language query into target fetch tasks.
pub struct ResearchPlanner {
    max_sources: usize,
}

impl ResearchPlanner {
    /// Constructs a new `ResearchPlanner` with a specified source limit.
    pub fn new(max_sources: usize) -> Self {
        Self { max_sources }
    }

    /// Decomposes a query into a list of search/fetch tasks.
    pub fn plan(&self, query: &str) -> Vec<SearchTask> {
        let mut tasks = Vec::new();
        let query_lower = query.to_lowercase();

        // 1. Crate lookup detection
        let is_crate_query = query_lower.contains("crate")
            || query_lower.contains("cargo")
            || query_lower.contains("rust crate")
            || ["reqwest", "tokio", "serde", "rusqlite", "axum", "ratatui"].iter().any(|c| query_lower.contains(c));

        if is_crate_query {
            // Extract probable crate name
            let crate_name = ["reqwest", "tokio", "serde", "rusqlite", "axum", "ratatui"]
                .iter()
                .find(|c| query_lower.contains(**c))
                .copied()
                .unwrap_or("reqwest");

            tasks.push(SearchTask {
                url: format!("https://docs.rs/{}", crate_name),
                query_hint: query.to_string(),
            });
            tasks.push(SearchTask {
                url: format!("https://crates.io/crates/{}", crate_name),
                query_hint: query.to_string(),
            });
        }

        // 2. DuckDuckGo search fallback
        let encoded_query = query.replace(' ', "+");
        tasks.push(SearchTask {
            url: format!("https://html.duckduckgo.com/html/?q={}+rust", encoded_query),
            query_hint: query.to_string(),
        });

        tasks.truncate(self.max_sources);
        tasks
    }
}

impl Default for ResearchPlanner {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_creates_search_tasks() {
        let planner = ResearchPlanner::new(5);
        let tasks = planner.plan("reqwest async post");
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t.url.contains("docs.rs/reqwest")));
    }

    #[test]
    fn planner_respects_max_sources() {
        let planner = ResearchPlanner::new(2);
        let tasks = planner.plan("reqwest async post");
        assert_eq!(tasks.len(), 2);
    }
}
