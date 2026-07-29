pub mod classifier;
pub mod store;
pub mod sanitize;
pub mod planner;
pub mod worker;
pub mod synthesizer;

pub use classifier::FreshnessClassifier;
pub use planner::{ResearchPlanner, SearchTask};
pub use store::EvidenceStore;
pub use synthesizer::{Citation, EvidenceSynthesizer, ResearchBrief};
pub use worker::WorkerPool;

use std::sync::Arc;
use tokio::sync::Mutex;

/// Facade orchestrating the full web research pipeline (FreshnessClassifier -> ResearchPlanner -> WorkerPool).
pub struct WebResearchPipeline {
    pub planner: ResearchPlanner,
    pub pool: WorkerPool,
    pub classifier: FreshnessClassifier,
}

impl WebResearchPipeline {
    /// Constructs a new `WebResearchPipeline` with defaults.
    pub fn new() -> Self {
        Self {
            planner: ResearchPlanner::new(5),
            pool: WorkerPool::new(4),
            classifier: FreshnessClassifier::new(),
        }
    }

    /// Runs the research pipeline for a query if classified as requiring live search.
    pub async fn run(
        &self,
        query: &str,
        store: Arc<Mutex<EvidenceStore>>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        {
            let store_guard = store.lock().await;
            if !self.classifier.needs_live_search(query, &store_guard) {
                return Ok(0);
            }
        }

        let tasks = self.planner.plan(query);
        let count = self.pool.run(tasks, store.clone()).await?;

        if count > 0 {
            let mut store_guard = store.lock().await;
            let ttl = self.classifier.select_ttl(query);
            let _ = store_guard.upsert_freshness(query, None, ttl);
        }

        Ok(count)
    }
}

impl Default for WebResearchPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[tokio::test]
    async fn pipeline_respects_ssrf_block() {
        let pipeline = WebResearchPipeline::new();
        let conn = Connection::open_in_memory().unwrap();
        let store = Arc::new(Mutex::new(EvidenceStore::new(conn).unwrap()));

        // Even if live search is triggered, invalid/SSRF URLs return 0 evidence rows stored
        let tasks = vec![SearchTask {
            url: "http://127.0.0.1/test".to_string(),
            query_hint: "test".to_string(),
        }];

        let result = pipeline.pool.run(tasks, store).await.unwrap();
        assert_eq!(result, 0);
    }
}
