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
    use tempfile::tempdir_in;
    use std::env;

    #[tokio::test]
    async fn test_rebuild_prompt() {
        let temp_dir = tempdir_in(env::temp_dir()).unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let resetter = ContextResetter::new("OS: Linux".to_string());
        
        // Test Plan phase
        let prompt = resetter.rebuild_prompt(Phase::Plan, 1, "test").await.unwrap();
        assert!(prompt.contains("OS: Linux"));
        assert!(prompt.contains("Entering Phase: Plan"));
        assert!(prompt.contains("Task: Create an execution plan."));

        // Test Execute phase with Plan artifact
        let plan_artifact = PhaseArtifact::Plan {
            summary: "Test summary".to_string(),
            tasks: vec!["Task A".to_string()],
        };
        save_artifact(1, "test", "plan.json", &plan_artifact).await.unwrap();

        let prompt_exec = resetter.rebuild_prompt(Phase::Execute, 1, "test").await.unwrap();
        assert!(prompt_exec.contains("Test summary"));
        assert!(prompt_exec.contains("Task A"));
    }
}
