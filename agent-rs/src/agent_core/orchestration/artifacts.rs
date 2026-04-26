use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PhaseArtifact {
    Plan { summary: String, tasks: Vec<String> },
    ExecutionReceipt { success: bool, output: String },
    VerifyResult { passed: bool, notes: String },
}

pub fn get_phase_dir(phase_number: u8, slug: &str) -> PathBuf {
    let mut path = PathBuf::from(".planning");
    path.push("phases");
    path.push(format!("{:02}-{}", phase_number, slug));
    path
}

pub async fn save_artifact(
    phase_number: u8,
    slug: &str,
    filename: &str,
    artifact: &PhaseArtifact,
) -> Result<(), String> {
    let dir = get_phase_dir(phase_number, slug);
    if let Err(e) = fs::create_dir_all(&dir).await {
        return Err(format!("Failed to create directory {:?}: {}", dir, e));
    }
    
    let file_path = dir.join(filename);
    
    let content = match serde_json::to_string_pretty(artifact) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize artifact: {}", e)),
    };
    
    match fs::write(&file_path, content).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write artifact to {:?}: {}", file_path, e)),
    }
}

pub async fn load_artifact(
    phase_number: u8,
    slug: &str,
    filename: &str,
) -> Result<PhaseArtifact, String> {
    let file_path = get_phase_dir(phase_number, slug).join(filename);
    
    let content = match fs::read_to_string(&file_path).await {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to read artifact from {:?}: {}", file_path, e)),
    };
    
    match serde_json::from_str(&content) {
        Ok(a) => Ok(a),
        Err(e) => Err(format!("Failed to deserialize artifact: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_load_artifact() {
        let artifact = PhaseArtifact::Plan {
            summary: "Test plan".to_string(),
            tasks: vec!["Task 1".to_string(), "Task 2".to_string()],
        };

        // Use a unique dummy phase number for this test to avoid conflicts
        let phase_num = 99;
        let slug = "test-slug-save-load";

        let result = save_artifact(phase_num, slug, "plan.json", &artifact).await;
        assert!(result.is_ok(), "Should save artifact successfully");

        let loaded = load_artifact(phase_num, slug, "plan.json").await.unwrap();
        assert_eq!(artifact, loaded);

        // Cleanup
        let _ = fs::remove_dir_all(get_phase_dir(phase_num, slug)).await;
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let phase_num = 98;
        let slug = "missing-test";

        let result = load_artifact(phase_num, slug, "none.json").await;
        assert!(result.is_err(), "Should return error for missing file");
    }
}
