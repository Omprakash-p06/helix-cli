use agent_rs::agent_core::orchestration::artifacts::{load_artifact, PhaseArtifact};
use agent_rs::agent_core::orchestration::context_reset::ContextResetter;
use agent_rs::agent_core::orchestration::phase_state::{Phase, PhaseStateMachine};
use agent_rs::agent_core::orchestration::recovery::{RecoveryDecisionMatrix, RecoveryAction};
use agent_rs::agent_core::orchestration::advance_phase;
use agent_rs::tui::commands::{default_commands, execute_command, Command, CommandCategory};
use agent_rs::tui::TuiAction;
use agent_rs::audit::AuditStore;
use serde_json::json;
use std::env;
use tempfile::tempdir_in;

#[tokio::test]
async fn test_full_flow_integration() {
    let temp_dir = tempdir_in(env::temp_dir()).unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let mut state_machine = PhaseStateMachine::new();
    let slug = "test-repair";
    let system_state = "OS: Linux, Error: Permission denied".to_string();

    let mut current_phase = Phase::Discover;
    let phase_number = 1;

    // Simulate Discover -> Discuss -> Plan -> Execute -> Verify -> Close
    while current_phase != Phase::Close {
        let outcome = advance_phase(
            current_phase,
            phase_number,
            slug,
            system_state.clone(),
            json!({}),
            None,
            None,
        )
        .await
        .unwrap();

        // Verify artifact is created
        let filename = outcome.artifact_path.split('/').last().unwrap();
        let artifact = load_artifact(phase_number, slug, filename).await.unwrap();

        match current_phase {
            Phase::Discover => {
                if let PhaseArtifact::Plan { summary, .. } = artifact {
                    assert_eq!(summary, "Discovery completed");
                } else {
                    panic!("Expected Plan artifact for Discover");
                }
            }
            Phase::Plan => {
                if let PhaseArtifact::Plan { tasks, .. } = artifact {
                    assert!(!tasks.is_empty());
                } else {
                    panic!("Expected Plan artifact for Plan");
                }
            }
            Phase::Execute => {
                if let PhaseArtifact::ExecutionReceipt { success, .. } = artifact {
                    assert!(success);
                } else {
                    panic!("Expected ExecutionReceipt for Execute");
                }
            }
            _ => {}
        }

        // Context reset test: before advancing to next phase, ensure context includes previous artifact
        if let Some(next_phase) = outcome.next {
            let resetter = ContextResetter::new(system_state.clone());
            let prompt = resetter.rebuild_prompt(next_phase, phase_number, slug).await.unwrap();
            
            assert!(prompt.contains(&system_state));
            assert!(prompt.contains(&format!("Entering Phase: {}", next_phase)));

            if next_phase == Phase::Execute {
                assert!(prompt.contains("Previous Plan:"));
                assert!(prompt.contains("Task 1"));
            }

            state_machine.transition_to(next_phase).unwrap();
            current_phase = next_phase;
        } else {
            break; // Finished Close
        }
    }
}

#[test]
fn test_recovery_cycle_on_execute_failure() {
    let mut state_machine = PhaseStateMachine::new();
    
    // Move to Execute
    state_machine.transition_to(Phase::Discuss).unwrap();
    state_machine.transition_to(Phase::Plan).unwrap();
    state_machine.transition_to(Phase::Execute).unwrap();

    // Simulate execution failure -> Verify fails -> Need to Retry
    state_machine.transition_to(Phase::Verify).unwrap();
    
    let recovery_matrix = RecoveryDecisionMatrix::default(); // max_retries: 2, max_decompose: 2
    
    // First failure -> Retry
    let action1 = recovery_matrix.decide(0, 0, false);
    assert_eq!(action1, RecoveryAction::Retry);
    state_machine.transition_to(Phase::Execute).unwrap(); // Transition back to Execute
    
    // Simulate another failure -> Verify fails again -> Need to Retry
    state_machine.transition_to(Phase::Verify).unwrap();
    let action2 = recovery_matrix.decide(1, 0, false);
    assert_eq!(action2, RecoveryAction::Retry);
    state_machine.transition_to(Phase::Execute).unwrap(); // Back to Execute
    
    // Third failure -> Verify fails -> No more retries, so Decompose
    state_machine.transition_to(Phase::Verify).unwrap();
    let action3 = recovery_matrix.decide(2, 0, false);
    assert_eq!(action3, RecoveryAction::Decompose);
    state_machine.transition_to(Phase::Plan).unwrap(); // Re-plan (Decompose)
}

#[tokio::test]
async fn test_protocol_validation() {
    let temp_dir = tempdir_in(env::temp_dir()).unwrap();
    let db_path = temp_dir.path().join("audit.db");
    
    let audit_store = AuditStore::new(&db_path).unwrap();
    let slug = "protocol-test";
    
    // Log a transition to AuditStore
    let tx_hash = audit_store.append_event(
        "agent",
        "orchestration",
        "phase_transition",
        "advance_phase",
        Some("allow"),
        Some("success"),
        Some("Transition to Discuss"),
        None,
        &format!("slug={}", slug),
        None,
        Some(150),
    ).unwrap();
    
    assert!(!tx_hash.is_empty());
    
    let events = audit_store.query_events(None, None, Some("orchestration"), None, None, None).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "phase_transition");
    assert_eq!(events[0].reason.as_deref(), Some("Transition to Discuss"));
    
    // Verify context rebuild logic has necessary artifacts
    env::set_current_dir(temp_dir.path()).unwrap();
    
    let artifact = PhaseArtifact::Plan {
        summary: "Plan summary".to_string(),
        tasks: vec!["Task X".to_string()],
    };
    agent_rs::agent_core::orchestration::artifacts::save_artifact(2, slug, "plan.json", &artifact).await.unwrap();
    
    let resetter = ContextResetter::new("State X".to_string());
    let prompt = resetter.rebuild_prompt(Phase::Execute, 2, slug).await.unwrap();
    
    assert!(prompt.contains("Task X"));
    assert!(prompt.contains("Plan summary"));
}

#[test]
fn test_gsd_slash_commands_are_available_in_default_commands() {
    let commands = default_commands();
    assert!(
        commands.iter().any(|c| c.id == "gsd_plan_phase" && c.name == "/gsd-plan-phase"),
        "missing gsd_plan_phase command from default command list"
    );
    assert!(
        commands
            .iter()
            .any(|c| c.id == "gsd_execute_phase" && c.name == "/gsd-execute-phase"),
        "missing gsd_execute_phase command from default command list"
    );
}

#[test]
fn test_gsd_slash_commands_dispatch_contract() {
    let gsd_plan = Command {
        id: "gsd_plan".to_string(),
        name: "/gsd plan".to_string(),
        description: "Plan a new GSD phase".to_string(),
        example: "/gsd plan \"fix network\"".to_string(),
        shortcut: None,
        category: CommandCategory::Mode,
        immediate: false,
    };
    assert!(matches!(
        execute_command(&gsd_plan),
        Some(TuiAction::SystemCommand(cmd)) if cmd == "/gsd plan"
    ));

    let gsd_execute = Command {
        id: "gsd_execute".to_string(),
        name: "/gsd execute".to_string(),
        description: "Execute the active GSD phase".to_string(),
        example: "/gsd execute".to_string(),
        shortcut: None,
        category: CommandCategory::Mode,
        immediate: true,
    };
    assert!(matches!(
        execute_command(&gsd_execute),
        Some(TuiAction::SystemCommand(cmd)) if cmd == "/gsd execute"
    ));
}
