use crate::agent_core::orchestration::artifacts::{load_artifact, PhaseArtifact};
use crate::agent_core::orchestration::phase_state::Phase;

pub struct ContextResetter {
    pub system_state: String,
}

impl ContextResetter {
    pub fn new(system_state: String) -> Self {
        Self { system_state }
    }

    pub async fn rebuild_prompt(
        &self,
        target_phase: Phase,
        phase_number: u8,
        slug: &str,
    ) -> Result<String, String> {
        let mut prompt = format!("System State:\n{}\n\n", self.system_state);
        prompt.push_str(&format!("Entering Phase: {}\n\n", target_phase));

        match target_phase {
            Phase::Discover => {
                prompt.push_str("Task: Discover system information.\n");
            }
            Phase::Discuss => {
                prompt.push_str("Task: Discuss findings.\n");
            }
            Phase::Plan => {
                prompt.push_str("Task: Create an execution plan.\n");
            }
            Phase::Execute => {
                prompt.push_str("Task: Execute the plan.\n");
                // Load plan
                if let Ok(PhaseArtifact::Plan { summary, tasks }) =
                    load_artifact(phase_number, slug, "plan.json").await
                {
                    prompt.push_str("Previous Plan:\n");
                    prompt.push_str(&format!("Summary: {}\n", summary));
                    prompt.push_str(&format!("Tasks: {:?}\n", tasks));
                }
            }
            Phase::Verify => {
                prompt.push_str("Task: Verify the execution.\n");
                if let Ok(PhaseArtifact::ExecutionReceipt { success, output }) =
                    load_artifact(phase_number, slug, "execution.json").await
                {
                    prompt.push_str("Execution Receipt:\n");
                    prompt.push_str(&format!("Success: {}\n", success));
                    prompt.push_str(&format!("Output: {}\n", output));
                }
            }
            Phase::Close => {
                prompt.push_str("Task: Close the phase.\n");
                if let Ok(PhaseArtifact::VerifyResult { passed, notes }) =
                    load_artifact(phase_number, slug, "verify.json").await
                {
                    prompt.push_str("Verify Result:\n");
                    prompt.push_str(&format!("Passed: {}\n", passed));
                    prompt.push_str(&format!("Notes: {}\n", notes));
                }
            }
        }
        
        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_core::orchestration::artifacts::save_artifact;
    #[tokio::test]
    async fn test_rebuild_prompt() {
        let resetter = ContextResetter::new("OS: Linux".to_string());

        let phase_num = 92;
        let slug = "test-rebuild";

        // Test Plan phase
        let prompt = resetter
            .rebuild_prompt(Phase::Plan, phase_num, slug)
            .await
            .unwrap();
        assert!(prompt.contains("OS: Linux"));
        assert!(prompt.contains("Entering Phase: Plan"));
        assert!(prompt.contains("Task: Create an execution plan."));

        // Test Execute phase with Plan artifact
        let plan_artifact = PhaseArtifact::Plan {
            summary: "Test summary".to_string(),
            tasks: vec!["Task A".to_string()],
        };
        save_artifact(phase_num, slug, "plan.json", &plan_artifact)
            .await
            .unwrap();

        let execute_prompt = resetter
            .rebuild_prompt(Phase::Execute, phase_num, slug)
            .await
            .unwrap();
        assert!(execute_prompt.contains("Test summary"));
        assert!(execute_prompt.contains("Task A"));

        let _ = tokio::fs::remove_dir_all(
            crate::agent_core::orchestration::artifacts::get_phase_dir(phase_num, slug),
        )
        .await;
    }
}
