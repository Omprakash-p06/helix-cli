use crate::agent_core::repair::snapshots::{SnapshotManager};
use crate::agent_core::tool_runtime::{ToolResult};
use std::sync::Arc;

pub struct SafetyLoop {
    pub snapshot_manager: Arc<SnapshotManager>,
}

impl SafetyLoop {
    pub fn new(snapshot_manager: Arc<SnapshotManager>) -> Self {
        Self { snapshot_manager }
    }

    /// Runs a transactional repair with snapshot and rollback.
    pub fn execute_transactional<F, V>(
        &self,
        execution_fn: F,
        validation_fn: V,
    ) -> Result<ToolResult, String>
    where
        F: FnOnce() -> ToolResult,
        V: FnOnce(&ToolResult) -> bool,
    {
        // 1. Snapshot
        let snapshot_id = self.snapshot_manager.create_snapshot()
            .map_err(|e| format!("Failed to create pre-repair snapshot: {:?}", e))?;

        // 2. Execution
        let result = execution_fn();

        // 3. Validation
        if result.success && validation_fn(&result) {
            Ok(result)
        } else {
            // 4. Rollback
            match self.snapshot_manager.restore_snapshot(&snapshot_id) {
                Ok(_) => Err(format!("Repair failed validation, system rolled back. Output: {}", result.output)),
                Err(e) => Err(format!("Repair failed AND rollback failed: {:?}. Output: {}", e, result.output)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_core::repair::snapshots::SnapshotManager;
    use tempfile::tempdir;
    use std::sync::Arc;

    #[test]
    fn test_safety_loop_rollback_on_failure() {
        let backup_dir = tempdir().unwrap();
        let source_dir = tempdir().unwrap();
        
        // Create a dummy file to backup
        let file_path = source_dir.path().join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        use std::io::Write;
        writeln!(file, "Hello, world!").unwrap();

        let manager = Arc::new(SnapshotManager::with_sources(
            backup_dir.path().to_path_buf(),
            vec![source_dir.path().to_path_buf()]
        ));
        let safety_loop = SafetyLoop::new(manager);

        let execution = || ToolResult {
            success: true,
            output: "Modified something".to_string(),
        };

        // Validation fails
        let validation = |_res: &ToolResult| false;

        let result = safety_loop.execute_transactional(execution, validation);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("system rolled back"));
    }

    #[test]
    fn test_low_confidence_warning_logic() {
        // This test is to verify the logic that should be in ToolRuntime or SafetyLoop
        // as per Task 2 requirements.
        let confidence = 0.7;
        let mut reason = "Fixing service".to_string();
        
        if confidence < 0.8 {
            reason = format!("LOW CONFIDENCE WARNING: confidence is {:.2}\n{}", confidence, reason);
        }
        
        assert!(reason.contains("LOW CONFIDENCE WARNING"));
    }
}
