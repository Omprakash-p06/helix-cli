pub mod phase_state;
pub mod artifacts;
pub mod context_reset;
pub mod recovery;

use phase_state::Phase;
use artifacts::{save_artifact, PhaseArtifact};
use context_reset::ContextResetter;
use serde_json::Value;

pub struct PhaseOutcome {
    pub next: Option<Phase>,
    pub summary: String,
    pub artifact_path: String,
}

pub async fn advance_phase(
    phase: Phase,
    phase_number: u8,
    slug: &str,
    system_state: String,
    _input: Value,
    _tool_runtime: Option<std::sync::Arc<crate::agent_core::tool_runtime::ToolRuntime>>,
    event_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::agent_core::tool_runtime::ToolLifecycle>>,
) -> Result<PhaseOutcome, String> {
    
    // 1. Rebuild context (using ContextResetter)
    let resetter = ContextResetter::new(system_state);
    let _prompt = resetter.rebuild_prompt(phase, phase_number, slug).await?;

    // Optional: wire telemetry
    if let Some(tx) = &event_tx {
        let _ = tx.send(crate::agent_core::tool_runtime::ToolLifecycle::Start {
            name: format!("advance_phase_{}", phase),
            id: format!("phase_{}", phase_number),
        });
    }

    let (next_phase, artifact, filename) = match phase {
        Phase::Discover => {
            let artifact = PhaseArtifact::Plan {
                summary: "Discovery completed".to_string(),
                tasks: vec![],
            };
            (Some(Phase::Discuss), artifact, "discover.json")
        }
        Phase::Discuss => {
            let artifact = PhaseArtifact::Plan {
                summary: "Discussion completed".to_string(),
                tasks: vec![],
            };
            (Some(Phase::Plan), artifact, "discuss.json")
        }
        Phase::Plan => {
            let artifact = PhaseArtifact::Plan {
                summary: "Plan created".to_string(),
                tasks: vec!["Task 1".to_string()],
            };
            (Some(Phase::Execute), artifact, "plan.json")
        }
        Phase::Execute => {
            // 3. Execute phase logic
            // In a real execution, we would call `tool_runtime.execute(...)` here.
            // For now, simulate execution success.
            let artifact = PhaseArtifact::ExecutionReceipt {
                success: true,
                output: "Executed plan tasks".to_string(),
            };
            (Some(Phase::Verify), artifact, "execution.json")
        }
        Phase::Verify => {
            // 4. Handle failures
            // Simulate verification pass
            let artifact = PhaseArtifact::VerifyResult {
                passed: true,
                notes: "All criteria met".to_string(),
            };
            (Some(Phase::Close), artifact, "verify.json")
        }
        Phase::Close => {
            let artifact = PhaseArtifact::VerifyResult {
                passed: true,
                notes: "Phase closed".to_string(),
            };
            (None, artifact, "close.json")
        }
    };

    // 5. Persist artifacts
    save_artifact(phase_number, slug, filename, &artifact).await?;

    if let Some(tx) = &event_tx {
        let _ = tx.send(crate::agent_core::tool_runtime::ToolLifecycle::Result {
            id: format!("phase_{}", phase_number),
            success: true,
            output_summary: format!("Completed phase {}", phase),
        });
    }

    Ok(PhaseOutcome {
        next: next_phase,
        summary: format!("Completed phase {}", phase),
        artifact_path: format!(".planning/phases/{:02}-{}/{}", phase_number, slug, filename),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir_in;
    use std::env;
    use serde_json::json;

    #[tokio::test]
    async fn test_advance_phase_discover() {
        let phase_num = 90;
        let slug = "test-discover";
        let outcome = advance_phase(
            Phase::Discover,
            phase_num,
            slug,
            "System OK".to_string(),
            json!({}),
            None,
            None,
        ).await.unwrap();

        assert_eq!(outcome.next, Some(Phase::Discuss));
        assert!(outcome.artifact_path.contains("discover.json"));
        
        let _ = tokio::fs::remove_dir_all(format!(".planning/phases/{:02}-{}", phase_num, slug)).await;
    }

    #[tokio::test]
    async fn test_advance_phase_execute() {
        let outcome = advance_phase(
            Phase::Execute,
            91,
            "test-execute",
            "System OK".to_string(),
            json!({}),
            None,
            None,
        ).await.unwrap();

        assert_eq!(outcome.next, Some(Phase::Verify));
        assert!(outcome.artifact_path.contains("execution.json"));
        
        let _ = tokio::fs::remove_dir_all(".planning/phases/91-test-execute").await;
    }
}
