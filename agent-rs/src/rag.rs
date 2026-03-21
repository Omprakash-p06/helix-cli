use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use ignore::WalkBuilder;
use std::fs;

pub struct DocumentChunk {
    pub file_path: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

pub struct RagStore {
    chunks: Vec<DocumentChunk>,
    model: TextEmbedding,
}

impl RagStore {
    pub fn new(root_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // We use AllMiniLML6V2 as it's extremely fast and lightweight for local CPUs
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true))?;

        let mut store = Self {
            chunks: Vec::new(),
            model,
        };

        // Traverse project directory respecting .gitignore
        let walker = WalkBuilder::new(root_dir)
            .hidden(true)
            .git_ignore(true)
            .build();

        let mut all_texts = Vec::new();
        let mut all_metadata = Vec::new();

        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    let path = entry.path();
                    
                    // Only process likely text files by attempting utf-8 read
                    if let Ok(content) = fs::read_to_string(path) {
                        let chars: Vec<char> = content.chars().collect();
                        let chunk_size = 1200;
                        let overlap = 200;
                        let mut start = 0;
                        
                        while start < chars.len() {
                            let end = std::cmp::min(start + chunk_size, chars.len());
                            let chunk: String = chars[start..end].iter().collect();
                            
                            all_texts.push(chunk.clone());
                            all_metadata.push((path.to_string_lossy().to_string(), chunk));
                            
                            if end == chars.len() {
                                break;
                            }
                            start = end - overlap;
                        }
                    }
                }
            }
        }

        if !all_texts.is_empty() {
            println!("  [RAG] Encoding {} directory chunks into vector space...", all_texts.len());
            // FastEmbed natively batches under the hood for Rust CPU optimization
            let embeddings = store.model.embed(all_texts, None)?;
            for (i, emb) in embeddings.into_iter().enumerate() {
                store.chunks.push(DocumentChunk {
                    file_path: all_metadata[i].0.clone(),
                    content: all_metadata[i].1.clone(),
                    embedding: emb,
                });
            }
        }

        Ok(store)
    }

    pub fn search(&self, query: &str, top_k: usize) -> Result<String, Box<dyn std::error::Error>> {
        let query_emb = self.model.embed(vec![query.to_string()], None)?.pop().unwrap();
        
        // Compute pure Rust Cosine Similarity mapping
        let mut scored_chunks: Vec<(f32, &DocumentChunk)> = self.chunks.iter().map(|chunk| {
            let score = cosine_similarity(&query_emb, &chunk.embedding);
            (score, chunk)
        }).collect();

        // Sort descending by highest semantic match
        scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut results = String::new();
        for (i, (score, chunk)) in scored_chunks.iter().take(top_k).enumerate() {
            results.push_str(&format!("--- Match {} (Score: {:.2}, File: {}) ---\n{}\n\n", i + 1, score, chunk.file_path, chunk.content));
        }

        if results.is_empty() {
            Ok("No semantic codebase context found.".to_string())
        } else {
            Ok(results)
        }
    }
}

// Highly optimized inline dot-product map
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
