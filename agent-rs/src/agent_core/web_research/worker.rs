#![allow(clippy::collapsible_if)]

use crate::agent_core::web_research::planner::SearchTask;
use crate::agent_core::web_research::sanitize::{is_ssrf_safe, sanitize_html_to_markdown};
use crate::agent_core::web_research::store::EvidenceStore;
use governor::{clock::DefaultClock, state::direct::NotKeyed, state::InMemoryState, Quota, RateLimiter};
use nonzero_ext::nonzero;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Worker pool for processing concurrent web fetch tasks.
pub struct WorkerPool {
    pub num_workers: usize,
    pub request_timeout_secs: u64,
    pub max_body_bytes: usize,
}

impl WorkerPool {
    /// Constructs a new `WorkerPool` with defaults (N workers, 30s timeout, 1MB body limit).
    pub fn new(num_workers: usize) -> Self {
        Self {
            num_workers,
            request_timeout_secs: 30,
            max_body_bytes: 1_048_576, // 1MB
        }
    }

    /// Executes tasks concurrently across worker pool tasks.
    pub async fn run(
        &self,
        tasks: Vec<SearchTask>,
        store: Arc<Mutex<EvidenceStore>>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if tasks.is_empty() {
            return Ok(0);
        }

        let quota = Quota::per_second(nonzero!(2u32));
        let rate_limiter: SharedRateLimiter = Arc::new(RateLimiter::direct(quota));

        let (tx, rx) = mpsc::channel::<SearchTask>(32);
        for task in tasks {
            let _ = tx.send(task).await;
        }
        drop(tx);

        let rx = Arc::new(Mutex::new(rx));
        let mut handles = Vec::new();
        let timeout_secs = self.request_timeout_secs;
        let max_bytes = self.max_body_bytes;

        for _ in 0..self.num_workers {
            let rx = Arc::clone(&rx);
            let limiter = Arc::clone(&rate_limiter);
            let store = Arc::clone(&store);

            let handle = tokio::spawn(async move {
                let mut processed_count = 0;
                loop {
                    let task = {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    };

                    let task = match task {
                        Some(t) => t,
                        None => break,
                    };

                    limiter.until_ready().await;

                    if !is_ssrf_safe(&task.url) {
                        eprintln!("[WorkerPool] SSRF blocked: {}", task.url);
                        continue;
                    }

                    let fetch_res = timeout(
                        Duration::from_secs(timeout_secs),
                        reqwest_fetch(&task.url, max_bytes),
                    )
                    .await;

                    match fetch_res {
                        Ok(Ok((html, status_code))) => {
                            let content_source = sanitize_html_to_markdown(&html, &task.url);
                            let content_hash = hex::encode(Sha256::digest(content_source.content.as_bytes()));

                            let mut store_guard = store.lock().await;
                            if let Ok(source_id) = store_guard.insert_source(&task.url, None, &content_hash, status_code) {
                                if let Ok(evidence_id) = store_guard.insert_evidence(&content_hash, &content_source.content, 0.5) {
                                    let _ = store_guard.insert_citation(evidence_id, source_id, None);
                                    processed_count += 1;
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            eprintln!("[WorkerPool] fetch error for {}: {}", task.url, e);
                        }
                        Err(_) => {
                            eprintln!("[WorkerPool] fetch timeout: {}", task.url);
                        }
                    }
                }
                processed_count
            });

            handles.push(handle);
        }

        let mut total = 0;
        for handle in handles {
            if let Ok(count) = handle.await {
                total += count;
            }
        }

        Ok(total)
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new(4)
    }
}

async fn reqwest_fetch(url: &str, max_bytes: usize) -> Result<(String, u16), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;
    let status = response.status().as_u16();

    if let Some(cl) = response.content_length() {
        if cl as usize > max_bytes {
            return Err("Response body exceeds max_body_bytes limit".into());
        }
    }

    let bytes = response.bytes().await?;
    if bytes.len() > max_bytes {
        return Err("Response bytes exceed max_body_bytes limit".into());
    }

    let html = String::from_utf8_lossy(&bytes).into_owned();
    Ok((html, status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[tokio::test]
    async fn worker_pool_blocks_ssrf() {
        let pool = WorkerPool::new(2);
        let conn = Connection::open_in_memory().unwrap();
        let store = Arc::new(Mutex::new(EvidenceStore::new(conn).unwrap()));

        let tasks = vec![SearchTask {
            url: "http://127.0.0.1/secret".to_string(),
            query_hint: "test".to_string(),
        }];

        let result = pool.run(tasks, store).await.unwrap();
        assert_eq!(result, 0);
    }
}
